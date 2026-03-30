# Topic 13: Управление XML-конфигами VM

## Сравнение подходов

| Подход | Для проекта |
|---|---|
| `virt` crate (Rust libvirt bindings) | ✅ Рекомендуется — прямой API, Rust error handling |
| Terraform + libvirt provider | ❌ Overkill для динамического управления |
| virsh (CLI) | ⚠️ Shell вызовы — хрупко |
| Ручная генерация XML | ⚠️ Нет валидации |

## Рекомендация: `virt` crate

Rust bindings для libvirt (https://crates.io/crates/virt).

### API
```rust
use virt::connect::Connect;
use virt::domain::Domain;

let conn = Connect::open(Some("qemu:///system"))?;
let domain = Domain::define_xml(&conn, &xml)?;
domain.create()?;
domain.shutdown()?;
```

### Модули
- `domain` — создание/управление VM
- `network` — виртуальные сети
- `storage_pool` / `storage_vol` — хранилище
- `nodedev` — устройства хоста

### Требования
- `apt install libvirt-dev` на хосте
- `virt = "0.4"` в Cargo.toml

## XML-генерация
Генерировать через `format!()` или шаблонизатор (askama/tera).
Ключевые параметры: UUID, MAC, SMBIOS, CPU/RAM, disk overlay, virtiofs, VNC.

## Источники
- virt crate: https://docs.rs/virt/latest/virt/
- libvirt Domain XML: https://libvirt.org/formatdomain.html
