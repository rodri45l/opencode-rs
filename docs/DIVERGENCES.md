# Intentional divergences

The Rust server aims for wire compatibility with the reference implementation.
Any place where behaviour deliberately differs is recorded here.

| Area | Reference | opencode-rs | Rationale |
|---|---|---|---|
| (none yet) | | | |

Rules: every divergence needs a test that documents the old and new behaviour, and
a note here explaining why compatibility is not maintained.
