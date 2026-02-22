FROM rust:1.85-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs && cargo build --release && rm -rf src

COPY . .
RUN touch src/main.rs src/lib.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/novel-api /usr/local/bin/novel-api
COPY --from=builder /app/migrations /app/migrations

WORKDIR /app
ENV RUST_LOG=novel_api=info,tower_http=info
EXPOSE 3000
CMD ["novel-api"]
