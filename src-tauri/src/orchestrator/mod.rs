use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::detector::EnvironmentDetector;
use crate::pack::state::StateStore;
use crate::pack::types::{Manifest, PackId, PackState};

pub struct AppState {
    pub manifests: Vec<Manifest>,
    pub states: StateStore,
}

impl AppState {
    pub fn new(manifests: Vec<Manifest>) -> Self {
        let ids = manifests.iter().map(|m| m.pack_id.clone());
        Self {
            states: StateStore::new(ids),
            manifests,
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
        StateChangedPayload {
            pack_id: pack_id.clone(),
            state: state.clone(),
        },
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
        let installed_version = if let PackState::Installed { ref version } = state {
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
    state.states.set(result.pack_id.clone(), result.state.clone());
    emit_state(&app, &result.pack_id, &result.state);

    Ok(())
}
