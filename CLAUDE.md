# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Observation Tools is a developer data inspection toolkit designed to export, visualize, and inspect data from anywhere
in a program. It allows developers to instrument their code with observations and view them through a web UI.

The system consists of three major components:

- **Client library**: Instruments code to log observations (Rust/TypeScript)
- **Server**: HTTP API backend that collects, stores, indexes, and serves observations
- **Web UI**: Interface for searching and visualizing observations (served from server)

For more details refer to [Design](docs/design.md)

## Core Concepts

- **Execution**: The root scope for data collection. All observations are associated with one execution. A program may
  create multiple executions (e.g., per HTTP request).
- **Observation**: A single piece of collected data with metadata (source info, payload, labels, parent span, etc.)
- **Labels**: Hierarchical groupings using path convention (e.g., `api/request/headers`)

## Coding Guidelines

- Always use workspace imports for dependencies to ensure consistency across crates.