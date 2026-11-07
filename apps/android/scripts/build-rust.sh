#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
APP="$ROOT/apps/android/app/src/main"
cd "$ROOT"
cargo ndk -t arm64-v8a -t x86_64 -o "$APP/jniLibs" build --release -p masque-ffi
cargo run -q -p masque-ffi --features bindgen --bin uniffi-bindgen -- generate \
  --library target/aarch64-linux-android/release/libmasque_ffi.so \
  --language kotlin --no-format --out-dir "$APP/java"
