Last run: 2026-11-07

| Client ↓ / Proxy → | minimasque | masque-go | quiche |
|---|---|---|---|
| **minimasque** | ✅ | ✅ | ✅ |
| **masque-go** | ✅ | ✅ | ✅ |
| **quiche** | ✅ | ❌ | ✅ |

quiche → masque-go: the CONNECT-UDP tunnel opens, but the inner QUIC handshake times out (QUIC_NETWORK_IDLE_TIMEOUT). This pair does not involve minimasque.
