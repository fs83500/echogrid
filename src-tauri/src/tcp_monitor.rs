use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter};

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpResult {
    pub host: String,
    pub port: u16,
    pub open: bool,
    pub latency_ms: Option<f64>,
    pub error: Option<String>,
    pub lost_count: u32,
    pub total_count: u32,
    pub last_ok: Option<String>,
    pub latency_min: Option<f64>,
    pub latency_max: Option<f64>,
    pub jitter_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    pub host: String,
    pub port: u16,
    pub interval_ms: u64,
    pub timeout_ms: u64,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 80,
            interval_ms: 2000,
            timeout_ms: 3000,
        }
    }
}

/// Entrée d'état : handle du task + canal d'annulation
pub struct TcpEntry {
    pub handle: tokio::task::JoinHandle<()>,
    pub cancel: tokio::sync::watch::Sender<bool>,
}

/// État global : map "host:port" → TcpEntry
pub struct TcpState {
    pub active: Arc<Mutex<HashMap<String, TcpEntry>>>,
}

impl Default for TcpState {
    fn default() -> Self {
        Self {
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// ── TCP monitor loop ─────────────────────────────────────────────────────────

pub async fn run_tcp_monitor(
    app: AppHandle,
    config: TcpConfig,
    mut cancel: tokio::sync::watch::Receiver<bool>,
) {
    let key = format!("{}:{}", config.host, config.port);
    let interval = tokio::time::Duration::from_millis(config.interval_ms);
    let timeout = tokio::time::Duration::from_millis(config.timeout_ms);

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

        let start = tokio::time::Instant::now();
        let addr = format!("{}:{}", config.host, config.port);

        let result = match tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await {
            Ok(Ok(_stream)) => {
                let lat = start.elapsed().as_secs_f64() * 1000.0;
                total_count += 1;
                last_ok = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                latency_min = Some(latency_min.map_or(lat, |m| m.min(lat)));
                latency_max = Some(latency_max.map_or(lat, |m| m.max(lat)));
                if let Some(pl) = prev_latency {
                    jitter_sum += (lat - pl).abs();
                    jitter_count += 1;
                }
                prev_latency = Some(lat);
                TcpResult {
                    host: config.host.clone(),
                    port: config.port,
                    open: true,
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
                TcpResult {
                    host: config.host.clone(),
                    port: config.port,
                    open: false,
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
                TcpResult {
                    host: config.host.clone(),
                    port: config.port,
                    open: false,
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

        let _ = app.emit("tcp-result", result);

        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = cancel.changed() => {
                if *cancel.borrow() {
                    break;
                }
            }
        }
    }

    let _ = app.emit("tcp-stopped", &key);
}
