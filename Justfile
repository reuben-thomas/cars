linux_target := "x86_64-unknown-linux-gnu"
wasm_target := "wasm32-unknown-unknown"
web_port := "8000"

default:
    @just --list

build *args:
    cargo build --verbose {{ args }}

run *args:
    cargo run {{ args }}

build-web *args:
    cargo build --target {{ wasm_target }} {{ args }}
    wasm-bindgen \
        --target web \
        --no-typescript \
        --out-dir target/web \
        target/{{ wasm_target }}/{{ if args =~ '(^|\s)--release(\s|$)' { "release" } else { "debug" } }}/cars.wasm
    cp web/index.html web/favicon.svg target/web/

run-web *args: (build-web args)
    python3 -m http.server --directory target/web {{ web_port }}

#
# CI
#

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

lint *args:
    cargo clippy --all-targets {{ args }} -- -D warnings

precommit:
    just fmt-check
    just build --target {{ linux_target }}
    just lint --target {{ linux_target }}
    just build --target {{ wasm_target }}
    just lint --target {{ wasm_target }}
