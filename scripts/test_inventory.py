#!/usr/bin/env python3
"""Classify every reference test file and emit a tracked inventory.

Reads a checkout of anomalyco/opencode and writes:
  - tests/fixtures/test-inventory.json   machine-readable classification
  - docs/TEST-PORT.md                    human checklist (checkboxes per file)

Tiers:
  C  contract / black-box (wire, protocol, events, provider I/O)
  P  pure logic
  W  white-box (imports internal src; re-derive against the Rust design)
  n/a content-only package, not ported

Status is auto-detected: a reference test is `ported` when a matching Rust test
file exists under the mapped crate. A small exceptions map covers renamed ports.

The classification is heuristic and refined as each phase starts; the invariant
is that every test file has a row and a disposition.
"""
from __future__ import annotations

import argparse
import collections
import datetime as dt
import json
import pathlib
import re
import subprocess

HERE = pathlib.Path(__file__).resolve().parent
REPO = HERE.parent
FIXTURES = REPO / "tests" / "fixtures"

# package -> rust crate (mirrors docs/PLAN.md section 3)
PACKAGE_CRATE = {
    "schema": "crates/schema",
    "protocol": "crates/protocol",
    "core": "crates/core",
    "llm": "crates/llm",
    "server": "crates/server",
    "opencode": "crates/server",
    "plugin": "crates/plugin",
    "http-recorder": "crates/http-recorder",
    "httpapi-codegen": "crates/protocol",
    "effect-drizzle-sqlite": "crates/sqlite",
    "effect-sqlite-node": "crates/sqlite",
    "function": "crates/function",
    "identity": "crates/identity",
    "codemode": "crates/codemode",
    "cli": "crates/cli",
    "client": "crates/client",
    "sdk": "crates/sdk",
    "sdk-next": "crates/sdk",
    "tui": "crates/tui",
    "ui": "crates/ui",
    "session-ui": "crates/tui",
    "app": "crates/app",
    "desktop": "crates/desktop",
    "console": "crates/console",
    "stats": "crates/stats",
    "slack": "crates/integrations",
    "enterprise": "crates/integrations",
    "containers": "crates/integrations",
    "web": "crates/web",
    "docs": "docs",
    "storybook": "n/a",
    "script": "xtask",
}

# content-only packages not ported as code
NOT_PORTED = {"docs", "storybook"}

CONTRACT_SIGNALS = (
    "createOpencodeClient",
    "/api/",
    "operationId",
    "openapi",
    "HttpApiEndpoint",
    "HttpApiGroup",
    "cassette",
    "recording",
    "EventManifest",
    "Schema.decode",
    "Schema.encode",
    "fetch(",
)
SRC_IMPORT = re.compile(r'from "(\.\./)+src|from "@/')
TEST_GLOB = re.compile(r"\.(test|spec)\.(ts|tsx)$")

# reference test path -> rust test file stem(s), for ports whose names differ.
# Merged from tests/fixtures/port-map.json (central) and crates/*/port-map.json
# (per-crate, owned by the agents working in that crate).
def load_port_map() -> dict[str, list[str]]:
    merged: dict[str, list[str]] = {}
    sources = sorted((REPO / "crates").glob("*/port-map.json"))
    central = FIXTURES / "port-map.json"
    if central.exists():
        sources.append(central)
    for source in sources:
        try:
            data = json.loads(source.read_text())
        except Exception:
            continue
        for key, value in data.items():
            if key.startswith("_"):
                continue
            merged[key] = value if isinstance(value, list) else [value]
    return merged


def reference_commit(reference: pathlib.Path) -> str | None:
    try:
        return subprocess.check_output(
            ["git", "-C", str(reference), "rev-parse", "HEAD"],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
    except Exception:
        return None


def classify(package: str, text: str) -> str:
    if package in NOT_PORTED:
        return "n/a"
    if any(signal in text for signal in CONTRACT_SIGNALS):
        return "C"
    if SRC_IMPORT.search(text):
        return "W"
    return "P"


def load_na_map() -> dict[str, str]:
    """reference path -> reason, for tests deliberately not ported (visual,
    live-runtime, unwired upstream). Merged from crates/*/na-map.json + central."""
    merged: dict[str, str] = {}
    sources = sorted((REPO / "crates").glob("*/na-map.json"))
    central = FIXTURES / "na-map.json"
    if central.exists():
        sources.append(central)
    for source in sources:
        try:
            data = json.loads(source.read_text())
        except Exception:
            continue
        for key, value in data.items():
            if key.startswith("_"):
                continue
            merged[key] = value if isinstance(value, str) else value.get("reason", "")
    return merged


def rust_test_exists(crate: str, path: str, port_map: dict[str, list[str]]) -> bool:
    tests_dir = REPO / crate / "tests"
    if not tests_dir.is_dir():
        return False
    if path in port_map and any((tests_dir / f"{stem}.rs").is_file() for stem in port_map[path]):
        return True
    basename = re.sub(r"\.(test|spec)\.(ts|tsx)$", "", path.rsplit("/", 1)[-1])
    candidates = {
        basename.replace("-", "_").replace(".", "_"),
        basename.replace(".", "_"),
        basename.replace("-", "_"),
    }
    candidates |= {re.sub(r"(?<!^)(?=[A-Z])", "_", c).lower() for c in list(candidates)}
    files = [f.stem for f in tests_dir.glob("*.rs")]
    for stem in candidates:
        if stem in files or any(name.endswith(f"_{stem}") for name in files):
            return True
    return False


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference", type=pathlib.Path, required=True)
    args = parser.parse_args()

    tracked = subprocess.check_output(
        ["git", "-C", str(args.reference), "ls-files"], text=True
    ).splitlines()
    tests = [p for p in tracked if p.startswith("packages/") and TEST_GLOB.search(p)]
    port_map = load_port_map()
    na_map = load_na_map()

    files = []
    for path in tests:
        package = path.split("/")[1] if len(path.split("/")) > 1 else ""
        crate = PACKAGE_CRATE.get(package, "?")
        tier = classify(package, (args.reference / path).read_text(errors="ignore"))
        reason = ""
        if tier == "n/a":
            status = "n/a"
            reason = "content-only package (not ported)"
        elif path in na_map:
            status = "n/a"
            reason = na_map[path]
        elif rust_test_exists(crate, path, port_map):
            status = "ported"
        else:
            status = "pending"
        files.append(
            {
                "path": path,
                "package": package,
                "crate": crate,
                "tier": tier,
                "status": status,
                "reason": reason,
            }
        )
    files.sort(key=lambda f: (f["crate"], f["tier"], f["path"]))

    by_tier = collections.Counter(f["tier"] for f in files)
    by_status = collections.Counter(f["status"] for f in files)
    by_crate = collections.Counter(f["crate"] for f in files)

    inventory = {
        "source": "anomalyco/opencode test files under packages/**",
        "commit": reference_commit(args.reference),
        "extracted_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "test_file_count": len(files),
        "by_tier": dict(sorted(by_tier.items())),
        "by_status": dict(sorted(by_status.items())),
        "by_crate": dict(sorted(by_crate.items(), key=lambda kv: -kv[1])),
        "files": files,
    }
    FIXTURES.mkdir(parents=True, exist_ok=True)
    (FIXTURES / "test-inventory.json").write_text(json.dumps(inventory, indent=2) + "\n")

    lines = [
        "# Test port tracker",
        "",
        "Generated by `scripts/test_inventory.py` — do not edit by hand.",
        "",
        f"- Reference commit: `{inventory['commit']}`",
        f"- Test files: **{len(files)}**",
        "- Tiers: **C** contract · **P** pure logic · **W** white-box (re-derive) · **n/a** not code",
        "- Status is auto-detected from the presence of the ported Rust test file.",
        "",
        "## Summary by tier",
        "",
        "| Tier | Files |",
        "|---|---:|",
    ]
    for tier, count in sorted(by_tier.items()):
        lines.append(f"| {tier} | {count} |")
    lines += ["", "## Summary by status", "", "| Status | Files |", "|---|---:|"]
    for status, count in sorted(by_status.items()):
        lines.append(f"| {status} | {count} |")
    lines += ["", "## Summary by crate", "", "| Crate | Files | Ported |", "|---|---:|---:|"]
    for crate, count in sorted(by_crate.items(), key=lambda kv: -kv[1]):
        ported = sum(1 for f in files if f["crate"] == crate and f["status"] == "ported")
        lines.append(f"| `{crate}` | {count} | {ported} |")

    lines += ["", "## Checklist", ""]
    grouped: dict[str, list[dict]] = collections.defaultdict(list)
    for entry in files:
        grouped[entry["crate"]].append(entry)
    for crate in sorted(grouped):
        entries = grouped[crate]
        ported = sum(1 for e in entries if e["status"] == "ported")
        lines.append(f"### `{crate}` ({ported}/{len(entries)})")
        lines.append("")
        for entry in entries:
            box = "x" if entry["status"] != "pending" else " "
            lines.append(f"- [{box}] `{entry['tier']}` `{entry['path']}`")
        lines.append("")

    (REPO / "docs" / "TEST-PORT.md").write_text("\n".join(lines) + "\n")
    print(
        f"test files={len(files)} tiers={dict(sorted(by_tier.items()))} "
        f"status={dict(sorted(by_status.items()))} commit={inventory['commit']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
