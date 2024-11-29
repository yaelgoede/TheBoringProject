FROM rust:latest

WORKDIR /usr/src/simulator
COPY simulator .

RUN apt-get update && apt-get install -y libssl-dev build-essential cmake

RUN cargo build --release
CMD ["./target/release/simulator"]