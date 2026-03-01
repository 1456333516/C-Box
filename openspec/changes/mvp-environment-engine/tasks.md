## 1. Project Scaffolding

- [x] 1.1 Initialize Tauri 2 + Vue 3 + TypeScript project via `npm create tauri-app@latest`
- [x] 1.2 Configure Tauri plugins in `Cargo.toml`: shell, store, log, updater, fs
- [x] 1.3 Configure Tauri capabilities: Shell script executor whitelist (PowerShell/bash)
- [x] 1.4 Create Rust module structure: `orchestrator/`, `pack/`, `detector/`, `platform/`
- [x] 1.5 Create Vue frontend structure: `components/`, `composables/`, `stores/`, `types/`
- [x] 1.6 Set up vue-i18n with zh-CN locale file
- [x] 1.7 Create `packs/` directory with 6 built-in Pack `manifest.toml` files (nodejs, python, git, uv, claude-code, npm)

## 2. Pack Loader (Rust)

- [x] 2.1 Define `Pack` struct and `Manifest` deserialization types matching `manifest.toml` schema
- [x] 2.2 Implement `PackLoader::scan()` to discover and parse all `manifest.toml` from `packs/` directory
- [x] 2.3 Implement `schema_version` validation (reject versions higher than supported)
- [x] 2.4 Implement platform filtering (exclude Packs not matching current OS)
- [x] 2.5 Implement DAG dependency graph construction from `dependencies` fields
- [x] 2.6 Implement circular dependency detection with error reporting
- [x] 2.7 Implement topological sort to produce installation order

## 3. Pack State Machine (Rust)

- [x] 3.1 Define `PackState` enum with all states: Undetected, Detecting, NotInstalled, Downloading, Installing, Installed, Configured, DetectFailed, DownloadFailed, InstallFailed
- [x] 3.2 Implement state transition validation (only allow valid transitions)
- [ ] 3.3 Implement state persistence via `@tauri-apps/plugin-store` with autoSave and forced flush on critical transitions
- [ ] 3.4 Implement crash recovery logic: detect interrupted states on startup and reset to retryable state
- [x] 3.5 Implement Tauri Event emission (`pack:state-changed`) on every state transition

## 4. Environment Detector (Rust)

- [x] 4.1 Implement `detect_pack()` function: execute detection command via Shell plugin with 10s timeout
- [x] 4.2 Implement stdout parsing with version_regex extraction
- [x] 4.3 Implement fallback_command logic when primary detection fails
- [x] 4.4 Implement semver version comparison against `version_requirement`
- [x] 4.5 Implement `detect_all()` batch detection with sequential execution and result aggregation

## 5. Pack Installer (Rust)

- [ ] 5.1 Implement unified script executor: construct PowerShell commands from manifest templates
- [ ] 5.2 Implement `winget` installation method: command construction, execution, output streaming
- [ ] 5.3 Implement `scoop` installation method
- [ ] 5.4 Implement `script` installation method: locate and execute Pack-local .ps1 scripts
- [ ] 5.5 Implement command template validation: reject commands not derived from registered manifests
- [ ] 5.6 Implement stdout/stderr streaming via Tauri Events (`pack:install-output`)
- [ ] 5.7 Implement UAC privilege escalation for `requires_admin = true` Packs
- [ ] 5.8 Implement PATH refresh via `WM_SETTINGCHANGE` broadcast after installation
- [ ] 5.9 Implement post-install re-detection to verify installation success
- [ ] 5.10 Implement `requires_reboot` flag handling and notification
- [ ] 5.11 Implement checksum (SHA-256) verification for url/script methods

## 6. Orchestrator (Rust)

- [x] 6.1 Define `OrchestratorRequest` and `OrchestratorResponse` enums
- [x] 6.2 Implement Tauri Command `orchestrator_dispatch` as the single entry point
- [x] 6.3 Implement request routing: InstallPack, DetectAll, DetectPack, RetryPack
- [x] 6.4 Implement concurrent request guard (prevent duplicate operations on same Pack)
- [ ] 6.5 Implement dependency-aware installation: check and auto-install missing dependencies before target Pack

## 7. Environment Lock File (Rust)

- [ ] 7.1 Define `environment.lock.toml` schema and serialization types
- [ ] 7.2 Implement auto-generation after detection/installation: record pack_id, installed_version, installed_at, install_method, checksum
- [ ] 7.3 Implement lock file read on startup to initialize state from previous session

## 8. Frontend - Pack List UI (Vue 3)

- [x] 8.1 Create `PackCard` component: name, description, category badge, status indicator (color-coded), version display
- [x] 8.2 Create `PackList` view: grid/list of PackCards with "Detect All" and "Install All Missing" buttons
- [x] 8.3 Implement Pinia store for Pack state management (synced with Rust backend via Tauri Events)
- [x] 8.4 Implement "Detect All" button: invoke Orchestrator, show per-Pack detection spinners
- [ ] 8.5 Implement per-Pack "Install" button: invoke Orchestrator, show progress bar
- [ ] 8.6 Implement "Install All Missing" button: batch install with dependency-order progress
- [ ] 8.7 Implement collapsible log panel per Pack for real-time installation output
- [x] 8.8 Implement error state display: red indicator, error message, "Retry" button
- [ ] 8.9 Implement reboot notification banner for `pending_reboot` Packs

## 9. Integration & Testing

- [ ] 9.1 Write Rust unit tests for Pack loader (valid/invalid manifests, platform filtering, DAG sort)
- [ ] 9.2 Write Rust unit tests for state machine (transitions, persistence, crash recovery)
- [ ] 9.3 Write Rust unit tests for Orchestrator (routing, concurrency guard, dependency resolution)
- [ ] 9.4 Manual integration test: full lifecycle on Windows (detect → install → verify) for all 6 Packs
- [ ] 9.5 Manual test: crash recovery scenario (kill during installation, restart, verify state)
- [ ] 9.6 Manual test: edge cases (no network, UAC denied, version conflict)
