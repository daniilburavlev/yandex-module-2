<div>
    <div align="center">
        <h1>Генератор котировок</h1>
    </div>
</div>

___
# Описание
Сервер, позволяющий запускать генерацию котировок, обрабатывать подписки от клиентов

## Локальный запуск
```bash
RUST_LOG=info cargo run --release --package client -- \
  --tcp-port 8080 \
  --udp-port 9090 \
  --tickers-path resources/sub.txt
```

## Помощь
```bash
cargo run --release --package server -- --help
```