FROM rust:1.96-bookworm AS builder
WORKDIR /app

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked -p hippocore-cli \
    && cp /app/target/release/hippocore /tmp/hippocore

FROM debian:bookworm-slim
RUN useradd -m -u 10001 hippocore
COPY --from=builder /tmp/hippocore /usr/local/bin/hippocore
COPY docker/hippocore-entrypoint.sh /usr/local/bin/hippocore-entrypoint
RUN chmod 0755 /usr/local/bin/hippocore-entrypoint
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/hippocore-entrypoint"]
