## What changed

<!-- One or two sentences. Link the TEST-PORT.md row or phase. -->

## Porting checklist

- [ ] Tests added first and are red (`cargo test -p <crate>`) or `#[ignore = "porting: ..."]`
- [ ] Provenance header on every ported test file
- [ ] `docs/TEST-PORT.md` rows updated
- [ ] Benchmarks recorded in `docs/BENCHMARKS.md` (if a module was implemented)
- [ ] No reference source copied; no assertions weakened
- [ ] `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` pass locally

## Verification

- [ ] `gate` CI is green
- [ ] UI/visual changes verified manually (human) — or n/a

## Notes for reviewers

<!-- Anything the human should check, especially UI. -->
