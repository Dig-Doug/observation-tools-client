#!/bin/bash

bazel build //src/proto:artifacts_api_rust_proto
rm -rf src/client/rust/generated
mkdir src/client/rust/generated
cp -r bazel-bin/src/proto/artifacts_api_rust_proto.proto.rust/*.rs src/client/rust/generated
sed -i 's/use uuid/use self::uuid/' src/client/rust/generated/lib.rs
mv src/client/rust/generated/lib.rs src/client/rust/generated/mod.rs