<div>
    <div align="center">
        <h1>Библиотека со структурой котировки</h1>
    </div>
</div>

___

# Описание

Библиотека, обеспечивающая парсинг и сериализацию котировок, генерацию случайных данных.

## Пример использования

```rust
use quotes::StockQuote;

fn main() {
    let mut stock = StockQuote::new("AAPL", 180, 3000000);
    stock.update(200, 3500000);
}
```
