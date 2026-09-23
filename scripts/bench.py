#!/usr/bin/env python3
"""Measure peak RSS, CPU, and startup time of a server process.

Usage:
  scripts/bench.py --name opencode-rs -- cargo run --release -p opencode-cli -- serve --port 18081
  scripts/bench.py --name reference --url http://127.0.0.1:4096 -- bun run dev serve
  scripts/bench.py --name a -- <a-server> --  scripts/bench.py --name b -- <b-server>  (run separately)

It spawns the command, waits for the health URL (if given), then samples
/proc/<pid>/status (VmRSS) and /proc/<pid>/stat (utime+stime) for --duration
seconds, and prints a single CSV row.
"""
from __future__ import annotations

import argparse
import subprocess
import sys
import time
import urllib.request

CLK_TCK = 100.0  # sysconf(_SC_CLK_TCK); 100 on Linux


def read_rss_kb(pid: int) -> int:
    try:
        with open(f"/proc/{pid}/status") as fh:
            for line in fh:
                if line.startswith("VmRSS:"):
                    return int(line.split()[1])
    except OSError:
        pass
    return 0


def read_cpu_ticks(pid: int) -> int:
    try:
        with open(f"/proc/{pid}/stat") as fh:
            parts = fh.read().split()
        return int(parts[13]) + int(parts[14])  # utime + stime
    except OSError:
        return 0


def healthy(url: str, timeout: float = 0.5) -> bool:
    try:
        with urllib.request.urlopen(f"{url}/api/health", timeout=timeout) as resp:
            return resp.status == 200
    except Exception:
        return False


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--name", required=True)
    parser.add_argument("--duration", type=float, default=10.0)
    parser.add_argument("--interval", type=float, default=0.25)
    parser.add_argument("--url", help="health base url, e.g. http://127.0.0.1:18081")
    parser.add_argument("cmd", nargs=argparse.REMAINDER)
    args = parser.parse_args()

    cmd = args.cmd
    if cmd and cmd[0] == "--":
        cmd = cmd[1:]
    if not cmd:
        print("no command given", file=sys.stderr)
        return 2

    start = time.monotonic()
    proc = subprocess.Popen(cmd)
    try:
        startup_ms = float("nan")
        if args.url:
            deadline = time.monotonic() + 120
            while time.monotonic() < deadline:
                if proc.poll() is not None:
                    print(f"{args.name} exited early rc={proc.returncode}", file=sys.stderr)
                    return 1
                if healthy(args.url):
                    startup_ms = (time.monotonic() - start) * 1000
                    break
                time.sleep(0.05)
            if startup_ms != startup_ms:  # NaN still
                print(f"{args.name} never became healthy", file=sys.stderr)
                return 1

        peak_rss = 0
        cpu0 = read_cpu_ticks(proc.pid)
        t0 = time.monotonic()
        while time.monotonic() - t0 < args.duration:
            if proc.poll() is not None:
                break
            peak_rss = max(peak_rss, read_rss_kb(proc.pid))
            time.sleep(args.interval)
        elapsed = time.monotonic() - t0
        cpu1 = read_cpu_ticks(proc.pid)
        cpu_pct = (cpu1 - cpu0) / CLK_TCK / elapsed * 100 if elapsed > 0 else 0.0

        print(f"{args.name},{peak_rss / 1024:.1f},{cpu_pct:.1f},{startup_ms:.0f}")
        return 0
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    raise SystemExit(main())
