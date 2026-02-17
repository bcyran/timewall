default: test lint check-format

test:
  cargo test  

lint:
  cargo clippy --all-targets

check-format:
 cargo fmt --all -- --check

format:
 cargo fmt --all

coverage:
  cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
