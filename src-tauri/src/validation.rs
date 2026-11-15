//! Validation des configurations — fonctions pures testables sans Tauri

use crate::commands::AppSettings;
use crate::ping::PingConfig;
use crate::tcp_monitor::TcpConfig;

// ── Constantes ───────────────────────────────────────────────────────────────

pub const MIN_INTERVAL_MS: u64 = 100;
pub const MAX_INTERVAL_MS: u64 = 60_000;
pub const MIN_TIMEOUT_MS: u64 = 50;
pub const MAX_TIMEOUT_MS: u64 = 30_000;
pub const MIN_TTL: u32 = 1;
pub const MAX_TTL: u32 = 255;
pub const MIN_PACKET_SIZE: u32 = 1;
pub const MAX_PACKET_SIZE: u32 = 65_527;
pub const MIN_PORT: u16 = 1;
pub const MAX_PORT: u16 = 65_535;

// ── Erreurs ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    EmptyHost,
    HostWhitespace,
    HostTooLong(usize),
    InvalidHostname(String),
    IntervalOutOfRange(u64),
    TimeoutOutOfRange(u64),
    TimeoutGreaterThanInterval,
    TtlOutOfRange(u32),
    PacketSizeOutOfRange(u32),
    PortOutOfRange(u16),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyHost => write!(f, "L'hôte ne peut pas être vide"),
            Self::HostWhitespace => write!(f, "L'hôte ne peut pas contenir uniquement des espaces"),
            Self::HostTooLong(len) => write!(f, "L'hôte est trop long ({} chars, max 253)", len),
            Self::InvalidHostname(h) => write!(f, "Hostname invalide : {}", h),
            Self::IntervalOutOfRange(v) => write!(
                f,
                "Intervalle {} ms hors limites [{}, {}]",
                v, MIN_INTERVAL_MS, MAX_INTERVAL_MS
            ),
            Self::TimeoutOutOfRange(v) => write!(
                f,
                "Timeout {} ms hors limites [{}, {}]",
                v, MIN_TIMEOUT_MS, MAX_TIMEOUT_MS
            ),
            Self::TimeoutGreaterThanInterval => write!(f, "Le timeout ne peut pas dépasser l'intervalle"),
            Self::TtlOutOfRange(v) => write!(f, "TTL {} hors limites [{}, {}]", v, MIN_TTL, MAX_TTL),
            Self::PacketSizeOutOfRange(v) => write!(
                f,
                "Taille paquet {} hors limites [{}, {}]",
                v, MIN_PACKET_SIZE, MAX_PACKET_SIZE
            ),
            Self::PortOutOfRange(v) => write!(f, "Port {} hors limites [{}, {}]", v, MIN_PORT, MAX_PORT),
        }
    }
}

// ── Validation host ──────────────────────────────────────────────────────────

/// Valide un hostname ou adresse IP.
/// Accepte : IPv4, IPv6, noms DNS (lettres, chiffres, tirets, points)
pub fn validate_host(host: &str) -> Result<(), ValidationError> {
    // Check whitespace-only before trimming
    if !host.is_empty() && host.trim().is_empty() {
        return Err(ValidationError::HostWhitespace);
    }

    let trimmed = host.trim();

    if trimmed.is_empty() {
        return Err(ValidationError::EmptyHost);
    }

    if trimmed.len() > 253 {
        return Err(ValidationError::HostTooLong(trimmed.len()));
    }

    // Tenter de parser en IP
    if trimmed.parse::<std::net::IpAddr>().is_ok() {
        return Ok(());
    }

    // Valider comme hostname DNS
    validate_dns_hostname(trimmed)
}

fn validate_dns_hostname(name: &str) -> Result<(), ValidationError> {
    // Chaque label (séparé par '.') : 1-63 chars, alphanum + tiret, pas de tiret en début/fin
    for label in name.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(ValidationError::InvalidHostname(name.to_string()));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(ValidationError::InvalidHostname(name.to_string()));
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ValidationError::InvalidHostname(name.to_string()));
        }
    }
    Ok(())
}

// ── Validation PingConfig ────────────────────────────────────────────────────

pub fn validate_ping_config(config: &PingConfig) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if let Err(e) = validate_host(&config.host) {
        errors.push(e);
    }
    if config.interval_ms < MIN_INTERVAL_MS || config.interval_ms > MAX_INTERVAL_MS {
        errors.push(ValidationError::IntervalOutOfRange(config.interval_ms));
    }
    if config.timeout_ms < MIN_TIMEOUT_MS || config.timeout_ms > MAX_TIMEOUT_MS {
        errors.push(ValidationError::TimeoutOutOfRange(config.timeout_ms));
    }
    if config.timeout_ms > config.interval_ms {
        errors.push(ValidationError::TimeoutGreaterThanInterval);
    }
    if config.ttl < MIN_TTL || config.ttl > MAX_TTL {
        errors.push(ValidationError::TtlOutOfRange(config.ttl));
    }
    if config.packet_size < MIN_PACKET_SIZE || config.packet_size > MAX_PACKET_SIZE {
        errors.push(ValidationError::PacketSizeOutOfRange(config.packet_size));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// ── Validation TcpConfig ─────────────────────────────────────────────────────

pub fn validate_tcp_config(config: &TcpConfig) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if let Err(e) = validate_host(&config.host) {
        errors.push(e);
    }
    if config.port < MIN_PORT || config.port > MAX_PORT {
        errors.push(ValidationError::PortOutOfRange(config.port));
    }
    if config.interval_ms < MIN_INTERVAL_MS || config.interval_ms > MAX_INTERVAL_MS {
        errors.push(ValidationError::IntervalOutOfRange(config.interval_ms));
    }
    if config.timeout_ms < MIN_TIMEOUT_MS || config.timeout_ms > MAX_TIMEOUT_MS {
        errors.push(ValidationError::TimeoutOutOfRange(config.timeout_ms));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// ── Validation AppSettings ───────────────────────────────────────────────────

pub fn validate_settings(settings: &AppSettings) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if settings.ping_interval_ms < MIN_INTERVAL_MS || settings.ping_interval_ms > MAX_INTERVAL_MS {
        errors.push(ValidationError::IntervalOutOfRange(settings.ping_interval_ms));
    }
    if settings.ping_timeout_ms < MIN_TIMEOUT_MS || settings.ping_timeout_ms > MAX_TIMEOUT_MS {
        errors.push(ValidationError::TimeoutOutOfRange(settings.ping_timeout_ms));
    }
    if settings.ping_timeout_ms > settings.ping_interval_ms {
        errors.push(ValidationError::TimeoutGreaterThanInterval);
    }
    if settings.ttl < MIN_TTL || settings.ttl > MAX_TTL {
        errors.push(ValidationError::TtlOutOfRange(settings.ttl));
    }
    if settings.packet_size < MIN_PACKET_SIZE || settings.packet_size > MAX_PACKET_SIZE {
        errors.push(ValidationError::PacketSizeOutOfRange(settings.packet_size));
    }
    if settings.tcp_timeout_ms < MIN_TIMEOUT_MS || settings.tcp_timeout_ms > MAX_TIMEOUT_MS {
        errors.push(ValidationError::TimeoutOutOfRange(settings.tcp_timeout_ms));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// ── Sanitisation ─────────────────────────────────────────────────────────────

/// Clamp les valeurs d'un PingConfig dans les limites valides
pub fn sanitize_ping_config(config: &mut PingConfig) {
    config.host = config.host.trim().to_string();
    config.interval_ms = config.interval_ms.clamp(MIN_INTERVAL_MS, MAX_INTERVAL_MS);
    config.timeout_ms = config.timeout_ms.clamp(MIN_TIMEOUT_MS, MAX_TIMEOUT_MS);
    if config.timeout_ms > config.interval_ms {
        config.timeout_ms = config.interval_ms;
    }
    config.ttl = config.ttl.clamp(MIN_TTL, MAX_TTL);
    config.packet_size = config.packet_size.clamp(MIN_PACKET_SIZE, MAX_PACKET_SIZE);
}

/// Clamp les valeurs d'un TcpConfig dans les limites valides
pub fn sanitize_tcp_config(config: &mut TcpConfig) {
    config.host = config.host.trim().to_string();
    config.port = config.port.clamp(MIN_PORT, MAX_PORT);
    config.interval_ms = config.interval_ms.clamp(MIN_INTERVAL_MS, MAX_INTERVAL_MS);
    config.timeout_ms = config.timeout_ms.clamp(MIN_TIMEOUT_MS, MAX_TIMEOUT_MS);
}
