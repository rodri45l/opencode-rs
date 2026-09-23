# Benchmarks

Measured with `scripts/bench.py` (peak RSS, CPU %, startup time). A module is only
considered ported when it passes its tests **and** shows parity or an improvement
here, or has a documented reason.

| Date | Binary | Peak RSS (MiB) | CPU (%) | Startup (ms) | Notes |
|---|---|---:|---:|---:|---|
| 2026-09-23 | opencode-rs `serve` (health+SSE only) | 5.7 | ~0.0 | 5 | release build; **no session/agent loop yet** |
| 2026-09-23 | opencode (reference, `bun … serve`) | ~669 (idle) | 1.6 (idle) | ~11-20 | 59 threads; measured on this box after `bun install` |

Reference vs empty-server only. Not comparable until the session/agent slice is
implemented in `opencode-rs`; re-measure after that.

## Reference setup (this box)

```sh
npm install -g bun                     # bun 1.4.2
cd /path/to/opencode && bun install --ignore-scripts   # tree-sitter-powershell native build fails; irrelevant to serving
bun run --cwd packages/opencode src/index.ts serve --port 4096
# idle: RSS ~669 MiB, 59 threads, ~1.6% CPU
```

## How to run

```sh
# Rust server
cargo build --release -p opencode-cli
scripts/bench.py --name opencode-rs --url http://127.0.0.1:18081 -- \
  target/release/opencode-rs serve --port 18081

# Reference server (Bun)
scripts/bench.py --name reference --url http://127.0.0.1:4096 -- \
  bun run --cwd /path/to/opencode dev serve --port 4096
```

Report format: `name,peak_rss_mib,cpu_pct,startup_ms`.
