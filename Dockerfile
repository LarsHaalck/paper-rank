FROM rust:1.53.0 as builder
WORKDIR /usr/src/prank
COPY Cargo.toml ./
COPY src  ./src
RUN apt update && apt install -y libsqlite3-dev
RUN cargo install --path .

FROM debian:buster-slim
WORKDIR /opt/prank
COPY static ./static
COPY templates ./templates
RUN apt update && apt install -y libsqlite3-0 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/prank /usr/local/bin/prank
EXPOSE 8000
CMD ["prank-server"]
