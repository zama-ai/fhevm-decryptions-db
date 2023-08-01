FROM rust:1.70-bullseye AS builder
RUN apt-get update && apt-get install -y clang

WORKDIR /usr/local/app
ADD . .
RUN cargo build --release 

FROM rust:1.70-slim-bullseye
WORKDIR /usr/local/app
RUN apt-get update && apt-get install -y clang
RUN apt-get install libc6 -y
COPY --from=builder /usr/local/app/target/release/fhevm-decryptions-db .
COPY --from=builder /usr/local/app/Rocket.toml .

EXPOSE 8001/tcp

CMD ["/usr/local/app/fhevm-decryptions-db"]
