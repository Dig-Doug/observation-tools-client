#!/bin/bash
set -e
rm -rf ./pkg
wasm-pack build --target nodejs --dev --features wasm