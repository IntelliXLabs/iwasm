#!/bin/sh

cd iwasm/lib_iwasm
cargo build --release --target x86_64-unknown-linux-musl --example muslc
cp target/x86_64-unknown-linux-musl/release/examples/libmuslc.a ./../api/lib_iwasm_muslc.x86_64.a