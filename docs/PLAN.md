# opencode-rs — Long-Term Plan

## 1. Objective

Ship a **drop-in Rust replacement for the opencode server** that existing clients
(TUI, desktop, web, JS SDK) can talk to without modification.

- **In scope:** HTTP API (`/api/*` and legacy `/event`, `/config`, ...), event
  stream (SSE + durable events), session lifecycle, agent loop, tool execution,
  permission/question flow, provider integration, storage, MCP, LSP, PTY.
- **Out of scope:** the TUI, desktop app, web UI, and generated JS SDK. These stay
  TypeScript and connect over the existing HTTP contract.
- **Compatibility target:** the `openapi.json` published in `packages/sdk`, plus
  recorded wire traffic (cassettes) for behaviour the spec does not capture.

## 2. Strategy

1. **Server-only port.** The client/server split already exists; every client
   speaks HTTP + SSE. This removes ~60% of the upstream codebase from scope.
2. **Contract-first, test-first.** The reference repo already encodes the contract
   as machine-checkable artefacts: `openapi.json` (162 paths), a formal event
   manifest, contract-hygiene/compatibility tests, and an HTTP record/replay
   recorder. Port the *tests* before the *implementation*.
3. **Vertical slices.** Each milestone ends with a client-visible capability that
   passes conformance against the reference, not with "all the types compile".
4. **Fixture, don't fork.** Fixtures are generated from the reference and pinned by
   commit. No upstream code is copied; only the observable contract is reproduced.

### Why test-first is mandatory here

This is a **port**, not greenfield. The specification lives in a running process,
not in readable source (the reference is effect-ts — heavy, indirect, and hard to
read as a spec). Therefore the reference behaviour itself must be captured first:

- **Golden/characterization tests** for payload shapes, event envelopes, error
  codes, ID prefixes, and status codes.
- **Record/replay cassettes** for provider traffic (the reference already ships
  `packages/http-recorder`), so LLM adapters can be tested deterministically.
- **Black-box conformance** run side-by-side: same request against reference and
  Rust server, assert semantic equivalence (ignoring timestamps/IDs).

Writing unit tests after implementation would only test our own assumptions. The
tests are the migration specification, so they come first.

## 3. Workspace architecture

| Crate | Responsibility | Mirrors |
|---|---|---|
| `opencode-schema` | Wire types, IDs, event manifest | `packages/schema` |
| `opencode-protocol` | Routes, request/response, error taxonomy | `packages/protocol` |
| `opencode-core` | Service traits, storage, session engine | `packages/core` |
| `opencode-llm` | Provider abstraction + streaming | `packages/llm` |
| `opencode-server` | axum router, handlers, middleware, SSE | `packages/server` |
| `opencode-client` | HTTP + SSE client (tests, CLI) | `packages/sdk` |
| `opencode-cli` | `serve` / `attach` / `run` binaries | `packages/opencode/src/cli` |

Dependency direction: `schema → protocol → core → server`; `llm` and `client`
depend on `schema`/`protocol`; `cli` depends on `server` + `client`.

## 4. Phased roadmap

Each phase has an **exit criterion** that is observable from a stock opencode client.

### Phase 0 — Foundation (this commit)
- Workspace, toolchain, licence, CI hooks.
- Contract extraction scripts + pinned fixtures.
- **Exit:** `cargo test` green; event manifest fixture validated.

### Phase 1 — Health & events
- `GET /api/health`.
- `GET /api/event` (SSE) + `GET /event` with the durable envelope.
- Server-sent event bus; event ID prefixes (`evt_`).
- **Exit:** a stock client connects, health-checks, and receives synthetic events
  that validate against `V2Event`.

### Phase 2 — Sessions (read path)
- `GET /api/session`, `GET /api/session/{id}`, history, context, messages.
- SQLite storage; cursor pagination; ID prefixes (`ses`, `msg`).
- **Exit:** `opencode sessions` / TUI lists sessions from the Rust server.

### Phase 3 — Sessions (write path) + agent loop
- `POST /api/session`, prompt/admit, interrupt, revert stage/commit/clear.
- Agent loop, streaming parts (text/reasoning/tool deltas), step lifecycle events.
- **Exit:** single-provider prompt round-trip; TUI streams an assistant reply.

### Phase 4 — Tools
- `read`, `write`/`edit`, `glob`, `grep` (ripgrep), `bash`, `task`/subagent,
  `webfetch`, `todowrite`, `skill`.
- Permission + question flow (`/api/permission/*`, `/api/question/*`).
- **Exit:** agent completes an edit task end-to-end with permission prompts.

### Phase 5 — Provider matrix
- Streaming adapters for the major providers; model/provider listing from
  models.dev; auth/credential storage.
- Record/replay tests for each adapter.
- **Exit:** provider parity for the top N providers; cassette tests green.

### Phase 6 — Ecosystem
- MCP client (stdio + HTTP), LSP integration, PTY endpoints, plugins bridge.
- Plugin strategy decision: embed a JS runtime (`deno_core`/`rquickjs`) for TS
  plugins, or define a WASM/JSON-RPC plugin ABI.
- **Exit:** an external MCP server and an LSP attach and function.

### Phase 7 — Parity, hardening, release
- Full conformance suite green; fuzzing on the event/payload decoders.
- Packaging (arch/deb/rpm/cargo), cross-compilation, benchmarks vs reference.
- **Exit:** published releases; documented divergences.

## 5. Test strategy detail

| Layer | Tool | What it pins |
|---|---|---|
| Schema | `crates/schema/tests/*` | event names, payload shapes, ID prefixes |
| Protocol | `crates/protocol/tests/*` | status codes, error bodies |
| Router | `tower::ServiceExt::oneshot` | route wiring without sockets |
| Conformance | `opencode-client` vs live reference | semantic equivalence |
| Providers | cassettes (record/replay) | streaming/tool-call parsing |
| Load | criterion | startup time, RSS, latency budgets |

### Conformance harness

`REFERENCE_BASE_URL=http://127.0.0.1:PORT cargo test -p opencode-server --test conformance`
runs the same request against the reference and the Rust server and diffs the
normalised responses (IDs/timestamps masked).

## 6. Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Contract drift (upstream ships daily) | High | High | Pin reference commit per release; nightly "drift" job diffs `openapi.json` and fails |
| Event model complexity (durable/live) | High | High | Golden fixtures + replay from real sessions before implementing |
| Provider matrix breadth | High | Medium | Priority list; cassettes; allow partial coverage with capability flags |
| TS plugin incompatibility | High | Medium | Isolate behind a bridge crate; decide runtime in Phase 6 |
| Effect-ts semantics (concurrency, retries) | Medium | High | Re-derive behaviour from tests/cassettes, not from source structure |
| Hidden behaviour not in OpenAPI | High | Medium | Record/replay against a live reference; expand fixtures per phase |
| Maintainer bandwidth | Medium | High | Vertical slices; each phase independently useful |

## 7. Divergence & upstream policy

- Track a pinned reference commit in `tests/fixtures/UPSTREAM`.
- A scheduled job re-extracts fixtures; a change that breaks conformance opens an
  issue rather than silently shipping.
- We accept intentional divergence only where documented in `docs/DIVERGENCES.md`
  (e.g. plugin ABI, storage engine).

## 8. Definition of done (v1.0)

A user can point an unmodified opencode client at `opencode-rs serve`, run a full
agent session (prompt → streamed tools → edits → permissions → persistence), and
reconnect/resume across restarts, with lower baseline memory and startup time than
the reference server.
