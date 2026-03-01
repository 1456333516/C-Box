use std::time::Duration;

use regex::Regex;
use semver::{Version, VersionReq};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tokio::time::timeout;

use crate::pack::types::{DetectResult, Manifest};

pub struct EnvironmentDetector;

impl EnvironmentDetector {
    pub async fn detect_pack(app: &AppHandle, manifest: &Manifest) -> DetectResult {
        let regex = match Regex::new(&manifest.detect.version_regex) {
            Ok(r) => r,
            Err(e) => {
                return DetectResult::failed(&manifest.pack_id, format!("invalid regex: {e}"))
            }
        };

        match run_command(app, &manifest.detect.command, &regex).await {
            Ok(version) => evaluate_version(manifest, version),
            Err(primary) => match manifest.detect.fallback_command.as_deref() {
                Some(fallback) => match run_command(app, fallback, &regex).await {
                    Ok(version) => evaluate_version(manifest, version),
                    Err(_) => to_detect_result(&manifest.pack_id, primary),
                },
                None => to_detect_result(&manifest.pack_id, primary),
            },
        }
    }
}

enum CmdError {
    NotFound,
    Failed(String),
}

async fn run_command(app: &AppHandle, cmd: &str, regex: &Regex) -> Result<String, CmdError> {
    // Prepend UTF-8 encoding declaration to fix GBK locale on Chinese Windows
    let full_cmd = format!("[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; {cmd}");

    let shell_cmd = app
        .shell()
        .command("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &full_cmd]);

    let output = match timeout(Duration::from_secs(10), shell_cmd.output()).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => return Err(CmdError::Failed(e.to_string())),
        Err(_) => return Err(CmdError::Failed(format!("timeout: {cmd}"))),
    };

    if !output.status.success() {
        return Err(CmdError::NotFound);
    }

    let stdout = String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .trim()
        .to_string();

    extract_version(&stdout, regex)
        .ok_or_else(|| CmdError::Failed(format!("no version match in output of: {cmd}")))
}

fn extract_version(stdout: &str, regex: &Regex) -> Option<String> {
    let caps = regex.captures(stdout)?;
    caps.name("version")
        .or_else(|| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
}

fn evaluate_version(manifest: &Manifest, version: String) -> DetectResult {
    let satisfies = match &manifest.version_requirement {
        None => true,
        Some(req) => match (Version::parse(&version), VersionReq::parse(req)) {
            (Ok(v), Ok(r)) => r.matches(&v),
            _ => false,
        },
    };

    if satisfies {
        DetectResult::installed(&manifest.pack_id, version)
    } else {
        DetectResult::not_installed(&manifest.pack_id)
    }
}

fn to_detect_result(pack_id: &str, err: CmdError) -> DetectResult {
    match err {
        CmdError::NotFound => DetectResult::not_installed(pack_id),
        CmdError::Failed(reason) => DetectResult::failed(pack_id, reason),
    }
}
