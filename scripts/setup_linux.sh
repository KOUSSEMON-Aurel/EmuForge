#!/bin/bash

# EmuForge - Linux Dependency Setup Script
# Installs necessary dependencies for building and running Tauri apps

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}🔧 EmuForge Dependency Installer${NC}"

if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Please run as root (use sudo)${NC}"
  exit 1
fi

DISTRO=""

if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
fi

echo "Detailed Info: Detected Distro: $DISTRO"

case $DISTRO in
    ubuntu|debian|pop|linuxmint|kali)
        echo "Debian/Ubuntu based system detected."
        apt-get update
        apt-get install -y libwebkit2gtk-4.1-dev \
            build-essential \
            curl \
            wget \
            file \
            libssl-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev
        ;;
    fedora)
        echo "Fedora detected."
        dnf check-update
        dnf install -y webkit2gtk4.1-devel \
            openssl-devel \
            curl \
            wget \
            file \
            libappindicator-gtk3-devel \
            librsvg2-devel
        sudo dnf group install -y "C Development Tools and Libraries"
        ;;
    arch|manjaro)
        echo "Arch Linux based system detected."
        pacman -Syu --noconfirm
        pacman -S --noconfirm \
            webkit2gtk-4.1 \
            base-devel \
            curl \
            wget \
            file \
            openssl \
            appmenu-gtk-module \
            gtk3 \
            libappindicator-gtk3 \
            librsvg \
            libvips
        ;;
    *)
        echo -e "${RED}Unsupported distribution: $DISTRO${NC}"
        echo "Please manually install 'webkit2gtk-4.1' and build tools."
        exit 1
        ;;
esac

echo -e "${GREEN}✅ System dependencies installed successfully!${NC}"

# Install Node dependencies
if [ -d "ui" ]; then
    echo "Installing UI dependencies..."
    npm install --prefix ui
else
    echo -e "${RED}UI directory not found. Skipping npm install.${NC}"
fi

echo -e "${GREEN}🎉 Setup complete!${NC}"
echo "You can now run: npm run tauri dev --prefix ui"

