// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod database;
mod error;
mod hotkey;
mod indexer;
mod search;
mod tray;
mod windows;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // Initialize structured logging per Section 33
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,spotlight_for_windows=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = core::app::create_app();

    app.run(tauri::generate_context!())
        .expect("error while running Spotlight application");
}
