FROM rust:1-slim

# The official image uses rustup's minimal profile, so lint/format tools aren't included.
RUN rustup component add clippy rustfmt

WORKDIR /workspace
