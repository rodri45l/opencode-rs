# Benchmarks

Measured with `scripts/bench.py` (peak RSS, CPU %, startup time). A module is only
considered ported when it passes its tests **and** shows parity or an improvement
here, or has a documented reason.

Session read-path comparison, same workload (create N sessions via `POST /api/session`, then `GET /api/session?limit=100`), measured on this box. Reference run on an isolated `XDG_DATA_HOME` and port 4098.

| Binary | Idle RSS | After N sessions | Threads (idle) | Idle CPU | Startup |
|---|---:|---:|---:|---:|---:|
| **opencode-rs** `serve` (release) | **6.4 MiB** | **7.9 MiB** (N=500) | 33 | ~0.0% | 5 ms |
| **opencode** reference (`bun … serve`) | **341 MiB** | **456 MiB** (N=200) | 19 | 0.7% | ~20 ms |
| opencode (user's live server, port 4096) | 669 MiB | — | 59 | 1.6% | — |

Delta:
- **Idle**: 6.4 MiB vs 341 MiB → ~**53×** lower RSS.
- **Per session**: ours ~1.6 KiB/session; reference ~0.57 MiB/session → ~**360×** less growth.
- **Threads**: 33 vs 19-59.

Caveats (do not overclaim):
- `opencode-rs` does **not** yet load the provider/model catalog, provider SDKs,
  plugins, MCP, LSP, or a real agent loop. Those are where much of the reference's
  baseline and growth comes from, and they are not in our number.
- Rows are idle+list only; no LLM calls, tool execution, or streaming.
- `opencode-rs` uses 32 tokio worker threads (default = ncpus); can be capped.
- So this is a genuine session-path comparison, **not** yet a full-server one.
  Re-measure after providers/tools/agent-loop land.

_Historical:_ opencode-rs at health+SSE only: 5.7 MiB RSS, ~0% CPU, 5 ms.

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
