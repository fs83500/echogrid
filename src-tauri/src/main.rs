#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ping;
mod tcp_monitor;
mod commands;
mod validation;

#[cfg(test)]
mod tests;

use commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .manage(ping::PingState::default())
        .manage(tcp_monitor::TcpState::default())
        .invoke_handler(tauri::generate_handler![
            start_ping,
            stop_ping,
            stop_all_pings,
            start_tcp_monitor,
            stop_tcp_monitor,
            stop_all_tcp_monitors,
            get_settings,
            save_settings,
            validate_ping,
            validate_tcp,
            validate_settings_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running EchoGrid");
}
