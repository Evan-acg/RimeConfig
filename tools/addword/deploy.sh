#!/bin/bash

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ "$1" != "--no-build" ]; then
    echo "Building release..."
    cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"
fi

SOURCE="$PROJECT_DIR/target/release/adw"
DEST="/usr/local/bin/adw"

if [ ! -f "$SOURCE" ]; then
    echo "Error: not found: $SOURCE"
    exit 1
fi

cp "$SOURCE" "$DEST"
chmod +x "$DEST"
echo "Deployed to $DEST"

echo ""
echo "Done! Use 'adw' from anywhere."
