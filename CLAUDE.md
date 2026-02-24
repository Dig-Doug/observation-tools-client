# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Observation Tools is a developer data inspection toolkit designed to export, visualize, and inspect data from anywhere
in a program. It allows developers to instrument their code with observations and view them through a web UI.

The system consists of three major components:

- **Client library**: Instruments code to log observations (Rust/TypeScript)
- **Server**: HTTP API backend that collects, stores, indexes, and serves observations
- **Web UI**: Interface for searching and visualizing observations (served from server)

## Architecture

Cargo workspace with 4 crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `observation-tools-client` | lib (cdylib + rlib) | Rust client + Node.js native module (NAPI) |
| `observation-tools-server` | bin + lib | HTTP API, web UI, storage (sled + object_store) |
| `observation-tools-shared` | lib | Core types shared across crates (WASM-compatible) |
| `observation-tools-macros` | proc-macro | `observe!()` and `group!()` macros |

Client has optional feature flags: `axum` (middleware), `tracing` (subscriber integration).

## Commands

```bash
cargo build --workspace --all-features    # Build
cargo run --bin observation-tools -- serve # Run server (default port 3000)
cargo test --workspace --all-features     # Rust tests
```

For UI tests, formatting, and contribution setup, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Core Concepts

- **Execution**: The root scope for data collection. All observations are associated with one execution. A program may
  create multiple executions (e.g., per HTTP request).
- **Observation**: A single piece of collected data with metadata (source info, payload, labels, parent span, etc.)
- **Labels**: Hierarchical groupings using path convention (e.g., `api/request/headers`)

## Coding Guidelines

- Always use workspace imports for dependencies to ensure consistency across crates.
- Never use `unwrap()`. Always handle or propagate errors.
- Only comment code when necessary. Prefer self-documenting code.

## Environment

| Variable | Purpose |
|----------|---------|
| `PORT` | Server listen port (default: 3000) |
| `SERVER_URL` | Point integration tests at an external server instead of spawning one |
| `RUST_LOG` | Logging filter (default set in `.cargo/config.toml`) |

## Testing

- Rust integration tests are in `crates/observation-tools-client/tests/` with a shared `TestServer` helper
  that auto-spawns a server on a random port (or uses `SERVER_URL` for an external server).
- Playwright E2E tests are in `tests/` and test the web UI in Chromium.
- Always use `--all-features` when running `cargo test` to cover axum/tracing integrations.
