ARG RUST_VERSION=1.89
ARG DEBIAN_CODENAME=bookworm

FROM rust:${RUST_VERSION}-slim-${DEBIAN_CODENAME} AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libtesseract-dev \
    libleptonica-dev \
    tesseract-ocr-eng \
    clang \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

ENV SQLX_OFFLINE=true \
    CARGO_TERM_COLOR=always


FROM base AS development

# cargo-watch is used only in local development.
RUN cargo install cargo-watch --locked

COPY Cargo.toml Cargo.lock ./

# Warm the debug cache to speed up the first local run.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    mkdir -p src && \
    printf 'fn main() {}\n' > src/main.rs && \
    cargo build --locked && \
    rm -rf src

COPY .sqlx ./.sqlx
COPY migrations ./migrations
COPY seeds ./seeds
COPY src ./src

EXPOSE 3000

CMD ["cargo", "watch", "-w", "src", "-w", "migrations", "-x", "run"]


FROM base AS builder

COPY Cargo.toml Cargo.lock ./

# Build dependencies first so source-only changes reuse the cache.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/cargo-target \
    mkdir -p src && \
    printf 'fn main() {}\n' > src/main.rs && \
    CARGO_TARGET_DIR=/cargo-target cargo build --release --locked --bin iviss-backend && \
    rm -rf src

COPY .sqlx ./.sqlx
COPY migrations ./migrations
COPY seeds ./seeds
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/cargo-target \
    CARGO_TARGET_DIR=/cargo-target cargo build --release --locked --bin iviss-backend && \
    install -D /cargo-target/release/iviss-backend /out/iviss-backend


FROM debian:${DEBIAN_CODENAME}-slim AS production

RUN apt-get update && apt-get install -y --no-install-recommends \
    libtesseract5 \
    liblept5 \
    tesseract-ocr-eng \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* && \
    groupadd --system iviss && \
    useradd --system --gid iviss --home-dir /app --create-home iviss

WORKDIR /app

COPY --from=builder --chown=iviss:iviss /out/iviss-backend /app/iviss-backend

ENV TESSDATA_PREFIX=/usr/share/tesseract-ocr/5/tessdata \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=3000

USER iviss

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:3000/health || exit 1

CMD ["/app/iviss-backend"]
