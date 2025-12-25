#!/bin/bash
set -e

# Ensure linuxdeploy is present
if [ ! -f "bin/linuxdeploy" ]; then
    echo "Linuxdeploy not found. Running installer..."
    bash scripts/install_deps.sh
fi

# Add local bin to PATH
export PATH="$(pwd)/bin:$PATH"

# Workaround for running linuxdeploy AppImage without FUSE (common in containers/WSL)
export APPIMAGE_EXTRACT_AND_RUN=1

echo "🚀 Starting EmuForge Fast Build (No Bundling)..."
npm run tauri build --prefix ui -- --debug --no-bundle

echo "✅ Build Complete!"
echo "Exécutable disponible ici : target/debug/ui"
echo "(Pour créer les installateurs .deb/.AppImage, retirez '--no-bundle' dans ce script)"
