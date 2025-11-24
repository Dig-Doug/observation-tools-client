# observation-tools-client

[![Crates.io](https://img.shields.io/crates/v/observation-tools)](https://crates.io/crates/observation-tools) [![docs.rs](https://img.shields.io/docsrs/observation-tools)](https://docs.rs/observation-tools) [![npm (scoped)](https://img.shields.io/npm/v/%40observation-tools/client)](https://www.npmjs.com/package/@observation-tools/client)

Export, visualize, and inspect data from anywhere in your program.

## Getting started

Run the server:

```bash
cargo run --bin observation-tools -- serve
```

## Building from source

## Contributing

### Running tests

We have two test suites: rust-based client+server integration tests and a playwright based UI test suite. By default,
every test will start up its own server instance.

- You can use `SERVER_URL` to point the tests to a running server instead of starting a new one,
  though keep in mind many tests assume a clean server state.

#### Rust tests

```bash
cargo test
```

#### UI tests

To run the tests, you must build the NodeJS client library so the test can import it.

```bash
pnpm --dir crates/observation-tools-client install
pnpm --dir crates/observation-tools-client build
pnpm --dir tests install
pnpm --dir tests run test
```

The test suite uses [Playwright](http://playwright.dev/). You can use all of its debugging tools, e.g. open the
inspector:

```bash
cd tests
pnpm playwright test --ui
```

### Formatting

```bash
cargo +nightly fmt
pnpm dlx prettier --write .
```

## License

All code, except the client library, is distributed under the GNU AGPLv3 license. The client library is licensed under
the Apache-2.0 license to allow usage in commercial projects.
