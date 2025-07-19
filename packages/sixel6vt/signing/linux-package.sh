#!/bin/bash
# Linux Package Creation and Signing Script for rio-ext-test

set -euo pipefail

# Configuration
APP_NAME="rio-ext-test"
VERSION="0.1.0"
MAINTAINER="${LINUX_MAINTAINER:-SweetMCP <support@sweetmcp.com>}"
DESCRIPTION="Advanced terminal emulator with sixel graphics support"
HOMEPAGE="https://github.com/sweetmcp/rio-ext-test"

# GPG signing configuration
GPG_KEY_ID="${LINUX_GPG_KEY_ID:-}"
GPG_PASSPHRASE="${LINUX_GPG_PASSPHRASE:-}"

# Paths
BINARY_PATH="target/release/${APP_NAME}"
PACKAGE_DIR="target/packages"
DEB_DIR="$PACKAGE_DIR/deb"
RPM_DIR="$PACKAGE_DIR/rpm"
TAR_DIR="$PACKAGE_DIR/tar"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if [ ! -f "$BINARY_PATH" ]; then
        log_error "Binary not found at $BINARY_PATH"
        log_error "Please run 'cargo build --release' first"
        exit 1
    fi
    
    # Create package directories
    mkdir -p "$DEB_DIR" "$RPM_DIR" "$TAR_DIR"
    
    return 0
}

create_deb_package() {
    log_info "Creating DEB package..."
    
    local deb_build_dir="$DEB_DIR/build"
    local deb_root="$deb_build_dir/${APP_NAME}_${VERSION}"
    
    # Create directory structure
    mkdir -p "$deb_root/DEBIAN"
    mkdir -p "$deb_root/usr/bin"
    mkdir -p "$deb_root/usr/share/applications"
    mkdir -p "$deb_root/usr/share/doc/$APP_NAME"
    
    # Copy binary
    cp "$BINARY_PATH" "$deb_root/usr/bin/"
    chmod +x "$deb_root/usr/bin/$APP_NAME"
    
    # Create control file
    cat > "$deb_root/DEBIAN/control" << EOF
Package: $APP_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.31), libgcc-s1 (>= 3.0), libstdc++6 (>= 3.4)
Maintainer: $MAINTAINER
Description: $DESCRIPTION
 Advanced terminal emulator with sixel graphics support,
 built with modern Rust technologies for performance and reliability.
Homepage: $HOMEPAGE
EOF
    
    # Create desktop entry
    cat > "$deb_root/usr/share/applications/$APP_NAME.desktop" << EOF
[Desktop Entry]
Name=Rio Ext Test
Comment=$DESCRIPTION
Exec=$APP_NAME
Icon=terminal
Type=Application
Categories=System;TerminalEmulator;
StartupNotify=true
EOF
    
    # Create copyright file
    cat > "$deb_root/usr/share/doc/$APP_NAME/copyright" << EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: $APP_NAME
Upstream-Contact: $MAINTAINER
Source: $HOMEPAGE

Files: *
Copyright: 2024 SweetMCP
License: MIT
EOF
    
    # Build the package
    local deb_file="$DEB_DIR/${APP_NAME}_${VERSION}_amd64.deb"
    dpkg-deb --build --root-owner-group "$deb_root" "$deb_file" 2>/dev/null || {
        log_warn "dpkg-deb not available, creating manual DEB structure"
        tar -czf "$deb_file" -C "$deb_build_dir" "${APP_NAME}_${VERSION}"
    }
    
    log_info "DEB package created: $deb_file"
    
    # Sign the package if GPG key is available
    sign_deb_package "$deb_file"
    
    # Clean up build directory
    rm -rf "$deb_build_dir"
}

sign_deb_package() {
    local deb_file="$1"
    
    if [ -z "$GPG_KEY_ID" ]; then
        log_warn "GPG_KEY_ID not set, skipping DEB signing"
        return 0
    fi
    
    if ! command -v gpg &> /dev/null; then
        log_warn "GPG not found, skipping DEB signing"
        return 0
    fi
    
    log_info "Signing DEB package with GPG key: $GPG_KEY_ID"
    
    # Create detached signature
    local sig_file="${deb_file}.sig"
    if [ -n "$GPG_PASSPHRASE" ]; then
        echo "$GPG_PASSPHRASE" | gpg --batch --yes --passphrase-fd 0 \
            --default-key "$GPG_KEY_ID" --detach-sign --armor \
            --output "$sig_file" "$deb_file"
    else
        gpg --default-key "$GPG_KEY_ID" --detach-sign --armor \
            --output "$sig_file" "$deb_file"
    fi
    
    log_info "DEB signature created: $sig_file"
}

create_rpm_package() {
    log_info "Creating RPM package..."
    
    # Check for rpmbuild
    if ! command -v rpmbuild &> /dev/null; then
        log_warn "rpmbuild not found, skipping RPM creation"
        return 0
    fi
    
    local rpm_build_dir="$RPM_DIR/build"
    local spec_file="$rpm_build_dir/SPECS/$APP_NAME.spec"
    
    # Create RPM build structure
    mkdir -p "$rpm_build_dir"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
    
    # Copy binary to SOURCES
    cp "$BINARY_PATH" "$rpm_build_dir/SOURCES/"
    
    # Create spec file
    cat > "$spec_file" << EOF
Name:           $APP_NAME
Version:        $VERSION
Release:        1%{?dist}
Summary:        $DESCRIPTION
License:        MIT
URL:            $HOMEPAGE
Source0:        %{name}
BuildArch:      x86_64

%description
Advanced terminal emulator with sixel graphics support,
built with modern Rust technologies for performance and reliability.

%prep
# No preparation needed for binary package

%build
# No build needed for binary package

%install
mkdir -p %{buildroot}/usr/bin
cp %{SOURCE0} %{buildroot}/usr/bin/%{name}
chmod +x %{buildroot}/usr/bin/%{name}

mkdir -p %{buildroot}/usr/share/applications
cat > %{buildroot}/usr/share/applications/%{name}.desktop << 'DESKTOP_EOF'
[Desktop Entry]
Name=Rio Ext Test
Comment=$DESCRIPTION
Exec=%{name}
Icon=terminal
Type=Application
Categories=System;TerminalEmulator;
StartupNotify=true
DESKTOP_EOF

%files
/usr/bin/%{name}
/usr/share/applications/%{name}.desktop

%changelog
* $(date +"%%a %%b %%d %%Y") $MAINTAINER - $VERSION-1
- Initial package
EOF
    
    # Build the RPM
    rpmbuild --define "_topdir $rpm_build_dir" -bb "$spec_file"
    
    # Move RPM to target location
    local rpm_file=$(find "$rpm_build_dir/RPMS" -name "*.rpm" -type f)
    if [ -n "$rpm_file" ]; then
        mv "$rpm_file" "$RPM_DIR/"
        log_info "RPM package created: $(basename "$rpm_file")"
        
        # Sign the RPM if GPG key is available
        sign_rpm_package "$RPM_DIR/$(basename "$rpm_file")"
    fi
    
    # Clean up build directory
    rm -rf "$rpm_build_dir"
}

sign_rpm_package() {
    local rpm_file="$1"
    
    if [ -z "$GPG_KEY_ID" ]; then
        log_warn "GPG_KEY_ID not set, skipping RPM signing"
        return 0
    fi
    
    if ! command -v rpm &> /dev/null; then
        log_warn "rpm command not found, skipping RPM signing"
        return 0
    fi
    
    log_info "Signing RPM package with GPG key: $GPG_KEY_ID"
    
    # RPM signing requires rpm-sign package and proper configuration
    if command -v rpmsign &> /dev/null; then
        rpmsign --define "_gpg_name $GPG_KEY_ID" --addsign "$rpm_file"
        log_info "RPM signed successfully"
    else
        log_warn "rpmsign not available, creating detached signature instead"
        local sig_file="${rpm_file}.sig"
        if [ -n "$GPG_PASSPHRASE" ]; then
            echo "$GPG_PASSPHRASE" | gpg --batch --yes --passphrase-fd 0 \
                --default-key "$GPG_KEY_ID" --detach-sign --armor \
                --output "$sig_file" "$rpm_file"
        else
            gpg --default-key "$GPG_KEY_ID" --detach-sign --armor \
                --output "$sig_file" "$rpm_file"
        fi
        log_info "RPM signature created: $sig_file"
    fi
}

create_tar_package() {
    log_info "Creating TAR.XZ package..."
    
    local tar_build_dir="$TAR_DIR/build"
    local tar_root="$tar_build_dir/${APP_NAME}-${VERSION}"
    
    # Create directory structure
    mkdir -p "$tar_root/bin"
    mkdir -p "$tar_root/share/applications"
    mkdir -p "$tar_root/share/doc"
    
    # Copy binary
    cp "$BINARY_PATH" "$tar_root/bin/"
    chmod +x "$tar_root/bin/$APP_NAME"
    
    # Create desktop entry
    cat > "$tar_root/share/applications/$APP_NAME.desktop" << EOF
[Desktop Entry]
Name=Rio Ext Test
Comment=$DESCRIPTION
Exec=$APP_NAME
Icon=terminal
Type=Application
Categories=System;TerminalEmulator;
StartupNotify=true
EOF
    
    # Create README
    cat > "$tar_root/README.md" << EOF
# $APP_NAME

$DESCRIPTION

## Installation

1. Extract the archive to your preferred location
2. Add the bin directory to your PATH
3. Run \`$APP_NAME\` from terminal

## Version

$VERSION

## Homepage

$HOMEPAGE
EOF
    
    # Create the archive
    local tar_file="$TAR_DIR/${APP_NAME}-${VERSION}-linux-x86_64.tar.xz"
    tar -cJf "$tar_file" -C "$tar_build_dir" "${APP_NAME}-${VERSION}"
    
    log_info "TAR.XZ package created: $tar_file"
    
    # Sign the archive
    sign_tar_package "$tar_file"
    
    # Clean up build directory
    rm -rf "$tar_build_dir"
}

sign_tar_package() {
    local tar_file="$1"
    
    if [ -z "$GPG_KEY_ID" ]; then
        log_warn "GPG_KEY_ID not set, skipping TAR signing"
        return 0
    fi
    
    if ! command -v gpg &> /dev/null; then
        log_warn "GPG not found, skipping TAR signing"
        return 0
    fi
    
    log_info "Signing TAR package with GPG key: $GPG_KEY_ID"
    
    # Create detached signature
    local sig_file="${tar_file}.sig"
    if [ -n "$GPG_PASSPHRASE" ]; then
        echo "$GPG_PASSPHRASE" | gpg --batch --yes --passphrase-fd 0 \
            --default-key "$GPG_KEY_ID" --detach-sign --armor \
            --output "$sig_file" "$tar_file"
    else
        gpg --default-key "$GPG_KEY_ID" --detach-sign --armor \
            --output "$sig_file" "$tar_file"
    fi
    
    log_info "TAR signature created: $sig_file"
    
    # Create checksum file
    local checksum_file="${tar_file}.sha256"
    sha256sum "$tar_file" > "$checksum_file"
    log_info "Checksum created: $checksum_file"
}

main() {
    log_info "Starting Linux packaging process for $APP_NAME"
    
    check_prerequisites
    
    create_deb_package
    create_rpm_package
    create_tar_package
    
    log_info "Linux packaging process completed"
    log_info "Packages created in: $PACKAGE_DIR"
    
    # List created packages
    find "$PACKAGE_DIR" -type f \( -name "*.deb" -o -name "*.rpm" -o -name "*.tar.xz" \) -exec basename {} \; | while read -r file; do
        log_info "  - $file"
    done
}

main "$@"