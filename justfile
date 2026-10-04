# Metadata-only gates. These are not Rust/Lua product evidence.
prettier_version := "3.9.6"
markdownlint_version := "0.23.1"
actionlint_version := "1.7.12"

fmt-check:
    bunx --bun prettier@{{prettier_version}} --check . --ignore-unknown

markdownlint:
    bunx --bun markdownlint-cli2@{{markdownlint_version}}

metadata:
    test -s README.md && test -s AGENTS.md && test -s TODO.md && test -s repo.toml
    test -s .carryctx/config.toml
    python3 -c 'import tomllib; from pathlib import Path; [tomllib.loads(p.read_text()) for p in [Path("repo.toml"), Path(".carryctx/config.toml")]]'

hygiene:
    #!/usr/bin/env bash
    set -euo pipefail
    bad=0
    while IFS= read -r -d '' f; do
        case "$f" in
            *.sqlite|*.db|*.zst|*.tar|*.tar.gz|*.tgz|node_modules/*|target/*|dist/*|.worktrees/*)
                echo "unexpected artifact tracked: $f" >&2; bad=1 ;;
        esac
    done < <(git ls-files -z --cached --others --exclude-standard)
    test "$bad" -eq 0

paths:
    #!/usr/bin/env bash
    set -euo pipefail
    pattern='(/hom''e/|/Use''rs/|/mn''t/[A-Za-z]|[A-Za-z]:[\\/]Use''rs[\\/])'
    found=0
    while IFS= read -r -d '' f; do
        # Excluded, mirroring the product workspace gate scope: vendored
        # binary fixtures embed neutral placeholder bytes, and integration
        # tests carry socket-dir fixtures plus the absence-assertion literals
        # that prove no host path leaks into artifacts. Fixture text
        # (READMEs, snapshots) stays scanned.
        case "$f" in
            *.bin|*/tests/*) continue ;;
        esac
        if grep -nEI "$pattern" "$f"; then found=1; fi
    done < <(git ls-files -z --cached --others --exclude-standard)
    if [ "$found" -ne 0 ]; then
        echo 'hardcoded host path detected (portable-path gate)' >&2
        exit 1
    fi
    echo 'portable-path gate passed'

actionlint:
    @installed="$(actionlint --version | head -n 1)"; test "$installed" = "{{actionlint_version}}" || { echo "actionlint {{actionlint_version}} required; found $installed" >&2; exit 1; }
    actionlint -color -shellcheck=

check:
    just fmt-check
    just markdownlint
    just metadata
    just hygiene
    just paths
    just rust-fmt
    just rust-clippy
    just rust-test
    just supply-chain

workflows:
    actionlint
    act -n

# Rust validation-suite gates (W-105 relocation, bitty CTX-0931). These prove
# the moved suite against its pinned production revision (see
# crates/bitty-perf/Cargo.toml), including the bench compile/run gate and the
# parser-throughput regression floor; they are suite evidence, unlike the
# metadata gates above. Run via the justfile, never bare.
rust-fmt:
    cargo fmt --all -- --check

rust-clippy:
    cargo clippy --workspace --all-targets --locked -- -D warnings

rust-test:
    cargo test --workspace --locked --all-targets
    cargo test --workspace --doc --locked

rust-typecheck:
    cargo check --workspace --all-targets --locked

# Local supply-chain gate mirroring the CI `Supply chain (deny/audit)` job:
# `cargo deny check` (advisories, bans, licenses, sources per deny.toml) plus
# `cargo audit` with the same two advisory ignores. Audit uses a fresh
# advisory-db checkout so the shared cache that `cargo deny` populates is
# left untouched.
supply-chain:
    #!/usr/bin/env bash
    set -euo pipefail
    echo '==> cargo deny check'
    cargo deny check
    echo '==> cargo audit'
    tmpdb="$(mktemp -d)"
    trap 'rm -rf "$tmpdb"' EXIT INT TERM
    cargo audit --db "$tmpdb" --ignore RUSTSEC-2024-0436 --ignore RUSTSEC-2026-0192
    trap - EXIT INT TERM
    rm -rf "$tmpdb"

# Publish a redacted CarryCtx snapshot to refs/heads/carryctx-snapshots.
workflow-publish:
    #!/usr/bin/env bash
    set -euo pipefail
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    carryctx export --pack-format dir -o "$tmp" --publication
    git push origin refs/heads/carryctx-snapshots

# Fetch and import the published CarryCtx snapshot (fresh-clone recovery).
workflow-import:
    git fetch origin refs/heads/carryctx-snapshots:refs/remotes/origin/carryctx-snapshots
    carryctx import --from-git refs/remotes/origin/carryctx-snapshots

# No source exists: fail rather than claim product verification.
product:
    @echo 'Blocked: approved source and product gates have not landed.' >&2
    @exit 1
