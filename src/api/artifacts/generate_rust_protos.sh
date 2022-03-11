#!/bin/bash

bazel build //src/api/artifacts:artifacts_api_rust_proto
rm -rf src/api/artifacts/generated
mkdir src/api/artifacts/generated
cp -r bazel-bin/src/api/artifacts/artifacts_api_rust_proto.proto.rust/*.rs src/api/artifacts/generated