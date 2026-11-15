# EchoGrid

**Visual Multi Ping** — Cross-platform ping & TCP port monitoring, built with Tauri 2 & Rust.

![EchoGrid](https://img.shields.io/badge/built%20with-Tauri%202%20%2B%20Rust-blue)

## ✨ Features

- 🔵 **ICMP Ping** multi-hôtes continu
- 🟢 **TCP Port monitoring** — vérifie si un port est ouvert/fermé
- 🎨 **Interface colorée** — vert = UP, rouge = DOWN, orange = erreur
- 📐 **Grille dynamique** auto-redimensionnable
- 📊 **Barre de stats** — nombre d'hôtes, up/down en temps réel
- ⚙️ **Configurable** — intervalle, timeout, TTL, taille paquet
- 🖥️ **Cross-platform** — Windows · Linux · macOS
- 🪶 **Léger** — un seul binaire, ~3 Mo

## 🚀 Quick Start

```bash
# Installer les dépendances
npm install

# Mode développement
npm run dev

# Build production
npm run build
```

## 📋 Prérequis

| Outil | Version |
|-------|---------|
| Rust  | 1.70+ (via rustup) |
| Node.js | 18+ |
| npm | 9+ |

### Linux (dépendances système)

```bash
# Ubuntu / Debian
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel openssl-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel
```

### Linux — ICMP capabilities

Le ping ICMP sous Linux nécessite `CAP_NET_RAW` :

```bash
# Après le build release
sudo setcap cap_net_raw+ep src-tauri/target/release/echogrid

# Ou lancer avec sudo (développement)
sudo npm run dev
```

## 🏗️ Architecture

```
EchoGrid/
├── package.json
├── src/                      # Frontend (HTML/CSS/JS vanilla)
│   ├── index.html
│   ├── style.css
│   └── app.js
└── src-tauri/                # Backend Rust
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── build.rs
    └── src/
        ├── main.rs           # Point d'entrée Tauri
        ├── commands.rs       # Commandes IPC exposées au front
        ├── ping.rs           # Ping ICMP cross-platform (surge-ping)
        └── tcp_monitor.rs    # Monitoring TCP (tokio)
```

| Couche | Techno | Rôle |
|--------|--------|------|
| Frontend | HTML/CSS/JS vanilla | Grille, tuiles colorées, modale settings |
| Bridge | Tauri 2 IPC | `start_ping`, `stop_ping`, `start_tcp_monitor`… |
| Backend | Rust + Tokio | Ping ICMP, monitoring TCP, tâches async annulables |
| Events | `emit`/`listen` | Résultats temps réel poussés du Rust vers le JS |

## 📦 Crates Rust utilisées

- **`tauri`** — framework desktop cross-platform
- **`tokio`** — runtime async (tâches de ping parallèles)
- **`surge-ping`** — ping ICMP cross-platform (raw socket Linux, `IcmpSendEcho` Win32)
- **`serde`** — sérialisation JSON pour l'IPC

## 📄 License

MIT
