# observation-tools-client

[![Crates.io](https://img.shields.io/crates/v/observation-tools)](https://crates.io/crates/observation-tools) [![docs.rs](https://img.shields.io/docsrs/observation-tools)](https://docs.rs/observation-tools) [![npm (scoped)](https://img.shields.io/npm/v/%40observation-tools/client)](https://www.npmjs.com/package/@observation-tools/client)

Export, visualize, and inspect data from anywhere in your program.

## Building from source

### Generating an OpenAPI spec

```bash
cargo run --bin observation-tools -- export-openapi --output ./crates/observation-tools-client/openapi.json
```

## Contributing

### Formatting

```bash
cargo +nightly fmt
pnpm dlx prettier --write .
```

## License

All code, except the client library, is distributed under the GNU AGPLv3 license. The client library is licensed under
the Apache-2.0 license to allow usage in commercial projects.
