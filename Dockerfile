FROM rust:1-trixie AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Install `dx` before copying source so edits don't bust the dx layer.
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --version 0.7.9 --root /.cargo -y --force --disable-strategies compile
ENV PATH="/.cargo/bin:$PATH"

COPY . .

# Bundle the web package with release-profile optimizations.
RUN dx bundle --package web --web --release

FROM debian:trixie-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 app
COPY --from=builder /app/target/dx/web/release/web/ /usr/local/app

# Listen on all interfaces so the container is reachable.
ENV PORT=8080
ENV IP=0.0.0.0
EXPOSE 8080

USER app
WORKDIR /usr/local/app
ENTRYPOINT [ "/usr/local/app/server" ]
