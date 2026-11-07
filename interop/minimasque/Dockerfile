FROM rust:1.96 AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p minimasque

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends dnsutils openssl ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/minimasque /usr/local/bin/minimasque
