#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# DM+ Firmware Studio & toseven_pack Local Release Packager
# ==============================================================================
# Builds release binaries for the CURRENT host architecture and packages them
# into dist/ with SHA-256 checksums.
#
# NOTE: Cross-compiling GUI binaries (macOS -> Linux/Windows) locally requires
# heavy cross-sysroots and C libraries. For full 3-OS matrix builds (Windows,
# macOS, Linux), use GitHub Actions (.github/workflows/release.yml).
# ==============================================================================

VERSION="0.3.0"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${REPO_ROOT}/dist"
STAGE_DIR="${REPO_ROOT}/target/stage"

OS="$(uname -s)"
ARCH="$(uname -m)"

echo "=== Packaging DM+ Firmware Toolchain for ${OS} (${ARCH}) ==="

mkdir -p "${DIST_DIR}"
rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}"

# 1. Build CLI release binary
echo "--> Building release CLI (toseven_pack)..."
cargo build --manifest-path "${REPO_ROOT}/dm01-firmware/Cargo.toml" --bin toseven_pack --release

# 2. Build GUI release binary
echo "--> Building release Desktop GUI (dm-plus-gui)..."
cargo build --manifest-path "${REPO_ROOT}/dm-plus-gui/Cargo.toml" --release

# 3. Package based on operating system
if [ "${OS}" = "Darwin" ]; then
    CLI_BIN="${REPO_ROOT}/target/release/toseven_pack"
    GUI_BIN="${REPO_ROOT}/target/release/dm-plus-gui"
    
    # CLI archive
    CLI_ARCHIVE="${DIST_DIR}/toseven_pack-macos-${ARCH}.tar.gz"
    mkdir -p "${STAGE_DIR}/cli"
    cp "${CLI_BIN}" "${STAGE_DIR}/cli/"
    cp "${REPO_ROOT}/README.md" "${STAGE_DIR}/cli/"
    tar -czf "${CLI_ARCHIVE}" -C "${STAGE_DIR}/cli" .
    echo "✓ Packaged CLI: ${CLI_ARCHIVE}"

    # GUI macOS .app Bundle
    APP_DIR="${STAGE_DIR}/gui/DM+ Firmware Studio.app/Contents"
    mkdir -p "${APP_DIR}/MacOS"
    mkdir -p "${APP_DIR}/Resources"
    cp "${GUI_BIN}" "${APP_DIR}/MacOS/"
    cp "${REPO_ROOT}/dm-plus-gui/assets/AppIcon.icns" "${APP_DIR}/Resources/AppIcon.icns"
    
    cat << 'EOF' > "${APP_DIR}/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>dm-plus-gui</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>org.toseven.dmplus</string>
    <key>CFBundleName</key>
    <string>DM+ Firmware Studio</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.3.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

    cp "${REPO_ROOT}/README.md" "${STAGE_DIR}/gui/"
    GUI_ARCHIVE="${DIST_DIR}/dm-plus-gui-macos-${ARCH}.zip"
    (cd "${STAGE_DIR}/gui" && zip -q -r -y "${GUI_ARCHIVE}" .)
    echo "✓ Packaged GUI App: ${GUI_ARCHIVE}"

elif [ "${OS}" = "Linux" ]; then
    CLI_BIN="${REPO_ROOT}/target/release/toseven_pack"
    GUI_BIN="${REPO_ROOT}/target/release/dm-plus-gui"
    
    CLI_ARCHIVE="${DIST_DIR}/toseven_pack-linux-${ARCH}.tar.gz"
    mkdir -p "${STAGE_DIR}/cli"
    cp "${CLI_BIN}" "${STAGE_DIR}/cli/"
    cp "${REPO_ROOT}/README.md" "${STAGE_DIR}/cli/"
    tar -czf "${CLI_ARCHIVE}" -C "${STAGE_DIR}/cli" .
    echo "✓ Packaged CLI: ${CLI_ARCHIVE}"

    GUI_ARCHIVE="${DIST_DIR}/dm-plus-gui-linux-${ARCH}.tar.gz"
    mkdir -p "${STAGE_DIR}/gui"
    cp "${GUI_BIN}" "${STAGE_DIR}/gui/"
    cp "${REPO_ROOT}/dm-plus-gui/assets/logo.png" "${STAGE_DIR}/gui/"
    cp "${REPO_ROOT}/dm-plus-gui/assets/logo.svg" "${STAGE_DIR}/gui/"
    cp "${REPO_ROOT}/README.md" "${STAGE_DIR}/gui/"
    cat << 'EOF' > "${STAGE_DIR}/gui/dm-plus-gui.desktop"
[Desktop Entry]
Type=Application
Name=DM+ Firmware Studio
Comment=ToSeven Mid-Drive Motor Tuning & Display Patching Tool
Exec=dm-plus-gui
Icon=logo
Terminal=false
Categories=Utility;HardwareSettings;
EOF
    tar -czf "${GUI_ARCHIVE}" -C "${STAGE_DIR}/gui" .
    echo "✓ Packaged GUI: ${GUI_ARCHIVE}"

else
    echo "Unsupported host OS for local packaging script: ${OS}."
    echo "Use GitHub Actions (.github/workflows/release.yml) for cross-platform builds."
    exit 1
fi

# Clean temporary stage directory
rm -rf "${STAGE_DIR}"

# Compute Checksums
echo "--> Generating SHA-256 Checksums..."
(
    cd "${DIST_DIR}"
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 *.* > SHA256SUMS.txt
    elif command -v sha256sum >/dev/null 2>&1; then
        sha256sum *.* > SHA256SUMS.txt
    fi
    cat SHA256SUMS.txt
)

echo "=== Release packaging complete. Output saved to dist/ ==="

