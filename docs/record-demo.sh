#!/usr/bin/env bash
set -euo pipefail
docker build -q -t minimasque-vhs -f docs/vhs/Dockerfile . >/dev/null
docker run --rm -v "$PWD:/vhs" minimasque-vhs docs/demo.tape
