mod detector;
mod lock_file;
mod orchestrator;
mod pack;
mod platform;

use std::path::PathBuf;

use tauri::Manager;

use lock_file::LockFile;
use orchestrator::AppState;
use pack::loader::PackLoader;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(|app| {
            let packs_dir = resolve_packs_dir();
            let lock_path = resolve_lock_path();

            let manifests = match PackLoader::scan(&packs_dir) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("pack loading failed: {e}");
                    vec![]
                }
            };

            // 7.3 / 3.3: Load persisted installation records; 3.4 crash recovery is implicit
            let lock_file = LockFile::load(&lock_path);
            log::info!("lock file loaded: {} entries from {}", lock_file.packs.len(), lock_path.display());

            app.manage(AppState::new(manifests, packs_dir, lock_file, lock_path));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            orchestrator::load_packs,
            orchestrator::detect_all,
            orchestrator::detect_pack,
            orchestrator::install_pack,
        ])
        .run(tauri::generate_context!())
        .expect("C-Box failed to start");
}

fn resolve_packs_dir() -> PathBuf {
    let candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("packs"))
        .unwrap_or_else(|| PathBuf::from("packs"));

    if candidate.exists() {
        return candidate;
    }

    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("packs")))
        .unwrap_or_else(|| PathBuf::from("packs"))
}

fn resolve_lock_path() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    if root.exists() {
        return root.join("environment.lock.toml");
    }

    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("environment.lock.toml")))
        .unwrap_or_else(|| PathBuf::from("environment.lock.toml"))
}
