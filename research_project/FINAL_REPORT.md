# Финальный отчёт: Распределённая система фарма кейсов CS2

## Оглавление

1. [Введение и архитектура системы](#1-введение-и-архитектура-системы)
2. [Выбор ОС для VM: Windows 10 LTSC vs Linux + OpenBox](#2-выбор-ос-для-vm-windows-10-ltsc-vs-linux--openbox)
3. [KVM/QEMU: GPU без passthrough](#3-kvmqemu-gpu-без-passthrough)
4. [Спуфинг железа: MAC, серийники, SMBIOS](#4-спуфинг-железа-mac-серийники-smbios)
5. [Оптимизация памяти: ZRAM, KSM, ballooning](#5-оптимизация-памяти-zram-ksm-ballooning)
6. [CS2 в минимальном режиме (384×288)](#6-cs2-в-минимальном-режиме-384288)
7. [Управление вводом и захват экрана VM](#7-управление-вводом-и-захват-экрана-vm)
8. [Архитектура воркера: Custom ISO vs Install Script](#8-архитектура-воркера-custom-iso-vs-install-script)
9. [Параллельное управление VM и масштабирование](#9-параллельное-управление-vm-и-масштабирование)
10. [Коммуникация сервер-воркер](#10-коммуникация-сервер-воркер)
11. [Обнаружение VM и изоляция отпечатков](#11-обнаружение-vm-и-изоляция-отпечатков)
12. [Итоговые рекомендации и архитектурная схема](#12-итоговые-рекомендации-и-архитектурная-схема)
13. [Список литературы](#13-список-литературы)

---

## 1. Введение и архитектура системы

Цель — построить распределённую систему, где:
- **Центральный сервер** управляет очередью аккаунтов и отдаёт команды воркерам
- **Воркер** — физический хост с KVM/QEMU, создающий несколько VM с CS2
- **VM** — изолированная среда с поддельным железом, запускающая Steam + CS2
- **Управление VM** — хост читает буфер дисплея VM, отправляет команды мышь/клавиатура снаружи

### Высокоуровневая схема

```
┌─────────────────────────────────────────────────────┐
│                 Центральный сервер                   │
│  - REST API для управления                           │
│  - MQTT broker (Mosquitto)                           │
│  - Redis: очередь аккаунтов, статусы воркеров        │
│  - Web dashboard (Grafana)                           │
└─────────────────┬───────────────────────────────────┘
                  │ MQTT / Redis pub/sub
      ┌───────────┴─────────────┐
      ▼                         ▼
┌─────────────┐           ┌─────────────┐
│  Воркер 1   │           │  Воркер N   │
│  (хост)     │           │  (хост)     │
│ Ubuntu 22+  │           │ Ubuntu 22+  │
│ KVM + QEMU  │    ...    │ KVM + QEMU  │
│ ┌────┐┌───┐ │           │ ┌────┐┌───┐ │
│ │VM 1││VM2│ │           │ │VM 1││VM2│ │
│ │CS2 ││CS2│ │           │ │CS2 ││CS2│ │
│ └────┘└───┘ │           │ └────┘└───┘ │
└─────────────┘           └─────────────┘
```

---

## 2. Выбор ОС для VM: Windows 10 LTSC vs Linux + OpenBox

### Сравнительная таблица

| Параметр | Windows 10 LTSC | Linux + OpenBox |
|---|---|---|
| CS2 native поддержка | Да (DirectX 11) | **Да (Vulkan, first-party)** |
| VAC совместимость | Полная | **Полная (официальная поддержка Valve)** |
| Idle RAM | 900 MB – 1.5 GB | **150–350 MB** |
| Доступно для CS2 (8 GB хост) | ~6.5–7 GB | **~7.6–7.8 GB** |
| Холодный запуск (SSD) | 20–40 сек | **5–15 сек** |
| Vulkan в VM (без passthrough) | Недоступен (Venus — только Linux guest) | **Venus layer (QEMU 9.2+)** |
| NVIDIA Error 43 в VM | Да (требует спуфинга) | **Нет** |
| Стоимость лицензии | Требуется | **Бесплатно** |
| Размер установки | ~20–30 GB (stripped) | **~3–8 GB** |

### Вывод: **Linux + OpenBox**

Linux выигрывает по всем ключевым параметрам:
- Потребление RAM в 5–8 раз меньше → больше VM на одном хосте
- Единственный путь для Vulkan без passthrough (Venus layer)
- CS2 имеет нативную Linux-сборку с рабочим VAC
- Нет проблем с NVIDIA Error 43
- Быстрее загружается → быстрее масштабирование

**Рекомендуемый стек для guest VM:**
- OS: Debian 12 или Ubuntu 22.04 (минимальная установка, без DE)
- WM: OpenBox или i3 (RAM ~150–250 MB idle)
- Display: X11 (Xvfb или VNC output)
- GPU driver: Mesa 24.2+ с поддержкой Venus (для Vulkan)

---

## 3. KVM/QEMU: GPU без passthrough

CS2 на Linux **требует Vulkan** — OpenGL (virgl) недостаточен.

### Сравнение подходов к GPU в VM

| Подход | Vulkan | Windows guest | Для CS2 | Примечание |
|---|---|---|---|---|
| VirtIO-GPU virgl | Нет (OpenGL только) | Ограничено | ❌ | Недостаточно для CS2 |
| VirtIO-GPU Venus | **Да (Vulkan 1.3)** | Нет | **✅ (Linux guest)** | QEMU 9.2+, kernel 6.13+ |
| QXL | Нет (2D только) | Да | ❌ | Только удалённый рабочий стол |
| llvmpipe / lavapipe | Да (CPU) | Нет | ⚠️ (<15 FPS) | Запасной вариант |
| GPU Passthrough (VFIO) | Да | Да | ✅✅ | Лучшая производительность |

### Вывод

**Без passthrough единственный рабочий путь — Venus (VirtIO-GPU Vulkan):**

```xml
<devices>
  <video>
    <model type='virtio'/>
  </video>
  <graphics type='vnc' port='-1' autoport='yes' listen='127.0.0.1'/>
</devices>
```

QEMU commandline для Venus:
```bash
-device virtio-vga-gl,blob=on,hostmem=4G,venus=on \
-vga none \
-display vnc=127.0.0.1:0 \
-object memory-backend-memfd,id=mem1,size=4G \
-machine memory-backend=mem1
```

**Требования на хосте:**
- QEMU ≥ 9.2.0
- Linux kernel ≥ 6.13
- Mesa ≥ 24.2
- Host GPU с поддержкой Vulkan (NVIDIA, AMD GFX9+, Intel Gen12+)

**Читать буфер дисплея с хоста:** VNC client или QEMU QMP `screendump` — оба работают без passthrough.

---

## 4. Спуфинг железа: MAC, серийники, SMBIOS

Каждая VM должна выглядеть как уникальный физический компьютер. Все настройки делаются в XML libvirt.

### 4.1 MAC-адрес

```xml
<interface type='network'>
  <mac address='00:1A:A0:3F:B2:77'/>   <!-- Real Intel OUI, не 52:54:00 (QEMU) -->
  <source network='worker-net'/>
  <model type='e1000e'/>               <!-- e1000e, NOT virtio-net -->
</interface>
```

Генерация случайного MAC с реальным OUI:
```bash
printf '00:1A:A0:%02x:%02x:%02x\n' $((RANDOM%256)) $((RANDOM%256)) $((RANDOM%256))
```

### 4.2 Серийный номер диска

```xml
<disk type='file' device='disk'>
  <source file='/path/to/vm.qcow2'/>
  <target dev='sda' bus='sata'/>
  <serial>WD-WXE1A80K3KJH</serial>    <!-- max 20 символов, под реальный диск -->
</disk>
```

### 4.3 SMBIOS (полный спуфинг)

```xml
<domain type='kvm' xmlns:qemu='http://libvirt.org/schemas/domain/qemu/1.0'>
  <uuid>a3b4c5d6-e7f8-1234-abcd-1234567890ab</uuid>

  <sysinfo type='smbios'>
    <bios>
      <entry name='vendor'>American Megatrends International, LLC.</entry>
      <entry name='version'>F.70</entry>
      <entry name='date'>11/02/2021</entry>
    </bios>
    <system>
      <entry name='manufacturer'>ASUS</entry>
      <entry name='product'>ROG STRIX B550-F GAMING</entry>
      <entry name='serial'>M80ABCDEF01234</entry>
      <entry name='uuid'>a3b4c5d6-e7f8-1234-abcd-1234567890ab</entry>
    </system>
    <baseBoard>
      <entry name='manufacturer'>ASUSTeK COMPUTER INC.</entry>
      <entry name='product'>ROG STRIX B550-F GAMING</entry>
      <entry name='serial'>M80ABCDEF01234</entry>
    </baseBoard>
    <chassis>
      <entry name='serial'>Default string</entry>
    </chassis>
  </sysinfo>

  <os>
    <type arch='x86_64' machine='q35'>hvm</type>
    <smbios mode='sysinfo'/>    <!-- ОБЯЗАТЕЛЬНО для активации sysinfo -->
  </os>

  <features>
    <hyperv mode='custom'>
      <vendor_id state='on' value='GenuineIntel'/>   <!-- 12 символов точно -->
    </hyperv>
    <kvm>
      <hidden state='on'/>     <!-- Скрывает KVM CPUID leaf -->
    </kvm>
    <vmport state='off'/>
  </features>

  <cpu mode='host-passthrough' check='none'>
    <topology sockets='1' dies='1' cores='4' threads='2'/>
    <feature policy='disable' name='hypervisor'/>
  </cpu>

  <devices>
    <memballoon model='none'/>  <!-- Убрать balloon = убрать один сигнал VM -->
  </devices>

  <!-- CPU brand string -->
  <qemu:commandline>
    <qemu:arg value='-cpu'/>
    <qemu:arg value='host,model_id=Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz,hypervisor=off'/>
  </qemu:commandline>
</domain>
```

### 4.4 Что нельзя спрятать программно

| Вектор | Статус |
|---|---|
| RDTSC timing overhead (~1000–5000 cycles в KVM) | Требует патча ядра (kvm-rdtsc-hack) |
| WMI thermal/fan sensors (пусты в VM) | Нет решения |
| ACPI table строки (BOCHS/BXPC) | Патч исходников QEMU (qemu-anti-detection) |

---

## 5. Оптимизация памяти: ZRAM, KSM, ballooning

### 5.1 ZRAM vs Swap — сравнение

| Характеристика | ZRAM | Swap на диске | zswap |
|---|---|---|---|
| Латентность | **Микросекунды (RAM)** | Миллисекунды (HDD) / ~100µs (NVMe) | Микросекунды |
| Нужен диск | Нет | Да | Да (backing store) |
| Умная вытесняция | Нет | Нет | **Да (shrinker daemon)** |
| При переполнении | Зависание 20-30 мин / OOM | Продолжает работать | Вытесняет в swap |
| Рекомендуется для серверов | Только без диска | NVMe + большой своп | **Да** |

**Вывод по ZRAM vs zswap:** Для сервера с NVMe — **zswap + небольшой swap partition** предпочтительнее ZRAM. ZRAM хорош только когда нет диска. Важно: zswap — это кеш перед swap, а не замена.

Если нет диска — ZRAM с zstd:
```bash
# /etc/systemd/zram-generator.conf
[zram0]
zram-size = min(ram, 8192)
compression-algorithm = zstd
swap-priority = 100
```

Оптимальные sysctl:
```ini
vm.swappiness = 150      # Linux 5.8+: >100 = сильно предпочитать zram
vm.vfs_cache_pressure = 500
vm.dirty_background_ratio = 1
```

### 5.2 KSM — дедупликация страниц между VM

KSM ищет одинаковые страницы памяти в разных VM и объединяет их (Copy-on-Write).

**Экономия для одинаковых CS2 VM:**
- Одна и та же бинарная сборка CS2 → ~30–50% дедупликация
- 8 VM × 1.5 GB → эффективно ~0.9 GB уникальной RAM на VM

```bash
echo 1    | sudo tee /sys/kernel/mm/ksm/run
echo 1000 | sudo tee /sys/kernel/mm/ksm/pages_to_scan
echo 50   | sudo tee /sys/kernel/mm/ksm/sleep_millisecs

# Проверить экономию
saved=$(( ($(cat /sys/kernel/mm/ksm/pages_sharing) - $(cat /sys/kernel/mm/ksm/pages_shared)) * 4 / 1024 ))
echo "KSM saves: ${saved} MB"
```

**Важно:** KSM несовместим с Huge Pages.

### 5.3 Memory Ballooning

Динамически отбирает RAM у VM обратно на хост, если VM её не использует.

```xml
<memory unit='MiB'>2048</memory>         <!-- Максимум -->
<currentMemory unit='MiB'>1024</currentMemory>  <!-- Стартовое -->
<devices>
  <memballoon model='virtio'><stats period='5'/></memballoon>
</devices>
```

**Примечание:** Для anti-detection — убрать balloon (`model='none'`). Баланс: balloon помогает с памятью, но его наличие сигнализирует о VM.

### 5.4 Реальные требования CS2 по RAM

| Режим | RAM |
|---|---|
| Официальный минимум (клиент) | 8 GB |
| CS2 dedicated server (10–12 игроков) | 4 GB рекомендуется |
| CS2 процесс при idle/загрузке | **800 MB – 1.2 GB RSS** |
| CS2 во время активного матча (10 игроков) | **1.5–2.0 GB RSS** |

**500 MB — НЕРЕАЛЬНО** для CS2. Минимальная практическая цель — **1.5–2 GB на VM**, с KSM эффективно ~900 MB после конвергенции.

### 5.5 Рекомендованная стратегия памяти

Для хоста 32 GB RAM, 8+ VM:

```
Слой 1: KSM (включить первым)
  → 30-50% экономии на одинаковых CS2 VM
  → Бесплатно (только CPU время)

Слой 2: Memory Ballooning
  → Provision 8 × 2 GB max, start 1 GB current
  → VMs растут по потребности

Слой 3: Host zswap + NVMe swap
  → Защитная сетка для пиковой нагрузки

Бюджет: 8 VM × 1.5 GB effective + 3 GB host = ~15 GB
На 32 GB хосте: комфортный запас
```

---

## 6. CS2 в минимальном режиме (384×288)

### 6.1 Запуск на 384×288

CS2 **поддерживает произвольное разрешение** через параметры запуска:

```
-window -w 384 -h 288 -noborder
```

### 6.2 Полные минимальные параметры

```
-window -w 384 -h 288 -noborder -novid -nojoy -nohltv \
-softparticlesdefaultoff -forcenovsync \
+fps_max 20 +fps_max_menu 5 \
+mat_queue_mode 2 +r_dynamic 0 +mat_disable_fancy_blending 1
```

| Параметр | Работает | Эффект |
|---|---|---|
| `-window -w 384 -h 288` | ✅ | Минимальный размер окна |
| `-noborder` | ✅ | Без рамки (borderless) |
| `-novid` | ✅ | Пропустить intro видео |
| `-nojoy` | ✅ | Отключить геймпад |
| `-nohltv` | ✅ | Отключить GOTV relay |
| `-softparticlesdefaultoff` | ✅ | Отключить depth particles |
| `-forcenovsync` | ✅ | Отключить VSync |
| `+fps_max 20` | ✅ | Ограничить FPS |
| `+r_dynamic 0` | ✅ | Отключить динамическое освещение |
| `-threads N` | ⚠️ | Не рекомендуется, Source 2 сам управляет |
| `-noreactlogin` | ❌ Dead | Удалён из Steam |
| `-no-browser` | ❌ Dead | Удалён из Steam |

### 6.3 Система дропов CS2 (актуально 2026)

**Система Weekly Care Package** требует активной игры:
- Нужно зарабатывать XP через официальный matchmaking
- Каждые 5000 XP = повышение уровня = Care Package (2 предмета из 4)
- Сброс еженедельно по средам 00:00 UTC
- **Prime Status обязателен** для получения дропов
- AFK игроки зарабатывают минимальный/нулевой XP
- ~30 минут активной игры в неделю достаточно для одного дропа

### 6.4 Запуск Steam без UI

```
steam -silent -nofriendsui -nochatui
```

**Авто-логин:** `-login username password` **сломан** с mid-2023. Используйте сохранённую сессию (логин с "Remember Me" один раз).

### 6.5 Запуск Steam headless (без монитора)

```bash
Xvfb :99 -screen 0 1024x768x24 &
export DISPLAY=:99
steam -silent -nofriendsui &
sleep 15
steam steam://rungameid/730 -- -window -w 384 -h 288 -noborder -novid +fps_max 20
```

**CS2 НЕ работает без GPU** — нужен реальный Vulkan-совместимый GPU или Venus (QEMU 9.2+).

---

## 7. Управление вводом и захват экрана VM

### 7.1 Архитектура управления

```
┌─────────── Хост ─────────────────┐
│                                   │
│  ┌─────────────┐                 │
│  │ Controller  │──────┐          │
│  │  (Python)   │      │          │
│  └─────────────┘      │          │
│         ▲             ▼          │
│    read frame   send commands    │
│         │             │          │
│    ┌────┴─────────────┴───────┐  │
│    │        VNC client        │  │
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

### 7.2 VNC-подход (рекомендуется)

В libvirt XML добавить VNC:
```xml
<graphics type='vnc' port='-1' autoport='yes' listen='127.0.0.1'>
  <listen type='address' address='127.0.0.1'/>
</graphics>
```

Python контроллер (vncdotool):
```python
from vncdotool import api
import cv2

class VMController:
    def __init__(self, host='127.0.0.1', port=5900):
        self.client = api.connect(host, port=port)

    def get_frame(self):
        self.client.captureScreen('/tmp/frame.png')
        return cv2.imread('/tmp/frame.png')

    def click(self, x, y):
        self.client.mouseMove(x, y)
        self.client.mousePress(1)
        self.client.mouseRelease(1)

    def key_press(self, key):
        self.client.keyPress(key)

    def find_button(self, template_path, threshold=0.8):
        """Найти кнопку по шаблону и вернуть (x, y)"""
        frame = self.get_frame()
        template = cv2.imread(template_path)
        result = cv2.matchTemplate(frame, template, cv2.TM_CCOEFF_NORMED)
        _, max_val, _, max_loc = cv2.minMaxLoc(result)
        if max_val > threshold:
            h, w = template.shape[:2]
            return (max_loc[0] + w//2, max_loc[1] + h//2)
        return None
```

### 7.3 QEMU QMP (альтернатива)

Добавить в QEMU командную строку:
```xml
<qemu:commandline>
  <qemu:arg value='-qmp'/>
  <qemu:arg value='unix:/tmp/vm1-qmp.sock,server=on,wait=off'/>
</qemu:commandline>
```

Скриншот через QMP:
```python
import socket, json

def qmp_command(sock_path, cmd, args=None):
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.connect(sock_path)
        s.recv(4096)  # capabilities
        s.send(json.dumps({"execute": "qmp_capabilities"}).encode())
        s.recv(4096)
        msg = {"execute": cmd}
        if args: msg["arguments"] = args
        s.send(json.dumps(msg).encode())
        return json.loads(s.recv(4096))

# Сделать скриншот
qmp_command('/tmp/vm1-qmp.sock', 'screendump', {"filename": "/tmp/frame.ppm"})

# Нажать кнопку мыши
qmp_command('/tmp/vm1-qmp.sock', 'input-send-event', {
    "events": [{"type": "btn", "data": {"down": True, "button": "left"}}]
})
```

### 7.4 Сравнение методов

| Метод | Скриншот | Ввод | Непрерывный поток | Требует GPU |
|---|---|---|---|---|
| VNC (vncdotool) | ✅ | ✅ | ✅ | Нет |
| QEMU QMP | ✅ | ✅ | Нет (only snapshots) | Нет |
| libvirt API | ✅ | Нет | Нет | Нет |
| Looking Glass | ✅✅ (zero-latency) | ✅ | ✅ | **PASSTHROUGH REQUIRED** |

**Рекомендация:** VNC + vncdotool — наиболее гибкий подход. QMP — хорошо для разовых скриншотов.

### 7.5 AI inference pipeline

```python
# Основной цикл управления VM
controller = VMController('127.0.0.1', port=5900)

while True:
    # 1. Захватить кадр
    frame = controller.get_frame()

    # 2. Запустить инференс (YOLO или шаблонный поиск)
    detections = model.predict(frame)

    # 3. Принять решение
    action = decision_logic(detections)

    # 4. Выполнить действие
    if action.type == 'click':
        controller.click(action.x, action.y)
    elif action.type == 'key':
        controller.key_press(action.key)

    time.sleep(0.1)  # ~10 FPS принятие решений
```

---

## 8. Архитектура воркера: Custom ISO vs Install Script

### Сравнение

| Критерий | Custom ISO (Packer) | Cloud-Init + Ansible |
|---|---|---|
| Начальная сложность | Высокая (дни) | **Низкая (часы)** |
| Скорость обновлений | Пересборка + redistribution (медленно) | **Push playbook на живой флот (минуты)** |
| Работа без интернета | Да | Нужен APT mirror |
| Дрейф конфигурации | Нет (immutable) | Возможен (митигируется cron-запусками) |
| CI/CD интеграция | Умеренная (Packer) | **Простая (ansible-playbook)** |
| Масштаб деплоя | **Лучше для 100+ стабильных нод** | Лучше для активной разработки |

### Вывод: Гибридный подход

1. **Packer строит "golden base image"** с OS + общими зависимостями
2. **Cloud-init обрабатывает первый boot** (hostname, credentials, server URL)
3. **Ansible — day-2 операции** (обновления ПО, изменения конфигов)

### Cloud-init user-data для воркера

```yaml
#cloud-config
hostname: worker-01
users:
  - name: worker
    groups: [sudo, kvm, libvirt]
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys: [ssh-ed25519 AAAA...]

package_update: true
packages:
  - qemu-kvm
  - libvirt-daemon-system
  - libvirt-clients
  - python3
  - python3-pip
  - curl
  - xvfb

runcmd:
  - usermod -aG kvm,libvirt worker
  - pip3 install libvirt-python paho-mqtt vncdotool opencv-python
  - curl -fsSL https://server.example.com/install-agent.sh | bash
  - systemctl enable --now worker-agent
```

### Install script (минималистичный)

```bash
#!/bin/bash
set -e

# Зависимости
apt-get update && apt-get install -y \
    qemu-kvm libvirt-daemon-system libvirt-clients \
    python3 python3-pip xvfb

# Python пакеты
pip3 install libvirt-python paho-mqtt vncdotool opencv-python

# Настройка
cat > /etc/worker-agent/config.json << EOF
{
  "server": "${SERVER_URL}",
  "worker_id": "${WORKER_ID:-$(hostname)}",
  "max_vms": ${MAX_VMS:-4}
}
EOF

systemctl enable --now worker-agent
```

---

## 9. Параллельное управление VM и масштабирование

### 9.1 Terraform + libvirt (Infrastructure as Code)

```hcl
resource "libvirt_domain" "cs2_workers" {
  count  = var.worker_count    # Изменить одно число = масштабировать
  name   = "cs2-worker-${count.index + 1}"
  memory = 2048
  vcpu   = 2
  cloudinit = libvirt_cloudinit_disk.init[count.index].id
  disk { volume_id = libvirt_volume.worker[count.index].id }
}
```

### 9.2 Быстрое клонирование (qcow2 backing store)

```bash
# 1. Подготовить базовый образ (один раз)
virt-sysprep -d worker-base   # Сбросить machine-id, SSH keys

# 2. Создать thin clone (занимает ~1 секунду, использует ~1 MB)
qemu-img create -f qcow2 -F qcow2 \
  -b /var/lib/libvirt/images/base.qcow2 \
  /var/lib/libvirt/images/worker-06.qcow2

# 3. Клонировать определение VM
virt-clone --original worker-base --name worker-06 --auto-clone
```

10 VM с одним 4 GB base = ~4 GB суммарно вместо 40 GB.

### 9.3 Ограничения ресурсов на VM

CPU pinning и квоты:
```xml
<cputune>
  <vcpupin vcpu='0' cpuset='2'/>
  <vcpupin vcpu='1' cpuset='3'/>
  <period>100000</period>
  <quota>150000</quota>   <!-- 150% = 1.5 ядра -->
</cputune>
```

Disk I/O throttle:
```xml
<iotune>
  <total_iops_sec>300</total_iops_sec>
  <read_bytes_sec>31457280</read_bytes_sec>   <!-- 30 MB/s -->
  <write_bytes_sec>15728640</write_bytes_sec>  <!-- 15 MB/s -->
</iotune>
```

### 9.4 Мониторинг и авто-перезапуск

Prometheus + libvirt-exporter:
```bash
docker run -d \
  -v /var/run/libvirt/libvirt-sock:/var/run/libvirt/libvirt-sock \
  -p 9177:9177 \
  ghcr.io/tinkoff/libvirt-exporter
```

Watchdog (cron каждые 60 секунд):
```bash
#!/bin/bash
for vm in $(virsh list --all --name | grep "^cs2-"); do
  state=$(virsh domstate "$vm")
  if [[ "$state" != "running" ]]; then
    logger "Restarting crashed VM: $vm"
    virsh start "$vm"
  fi
done
```

### 9.5 Сколько VM влезет на хост?

Для CS2-воркеров (2 GB RAM, 2 vCPU каждая):

| Хост | VMs комфортно | VMs максимум (с KSM) |
|---|---|---|
| 16 GB RAM / 4 core | 4–5 | 6–7 |
| 32 GB RAM / 8 core | 8–10 | **12–16** |
| 64 GB RAM / 16 core | 18–22 | 28–32 |

**Правило:** Никогда не выделять больше 80–85% RAM хоста на VM. Оставить место для host page cache.

---

## 10. Коммуникация сервер-воркер

### 10.1 Сравнение протоколов

| Протокол | Overhead | Worker→Server push | Брокер | Лучше для |
|---|---|---|---|---|
| REST (HTTP) | Средний | Только polling | Нет | Простой CRUD, малый флот |
| **MQTT** | **Минимальный** | **Да (pub/sub)** | **Mosquitto** | **10–50 воркеров (рекомендуется)** |
| gRPC | Низкий | Да (streaming) | Нет | Большой флот, строгая типизация |
| Redis Pub/Sub | Низкий | Да | Redis | Если Redis уже используется |

**Вывод:** Для 10–50 воркеров — **MQTT (Mosquitto) + Redis** (Redis для очереди аккаунтов, MQTT для команд).

### 10.2 Архитектурная схема

```
┌─────────────────────────────────────────┐
│           Центральный сервер             │
│                                          │
│  FastAPI (REST admin) ←→ Redis (queue)  │
│       ↕                                  │
│  MQTT (Mosquitto broker)                 │
└───────────────┬─────────────────────────┘
                │
     ┌──────────┴──────────┐
     ▼                     ▼
Worker Agent (Python)   Worker Agent (Python)
 - subscribe commands    - subscribe commands
 - publish status        - publish status
 - libvirt API           - libvirt API
 - VNC controller        - VNC controller
```

### 10.3 Очередь аккаунтов (Redis)

```python
# Сервер: добавить аккаунт в очередь
r.rpush('accounts:queue', json.dumps({
    "username": "steamuser123",
    "password": "pass",
    "guard_code": None
}))

# Воркер: атомарно взять аккаунт
account_data = r.blmove('accounts:queue', f'accounts:processing:{worker_id}', timeout=30)

# Воркер: отметить выполненным
r.rpush('accounts:done', account_id)
```

### 10.4 Worker Agent (пример)

```python
import paho.mqtt.client as mqtt
import libvirt
import json
import threading
import time

class WorkerAgent:
    def __init__(self, broker, worker_id, redis_client):
        self.worker_id = worker_id
        self.redis = redis_client
        self.active_vms = {}

        # MQTT
        self.mqtt = mqtt.Client()
        self.mqtt.on_message = self.on_command
        self.mqtt.connect(broker)
        self.mqtt.subscribe(f"workers/{worker_id}/commands/#")
        self.mqtt.loop_start()

        # libvirt
        self.virt = libvirt.open('qemu:///system')

        # Heartbeat thread
        threading.Thread(target=self._heartbeat, daemon=True).start()

    def on_command(self, client, userdata, msg):
        cmd = json.loads(msg.payload)
        action = msg.topic.split('/')[-1]

        if action == 'start_account':
            self.start_vm_for_account(cmd['account_id'], cmd['vm_config'])
        elif action == 'stop_account':
            self.stop_vm(cmd['vm_id'])

    def _heartbeat(self):
        while True:
            self.mqtt.publish(f"workers/{self.worker_id}/status", json.dumps({
                "state": "online",
                "active_vms": len(self.active_vms),
                "timestamp": time.time()
            }))
            time.sleep(30)
```

---

## 11. Обнаружение VM и изоляция отпечатков

### 11.1 Основные векторы детекции и митигация

| Вектор | Сложность фикса | Решение |
|---|---|---|
| CPUID hypervisor bit | ✅ Легко | `<feature policy='disable' name='hypervisor'/>` |
| KVM CPUID leaf | ✅ Легко | `<kvm><hidden state='on'/></kvm>` |
| HyperV vendor_id | ✅ Легко | `<vendor_id state='on' value='GenuineIntel'/>` |
| MAC адрес (QEMU OUI) | ✅ Легко | Использовать реальный vendor OUI |
| NIC model (virtio = VEN_1AF4) | ✅ Легко | `<model type='e1000e'/>` |
| Disk serial ("QEMU HARDDISK") | ✅ Легко | Задать кастомный `<serial>` |
| SMBIOS strings (QEMU/BOCHS) | ✅ Легко | `<sysinfo>` блок |
| Memory balloon driver | ✅ Легко | `<memballoon model='none'/>` |
| RDTSC timing (1000–5000 cycles) | ❗ Сложно | kvm-rdtsc-hack (патч ядра) |
| ACPI table строки (BOCHS) | ❗ Сложно | qemu-anti-detection (патч QEMU) |
| WMI thermal/fan sensors | ❌ Нерешаемо программно | — |

### 11.2 Эффективность против VAC (CS2)

VAC — серверный античит без kernel-level доступа. Базовый спуфинг (SMBIOS + CPUID) даёт **высокую эффективность** против VAC.

Рейтинг по сложности обхода (от лёгкого к сложному):
1. **VAC** — базовый спуфинг достаточен
2. **EAC (Easy Anti-Cheat)** — нужен qemu-anti-detection
3. **BattlEye** — нужен RDTSC fix
4. **Vanguard** — kernel-level, очень сложно

### 11.3 Сетевая изоляция

Для корреляции аккаунтов нужны разные IP-адреса:

| Вариант | Сложность | Качество изоляции |
|---|---|---|
| Все VM через один IP (NAT) | Нет доп. затрат | Плохая |
| WireGuard VPN per VM | Умеренная | Хорошая |
| Residential proxy per VM | Простая | Хорошая |
| Отдельные IP от провайдера | Средняя | Отличная |

### 11.4 Поведенческая изоляция

Одинаковые паттерны поведения могут коррелировать аккаунты:
- Рандомизировать длину сессий (±20%)
- Рандомизировать время запуска
- Добавить jitter в движения мыши
- Использовать разные карты/режимы
- Имитировать человеческие паузы

---

## 12. Итоговые рекомендации и архитектурная схема

### Компонентный стек

| Компонент | Выбор | Обоснование |
|---|---|---|
| Host OS | Ubuntu 22.04+ LTS | Стабильный, хорошая поддержка KVM/libvirt |
| VM OS | Debian 12 minimal + OpenBox | Минимальный RAM overhead (~200 MB) |
| VM GPU | VirtIO-GPU Venus | Единственный Vulkan без passthrough |
| Скриншоты | VNC + vncdotool | Непрерывный поток, Python API |
| Input injection | VNC (vncdotool) | Высокоуровневое API, mouse+keyboard |
| VM память | 2 GB max + KSM + zswap | KSM даёт 30-50% экономию на identical VMs |
| VM клонирование | qcow2 backing store | ~1 сек создание, минимальное место |
| IaC | Terraform + libvirt | Декларативное управление флотом |
| Мониторинг | Prometheus + libvirt-exporter + Grafana | Стандартный стек |
| Коммуникация | MQTT (Mosquitto) | Легковесный, IoT-стандарт |
| Job queue | Redis | Атомарные операции с очередью |
| Спуфинг | SMBIOS + CPUID hide + e1000e NIC | Базовая защита от детекции |

### Полная схема развёртывания

```
┌────────────────────────────────────────────────────────┐
│                 Центральный сервер                      │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ FastAPI REST │  │    Redis     │  │  Mosquitto   │  │
│  │ (admin UI)   │  │ (job queue)  │  │   (MQTT)     │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└──────────────────────────┬─────────────────────────────┘
                            │ MQTT + Redis
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
         ┌──────────┐  ┌──────────┐  ┌──────────┐
         │ Воркер 1 │  │ Воркер 2 │  │ Воркер N │
         │ Ubuntu   │  │ Ubuntu   │  │ Ubuntu   │
         │ 32GB RAM │  │ 32GB RAM │  │ 32GB RAM │
         │          │  │          │  │          │
         │ KVM VMs: │  │ KVM VMs: │  │ KVM VMs: │
         │ ┌──┐┌──┐ │  │ ┌──┐┌──┐ │  │ ┌──┐┌──┐ │
         │ │v1││v2│ │  │ │v1││v2│ │  │ │v1││v2│ │
         │ └──┘└──┘ │  │ └──┘└──┘ │  │ └──┘└──┘ │
         └──────────┘  └──────────┘  └──────────┘
              │
              ▼
    Worker Agent (Python daemon)
    - MQTT: получает команды
    - Redis: берёт аккаунты из очереди
    - libvirt: управляет VM
    - VNC: читает экран, отправляет ввод
    - AI inference: принимает решения
```

### Примерная пропускная способность

Один хост (32 GB RAM, 8 core):
- **8–10 VM** с CS2 комфортно (2 GB RAM / 2 vCPU каждая)
- **12–16 VM** с KSM (эффективная деупликация одинаковых VM)
- ~30 мин активной игры в неделю на аккаунт для дропа

При N хостах: N × 8–16 параллельных аккаунтов.

---

## 13. Список литературы

### CS2 и Steam
- CS2 System Requirements: https://store.steampowered.com/app/730/CounterStrike_2/
- CS2 Weekly Drop Guide: https://tradeit.gg/blog/cs2-drop-pool/
- CS2 Launch Options: https://totalcsgo.com/launch-options
- GamingOnLinux Anti-Cheat Tracker: https://www.gamingonlinux.com/anticheat/
- ProtonDB CS2: https://www.protondb.com/app/730
- Steam Headless (gist): https://gist.github.com/joshuaboniface/50690ad188df15033c5f04b3cac31845

### KVM/QEMU
- QEMU VirtIO-GPU Venus: https://docs.mesa3d.org/drivers/venus.html
- QEMU Guest Graphics Acceleration (ArchWiki): https://wiki.archlinux.org/title/QEMU/Guest_graphics_acceleration
- virglrenderer State (Collabora, 2025): https://www.collabora.com/news-and-blog/blog/2025/01/15/the-state-of-gfx-virtualization-using-virglrenderer/
- Venus QEMU setup: https://gist.github.com/peppergrayxyz/fdc9042760273d137dddd3e97034385f
- QEMU QMP Reference: https://qemu-project.gitlab.io/qemu/interop/qemu-qmp-ref.html
- vncdotool: https://github.com/sibson/vncdotool

### libvirt и управление VM
- libvirt Domain XML: https://libvirt.org/formatdomain.html
- Terraform + libvirt: https://computingforgeeks.com/how-to-provision-vms-on-kvm-with-terraform/
- qcow2 Thin Clones: https://metamost.com/post/tech/libvirt-linked-clones/
- libvirt-exporter (Prometheus): https://github.com/Tinkoff/libvirt-exporter
- Grafana Dashboard #15682: https://grafana.com/grafana/dashboards/15682-libvirt/

### Оптимизация памяти
- zswap vs zram (Chris Down, 2026): https://chrisdown.name/2026/03/24/zswap-vs-zram-when-to-use-what.html
- ZRAM (ArchWiki): https://wiki.archlinux.org/title/Zram
- KSM (KVM): https://www.linux-kvm.org/page/KSM
- KSM (Proxmox): https://pve.proxmox.com/wiki/Kernel_Samepage_Merging_(KSM)
- Huge Pages Benchmark: https://developers.redhat.com/blog/2021/04/27/benchmarking-transparent-versus-1gib-static-huge-page-performance-in-linux-virtual-machines

### Спуфинг железа и anti-detection
- SMBIOS in Virtualization: https://michael2012z.medium.com/smbios-in-virtualization-939af825eb19
- qemu-anti-detection: https://github.com/zhaodice/qemu-anti-detection
- Anti-cheat VM detection: https://secret.club/2020/04/13/how-anti-cheats-detect-system-emulation.html
- kvm-rdtsc-hack: https://github.com/h33p/kvm-rdtsc-hack

### Деплоймент и архитектура
- Packer Ubuntu: https://developer.hashicorp.com/packer/guides/automatic-operating-system-installs/preseed_ubuntu
- cloud-init: https://ubuntu.com/server/docs/explanation/intro-to/cloud-init/
- paho-mqtt: https://pypi.org/project/paho-mqtt/
- Redis Queue: https://redis.io/docs/manual/pubsub/
- Mosquitto MQTT: https://mosquitto.org/
