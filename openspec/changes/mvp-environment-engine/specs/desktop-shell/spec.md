## ADDED Requirements

### Requirement: Application bootstrap
The application SHALL start as a Tauri 2 desktop window with a Vue 3 frontend. The Rust backend SHALL initialize the Pack loader, state machine, and Orchestrator during the `setup` hook before the main window is shown.

#### Scenario: Application cold start
- **WHEN** the user launches C-Box
- **THEN** the application window appears within 2 seconds, displaying a loading indicator while backend initialization completes

#### Scenario: Backend initialization failure
- **WHEN** the Pack loader fails to read the `packs/` directory during startup
- **THEN** the application displays an error message in the main window with details and a "Retry" button

### Requirement: Pack list view
The frontend SHALL display a list of all loaded Packs, showing for each: name, description, category, current state (as a colored status indicator), and installed version (if detected).

#### Scenario: Display Pack list after detection
- **WHEN** backend initialization and detection complete with 6 Packs (3 installed, 3 not installed)
- **THEN** the UI renders 6 Pack cards: 3 with green status indicators showing versions, 3 with gray indicators showing "Not Installed"

### Requirement: One-click detect all
The frontend SHALL provide a "Detect All" button that triggers batch detection of all Packs and updates the UI in real-time as each Pack's detection completes.

#### Scenario: Detect all button
- **WHEN** the user clicks "Detect All"
- **THEN** each Pack card transitions to a "Detecting..." state with a spinner, and updates to the result (installed/not installed) as each detection completes

### Requirement: One-click install
Each Pack card in `NotInstalled` state SHALL display an "Install" button. Clicking it SHALL trigger installation via the Orchestrator and display a progress indicator.

#### Scenario: Install single Pack
- **WHEN** the user clicks "Install" on the Node.js Pack card
- **THEN** the card shows an installation progress bar with streaming output, and transitions to "Installed" with a green indicator upon success

#### Scenario: Install failure display
- **WHEN** a Pack installation fails
- **THEN** the card shows a red status indicator, an error message summary, and a "Retry" button

### Requirement: Install all missing
The frontend SHALL provide an "Install All Missing" button that installs all Packs in `NotInstalled` state, respecting dependency order.

#### Scenario: Install all missing Packs
- **WHEN** 3 Packs are in `NotInstalled` state and the user clicks "Install All Missing"
- **THEN** the system installs them in topological order, showing progress for each, and reports a summary upon completion

### Requirement: Real-time progress display
The frontend SHALL subscribe to Tauri Events (`pack:state-changed`, `pack:install-output`) and render live progress: state transitions, progress percentage, and installation command output.

#### Scenario: Live output streaming
- **WHEN** a Pack is being installed and the backend emits `pack:install-output` events
- **THEN** the frontend displays the output lines in a collapsible log panel within the Pack card

### Requirement: i18n architecture
The frontend SHALL use vue-i18n with locale files. The MVP SHALL include Chinese (zh-CN) locale only, with the architecture supporting additional locales without code changes.

#### Scenario: Display Chinese UI
- **WHEN** the application starts with default locale zh-CN
- **THEN** all UI labels, buttons, status messages, and error messages are displayed in Chinese
