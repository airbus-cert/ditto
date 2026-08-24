# WASM build for ditto

Run ditto in your browser!

## Requirements

### With devenv

If you use the devenv setup, you just have to install `jco`.
There is a `justfile` recipe that handle this, and that is run before each over just command.

You can trigger it manually with:
```bash
just init
```

### Without devenv

Setup:
```bash
rustup target add "wasm32-wasip1"
rustup target add "wasm32-wasip2"
cargo install cargo-component --locked
npm install @bytecodealliance/jco
```

Also install [`just`](https://github.com/casey/just) for you system to run build commands.

## Build step

Just use the `justfile` recipe:

```bash
just build
```

The project is build with wasm32-wasip2, then transpiled to JS thanks to `jco`.
JS dependencies to `preview2-shim` components are locally loaded.

For documentation on building the project on WASM, check [this article](https://benw.is/posts/compiling-rust-to-wasi).

## Usage

```bash
just serve
```
