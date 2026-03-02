use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::detector::EnvironmentDetector;
use crate::lock_file::LockFile;
use crate::pack::installer::{InstallOutcome, PackInstaller, refresh_path};
use crate::pack::state::StateStore;
use crate::pack::types::{Manifest, PackId, PackState};

pub struct AppState {
    pub manifests: Vec<Manifest>,
    pub states: StateStore,
    pub packs_dir: PathBuf,
    pub lock: Mutex<LockFile>,
    pub lock_path: PathBuf,
}

impl AppState {
    pub fn new(manifests: Vec<Manifest>, packs_dir: PathBuf, lock_file: LockFile, lock_path: PathBuf) -> Self {
        // 7.3 / 3.3: Initialize states from lock file entries
        let initial = lock_file.to_initial_states();
        let ids = manifests.iter().map(|m| m.pack_id.clone());
        Self {
            states: StateStore::new_with_states(ids, initial),
            manifests,
            packs_dir,
            lock: Mutex::new(lock_file),
            lock_path,
        }
    }

    /// 7.2: Flush an Installed state to the lock file.
    fn persist_installed(&self, pack_id: &PackId, version: &str, method: &str) {
        let mut lf = self.lock.lock().unwrap();
        lf.record(pack_id, version, method, None);
        if let Err(e) = lf.save(&self.lock_path) {
            log::warn!("lock file save failed: {e}");
        }
    }

    /// Remove a pack from the lock file (detected as no longer installed).
    fn remove_from_lock(&self, pack_id: &PackId) {
        let mut lf = self.lock.lock().unwrap();
        lf.remove(pack_id);
        if let Err(e) = lf.save(&self.lock_path) {
            log::warn!("lock file save failed: {e}");
        }
    }
}

#[derive(Serialize, Clone)]
struct StateChangedPayload {
    pack_id: PackId,
    state: PackState,
}

fn emit_state(app: &AppHandle, pack_id: &PackId, state: &PackState) {
    let _ = app.emit(
        "pack:state-changed",
        StateChangedPayload { pack_id: pack_id.clone(), state: state.clone() },
    );
}

#[derive(Serialize)]
pub struct PackSummary {
    pack_id: PackId,
    name: String,
    description: String,
    category: String,
    state: PackState,
    installed_version: Option<String>,
}

impl PackSummary {
    fn from(m: &Manifest, state: PackState) -> Self {
        let installed_version = if let PackState::Installed { ref version, .. } = state {
            Some(version.clone())
        } else {
            None
        };
        Self {
            pack_id: m.pack_id.clone(),
            name: m.name.clone(),
            description: m.description.clone(),
            category: m.category.clone(),
            state,
            installed_version,
        }
    }
}

#[tauri::command]
pub fn load_packs(state: State<AppState>) -> Vec<PackSummary> {
    let snap = state.states.snapshot();
    state
        .manifests
        .iter()
        .map(|m| {
            let s = snap.get(&m.pack_id).cloned().unwrap_or(PackState::Undetected);
            PackSummary::from(m, s)
        })
        .collect()
}

#[tauri::command]
pub async fn detect_all(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    for manifest in &state.manifests {
        let id = &manifest.pack_id;
        state.states.set(id.clone(), PackState::Detecting);
        emit_state(&app, id, &PackState::Detecting);

        let result = EnvironmentDetector::detect_pack(&app, manifest).await;
        // 7.2: Sync lock file with detection result
        sync_lock_after_detect(&state, &result.pack_id, &result.state);
        state.states.set(result.pack_id.clone(), result.state.clone());
        emit_state(&app, &result.pack_id, &result.state);
    }
    Ok(())
}

#[tauri::command]
pub async fn detect_pack(
    app: AppHandle,
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<(), String> {
    let manifest = state
        .manifests
        .iter()
        .find(|m| m.pack_id == pack_id)
        .ok_or_else(|| format!("pack not found: {pack_id}"))?
        .clone();

    state.states.set(pack_id.clone(), PackState::Detecting);
    emit_state(&app, &pack_id, &PackState::Detecting);

    let result = EnvironmentDetector::detect_pack(&app, &manifest).await;
    // 7.2: Sync lock file
    sync_lock_after_detect(&state, &result.pack_id, &result.state);
    state.states.set(result.pack_id.clone(), result.state.clone());
    emit_state(&app, &result.pack_id, &result.state);

    Ok(())
}

/// 5.1–5.11 + 6.5: Install a pack and all its uninstalled dependencies.
#[tauri::command]
pub async fn install_pack(
    app: AppHandle,
    state: State<'_, AppState>,
    pack_id: String,
) -> Result<(), String> {
    // 5.5: Reject unknown pack_ids
    let _ = state
        .manifests
        .iter()
        .find(|m| m.pack_id == pack_id)
        .ok_or_else(|| {
            log::warn!("install rejected: unknown pack_id '{pack_id}'");
            format!("unknown pack: {pack_id}")
        })?;

    // 6.5: Collect install order (deps first, then target)
    let order = build_install_order(&state.manifests, &pack_id);

    for manifest in order {
        let id = manifest.pack_id.clone();
        // Skip already installed
        if matches!(state.states.get(&id), PackState::Installed { .. }) {
            continue;
        }
        install_single(&app, &state, &manifest).await;
    }

    Ok(())
}

// --- helpers ---

/// 7.2: Update lock file based on detection outcome.
fn sync_lock_after_detect(state: &AppState, pack_id: &PackId, pack_state: &PackState) {
    match pack_state {
        PackState::Installed { version, .. } => {
            state.persist_installed(pack_id, version, "detected");
        }
        PackState::NotInstalled => {
            state.remove_from_lock(pack_id);
        }
        _ => {}
    }
}

/// Install a single pack, emit state transitions, update lock on success.
async fn install_single(app: &AppHandle, state: &AppState, manifest: &Manifest) {
    let id = &manifest.pack_id;

    state.states.set(id.clone(), PackState::Installing);
    emit_state(app, id, &PackState::Installing);

    let outcome = PackInstaller::install(app, manifest, &state.packs_dir).await;

    match outcome {
        InstallOutcome::Success { pending_reboot } => {
            // 5.8: PATH refresh
            refresh_path(app).await;
            // 5.9: Re-detect to confirm
            let result = EnvironmentDetector::detect_pack(app, manifest).await;

            let final_state = match result.state {
                PackState::Installed { version, .. } => {
                    let method = install_method_str(manifest);
                    state.persist_installed(id, &version, &method); // 7.2
                    PackState::Installed { version, pending_reboot }
                }
                _ => PackState::InstallFailed {
                    reason: "post-install verification failed: tool not found in PATH".to_string(),
                },
            };

            state.states.set(id.clone(), final_state.clone());
            emit_state(app, id, &final_state);
        }
        InstallOutcome::Failed { reason } => {
            let failed = PackState::InstallFailed { reason };
            state.states.set(id.clone(), failed.clone());
            emit_state(app, id, &failed);
        }
    }
}

/// 6.5: DFS traversal to collect install order (deps before dependents).
pub fn build_install_order(manifests: &[Manifest], target_id: &str) -> Vec<Manifest> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    collect_deps(manifests, target_id, &mut result, &mut visited);
    result
}

fn collect_deps(
    manifests: &[Manifest],
    pack_id: &str,
    result: &mut Vec<Manifest>,
    visited: &mut HashSet<String>,
) {
    if !visited.insert(pack_id.to_string()) {
        return;
    }
    let Some(m) = manifests.iter().find(|m| m.pack_id == pack_id) else {
        return;
    };
    for dep_id in &m.dependencies {
        collect_deps(manifests, dep_id, result, visited);
    }
    result.push(m.clone());
}

fn install_method_str(manifest: &Manifest) -> String {
    manifest
        .install
        .as_ref()
        .and_then(|ic| ic.windows.as_ref())
        .map(|p| format!("{:?}", p.method).to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

// --- 9.3: unit tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::types::{DetectConfig, InstallConfig, InstallMethod, PlatformInstall};

    fn manifest(id: &str, deps: &[&str]) -> Manifest {
        Manifest {
            schema_version: "1.0".to_string(),
            pack_id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            category: "test".to_string(),
            platforms: vec!["windows".to_string()],
            version_requirement: None,
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            detect: DetectConfig {
                command: "true".to_string(),
                version_regex: r"(?P<version>\d+\.\d+\.\d+)".to_string(),
                fallback_command: None,
            },
            install: None,
        }
    }

    #[test]
    fn build_install_order_no_deps() {
        let manifests = vec![manifest("a", &[])];
        let order = build_install_order(&manifests, "a");
        assert_eq!(order.len(), 1);
        assert_eq!(order[0].pack_id, "a");
    }

    #[test]
    fn build_install_order_single_dep() {
        // b depends on a → install order: [a, b]
        let manifests = vec![manifest("a", &[]), manifest("b", &["a"])];
        let order = build_install_order(&manifests, "b");
        assert_eq!(order.len(), 2);
        assert_eq!(order[0].pack_id, "a");
        assert_eq!(order[1].pack_id, "b");
    }

    #[test]
    fn build_install_order_chain() {
        // c → b → a → install order: [a, b, c]
        let manifests = vec![manifest("a", &[]), manifest("b", &["a"]), manifest("c", &["b"])];
        let order = build_install_order(&manifests, "c");
        assert_eq!(order.iter().map(|m| m.pack_id.as_str()).collect::<Vec<_>>(), vec!["a", "b", "c"]);
    }

    #[test]
    fn build_install_order_unknown_target() {
        let manifests = vec![manifest("a", &[])];
        let order = build_install_order(&manifests, "nonexistent");
        assert!(order.is_empty());
    }

    #[test]
    fn install_method_str_winget() {
        let mut m = manifest("x", &[]);
        m.install = Some(InstallConfig {
            windows: Some(PlatformInstall {
                method: InstallMethod::Winget,
                package: Some("Foo.Bar".to_string()),
                script: None,
                checksum: None,
                requires_admin: false,
                requires_reboot: false,
            }),
        });
        assert_eq!(install_method_str(&m), "winget");
    }
}
