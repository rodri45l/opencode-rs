#!/usr/bin/env python3
"""Extract machine-checkable contract fixtures from the reference opencode repo.

Reads packages/sdk/openapi.json (OpenAPI 3.1) and emits:
  - tests/fixtures/events.json    the public V2Event manifest
  - tests/fixtures/UPSTREAM       the pinned reference commit (if git available)

Requires only Python 3 — no Bun/Node.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import pathlib
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent
FIXTURES = REPO / "tests" / "fixtures"


def git_commit(reference: pathlib.Path) -> str | None:
    try:
        out = subprocess.check_output(
            ["git", "-C", str(reference), "rev-parse", "HEAD"],
            stderr=subprocess.DEVNULL,
            text=True,
        )
        return out.strip()
    except Exception:
        return None


def extract_events(openapi: dict) -> dict:
    schemas = openapi["components"]["schemas"]
    refs = [r["$ref"].split("/")[-1] for r in schemas["V2Event"]["anyOf"]]
    events = []
    for name in refs:
        schema = schemas[name]
        props = schema.get("properties", {})
        tp = props.get("type", {})
        event_type = (tp.get("enum") or [tp.get("const")])[0]
        required = schema.get("required", [])
        events.append(
            {
                "type": event_type,
                "schema": name,
                "durable": "durable" in required,
            }
        )
    events.sort(key=lambda e: e["type"])
    return {
        "source": "anomalyco/opencode packages/sdk/openapi.json#/components/schemas/V2Event",
        "openapi_version": openapi.get("openapi"),
        "event_count": len(events),
        "durable_count": sum(1 for e in events if e["durable"]),
        "events": events,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--reference",
        type=pathlib.Path,
        required=True,
        help="Path to a checkout of anomalyco/opencode",
    )
    args = parser.parse_args()

    spec_path = args.reference / "packages" / "sdk" / "openapi.json"
    if not spec_path.exists():
        print(f"openapi.json not found at {spec_path}", file=sys.stderr)
        return 1

    openapi = json.loads(spec_path.read_text())
    manifest = extract_events(openapi)

    FIXTURES.mkdir(parents=True, exist_ok=True)
    (FIXTURES / "events.json").write_text(json.dumps(manifest, indent=2) + "\n")

    commit = git_commit(args.reference)
    marker = {
        "repository": "anomalyco/opencode",
        "commit": commit,
        "extracted_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "openapi_version": openapi.get("openapi"),
        "path_count": len(openapi.get("paths", {})),
        "schema_count": len(openapi.get("components", {}).get("schemas", {})),
    }
    (FIXTURES / "UPSTREAM").write_text(json.dumps(marker, indent=2) + "\n")

    print(
        f"events={manifest['event_count']} durable={manifest['durable_count']} "
        f"paths={marker['path_count']} schemas={marker['schema_count']} commit={commit}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
