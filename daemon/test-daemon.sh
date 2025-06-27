#!/bin/bash
# Test daemon locally without system installation

# Create temp directories
mkdir -p ./test-run/etc/cyrupd
mkdir -p ./test-run/var/log/cyrupd
mkdir -p ./test-run/var/run

# Copy test config
cp test-config.toml ./test-run/etc/cyrupd/cyrupd.toml

# Run daemon with test paths
export CYRUPD_CONFIG_PATH=./test-run/etc/cyrupd/cyrupd.toml
export CYRUPD_LOG_DIR=./test-run/var/log/cyrupd
export CYRUPD_PID_FILE=./test-run/var/run/cyrupd.pid

# Run in foreground
./target/release/cyrupd run --foreground