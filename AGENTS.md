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
