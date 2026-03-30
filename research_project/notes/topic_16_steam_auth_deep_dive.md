# Topic 16: Steam Authentication Deep Dive

## Современная система аутентификации Steam (2024+)

### Обзор
Steam перешёл на JWT-based систему аутентификации. Старый метод `-login user pass` через CLI сломан с mid-2023. Новая система основана на refresh tokens и access tokens.

### Типы токенов

#### Refresh Token
- JWT, срок жизни ~200 дней
- Используется для получения новых access tokens
- При обновлении старый refresh token становится невалидным
- Хранится в файлах конфигурации Steam клиента

#### Access Token
- JWT, короткий срок жизни
- Используется для API запросов
- Генерируется из refresh token

#### Machine Auth Token (Steam Guard)
- SHA-1 хэш sentry файла
- Позволяет обойти повторный ввод email кода
- Специфичен для платформы SteamClient

### Методы аутентификации

#### 1. Логин по credentials (username + password)
**Protobuf:** `CAuthentication_BeginAuthSessionViaCredentials_Request`
- RSA-шифрование пароля
- После успешного логина требуется удовлетворить Steam Guard
- Результат: refresh token + access token

#### 2. QR код аутентификация
**Protobuf:** `CAuthentication_BeginAuthSessionViaQR_Request`
- Генерируется challenge URL: `https://s.team/q/{version}/{clientId}`
- URL кодируется в QR код
- Сканирование мобильным приложением Steam
- Подтверждение через DeviceConfirmation guard

**Программная автоматизация QR:**
Библиотека `node-steam-session` реализует `LoginApprover`:
1. Создаётся QR challenge на стороне клиента (`startWithQR()`)
2. `LoginApprover` с access token + shared_secret подписывает challenge
3. HMAC-SHA256 подпись из: version + clientId + steamID
4. Отправка через `submitMobileConfirmation()`

#### 3. Refresh Token реюз
- Самый стабильный метод
- Присвоение существующего refresh token новой сессии
- Не требует повторной аутентификации
- Работает ~200 дней до истечения

### Steam Guard типы

| Тип | Описание | Автоматизация |
|-----|----------|---------------|
| EmailCode | Код на email | Мониторинг почты + RPC передача |
| DeviceCode | TOTP из мобильного аутентификатора | Генерация из shared_secret |
| DeviceConfirmation | Подтверждение в мобильном приложении | LoginApprover с shared_secret |
| EmailConfirmation | Ссылка на email | Мониторинг почты |

### Polling механизм
**Protobuf:** `CAuthentication_PollAuthSessionStatus_Request`
- После отправки guard кода клиент опрашивает статус
- При успехе возвращает refresh_token и access_token
- Таймаут настраивается через `loginTimeout`

### Ключевые файлы Steam на Linux
- `~/.steam/steam/config/config.vdf` — OAuth токены, refresh tokens
- `~/.steam/steam/config/loginusers.vdf` — список пользователей с "Remember Me"
- `~/.steam/registry.vdf` — реестр Steam (баг с потерей паролей)
- `~/.steam/steam/ssfn*` — sentry файлы для Machine Auth

### Источники
- https://github.com/DoctorMcKay/node-steam-session — полная реализация Steam auth
- https://github.com/DoctorMcKay/node-steam-user — эмуляция Steam клиента
- https://github.com/SteamDatabase/SteamTracking — protobuf определения
- https://partner.steamgames.com/doc/features/auth — официальная документация
- https://wiki.archlinux.org/title/Steam/Troubleshooting — Linux конфигурация
