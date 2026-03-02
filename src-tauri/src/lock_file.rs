use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::pack::types::{PackId, PackState};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LockFile {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(default)]
    pub packs: HashMap<PackId, LockEntry>,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub installed_version: String,
    pub installed_at: u64, // Unix timestamp
    pub install_method: String,
    #[serde(default)]
    pub checksum: Option<String>,
}

impl LockFile {
    /// 7.3: Load from disk; returns empty if file missing or unparseable.
    pub fn load(path: &Path) -> Self {
        let Ok(src) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        toml::from_str(&src).unwrap_or_default()
    }

    /// Flush to disk.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let src = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, src).map_err(|e| e.to_string())
    }

    /// 7.3: Build initial PackState map from lock entries.
    /// 3.4: Transient states (Installing/Detecting) are never stored here,
    ///      so crash recovery is inherent — interrupted packs start as Undetected.
    pub fn to_initial_states(&self) -> HashMap<PackId, PackState> {
        self.packs
            .iter()
            .map(|(id, e)| {
                (
                    id.clone(),
                    PackState::Installed { version: e.installed_version.clone(), pending_reboot: false },
                )
            })
            .collect()
    }

    /// 7.2: Record a successful install or detection result.
    pub fn record(&mut self, pack_id: &PackId, version: &str, method: &str, checksum: Option<String>) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        self.packs.insert(
            pack_id.clone(),
            LockEntry {
                installed_version: version.to_string(),
                installed_at: now,
                install_method: method.to_string(),
                checksum,
            },
        );
    }

    /// Remove a pack from the lock (e.g., detected as NotInstalled after being in lock).
    pub fn remove(&mut self, pack_id: &PackId) {
        self.packs.remove(pack_id);
    }
}
