#!/bin/bash
# macOS Code Signing Script for rio-ext-test

set -euo pipefail

# Configuration
APP_NAME="rio-ext-test"
DEVELOPER_ID="${MACOS_DEVELOPER_ID:-}"
KEYCHAIN_PROFILE="${MACOS_KEYCHAIN_PROFILE:-}"
NOTARIZATION_TEAM_ID="${MACOS_TEAM_ID:-}"
BUNDLE_ID="com.sweetmcp.rio-ext-test"

# Paths
BINARY_PATH="target/release/${APP_NAME}"
SIGNED_BINARY_PATH="target/release/${APP_NAME}-signed"
DMG_PATH="target/release/${APP_NAME}.dmg"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if [ ! -f "$BINARY_PATH" ]; then
        log_error "Binary not found at $BINARY_PATH"
        log_error "Please run 'cargo build --release' first"
        exit 1
    fi
    
    if [ -z "$DEVELOPER_ID" ]; then
        log_warn "MACOS_DEVELOPER_ID not set, skipping code signing"
        return 1
    fi
    
    if ! command -v codesign &> /dev/null; then
        log_error "codesign not found. Please install Xcode command line tools"
        exit 1
    fi
    
    return 0
}

sign_binary() {
    log_info "Signing binary with Developer ID: $DEVELOPER_ID"
    
    # Copy binary for signing
    cp "$BINARY_PATH" "$SIGNED_BINARY_PATH"
    
    # Sign the binary
    codesign \
        --sign "$DEVELOPER_ID" \
        --timestamp \
        --options runtime \
        --verbose \
        "$SIGNED_BINARY_PATH"
    
    # Verify the signature
    codesign --verify --verbose "$SIGNED_BINARY_PATH"
    log_info "Binary signed successfully"
}

notarize_binary() {
    if [ -z "$KEYCHAIN_PROFILE" ] || [ -z "$NOTARIZATION_TEAM_ID" ]; then
        log_warn "Keychain profile or team ID not set, skipping notarization"
        return 0
    fi
    
    log_info "Creating ZIP for notarization..."
    zip -j "target/release/${APP_NAME}.zip" "$SIGNED_BINARY_PATH"
    
    log_info "Submitting for notarization..."
    xcrun notarytool submit \
        "target/release/${APP_NAME}.zip" \
        --keychain-profile "$KEYCHAIN_PROFILE" \
        --team-id "$NOTARIZATION_TEAM_ID" \
        --wait
    
    log_info "Notarization completed"
    rm "target/release/${APP_NAME}.zip"
}

create_dmg() {
    log_info "Creating DMG installer..."
    
    # Create temporary directory for DMG contents
    TEMP_DIR=$(mktemp -d)
    APP_DIR="$TEMP_DIR/${APP_NAME}"
    mkdir -p "$APP_DIR"
    
    # Copy signed binary
    cp "$SIGNED_BINARY_PATH" "$APP_DIR/$APP_NAME"
    
    # Create DMG
    hdiutil create -size 50m -volname "$APP_NAME" -srcfolder "$TEMP_DIR" -ov -format UDZO "$DMG_PATH"
    
    # Clean up
    rm -rf "$TEMP_DIR"
    
    log_info "DMG created at $DMG_PATH"
}

main() {
    log_info "Starting macOS signing process for $APP_NAME"
    
    if check_prerequisites; then
        sign_binary
        notarize_binary
        create_dmg
        log_info "macOS signing process completed successfully"
    else
        log_warn "Skipping signing due to missing prerequisites"
        # Create unsigned DMG for development
        if [ -f "$BINARY_PATH" ]; then
            cp "$BINARY_PATH" "$SIGNED_BINARY_PATH"
            create_dmg
            log_info "Created unsigned DMG for development"
        fi
    fi
}

main "$@"