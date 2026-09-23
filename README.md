# opencode-rs

A Rust reimplementation of the **opencode server** — HTTP API, agent loop, tools, and
event stream — designed to be wire-compatible with existing opencode clients
(TUI, desktop, web, SDK).

The client stays JavaScript. This project replaces only the server, so a running
`opencode-rs serve` can be used with `opencode attach`, `opencode run --attach`, and
the generated SDKs.

**Status:** pre-alpha. Contract fixtures extracted; health + event subscription slices
in progress. See [`docs/PLAN.md`](docs/PLAN.md) for the roadmap.

## Why

The upstream server is a Bun/TypeScript process whose baseline footprint is heavy
(multi-GB RSS, 40+ threads, single-core spin at startup). This project targets a
single static binary with predictable memory, while preserving the HTTP contract so
the ecosystem keeps working.

## Layout

```
crates/
  schema/    wire types + event manifest (contract-driven)
  protocol/  HTTP route groups, error taxonomy, payloads
  core/      service traits + storage (in-memory first, SQLite later)
  llm/       provider abstraction + streaming adapters
  server/    axum router, handlers, middleware, SSE
  client/    Rust HTTP + SSE client (used by conformance tests and the CLI)
  cli/       `opencode-rs` binary (serve / attach / run)
tests/fixtures/   contract fixtures extracted from the reference implementation
```

## Build

```sh
cargo build
cargo test
```

## Contract-first

Behaviour is pinned by fixtures extracted from the reference server
(`scripts/extract_contract.py`) and by conformance tests run against a live
reference instance. See [`docs/CONTRACT.md`](docs/CONTRACT.md).

## License

MIT
