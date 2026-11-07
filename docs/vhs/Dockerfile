FROM rust:1.96-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p minimasque

FROM ghcr.io/charmbracelet/vhs:v0.10.0
COPY --from=build /src/target/release/minimasque /usr/local/bin/minimasque
