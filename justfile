set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

clippy:
    cargo clippy -- -D warnings

build-release:
    cargo build --release

run-release: build-release
    ./target/release/find_my_shit

run:
    cargo run

test:
    cargo test

help:
    @echo "Available commands:"
    @echo "  clippy        Run clippy with warnings as errors"
    @echo "  build-release Build the release version"
    @echo "  run-release   Build and run the release version"
    @echo "  run           Run in development mode"
