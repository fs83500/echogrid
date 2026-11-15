# EchoGrid — Suivi d'avancement

Dernière mise à jour : **2026-11-15** · Version courante : **1.4.0**

---

## 📊 Résumé

| Indicateur | Valeur |
|------------|--------|
| Version | 1.4.0 |
| Tests | 66 / 66 ✅ |
| Compilation Rust | ✅ `cargo check` OK |
| Build Tauri .deb | ✅ `EchoGrid_1.4.0_amd64.deb` (5.77 MiO) |
| IPC commands | 11 |
| Événements Rust→JS | 4 |
| Fichiers Rust | 6 (main, commands, ping, tcp_monitor, validation, tests) |
| Fichiers Frontend | 3 (index.html, style.css, app.js) |

---

## ✅ Fonctionnalités implémentées

### v1.0.0 — Version initiale
- [x] Ping ICMP multiple continu (surge-ping 0.8)
- [x] Monitoring TCP multiple continu (tokio::net::TcpStream)
- [x] Grille dynamique de tuiles colorées (vert=UP, rouge=DOWN, orange=ERREUR)
- [x] Validation exhaustive côté Rust (host, PingConfig, TcpConfig, AppSettings)
- [x] Sanitisation (clamp, trim) avant validation
- [x] Déduplication des hôtes (arrêt de l'ancien task avant insertion)
- [x] Arrêt gracieux (cancel.send + handle.await)
- [x] 11 commandes IPC exposées
- [x] 4 événements Tauri (ping-result, tcp-result, ping-stopped, tcp-stopped)
- [x] Modale settings (interval, timeout, TTL, packet size, TCP timeout)
- [x] Thème sombre GitHub-like avec glow sur les tuiles
- [x] 63 tests unitaires (validation, sanitisation, sérialisation, state, boundary)
- [x] Icônes (32x32, 128x128, 128x128@2x, .ico, .icns)

### v1.0.1 — PATCH
- [x] Fix : événements Tauri 2 bloqués (capabilities `core:default` manquantes)
- [x] Fichier `src-tauri/capabilities/default.json` créé

### v1.1.0 — MINOR
- [x] Métriques cumulatives : `lost_count`, `total_count`, `last_ok`, `latency_min`, `latency_max`, `jitter_ms`
- [x] Affichage métriques dans chaque tuile (4 spans : perdus, dernier OK, jitter, min/max)
- [x] Classe CSS `has-loss` (rouge) quand `lost_count > 0`
- [x] Classe CSS `high-jitter` (orange) quand `jitter_ms > 50`
- [x] Notifications système KO (plugin `tauri-plugin-notification`)
- [x] Notification uniquement sur transition UP→KO + app pas au premier plan
- [x] Tracking focus fenêtre (`visibilitychange`, `focus`/`blur`)
- [x] Permission notification demandée au démarrage
- [x] Dépendance `chrono = "0.4"` ajoutée
- [x] Dépendance `tauri-plugin-notification = "2"` ajoutée
- [x] Capability `notification:default` ajoutée
- [x] Tests sérialisation mis à jour avec les 6 nouveaux champs

### v1.2.0 — MINOR
- [x] Panneau historique des changements d'état (tableau scrollable, 500 entrées max)
- [x] Colonnes : Heure / Hôte / État / Latence / Détail
- [x] Couleurs d'état : UP=vert, DOWN=rouge, ERR=orange
- [x] Bouton Historique dans la toolbar
- [x] Fix style `<select>` : flèche SVG custom, couleurs forcées pour thème sombre

### v1.3.0 — MINOR
- [x] **Persistance settings** : lecture/écriture via `dirs::config_dir()` + JSON (`~/.config/echogrid/settings.json`)
- [x] **Persistance hôtes** : sauvegarde/restauration via `localStorage` (clé `echogrid-hosts`)
- [x] **Gestion d'erreur frontend** : toasts visuels (rouge=erreur, vert=succès) quand `invoke()` échoue
- [x] **Uptime %** dans la stats bar : calculé à partir des compteurs cumulatifs de toutes les tuiles
- [x] **Confirmation "Stop tout"** : modale de confirmation avant d'arrêter tous les monitorings
- [x] **Filtre historique** : dropdown Tous / UP / DOWN / ERR dans le panneau historique
- [x] **Presets ports TCP** : datalist HTML avec 18 ports courants (FTP, SSH, HTTP, HTTPS, MySQL, PostgreSQL, Redis, RDP, MongoDB…)
- [x] **last_ok raccourci** : affiche uniquement HH:MM:SS au lieu de la date complète
- [x] **Latence colorée par seuil** : configurable (warn=orange, crit=rouge) dans les paramètres
- [x] **Son alerte KO** : bip AudioContext (880→440 Hz) quand hôte passe KO, désactivable dans les paramètres
- [x] **AppSettings enrichi** : +3 champs (`alert_sound`, `latency_warn_ms`, `latency_crit_ms`)
- [x] **3 nouveaux tests** : `settings_default_alert_sound_true`, `settings_default_latency_thresholds`, `settings_serialization_roundtrip`
- [x] **Dépendance `dirs = "5"`** ajoutée pour résoudre le chemin de config

### v1.4.0 — MINOR
- [x] **Icône applicative radar/sonar** noir et blanc (cercle noir, anneaux blancs, sweep, blips) — générée via Pillow
- [x] **Toutes les icônes en SVG noir et blanc** : plus aucun emoji dans l'interface
- [x] **Empty state** : illustration radar SVG + message quand aucun hôte n'est monitoré
- [x] **Fieldsets settings** : ICMP / TCP / Affichage / Alertes groupés visuellement
- [x] **Stats bar avec icônes SVG** : grille (total), triangle up, triangle down, horloge (uptime)
- [x] **Metrics tuile avec icônes SVG** : ✗/✓ (lost), horloge (last_ok), jitter wave, min/max chart
- [x] **Bouton stop tuile** : icône ✕ SVG au lieu de `×` texte
- [x] **Toasts avec icônes** : ✕ rouge (erreur), ✓ vert (succès)
- [x] **Modales avec icônes** : gear (settings), warning (confirm), list (historique)
- [x] **UX améliorée** : espacement, transitions, focus states, hiérarchie visuelle claire

---

## ⬜ Fonctionnalités à faire (backlog)

| # | Fonctionnalité | Priorité | Complexité | Version cible | Statut |
|---|----------------|----------|------------|---------------|--------|
| 1 | Tests d'intégration IPC (Tauri test harness) | Moyenne | Élevée | — | ⬜ |
| 2 | Export CSV/JSON des résultats | Basse | Moyenne | — | ⬜ |
| 3 | Tri/filtrage des tuiles (par statut, latence, nom) | Basse | Faible | — | ⬜ |
| 4 | Thème clair/sombre switchable | Basse | Moyenne | — | ⬜ |

---

## 🔨 Historique des builds

| Version | Date | Commande | Résultat | Artefact |
|---------|------|----------|----------|----------|
| 1.0.1 | 2026-11-14 | `CI=false npx tauri build --bundles deb` | ✅ Succès | `EchoGrid_1.0.1_amd64.deb` |
| 1.1.0 | 2026-11-14 | `CI=false npx tauri build --bundles deb` | ✅ Succès | `EchoGrid_1.1.0_amd64.deb` (5.78 MiO) |
| 1.2.0 | 2026-11-14 | `CI=false npx tauri build --bundles deb` | ✅ Succès | `EchoGrid_1.2.0_amd64.deb` (5.78 MiO) |
| 1.3.0 | 2026-11-15 | `CI=false npx tauri build --bundles deb` | ✅ Succès | `EchoGrid_1.3.0_amd64.deb` (5.78 MiO) |
| 1.4.0 | 2026-11-15 | `CI=false npx tauri build --bundles deb` | ✅ Succès | `EchoGrid_1.4.0_amd64.deb` (5.77 MiO) |

---

## 🧪 Historique des tests

| Version | Date | Commande | Résultat |
|---------|------|----------|----------|
| 1.0.0 | 2026-11-14 | `cargo test` | 63/63 ✅ |
| 1.0.1 | 2026-11-14 | `cargo test` | 63/63 ✅ |
| 1.1.0 | 2026-11-14 | `cargo test` | 63/63 ✅ |
| 1.2.0 | 2026-11-14 | `cargo test` | 63/63 ✅ |
| 1.3.0 | 2026-11-15 | `cargo test` | 66/66 ✅ |
| 1.4.0 | 2026-11-15 | `cargo test` | 66/66 ✅ |

---

## 📝 Journal des modifications (changelog technique)

### 2026-11-15 — v1.4.0
- `src-tauri/icons/*` : régénération complète — icône radar/sonar noir & blanc (Pillow, RGBA)
- `src/index.html` : réécriture complète — toutes les icônes en SVG inline N&B (gear, stop, list, cross, check, up, down, clock, jitter, minmax, lost, ok), empty state avec illustration radar, fieldsets settings (ICMP, TCP, Affichage, Alertes)
- `src/style.css` : réécriture complète — système d'icônes `.icon`/`.icon-sm`, empty state (`#empty-state`, `.empty-icon`, `.empty-title`, `.empty-sub`), fieldsets settings, stats bar avec icônes, transitions focus, scrollbar custom
- `src/app.js` : réécriture complète — dictionnaire `SVG` (12 icônes), stop-btn icône cross, metrics avec icônes (lost/ok, clock, jitter, minmax), toasts avec icônes, `toggleEmptyState()`, stats bar mise à jour sélective
- 3 fichiers version → 1.4.0

### 2026-11-15 — v1.3.0
- `src-tauri/src/commands.rs` : `AppSettings` enrichi (+`alert_sound`, +`latency_warn_ms`, +`latency_crit_ms`), persistance settings via `dirs::config_dir()` + `fs::read/write`, fonctions `settings_path()`, `load_settings_from_file()`, `save_settings_to_file()`
- `src-tauri/Cargo.toml` : ajout `dirs = "5"`, version → 1.3.0
- `src-tauri/src/tests.rs` : +3 tests (alert_sound default, latency thresholds, serialization roundtrip), mise à jour `settings_all_valid` et `settings_boundary_all_at_limits` avec nouveaux champs
- `src/index.html` : ajout `#confirm-modal`, `#toast-container`, `#history-filter`, `#common-ports` datalist (18 ports), `#stats-uptime`, checkbox alert sound + inputs latency warn/crit dans settings
- `src/style.css` : styles `.latency-warn` (orange), `.latency-crit` (rouge), `#toast-container` + `.toast`, `#history-controls`, `#history-filter`, `.stat-uptime`, `.checkbox-label`, `#confirm-modal p`
- `src/app.js` : réécriture complète — persistance hôtes localStorage, toasts d'erreur, modale confirmation stop all, filtre historique, uptime % stats bar, last_ok raccourci HH:MM:SS, latence colorée par seuil, son alerte AudioContext (880→440 Hz), settings enrichis (alert_sound, latency_warn_ms, latency_crit_ms)
- 3 fichiers version → 1.3.0

### 2026-11-14 — v1.2.0
- `src/index.html` : ajout panneau `#history-panel` (tableau 5 colonnes) + bouton Historique
- `src/style.css` : fix `<select>` thème sombre (appearance none, flèche SVG, option colors) + styles `#history-panel`, `#history-table`, `.history-up/down/error`
- `src/app.js` : ajout tableau `history[]` (max 500), fonctions `addHistory()`, `renderHistory()`, log sur changements d'état dans `updateTile()`
- 3 fichiers version → 1.2.0

### 2026-11-14 — v1.1.0
- `src-tauri/src/ping.rs` : +6 champs métriques sur `PingResult`, accumulateurs dans la boucle async
- `src-tauri/src/tcp_monitor.rs` : +6 champs métriques sur `TcpResult`, même logique
- `src-tauri/Cargo.toml` : ajout `chrono = "0.4"`, `tauri-plugin-notification = "2"`, version → 1.1.0
- `src-tauri/tauri.conf.json` : version → 1.1.0
- `package.json` : version → 1.1.0
- `src-tauri/src/main.rs` : ajout `.plugin(tauri_plugin_notification::init())`
- `src-tauri/capabilities/default.json` : ajout `"notification:default"`
- `src-tauri/src/tests.rs` : 4 tests sérialisation mis à jour avec nouveaux champs
- `src/app.js` : réécriture complète (338 lignes) — notifications KO, focus tracking, métriques tuile
- `src/style.css` : styles `.metrics`, `.has-loss`, `.high-jitter`, `.metric-last-ok`

### 2026-11-14 — v1.0.1
- `src-tauri/capabilities/default.json` : **créé** avec `"permissions": ["core:default"]`
- 3 fichiers version → 1.0.1

### 2026-11-14 — v1.0.0
- Version initiale du projet
