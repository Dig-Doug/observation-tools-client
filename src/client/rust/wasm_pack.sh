#!/bin/bash
wasm-pack build --scope observation-tools --target nodejs --dev --features wasm
sed -i 's/observation-tools\/observation-tools-client/observation-tools\/client/' pkg/package.json
