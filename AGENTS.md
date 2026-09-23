# AGENTS.md

Instructions for any agent (or human) working in **opencode-rs**, a from-scratch
Rust port of `anomalyco/opencode`. Read this before touching anything.

## What this repo is

A full-parity Rust port of opencode. We port **tests first** (red), then the
modules that make them pass (green). Progress is tracked file-by-file in
`docs/TEST-PORT.md`.

- Reference pinned commit: `tests/fixtures/UPSTREAM`
- Roadmap: `docs/PLAN.md`
- Porting rules: `docs/PORTING.md`
- Contract sources: `docs/CONTRACT.md`
- Benchmarks: `docs/BENCHMARKS.md`

## Golden rules

1. **Never copy reference source.** Reproduce observable behaviour only. Port tests
   faithfully; derive implementation independently.
2. **Tests first.** A module is ported by making its already-written tests pass.
3. **No silent drops.** Every reference test in `docs/TEST-PORT.md` ends as
   `ported`, `re-derived`, or `n/a` (with a reason). Update its checkbox.
4. **Provenance header** on every ported test:
   `//! Port of packages/<pkg>/test/<file>.test.ts (upstream <short-sha>).`
5. **Never weaken an assertion to make a test pass.** Fix the code or leave it red.
6. **Bench a module when you implement it.** Record RSS/CPU/startup in
   `docs/BENCHMARKS.md`; a module is only "ported" when it passes tests *and*
   shows parity/improvement (or a documented reason).
7. **Never commit secrets.** No API keys, tokens, or `.env` contents.
8. **Never push to `main`.** All work lands via pull request.

## Commands

```sh
cargo fmt --all                     # required before commit
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace              # the green suite (must pass)
cargo test --workspace -- --include-ignored   # the red porting suite (expected to fail)
python3 scripts/extract_contract.py --reference <checkout>
python3 scripts/test_inventory.py --reference <checkout>
scripts/bench.py --name opencode-rs --url http://127.0.0.1:18081 -- \
  target/release/opencode-rs serve --port 18081
```

## Red-first test policy

Tests for unimplemented modules must compile and **fail**, and must not break the
merge gate:

- Mark them `#[ignore = "porting: <module> not implemented"]`.
- The default suite (CI `test` job) stays green and gates merges.
- The CI `porting` job runs `--include-ignored`, is **non-blocking**, and reports
  the red count. When the port is complete we remove the ignores and make it
  blocking (the "ratchet").
- Define the API surface as stubs returning a typed `NotImplemented` error. Do
  **not** use `panic!`/`todo!()` in library code.

## Central services — do NOT re-implement these

To keep agents fast, the following are provided once. Use them; do not duplicate
them per crate (this was a real source of waste in early waves):

- **`opencode-test-support`** (dev-dependency): `fixture()`, `fixture_json()`,
  `repo_root()`, `reference_root()`, `encode_b64url()`/`decode_b64url()`,
  `sse_data_payloads()`, `first_sse_json()`. Do **not** hand-roll base64, fixture
  paths, JSON loading, or SSE parsing. `use opencode_test_support as ts;`
- **Curated workspace deps**: `base64`, `url`, `regex`, `serde`, `serde_json`,
  `uuid`, `thiserror`, `tokio`, `axum`, `reqwest`, `futures`, `async-stream`.
  Declare them as `foo.workspace = true` in your crate. Do not hand-write
  codec/URL/error plumbing, and do not add brand-new crates without asking.
- **`crates/<crate>/port-map.json`** (or central `tests/fixtures/port-map.json`):
  maps a reference test path to your Rust test file stem when names differ. Use
  it instead of renaming awkwardly or skipping a file because another crate has a
  same-named test. You own your crate's file.
- **`scripts/sync_fixtures.py`**: copy reference fixtures into `crates/<crate>/testdata`
  (don't copy by hand).

The **orchestrator** owns `docs/**`, `tests/fixtures/**` (except port-map),
`Cargo.lock`, root `Cargo.toml`, and `.github/**`. Writer agents never touch them.

## Writer agent brief (canonical)

Every writer agent gets this contract:

1. Read `AGENTS.md`, `docs/PORTING.md`, `docs/CONTRACT.md`, `docs/TEST-PORT.md`.
2. **Writer mode**: write code only. Run **no** cargo (or at most one
   `cargo check -p <crate>` with a private `CARGO_TARGET_DIR`). The orchestrator
   compiles the workspace once per wave.
3. Use the central services above; add no dependencies.
4. Port tests **red-first**: `#[ignore = "porting: <topic> not implemented"]`,
   typed `NotImplemented` stubs (never `panic!`/`todo!` in library code), never
   weaken an assertion, never copy upstream source.
5. Provenance header on every ported test file.
6. Record renames/mappings in your `crates/<crate>/port-map.json`.
7. Stay inside `crates/<crate>/`.
8. Return a concise report: files, tests (red/green), covered vs skipped + why,
   compile risks, blockers. Do not commit or push.

**One writer per crate per wave.** Two agents editing the same crate clobber each
other's `src/lib.rs` module declarations and `port-map.json` (this happened in
wave 4 to `crates/server`). If a crate needs more than one agent's worth of work,
split by *file group* and serialize, or give each a distinct crate. Shared files
inside a crate (`src/lib.rs`, `port-map.json`, `Cargo.toml`) must have a single
writer.

## Fast wave protocol (many writers per crate)

Waves were too slow because agents shared `src/lib.rs` / `port-map.json` and had to
be serialised to one-per-crate. Instead:

1. **Local stubs by default.** A red-first test defines the types it needs locally
   and does **not** touch shared files (`src/lib.rs`, `port-map.json`,
   `Cargo.toml`). That makes every test file independent, so many agents can write
   into one crate at once.
   - If the behaviour is **self-contained pure logic** and you want the test green,
     put the logic in a **uniquely-named** `src/<module>.rs` (one module per
     writer) and import it from the test. Do **not** edit `src/lib.rs`; the
     orchestrator/fixer wires `pub mod` declarations at integration.
   - Never inline a fake implementation inside a test to satisfy its own
     assertions. A green test **must exercise the crate's real API**
     (`opencode_<crate>::…`). Otherwise leave it red (`#[ignore = "porting: …"]`).
2. **Disjoint files only.** A writer owns a set of *reference paths* and writes
   one `tests/<stem>.rs` per path. Never edit a file another writer may touch.
3. **No cargo.** Writers run `rustfmt --edition 2021` at most (parse/format only).
   The orchestrator fixes and compiles once. Never run cargo in parallel — this
   box has ~6 GB RAM and would OOM.
4. **Terse status.** Write `crates/<crate>/PORT-STATUS.<tag>.json`
   (`{"files":N,"green":N,"red":N,"skipped":[...]}`) and return **≤10 lines** to
   the orchestrator. Long prose reports are not read.
5. **Inventory.** The orchestrator owns `port-map.json` and `docs/TEST-PORT.md`;
   report any renamed stem in your status file.

This allows a single wave of 10-16 writers across the large crates (server, core,
tui, app) with no shared-file contention.

## Metrics: ported vs implemented

Two different numbers, not one:

- **ported** — a Rust test file exists for the reference test row (docs/TEST-PORT.md).
- **implemented (green)** — the behaviour is actually implemented and the test
  passes. Pure logic is implemented on contact, so early greens are expected and
  legitimate; module-dependent tests stay red (`#[ignore = "porting: …"]`) until
  their phase.

CI reports both. Do not present "green" as "ported" (or vice versa).

## Git flow

- `main` is protected: no direct pushes, PRs only, required status check **`gate`**.
- Branch from `main`: `feat/<scope>`, `fix/<scope>`, `test/<scope>`, `chore/<scope>`.
- Conventional commit subjects (`feat:`, `fix:`, `test:`, `docs:`, `chore:`, `ci:`).
- Keep PRs focused on one crate or one phase slice.
- A PR merges only when `gate` passes (fmt + clippy + green suite) and the branch
  is up to date with `main`.

## CI/CD

- `ci` workflow: `fmt`, `clippy`, `test` feed a `gate` job. **`gate` is the
  required check**; `porting` is informational.
- `release` workflow: builds cross-platform binaries on `v*` tags and attaches
  them to a GitHub release. Never tag without `gate` green on `main`.

## Picking up work

1. Open `docs/TEST-PORT.md`, pick the next `[ ]` row for your crate.
2. Read the reference test in the pinned checkout and port it under
   `crates/<crate>/tests/`. Add stubs as needed.
3. Run `cargo test -p <crate>` — new tests should be red (or ignored-red).
4. Implement until green; bench; update `docs/TEST-PORT.md` and `docs/BENCHMARKS.md`.
5. Open a PR. Do not merge it yourself unless `gate` is green and you own the repo.

## Human-in-the-loop

- **UI testing is the human's job.** If a change needs visual/desktop/TUI
  verification, stop and ask in the PR description rather than guessing.
- Ask before: changing contract fixtures, adding a dependency to a core crate,
  or touching pinned reference data.
