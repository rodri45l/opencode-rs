#!/usr/bin/env python3
"""Copy reference fixtures into a crate's testdata directory.

Instead of each agent hand-copying recordings, use:

  scripts/sync_fixtures.py --reference /tmp/opencode/loc/opencode \
    --from packages/llm/test/fixtures/recordings \
    --to crates/llm/testdata/recordings

Copies tracked files only (skips .git and untracked junk) and preserves layout.
"""
from __future__ import annotations

import argparse
import pathlib
import shutil
import subprocess

REPO = pathlib.Path(__file__).resolve().parent.parent


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference", type=pathlib.Path, required=True)
    parser.add_argument("--from", dest="src", required=True, help="path relative to the reference root")
    parser.add_argument("--to", dest="dst", required=True, help="path relative to this repo root")
    args = parser.parse_args()

    source = args.reference / args.src
    if not source.is_dir():
        print(f"source not found: {source}")
        return 1

    tracked = subprocess.check_output(
        ["git", "-C", str(args.reference), "ls-files", "--", str(args.src)], text=True
    ).splitlines()
    if not tracked:
        print(f"no tracked files under {args.src}")
        return 1

    dest_root = REPO / args.dst
    copied = 0
    for rel in tracked:
        src_file = args.reference / rel
        rel_inside = pathlib.Path(rel).relative_to(args.src)
        dst_file = dest_root / rel_inside
        dst_file.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src_file, dst_file)
        copied += 1
    print(f"copied {copied} files -> {dest_root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
