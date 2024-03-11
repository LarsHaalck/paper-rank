FROM rust:1.76.0 as builder
WORKDIR /usr/src/prank
COPY Cargo.toml ./
COPY src  ./src
RUN cargo install --path .

FROM debian:bookworm-slim
WORKDIR /opt/prank
COPY static ./static
COPY templates ./templates
RUN apt update && apt install -y libsqlite3-0 openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/prank-server /usr/local/bin/prank-server
COPY --from=builder /usr/local/cargo/bin/prankctl /usr/local/bin/prankctl
EXPOSE 8000
CMD ["prank-server"]
