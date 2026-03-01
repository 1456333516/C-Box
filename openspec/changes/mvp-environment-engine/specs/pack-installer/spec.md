## ADDED Requirements

### Requirement: Multi-method installation
The system SHALL support multiple installation methods as declared in `manifest.toml`: `winget`, `scoop`, `script`. The method is selected based on the `[install.<platform>].method` field.

#### Scenario: Install via winget
- **WHEN** a Pack declares `method = "winget"` and `package = "OpenJS.NodeJS.LTS"` on Windows
- **THEN** the system executes `winget install --id OpenJS.NodeJS.LTS --accept-source-agreements --accept-package-agreements` via the PowerShell script executor

#### Scenario: Install via script
- **WHEN** a Pack declares `method = "script"` and `script = "install.ps1"` on Windows
- **THEN** the system executes the script located at `packs/<pack_id>/install.ps1` via the PowerShell script executor

### Requirement: Unified script executor
All installation commands SHALL be executed through the unified script executor (PowerShell on Windows). The system SHALL NOT allow the frontend to pass arbitrary command strings; only commands derived from registered Pack manifests are permitted.

#### Scenario: Reject unregistered command
- **WHEN** a request to execute a command arrives that does not match any registered Pack manifest template
- **THEN** the system rejects the request with an error and logs the attempted command for security audit

### Requirement: UAC privilege escalation
Packs with `requires_admin = true` SHALL trigger a UAC elevation prompt on Windows before installation begins.

#### Scenario: Install requiring admin privileges
- **WHEN** a Pack declares `requires_admin = true` and the current process is not elevated
- **THEN** the system requests UAC elevation; if the user denies, the Pack transitions to `InstallFailed` with a "permission denied" message

### Requirement: Reboot notification
Packs with `requires_reboot = true` SHALL display a notification after successful installation, informing the user that a system reboot is needed. The Pack state SHALL be set to `Installed` with a `pending_reboot` flag.

#### Scenario: Installation requiring reboot
- **WHEN** a Pack with `requires_reboot = true` completes installation
- **THEN** the Pack state is `Installed` with metadata `pending_reboot: true`, and the UI displays a reboot notification

### Requirement: PATH environment refresh
After successful installation, the system SHALL refresh the PATH environment on Windows by broadcasting `WM_SETTINGCHANGE` to notify other processes of the environment change.

#### Scenario: PATH refresh after Node.js installation
- **WHEN** Node.js is installed via winget and the install completes
- **THEN** the system broadcasts `WM_SETTINGCHANGE` and re-runs detection to verify the installation succeeded

### Requirement: Installation progress streaming
The system SHALL stream stdout/stderr from installation commands in real-time via Tauri Events, enabling the frontend to display live installation output.

#### Scenario: Stream installation output
- **WHEN** a winget installation is running and producing output lines
- **THEN** each line is emitted as a Tauri Event `pack:install-output` with payload `{ pack_id, stream: "stdout"|"stderr", line }`

### Requirement: Installation checksum verification
For Packs using `url` or `script` method with a non-empty `checksum` field, the system SHALL verify the SHA-256 hash of the downloaded file before execution. Mismatched checksums SHALL abort the installation.

#### Scenario: Checksum mismatch
- **WHEN** a downloaded installer file's SHA-256 hash does not match the declared `checksum`
- **THEN** the system deletes the file, transitions the Pack to `InstallFailed`, and logs a tamper warning
