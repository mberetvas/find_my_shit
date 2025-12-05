
clippy:
    cargo clippy -- -D warnings

build-release:
    cargo build --release

run-release: build-release
    ./target/release/find_my_shit

run:
    cargo run
