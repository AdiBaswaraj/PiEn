#!/usr/bin/env bash
# Build the wasm bundle into ./docs. Commit that folder and enable GitHub
# Pages for the repo (Settings → Pages → Deploy from a branch → /docs).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PROFILE="web-release"
TARGET="wasm32-unknown-unknown"
OUT="$ROOT/docs"

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen is not installed. Run:"
  echo "  cargo install wasm-bindgen-cli --version 0.2.118"
  exit 1
fi

rustup target add "$TARGET" >/dev/null

echo "==> cargo build --profile $PROFILE --target $TARGET"
cargo build --profile "$PROFILE" --target "$TARGET"

WASM_IN="$ROOT/target/$TARGET/$PROFILE/pien.wasm"
echo "==> wasm-bindgen → $OUT"
mkdir -p "$OUT"
# wasm-bindgen uses rayon internally; deep parallel iterators on our large
# input can overflow worker stacks. Force single-thread + large main stack.
RUST_MIN_STACK=67108864 RAYON_NUM_THREADS=1 wasm-bindgen \
  --target web --no-typescript \
  --out-dir "$OUT" --out-name pien "$WASM_IN"

echo "==> done: $OUT/pien.js + $OUT/pien_bg.wasm"
ls -lh "$OUT" | sed 's/^/   /'
