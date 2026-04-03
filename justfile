# The Gorka Dev Commands

# Форматирование Rust-кода
fmt:
    cargo fmt --all

# Форматирование всех Cargo.toml через Taplo
fmt-toml:
    taplo fmt

# Форматирование всего проекта (Rust + TOML)
fmt-all: fmt fmt-toml

# Проверка форматирования без изменения файлов (CI-safe)
fmt-check:
    cargo fmt --all -- --check
    taplo fmt --check

# Clippy: все таргеты и фичи, warnings -> errors
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Проверка deny.toml (локально нужно через '--' перед аргументами)
deny:
    cargo deny check all

# Тесты через cargo test
test:
    cargo test --all-features

# Тесты через nextest
test-next:
    cargo nextest run --all-features

# Проверка no_std под embedded target
no-std:
    rustup target add thumbv7em-none-eabihf
    cargo check --target thumbv7em-none-eabihf --no-default-features --features alloc --lib

# Документация
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# MSRV check
msrv:
    cargo +1.75.0 check --all-features

# Бенчи
bench:
    cargo bench

# Очистка
clean:
    cargo clean

# Локальный аналог CI перед пушем

# Сначала форматируем Rust + TOML, чтобы fmt-check точно проходил
ci-local:
    just fmt-all
    just fmt-check
    just lint
    just deny
    just test
    just test-next
    just no-std
    just doc

# Dev shortcut: формат + все проверки
dev: fmt-all ci-local
