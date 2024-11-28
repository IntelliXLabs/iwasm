#!/bin/bash

cd iwasm/lib_iwasm
cargo build --release
cp target/release/deps/lib_iwasm.so ./../api