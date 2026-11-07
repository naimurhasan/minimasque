#!/usr/bin/env bash
set -uo pipefail

IMPLS=(minimasque masque-go quiche)
NETWORK=masque-interop
PORT=4433
WELL_KNOWN="https://proxy:$PORT/.well-known/masque/udp/{target_host}/{target_port}/"
RESULTS="${RESULTS:-interop/results.md}"

image() { echo "minimasque-interop-$1"; }

template() {
  case $1 in
    masque-go) echo "https://proxy:$PORT/masque?h={target_host}&p={target_port}" ;;
    *) echo "$WELL_KNOWN" ;;
  esac
}

build() {
  docker build -q -t "$(image minimasque)" -f interop/minimasque/Dockerfile . >/dev/null
  docker build -q -t "$(image masque-go)" interop/masque-go >/dev/null
  docker build -q -t "$(image quiche)" interop/quiche >/dev/null
}

openssl_cert() {
  docker run --rm -v "$1:/certs" "$(image masque-go)" sh -c '
    openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -days 1 \
      -subj /CN=proxy -addext subjectAltName=DNS:proxy -addext basicConstraints=critical,CA:FALSE \
      -keyout /certs/key.pem -out /certs/cert.pem 2>/dev/null &&
    openssl x509 -in /certs/cert.pem -outform der -out /certs/cert.der'
}

start_proxy() {
  local impl=$1 certs=$2
  local run=(docker run -d --rm --name proxy --network "$NETWORK" -v "$certs:/certs")
  case $impl in
    minimasque)
      "${run[@]}" "$(image minimasque)" minimasque server --listen "0.0.0.0:$PORT" --name proxy --cert-out /certs/cert.der ;;
    masque-go)
      openssl_cert "$certs"
      "${run[@]}" "$(image masque-go)" masque-go-proxy -b "0.0.0.0:$PORT" -c /certs/cert.pem -k /certs/key.pem -t "$(template masque-go)" ;;
    quiche)
      openssl_cert "$certs"
      "${run[@]}" "$(image quiche)" masque_server --port="$PORT" --certificate_file=/certs/cert.pem --key_file=/certs/key.pem ;;
  esac >/dev/null
  for _ in $(seq 20); do [ -s "$certs/cert.der" ] && sleep 1 && return 0; sleep 0.5; done
  return 1
}

run_client() {
  local impl=$1 template=$2 certs=$3
  local run=(docker run --rm --network "$NETWORK" -v "$certs:/certs" -e "T=$template")
  case $impl in
    minimasque)
      "${run[@]}" "$(image minimasque)" sh -c '
        minimasque client --proxy "$(getent hosts proxy | cut -d" " -f1):'"$PORT"'" --name proxy \
          --ca /certs/cert.der --target 1.1.1.1:53 --template "$T" & sleep 2
        dig +time=3 +tries=2 @127.0.0.1 -p 5300 example.com A' | grep -q "status: NOERROR" ;;
    masque-go)
      "${run[@]}" "$(image masque-go)" sh -c '
        openssl x509 -inform der -in /certs/cert.der -out /usr/local/share/ca-certificates/proxy.crt &&
        update-ca-certificates >/dev/null 2>&1 &&
        timeout 20 masque-go-client -t "$T" https://cloudflare.com/cdn-cgi/trace 2>&1' | grep -q "HTTP status: 200" ;;
    quiche)
      "${run[@]}" "$(image quiche)" sh -c '
        timeout 20 masque_client --stderrthreshold=0 --disable_certificate_verification "$T" \
          https://cloudflare.com/cdn-cgi/trace 2>&1' | grep -q "Request succeeded" ;;
  esac
}

pair() {
  local client=$1 proxy=$2 certs ok=1
  certs=$(mktemp -d "${TMPDIR:-/tmp}/masque-interop.XXXXXX")
  start_proxy "$proxy" "$certs" && run_client "$client" "$(template "$proxy")" "$certs" && ok=0
  docker rm -f proxy >/dev/null 2>&1
  rm -rf "$certs"
  return $ok
}

main() {
  [[ ${SKIP_BUILD:-0} == 1 ]] || build
  docker network create "$NETWORK" >/dev/null 2>&1
  local failed=0 header="| Client ↓ / Proxy → |" divider="|---|"
  for proxy in "${IMPLS[@]}"; do header+=" $proxy |"; divider+="---|"; done
  local table=("$header" "$divider")
  for client in "${IMPLS[@]}"; do
    local row="| **$client** |"
    for proxy in "${IMPLS[@]}"; do
      if pair "$client" "$proxy" || pair "$client" "$proxy"; then
        row+=" ✅ |"
      else
        row+=" ❌ |"
        [[ $client == minimasque || $proxy == minimasque ]] && failed=1
      fi
      echo "$client -> $proxy done" >&2
    done
    table+=("$row")
  done
  docker network rm "$NETWORK" >/dev/null 2>&1
  { echo "Last run: $(date -u +%Y-%m-%d)"; echo; printf '%s\n' "${table[@]}"; } | tee "$RESULTS"
  return $failed
}

main "$@"
