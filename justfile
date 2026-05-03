export PATH := env_var('HOME') + "/.cargo/bin:" + env_var('PATH')

default: test

build:
    cargo build

test: pre-test
    cargo test

pre-test: build-example-wasm

build-example-wasm:
    cargo build \
        --manifest-path example-module/Cargo.toml \
        --target wasm32-unknown-unknown \
        --release

install:
    cargo install --path .

check: fmt lint test audit doc

fmt:
    cargo fmt --check

lint:
    cargo clippy -- -D warnings

audit:
    cargo audit
    cargo deny check
    cargo machete

doc:
    cargo doc --no-deps -D warnings

ci: check
    cargo build --release
