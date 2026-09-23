# Benchmarks

Measured with `scripts/bench.py` (peak RSS, CPU %, startup time). A module is only
considered ported when it passes its tests **and** shows parity or an improvement
here, or has a documented reason.

| Date | Binary | Peak RSS (MiB) | CPU (%) | Startup (ms) | Notes |
|---|---|---:|---:|---:|---|
| 2026-09-23 | opencode-rs (health+SSE, debug) | _pending_ | _pending_ | _pending_ | run `scripts/bench.py --url http://127.0.0.1:18081 -- cargo run --release -p opencode-cli -- serve --port 18081` |
| 2026-09-23 | opencode (reference) | _pending_ | _pending_ | _pending_ | reference needs a Bun install; fill when available |

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
