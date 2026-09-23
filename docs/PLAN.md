# opencode-rs — Long-Term Plan

## 1. Objective

Port the **entire** opencode codebase to Rust: server, agent core, tools, providers,
plugin host, CLI, TUI, and the client applications — with full test parity.

No scope cuts. Every package and every test is accounted for in
[`docs/TEST-PORT.md`](TEST-PORT.md) with one of four dispositions: **ported**,
**re-derived**, **asset**, or **n/a** (with justification).

Reference: `anomalyco/opencode` @ pinned commit in `tests/fixtures/UPSTREAM`.

| Metric | Value |
|---|---|
| Hand-written TS/TSX source | ~457,000 lines |
| Test files (packages) | ~744 |
| Test LOC | ~178,500 lines |
| Packages | 32 |

## 2. Strategy

1. **Full parity.** All 32 packages get a Rust home (see the crate map). Browser
   surfaces use a Rust web framework; desktop keeps a Rust host; content-only
   packages are marked `n/a` with a reason.
2. **Contract-first, test-first.** The reference ships machine-checkable contract
   artefacts (`openapi.json`, an event manifest, error taxonomy, record/replay
   cassettes). Fixtures are extracted from these and pin the Rust types; tests
   lead each slice.
3. **No test left behind.** Every test file is classified and tracked in
   `TEST-PORT.md`. "Port all the tests" is executed as a finite, checkable
   workstream rather than a big-bang rewrite.
4. **Vertical slices.** Each phase ends with an end-to-end capability using
   stock behaviour, not with "types compile".
5. **Fixture, don't fork.** Only observable contract is reproduced; no upstream
   source is copied.

### Test disposition tiers

| Tier | Meaning | Action |
|---|---|---|
| **C** | contract / black-box (wire, status, events, DB output, provider I/O) | port first, per phase |
| **P** | pure logic (cursors, config resolution, diff, tokens, ids) | port with the module |
| **W** | white-box (Effect internals, module coupling) | **re-derive against the Rust design**, keep the behavioural assertions |
| **F** | fixtures / cassettes / snapshots | reuse as assets |

"Everything" means every file gets a row and a final state of `ported` or
`re-derived` (or `n/a` with a documented reason) — no silent drops.

## 3. Crate map (all 32 packages)

| Reference package | Rust home | Disposition |
|---|---|---|
| `schema` | `crates/schema` | done |
| `protocol` | `crates/protocol` | done |
| `core` | `crates/core` | port |
| `llm` | `crates/llm` | port |
| `server` | `crates/server` | port |
| `opencode` (CLI + host) | `crates/cli` + `crates/server` | port |
| `plugin` | `crates/plugin` | port (JS bridge or native ABI) |
| `http-recorder` | `crates/http-recorder` | port (test infra) |
| `httpapi-codegen` | build tool | port (generates `crates/protocol`) |
| `effect-drizzle-sqlite`, `effect-sqlite-node` | `crates/sqlite` | port |
| `function` | `crates/function` | port |
| `identity` | `crates/identity` | port |
| `codemode` | `crates/codemode` | port |
| `cli` | `crates/cli` | port |
| `client` | `crates/client` | port |
| `sdk`, `sdk-next` | `crates/sdk` (generated) | port |
| `tui` | `crates/tui` | port (ratatui or OpenTUI binding) |
| `ui` | `crates/ui` | port |
| `session-ui` | `crates/tui` | port |
| `app` | `crates/app` | port (Rust web/desktop framework) |
| `desktop` | `crates/desktop` (Tauri host stays Rust) | port |
| `console` | `crates/console` | port |
| `stats` | `crates/stats` | port |
| `slack`, `enterprise`, `containers` | `crates/integrations` | port |
| `script` | `xtask` | port |
| `docs` (MDX content) | `docs/` | n/a — content, not code |
| `storybook` | — | n/a — dev tooling |
| `web` (marketing site) | `crates/web` | port if desired — mostly content |

Open decisions recorded here rather than hidden: **TUI** = ratatui vs an OpenTUI
binding; **web/console** = Leptos vs Dioxus; **plugin** = embedded JS runtime vs
native ABI. These are decided at the start of their phase.

## 4. Phased roadmap

Each phase ends with an observable capability **and** its Tier-C tests green.

- **Phase 0 — Foundation.** Workspace, contract extractor, fixtures, CI. *(done)*
- **Phase 1 — Health & events.** `/api/health`, `/api/event` (+ legacy), SSE bus. *(done)*
- **Phase 2 — Sessions (read).** list/get/history/context/messages, cursors, SQLite. Test parity: cursor, storage.
- **Phase 3 — Sessions (write) + agent loop.** create, prompt/admit, interrupt, revert; streaming parts and step lifecycle.
- **Phase 4 — Tools.** read/write/edit/glob/grep/bash/task/webfetch/todowrite/level; permission + question flow.
- **Phase 5 — Providers.** streaming adapters + models.dev catalog; cassettes from `llm/test/fixtures/recordings`.
- **Phase 6 — Ecosystem.** MCP, LSP, PTY, plugin host, integrations.
- **Phase 7 — TUI.** Rust terminal client (`tui`, `session-ui`, `ui`), attach/run.
- **Phase 8 — Apps.** `app`, `desktop`, `console`, `web`, `stats`.
- **Phase 9 — Parity & release.** Full test disposition complete, fuzzing, packaging, benchmarks.

## 5. Test-parity operations

- `scripts/test_inventory.py` classifies every reference test file (scope, tier,
  crate) and regenerates `tests/fixtures/test-inventory.json` + `docs/TEST-PORT.md`.
- Each phase flips its rows from `pending` to `ported`/`re-derived`.
- A CI job re-runs the inventory against the pinned commit and fails if a test
  file appears or disappears without the inventory being updated (drift gate).
- Provider cassettes and `models-api.json` are reused as assets, not rewritten.

## 6. Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Test volume (~744 files) | High | High | Inventory + per-phase rows; no silent drops |
| UI port domain shift (web/desktop) | High | High | Framework decided per phase; Tauri host already Rust |
| OpenTUI is Zig+TS | Medium | Medium | Decide ratatui vs binding at Phase 7 |
| Plugin system (TS modules) | High | Medium | Embedded JS runtime or native ABI at Phase 6 |
| Effect-ts semantics | Medium | High | Re-derive from cassettes/tests, not source shape |
| Contract drift (daily upstream) | High | High | Pinned commit + drift gate |
| Maintainer bandwidth | Medium | High | Vertical slices; every phase independently useful |

## 7. Definition of done (v1.0)

Every package has a Rust implementation or a documented `n/a`; every reference test
file has a final disposition; a user can run the full stack from a Rust binary
(server + TUI + apps) with lower baseline memory and startup time than the
reference, and all Tier-C tests pass against the pinned contract.
