//! Tests unitaires et d'intégration pour EchoGrid

use crate::commands::AppSettings;
use crate::ping::{PingConfig, PingResult, PingState};
use crate::tcp_monitor::{TcpConfig, TcpResult, TcpState};
use crate::validation::*;

// ═════════════════════════════════════════════════════════════════════════════
//  1. VALIDATION HOST
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn host_valid_ipv4() {
    assert!(validate_host("192.168.1.1").is_ok());
    assert!(validate_host("10.0.0.1").is_ok());
    assert!(validate_host("255.255.255.255").is_ok());
    assert!(validate_host("0.0.0.0").is_ok());
}

#[test]
fn host_valid_ipv6() {
    assert!(validate_host("::1").is_ok());
    assert!(validate_host("fe80::1").is_ok());
    assert!(validate_host("2001:db8::1").is_ok());
}

#[test]
fn host_valid_dns() {
    assert!(validate_host("localhost").is_ok());
    assert!(validate_host("example.com").is_ok());
    assert!(validate_host("sub.domain.example.com").is_ok());
    assert!(validate_host("host-1.example-domain.com").is_ok());
    assert!(validate_host("a").is_ok());
    assert!(validate_host("1host").is_ok());
}

#[test]
fn host_empty_fails() {
    assert_eq!(validate_host(""), Err(ValidationError::EmptyHost));
}

#[test]
fn host_whitespace_only_fails() {
    assert_eq!(validate_host("   "), Err(ValidationError::HostWhitespace));
}

#[test]
fn host_too_long_fails() {
    let long_host = "a".repeat(254);
    assert!(matches!(validate_host(&long_host), Err(ValidationError::HostTooLong(254))));
}

#[test]
fn host_max_length_ok() {
    // 4 labels of 63 chars = 252 chars + 3 dots = 255 total (> 253)
    // Use 3 labels of 63 + 1 label of 61 = 253 chars, 3 dots = 250 chars
    let label63 = "a".repeat(63);
    let host = format!("{}.{}.{}.aa", label63, label63, label63);
    // 63+1+63+1+63+1+2 = 194 chars — valid
    assert!(validate_host(&host).is_ok());
    assert!(host.len() <= 253);
}

#[test]
fn host_invalid_chars_fails() {
    assert!(matches!(validate_host("host!invalid"), Err(ValidationError::InvalidHostname(_))));
    assert!(matches!(validate_host("host@bad"), Err(ValidationError::InvalidHostname(_))));
    assert!(matches!(validate_host("host space.com"), Err(ValidationError::InvalidHostname(_))));
}

#[test]
fn host_label_starts_with_dash_fails() {
    assert!(matches!(
        validate_host("-host.example.com"),
        Err(ValidationError::InvalidHostname(_))
    ));
}

#[test]
fn host_label_ends_with_dash_fails() {
    assert!(matches!(
        validate_host("host-.example.com"),
        Err(ValidationError::InvalidHostname(_))
    ));
}

#[test]
fn host_empty_label_fails() {
    assert!(matches!(
        validate_host("host..example.com"),
        Err(ValidationError::InvalidHostname(_))
    ));
}

#[test]
fn host_label_too_long_fails() {
    let label = "a".repeat(64);
    let host = format!("{}.com", label);
    assert!(matches!(validate_host(&host), Err(ValidationError::InvalidHostname(_))));
}

#[test]
fn host_label_max_length_ok() {
    let label = "a".repeat(63);
    let host = format!("{}.com", label);
    assert!(validate_host(&host).is_ok());
}

// ═════════════════════════════════════════════════════════════════════════════
//  2. VALIDATION PingConfig
// ═════════════════════════════════════════════════════════════════════════════

fn valid_ping_config() -> PingConfig {
    PingConfig {
        host: "8.8.8.8".into(),
        interval_ms: 2000,
        timeout_ms: 1000,
        ttl: 64,
        packet_size: 32,
    }
}

#[test]
fn ping_config_valid() {
    assert!(validate_ping_config(&valid_ping_config()).is_ok());
}

#[test]
fn ping_config_default_with_host_is_valid() {
    let mut config = PingConfig::default();
    config.host = "localhost".into();
    assert!(validate_ping_config(&config).is_ok());
}

#[test]
fn ping_config_empty_host_fails() {
    let mut config = valid_ping_config();
    config.host = "".into();
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.contains(&ValidationError::EmptyHost));
}

#[test]
fn ping_config_interval_too_low() {
    let mut config = valid_ping_config();
    config.interval_ms = 50;
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::IntervalOutOfRange(50))));
}

#[test]
fn ping_config_interval_too_high() {
    let mut config = valid_ping_config();
    config.interval_ms = 61_000;
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::IntervalOutOfRange(61_000))));
}

#[test]
fn ping_config_timeout_too_low() {
    let mut config = valid_ping_config();
    config.timeout_ms = 10;
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::TimeoutOutOfRange(10))));
}

#[test]
fn ping_config_timeout_too_high() {
    let mut config = valid_ping_config();
    config.timeout_ms = 35_000;
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::TimeoutOutOfRange(35_000))));
}

#[test]
fn ping_config_timeout_greater_than_interval() {
    let mut config = valid_ping_config();
    config.interval_ms = 500;
    config.timeout_ms = 1000;
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::TimeoutGreaterThanInterval)));
}

#[test]
fn ping_config_ttl_zero_fails() {
    let mut config = valid_ping_config();
    config.ttl = 0;
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::TtlOutOfRange(0))));
}

#[test]
fn ping_config_ttl_256_fails() {
    let mut config = valid_ping_config();
    config.ttl = 256;
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::TtlOutOfRange(256))));
}

#[test]
fn ping_config_ttl_boundary_ok() {
    let mut config = valid_ping_config();
    config.ttl = 1;
    assert!(validate_ping_config(&config).is_ok());
    config.ttl = 255;
    assert!(validate_ping_config(&config).is_ok());
}

#[test]
fn ping_config_packet_size_zero_fails() {
    let mut config = valid_ping_config();
    config.packet_size = 0;
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::PacketSizeOutOfRange(0))));
}

#[test]
fn ping_config_packet_size_too_high() {
    let mut config = valid_ping_config();
    config.packet_size = 70_000;
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::PacketSizeOutOfRange(70_000))));
}

#[test]
fn ping_config_multiple_errors() {
    let config = PingConfig {
        host: "".into(),
        interval_ms: 10,
        timeout_ms: 5,
        ttl: 0,
        packet_size: 0,
    };
    let errors = validate_ping_config(&config).unwrap_err();
    assert!(errors.len() >= 4, "Expected at least 4 errors, got {}: {:?}", errors.len(), errors);
}

// ═════════════════════════════════════════════════════════════════════════════
//  3. VALIDATION TcpConfig
// ═════════════════════════════════════════════════════════════════════════════

fn valid_tcp_config() -> TcpConfig {
    TcpConfig {
        host: "example.com".into(),
        port: 443,
        interval_ms: 2000,
        timeout_ms: 3000,
    }
}

#[test]
fn tcp_config_valid() {
    assert!(validate_tcp_config(&valid_tcp_config()).is_ok());
}

#[test]
fn tcp_config_empty_host_fails() {
    let mut config = valid_tcp_config();
    config.host = "".into();
    let errors = validate_tcp_config(&config).unwrap_err();
    assert!(errors.contains(&ValidationError::EmptyHost));
}

#[test]
fn tcp_config_port_zero_fails() {
    let mut config = valid_tcp_config();
    config.port = 0;
    let errors = validate_tcp_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::PortOutOfRange(0))));
}

#[test]
fn tcp_config_port_boundary_ok() {
    let mut config = valid_tcp_config();
    config.port = 1;
    assert!(validate_tcp_config(&config).is_ok());
    config.port = 65535;
    assert!(validate_tcp_config(&config).is_ok());
}

#[test]
fn tcp_config_interval_out_of_range() {
    let mut config = valid_tcp_config();
    config.interval_ms = 10;
    let errors = validate_tcp_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::IntervalOutOfRange(10))));
}

#[test]
fn tcp_config_timeout_out_of_range() {
    let mut config = valid_tcp_config();
    config.timeout_ms = 50_000;
    let errors = validate_tcp_config(&config).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::TimeoutOutOfRange(50_000))));
}

// ═════════════════════════════════════════════════════════════════════════════
//  4. VALIDATION AppSettings
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn settings_default_is_valid() {
    assert!(validate_settings(&AppSettings::default()).is_ok());
}

#[test]
fn settings_timeout_greater_than_interval() {
    let mut s = AppSettings::default();
    s.ping_interval_ms = 500;
    s.ping_timeout_ms = 1000;
    let errors = validate_settings(&s).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::TimeoutGreaterThanInterval)));
}

#[test]
fn settings_ttl_out_of_range() {
    let mut s = AppSettings::default();
    s.ttl = 300;
    let errors = validate_settings(&s).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::TtlOutOfRange(300))));
}

#[test]
fn settings_packet_size_out_of_range() {
    let mut s = AppSettings::default();
    s.packet_size = 100_000;
    let errors = validate_settings(&s).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::PacketSizeOutOfRange(100_000))));
}

#[test]
fn settings_tcp_timeout_out_of_range() {
    let mut s = AppSettings::default();
    s.tcp_timeout_ms = 40_000;
    let errors = validate_settings(&s).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, ValidationError::TimeoutOutOfRange(40_000))));
}

#[test]
fn settings_all_valid() {
    let s = AppSettings {
        ping_interval_ms: 3000,
        ping_timeout_ms: 2000,
        ttl: 128,
        packet_size: 64,
        tcp_timeout_ms: 5000,
        alert_sound: true,
        latency_warn_ms: 50.0,
        latency_crit_ms: 150.0,
    };
    assert!(validate_settings(&s).is_ok());
}

#[test]
fn settings_default_alert_sound_true() {
    let s = AppSettings::default();
    assert!(s.alert_sound);
}

#[test]
fn settings_default_latency_thresholds() {
    let s = AppSettings::default();
    assert_eq!(s.latency_warn_ms, 50.0);
    assert_eq!(s.latency_crit_ms, 150.0);
}

#[test]
fn settings_serialization_roundtrip() {
    let s = AppSettings::default();
    let json = serde_json::to_string(&s).unwrap();
    let decoded: AppSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.ping_interval_ms, s.ping_interval_ms);
    assert_eq!(decoded.alert_sound, s.alert_sound);
    assert_eq!(decoded.latency_warn_ms, s.latency_warn_ms);
    assert_eq!(decoded.latency_crit_ms, s.latency_crit_ms);
}

// ═════════════════════════════════════════════════════════════════════════════
//  5. SANITISATION PingConfig
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn sanitize_ping_clamps_interval() {
    let mut config = PingConfig {
        host: "localhost".into(),
        interval_ms: 10,
        timeout_ms: 1000,
        ttl: 64,
        packet_size: 32,
    };
    sanitize_ping_config(&mut config);
    assert_eq!(config.interval_ms, MIN_INTERVAL_MS);
}

#[test]
fn sanitize_ping_clamps_timeout() {
    let mut config = PingConfig {
        host: "localhost".into(),
        interval_ms: 2000,
        timeout_ms: 5,
        ttl: 64,
        packet_size: 32,
    };
    sanitize_ping_config(&mut config);
    assert_eq!(config.timeout_ms, MIN_TIMEOUT_MS);
}

#[test]
fn sanitize_ping_timeout_capped_to_interval() {
    let mut config = PingConfig {
        host: "localhost".into(),
        interval_ms: 500,
        timeout_ms: 2000,
        ttl: 64,
        packet_size: 32,
    };
    sanitize_ping_config(&mut config);
    assert_eq!(config.timeout_ms, 500);
}

#[test]
fn sanitize_ping_clamps_ttl() {
    let mut config = PingConfig {
        host: "localhost".into(),
        interval_ms: 2000,
        timeout_ms: 1000,
        ttl: 0,
        packet_size: 32,
    };
    sanitize_ping_config(&mut config);
    assert_eq!(config.ttl, MIN_TTL);

    config.ttl = 300;
    sanitize_ping_config(&mut config);
    assert_eq!(config.ttl, MAX_TTL);
}

#[test]
fn sanitize_ping_clamps_packet_size() {
    let mut config = PingConfig {
        host: "localhost".into(),
        interval_ms: 2000,
        timeout_ms: 1000,
        ttl: 64,
        packet_size: 0,
    };
    sanitize_ping_config(&mut config);
    assert_eq!(config.packet_size, MIN_PACKET_SIZE);

    config.packet_size = 100_000;
    sanitize_ping_config(&mut config);
    assert_eq!(config.packet_size, MAX_PACKET_SIZE);
}

#[test]
fn sanitize_ping_trims_host() {
    let mut config = PingConfig {
        host: "  localhost  ".into(),
        interval_ms: 2000,
        timeout_ms: 1000,
        ttl: 64,
        packet_size: 32,
    };
    sanitize_ping_config(&mut config);
    assert_eq!(config.host, "localhost");
}

#[test]
fn sanitize_ping_valid_config_unchanged() {
    let mut config = valid_ping_config();
    let original = config.clone();
    sanitize_ping_config(&mut config);
    assert_eq!(config.host, original.host);
    assert_eq!(config.interval_ms, original.interval_ms);
    assert_eq!(config.timeout_ms, original.timeout_ms);
    assert_eq!(config.ttl, original.ttl);
    assert_eq!(config.packet_size, original.packet_size);
}

// ═════════════════════════════════════════════════════════════════════════════
//  6. SANITISATION TcpConfig
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn sanitize_tcp_clamps_port() {
    let mut config = TcpConfig {
        host: "localhost".into(),
        port: 0,
        interval_ms: 2000,
        timeout_ms: 3000,
    };
    sanitize_tcp_config(&mut config);
    assert_eq!(config.port, MIN_PORT);
}

#[test]
fn sanitize_tcp_clamps_interval() {
    let mut config = TcpConfig {
        host: "localhost".into(),
        port: 80,
        interval_ms: 5,
        timeout_ms: 3000,
    };
    sanitize_tcp_config(&mut config);
    assert_eq!(config.interval_ms, MIN_INTERVAL_MS);
}

#[test]
fn sanitize_tcp_trims_host() {
    let mut config = TcpConfig {
        host: "  example.com  ".into(),
        port: 443,
        interval_ms: 2000,
        timeout_ms: 3000,
    };
    sanitize_tcp_config(&mut config);
    assert_eq!(config.host, "example.com");
}

#[test]
fn sanitize_tcp_valid_config_unchanged() {
    let mut config = valid_tcp_config();
    let original = config.clone();
    sanitize_tcp_config(&mut config);
    assert_eq!(config.host, original.host);
    assert_eq!(config.port, original.port);
    assert_eq!(config.interval_ms, original.interval_ms);
    assert_eq!(config.timeout_ms, original.timeout_ms);
}

// ═════════════════════════════════════════════════════════════════════════════
//  7. TYPES — Sérialisation PingResult / TcpResult
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn ping_result_serialization() {
    let result = PingResult {
        host: "8.8.8.8".into(),
        alive: true,
        latency_ms: Some(12.5),
        error: None,
        lost_count: 0,
        total_count: 10,
        last_ok: Some("2026-11-14 12:00:00".into()),
        latency_min: Some(10.0),
        latency_max: Some(15.0),
        jitter_ms: Some(2.5),
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"alive\":true"));
    assert!(json.contains("\"latency_ms\":12.5"));
    assert!(json.contains("\"error\":null"));
    assert!(json.contains("\"lost_count\":0"));
    assert!(json.contains("\"total_count\":10"));
    assert!(json.contains("\"jitter_ms\":2.5"));

    let decoded: PingResult = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.host, "8.8.8.8");
    assert!(decoded.alive);
    assert_eq!(decoded.latency_ms, Some(12.5));
    assert_eq!(decoded.lost_count, 0);
}

#[test]
fn ping_result_error_serialization() {
    let result = PingResult {
        host: "bad.host".into(),
        alive: false,
        latency_ms: None,
        error: Some("DNS resolution failed".into()),
        lost_count: 3,
        total_count: 5,
        last_ok: Some("2026-11-14 11:55:00".into()),
        latency_min: None,
        latency_max: None,
        jitter_ms: None,
    };
    let json = serde_json::to_string(&result).unwrap();
    let decoded: PingResult = serde_json::from_str(&json).unwrap();
    assert!(!decoded.alive);
    assert_eq!(decoded.error, Some("DNS resolution failed".into()));
    assert_eq!(decoded.lost_count, 3);
}

#[test]
fn tcp_result_serialization() {
    let result = TcpResult {
        host: "example.com".into(),
        port: 443,
        open: true,
        latency_ms: Some(45.3),
        error: None,
        lost_count: 0,
        total_count: 8,
        last_ok: Some("2026-11-14 12:00:00".into()),
        latency_min: Some(40.0),
        latency_max: Some(50.0),
        jitter_ms: Some(5.0),
    };
    let json = serde_json::to_string(&result).unwrap();
    let decoded: TcpResult = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.port, 443);
    assert!(decoded.open);
    assert_eq!(decoded.jitter_ms, Some(5.0));
}

#[test]
fn tcp_result_closed_serialization() {
    let result = TcpResult {
        host: "example.com".into(),
        port: 22,
        open: false,
        latency_ms: None,
        error: Some("Connection refused".into()),
        lost_count: 2,
        total_count: 3,
        last_ok: None,
        latency_min: None,
        latency_max: None,
        jitter_ms: None,
    };
    let json = serde_json::to_string(&result).unwrap();
    let decoded: TcpResult = serde_json::from_str(&json).unwrap();
    assert!(!decoded.open);
    assert_eq!(decoded.error, Some("Connection refused".into()));
    assert_eq!(decoded.lost_count, 2);
}

// ═════════════════════════════════════════════════════════════════════════════
//  8. STATE — PingState / TcpState
// ═════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn ping_state_default_is_empty() {
    let state = PingState::default();
    let active = state.active.lock().await;
    assert!(active.is_empty());
}

#[tokio::test]
async fn tcp_state_default_is_empty() {
    let state = TcpState::default();
    let active = state.active.lock().await;
    assert!(active.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════════
//  9. VALIDATION ERRORS — Display
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn validation_error_display_messages() {
    assert_eq!(ValidationError::EmptyHost.to_string(), "L'hôte ne peut pas être vide");
    assert_eq!(
        ValidationError::HostTooLong(300).to_string(),
        "L'hôte est trop long (300 chars, max 253)"
    );
    assert_eq!(
        ValidationError::IntervalOutOfRange(10).to_string(),
        "Intervalle 10 ms hors limites [100, 60000]"
    );
    assert_eq!(
        ValidationError::TimeoutOutOfRange(5).to_string(),
        "Timeout 5 ms hors limites [50, 30000]"
    );
    assert_eq!(
        ValidationError::TimeoutGreaterThanInterval.to_string(),
        "Le timeout ne peut pas dépasser l'intervalle"
    );
    assert_eq!(
        ValidationError::TtlOutOfRange(0).to_string(),
        "TTL 0 hors limites [1, 255]"
    );
    assert_eq!(
        ValidationError::PortOutOfRange(0).to_string(),
        "Port 0 hors limites [1, 65535]"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  10. BOUNDARY VALUES — limites exactes
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn ping_config_boundary_interval_min() {
    let mut config = valid_ping_config();
    config.interval_ms = MIN_INTERVAL_MS;
    config.timeout_ms = MIN_TIMEOUT_MS; // timeout must be <= interval
    assert!(validate_ping_config(&config).is_ok());
    config.interval_ms = MIN_INTERVAL_MS - 1;
    assert!(validate_ping_config(&config).is_err());
}

#[test]
fn ping_config_boundary_interval_max() {
    let mut config = valid_ping_config();
    config.interval_ms = MAX_INTERVAL_MS;
    assert!(validate_ping_config(&config).is_ok());
    config.interval_ms = MAX_INTERVAL_MS + 1;
    assert!(validate_ping_config(&config).is_err());
}

#[test]
fn ping_config_boundary_timeout_min() {
    let mut config = valid_ping_config();
    config.timeout_ms = MIN_TIMEOUT_MS;
    assert!(validate_ping_config(&config).is_ok());
    config.timeout_ms = MIN_TIMEOUT_MS - 1;
    assert!(validate_ping_config(&config).is_err());
}

#[test]
fn tcp_config_boundary_port_min() {
    let mut config = valid_tcp_config();
    config.port = MIN_PORT;
    assert!(validate_tcp_config(&config).is_ok());
    config.port = 0;
    assert!(validate_tcp_config(&config).is_err());
}

#[test]
fn tcp_config_boundary_port_max() {
    let mut config = valid_tcp_config();
    config.port = MAX_PORT;
    assert!(validate_tcp_config(&config).is_ok());
}

#[test]
fn settings_boundary_all_at_limits() {
    let s = AppSettings {
        ping_interval_ms: MIN_INTERVAL_MS,
        ping_timeout_ms: MIN_TIMEOUT_MS,
        ttl: MIN_TTL,
        packet_size: MIN_PACKET_SIZE,
        tcp_timeout_ms: MIN_TIMEOUT_MS,
        alert_sound: false,
        latency_warn_ms: 100.0,
        latency_crit_ms: 200.0,
    };
    // timeout <= interval (50 <= 100) → OK
    assert!(validate_settings(&s).is_ok());
}

