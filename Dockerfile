FROM rust:buster
COPY . .
RUN apt-get update && apt-get install -y clang
RUN cargo build 
CMD cargo run 