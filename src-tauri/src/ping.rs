use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter};

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub host: String,
    pub alive: bool,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
    pub lost_count: u32,
    pub total_count: u32,
    pub last_ok: Option<String>,      // ISO 8601 datetime
    pub latency_min: Option<f64>,
    pub latency_max: Option<f64>,
    pub jitter_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingConfig {
    pub host: String,
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub ttl: u32,
    pub packet_size: u32,
}

impl Default for PingConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            interval_ms: 2000,
            timeout_ms: 1000,
            ttl: 64,
            packet_size: 32,
        }
    }
}

/// Entrée d'état pour un hôte monitoré : handle du task + canal d'annulation
pub struct PingEntry {
    pub handle: tokio::task::JoinHandle<()>,
    pub cancel: tokio::sync::watch::Sender<bool>,
}

/// État global : map host → PingEntry (pour stop gracieux)
pub struct PingState {
    pub active: Arc<Mutex<HashMap<String, PingEntry>>>,
}

impl Default for PingState {
    fn default() -> Self {
        Self {
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// ── Ping loop ────────────────────────────────────────────────────────────────

/// Lance un ping continu pour un hôte et émet les résultats via Tauri events.
/// Utilise `surge-ping` pour un ping ICMP cross-platform (raw socket Linux,
/// IcmpSendEcho Win32).
pub async fn run_ping(
    app: AppHandle,
    config: PingConfig,
    mut cancel: tokio::sync::watch::Receiver<bool>,
) {
    let host = config.host.clone();
    let interval = tokio::time::Duration::from_millis(config.interval_ms);
    let timeout = tokio::time::Duration::from_millis(config.timeout_ms);

    // Résolution DNS → IpAddr
    let addr: IpAddr = match tokio::net::lookup_host(format!("{}:0", &host)).await {
        Ok(mut addrs) => match addrs.next() {
            Some(a) => a.ip(),
            None => {
                let _ = app.emit(
                    "ping-result",
                    PingResult {
                        host: host.clone(),
                        alive: false,
                        latency_ms: None,
                        error: Some("DNS: no address found".into()),
                        lost_count: 1,
                        total_count: 1,
                        last_ok: None,
                        latency_min: None,
                        latency_max: None,
                        jitter_ms: None,
                    },
                );
                return;
            }
        },
        Err(e) => {
            let _ = app.emit(
                "ping-result",
                PingResult {
                    host: host.clone(),
                    alive: false,
                    latency_ms: None,
                    error: Some(format!("DNS resolution failed: {}", e)),
                    lost_count: 1,
                    total_count: 1,
                    last_ok: None,
                    latency_min: None,
                    latency_max: None,
                    jitter_ms: None,
                },
            );
            return;
        }
    };

    // Client surge-ping (socket partagé pour un hôte)
    // TTL appliqué au niveau socket via le champ Config.ttl
    let surge_config = surge_ping::Config {
        ttl: Some(config.ttl),
        ..surge_ping::Config::default()
    };
    let client = match surge_ping::Client::new(&surge_config) {
        Ok(c) => c,
        Err(e) => {
            let _ = app.emit(
                "ping-result",
                PingResult {
                    host: host.clone(),
                    alive: false,
                    latency_ms: None,
                    error: Some(format!("Ping client init failed: {}", e)),
                    lost_count: 1,
                    total_count: 1,
                    last_ok: None,
                    latency_min: None,
                    latency_max: None,
                    jitter_ms: None,
                },
            );
            return;
        }
    };
    // client.pinger est async et prend un PingIdentifier (u16 dans les tests)
    let mut pinger = client.pinger(addr, surge_ping::PingIdentifier(0)).await;
    pinger.timeout(timeout);

    // Payload ICMP echo data de la taille demandée
    let payload = vec![0u8; config.packet_size as usize];
    let mut seq: u16 = 0;

    // ── Métriques cumulatives ──
    let mut lost_count: u32 = 0;
    let mut total_count: u32 = 0;
    let mut last_ok: Option<String> = None;
    let mut latency_min: Option<f64> = None;
    let mut latency_max: Option<f64> = None;
    let mut prev_latency: Option<f64> = None;
    let mut jitter_sum: f64 = 0.0;
    let mut jitter_count: u32 = 0;

    loop {
        if *cancel.borrow() {
            break;
        }

        let result = match tokio::time::timeout(
            timeout,
            pinger.ping(surge_ping::PingSequence(seq), &payload),
        )
        .await
        {
            Ok(Ok((_packet, duration))) => {
                let lat = duration.as_secs_f64() * 1000.0;
                total_count += 1;
                last_ok = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                latency_min = Some(latency_min.map_or(lat, |m| m.min(lat)));
                latency_max = Some(latency_max.map_or(lat, |m| m.max(lat)));
                if let Some(pl) = prev_latency {
                    jitter_sum += (lat - pl).abs();
                    jitter_count += 1;
                }
                prev_latency = Some(lat);
                PingResult {
                    host: host.clone(),
                    alive: true,
                    latency_ms: Some(lat),
                    error: None,
                    lost_count,
                    total_count,
                    last_ok: last_ok.clone(),
                    latency_min,
                    latency_max,
                    jitter_ms: if jitter_count > 0 { Some(jitter_sum / jitter_count as f64) } else { None },
                }
            }
            Ok(Err(e)) => {
                total_count += 1;
                lost_count += 1;
                PingResult {
                    host: host.clone(),
                    alive: false,
                    latency_ms: None,
                    error: Some(format!("{}", e)),
                    lost_count,
                    total_count,
                    last_ok: last_ok.clone(),
                    latency_min,
                    latency_max,
                    jitter_ms: if jitter_count > 0 { Some(jitter_sum / jitter_count as f64) } else { None },
                }
            }
            Err(_) => {
                // Timeout → DOWN (KO) direct, pas une erreur
                total_count += 1;
                lost_count += 1;
                PingResult {
                    host: host.clone(),
                    alive: false,
                    latency_ms: None,
                    error: None,
                    lost_count,
                    total_count,
                    last_ok: last_ok.clone(),
                    latency_min,
                    latency_max,
                    jitter_ms: if jitter_count > 0 { Some(jitter_sum / jitter_count as f64) } else { None },
                }
            }
        };

        let _ = app.emit("ping-result", result);

        seq = seq.wrapping_add(1);

        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = cancel.changed() => {
                if *cancel.borrow() {
                    break;
                }
            }
        }
    }

    let _ = app.emit("ping-stopped", &host);
}
