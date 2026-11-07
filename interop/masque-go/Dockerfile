FROM golang:1.26 AS build
ARG MASQUE_GO_VERSION=v0.6.0
RUN GOBIN=/out go install github.com/quic-go/masque-go/cmd/proxy@${MASQUE_GO_VERSION} \
 && GOBIN=/out go install github.com/quic-go/masque-go/cmd/client@${MASQUE_GO_VERSION}

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /out/proxy /usr/local/bin/masque-go-proxy
COPY --from=build /out/client /usr/local/bin/masque-go-client
