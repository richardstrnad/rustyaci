alias b := build
alias l := lint
alias v := verify

build:
  @echo "Building..."
  cargo build --release
  @echo "Done."

@verify: lint test

test:
  cargo test

lint:
  @echo "Running fmt..."
  cargo fmt --all -- --check
  @echo "Running clippy..."
  cargo clippy

fmt:
  cargo fmt
