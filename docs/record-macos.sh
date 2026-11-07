#!/usr/bin/env bash
set -euo pipefail
cargo build -q --release -p minimasque
PATH="$PWD/target/release:$PATH" vhs docs/server.tape
mv docs/assets/server.png docs/assets/macos-server.png
