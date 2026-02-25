# Contribution guidelines

## Running tests

We have two test suites: rust-based client+server integration tests and a playwright based UI test suite. By default,
every test will start up its own server instance.

- You can use `SERVER_URL` to point the tests to a running server instead of starting a new one,
  though keep in mind many tests assume a clean server state.

### Rust tests

```bash
cargo test --workspace --all-features
```

### UI tests

To run the tests, you must build the NodeJS client library so the test can import it.

```bash
pnpm --dir crates/observation-tools-client install
pnpm --dir crates/observation-tools-client build:debug
pnpm --dir tests install
pnpm --dir tests run test
```

The test suite uses [Playwright](http://playwright.dev/). You can use all of its debugging tools, e.g. open the
inspector:

```bash
cd tests
pnpm playwright test --ui
```

## Formatting

```bash
cargo +nightly fmt
pnpm dlx prettier --write .
```

