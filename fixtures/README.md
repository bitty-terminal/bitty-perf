# Vendored production fixtures (W-105 relocation, bitty CTX-0931)

These directories are byte-identical copies of production-repository files at
the tested production revision recorded in
`crates/bitty-perf/Cargo.toml`
(`bitty@9bc73207ab559d4d5de4ef283a4056dc8857de01`, origin/main at move time).
The suite reads fixtures here, never a mutable product checkout.

- `bitty-vt-seeds/`: copy of `bitty/crates/bitty-vt/seeds/` (parser corpus
  seeds consumed by `bitty_perf::parser_throughput::load_segments`).
- `compat/`: copy of `bitty/tests/compat/` (M1 surface corpora consumed by
  the same loader). This duplicates the corpora owned by the
  `bitty-compat-lab` repository on purpose: each validation repository stays
  self-contained and bound only to its pinned production revision, with no
  cross-dependency between validation repositories.

Freshness: after any production-revision pin bump, refresh both trees from
the new pinned revision and confirm byte equality of every file that the
bump did not intentionally change. The committed
`crates/bitty-perf/baselines/parser-throughput.json` numbers were measured
against these exact bytes; silently divergent fixtures would void the
regression gate, so any fixture change requires re-measuring the baseline
with provenance (see `crates/bitty-perf/baselines/README.md`).

No host paths, usernames, or absolute checkout locations may appear in this
directory. Provenance lives in this file, not in the fixture bytes.
