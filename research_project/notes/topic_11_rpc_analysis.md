# Topic 11: Анализ кастомного RPC (shai)

## Обзор

shai — кастомный Rust RPC фреймворк с zero-copy сериализацией (rkyv) и QUIC транспортом (quinn).

## Архитектура

- **Workspace:** 2 crate — `shai` (core) и `shai-macros` (proc macros)
- **Edition:** Rust 2024
- **Wire format:** 24-byte header (little-endian) + rkyv payload
- **Transport:** QUIC (quinn) + Local (mpsc channels для тестов)

## Ключевые абстракции

### Message trait
```rust
trait Message: Archive + Serialize {
    const ID: MessageId;  // u16, бит 15 = response flag
    type Response: Message;
}
```

### Router (tower::Service)
- FxHashMap<MessageId, Arc<dyn ErasedHandler<S>>>
- Generic state type S
- Поддержка 0-3 extractors через макро

### Extractors (паттерн axum)
- `State<S>` — состояние приложения
- `Archive<M>` — zero-copy доступ к payload
- `Rpc<M>` — полная десериализация
- `Unchecked<M>` — unsafe zero-copy (trusted sources)
- `Extension<T>` — per-peer extensions
- `Peer` — информация о пире

### Per-peer Extensions
- Arc<RwLock<HashMap<TypeId, Box<dyn AnyClone>>>>
- Идеально для auth context (WorkerId, VmId)

## Применение в проекте

### Message types (предложенные)
- `StartAccount` / `StopAccount` — управление VM
- `WorkerHeartbeat` — heartbeat
- `AccountStatus` — статус аккаунта
- `SteamGuardCode` — relay Steam Guard кода

### Преимущества для проекта
1. Zero-copy — минимальный overhead для status updates
2. QUIC — мультиплексирование, TLS 1.3 из коробки
3. 16-byte trace ID — трассировка через всю систему
4. Per-peer extensions — auth без middleware
5. tower::Service — composable middleware

## Бенчмарки
- `examples/quic-load.rs`: 100K concurrent streams
- `benches/router.rs`: zero-copy unary RPC throughput
- `benches/frame_encode.rs`: payload 0 — 1MB

## Источники
- Исходный код: `/rpc/crates/shai/`
- rkyv: https://rkyv.org/
- quinn: https://github.com/quinn-rs/quinn
