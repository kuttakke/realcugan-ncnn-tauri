#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{
  utils::{set_window_shadow}
};
 
mod utils;

use realcugan_ncnn_tauri::realcugan;
use tauri_plugin_log::{fern::colors::ColoredLevelConfig, LogTarget};

#[cfg(debug_assertions)]
const LOG_TARGETS: [LogTarget; 3] = [LogTarget::Stdout, LogTarget::Webview, LogTarget::LogDir];

#[cfg(not(debug_assertions))]
const LOG_TARGETS: [LogTarget; 2] = [LogTarget::Stdout, LogTarget::LogDir];

fn main() {
    let guard = sentry_tauri::sentry::init(("https://6374e62bb0725349ff5caa2fa56ba5e1@o4504503464230912.ingest.sentry.io/4506367005687808", sentry_tauri::sentry::ClientOptions {
      release: sentry_tauri::sentry::release_name!(),
      ..Default::default()
    }));
    let _guard = sentry_tauri::minidump::init(&guard);
    tauri::Builder::default()
        .setup(|app| {
            set_window_shadow(app);
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets(LOG_TARGETS)
                .with_colors(ColoredLevelConfig::default())
                .build(),
        )
        .plugin(sentry_tauri::plugin()) // sentry
        .invoke_handler(tauri::generate_handler![realcugan::run_realcugan])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
