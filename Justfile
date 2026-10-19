host_target := `rustc -vV | sed -n 's/^host: //p'`

default:
    @just --list

build target=host_target:
    cargo build --verbose --target {{ target }}

run *args:
    cargo run -- {{ args }}

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint-check target=host_target:
    cargo clippy --all-targets --target {{ target }} -- -D warnings

precommit target=host_target: fmt-check (build target) (lint-check target)
