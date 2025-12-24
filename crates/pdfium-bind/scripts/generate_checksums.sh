#!/bin/bash

# Configuration
VERSION_DYN="7592"
VERSION_STATIC="7442c"

echo "Calculating checksums for dynamic binaries (bblanchon)..."
PLATFORMS=(
    "mac-arm64"
    "mac-x64"
    "linux-x64"
    "linux-arm64"
    "win-x64"
    "win-arm64"
    "win-x86"
)

for PLATFORM in "${PLATFORMS[@]}"; do
    FILENAME="pdfium-${PLATFORM}.tgz"
    URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F${VERSION_DYN}/${FILENAME}"
    echo "Processing ${FILENAME}..."
    curl -L -s -O "${URL}"
    shasum -a 256 "${FILENAME}"
    rm "${FILENAME}"
done

echo ""
echo "Calculating checksum for static binary (paulocoutinhox)..."
URL_STATIC="https://github.com/paulocoutinhox/pdfium-lib/releases/download/${VERSION_STATIC}/macos.tgz"
echo "Processing macos.tgz..."
curl -L -s -O "${URL_STATIC}"
shasum -a 256 "macos.tgz"
rm "macos.tgz"
