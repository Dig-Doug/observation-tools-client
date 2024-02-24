#!/bin/bash
wasm-pack build src/client/rust $@ --scope observation-tools --target nodejs --features wasm --no-default-features
sed -i 's/observation-tools\/observation-tools/observation-tools\/client/' src/client/rust/pkg/package.json
