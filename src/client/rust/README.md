# Development guide

## Regenerating proto files

Our internal API is defined using [Protocol Buffers](https://developers.google.com/protocol-buffers). We currently
generate the Rust code using [Bazel](https://bazel.build/) and include the generated code in the repository so it can be packaged with [cargo](https://doc.rust-lang.org/cargo/).

To regenerate the proto files, install `bazel` and then run the following command from the repository root:

```sh
./src/proto/generate_rust_protos.sh
```

## Previewing crate documentation

Regenerate the HTML on local changes:

```sh
cargo watch --ignore "_tmp*" -s 'cargo doc --lib' 
```

Serve the HTML files with automatic refresh:

```sh
pnpm dlx browser-sync start --serveStatic target/doc --server target/doc --directory --files target/doc/** --no-open
```

## Building the JS package

```sh
./src/client/rust/wasm_pack.sh
```

### Test examples with local changes

```shell
pnpm link ./src/client/rust/pkg

# Undo with:
pnpm unlink
```

## Running the doc tests

```
RUSTDOCFLAGS="-Z unstable-options --nocapture" cargo +nightly test --doc
```