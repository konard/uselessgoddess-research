# Финальный отчёт: Распределённая система фарма кейсов CS2

## Оглавление

1. [Введение и архитектура системы](#1-введение-и-архитектура-системы)
2. [Анализ кастомного RPC (shai)](#2-анализ-кастомного-rpc-shai)
3. [KVM/QEMU: GPU без passthrough — VirtIO-GPU Venus](#3-kvmqemu-gpu-без-passthrough--virtio-gpu-venus)
4. [Спуфинг железа: MAC, серийники, SMBIOS](#4-спуфинг-железа-mac-серийники-smbios)
5. [Оптимизация памяти: zswap + KSM](#5-оптимизация-памяти-zswap--ksm)
6. [CS2 в минимальном режиме (384×288)](#6-cs2-в-минимальном-режиме-384288)
7. [Вход в Steam: мульти-аккаунт стратегия](#7-вход-в-steam-мульти-аккаунт-стратегия)
8. [Управление вводом и захват экрана VM](#8-управление-вводом-и-захват-экрана-vm)
9. [Управление XML-конфигами VM](#9-управление-xml-конфигами-vm)
10. [Шаринг CS2 между VM (virtiofs)](#10-шаринг-cs2-между-vm-virtiofs)
11. [Параллельное управление VM и масштабирование](#11-параллельное-управление-vm-и-масштабирование)
12. [Стратегия разработки: снизу вверх (bottom-up)](#12-стратегия-разработки-снизу-вверх-bottom-up)
13. [Обнаружение VM и изоляция отпечатков](#13-обнаружение-vm-и-изоляция-отпечатков)
14. [Итоговые рекомендации и архитектурная схема](#14-итоговые-рекомендации-и-архитектурная-схема)
15. [Список литературы](#15-список-литературы)

---

## 1. Введение и архитектура системы

Цель — построить распределённую систему, где:
- **Центральный сервер (Rust, shai RPC)** управляет очередью аккаунтов и отдаёт команды воркерам
- **Воркер (Rust daemon)** — физический хост с KVM/QEMU, создающий несколько VM с CS2
- **VM** — Linux (Debian 12 minimal + OpenBox), VirtIO-GPU Venus для Vulkan, CS2 native
- **Управление VM** — хост читает буфер дисплея VM через VNC, отправляет мышь/клавиатуру снаружи

### Принятые решения (после начального исследования)

| Вопрос | Решение | Обоснование |
|---|---|---|
| ОС внутри VM | **Linux (Debian 12 + OpenBox)** | RAM в 5-8x меньше, Venus только для Linux guest, нативный CS2 |
| GPU в VM | **VirtIO-GPU Venus** | Единственный Vulkan без passthrough, работает со всеми GPU |
| Память/swap | **zswap + NVMe swap** | Лучше ZRAM для серверов с NVMe (shrinker daemon, fallback на диск) |
| Протокол | **Кастомный RPC (shai)** | Zero-copy (rkyv), QUIC транспорт, низкий overhead |
| Воркер agent | **Rust daemon** | Единый стек с сервером, нет Python зависимостей |
| VM storage | **virtiofs** | Шаринг CS2 между VM без дупликации |

### Высокоуровневая схема

```
┌─────────────────────────────────────────────────────────┐
│                 Центральный сервер (Rust)                │
│  - shai RPC (QUIC) для управления воркерами             │
│  - Очередь аккаунтов (in-memory / Redis)                │
│  - Dashboard                                             │
└─────────────────┬───────────────────────────────────────┘
                  │ shai RPC (QUIC)
      ┌───────────┴─────────────┐
      ▼                         ▼
┌─────────────┐           ┌─────────────┐
│  Воркер 1   │           │  Воркер N   │
│  (Rust)     │           │  (Rust)     │
│ KVM + QEMU  │    ...    │ KVM + QEMU  │
│ libvirt API │           │ libvirt API │
│ ┌────┐┌───┐ │           │ ┌────┐┌───┐ │
│ │VM 1││VM2│ │           │ │VM 1││VM2│ │
│ │CS2 ││CS2│ │           │ │CS2 ││CS2│ │
│ └────┘└───┘ │           │ └────┘└───┘ │
└─────────────┘           └─────────────┘

  CS2 установлена один раз → virtiofs шарит во все VM
```

---

## 2. Анализ кастомного RPC (shai)

### 2.1 Обзор архитектуры

**shai** — это легковесный zero-copy RPC фреймворк на Rust. Основные компоненты:

| Компонент | Файл | Назначение |
|-----------|------|------------|
| Message Protocol | `rpc/mod.rs`, `rpc/frame.rs` | Определение сообщений, ID-схема, wire format |
| Сериализация | `rpc/mod.rs` | Интеграция rkyv, zero-copy traits |
| Router | `router/mod.rs` | Таблица маршрутов, tower::Service |
| Handlers | `router/context.rs` | Handler trait, 0-3 extractors |
| Extractors | `router/extract.rs` | State, Archive, Rpc, Unchecked, Extension, Peer |
| QUIC Transport | `transport/quic/` | quinn wrapper, мультиплексирование потоков |
| Local Transport | `local.rs` | mpsc-каналы для тестирования |
| Codec | `transport/codec.rs` | 24-byte header, little-endian |
| Extensions | `util/extensions.rs` | Типизированное per-peer хранилище |
| Macros | `shai-macros/` | `#[shai::message]`, `rpc!()` |

### 2.2 Wire format

Каждый фрейм имеет 24-байтный заголовок:

```
[0..2]:  MessageId (u16, little-endian)
[2]:     Flags (u8)
[3]:     Status (u8) — Ok, NotFound, DecodeError, EncodeError, InternalError, Unauthorized
[4..8]:  Payload Length (u32, little-endian)
[8..24]: Trace ID (16 bytes) — для distributed tracing
[24+]:   Payload (rkyv-сериализованные данные)
```

### 2.3 Ключевые паттерны

**Extractor pattern (вдохновлён axum):**
```rust
// Handler принимает типизированные extractors
async fn handle_farm_task(
    State(state): State<AppState>,     // Состояние приложения
    Archive(task): Archive<FarmTask>,   // Zero-copy десериализация
    peer: Peer,                         // Информация о пире
) -> rpc::Result<FarmTaskResponse> {
    // ...
}
```

**Регистрация маршрутов:**
```rust
shai::rpc! {
    1: FarmTask       => FarmTaskResponse,
    2: WorkerStatus   => WorkerStatusResponse,
    3: AccountRequest => AccountResponse,
}

let router = Router::new(state)
    .route::<FarmTask, _, _>(handle_farm_task)
    .route::<WorkerStatus, _, _>(handle_status)
    .route::<AccountRequest, _, _>(handle_account);
```

**Per-peer extensions (идеальны для auth):**
```rust
// При подключении — сохранить WorkerId
peer.insert_extension(WorkerId(42));

// В handler'е — извлечь
async fn handle(Extension(id): Extension<WorkerId>) -> Result<...> {
    // id доступен без дополнительных запросов
}
```

### 2.4 Преимущества shai для данного проекта

| Свойство | Значение для проекта |
|---|---|
| **Zero-copy (rkyv)** | Минимальный overhead при передаче статусов VM, фреймов дисплея |
| **QUIC транспорт** | Мультиплексирование потоков, встроенное шифрование (TLS 1.3) |
| **Trace ID** | Трассировка команд от сервера через воркер до конкретной VM |
| **Per-peer extensions** | Авторизация воркеров, хранение WorkerId/VmId в контексте |
| **tower::Service** | Composable middleware — rate limiting, metrics, auth |
| **Bidirectional streams** | Каждый запрос-ответ — отдельный QUIC stream, отмена на уровне соединения |

### 2.5 Применение shai в системе

**Сервер → Воркер (команды):**
```rust
#[shai::message]
pub struct StartAccount {
    pub account_id: u64,
    pub steam_login: String,
    pub vm_config: VmConfig,
}

#[shai::message]
pub struct StopAccount {
    pub vm_id: u32,
}

#[shai::message]
pub struct WorkerHeartbeat {
    pub active_vms: u32,
    pub cpu_usage: f32,
    pub memory_free_mb: u32,
}
```

**Воркер → Сервер (статусы):**
```rust
#[shai::message]
pub struct AccountStatus {
    pub account_id: u64,
    pub state: AccountState,  // Logging, InGame, Farming, Error
    pub xp_earned: u32,
}
```

### 2.6 Бенчмарки (из `examples/quic-load.rs`)

Тест настроен на 100,000 concurrent streams. Результаты показывают:
- SmallReq: высокие RPS при минимальной задержке
- LargeReq (300 KB payload): измеряет MB/s throughput
- QUIC настройка: 256MB send window, 64MB receive window

---

## 3. KVM/QEMU: GPU без passthrough — VirtIO-GPU Venus

**Venus — единственный рабочий путь для Vulkan без passthrough.** Это подтверждённый фаворит.

### Сравнение подходов

| Подход | Vulkan | Для CS2 | Примечание |
|---|---|---|---|
| VirtIO-GPU virgl | Нет (OpenGL только) | ❌ | Недостаточно для CS2 |
| **VirtIO-GPU Venus** | **Да (Vulkan 1.3)** | **✅** | **QEMU 9.2+, kernel 6.13+** |
| QXL | Нет (2D только) | ❌ | Только RDP/VNC |
| llvmpipe / lavapipe | Да (CPU) | ⚠️ (<15 FPS) | Только как fallback |
| GPU Passthrough (VFIO) | Да | ✅✅ | Один GPU на VM |

### Актуальные требования Venus (по данным docs.mesa3d.org)

**Хост:**
- QEMU ≥ 9.2.0 (рекомендуется 11.0+)
- Linux kernel ≥ 6.13 (рекомендуется 6.16+ для Intel/AMD dGPU)
- Mesa ≥ 24.2
- Протестированные хост-драйверы:
  - **ANV** (Intel) 21.1+ — kernel 6.16+ для Meteor Lake
  - **RADV** (AMD) 21.1+ — kernel 6.13+ (нужен патч KVM PFNMAP)
  - **NVIDIA (Proprietary)** 570.86+ — работает!
  - Turnip, PanVK, Lavapipe — тоже поддерживаются

**Гость:**
- Linux kernel ≥ 5.16 (virtio-gpu driver с VIRTGPU_PARAM_*)
- Mesa с Venus driver

### QEMU commandline

```
-device virtio-gpu-gl,hostmem=4G,blob=true,venus=true \
-vga none \
-display vnc=127.0.0.1:0 \
-object memory-backend-memfd,id=mem1,size=4G \
-machine memory-backend=mem1
```

> **Важно:** Для Intel + AMD dGPU и Intel + NVIDIA комбинаций нужен `-accel kvm,honor-guest-pat=on` (QEMU 11.0+).

---

## 4. Спуфинг железа: MAC, серийники, SMBIOS

Каждая VM должна выглядеть как уникальный физический компьютер. Все настройки — в XML libvirt, генерируются воркером программно.

### 4.1 Полный XML-шаблон спуфинга

```xml
<domain type='kvm' xmlns:qemu='http://libvirt.org/schemas/domain/qemu/1.0'>
  <uuid>{generated_uuid}</uuid>

  <sysinfo type='smbios'>
    <bios>
      <entry name='vendor'>American Megatrends International, LLC.</entry>
      <entry name='version'>F.70</entry>
      <entry name='date'>11/02/2021</entry>
    </bios>
    <system>
      <entry name='manufacturer'>{random: ASUS|MSI|Gigabyte}</entry>
      <entry name='product'>{random_product}</entry>
      <entry name='serial'>{random_serial}</entry>
      <entry name='uuid'>{same_uuid}</entry>
    </system>
    <baseBoard>
      <entry name='manufacturer'>{same_manufacturer}</entry>
      <entry name='product'>{same_product}</entry>
      <entry name='serial'>{random_serial_2}</entry>
    </baseBoard>
  </sysinfo>

  <os>
    <type arch='x86_64' machine='q35'>hvm</type>
    <smbios mode='sysinfo'/>
  </os>

  <features>
    <hyperv mode='custom'>
      <vendor_id state='on' value='GenuineIntel'/>
    </hyperv>
    <kvm><hidden state='on'/></kvm>
    <vmport state='off'/>
  </features>

  <cpu mode='host-passthrough' check='none'>
    <topology sockets='1' dies='1' cores='2' threads='2'/>
    <feature policy='disable' name='hypervisor'/>
  </cpu>

  <devices>
    <interface type='network'>
      <mac address='{random_real_oui_mac}'/>
      <source network='worker-net'/>
      <model type='e1000e'/>
    </interface>

    <disk type='file' device='disk'>
      <source file='{vm_overlay_path}'/>
      <target dev='sda' bus='sata'/>
      <serial>{random_disk_serial}</serial>
    </disk>

    <memballoon model='none'/>
  </devices>

  <qemu:commandline>
    <qemu:arg value='-cpu'/>
    <qemu:arg value='host,model_id={random_cpu_string},hypervisor=off'/>
  </qemu:commandline>
</domain>
```

### 4.2 Генерация воркером (Rust)

Воркер генерирует уникальные значения для каждой VM программно:

```rust
// Пример структуры для генерации VM identity
pub struct VmIdentity {
    pub uuid: Uuid,
    pub mac: MacAddress,         // Реальный OUI (Intel, Realtek и тп)
    pub disk_serial: String,     // WD-WXE... формат, до 20 символов
    pub smbios: SmbiosProfile,   // Случайная материнская плата
    pub cpu_brand: String,       // Intel Core i7-10700K и тп
}
```

### 4.3 Что нельзя спрятать программно

| Вектор | Статус |
|---|---|
| RDTSC timing (~1000–5000 cycles в KVM) | Патч ядра (kvm-rdtsc-hack) |
| WMI thermal/fan sensors (пусты в VM) | Нет решения |
| ACPI table строки (BOCHS/BXPC) | Патч QEMU (qemu-anti-detection) |

Для VAC (серверный античит) базового спуфинга SMBIOS + CPUID hide достаточно.

---

## 5. Оптимизация памяти: zswap + KSM

### 5.1 Почему zswap, а не ZRAM

| Характеристика | ZRAM | zswap + NVMe swap |
|---|---|---|
| Латентность | Микросекунды (RAM) | Микросекунды (RAM cache) |
| При переполнении | **Зависание 20-30 мин / OOM** | **Вытесняет в swap — продолжает работать** |
| Shrinker daemon | Нет | **Да — умная вытесняция холодных страниц** |
| Для серверов с NVMe | Не рекомендуется | **Рекомендуется** |

**Вывод:** zswap — кеш-слой перед swap на NVMe. Горячие страницы в RAM (сжаты), холодные — на NVMe. Это безопаснее ZRAM: при нехватке памяти система продолжает работать, а не зависает.

### 5.2 Настройка zswap на хосте

Kernel boot parameters:
```
zswap.enabled=1 zswap.shrinker_enabled=1 zswap.compressor=lz4 zswap.max_pool_percent=30
```

Sysctl для хоста с VM:
```ini
vm.swappiness = 150      # Linux 5.8+: >100 = предпочитать swap/zswap
vm.vfs_cache_pressure = 500
vm.dirty_background_ratio = 1
```

### 5.3 KSM — дедупликация между VM

KSM находит одинаковые страницы памяти (CS2 бинарники, Mesa, ядро) и объединяет их CoW:

- 8 идентичных CS2 VM × 1.5 GB → ~0.9 GB уникальной RAM на VM (30-50% экономия)
- **Важно:** KSM несовместим с Huge Pages

Активация:
```ini
# /etc/sysctl.d/ksm.conf
kernel.mm.ksm.run = 1
kernel.mm.ksm.pages_to_scan = 1000
kernel.mm.ksm.sleep_millisecs = 50
```

### 5.4 Стратегия памяти (32 GB хост)

```
Слой 1: KSM
  → 30-50% экономии на одинаковых CS2 VM
  → Бесплатно (только CPU время)

Слой 2: zswap + NVMe swap (30% RAM = ~10 GB compressed pool)
  → Горячие страницы в RAM, холодные на NVMe
  → Shrinker автоматически вытесняет холодные данные

Бюджет: 8 VM × 1.5 GB effective + 3 GB host = ~15 GB
На 32 GB хосте: комфортный запас
```

---

## 6. CS2 в минимальном режиме (384×288)

### 6.1 Параметры запуска

```
-window -w 384 -h 288 -noborder -novid -nojoy -nohltv \
-softparticlesdefaultoff -forcenovsync \
+fps_max 20 +fps_max_menu 5 \
+mat_queue_mode 2 +r_dynamic 0 +mat_disable_fancy_blending 1
```

### 6.2 Реальные требования CS2 по RAM

| Режим | RAM |
|---|---|
| CS2 процесс при idle/загрузке | **800 MB – 1.2 GB RSS** |
| CS2 во время активного матча | **1.5–2.0 GB RSS** |

**500 MB нереально.** Минимальная цель — **1.5–2 GB на VM**, с KSM эффективно ~900 MB.

### 6.3 Система дропов CS2

- **Prime Status обязателен** для дропов
- Weekly Care Package: 5000 XP = уровень = Care Package (2 предмета из 4)
- Сброс по средам 00:00 UTC
- ~30 минут активной игры в неделю достаточно для дропа
- AFK — минимальный/нулевой XP

---

## 7. Вход в Steam: мульти-аккаунт стратегия

### 7.1 Проблема

Каждая VM запускает разные аккаунты. Нужен способ автоматически входить в конкретный Steam аккаунт при старте VM.

### 7.2 Сравнение подходов

| Метод | Автоматизация | Надёжность | Примечание |
|---|---|---|---|
| `-login user pass` (Steam CLI) | ❌ **Сломан с 2023** | — | Больше не работает |
| Предзаполненная сессия (loginusers.vdf) | ✅ Высокая | ✅ Высокая | **Рекомендуется** |
| SteamCMD + cached credentials | ✅ Средняя | ✅ Средняя | Для скачивания CS2 |
| VNC автоматизация логина | ⚠️ Хрупкая | ⚠️ Ломается при обновлениях UI | Только как fallback |

### 7.3 Рекомендуемый подход: предзаполненная сессия

**Принцип:** Один раз залогиниться с "Remember Me", сохранить файлы сессии как шаблон. При создании VM — подставить нужные файлы.

**Файлы Steam сессии (Linux):**
```
~/.steam/steam/config/
├── config.vdf          # Основной конфиг (содержит токены)
├── loginusers.vdf      # Список залогиненных пользователей
├── DialogConfig.vdf    # UI настройки
└── htmlcache/          # Web кеш
```

**Ключевой файл — `loginusers.vdf`:**
```vdf
"users"
{
    "7656119XXXXXXXXXX"
    {
        "AccountName"    "steam_user_123"
        "PersonaName"    "DisplayName"
        "RememberPassword"    "1"
        "MostRecent"    "1"
        "Timestamp"    "1711800000"
    }
}
```

### 7.4 Workflow входа в аккаунт

```
1. Сервер отправляет воркеру команду StartAccount {
     account_id, steam_session_files
   }

2. Воркер:
   a) Создаёт VM из base overlay (qcow2 thin clone)
   b) Через virtiofs монтирует CS2
   c) Записывает session files в VM overlay
   d) Запускает VM

3. Внутри VM (автозагрузка):
   a) Steam запускается с -silent
   b) Автологин через сохранённую сессию
   c) steam://rungameid/730 — запуск CS2

4. Воркер мониторит через VNC:
   a) Проверяет успешный вход
   b) При Steam Guard prompt → отправляет код через RPC от сервера
   c) Принимает матч / управляет игрой
```

### 7.5 Обновление сессий

Steam токены имеют ограниченный срок жизни. Стратегия обновления:

1. **Batch provisioning:** Отдельный процесс раз в неделю обновляет сессии для всех аккаунтов
2. **Credential store на сервере:** Encrypted хранилище session files, привязанных к account_id
3. **SteamCMD для начальной авторизации:** `steamcmd +login user pass +quit` сохраняет cached credentials

### 7.6 Steam Guard

При первом входе с новой "машины" (VM) Steam Guard запросит код. Варианты:
- **Email code:** Сервер мониторит почту, передаёт код воркеру через RPC
- **Mobile authenticator (TOTP):** Генерация кода из shared_secret (через Rust crate `steam-totp` или аналог)
- **Отключить Steam Guard:** Не рекомендуется (снижает доверие аккаунта)

---

## 8. Управление вводом и захват экрана VM

### 8.1 Архитектура

```
┌─────────── Хост ─────────────────┐
│                                   │
│  ┌─────────────┐                 │
│  │   Воркер    │──────┐          │
│  │   (Rust)    │      │          │
│  └─────────────┘      │          │
│         ▲             ▼          │
│    read frame   send commands    │
│         │             │          │
│    ┌────┴─────────────┴───────┐  │
│    │     VNC client (Rust)    │  │
│    └────────────────┬─────────┘  │
└─────────────────────┼────────────┘
                       │ VNC
┌─────────── VM ───────┼────────────┐
│                       │           │
│    ┌──────────────────┴───────┐   │
│    │    QEMU VNC server       │   │
│    └──────────────────────────┘   │
│              CS2 window           │
└───────────────────────────────────┘
```

### 8.2 VNC (рекомендуется)

libvirt XML:
```xml
<graphics type='vnc' port='-1' autoport='yes' listen='127.0.0.1'>
  <listen type='address' address='127.0.0.1'/>
</graphics>
```

Для управления из Rust — использовать VNC client библиотеку (например `vnc-rs` crate) или обёртку над libvnc. Воркер напрямую:
- Читает framebuffer через VNC
- Отправляет mouse/keyboard events
- При необходимости — запускает inference модель для навигации

### 8.3 QEMU QMP (для скриншотов)

```xml
<qemu:commandline>
  <qemu:arg value='-qmp'/>
  <qemu:arg value='unix:/tmp/vm{id}-qmp.sock,server=on,wait=off'/>
</qemu:commandline>
```

Воркер подключается к QMP socket и выполняет:
- `screendump` — одиночный скриншот
- `input-send-event` — ввод мыши/клавиатуры

### 8.4 Сравнение методов

| Метод | Скриншот | Ввод | Непрерывный поток | Требует GPU |
|---|---|---|---|---|
| **VNC** | ✅ | ✅ | **✅** | Нет |
| QEMU QMP | ✅ | ✅ | Нет (snapshots) | Нет |
| libvirt API | ✅ | Нет | Нет | Нет |

**Рекомендация:** VNC для непрерывного управления, QMP как дополнение для разовых операций.

---

## 9. Управление XML-конфигами VM

### 9.1 Сравнение: библиотека vs самостоятельно

| Подход | Плюсы | Минусы |
|---|---|---|
| **`virt` crate (libvirt Rust bindings)** | Полный API libvirt, создание/управление VM программно, проверенный в production | Зависимость от libvirt-dev, сложный API |
| **Terraform + libvirt provider** | Декларативный IaC, хорошо для статического флота | Overkill для динамического управления из Rust daemon |
| **Генерация XML + virsh** | Простой, нет зависимостей | Хрупкий, нет проверки ошибок на этапе компиляции |
| **Прямая генерация XML (Rust)** | Полный контроль, нет внешних зависимостей | Нужно самому валидировать XML |

### 9.2 Рекомендация: `virt` crate + шаблоны

Для динамического управления VM из Rust воркера лучший подход — **`virt` crate** (Rust bindings для libvirt):

```rust
use virt::connect::Connect;
use virt::domain::Domain;

// Подключение к libvirt
let conn = Connect::open(Some("qemu:///system"))?;

// Создание VM из XML
let xml = generate_vm_xml(&vm_identity, &vm_config);
let domain = Domain::define_xml(&conn, &xml)?;
domain.create()?;  // Запуск VM

// Управление
domain.shutdown()?;
domain.destroy()?;  // Force stop
domain.undefine()?; // Удаление
```

**Преимущества:**
- Нет вызовов shell (virsh) — всё через C API
- Rust error handling
- Прямой доступ к domain info, memory stats, CPU info
- Модули: `domain`, `network`, `storage_pool`, `storage_vol`

**Требования:**
- На хосте: `apt install libvirt-dev`
- В Cargo.toml: `virt = "0.4"`

### 9.3 XML-генерация

Конфигурация VM не самая сложная — её можно генерировать через Rust `format!()` или шаблонизатор (`askama`, `tera`). Ключевые параметры:
- UUID, MAC, SMBIOS — генерируются уникально
- CPU, RAM — задаются воркером на основе доступных ресурсов
- Диск — qcow2 overlay от base image
- virtiofs — монтирование CS2
- VNC — управление

---

## 10. Шаринг CS2 между VM (virtiofs)

### 10.1 Проблема

CS2 занимает ~35 GB. При 8 VM — это 280 GB без шаринга. Нужен способ шарить один экземпляр CS2.

### 10.2 Сравнение подходов

| Метод | Производительность | Read-only | Простота | Примечание |
|---|---|---|---|---|
| **virtiofs** | **Нативная FS** | R/W | **Простая** | **Рекомендуется** |
| 9p (virtio-9p) | Медленная | R/W | Простая | Устаревший, плохая производительность |
| NFS внутри VM | Сетевая | R/W | Средняя | Лишний overhead |
| qcow2 backing store | Дисковая | R (через CoW) | Простая | CS2 в base image — сложнее обновлять |

### 10.3 virtiofs — рекомендуемое решение

**Принцип:** Хост делит директорию `/opt/cs2-shared/` во все VM. Одна VM скачивает CS2 → все остальные видят обновлённые файлы.

**libvirt XML:**
```xml
<memoryBacking>
  <source type='memfd'/>
  <access mode='shared'/>
</memoryBacking>

<devices>
  <filesystem type='mount' accessmode='passthrough'>
    <driver type='virtiofs' queue='1024'/>
    <source dir='/opt/cs2-shared'/>
    <target dir='cs2'/>
  </filesystem>
</devices>
```

**В гостевой VM:**
```bash
mount -t virtiofs cs2 /opt/cs2
```

### 10.4 Workflow скачивания CS2

```
1. Воркер при первом запуске:
   - Запускает одну "bootstrap" VM
   - Через SteamCMD скачивает CS2 в /opt/cs2-shared/
   - Останавливает bootstrap VM

2. Все последующие VM:
   - Монтируют /opt/cs2 через virtiofs (read-only для безопасности)
   - Каждая VM имеет свою ~/.steam/ (в overlay, не shared)
   - CS2 запускается из shared location

3. Обновление CS2:
   - Один из VM запускает steamcmd +app_update 730
   - Все остальные VM автоматически видят обновление
   - Перезапуск CS2 в каждой VM
```

### 10.5 Альтернатива: qcow2 backing store

Если virtiofs по какой-то причине не подходит:

```
# Base image с предустановленной CS2
/var/lib/libvirt/images/base-cs2.qcow2  (~40 GB)

# Thin clone для каждой VM (~1 MB, растёт при записи)
qemu-img create -f qcow2 -F qcow2 \
  -b /var/lib/libvirt/images/base-cs2.qcow2 \
  /var/lib/libvirt/images/vm-01.qcow2
```

**Минус:** Обновление CS2 требует пересоздания base image.

### 10.6 Вывод

**virtiofs** — оптимальное решение:
- Одна копия CS2 на диске
- Все VM видят обновления моментально
- Нативная производительность файловой системы
- virtiofsd daemon написан на Rust (`virtiofsd-rs`)
- Поддержка DAX (Direct Access) для ещё лучшей производительности

---

## 11. Параллельное управление VM и масштабирование

### 11.1 Быстрое клонирование (qcow2 overlay)

```
# Base image (один раз)
virt-sysprep -d worker-base

# Thin clone (~1 MB, 1 секунда)
qemu-img create -f qcow2 -F qcow2 \
  -b /var/lib/libvirt/images/base.qcow2 \
  /var/lib/libvirt/images/worker-06.qcow2
```

10 VM с одним 4 GB base = ~4 GB суммарно вместо 40 GB.

### 11.2 Ограничения ресурсов

CPU pinning и квоты в XML:
```xml
<cputune>
  <vcpupin vcpu='0' cpuset='2'/>
  <vcpupin vcpu='1' cpuset='3'/>
  <period>100000</period>
  <quota>150000</quota>
</cputune>
```

### 11.3 Сколько VM на хост?

| Хост | VM комфортно | VM макс. (с KSM) |
|---|---|---|
| 16 GB RAM / 4 core | 4–5 | 6–7 |
| 32 GB RAM / 8 core | 8–10 | **12–16** |
| 64 GB RAM / 16 core | 18–22 | 28–32 |

---

## 12. Стратегия разработки: снизу вверх (bottom-up)

### 12.1 Сравнение подходов

| Подход | Плюсы | Минусы |
|---|---|---|
| **Сверху вниз** (сервер → воркер → VM) | Архитектура продумана заранее | Долго до первого видимого результата, нечем тестировать |
| **Снизу вверх** (VM → воркер → сервер) | **Быстрая проверка концепции**, видимый прогресс | Возможен рефакторинг при интеграции |

### 12.2 Рекомендация: снизу вверх

**Начинать с VM** — это позволяет:
1. Сразу проверить концепцию (CS2 на Venus работает?)
2. Измерить реальное потребление памяти
3. Убедиться что VNC управление работает
4. Протестировать virtiofs для шаринга CS2
5. Отладить спуфинг железа

### 12.3 Дорожная карта

```
Фаза 1: VM + CS2 (proof of concept)
├── [ ] Настроить QEMU с Venus на одном хосте
├── [ ] Установить CS2, запустить на 384x288
├── [ ] Проверить VNC управление
├── [ ] Настроить virtiofs для шаринга CS2
├── [ ] Измерить RAM/CPU при игре
└── [ ] Проверить спуфинг (dmidecode, lshw внутри VM)

Фаза 2: Воркер (локальное управление)
├── [ ] Rust daemon с libvirt API (virt crate)
├── [ ] Генерация XML с рандомным fingerprint
├── [ ] Создание/запуск/остановка VM
├── [ ] VNC контроллер: чтение экрана, ввод
├── [ ] Steam автологин через session files
└── [ ] Параллельное управление N VM

Фаза 3: Сервер + RPC (распределённое управление)
├── [ ] shai RPC: определение message types
├── [ ] Сервер: очередь аккаунтов, assignment воркерам
├── [ ] Воркер: подключение к серверу, получение задач
├── [ ] Heartbeat / status reporting
├── [ ] Steam Guard code relay
└── [ ] Dashboard / мониторинг

Фаза 4: Масштабирование
├── [ ] Multi-worker deployment
├── [ ] Install script для новых хостов
├── [ ] Оптимизация: KSM, zswap tunning
├── [ ] AI inference для навигации
└── [ ] Поведенческая рандомизация
```

### 12.4 Почему не сверху вниз?

Сервер и RPC уже есть (shai). Нет смысла полировать его до того, как убедимся, что VM-часть вообще работает. Основные риски — на уровне VM:
- Venus может не работать с конкретным GPU
- CS2 может потреблять больше памяти чем ожидается
- VNC latency может быть слишком высокой для управления
- virtiofs может иметь проблемы с CS2 (anti-cheat, file locks)

**Сначала снимаем риски → потом строим инфраструктуру.**

---

## 13. Обнаружение VM и изоляция отпечатков

### 13.1 Митигация

| Вектор | Сложность | Решение |
|---|---|---|
| CPUID hypervisor bit | ✅ Легко | `<feature policy='disable' name='hypervisor'/>` |
| KVM CPUID leaf | ✅ Легко | `<kvm><hidden state='on'/></kvm>` |
| HyperV vendor_id | ✅ Легко | `<vendor_id state='on' value='GenuineIntel'/>` |
| MAC (QEMU OUI) | ✅ Легко | Реальный vendor OUI |
| NIC model (virtio = VEN_1AF4) | ✅ Легко | `e1000e` |
| Disk serial | ✅ Легко | Кастомный `<serial>` |
| SMBIOS strings | ✅ Легко | `<sysinfo>` блок |
| Memory balloon | ✅ Легко | `<memballoon model='none'/>` |
| RDTSC timing | ❗ Сложно | kvm-rdtsc-hack |
| ACPI tables | ❗ Сложно | qemu-anti-detection |

### 13.2 Сетевая изоляция

| Вариант | Качество |
|---|---|
| Все VM через один IP (NAT) | Плохая |
| WireGuard VPN per VM | Хорошая |
| Residential proxy per VM | Хорошая |
| Отдельные IP от провайдера | Отличная |

### 13.3 Поведенческая рандомизация

- Рандомизировать длину сессий (±20%)
- Рандомизировать время запуска
- Jitter в движениях мыши
- Разные карты/режимы
- Человеческие паузы

---

## 14. Итоговые рекомендации и архитектурная схема

### Компонентный стек

| Компонент | Выбор | Обоснование |
|---|---|---|
| Host OS | Ubuntu 22.04+ / Arch | Стабильная KVM/libvirt поддержка |
| VM OS | **Debian 12 minimal + OpenBox** | ~200 MB idle RAM |
| VM GPU | **VirtIO-GPU Venus** | Единственный Vulkan без passthrough |
| VM memory | **zswap + NVMe swap + KSM** | Безопаснее ZRAM, 30-50% экономия через KSM |
| CS2 storage | **virtiofs** | Одна копия CS2 на все VM |
| VM диск | **qcow2 thin clone** | ~1 сек создание |
| RPC | **shai (кастомный Rust)** | Zero-copy, QUIC, per-peer extensions |
| VM управление | **`virt` crate** | Rust bindings для libvirt |
| VNC контроль | **Rust VNC client** | Чтение экрана + ввод |
| Steam login | **Предзаполненная сессия** | loginusers.vdf + token files |
| Спуфинг | **SMBIOS + CPUID hide + e1000e** | Достаточно для VAC |
| Мониторинг | Prometheus + libvirt-exporter | Стандартный стек |

### Полная архитектура

```
┌──────────────────────────────────────────────────────────────┐
│                   Центральный сервер (Rust)                    │
│                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │ shai RPC     │  │    Redis     │  │ Account Credential │  │
│  │ (QUIC)       │  │ (job queue)  │  │ Store (encrypted)  │  │
│  └──────────────┘  └──────────────┘  └────────────────────┘  │
└──────────────────────────┬───────────────────────────────────┘
                            │ shai RPC (QUIC, TLS 1.3)
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
         ┌──────────┐  ┌──────────┐  ┌──────────┐
         │ Воркер 1 │  │ Воркер 2 │  │ Воркер N │
         │ (Rust)   │  │ (Rust)   │  │ (Rust)   │
         │          │  │          │  │          │
         │ libvirt  │  │ libvirt  │  │ libvirt  │
         │ virt     │  │ virt     │  │ virt     │
         │ crate    │  │ crate    │  │ crate    │
         │          │  │          │  │          │
         │ virtiofs │  │ virtiofs │  │ virtiofs │
         │ /opt/cs2 │  │ /opt/cs2 │  │ /opt/cs2 │
         │          │  │          │  │          │
         │ ┌──┐┌──┐ │  │ ┌──┐┌──┐ │  │ ┌──┐┌──┐ │
         │ │v1││v2│ │  │ │v1││v2│ │  │ │v1││v2│ │
         │ │CS│││CS│ │  │ │CS│││CS│ │  │ │CS│││CS│ │
         │ └──┘└──┘ │  │ └──┘└──┘ │  │ └──┘└──┘ │
         └──────────┘  └──────────┘  └──────────┘
```

### Пропускная способность

Один хост (32 GB RAM, 8 core):
- **8–10 VM** комфортно (2 GB RAM / 2 vCPU)
- **12–16 VM** с KSM
- ~30 мин активной игры в неделю на аккаунт

При N хостах: N × 8–16 параллельных аккаунтов.

---

## 15. Список литературы

### RPC и сериализация
- rkyv (zero-copy deserialization): https://rkyv.org/
- quinn (QUIC for Rust): https://github.com/quinn-rs/quinn
- tower (Service trait): https://github.com/tower-rs/tower

### Venus и GPU
- VirtIO-GPU Venus (Mesa docs): https://docs.mesa3d.org/drivers/venus.html
- virglrenderer: https://gitlab.freedesktop.org/virgl/virglrenderer
- Collabora: State of GFX virtualization (2025): https://www.collabora.com/news-and-blog/blog/2025/01/15/the-state-of-gfx-virtualization-using-virglrenderer/

### libvirt и VM управление
- virt crate (Rust bindings): https://crates.io/crates/virt
- libvirt Domain XML: https://libvirt.org/formatdomain.html
- virtiofs с libvirt: https://libvirt.org/kbase/virtiofs.html
- virtiofs official: https://virtio-fs.gitlab.io/
- virtiofsd-rs (Rust daemon): https://gitlab.com/virtio-fs/virtiofsd

### Оптимизация памяти
- zswap (ArchWiki): https://wiki.archlinux.org/title/Zswap
- Chris Down: zswap vs zram: https://chrisdown.name/2026/03/24/zswap-vs-zram-when-to-use-what.html
- KSM (KVM): https://www.linux-kvm.org/page/KSM

### CS2 и Steam
- CS2 System Requirements: https://store.steampowered.com/app/730/CounterStrike_2/
- CS2 Launch Options: https://totalcsgo.com/launch-options
- Steam сессия: loginusers.vdf, config.vdf

### Спуфинг и anti-detection
- qemu-anti-detection: https://github.com/zhaodice/qemu-anti-detection
- kvm-rdtsc-hack: https://github.com/h33p/kvm-rdtsc-hack
- Anti-cheat VM detection: https://secret.club/2020/04/13/how-anti-cheats-detect-system-emulation.html
