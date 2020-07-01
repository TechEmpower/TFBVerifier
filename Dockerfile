FROM rust:latest

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY src src

RUN cargo build --release

CMD target/release/tfb_verifier
