use crate::ping::{self, PingConfig, PingEntry, PingState};
use crate::tcp_monitor::{self, TcpConfig, TcpEntry, TcpState};
use crate::validation::{
    sanitize_ping_config, sanitize_tcp_config, validate_ping_config, validate_tcp_config,
    validate_settings as validate_app_settings, ValidationError,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, State};

// ── Settings ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub ping_interval_ms: u64,
    pub ping_timeout_ms: u64,
    pub ttl: u32,
    pub packet_size: u32,
    pub tcp_timeout_ms: u64,
    pub alert_sound: bool,
    pub latency_warn_ms: f64,
    pub latency_crit_ms: f64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ping_interval_ms: 2000,
            ping_timeout_ms: 1000,
            ttl: 64,
            packet_size: 32,
            tcp_timeout_ms: 3000,
            alert_sound: true,
            latency_warn_ms: 50.0,
            latency_crit_ms: 150.0,
        }
    }
}

// ── Settings file persistence ────────────────────────────────────────────────

fn settings_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("echogrid").join("settings.json")
}

fn load_settings_from_file() -> Option<AppSettings> {
    let path = settings_path();
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_settings_to_file(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create config dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(settings).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Cannot write settings: {}", e))
}

// ── Validation result for frontend ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub valid: bool,
    pub errors: Vec<String>,
}

impl From<Result<(), Vec<ValidationError>>> for ValidationResponse {
    fn from(result: Result<(), Vec<ValidationError>>) -> Self {
        match result {
            Ok(()) => ValidationResponse {
                valid: true,
                errors: vec![],
            },
            Err(errors) => ValidationResponse {
                valid: false,
                errors: errors.iter().map(|e| e.to_string()).collect(),
            },
        }
    }
}

// ── ICMP Ping commands ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_ping(
    app: AppHandle,
    state: State<'_, PingState>,
    mut config: PingConfig,
) -> Result<(), String> {
    sanitize_ping_config(&mut config);
    let validation = validate_ping_config(&config);
    if let Err(errors) = validation {
        return Err(errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", "));
    }

    let key = config.host.clone();

    // Déduplication : arrêter l'ancienne entrée gracieusement
    {
        let mut active = state.active.lock().await;
        if let Some(old) = active.remove(&key) {
            let _ = old.cancel.send(true);
            let _ = old.handle.await;
        }
    }

    let (tx, rx) = tokio::sync::watch::channel(false);

    let app_clone = app.clone();
    let config_clone = config.clone();

    let handle = tokio::spawn(async move {
        ping::run_ping(app_clone, config_clone, rx).await;
    });

    let entry = PingEntry { handle, cancel: tx };

    let mut active = state.active.lock().await;
    active.insert(key, entry);

    Ok(())
}

#[tauri::command]
pub async fn stop_ping(
    state: State<'_, PingState>,
    host: String,
) -> Result<(), String> {
    let entry = {
        let mut active = state.active.lock().await;
        active.remove(&host)
    };
    if let Some(e) = entry {
        let _ = e.cancel.send(true);
        let _ = e.handle.await;
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_all_pings(
    state: State<'_, PingState>,
) -> Result<(), String> {
    let entries: Vec<PingEntry> = {
        let mut active = state.active.lock().await;
        active.drain().map(|(_, v)| v).collect()
    };
    for e in entries {
        let _ = e.cancel.send(true);
        let _ = e.handle.await;
    }
    Ok(())
}

// ── TCP Monitor commands ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_tcp_monitor(
    app: AppHandle,
    state: State<'_, TcpState>,
    mut config: TcpConfig,
) -> Result<(), String> {
    sanitize_tcp_config(&mut config);
    let validation = validate_tcp_config(&config);
    if let Err(errors) = validation {
        return Err(errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", "));
    }

    let key = format!("{}:{}", config.host, config.port);

    // Déduplication gracieuse
    {
        let mut active = state.active.lock().await;
        if let Some(old) = active.remove(&key) {
            let _ = old.cancel.send(true);
            let _ = old.handle.await;
        }
    }

    let (tx, rx) = tokio::sync::watch::channel(false);

    let app_clone = app.clone();
    let config_clone = config.clone();

    let handle = tokio::spawn(async move {
        tcp_monitor::run_tcp_monitor(app_clone, config_clone, rx).await;
    });

    let entry = TcpEntry { handle, cancel: tx };

    let mut active = state.active.lock().await;
    active.insert(key, entry);

    Ok(())
}

#[tauri::command]
pub async fn stop_tcp_monitor(
    state: State<'_, TcpState>,
    host: String,
    port: u16,
) -> Result<(), String> {
    let key = format!("{}:{}", host, port);
    let entry = {
        let mut active = state.active.lock().await;
        active.remove(&key)
    };
    if let Some(e) = entry {
        let _ = e.cancel.send(true);
        let _ = e.handle.await;
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_all_tcp_monitors(
    state: State<'_, TcpState>,
) -> Result<(), String> {
    let entries: Vec<TcpEntry> = {
        let mut active = state.active.lock().await;
        active.drain().map(|(_, v)| v).collect()
    };
    for e in entries {
        let _ = e.cancel.send(true);
        let _ = e.handle.await;
    }
    Ok(())
}

// ── Settings commands ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    Ok(load_settings_from_file().unwrap_or_default())
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    let validation = validate_app_settings(&settings);
    if let Err(errors) = validation {
        return Err(errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", "));
    }
    save_settings_to_file(&settings)
}

// ── Validation commands (pour le frontend) ───────────────────────────────────

#[tauri::command]
pub async fn validate_ping(config: PingConfig) -> ValidationResponse {
    ValidationResponse::from(validate_ping_config(&config))
}

#[tauri::command]
pub async fn validate_tcp(config: TcpConfig) -> ValidationResponse {
    ValidationResponse::from(validate_tcp_config(&config))
}

#[tauri::command]
pub async fn validate_settings_cmd(settings: AppSettings) -> ValidationResponse {
    ValidationResponse::from(validate_app_settings(&settings))
}
