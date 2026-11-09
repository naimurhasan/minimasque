#!/usr/bin/env bash
set -euo pipefail

IMAGE=minimasque-connect-ip
NETWORK=masque-connect-ip
POOL=10.89.0.0/24
TUN=(--cap-add NET_ADMIN --device /dev/net/tun)

cleanup() {
  docker rm -f proxy >/dev/null 2>&1 || true
  docker network rm "$NETWORK" >/dev/null 2>&1 || true
  docker volume rm "$NETWORK-certs" >/dev/null 2>&1 || true
}
trap cleanup EXIT

[[ ${SKIP_BUILD:-0} == 1 ]] || docker build -q -t "$IMAGE" -f demo/connect-ip/Dockerfile . >/dev/null
cleanup
docker network create "$NETWORK" >/dev/null

docker run -d --name proxy --network "$NETWORK" "${TUN[@]}" --sysctl net.ipv4.ip_forward=1 \
  -v "$NETWORK-certs:/certs" "$IMAGE" sh -c "
    iptables -t nat -A POSTROUTING -s $POOL -o eth0 -j MASQUERADE &&
    exec minimasque server --listen 0.0.0.0:4433 --name proxy --cert-out /certs/cert.der --ip-pool $POOL -v" >/dev/null
sleep 2

docker run --rm --network "$NETWORK" "${TUN[@]}" -v "$NETWORK-certs:/certs" "$IMAGE" sh -c '
  minimasque ip --proxy "$(getent hosts proxy | cut -d" " -f1):4433" --name proxy --ca /certs/cert.der --route 1.1.1.1/32 &
  sleep 2
  echo "--- route";  ip route get 1.1.1.1
  echo "--- ping";   ping -c 3 -W 2 1.1.1.1
  echo "--- dns";    dig +time=3 +tries=1 +short @1.1.1.1 example.com A
  echo "--- https";  curl -s --max-time 5 https://1.1.1.1/cdn-cgi/trace | grep -E "^(ip|warp|http)="
'

echo "--- proxy log"
docker logs proxy 2>&1 | grep -E "connect-ip|ADDRESS_ASSIGN|ROUTE_ADVERTISEMENT|10\.89" | head -12
