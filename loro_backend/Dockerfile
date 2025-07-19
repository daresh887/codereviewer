FROM rust:1.82 as builder
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/loro_backend /usr/local/bin/loro_backend

EXPOSE 3002

CMD ["loro_backend"]