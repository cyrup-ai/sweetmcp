# Multi-stage Dockerfile for PTY Terminal cross-platform builds
# This Dockerfile creates a build environment that can produce binaries for multiple platforms

# Build stage for Linux builds
FROM rust:1.75-slim as linux-builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.0-dev \
    libfontconfig1-dev \
    libfreetype6-dev \
    libasound2-dev \
    libudev-dev \
    wget \
    file \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install cross-compilation tools
RUN rustup component add rustfmt clippy
RUN cargo install cross --git https://github.com/cross-rs/cross

# Add additional targets
RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup target add aarch64-unknown-linux-gnu

WORKDIR /app

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY build-installers.rs ./
COPY build-config.toml ./
COPY src/ ./src/
COPY assets/ ./assets/
COPY signing/ ./signing/
COPY scripts/ ./scripts/

# Build for Linux targets
RUN chmod +x scripts/build-local.sh
RUN ./scripts/build-local.sh --release --target x86_64-unknown-linux-gnu
RUN ./scripts/build-local.sh --release --target aarch64-unknown-linux-gnu

# Create installers
RUN cargo run --bin build-installers -- --platform linux-x86_64
RUN cargo run --bin build-installers -- --platform linux-aarch64

# Development build stage with additional tools
FROM linux-builder as development

# Install additional development tools
RUN cargo install cargo-audit cargo-outdated cargo-tree
RUN apt-get update && apt-get install -y \
    valgrind \
    gdb \
    strace \
    && rm -rf /var/lib/apt/lists/*

# Set up development environment
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=1

# Development command
CMD ["bash"]

# Production stage - minimal runtime
FROM ubuntu:22.04 as runtime

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    libgtk-3-0 \
    libayatana-appindicator3-1 \
    librsvg2-2 \
    libwebkit2gtk-4.0-37 \
    libfontconfig1 \
    libfreetype6 \
    libasound2 \
    libudev1 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash ptyuser

# Copy binary from builder
COPY --from=linux-builder /app/dist/rio-ext-test /usr/local/bin/pty-terminal
RUN chmod +x /usr/local/bin/pty-terminal

# Switch to non-root user
USER ptyuser
WORKDIR /home/ptyuser

# Default command
CMD ["/usr/local/bin/pty-terminal"]

# Windows cross-compilation stage (using Wine)
FROM linux-builder as windows-builder

# Install Wine and Windows dependencies
RUN dpkg --add-architecture i386
RUN apt-get update && apt-get install -y \
    wine64 \
    wine32 \
    winetricks \
    && rm -rf /var/lib/apt/lists/*

# Add Windows targets
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add aarch64-pc-windows-msvc

# Configure Wine
ENV WINEPREFIX=/wine
RUN winecfg

# Install Windows build tools (this would need additional setup)
# Note: Full Windows builds require Visual Studio Build Tools or MinGW

# macOS cross-compilation stage (limited - requires macOS SDK)
FROM linux-builder as macos-builder

# Note: Full macOS cross-compilation requires macOS SDK and additional setup
# This is a placeholder for a more complex setup that would require:
# - macOS SDK (legally obtained)
# - osxcross toolchain
# - Apple developer tools

# Add macOS targets (requires additional setup)
# RUN rustup target add x86_64-apple-darwin
# RUN rustup target add aarch64-apple-darwin

# Multi-platform builder that attempts all platforms
FROM linux-builder as multi-platform

COPY --from=windows-builder /app/target /app/target-windows
COPY --from=macos-builder /app/target /app/target-macos

# Build script that handles all platforms
COPY scripts/docker-build-all.sh ./scripts/
RUN chmod +x scripts/docker-build-all.sh

# Set build output directory
ENV BUILD_OUTPUT_DIR=/app/dist

# Create output directory
RUN mkdir -p $BUILD_OUTPUT_DIR

# Final stage for CI/CD
FROM multi-platform as ci

# Install additional CI tools
RUN cargo install cargo-tarpaulin cargo-benchcmp
RUN apt-get update && apt-get install -y \
    jq \
    curl \
    && rm -rf /var/lib/apt/lists/*

# CI script
COPY scripts/ci-build.sh ./scripts/
RUN chmod +x scripts/ci-build.sh

# Set CI environment
ENV CI=true
ENV RUST_LOG=info

# Default CI command
CMD ["./scripts/ci-build.sh"]

# Testing stage
FROM linux-builder as testing

# Install test dependencies
RUN apt-get update && apt-get install -y \
    xvfb \
    && rm -rf /var/lib/apt/lists/*

# Test environment setup
ENV DISPLAY=:99

# Test runner script
COPY scripts/run-tests-headless.sh ./scripts/
RUN chmod +x scripts/run-tests-headless.sh

CMD ["./scripts/run-tests-headless.sh"]

# Documentation builder
FROM rust:1.75-slim as docs

RUN apt-get update && apt-get install -y \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Generate documentation
RUN cargo doc --no-deps --document-private-items

# Serve documentation
FROM nginx:alpine as docs-server
COPY --from=docs /app/target/doc /usr/share/nginx/html
EXPOSE 80

# Default stage is the runtime
FROM runtime