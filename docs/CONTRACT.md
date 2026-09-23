# Contract sources

The Rust server must match the observable behaviour of the reference implementation.
This document lists where that behaviour is pinned and how to regenerate fixtures.

## Reference

- Repository: `anomalyco/opencode`
- Pinned commit: see `tests/fixtures/UPSTREAM`
- Authoritative API spec: `packages/sdk/openapi.json` (OpenAPI 3.1, 162 paths)

## Fixtures

| File | Produced by | Consumed by |
|---|---|---|
| `tests/fixtures/events.json` | `scripts/extract_contract.py` | `opencode-schema` tests |
| `tests/fixtures/errors.json` | `scripts/extract_contract.py` | `opencode-protocol` tests |
| `tests/fixtures/routes.json` | `scripts/extract_contract.py` | `opencode-protocol` tests |
| `tests/fixtures/id_prefixes.json` | `scripts/extract_contract.py` | `opencode-schema` tests |
| `tests/fixtures/UPSTREAM` | `scripts/extract_contract.py` | drift job |
| `tests/fixtures/cassettes/*.json` | reference `http-recorder` | provider adapter tests |

## Known stale reference tests

- `packages/schema/test/event-manifest.test.ts` asserts `55/85/85/32` but is
  **not wired into CI** (no test script/task) and is stale at the pinned commit.
  The authoritative composition is `58/88/88/35`; `packages/opencode/test/
  event-manifest.test.ts` and the OpenAPI `V2Event` fixture both pin `88`. Our
  port asserts the authoritative values (`crates/schema/tests/event_manifest_surface.rs`).

## Regenerating

```sh
# against a checked-out reference repo
python3 scripts/extract_contract.py --reference /path/to/opencode
```

The script reads `packages/sdk/openapi.json` and writes `tests/fixtures/events.json`
plus the pinned commit marker. It requires only Python 3 (no Bun/Node).

## Rules

1. Fixtures are **generated**, never hand-edited. Edit the script instead.
2. Every fixture has a test asserting the Rust types round-trip it.
3. Behaviour not expressible in OpenAPI (streaming order, error timing) is captured
   as cassettes or conformance diffs, not guessed.
