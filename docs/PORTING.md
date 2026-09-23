# Porting conventions

Rules every contributor (human or agent) follows when porting a reference test or
module. The goal is a suite that is **red first, green as modules land**, with no
silent drops and no copied source.

## Scope

- Port order: **tests first**, then the modules that make them pass.
- Source of truth: `anomalyco/opencode` at the commit in `tests/fixtures/UPSTREAM`.
- Never copy reference source. Reproduce behaviour only.
- Every test file has a row in `docs/TEST-PORT.md`; update its status when done.

## Where things live

| Reference | Rust |
|---|---|
| `packages/<pkg>/test/foo.test.ts` | `crates/<crate>/tests/foo.rs` |
| `packages/<pkg>/src/foo.ts` (unit tests) | `crates/<crate>/src/foo.rs` + `#[cfg(test)]` |
| `packages/<pkg>/test/fixtures/**` | `crates/<crate>/testdata/**` (copy the asset) |

Each ported test file starts with a provenance header:

```rust
//! Port of packages/<pkg>/test/<file>.test.ts (upstream <short-sha>).
//! Behaviour pinned by <reference>; see docs/TEST-PORT.md.
```

## Stub policy (make tests compile, then fail)

Tests must **fail**, not be ignored or commented out. For a module that is not
implemented yet:

1. Define the public API surface the tests need (types, traits, functions).
2. Unimplemented bodies return an error — never `panic!`/`todo!()` in library
   code, so one failure does not abort the suite:

```rust
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    // ...
}
```

3. Tests assert real behaviour, so they fail with a clear `NotImplemented` value
   until the module lands. Do not weaken assertions to make them pass.

## Test tiers

From `scripts/test_inventory.py`:

- **C** contract/black-box — port faithfully, assertions unchanged in spirit.
- **P** pure logic — port faithfully.
- **W** white-box — keep the *behavioural* assertions, drop assertions about
  Effect internals (fibers, `Layer`, `Context`). Re-express against the Rust API.
- **n/a** — not code; leave a one-line justification.

## Progress metric

`cargo test --workspace` pass/fail counts are the porting progress. Red-first
tests are marked `#[ignore = "porting: <module> not implemented"]`:

- The default suite (CI `test` job) stays green and gates merges via the `gate`
  check.
- The CI `porting` job runs `cargo test --workspace -- --include-ignored`,
  is non-blocking, and reports the red count.
- When the port completes, the ignores are removed and the `porting` job becomes
  blocking (the ratchet).

## Benchmarks (required per module)

A module is "ported" only when its tests pass **and** `scripts/bench.py` shows
parity or an improvement versus the reference, or a documented reason in
`docs/BENCHMARKS.md`. Record:

- peak RSS (MiB)
- average and peak CPU (%)
- startup time to first healthy response (ms)

## Definition of done for a phase

1. Its Tier-C rows in `docs/TEST-PORT.md` are `ported`/`re-derived`.
2. `cargo test --workspace` is green for that crate.
3. `docs/BENCHMARKS.md` has a fresh row for the affected binary.
4. No `NotImplemented` remains in the touched module.
