#!/bin/bash
set -e

# Directory for local binaries
BIN_DIR="$(pwd)/bin"
mkdir -p "$BIN_DIR"

echo "Downloading linuxdeploy..."
wget -nc -O "$BIN_DIR/linuxdeploy" https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x "$BIN_DIR/linuxdeploy"

echo "Linuxdeploy installed to $BIN_DIR/linuxdeploy"
