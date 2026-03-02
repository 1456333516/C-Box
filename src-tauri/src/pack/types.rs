use serde::{Deserialize, Serialize};

pub type PackId = String;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Manifest {
    pub schema_version: String,
    pub pack_id: PackId,
    pub name: String,
    pub description: String,
    pub category: String,
    pub platforms: Vec<String>,
    pub version_requirement: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<PackId>,
    pub detect: DetectConfig,
    pub install: Option<InstallConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DetectConfig {
    pub command: String,
    pub version_regex: String,
    pub fallback_command: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstallConfig {
    pub windows: Option<PlatformInstall>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlatformInstall {
    pub method: InstallMethod,
    pub package: Option<String>,
    pub script: Option<String>,
    pub checksum: Option<String>,
    #[serde(default)]
    pub requires_admin: bool,
    #[serde(default)]
    pub requires_reboot: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstallMethod {
    Winget,
    Scoop,
    Script,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum PackState {
    Undetected,
    Detecting,
    NotInstalled,
    Downloading,
    Installing,
    Installed {
        version: String,
        #[serde(default)]
        pending_reboot: bool,
    },
    Configured,
    DetectFailed { reason: String },
    DownloadFailed { reason: String },
    InstallFailed { reason: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectResult {
    pub pack_id: PackId,
    pub state: PackState,
    pub installed_version: Option<String>,
}

impl DetectResult {
    pub fn installed(pack_id: &str, version: String) -> Self {
        Self {
            pack_id: pack_id.to_string(),
            installed_version: Some(version.clone()),
            state: PackState::Installed { version, pending_reboot: false },
        }
    }

    pub fn not_installed(pack_id: &str) -> Self {
        Self {
            pack_id: pack_id.to_string(),
            state: PackState::NotInstalled,
            installed_version: None,
        }
    }

    pub fn failed(pack_id: &str, reason: String) -> Self {
        Self {
            pack_id: pack_id.to_string(),
            state: PackState::DetectFailed { reason },
            installed_version: None,
        }
    }
}
