# Topic 14: Шаринг CS2 между VM

## Проблема
CS2 ~35 GB. При 8 VM = 280 GB без шаринга.

## Сравнение

| Метод | Производительность | Простота |
|---|---|---|
| **virtiofs** | **Нативная** | **Простая** |
| 9p (virtio-9p) | Медленная | Простая |
| NFS | Сетевая | Средняя |
| qcow2 backing store | Дисковая | Простая |

## virtiofs — рекомендуемое решение

### Принцип
Хост шарит `/opt/cs2-shared/` во все VM через virtiofs.
Одна VM скачивает CS2 → все видят обновления.

### libvirt XML
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

### В гостевой VM
```bash
mount -t virtiofs cs2 /opt/cs2
```

### Преимущества
- Одна копия на диске
- Моментальные обновления
- Нативная производительность
- DAX (Direct Access) — ещё быстрее
- virtiofsd-rs — Rust daemon

### Workflow обновления CS2
1. Одна VM запускает `steamcmd +app_update 730`
2. Все VM видят обновление через virtiofs
3. Перезапуск CS2 в каждой VM

## Источники
- virtiofs: https://virtio-fs.gitlab.io/
- virtiofs + libvirt: https://libvirt.org/kbase/virtiofs.html
- virtiofsd-rs: https://gitlab.com/virtio-fs/virtiofsd
