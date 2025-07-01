FROM rust:1.86-bullseye AS builder

# Install system dependencies for chromiumoxide, bevy, and graphical libraries
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    libx11-dev \
    libxcb-randr0-dev \
    libxcb-xtest0-dev \
    libxcb-xinerama0-dev \
    libxcb-shape0-dev \
    libxcb-xkb-dev \
    libxcb-render0-dev \
    libxcb-xinput-dev \
    libxcb1-dev \
    libxcb-xfixes0-dev \
    libxcb-icccm4-dev \
    libxcb-dri2-0-dev \
    libxcb-dri3-dev \
    libxkbcommon-dev \
    libxkbcommon-x11-dev \
    libasound2-dev \
    libudev-dev \
    libwayland-dev \
    libgbm-dev \
    libglib2.0-dev \
    libfreetype6-dev \
    libfontconfig1-dev \
    libdrm-dev \
    chromium \
    && rm -rf /var/lib/apt/lists/*

# Set up Rust toolchain for WASM
RUN rustup target add wasm32-wasip1 && \
    rustup component add rust-std --target wasm32-wasip1 && \
    cargo install cargo-auditable

# Set environment variables for headless rendering
ENV DISPLAY=:99
ENV HEADLESS=1
ENV BEVY_ASSET_ROOT=/workspace

WORKDIR /workspace
COPY . .

# Fetch dependencies
RUN cargo fetch

# Build with special flags for Bevy in a headless environment
RUN RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" \
    cargo auditable build --release --target wasm32-wasip1

FROM scratch
WORKDIR /
COPY --from=builder /workspace/target/wasm32-wasip1/release/fetch.wasm /plugin.wasm
