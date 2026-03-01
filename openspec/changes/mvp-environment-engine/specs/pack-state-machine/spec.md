## ADDED Requirements

### Requirement: State machine lifecycle
The system SHALL manage each Pack through a defined state machine with states: `Undetected`, `Detecting`, `NotInstalled`, `Downloading`, `Installing`, `Installed`, `Configured`, `DetectFailed`, `DownloadFailed`, `InstallFailed`.

#### Scenario: Normal lifecycle progression
- **WHEN** a Pack transitions from `Undetected` through detection and installation
- **THEN** the state progresses as: `Undetected` → `Detecting` → `NotInstalled` → `Downloading` → `Installing` → `Installed`

#### Scenario: Detection finds tool already installed
- **WHEN** a Pack is in `Detecting` state and the detection command returns a valid version matching `version_requirement`
- **THEN** the state transitions directly to `Installed`

### Requirement: Error states with retry
Each failure state (`DetectFailed`, `DownloadFailed`, `InstallFailed`) SHALL allow retry, transitioning back to the corresponding action state (`Detecting`, `Downloading`, `Installing`).

#### Scenario: Retry after install failure
- **WHEN** a Pack is in `InstallFailed` state and the user triggers retry
- **THEN** the state transitions to `Installing` and the installation is re-attempted

#### Scenario: Retry after detect failure
- **WHEN** a Pack is in `DetectFailed` state and the user triggers retry
- **THEN** the state transitions to `Detecting` and detection is re-attempted

### Requirement: State persistence
The system SHALL persist Pack states to local storage via `@tauri-apps/plugin-store`. Critical state transitions (entering `Downloading`, `Installing`, `Installed`, any failure state) SHALL trigger an immediate store flush.

#### Scenario: Persist state on install completion
- **WHEN** a Pack transitions to `Installed` state
- **THEN** the system immediately calls `store.save()` to flush the state to disk

#### Scenario: Recover from crash during installation
- **WHEN** the application was terminated while a Pack was in `Installing` state and the application restarts
- **THEN** the system reads the persisted state, detects the interrupted installation, and resets that Pack to `NotInstalled` with a retry flag

### Requirement: State change notification
The system SHALL emit a Tauri Event on every state transition, containing the Pack ID, previous state, new state, and optional progress percentage.

#### Scenario: Emit event on state change
- **WHEN** a Pack transitions from `Downloading` to `Installing`
- **THEN** a Tauri Event `pack:state-changed` is emitted with payload `{ pack_id, from: "Downloading", to: "Installing", percent: 100 }`
