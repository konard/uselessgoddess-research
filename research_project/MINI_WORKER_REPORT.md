# Анализ: Mini-Worker внутри VM — нужен или нет?

## Оглавление
1. [Резюме](#резюме)
2. [Контекст и постановка вопроса](#контекст-и-постановка-вопроса)
3. [Инструменты управления VM снаружи](#инструменты-управления-vm-снаружи)
4. [Анализ Steam авторизации](#анализ-steam-авторизации)
5. [Три архитектурных подхода](#три-архитектурных-подхода)
6. [Сравнительные таблицы по задачам](#сравнительные-таблицы-по-задачам)
7. [Главная таблица: Mini-Worker vs External Control](#главная-таблица-mini-worker-vs-external-control)
8. [Рекомендуемая архитектура](#рекомендуемая-архитектура)
9. [Реализация Steam Session Injection](#реализация-steam-session-injection)
10. [Заключение](#заключение)
11. [Источники](#источники)

---

## Резюме

**Вердикт: Полноценный мини-воркер НЕ нужен. Рекомендуется гибридный подход: QEMU Guest Agent + systemd-скрипт.**

Ключевые выводы:
- QEMU Guest Agent (`qemu-ga`) покрывает 90% задач мини-воркера: выполнение команд, запись файлов, проверка процессов
- Для инжекции Steam сессий достаточно `guest-file-write` — запись config.vdf и loginusers.vdf
- QR код логин **не рекомендуется** как основной метод — ненадёжен на 384×288 и зависит от VNC
- Основной метод: **Refresh Token Injection** через Guest Agent
- Визуальное управление (матчи, геймплей) остаётся через VNC + AI — это нужно в любом случае
- Простой systemd unit внутри VM для автозапуска Steam заменяет мини-воркер

---

## Контекст и постановка вопроса

### Система
- Хост-воркер (Rust, `vmctl`) управляет KVM/QEMU виртуальными машинами
- Каждая VM запускает Linux Debian + OpenBox + Steam + CS2
- Одна VM в конкретный момент фармит один аккаунт, но может переключаться
- Управление снаружи: VNC (framebuffer + input), QMP (keyboard/mouse), Guest Agent
- Коммуникация: shai RPC (QUIC, zero-copy, rkyv)

### Вопрос
Нужен ли мини-воркер — отдельный сервис внутри каждой VM, который:
1. Принимает команды от хост-воркера (войди в аккаунт X, запусти CS2)
2. Управляет Steam (записывает config файлы, мониторит процессы)
3. Отправляет обратную связь (статус, ошибки, health)

Или можно обойтись полностью внешним управлением?

---

## Инструменты управления VM снаружи

### QEMU Guest Agent (qemu-ga)
Стандартный демон внутри VM, коммуникация через virtio-serial (без сети).

| Команда | Описание | Версия QEMU |
|---------|----------|-------------|
| `guest-exec` | Выполнение произвольных команд | 2.5+ |
| `guest-exec-status` | Проверка статуса процесса (exit code, stdout/stderr) | 2.5+ |
| `guest-file-open` | Открытие файла в госте | 0.15+ |
| `guest-file-write` | Запись данных в файл (base64) | 0.15+ |
| `guest-file-read` | Чтение файла (до 48 MB) | 0.15+ |
| `guest-ping` | Проверка доступности агента | 0.15+ |
| `guest-shutdown` | Перезагрузка/выключение | 0.15+ |
| `guest-network-get-interfaces` | IP/MAC всех интерфейсов | 1.1+ |

**Ключевое:** Guest Agent может выполнять команды и записывать файлы внутри VM без сетевого подключения. Это стандартный компонент, не вызывающий подозрений у антивирусов/VAC.

### VNC
- Чтение framebuffer VM в реальном времени
- Отправка keyboard/mouse events
- Используется для визуального управления (принятие матча, gameplay)

### QMP (QEMU Machine Protocol)
- `input-send-event` — keyboard/mouse injection
- `screendump` — единичный скриншот в PPM формате
- Низкоуровневое управление VM

---

## Анализ Steam авторизации

### Современная система (2024+)
Steam перешёл на JWT-based аутентификацию. Ключевые факты:

| Аспект | Значение |
|--------|----------|
| Refresh Token срок жизни | ~200 дней |
| Access Token | Короткий срок, генерируется из refresh |
| `-login user pass` (CLI) | **Сломан с mid-2023** |
| QR логин | Требует shared_secret для автоматизации |
| Session файлы | config.vdf + loginusers.vdf |
| Machine Auth | ssfn файлы (SHA-1 hash) |

### Методы авторизации для автоматизации

#### 1. Refresh Token Injection (✅ РЕКОМЕНДУЕТСЯ)

**Принцип:** Записать refresh token в config.vdf → Steam автоматически входит.

**Как получить refresh token:**
- Программно через Steam Auth API (protobuf): `BeginAuthSessionViaCredentials` → `PollAuthSessionStatus` → `{ refresh_token }`
- Библиотеки: [node-steam-session](https://github.com/DoctorMcKay/node-steam-session), [ValvePython/steam](https://github.com/ValvePython/steam)
- Batch provisioning: скрипт на сервере получает tokens для всех аккаунтов

**Как инжектить:**
```bash
# Через QEMU Guest Agent (с хоста):
virsh qemu-agent-command vm-name '{"execute":"guest-exec","arguments":{"path":"/bin/bash","arg":["-c","pkill -f steam || true"]}}'
# Ждём завершения Steam

# Записываем config.vdf с refresh token
virsh qemu-agent-command vm-name '{"execute":"guest-file-open","arguments":{"path":"/home/user/.steam/steam/config/config.vdf","mode":"w"}}'
# → handle
virsh qemu-agent-command vm-name '{"execute":"guest-file-write","arguments":{"handle":1,"buf-b64":"<base64_config_vdf>"}}'
virsh qemu-agent-command vm-name '{"execute":"guest-file-close","arguments":{"handle":1}}'

# Аналогично loginusers.vdf
# Запускаем Steam
virsh qemu-agent-command vm-name '{"execute":"guest-exec","arguments":{"path":"/usr/bin/steam","arg":["-silent"]}}'
```

**Стабильность:** Высокая. Refresh token валиден ~200 дней. Steam подхватывает сессию автоматически.

#### 2. QR код через VNC (⚠️ FALLBACK)

**Принцип:** Steam показывает QR → хост читает через VNC → LoginApprover подтверждает.

**Проблемы:**
- QR код на 384×288 может быть нечитаемым (слишком мелкий)
- Зависимость от UI Steam (ломается при обновлениях)
- Требует shared_secret для каждого аккаунта
- Дополнительная задержка: VNC capture → QR decode → API call → confirm

**Когда использовать:** Только как fallback если refresh token истёк и нужно получить новый интерактивно.

#### 3. Предзаполненные session файлы (✅ АЛЬТЕРНАТИВА)

**Принцип:** На "golden" машине залогинились → экспортировали файлы → инжектим в overlay.

**Файлы:**
- `config.vdf` — AutoLoginUser, refresh tokens
- `loginusers.vdf` — RememberPassword=1, AllowAutoLogin=1, MostRecent=1
- `ssfn*` — Machine Auth sentry файлы

**Инжекция до загрузки VM:**
```bash
# Монтирование qcow2 overlay (VM должна быть выключена)
sudo modprobe nbd max_part=8
sudo qemu-nbd --connect=/dev/nbd0 /var/lib/libvirt/images/vm-overlay.qcow2
sudo mount /dev/nbd0p1 /mnt/vm-disk
# Копируем файлы
sudo cp config.vdf /mnt/vm-disk/home/user/.steam/steam/config/config.vdf
sudo cp loginusers.vdf /mnt/vm-disk/home/user/.steam/steam/config/loginusers.vdf
sudo umount /mnt/vm-disk
sudo qemu-nbd --disconnect /dev/nbd0
```

**Или после загрузки через Guest Agent** (см. метод 1).

---

## Три архитектурных подхода

### Подход A: Полностью внешнее управление

```
┌──────────────────────────────────┐
│        Хост-воркер (Rust)        │
│  ┌──────────┐  ┌──────────────┐  │
│  │ VNC      │  │ Guest Agent  │  │
│  │ client   │  │ (virsh cmd)  │  │
│  └────┬─────┘  └──────┬───────┘  │
│       │               │          │
│  ┌────┴───────────────┴───────┐  │
│  │         VM                 │  │
│  │  ┌─────────────────────┐   │  │
│  │  │ qemu-ga (standard)  │   │  │
│  │  │ Steam               │   │  │
│  │  │ CS2                 │   │  │
│  │  └─────────────────────┘   │  │
│  └────────────────────────────┘  │
└──────────────────────────────────┘
```

**VM содержит:** только ОС + qemu-ga + Steam + CS2. Нет кастомного кода.

### Подход B: Мини-воркер внутри VM

```
┌──────────────────────────────────┐
│        Хост-воркер (Rust)        │
│  ┌──────────┐  ┌──────────────┐  │
│  │ VNC      │  │ shai RPC     │  │
│  │ client   │  │ client       │  │
│  └────┬─────┘  └──────┬───────┘  │
│       │               │ (network)│
│  ┌────┴───────────────┴───────┐  │
│  │         VM                 │  │
│  │  ┌─────────────────────┐   │  │
│  │  │ mini-worker (Rust)  │←──┤  │
│  │  │   ├─ Steam control  │   │  │
│  │  │   ├─ TOTP gen       │   │  │
│  │  │   └─ Health report  │   │  │
│  │  │ Steam               │   │  │
│  │  │ CS2                 │   │  │
│  │  └─────────────────────┘   │  │
│  └────────────────────────────┘  │
└──────────────────────────────────┘
```

**VM содержит:** ОС + mini-worker binary + Steam + CS2. Кастомный Rust сервис.

### Подход C: Гибрид (Guest Agent + systemd скрипт) — РЕКОМЕНДУЕМЫЙ

```
┌──────────────────────────────────┐
│        Хост-воркер (Rust)        │
│  ┌──────────┐  ┌──────────────┐  │
│  │ VNC      │  │ Guest Agent  │  │
│  │ client   │  │ (virsh cmd)  │  │
│  └────┬─────┘  └──────┬───────┘  │
│       │               │          │
│  ┌────┴───────────────┴───────┐  │
│  │         VM                 │  │
│  │  ┌─────────────────────┐   │  │
│  │  │ qemu-ga (standard)  │   │  │
│  │  │ steam-launcher.sh   │   │  │
│  │  │   (systemd service) │   │  │
│  │  │ Steam               │   │  │
│  │  │ CS2                 │   │  │
│  │  └─────────────────────┘   │  │
│  └────────────────────────────┘  │
└──────────────────────────────────┘
```

**VM содержит:** ОС + qemu-ga + простой bash-скрипт (systemd) + Steam + CS2.

---

## Сравнительные таблицы по задачам

### Задача 1: Инжекция Steam сессии (переключение аккаунтов)

| Критерий | Guest Agent | Mini-Worker | VNC авто | qcow2 mount |
|----------|:-----------:|:-----------:|:--------:|:-----------:|
| Сложность реализации | Низкая | Средняя | Высокая | Низкая |
| Надёжность | ★★★★★ | ★★★★★ | ★★☆☆☆ | ★★★★★ |
| Скорость переключения | ~5 сек | ~3 сек | ~30 сек | ~60 сек (reboot) |
| VM должна быть запущена | Да | Да | Да | Нет |
| Зависимость от UI Steam | Нет | Нет | Да | Нет |
| Доп. ПО в VM | qemu-ga (стандарт) | Кастомный binary | Нет | Нет |

**Вывод:** Guest Agent — оптимальный баланс. Мини-воркер экономит ~2 секунды, но добавляет сложность.

### Задача 2: Мониторинг состояния Steam/CS2

| Критерий | Guest Agent polling | Mini-Worker | VNC анализ |
|----------|:-------------------:|:-----------:|:----------:|
| Реактивность | 2-5 сек | <100 мс | 200-500 мс |
| CPU overhead на хосте | Минимальный | Минимальный | Высокий (CV) |
| CPU overhead в VM | Минимальный | Низкий | Нет |
| Структурированные данные | Да (через скрипты) | Да (нативно) | Нет (только пиксели) |
| Детекция ошибок Steam | Через логи | Через API/логи | Через визуал |
| Детекция VAC | Нет | Потенциально | Нет |

**Вывод:** Для фарма polling 2-5 сек достаточен. Realtime мониторинг не критичен — CS2 матч длится минуты.

### Задача 3: Steam Guard обработка

| Критерий | TOTP на хосте + Guest Agent | Mini-Worker TOTP | QR + LoginApprover | VNC ввод кода |
|----------|:---------------------------:|:----------------:|:------------------:|:-------------:|
| Сложность | Низкая | Средняя | Средняя | Высокая |
| Надёжность | ★★★★★ | ★★★★★ | ★★★★☆ | ★★☆☆☆ |
| Требует shared_secret | Да | Да | Да | Нет (email) |
| Работает без сети в VM | Да (virtio-serial) | Нет (нужна сеть) | Нет (нужна сеть) | Только VNC |

**Вывод:** Генерация TOTP на хосте и ввод через guest-exec — самый простой и надёжный метод.

### Задача 4: Запуск и остановка CS2

| Критерий | Guest Agent exec | systemd service | Mini-Worker | VNC click |
|----------|:----------------:|:---------------:|:-----------:|:---------:|
| Надёжность запуска | ★★★★☆ | ★★��★★ | ★★★★★ | ★★☆☆☆ |
| Graceful shutdown | `kill -TERM` | `systemctl stop` | Нативно | Нет |
| Перезапуск при краше | Нет | `Restart=on-failure` | Да | Нет |
| Контроль параметров | Полный | Фиксированный | Полный | Визуальный |

**Вывод:** systemd service с `Restart=on-failure` — лучший вариант для автозапуска. Guest-exec — для динамических команд.

---

## Главная таблица: Mini-Worker vs External Control

| Критерий | Подход A: Внешнее | Подход B: Mini-Worker | Подход C: Гибрид |
|----------|:-----------------:|:---------------------:|:----------------:|
| **Сложность разработки** | ★☆☆☆☆ Низкая | ★★★★☆ Высокая | ★★☆☆☆ Низкая |
| **Сложность поддержки** | ★☆☆☆☆ Минимальная | ★★★☆☆ Средняя | ★☆☆☆☆ Минимальная |
| **Обновление логики** | На хосте (мгновенно) | В каждой VM (накладно) | Скрипты через GA |
| **Footprint в VM** | qemu-ga (2 MB RAM) | +binary (10-50 MB) | +bash скрипт |
| **Детекция VAC** | Нет риска | Потенциальный риск | Нет риска |
| **Скорость переключения** | ~5 сек | ~3 сек | ~5 сек |
| **Мониторинг** | Polling 2-5 сек | Realtime <100мс | Polling 2-5 сек |
| **Обратная связь** | Только по запросу | Push events | Только по запросу |
| **Зависимость от сети** | Нет (virtio-serial) | Да (TCP/UDP) | Нет (virtio-serial) |
| **Масштабирование** | 10+ VM без проблем | Каждая VM = процесс | 10+ VM без проблем |
| **Восстановление после краша** | systemd restart | Нужен supervisor | systemd restart |

### Итог сравнения

| Аспект | Победитель | Почему |
|--------|------------|--------|
| **Инжекция сессий** | Гибрид (C) | Guest Agent file-write достаточен |
| **Переключение аккаунтов** | Гибрид (C) | kill steam + write files + restart через GA |
| **Мониторинг** | Mini-Worker (B) | Realtime events, но polling достаточен для фарма |
| **Steam Guard** | Гибрид (C) | TOTP на хосте + guest-exec |
| **Безопасность (VAC)** | Гибрид (C) | Минимальный footprint, стандартные компоненты |
| **Простота** | Гибрид (C) | Нет кастомного кода в VM, легко обновлять |
| **Масштабируемость** | Гибрид (C) | Нет дополнительных сетевых соединений |

**Mini-Worker выигрывает только по мониторингу**, но для фарма CS2 кейсов realtime не нужен. Матч длится 5-40 минут, polling каждые 2-5 секунд более чем достаточен.

---

## Рекомендуемая архитектура

### Компоненты внутри VM (base image)

```
/etc/systemd/system/steam-farm.service    # systemd unit для Steam
/opt/farm/steam-launcher.sh               # Bash скрипт запуска Steam
```

#### steam-farm.service
```ini
[Unit]
Description=Steam CS2 Farming Session
After=network.target display-manager.service

[Service]
Type=simple
User=farmuser
Environment=DISPLAY=:0
ExecStart=/opt/farm/steam-launcher.sh
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

#### steam-launcher.sh
```bash
#!/bin/bash
# Ждём когда config файлы будут записаны guest-agent'ом
while [ ! -f /home/farmuser/.steam/steam/config/.ready ]; do
    sleep 1
done
rm /home/farmuser/.steam/steam/config/.ready

# Запускаем Steam в silent mode
exec steam -silent -no-browser -console \
    -w 384 -h 288 \
    +connect_lobby default
```

### Workflow переключения аккаунтов (на хосте)

```rust
// Псевдокод хост-воркера
async fn switch_account(vm: &str, account: &AccountSession) -> Result<()> {
    let ga = GuestAgent::new(vm);

    // 1. Остановить Steam
    ga.exec("pkill -TERM -f steam || true").await?;
    tokio::time::sleep(Duration::from_secs(3)).await;

    // 2. Записать session файлы
    ga.write_file(
        "/home/farmuser/.steam/steam/config/config.vdf",
        &account.config_vdf,
    ).await?;

    ga.write_file(
        "/home/farmuser/.steam/steam/config/loginusers.vdf",
        &account.loginusers_vdf,
    ).await?;

    // 3. Записать sentry файлы если есть
    if let Some(ssfn) = &account.ssfn_data {
        ga.write_file(
            &format!("/home/farmuser/.steam/steam/{}", account.ssfn_name),
            ssfn,
        ).await?;
    }

    // 4. Сигнал готовности для systemd service
    ga.write_file(
        "/home/farmuser/.steam/steam/config/.ready",
        b"1",
    ).await?;

    // 5. Если Steam уже запущен — перезапуск через systemd
    ga.exec("systemctl --user restart steam-farm.service").await?;

    // 6. Проверка через polling
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let status = ga.exec("pgrep -c steam").await?;
        if status.exit_code == 0 {
            return Ok(());
        }
    }

    Err(Error::SteamStartTimeout)
}
```

### Workflow мониторинга (на хосте)

```rust
// Periodic health check через Guest Agent
async fn health_check(vm: &str) -> VmHealth {
    let ga = GuestAgent::new(vm);

    // Проверка процессов
    let steam_running = ga.exec("pgrep -c steam").await.map(|r| r.exit_code == 0).unwrap_or(false);
    let cs2_running = ga.exec("pgrep -c cs2").await.map(|r| r.exit_code == 0).unwrap_or(false);

    // Проверка логов на ошибки
    let steam_log = ga.exec("tail -20 /home/farmuser/.steam/steam/logs/bootstrap_log.txt").await;

    VmHealth {
        steam_running,
        cs2_running,
        last_log: steam_log.map(|r| r.stdout).unwrap_or_default(),
    }
}
```

---

## Реализация Steam Session Injection

### Подготовка (batch provisioning на сервере)

Сервер периодически (раз в месяц) обновляет refresh tokens:

```python
# Используя node-steam-session или ValvePython/steam
from steam.client import SteamClient

client = SteamClient()
client.login(username, password, two_factor_code=totp_code)
# Сохранить refresh token и session данные
refresh_token = client.refresh_token
# Зашифровать и сохранить в БД
```

Или через Rust-реализацию Steam Auth protobuf API (рекомендуется для единого стека):

```
BeginAuthSessionViaCredentials {
    account_name: "user",
    encrypted_password: rsa_encrypt(password, public_key),
    persistence: Persistent,
}
→ steam_guard_required
→ submit_guard_code(totp_code)
→ PollAuthSessionStatus
→ { refresh_token, access_token }
```

### Генерация config.vdf для аккаунта

```rust
fn generate_config_vdf(account_name: &str, steam_id: &str) -> String {
    format!(r#""InstallConfigStore"
{{
    "Software"
    {{
        "Valve"
        {{
            "Steam"
            {{
                "AutoLoginUser"        "{account_name}"
                "RememberPassword"     "1"
                "Accounts"
                {{
                    "{account_name}"
                    {{
                        "SteamID"    "{steam_id}"
                    }}
                }}
            }}
        }}
    }}
}}"#)
}
```

### Генерация loginusers.vdf

```rust
fn generate_loginusers_vdf(account_name: &str, steam_id: &str, persona: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    format!(r#""users"
{{
    "{steam_id}"
    {{
        "AccountName"             "{account_name}"
        "PersonaName"             "{persona}"
        "RememberPassword"        "1"
        "WantsOfflineMode"        "0"
        "SkipOfflineModeWarning"  "0"
        "AllowAutoLogin"          "1"
        "MostRecent"              "1"
        "Timestamp"               "{timestamp}"
    }}
}}"#)
}
```

### Когда QR всё-таки нужен

QR логин имеет смысл **только** если:
1. Refresh token истёк и нет возможности получить новый через API
2. Аккаунт требует подтверждение нового устройства
3. Batch provisioning не покрывает все аккаунты

В этом случае:
1. Steam запускается без auto-login → показывает QR
2. Хост-воркер делает VNC screenshot
3. QR декодируется → `https://s.team/q/{version}/{clientId}`
4. LoginApprover (на сервере) подписывает challenge с shared_secret аккаунта
5. Steam авторизует сессию, генерируется новый refresh token
6. Хост-воркер сохраняет новый refresh token на сервере

**Важно:** При разрешении 384×288 QR код может быть нечитаемым. Для QR fallback рекомендуется временно увеличить разрешение VM до 1024×768.

---

## Заключение

### Ответ на главный вопрос

**Мини-воркер НЕ нужен.** QEMU Guest Agent + простой systemd скрипт полностью покрывают все задачи:

1. **Инжекция сессий** → `guest-file-write` для config.vdf / loginusers.vdf
2. **Запуск Steam** → systemd service с auto-restart
3. **Переключение аккаунтов** → kill steam + write new files + restart (через GA)
4. **Steam Guard** → TOTP генерируется на хосте, вводится через `guest-exec`
5. **Мониторинг** → `guest-exec` polling каждые 2-5 секунд (pgrep, tail logs)
6. **Визуальное управление** → VNC + AI (нужно в любом случае, не зависит от мини-воркера)

### Лучший способ входа в Steam

**Primary:** Refresh Token Injection через Guest Agent
- Сервер хранит зашифрованные refresh tokens (~200 дней жизни)
- При назначении аккаунта: записать config.vdf + loginusers.vdf → запустить Steam
- Batch provisioning раз в месяц через Steam Auth API

**Fallback:** QR код через VNC + LoginApprover
- Только если refresh token истёк
- Временное увеличение разрешения для читаемости QR

### Почему НЕ мини-воркер

1. **VAC риск**: Кастомный процесс в VM может быть задетектирован
2. **Сложность**: Нужно компилировать, деплоить, обновлять бинарник в каждой VM
3. **Сеть**: Требует сетевое соединение между VM и хостом (Guest Agent работает без сети)
4. **Минимальный выигрыш**: 2-3 секунды на переключение и realtime мониторинг — не критично для фарма
5. **Стандартность**: qemu-ga + systemd — стандартные компоненты Linux, не требуют обслуживания

### Когда мини-воркер МОЖЕТ понадобиться

Единственный сценарий: если в будущем понадобится **realtime обратная связь** из VM (например, мгновенная реакция на события Steam — кики, баны, ошибки сети). Но даже тогда лучше рассмотреть расширение Guest Agent через кастомные команды, а не отдельный сервис.

---

## Источники

### Steam Authentication
1. [node-steam-session](https://github.com/DoctorMcKay/node-steam-session) — Полная реализация Steam Auth flow (QR, credentials, refresh tokens)
2. [node-steam-user](https://github.com/DoctorMcKay/node-steam-user) — Эмуляция Steam клиента, refresh tokens ~200 дней
3. [SteamDatabase/SteamTracking](https://github.com/SteamDatabase/SteamTracking) — Protobuf определения Steam API
4. [Valve Authentication Docs](https://partner.steamgames.com/doc/features/auth) — Официальная документация Session Tickets, App Tickets
5. [ValvePython/steam](https://github.com/ValvePython/steam) — Python Steam клиент с SteamAuthenticator
6. [ArchWiki Steam](https://wiki.archlinux.org/title/Steam/Troubleshooting) — Linux конфигурация, registry.vdf баг

### QEMU / VM Management
7. [QEMU Guest Agent Reference](https://www.qemu.org/docs/master/interop/qemu-ga-ref.html) — guest-exec, guest-file-write, все команды
8. [QMP Reference](https://www.qemu.org/docs/master/interop/qemu-qmp-ref.html) — input-send-event, screendump

### Внутренние исследования
9. `research_project/notes/topic_16_steam_auth_deep_dive.md`
10. `research_project/notes/topic_17_qemu_guest_agent.md`
11. `research_project/notes/topic_18_mini_worker_analysis.md`
12. `research_project/notes/topic_19_steam_session_injection.md`
13. `research_project/notes/topic_12_steam_login.md`
