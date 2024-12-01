#!/bin/bash

cd iwasm/lib_iwasm
cargo build --release
cp target/release/deps/libruntime.so ./../api