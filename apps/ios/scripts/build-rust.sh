#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUT="$ROOT/apps/ios/Generated"
cd "$ROOT"
export IPHONEOS_DEPLOYMENT_TARGET=17.0
for target in aarch64-apple-ios aarch64-apple-ios-sim; do
  cargo build -q --release --target "$target" -p masque-ffi
done
rm -rf "$OUT" && mkdir -p "$OUT/headers"
cargo build -q -p masque-ffi
cargo run -q -p masque-ffi --features bindgen --bin uniffi-bindgen -- generate \
  --library target/debug/libmasque_ffi.dylib --language swift --out-dir "$OUT"
mv "$OUT/masque_ffiFFI.h" "$OUT/headers/"
mv "$OUT/masque_ffiFFI.modulemap" "$OUT/headers/module.modulemap"
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libmasque_ffi.a -headers "$OUT/headers" \
  -library target/aarch64-apple-ios-sim/release/libmasque_ffi.a -headers "$OUT/headers" \
  -output "$OUT/MasqueFFI.xcframework" >/dev/null
cd "$ROOT/apps/ios" && xcodegen generate --quiet
