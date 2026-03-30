# Topic 19: Steam Session Injection — способы инжекции сессий в VM

## Проблема
Одна VM должна фармить разные аккаунты в разное время. Нужен стабильный способ входа в конкретный Steam аккаунт внутри VM без ручного вмешательства.

## Метод 1: Предзаполнение файлов сессии (Pre-populated Session Files)

### Как работает
1. **Подготовка**: На "golden" машине логинимся в Steam с "Remember Me"
2. **Экспорт**: Копируем файлы сессии:
   - `~/.steam/steam/config/config.vdf` — refresh token, machine ID
   - `~/.steam/steam/config/loginusers.vdf` — список пользователей
   - `~/.steam/steam/ssfn*` — sentry файлы (Steam Guard machine token)
3. **Инжекция**: Записываем файлы в VM через:
   - **Вариант A**: Монтирование qcow2 overlay на хосте перед загрузкой VM
   - **Вариант B**: QEMU Guest Agent `guest-file-write` после загрузки VM
4. **Запуск**: Steam подхватывает сохранённую сессию, авто-логин

### Формат loginusers.vdf
```vdf
"users"
{
    "76561198012345678"
    {
        "AccountName"        "myaccount"
        "PersonaName"        "MyName"
        "RememberPassword"   "1"
        "WantsOfflineMode"   "0"
        "SkipOfflineModeWarning"    "0"
        "AllowAutoLogin"     "1"
        "MostRecent"         "1"
        "Timestamp"          "1709500000"
    }
}
```

### Формат config.vdf (ключевые секции)
```vdf
"InstallConfigStore"
{
    "Software"
    {
        "Valve"
        {
            "Steam"
            {
                "AutoLoginUser"        "myaccount"
                "RememberPassword"     "1"
                "Accounts"
                {
                    "myaccount"
                    {
                        "SteamID"    "76561198012345678"
                    }
                }
            }
        }
    }
}
```

### Плюсы
- Проверенный метод, используется account switcher'ами
- Не зависит от API Steam
- Работает до тех пор, пока refresh token валиден (~200 дней)

### Минусы
- Refresh token может быть отозван Valve (смена пароля, подозрительная активность)
- Нужен batch provisioning — периодическое обновление токенов
- Sentry файлы привязаны к machine ID

## Метод 2: Программная авторизация через Steam API (Refresh Token Injection)

### Как работает
1. **Сервер** хранит refresh tokens для всех аккаунтов
2. **При переключении**: сервер отправляет refresh token воркеру
3. **Воркер** записывает refresh token в config.vdf через guest-agent
4. **Steam** использует refresh token для автоматической авторизации

### Получение refresh token программно
Используя протокол Steam (protobuf API):
```
CAuthentication_BeginAuthSessionViaCredentials_Request
→ guard satisfaction (TOTP/email)
→ CAuthentication_PollAuthSessionStatus_Response { refresh_token, access_token }
```

### Библиотеки для получения tokens
- **node-steam-session** (JavaScript): Полная реализация auth flow
- **steam** (Python, ValvePython): SteamClient + WebAuth
- **Собственная реализация на Rust**: Через protobuf + HTTP/WebSocket

### Плюсы
- Программная генерация и обновление tokens
- Центральное хранение на сервере
- Не нужен "golden" машинный логин
- Можно автоматизировать batch provisioning

### Минусы
- Сложная реализация (protobuf, HMAC, JWT)
- Зависимость от стабильности Steam API
- Steam Guard всё равно нужно обработать при первом логине

## Метод 3: QR код авторизация

### Как работает
1. **Внутри VM**: Steam показывает QR код на экране логина
2. **Хост-воркер**: Читает QR с VNC framebuffer (OpenCV + QR decoder)
3. **Сервер**: Получает challenge URL из QR
4. **LoginApprover**: Подписывает challenge с shared_secret аккаунта
5. **Steam**: Авторизует сессию

### Техническая реализация
```
QR URL: https://s.team/q/{version}/{clientId}

LoginApprover flow:
1. decodeQrUrl(url) → { version, clientId }
2. signatureData = [version(2b), clientId(8b), steamID(8b)] // 18 bytes
3. signature = HMAC-SHA256(shared_secret, signatureData)
4. submitMobileConfirmation(clientId, steamID, signature, approve=true)
```

### Плюсы
- Не нужно хранить пароли
- Steam считает это "одобренным устройством" после подтверждения
- Работает с новой системой аутентификации Steam

### Минусы
- Зависимость от VNC для чтения QR кода
- QR может быть плохо виден на 384x288 разрешении
- Дополнительная задержка на распознавание и подтверждение
- Требует shared_secret для каждого аккаунта
- Работает только когда Steam клиент уже запущен и показывает login screen

## Метод 4: SteamCMD для начальной авторизации

### Как работает
1. `steamcmd +login user pass +quit` — кэширует credentials
2. Credential файлы используются основным Steam клиентом
3. Подходит только для начального bootstrap

### Ограничения
- `-login user pass` сломан для основного клиента Steam (mid-2023)
- SteamCMD может всё ещё работать для некоторых операций
- Не подходит для динамического переключения аккаунтов

## Рекомендуемая стратегия

### Primary: Refresh Token Injection через Guest-Agent
1. Сервер хранит refresh tokens (зашифрованные) для всех аккаунтов
2. При назначении аккаунта VM:
   - `guest-exec`: остановить Steam если запущен
   - `guest-file-write`: записать config.vdf с AutoLoginUser и refresh token
   - `guest-file-write`: записать loginusers.vdf с нужным аккаунтом
   - `guest-exec`: запустить Steam
3. Steam автоматически использует refresh token

### Fallback: QR код через VNC
- Если refresh token истёк
- LoginApprover подтверждает QR через API
- Новый refresh token сохраняется на сервере

### Token Renewal
- Batch процесс обновления tokens раз в месяц
- Rust-реализация протокола Steam Auth для серверной стороны
- Или использование node-steam-session/ValvePython как внешнего сервиса

## Источники
- https://github.com/DoctorMcKay/node-steam-session — auth flow reference
- https://github.com/SteamDatabase/SteamTracking — protobuf definitions
- topic_16_steam_auth_deep_dive.md — детальный анализ auth
- topic_17_qemu_guest_agent.md — guest-agent capabilities
