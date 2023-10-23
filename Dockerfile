FROM rust:1.73 as build
WORKDIR /usr/src/ascii_artinator

# build only dependencies to make docker build faster
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --release --locked
RUN rm src/main.rs

# build final binary
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 && apt clean && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/src/ascii_artinator/target/release/ascii_artinator ./
CMD ["./ascii_artinator"]
