FROM rust:1.76-slim AS builder

RUN rustup target add wasm32-wasip1 && \
    rustup component add rust-std --target wasm32-wasip1

WORKDIR /workspace
COPY . .
RUN cargo build --release --target wasm32-wasip1

# Final image
FROM scratch
WORKDIR /
COPY --from=builder /workspace/target/wasm32-wasip1/release/mcp_reasoner_plugin.wasm /plugin.wasm