<div align="center">
    <h1>Проектная работа №2 курса "Rust для действующих разработчиков"</h1>
</div>

___
# Структура проекта
- [Библиотека с общей структурой акции](/quotes/README.md)
- [Сервер генерации биржевых сводок](/server/README.md)
- [Клиент для получения биржевых сводок по подписке](/client/README.md)
- [Список компаний, по которым генерируются данные](/resources/tickers.txt)


## Настройка локального окружения

1. Установка RUST на Linux/MacOS
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
```bash
# Проверить готовность к работе
rustc --version && \
cargo --version
```

2. Клонировать проект
```bash
git clone https://github.com/daniilburavlev/yandex-module-2.git
```

3. Сборка проекта
```bash
cargo build --release
```

4. Установка
```bash
cargo install
```

5. Тестирование
```bash
cargo test
```