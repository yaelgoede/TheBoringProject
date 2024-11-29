FROM rust:latest

WORKDIR /usr/src/connector
COPY connector .

RUN apt-get update && apt-get install -y libssl-dev build-essential cmake

RUN cargo build --release
CMD ["./target/release/connector"]