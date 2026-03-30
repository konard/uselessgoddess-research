# Topic 12: Вход в Steam — мульти-аккаунт стратегия

## Проблема
Каждая VM должна запускать разные Steam аккаунты. Нужна автоматизация входа.

## Методы

### 1. `-login user pass` (Steam CLI) — ❌ НЕ РАБОТАЕТ
Сломан с mid-2023. Valve убрала поддержку.

### 2. Предзаполненная сессия — ✅ РЕКОМЕНДУЕТСЯ
Один раз залогиниться с "Remember Me", сохранить файлы:
- `~/.steam/steam/config/config.vdf` — токены
- `~/.steam/steam/config/loginusers.vdf` — список пользователей

При создании VM — подставить файлы в overlay. Steam автоматически использует сохранённую сессию.

### 3. SteamCMD — для скачивания CS2
`steamcmd +login user pass +app_update 730 +quit`
Сохраняет cached credentials. Подходит для bootstrap.

### 4. VNC автоматизация — fallback
Хрупкая, ломается при обновлениях Steam UI. Только как последний вариант.

## Steam Guard
- Email code: сервер мониторит почту, передаёт через RPC
- TOTP: генерация из shared_secret (crate steam-totp)
- Отключить: не рекомендуется

## Workflow
1. Сервер → воркер: StartAccount { account_id, session_files }
2. Воркер записывает session files в VM overlay
3. VM загружается, Steam подхватывает сессию
4. При Steam Guard → код через RPC
5. CS2 запускается автоматически

## Обновление сессий
- Batch provisioning раз в неделю
- Encrypted credential store на сервере
- SteamCMD для начальной авторизации

## Источники
- Steam config files: `~/.steam/steam/config/`
- loginusers.vdf формат: VDF (Valve Data Format)
