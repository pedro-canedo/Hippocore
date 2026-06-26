FROM rust:1.75-bookworm AS builder
WORKDIR /app

COPY . .
RUN cargo build --release -p hippocore-cli

FROM debian:bookworm-slim
RUN useradd -m -u 10001 hippocore
WORKDIR /data
COPY --from=builder /app/target/release/hippocore /usr/local/bin/hippocore
USER hippocore
EXPOSE 8080
ENTRYPOINT ["/bin/sh", "-lc"]
CMD ["exec hippocore serve --db /data --port 8080 --api-key \"${HIPPOCORE_API_KEY:-change-me}\""]
