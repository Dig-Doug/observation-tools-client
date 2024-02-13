#!/bin/bash
wasm-pack build src/client/rust $1 --scope observation-tools --target nodejs --features wasm
sed -i 's/observation-tools\/observation-tools/observation-tools\/client/' src/client/rust/pkg/package.json
