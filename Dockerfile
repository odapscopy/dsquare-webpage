# Stage 1: Build
FROM rust:1.70-slim as builder

WORKDIR /app
COPY . .

RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/axum-webserver /usr/local/bin/
COPY --from=builder /app/assets /usr/local/bin/assets
COPY --from=builder /app/styles /usr/local/bin/styles
COPY --from=builder /app/pages /usr/local/bin/pages
COPY --from=builder /app/index.html /usr/local/bin/

WORKDIR /usr/local/bin
ENV RUST_LOG=info

EXPOSE 3000
CMD ["axum-webserver"]