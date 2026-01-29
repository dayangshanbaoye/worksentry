#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod services;
mod utils;

pub use commands::{SearchResult, Config, HotkeyConfig};
pub use services::tantivy_engine;
