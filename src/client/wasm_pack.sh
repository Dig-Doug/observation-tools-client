#!/bin/bash
wasm-pack build src/client $@ --scope observation-tools --target nodejs --features wasm --no-default-features
sed -i 's/observation-tools\/observation-tools/observation-tools\/client/' src/client/pkg/package.json
