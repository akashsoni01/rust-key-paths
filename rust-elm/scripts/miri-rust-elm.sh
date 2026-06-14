#!/usr/bin/env bash
# Miri smoke tests for rust-elm (identified, shared, scope — no Tokio threads).
set -euo pipefail
cd "$(dirname "$0")/.."
echo "Running Miri on rust-elm miri_smoke + identified_scope + shared_integration (no runtime)…"
cargo +nightly miri test -p rust-elm \
  --test miri_smoke \
  --test identified_scope_integration \
  --test shared_integration \
  --no-default-features \
  --features serde
