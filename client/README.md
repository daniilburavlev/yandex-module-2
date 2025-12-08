<div>
    <div align="center">
        <h1>Клиент</h1>
    </div>
</div>

___
# Описание
CLI утилита, позволяющая подписываться на котировки для определенных акций

## Локальный запуск
```bash
RUST_LOG=info cargo run --release --package client -- \
  --remote-addr 127.0.0.1:8080 \
  --local-addr 127.0.0.1:9090 \
  --tickers resources/sub.txt
```

## Помощь
```bash
 cargo run --release --package client -- --help
```