#!/usr/bin/env bash
set -uo pipefail
cd "$(dirname "$0")/../.."
cargo build -q -p minimasque
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

check() {
  local transport=$1 port=$2
  target/debug/minimasque server -v --listen "127.0.0.1:$port" --cert-out "$work/$transport.der" \
    2>"$work/$transport.log" >/dev/null &
  local server=$!
  sleep 1
  local result
  result=$(swift interop/apple/relay-dns.swift localhost "$port" "$work/$transport.der" "$transport" 2>&1 | tail -1)
  kill "$server"
  wait "$server" 2>/dev/null
  if [[ $result == OK* ]] && grep -q ":protocol: connect-udp" "$work/$transport.log"; then
    echo "✅ Apple Network.framework relay → minimasque over $transport: $result"
  else
    echo "❌ Apple Network.framework relay → minimasque over $transport: $result"
    return 1
  fi
}

failed=0
check h3 4460 || failed=1
check h2 4461 || failed=1
exit $failed
