FROM rust:alpine

WORKDIR /app

COPY . .

RUN apk add --no-cache musl-dev
RUN apk add --no-cache libressl-dev
RUN rustup default nightly

RUN cargo build --release

# CMD ["./target/release/ceksiak"]
