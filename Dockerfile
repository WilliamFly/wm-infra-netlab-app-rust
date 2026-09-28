# --- Build stage ---
FROM rust:1.98.1-bookworm AS builder
WORKDIR /build

# Cache dependencies separately from source changes
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# --- Runtime stage ---
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/release/netlab-app-rust /app/netlab-app-rust
COPY migrations ./migrations

EXPOSE 8080
CMD ["/app/netlab-app-rust"]
