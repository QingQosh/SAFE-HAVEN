# syntax=docker/dockerfile:1

FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/app /usr/local/bin/app
COPY scripts/backup.sh /usr/local/bin/backup.sh
COPY scripts/restore.sh /usr/local/bin/restore.sh

RUN chmod +x /usr/local/bin/backup.sh /usr/local/bin/restore.sh \
    && mkdir -p /var/backups/app

ENV BACKUP_DIR=/var/backups/app \
    BACKUP_RETENTION_DAYS=30 \
    BACKUP_RETENTION_WEEKS=12 \
    BACKUP_RETENTION_MONTHS=12

EXPOSE 8080

CMD ["app"]
