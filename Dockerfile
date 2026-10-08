FROM docker.io/library/node:22-bookworm-slim AS docs
WORKDIR /app/docs
COPY docs/package*.json ./
RUN npm ci
COPY docs ./
COPY LICENSE NOTICE /app/
RUN npm run build

FROM docker.io/library/rust:1.94.0-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev make gcc libc6-dev \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY docs ./docs
COPY LICENSE NOTICE README.md rust-toolchain.toml Dockerfile ./
COPY --from=docs /app/docs/dist ./docs/dist
RUN tar -czf /tmp/source.tar.gz .
RUN cargo build --locked --release --package polkadot-rest-api

FROM docker.io/library/debian:bookworm-slim
ARG VERSION="0.3.2"
ARG VCS_REF="unknown"
ARG BUILD_DATE=""
LABEL org.opencontainers.image.title="Enjin Blockchain REST API" \
      org.opencontainers.image.vendor="Enjin" \
      org.opencontainers.image.source="https://github.com/enjin/blockchain-rest-api" \
      org.opencontainers.image.licenses="GPL-3.0-or-later" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.created="${BUILD_DATE}"
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/enjin-rest-api /usr/local/bin/
COPY LICENSE NOTICE /usr/share/licenses/blockchain-rest-api/
COPY --from=builder /tmp/source.tar.gz /usr/share/doc/blockchain-rest-api/source.tar.gz
ENV SAS_LOG_LEVEL=info SAS_EXPRESS_PORT=8080 SAS_EXPRESS_BIND_HOST=0.0.0.0
USER nobody
EXPOSE 8080
CMD ["enjin-rest-api"]
