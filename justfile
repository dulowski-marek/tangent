export PATH := env_var('HOME') + "/.cargo/bin:" + env_var('PATH')

default: test

build:
    cargo build

check:
    cargo check

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
