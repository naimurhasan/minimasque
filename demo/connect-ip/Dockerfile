FROM rust:1.96-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p minimasque

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl dnsutils iproute2 iptables iputils-ping \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/minimasque /usr/local/bin/minimasque
