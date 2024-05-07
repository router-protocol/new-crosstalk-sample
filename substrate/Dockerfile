# Use the Rust 1.76.0 image as the base image
FROM rust:1.76.0

WORKDIR /usr/src/app

# Set the default Rust toolchain to stable
RUN rustup default stable

# Update the stable toolchain
RUN rustup update stable

# Add the wasm32-unknown-unknown target to the stable toolchain
RUN rustup target add wasm32-unknown-unknown

# Update the nightly toolchain
RUN rustup update nightly

# Add the wasm32-unknown-unknown target to the nightly toolchain
RUN rustup target add wasm32-unknown-unknown --toolchain nightly

RUN rustup component add clippy

# Install cargo-contract
RUN cargo install --force --locked cargo-contract --version 3.2.0

# Install openssl (assuming this is for Linux-based systems, adjust as necessary)
RUN apt-get update && apt-get install -y openssl

# Install cargo-dylint and dylint-link
RUN cargo install cargo-dylint dylint-link

RUN rustup component add rust-src --toolchain stable-aarch64-unknown-linux-gnu

RUN export CARGO_NET_GIT_FETCH_WITH_CLI=true

#  BUILD IMAGE
# docker build -t substrate .
# BUILD CONTRACT
# docker run --rm -v $(pwd):/usr/src/app  substrate sh -c "cd contracts/gateway && cargo contract build --release && cd ../route_token && cargo contract build --release && ../dapp && cargo contract build --release"







