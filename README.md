# CBR USD Exchange Rate Parser

Парсер курса доллара США с сайта Центрального Банка России (cbr.ru).

## Требования

- Rust (установка: https://rustup.rs/)

## Установка Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## Сборка и запуск

```bash
# Сборка
cargo build --release

# Запуск
cargo run --release
```

## Использование

По умолчанию парсер загружает курсы доллара за период 23.01.2022 - 30.01.2024.

Для изменения периода отредактируйте переменные `from_date` и `to_date` в `src/main.rs`:

```rust
let from_date = "23.01.2022";  // формат: DD.MM.YYYY
let to_date = "30.01.2024";
```

## Результат

После выполнения создаётся файл `usd_rates.csv` с колонками:
- `date` — дата (YYYY-MM-DD)
- `units` — количество единиц валюты
- `rate` — курс в рублях

Пример содержимого:
```csv
date,units,rate
2022-01-25,1,77.3649
2022-01-26,1,78.6422
2022-01-27,1,78.9437
...
```
