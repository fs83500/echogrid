# EchoGrid — Instructions pour l'agent

Clone cross-platform de **vmPing.exe** (Visual Multi Ping) construit avec **Tauri 2 + Rust**.
Ping ICMP multiple continu + monitoring ports TCP, interface colorée (vert=up, rouge=down, orange=erreur), grille dynamique, métriques cumulatives, notifications système.

---

## 📦 Versioning sémantique (SemVer)

Le projet suit **Semantic Versioning 2.0.0** (`MAJOR.MINOR.PATCH`). La version est synchronisée dans **3 fichiers** qui doivent toujours être bumpés ensemble :

| Fichier | Champ | Exemple |
|---------|-------|---------|
| `package.json` | `"version"` | `"1.2.0"` |
| `src-tauri/Cargo.toml` | `version` | `"1.2.0"` |
| `src-tauri/tauri.conf.json` | `"version"` | `"1.2.0"` |

### Règles de bump

| Type de changement | Bump | Exemples |
|--------------------|------|----------|
| Bug fix interne, pas de changement d'API | **PATCH** (+0.0.1) | Correction d'un événement bloqué, fix sérialisation |
| Nouvelle fonctionnalité **rétro-compatible** | **MINOR** (+0.1.0, reset PATCH=0) | Ajout métriques cumulatives, notifications système, nouveau champ dans un event payload |
| Changement **cassant** (breaking change) | **MAJOR** (+1.0.0, reset MINOR=0, PATCH=0) | Renommage/suppression d'une commande IPC, changement du schema d'un event payload existant |

### Procédure de bump

1. Déterminer le type (PATCH / MINOR / MAJOR)
2. Calculer la nouvelle version
3. Mettre à jour les 3 fichiers simultanément
4. Relancer `cargo test` pour vérifier
5. Mettre à jour `AGENTS.md` — historique des versions, artefacts, nouvelles sections si besoin
6. Mettre à jour `PROGRESSION.md` — résumé, fonctionnalités, build, tests, changelog
7. Builder : `CI=false npx tauri build --bundles deb`

### Historique des versions

| Version | Date | Type | Changements |
|---------|------|------|-------------|
| **1.0.0** | — | — | Version initiale : ping ICMP, monitoring TCP, grille colorée, validation, 63 tests |
| **1.0.1** | — | PATCH | Fix : événements Tauri 2 bloqués (capabilities `core:default` manquantes) |
| **1.1.0** | — | MINOR | Métriques cumulatives (lost, total, last_ok, latency min/max, jitter) + notifications système KO (plugin `tauri-plugin-notification`) + tracking focus fenêtre |
| **1.2.0** | 2026-11-14 | MINOR | Panneau historique des changements d'état (tableau scrollable, 500 entrées max) + fix style `<select>` thème sombre (flèche SVG, couleurs forcées) |

---

## 🏗️ Architecture

```
EchoGrid/
├── package.json              # npm, Tauri CLI, version SemVer
├── AGENTS.md                 # CE FICHIER (architecture, conventions, pièges)
├── PROGRESSION.md            # Suivi d'avancement, backlog, builds, changelog
├── src/                      # Frontend vanilla (pas de bundler)
│   ├── index.html            # Toolbar, grille tuiles, modale settings, panneau historique
│   ├── style.css             # Thème sombre GitHub-like, tuiles glow, métriques, has-loss, high-jitter, select custom, historique
│   └── app.js                # Logique JS : add/stop hosts, events Tauri, settings, notifications KO, focus tracking, métriques tuile, historique
└── src-tauri/                # Backend Rust
    ├── Cargo.toml            # tauri 2, tokio, surge-ping 0.8, serde, chrono, tauri-plugin-notification
    ├── tauri.conf.json       # Config Tauri (withGlobalTauri: true, version SemVer)
    ├── build.rs              # tauri_build::build()
    ├── capabilities/
    │   └── default.json      # Permissions Tauri 2 : core:default + notification:default
    ├── icons/                # 32x32.png, 128x128.png, 128x128@2x.png, icon.ico, icon.icns
    └── src/
        ├── main.rs           # Point d'entrée : 11 commandes IPC + plugin notification
        ├── commands.rs       # Commandes IPC + validation + déduplication + arrêt gracieux
        ├── ping.rs           # Ping ICMP via surge-ping, boucle async avec cancel + métriques cumulatives
        ├── tcp_monitor.rs    # Monitoring TCP via tokio::net::TcpStream, boucle async avec cancel + métriques cumulatives
        ├── validation.rs     # Validation pure + sanitisation (constantes, ValidationError, fonctions testables)
        └── tests.rs          # 63 tests unitaires
```

| Couche | Techno | Rôle |
|--------|--------|------|
| Frontend | HTML/CSS/JS vanilla | Grille, tuiles colorées, métriques, modale settings, notifications KO, panneau historique |
| Bridge | Tauri 2 IPC (`withGlobalTauri`) | `window.__TAURI__.core.invoke`, `window.__TAURI__.event.listen` |
| Backend | Rust + Tokio | Ping ICMP, monitoring TCP, tâches async annulables, métriques cumulatives |
| Events | `emit`/`listen` Tauri | Résultats temps réel poussés du Rust vers le JS |
| Notifications | `tauri-plugin-notification` | Notification système quand hôte passe KO et app pas au premier plan |

### Commandes IPC exposées

| Commande | Paramètres | Rôle |
|----------|-----------|------|
| `start_ping` | `{ config: PingConfig }` | Lance un ping ICMP continu (sanitise, valide, déduplique) |
| `stop_ping` | `{ host: String }` | Arrêt gracieux d'un ping (cancel.send + handle.await) |
| `stop_all_pings` | — | Arrêt gracieux de tous les pings |
| `start_tcp_monitor` | `{ config: TcpConfig }` | Lance un monitoring TCP continu |
| `stop_tcp_monitor` | `{ host, port }` | Arrêt gracieux d'un monitor TCP |
| `stop_all_tcp_monitors` | — | Arrêt gracieux de tous les monitors TCP |
| `get_settings` | — | Retourne AppSettings par défaut |
| `save_settings` | `{ settings: AppSettings }` | Valide et (TODO) persiste les settings |
| `validate_ping` | `{ config: PingConfig }` | Validation côté frontend |
| `validate_tcp` | `{ config: TcpConfig }` | Validation côté frontend |
| `validate_settings_cmd` | `{ settings: AppSettings }` | Validation côté frontend |

### Événements Tauri (Rust → JS)

| Événement | Payload | Quand |
|-----------|---------|------|
| `ping-result` | `PingResult { host, alive, latency_ms, error, lost_count, total_count, last_ok, latency_min, latency_max, jitter_ms }` | Chaque ping ICMP |
| `tcp-result` | `TcpResult { host, port, open, latency_ms, error, lost_count, total_count, last_ok, latency_min, latency_max, jitter_ms }` | Chaque check TCP |
| `ping-stopped` | `host: String` | Ping arrêté |
| `tcp-stopped` | `key: String` (host:port) | Monitor TCP arrêté |

### Métriques cumulatives (v1.1.0)

Chaque événement `ping-result` / `tcp-result` inclut les métriques cumulatives depuis le démarrage du ping/monitor :

| Champ | Type | Description |
|-------|------|-------------|
| `lost_count` | `u32` | Nombre de pings/checks échoués |
| `total_count` | `u32` | Nombre total de pings/checks effectués |
| `last_ok` | `Option<String>` | Date/heure du dernier ping/check réussi (ISO 8601, timezone locale via `chrono::Local`) |
| `latency_min` | `Option<f64>` | Latence minimum observée (ms) |
| `latency_max` | `Option<f64>` | Latence maximum observée (ms) |
| `jitter_ms` | `Option<f64>` | Jitter = moyenne des \|Δlatence\| entre pings consécutifs (ms) |

#### Affichage frontend

- **Tuile** : div `.metrics` avec 4 spans — pings perdus (`lost/total`), dernier OK, jitter, min/max
- **`has-loss`** (classe CSS rouge) : `lost_count > 0`
- **`high-jitter`** (classe CSS orange) : `jitter_ms > 50`

### Notifications système (v1.1.0)

Quand un hôte passe de UP → KO **et** l'application n'est pas au premier plan :
- Le backend émet l'événement avec `alive: false`
- Le frontend détecte la transition (`!alive && wasAlive`) et appelle `notifyKO(host, error)`
- `notifyKO` utilise `window.__TAURI__.notification.sendNotification()` (plugin `tauri-plugin-notification`)
- Le focus fenêtre est suivi via `visibilitychange`, `focus`/`blur` → variable `windowFocused`
- La permission de notification est demandée au démarrage via `requestPermission()`

### Constantes de validation (validation.rs)

| Constante | Valeur | Rôle |
|-----------|--------|------|
| `MIN_INTERVAL_MS` | 100 | Intervalle minimum (ping/TCP) |
| `MAX_INTERVAL_MS` | 60 000 | Intervalle maximum |
| `MIN_TIMEOUT_MS` | 50 | Timeout minimum |
| `MAX_TIMEOUT_MS` | 30 000 | Timeout maximum |
| `MIN_TTL` | 1 | TTL minimum |
| `MAX_TTL` | 255 | TTL maximum |
| `MIN_PACKET_SIZE` | 1 | Taille paquet minimum |
| `MAX_PACKET_SIZE` | 65 527 | Taille paquet maximum (limite ICMP) |
| `MIN_PORT` | 1 | Port minimum |
| `MAX_PORT` | 65 535 | Port maximum |

---

## 🔨 Build

### Prérequis

- **Rust** 1.70+ (via rustup)
- **Node.js** 18+ / **npm** 9+
- **Linux** : libwebkit2gtk-4.1-dev, libgtk-3-dev, libssl-dev, build-essential, libayatana-appindicator3-dev, librsvg2-dev

### Commandes

```bash
# Installer les dépendances npm (Tauri CLI)
npm install

# Vérifier la compilation Rust (rapide, pas de link)
cargo check --manifest-path src-tauri/Cargo.toml

# Build release (binaire uniquement)
cargo build --release --manifest-path src-tauri/Cargo.toml -j 1

# Build complet Tauri + bundle .deb
CI=false npx tauri build --bundles deb

# Build complet Tauri + tous les bundles (AppImage, deb, rpm…)
CI=false npx tauri build
```

> ⚠️ `CI=false` est **obligatoire** car l'environnement de build a `CI=1` ce qui casse le CLI Tauri (`--ci` attend `true`/`false`).
> ⚠️ `-j 1` pour le cargo build direct (limite mémoire 2 GiB).

### Artefacts produits

Les chemins contiennent la version SemVer courante (`{version}`) :

| Artefact | Chemin |
|----------|--------|
| Binaire release | `src-tauri/target/release/echogrid` (~21 Mo) |
| .deb | `src-tauri/target/release/bundle/deb/EchoGrid_{version}_amd64.deb` (~6 Mo) |

Exemple pour la v1.2.0 : `src-tauri/target/release/bundle/deb/EchoGrid_1.2.0_amd64.deb`

### ICMP sur Linux

Le ping ICMP nécessite `CAP_NET_RAW` :

```bash
sudo setcap cap_net_raw+ep src-tauri/target/release/echogrid
```

### Installation du .deb

```bash
sudo dpkg -i src-tauri/target/release/bundle/deb/EchoGrid_{version}_amd64.deb
```

Dépendances auto : `libwebkit2gtk-4.1-0`, `libgtk-3-0`.

---

## 🧪 Tests

### Commande

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### Résultat actuel : **63 tests passent**

### Catégories de tests (tests.rs)

Les tests sont organisés en 10 sections couvrant toutes les fonctions pures du backend :

| # | Catégorie | Nb tests | Ce qui est testé |
|---|-----------|----------|------------------|
| 1 | **Validation host** | 11 | IPv4, IPv6, DNS valides ; empty, whitespace-only, too long, invalid chars, label dash start/end, empty label, label too long, max length |
| 2 | **Validation PingConfig** | 12 | Config valide par défaut ; empty host, interval/timeout out of range, timeout > interval, TTL 0/256/boundary, packet size 0/too high, multiple errors simultanées |
| 3 | **Validation TcpConfig** | 5 | Config valide ; empty host, port 0, interval/timeout out of range |
| 4 | **Validation AppSettings** | 6 | Default valide ; timeout > interval, TTL/packet size/tcp timeout out of range, all valid |
| 5 | **Sanitisation PingConfig** | 7 | Clamp interval, timeout, TTL, packet size ; timeout capped to interval ; trim host ; valid config inchangé |
| 6 | **Sanitisation TcpConfig** | 4 | Clamp port, interval ; trim host ; valid config inchangé |
| 7 | **Sérialisation** | 4 | PingResult/TcpResult ↔ JSON (alive, error, port, latency + tous les champs métriques v1.1.0) |
| 8 | **State** | 2 | PingState/TcpState default vide (async) |
| 9 | **ValidationError Display** | 1 | Messages français exacts pour chaque variante |
| 10 | **Boundary values** | 6 | Limites exactes : interval min/max, timeout min, port min/max, settings all-at-limits |

### Comment ajouter un test

1. Les tests sont dans `src-tauri/src/tests.rs` (module `#[cfg(test)]` dans main.rs)
2. Ils testent les **fonctions pures** de `validation.rs` (pas de Tauri, pas de réseau)
3. Pour tester une nouvelle fonction de validation :
   - L'ajouter dans `validation.rs` (fonction pure, pas de side-effect)
   - Ajouter les tests dans `tests.rs` avec les cas : valide, invalide, boundary
4. Les tests async utilisent `#[tokio::test]`
5. Les tests de sérialisation vérifient `serde_json::to_string` + `from_str`

### Stratégie de test à respecter

| Couche | Comment tester |
|--------|---------------|
| **validation.rs** | Tests unitaires exhaustifs (fonctions pures) — c'est la couche la plus testée |
| **commands.rs** | Tests via IPC (nécessite un runtime Tauri) — pas encore testés directement |
| **ping.rs / tcp_monitor.rs** | Tests d'intégration réseau (nécessitent un serveur/hôte réel) — pas testables sans réseau |
| **app.js** | Tests manuels dans l'UI : ajouter un hôte, vérifier la tuile, stopper, settings |

### Ce qui n'est PAS encore testé automatiquement

- Commandes IPC (nécessitent un AppHandle Tauri)
- Boucles ping/TCP réelles (nécessitent le réseau)
- Frontend JS (nécessite un navigateur/webview)
- Persistance des settings (TODO dans commands.rs)

---

## ⚠️ Pièges connus et conventions

### Pièges déjà rencontrés et corrigés

| Piège | Solution |
|-------|----------|
| `window.__TAURI__` est `undefined` | `"withGlobalTauri": true` dans `tauri.conf.json` → injecte `window.__TAURI__.core` et `.event` |
| Événements `emit`/`listen` bloqués (Tauri 2) | Créer `src-tauri/capabilities/default.json` avec `"permissions": ["core:default"]` (inclut `core:event:default` → allow-listen, allow-emit) |
| `CI=1` dans l'environnement | Lancer avec `CI=false npx tauri build` |
| Icônes PNG malformées (mix RGB/RGBA) | Générer en RGBA pur (color type 6, 4 bytes/pixel) |
| `ToSocketAddrs::to_socket_addrs` comme méthode statique | Utiliser `tokio::net::lookup_host()` à la place |
| `surge-ping` API : `PingIdentifier(pub u16)`, `PingSequence(pub u16)` | Tuple structs valides en 0.8, constructeur direct OK |
| `watch::Sender` droppé prématurément | Stocker `cancel: Sender<bool>` dans `PingEntry`/`TcpEntry` |
| `stop` brutal via `abort()` | Arrêt gracieux : `cancel.send(true)` puis `handle.await` |
| Timeout > interval dans les tests | Toujours s'assurer `timeout_ms <= interval_ms` dans les configs de test |
| DNS label > 63 chars | Un hostname de 253 chars doit avoir des labels ≤ 63 séparés par des `.` |
| Notifications plugin non chargé | Ajouter `.plugin(tauri_plugin_notification::init())` dans `main.rs` + `"notification:default"` dans capabilities |
| `notification.sendNotification()` non disponible | Vérifier `window.__TAURI__.notification` existe (injecté par le plugin) + appeler `requestPermission()` au démarrage |

### Conventions de code

- **Rust** : edition 2021, `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
- **Validation** : fonctions pures dans `validation.rs`, testables sans Tauri
- **Sanitisation** : clamp avant validation, jamais de panic
- **Annulation** : `tokio::sync::watch::channel(false)` → `cancel.send(true)` + `handle.await`
- **Déduplication** : arrêter l'ancien task avant d'insérer le nouveau dans la HashMap
- **Frontend** : vanilla JS, pas de framework, pas de bundler — `window.__TAURI__` injecté par Tauri
- **Icônes** : RGBA PNG (color type 6), ICO avec DIB 32bpp, ICNS avec chunk `ic07`
- **Capabilities** : fichier `src-tauri/capabilities/default.json` obligatoire en Tauri 2 — sans `core:default`, les événements `emit`/`listen` sont bloqués
- **Versionning** : SemVer synchronisé dans 3 fichiers (`package.json`, `Cargo.toml`, `tauri.conf.json`) — toujours bumper les 3 ensemble
- **Documentation** : `AGENTS.md` (architecture, conventions) + `PROGRESSION.md` (avancement, backlog, builds, changelog) — les deux sont mis à jour à **chaque build et chaque feature**
- **Métriques** : cumulatives depuis le démarrage du ping/monitor (reset à l'arrêt), calculées dans la boucle async Rust
- **Notifications** : uniquement sur transition UP→KO + app pas au premier plan (évite le spam)
- **Historique** : tableau scrollable des changements d'état (max 500 entrées), colonnes Heure/Hôte/État/Latence/Détail, couleurs UP=vert/DOWN=rouge/ERR=orange
- **Select custom** : `-webkit-appearance: none` + flèche SVG intégrée + `option` colors forcées pour thème sombre

---

## 📋 TODO restant

- [ ] Persistance des settings (lire/écrire via `app_config_dir` dans commands.rs)
- [ ] Persistance des hôtes côté frontend (localStorage ou fichier)
- [ ] Gestion d'erreur frontend (afficher une notification si `invoke()` échoue)
- [ ] Tests d'intégration IPC (via Tauri test harness)
- [ ] Icône applicative de meilleure qualité (actuellement un cercle bleu simple)
- [ ] Support AppImage / rpm (déjà configuré via `targets: "all"`)
