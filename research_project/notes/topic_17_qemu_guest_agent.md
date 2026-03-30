# Topic 17: QEMU Guest Agent — управление VM снаружи

## Что такое QEMU Guest Agent (qemu-ga)
Лёгкий демон (qemu-ga), работающий внутри гостевой ОС, общающийся с хостом через virtio-serial канал. Позволяет хосту выполнять команды внутри VM без сетевого подключения.

## Ключевые команды

### Выполнение команд
- **guest-exec** (QEMU 2.5+): Выполнение произвольных команд в гостевой ОС
  - Параметры: `path`, `arg[]`, `env[]`, `input-data` (base64), `capture-output`
  - Возвращает: PID процесса
- **guest-exec-status**: Проверка статуса процесса
  - Возвращает: exit code, stdout/stderr (base64), truncated флаги

### Файловые операции
- **guest-file-open**: Открытие файла в госте (path, mode → handle)
- **guest-file-write**: Запись base64-данных в файл (handle, data)
- **guest-file-read**: Чтение из файла (handle, count → base64 data), лимит 48 MB
- **guest-file-close**: Закрытие файла
- **guest-file-flush**: Синхронизация буферов
- **guest-file-seek**: Перемещение позиции (set/cur/end)

### Сетевая информация
- **guest-network-get-interfaces**: IP, MAC, netmask всех интерфейсов
- **guest-network-get-route** (QEMU 9.1+): Таблица маршрутизации

### Системные команды
- **guest-ping**: Проверка доступности агента
- **guest-info**: Версия агента и список поддерживаемых команд
- **guest-shutdown**: halt/powerdown/reboot

## Как настроить в libvirt XML

```xml
<channel type='unix'>
  <source mode='bind' path='/var/lib/libvirt/qemu/channel/target/domain-{name}/org.qemu.guest_agent.0'/>
  <target type='virtio' name='org.qemu.guest_agent.0'/>
</channel>
```

## Взаимодействие с хоста

```bash
# Через virsh
virsh qemu-agent-command <domain> '{"execute":"guest-ping"}'

# Выполнение команды
virsh qemu-agent-command <domain> '{
  "execute": "guest-exec",
  "arguments": {
    "path": "/bin/bash",
    "arg": ["-c", "steam -login user pass"],
    "capture-output": "separated"
  }
}'

# Запись файла
virsh qemu-agent-command <domain> '{
  "execute": "guest-file-open",
  "arguments": {"path": "/home/user/.steam/steam/config/config.vdf", "mode": "w"}
}'
# → {"return": {"handle": 1}}
virsh qemu-agent-command <domain> '{
  "execute": "guest-file-write",
  "arguments": {"handle": 1, "buf-b64": "<base64_content>"}
}'
```

## Преимущества
- Не требует сетевого подключения к VM (работает через virtio-serial)
- Может выполнять команды от любого пользователя (работает как root)
- Файловые операции без необходимости монтирования диска VM
- Стандартный инструмент, поддерживается всеми Linux дистрибутивами

## Ограничения
- Требует установки qemu-ga внутри VM
- Однонаправленная инициация (только хост → гость)
- Нет механизма callback/event из гостя в хост
- Асинхронные операции требуют polling (guest-exec-status)
- Нет streaming/realtime вывода

## Источники
- https://www.qemu.org/docs/master/interop/qemu-ga-ref.html — официальная документация
- https://www.qemu.org/docs/master/interop/qemu-qmp-ref.html — QMP reference
