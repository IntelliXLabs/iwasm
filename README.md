# WASM runtime

This package contains Go binding for the WASM runtime implemented in Rust.

To build the code, first install the Rust toolchain and then run:

```bash
cargo build --release
```

To run unit test:

```bash
LD_LIBRARY_PATH=target/release go test .
```
