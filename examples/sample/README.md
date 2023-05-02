# Hashira x Deno

A hashira template using `Deno`.

## Prerequisites

- Deno:
  - <https://deno.com/manual@v1.33.1/getting_started/installation>
        - `cargo install deno --locked`

- Cargo Make *(optional)*:
  - <https://github.com/sagiegurari/cargo-make>
        - `cargo install --force cargo-make`

## How to run

- 1. Build and Run
  - `cargo make run`
- 2. **Only** build:
  - `cargo make build`
- 3. **Only** run:
  - `cargo make deno-run`

> For a production build use: `cargo make --profile prod build`

## Run without `Cargo make`

The commands with arguments specified in the configuration file are:

1. Build the server wasm:
    - `cargo build --target wasm32-unknown-unknown`

2. Build the wasm bundle:
    - `wasm-bindgen target/wasm32-unknown-unknown/debug/{{crate_name}}.wasm --target deno --out-dir build/`

3. Build the client:
    - `cargo build --target wasm32-unknown-unknown --features client`

4. Bundle the client bundle:
    - `wasm-bindgen target/wasm32-unknown-unknown/debug/{{crate_name}}.wasm --target web --out-dir public/ --no-typescript`

5. Starts the deno server:
    - `deno run --allow-read --allow-net --allow-env src/index.ts`
    - Is required to set these environment variables:
       - `HASHIRA_WASM_LIB={{crate_name}}`
       - `HASHIRA_STATIC_DIR=static/`
       - `HASHIRA_HOST=127.0.0.1`
       - `HASHIRA_PORT=5000`

Note that `{{crate_name}}` needs to be replaced with the actual name of the Rust crate being built.
