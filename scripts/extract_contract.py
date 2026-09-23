#!/usr/bin/env python3
"""Extract machine-checkable contract fixtures from the reference opencode repo.

Reads:
  - packages/sdk/openapi.json        (OpenAPI 3.1)
  - packages/protocol/src/errors.ts  (error taxonomy)
and emits, under tests/fixtures/:
  - events.json        the public V2Event manifest
  - errors.json        the tagged error taxonomy (tag, status, fields)
  - routes.json        the route inventory (method, path, operationId, tags)
  - id_prefixes.json   resource id prefixes (evt_, ses, ...)
  - UPSTREAM           the pinned reference commit

Requires only Python 3 — no Bun/Node.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import pathlib
import re
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent
FIXTURES = REPO / "tests" / "fixtures"
METHODS = ("get", "post", "put", "patch", "delete", "head", "options")


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
        events.append({"type": event_type, "schema": name, "durable": "durable" in required})
    events.sort(key=lambda e: e["type"])
    return {
        "source": "anomalyco/opencode packages/sdk/openapi.json#/components/schemas/V2Event",
        "openapi_version": openapi.get("openapi"),
        "event_count": len(events),
        "durable_count": sum(1 for e in events if e["durable"]),
        "events": events,
    }


def extract_errors(reference: pathlib.Path) -> dict:
    source = (reference / "packages" / "protocol" / "src" / "errors.ts").read_text()
    pattern = re.compile(
        r"class\s+(\w+)\s+extends[\s\S]*?\)\(\s*\"(\w+)\"\s*,\s*\{([\s\S]*?)\}\s*,\s*\{\s*httpApiStatus:\s*(\d+)\s*\}"
    )
    errors = []
    for _class, tag, fields_src, status in pattern.findall(source):
        fields = []
        for match in re.finditer(r"(\w+)\s*:\s*(Schema\.optional\(\s*)?Schema\.String", fields_src):
            fields.append({"name": match.group(1), "required": match.group(2) is None})
        errors.append({"tag": tag, "status": int(status), "fields": fields})
    errors.sort(key=lambda e: e["tag"])
    return {
        "source": "anomalyco/opencode packages/protocol/src/errors.ts",
        "error_count": len(errors),
        "errors": errors,
    }


def extract_routes(openapi: dict) -> dict:
    routes = []
    for path, operations in openapi.get("paths", {}).items():
        for method, op in operations.items():
            if method.lower() not in METHODS or not isinstance(op, dict):
                continue
            routes.append(
                {
                    "method": method.upper(),
                    "path": path,
                    "operation_id": op.get("operationId"),
                    "tags": op.get("tags", []),
                }
            )
    routes.sort(key=lambda r: (r["path"], r["method"]))
    return {
        "source": "anomalyco/opencode packages/sdk/openapi.json#/paths",
        "route_count": len(routes),
        "routes": routes,
    }


def extract_id_prefixes(openapi: dict) -> dict:
    schemas = openapi["components"]["schemas"]
    found: dict[str, set[str]] = {}

    def walk(node, path=""):
        if isinstance(node, dict):
            pattern = node.get("pattern")
            if isinstance(pattern, str) and re.fullmatch(r"\^[A-Za-z][A-Za-z0-9_]*", pattern):
                found.setdefault(pattern[1:], set()).add(path or "/")
            for key, value in node.items():
                walk(value, f"{path}/{key}")
        elif isinstance(node, list):
            for item in node:
                walk(item, path)

    walk(schemas)
    prefixes = sorted(found)
    return {
        "source": "anomalyco/opencode packages/sdk/openapi.json patterns",
        "prefix_count": len(prefixes),
        "prefixes": prefixes,
        "sample_locations": {p: sorted(found[p])[0] for p in prefixes},
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
    FIXTURES.mkdir(parents=True, exist_ok=True)

    events = extract_events(openapi)
    errors = extract_errors(args.reference)
    routes = extract_routes(openapi)
    ids = extract_id_prefixes(openapi)

    for name, payload in (
        ("events.json", events),
        ("errors.json", errors),
        ("routes.json", routes),
        ("id_prefixes.json", ids),
    ):
        (FIXTURES / name).write_text(json.dumps(payload, indent=2) + "\n")

    commit = git_commit(args.reference)
    marker = {
        "repository": "anomalyco/opencode",
        "commit": commit,
        "extracted_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "openapi_version": openapi.get("openapi"),
        "path_count": len(openapi.get("paths", {})),
        "schema_count": len(openapi.get("components", {}).get("schemas", {})),
        "event_count": events["event_count"],
        "error_count": errors["error_count"],
        "route_count": routes["route_count"],
        "id_prefix_count": ids["prefix_count"],
    }
    (FIXTURES / "UPSTREAM").write_text(json.dumps(marker, indent=2) + "\n")

    print(
        f"events={events['event_count']} errors={errors['error_count']} "
        f"routes={routes['route_count']} id_prefixes={ids['prefix_count']} commit={commit}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
