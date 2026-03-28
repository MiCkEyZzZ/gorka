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

# Property-тесты (отдельный файл)
test-prop:
    cargo test --test bit_property

# Интеграционные тесты (tests/ файлы)
test-integration:
    cargo test --test test_bitstream

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
