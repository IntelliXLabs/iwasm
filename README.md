# WASM runtime

This package contains Go binding for the WASM runtime implemented in Rust.

To build the code, first install the Rust toolchain and then run:

```bash
make build
# equivalent to
# cargo build --release && mkdir -p lib && cp target/release/libruntime* lib/
```

To run unit test:

```bash
make test
# equivalent to
# LD_LIBRARY_PATH=../lib go test .
```

To run the main program:

```bash
make run
# equivalent to
# LD_LIBRARY_PATH=../lib go run main.go
```

Compile runtime in docker

```bash
make docker-image-linux
make release-linux
```