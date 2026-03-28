# The Gorka Dev Commands

# Форматирование Rust-кода
fmt:
    cargo fmt

# Форматирование всех Cargo.toml через Taplo
fmt-toml:
    taplo fmt

# Форматирование всего проекта (Rust + TOML)
fmt-all: fmt fmt-toml

# Clippy: все таргеты и фичи, warnings -> errors
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Все тесты через cargo test
test:
    cargo test

# Быстрые тесты без property-тестов
test-fast:
    cargo test --lib

# Property-тесты (отдельный файл)
test-prop:
    cargo test --test bit_property

# Интеграционные тесты (tests/ файлы)
test-integration:
    cargo test --test test_bitstream

# Полный запуск через nextest (рекомендуется для CI)
test-next:
    cargo nextest run

# Полный прогон проверок
check: fmt-all lint test-next

# Бенчи
bench:
    cargo bench

# Очистка
clean:
    cargo clean

# Dev shortcut: всё сразу (формат + линтер + тесты)
dev: fmt-all lint test-next
