# bitty-perf

Independent performance validation suite for Bitty (W-105 relocation, bitty CTX-0931 / bitty#1619). Read [AGENTS](AGENTS.md). Management lives in CarryCtx.

- Suite: `crates/bitty-perf` — headless, bounded performance baseline
  harness (startup/latency/idle/real-window/real-soak/throughput-floor/
  typical-session/dogfood-session modules, evidence and regression tests,
  committed baselines under `crates/bitty-perf/baselines/`). Benchmark
  binaries live under `benches/`. Vendored production corpus fixtures live
  under `fixtures/` (see `fixtures/README.md` for provenance).
- Tested production revision: the pinned immutable `bitty` commit recorded
  in `crates/bitty-perf/Cargo.toml` (W-75 pin discipline: exact commit, never
  a branch or tag, never a path dependency). Pin bumps are owned, reviewed
  changes.
- Gates: `just check` runs the metadata gates plus `just rust-fmt`,
  `just rust-clippy`, and `just rust-test` (bench compile/run gate and the
  parser-throughput regression floor against the pinned revision). The bitty
  product change path keeps thin required invocations of this suite (bench
  gate, parser-throughput floor) against the pinned suite revision.
- This repository is `publish = false` tooling with no shipped runtime
  authority; an independent repository is not an independent gate.

Prerequisite: W-75 / bitty-docs CTX-0265 / bitty-docs#404, and W-105 / bitty CTX-0931 / bitty#1619, under the bitty#1629 umbrella (metadata-only until contracts accepted). Preserve baseline provenance, budget gates and reproducibility; pin production revisions.

CTX-0001 -> CTX-0002 -> CTX-0003 -> CTX-0004 maps to Issues #4 -> #3 -> #2 -> #1. CTX-0001 (bootstrap) is complete: metadata gates, independent review, first publication, redacted CarryCtx snapshot and branch protection are recorded. Suite migration landed as PR #7 (CTX-0003, bitty@9bc73207); acceptance (including CTX-0004 independent verification) belongs to the owning tasks.
