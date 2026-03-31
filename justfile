# The Gorka Dev Commands

# Форматирование Rust-кода
fmt:
    cargo fmt

# Форматирование всех Cargo.toml через Taplo
fmt-toml:
    taplo fmt

# Форматирование всего проекта (Rust + TOML)
fmt-all: fmt fmt-toml

# Проверка форматирования без изменения файлов (CI-safe)
fmt-check:
    cargo fmt -- --check
    taplo fmt --check

# Clippy: все таргеты и фичи, warnings -> errors
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Только юнит-тесты (lib), без интеграционных и property
test-fast:
    cargo test --lib

# Property-тесты: bit_property.rs
test-prop-bit:
    cargo test --test bit_property

# Property-тесты: codec_property.rs
test-prop-codec:
    cargo test --test codec_property

# Property-тесты: оба файла подряд
test-prop: test-prop-bit test-prop-codec

# Интеграционные тесты (tests/ файлы)
test-integration:
    cargo test --tests

# Полный запуск через nextest (рекомендуется для CI)
test-next:
    cargo nextest run

# Бенчи
bench:
    cargo bench

# Очистка
clean:
    cargo clean

# CI: проверяет форматирование, линтер, тесты — файлы не меняет
check: fmt-check lint test-next

# Dev shortcut: формат + все проверки
dev: fmt-all check
