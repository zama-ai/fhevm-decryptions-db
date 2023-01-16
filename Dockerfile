FROM rust:buster
COPY . .
ENV RUST_LOG=info
RUN apt-get update && apt-get install -y clang
RUN cargo build 
CMD cargo run 