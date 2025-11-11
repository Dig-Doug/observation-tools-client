# Observation Tools Integration Tests

This directory contains Playwright-based integration tests that test the observation-tools client library and server together.

## Setup

1. Install dependencies:

```bash
cd tests
npm install
npx playwright install chromium
```

2. Build the server (from project root):

```bash
cargo build --bin observation-tools
```

## Running Tests

Run all tests:

```bash
npm test
```

Run tests in UI mode (interactive):

```bash
npm run test:ui
```

Run tests in debug mode:

```bash
npm run test:debug
```

Run a specific test file:

```bash
npx playwright test specs/01-empty-execution.spec.ts
```

Run tests in headed mode (see browser):

```bash
npx playwright test --headed
```

## Test Structure

- `specs/` - Test specification files
  - `01-empty-execution.spec.ts` - Tests creating empty executions with metadata
  - `02-execution-with-observations.spec.ts` - Tests executions with observations
  - `03-execution-pagination.spec.ts` - Tests execution list pagination (357 items)
  - `04-observation-pagination.spec.ts` - Tests observation list pagination (396 items)
  - `05-auto-refresh.spec.ts` - Tests auto-refresh functionality

- `helpers/` - Test helper utilities
  - `server.ts` - Helper to spawn server instances on random ports
  - `client.ts` - HTTP client for creating test data

## How It Works

Each test:

1. Spawns a fresh server instance on a random port
2. Uses a temporary data directory
3. Creates test data via the HTTP API
4. Uses Playwright to verify the UI displays the data correctly
5. Cleans up the server and temp directory after completion

## Environment Variables

- `CI` - Set to enable CI-specific behavior (retries, single worker)

## Troubleshooting

If tests fail to start:

- Ensure the server binary is built: `cargo build --bin observation-tools`
- Check that ports are available (tests use random ports)
- Increase timeout in `playwright.config.ts` if server startup is slow

If tests are flaky:

- Check the HTML report: `npx playwright show-report`
- Run with `--headed` to see what's happening
- Add more specific waits or selectors in the test code

## Notes

- Each test suite spawns its own isolated server instance
- Tests run in parallel by default (can be disabled with `--workers=1`)
- Server logs are captured during test execution
- Temporary data directories are automatically cleaned up
