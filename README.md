# ⚒️ EmuForge

EmuForge est une application de bureau moderne construite avec [Tauri](https://tauri.app/), [Rust](https://www.rust-lang.org/) et [React](https://react.dev/).

## 📋 Prérequis

Avant de commencer, assurez-vous d'avoir installé les outils nécessaires sur votre système.

### 🐧 Linux (Debian/Ubuntu/Mint)

Vous devez installer les dépendances de développement système :

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### 🪟 Windows

1. Installez **Microsoft Visual Studio C++ Build Tools** (disponible via l'installateur Visual Studio).
2. Assurez-vous de cocher "Développement Desktop C++".

### 🦀 Rust & Node.js (Toutes plateformes)

1. **Node.js** (v18 ou supérieur) : [Télécharger Node.js](https://nodejs.org/)
2. **Rust** (Stable) :

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

---

## 🚀 Installation

1. Clonez le projet :

    ```bash
    git clone https://github.com/votre-user/EmuForge.git
    cd EmuForge
    ```

2. Installez les dépendances JavaScript (Frontend) :

    ```bash
    cd ui
    npm install
    ```

---

## 🎮 Lancer en Développement

Pour lancer l'application avec le rechargement à chaud (HMR) :

```bash
cd ui
npm run tauri dev
```

> **Note :** La première compilation peut prendre quelques minutes le temps de compiler toutes les dépendances Rust.

---

## 📦 Compiler pour la Production

Pour créer un exécutable optimisé et standalone :

### Linux / macOS

```bash
./build.sh
```

ou manuellement :

```bash
cd ui
npm run tauri build
```

Les exécutables seront dans `src-tauri/target/release/bundle/`.

### Windows

```powershell
cd ui
npm run tauri build
```

L'installateur `.msi` ou l'exécutable `.exe` sera généré dans `src-tauri/target/release/bundle/nsis/`.
