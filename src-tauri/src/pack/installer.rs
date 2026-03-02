use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

use super::types::{InstallMethod, Manifest, PackId, PlatformInstall};

pub enum InstallOutcome {
    Success { pending_reboot: bool },
    Failed { reason: String },
}

#[derive(Clone, Serialize)]
struct InstallOutputPayload {
    pack_id: PackId,
    stream: &'static str,
    line: String,
}

pub struct PackInstaller;

impl PackInstaller {
    /// 5.1: Unified entry — construct command from manifest, never from user input.
    pub async fn install(app: &AppHandle, manifest: &Manifest, packs_dir: &Path) -> InstallOutcome {
        // 5.5: Validate install config exists (reject packs with no install definition)
        let Some(install_cfg) = &manifest.install else {
            log::warn!("install rejected: pack '{}' has no install config", manifest.pack_id);
            return InstallOutcome::Failed {
                reason: format!("pack '{}' has no install configuration", manifest.pack_id),
            };
        };

        let Some(platform) = &install_cfg.windows else {
            return InstallOutcome::Failed {
                reason: format!("pack '{}' has no Windows install config", manifest.pack_id),
            };
        };

        // 5.11: Checksum verification for script method
        if platform.method == InstallMethod::Script {
            if let Some(err) = verify_script_checksum(&manifest.pack_id, platform, packs_dir) {
                return InstallOutcome::Failed { reason: err };
            }
        }

        // Build command purely from manifest fields (5.5: template validation)
        let ps_cmd = match build_command(&manifest.pack_id, platform, packs_dir) {
            Ok(cmd) => cmd,
            Err(e) => return InstallOutcome::Failed { reason: e },
        };

        log::info!("installing '{}' via {:?}", manifest.pack_id, platform.method);

        // 5.7: Wrap in UAC elevation if required
        let final_cmd = if platform.requires_admin {
            wrap_as_admin(&ps_cmd)
        } else {
            ps_cmd
        };

        // 5.2 / 5.3 / 5.4 / 5.6: Execute with real-time streaming
        match run_streaming(app, &manifest.pack_id, &final_cmd).await {
            Ok(()) => InstallOutcome::Success { pending_reboot: platform.requires_reboot },
            Err(reason) => InstallOutcome::Failed { reason },
        }
    }
}

/// 5.2 / 5.3 / 5.4: Build the platform-specific install command from manifest fields only.
fn build_command(pack_id: &str, platform: &PlatformInstall, packs_dir: &Path) -> Result<String, String> {
    match platform.method {
        // 5.2: winget
        InstallMethod::Winget => {
            let pkg = platform.package.as_deref().ok_or("winget method requires 'package' field")?;
            Ok(format!(
                "winget install --id {pkg} --accept-source-agreements --accept-package-agreements"
            ))
        }
        // 5.3: scoop
        InstallMethod::Scoop => {
            let pkg = platform.package.as_deref().ok_or("scoop method requires 'package' field")?;
            Ok(format!("scoop install {pkg}"))
        }
        // 5.4: local .ps1 script
        InstallMethod::Script => {
            let script_name = platform.script.as_deref().ok_or("script method requires 'script' field")?;
            let script_path = packs_dir.join(pack_id).join(script_name);
            let path_str = script_path.to_str().ok_or("non-UTF8 script path")?;
            Ok(format!("Set-ExecutionPolicy Bypass -Scope Process -Force; & '{path_str}'"))
        }
    }
}

/// 5.11: Verify SHA-256 of the local script file before execution.
fn verify_script_checksum(pack_id: &str, platform: &PlatformInstall, packs_dir: &Path) -> Option<String> {
    let expected = platform.checksum.as_deref()?;
    let script_name = platform.script.as_deref()?;
    let path = packs_dir.join(pack_id).join(script_name);

    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => return Some(format!("cannot read script '{script_name}': {e}")),
    };

    let hash = hex::encode(Sha256::digest(&data));
    if hash != expected.to_lowercase() {
        log::warn!("TAMPER: checksum mismatch for '{}': expected {expected}, got {hash}", pack_id);
        let _ = std::fs::remove_file(&path);
        Some("checksum mismatch: script may have been tampered (file removed)".to_string())
    } else {
        None
    }
}

/// 5.7: Wrap command to request UAC elevation via PowerShell RunAs verb.
fn wrap_as_admin(cmd: &str) -> String {
    let inner = cmd.replace('\'', "''");
    format!(
        "Start-Process powershell.exe -Verb RunAs -Wait \
         -ArgumentList '-NoProfile -NonInteractive -Command ''{inner}'''"
    )
}

/// 5.6: Spawn process and stream stdout/stderr as `pack:install-output` events.
async fn run_streaming(app: &AppHandle, pack_id: &str, ps_cmd: &str) -> Result<(), String> {
    let full_cmd =
        format!("[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; {ps_cmd}");

    let (mut rx, _child) = app
        .shell()
        .command("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &full_cmd])
        .spawn()
        .map_err(|e| e.to_string())?;

    let mut exit_code: Option<i32> = None;

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(data) => {
                for line in String::from_utf8_lossy(&data).lines() {
                    let l = line.trim().to_string();
                    if !l.is_empty() {
                        emit_output(app, pack_id, "stdout", l);
                    }
                }
            }
            CommandEvent::Stderr(data) => {
                for line in String::from_utf8_lossy(&data).lines() {
                    let l = line.trim().to_string();
                    if !l.is_empty() {
                        emit_output(app, pack_id, "stderr", l);
                    }
                }
            }
            CommandEvent::Terminated(payload) => {
                exit_code = payload.code;
                break;
            }
            _ => {}
        }
    }

    match exit_code {
        Some(0) => Ok(()),
        Some(code) => Err(format!("process exited with code {code}")),
        None => Err("process terminated abnormally".to_string()),
    }
}

fn emit_output(app: &AppHandle, pack_id: &str, stream: &'static str, line: String) {
    let _ = app.emit(
        "pack:install-output",
        InstallOutputPayload { pack_id: pack_id.to_string(), stream, line },
    );
}

/// 5.8: Read updated PATH from registry and broadcast WM_SETTINGCHANGE.
pub async fn refresh_path(app: &AppHandle) {
    let ps = "[System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + \
              [System.Environment]::GetEnvironmentVariable('Path','User')";

    if let Ok(output) = app
        .shell()
        .command("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", ps])
        .output()
        .await
    {
        if output.status.success() {
            let new_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !new_path.is_empty() {
                std::env::set_var("PATH", &new_path);
            }
        }
    }

    // Broadcast WM_SETTINGCHANGE so other processes pick up the new PATH
    let broadcast = r#"Add-Type -MemberDefinition '[DllImport("user32.dll",CharSet=CharSet.Auto,SetLastError=true)] public static extern IntPtr SendMessageTimeout(IntPtr hWnd,uint Msg,UIntPtr wParam,string lParam,uint fuFlags,uint uTimeout,out UIntPtr lpdwResult);' -Name 'NativeAPI' -Namespace 'Win32' -ErrorAction SilentlyContinue; $r=[UIntPtr]::Zero; [Win32.NativeAPI]::SendMessageTimeout([IntPtr]0xFFFF,0x001A,[UIntPtr]::Zero,'Environment',2,5000,[ref]$r)|Out-Null"#;
    let _ = app
        .shell()
        .command("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", broadcast])
        .output()
        .await;
}
