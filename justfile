default: test lint check-format

clippy-args := "\
-D warnings \
-D clippy::complexity \
-D clippy::correctness \
-D clippy::nursery \
-A clippy::option_if_let_else \
-D clippy::pedantic \
-A clippy::module_name_repetitions \
-D clippy::perf \
-D clippy::style \
-D clippy::suspicious"

test:
  cargo test  

lint:
  cargo clippy -- {{ clippy-args }}

check-format:
 cargo fmt --all -- --check

format:
 cargo fmt --all

coverage:
  cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
