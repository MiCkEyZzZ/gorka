# запуск тестов
test:
    cargo test

# быстрые тесты без proptest (если захочешь разделить)
test-fast:
    cargo test --lib

# property тесты отдельно
test-prop:
    cargo test --test property

# форматирование
fmt:
    cargo fmt

# линтер
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# полный прогон
check: fmt lint test

# бенчи (на будущее)
bench:
    cargo bench
