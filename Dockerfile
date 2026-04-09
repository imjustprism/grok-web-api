FROM rust:1.85-slim AS chef
RUN apt-get update \
    && apt-get install -y --no-install-recommends nasm cmake clang perl pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin grok-server

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /usr/sbin/nologin grok
COPY --from=builder /app/target/release/grok-server /usr/local/bin/grok-server
USER grok
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -sf http://localhost:3000/health || exit 1
CMD ["grok-server"]
