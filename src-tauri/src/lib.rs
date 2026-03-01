mod detector;
mod orchestrator;
mod pack;
mod platform;

use std::path::PathBuf;

use tauri::Manager;

use orchestrator::AppState;
use pack::loader::PackLoader;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            let packs_dir = resolve_packs_dir();
            let manifests = match PackLoader::scan(&packs_dir) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("pack loading failed: {e}");
                    vec![]
                }
            };
            app.manage(AppState::new(manifests));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            orchestrator::load_packs,
            orchestrator::detect_all,
            orchestrator::detect_pack,
        ])
        .run(tauri::generate_context!())
        .expect("C-Box failed to start");
}

fn resolve_packs_dir() -> PathBuf {
    // Development: CARGO_MANIFEST_DIR = src-tauri/, parent = project root
    let candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("packs"))
        .unwrap_or_else(|| PathBuf::from("packs"));

    if candidate.exists() {
        return candidate;
    }

    // Production: relative to executable
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("packs")))
        .unwrap_or_else(|| PathBuf::from("packs"))
}
