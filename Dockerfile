FROM rust:1.53.0-alpine3.13 as builder
WORKDIR /usr/src/voters
COPY Cargo.toml ./
COPY src  ./src
RUN apk add --no-cache musl-dev sqlite-dev
RUN cargo install --path .

FROM alpine:3.13
WORKDIR /opt/voters
COPY Rocket.toml ./
COPY static ./static
COPY templates ./templates
RUN apk add --no-cache sqlite
COPY --from=builder /usr/local/cargo/bin/voters /usr/local/bin/voters
EXPOSE 8000
CMD ["voters"]
