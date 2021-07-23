FROM rust:1.53.0-alpine3.13
WORKDIR /usr/src/voters


COPY . .

RUN apk add --no-cache musl-dev sqlite-dev
RUN cargo install --path .

EXPOSE 8000

CMD ["voters"]
