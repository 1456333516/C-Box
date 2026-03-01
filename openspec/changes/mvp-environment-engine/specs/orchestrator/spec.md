## ADDED Requirements

### Requirement: Typed request routing
The Orchestrator SHALL accept typed `OrchestratorRequest` enums via Tauri Commands and route them to the appropriate module handler. Supported requests: `InstallPack`, `DetectAll`, `DetectPack`, `RetryPack`.

#### Scenario: Route InstallPack request
- **WHEN** the frontend invokes the `orchestrator_dispatch` Tauri Command with an `InstallPack { pack_id: "nodejs" }` request
- **THEN** the Orchestrator delegates to the Pack installer module and returns an `OrchestratorResponse` upon completion

#### Scenario: Route DetectAll request
- **WHEN** the frontend invokes `orchestrator_dispatch` with a `DetectAll` request
- **THEN** the Orchestrator triggers batch detection across all loaded Packs and returns a `DetectionReport`

### Requirement: Typed response contract
The Orchestrator SHALL return typed `OrchestratorResponse` enums for each request: `Progress`, `Complete`, `DetectionReport`, `Error`.

#### Scenario: Return error for unknown Pack
- **WHEN** an `InstallPack` request references a pack_id that does not exist in the loaded Packs
- **THEN** the Orchestrator returns an `Error` response with message "Pack not found: <pack_id>"

### Requirement: Concurrent request guard
The Orchestrator SHALL prevent concurrent operations on the same Pack. If a Pack is already being installed or detected, a new request for the same Pack SHALL be rejected with a "busy" error.

#### Scenario: Reject concurrent install
- **WHEN** Pack "nodejs" is currently in `Installing` state and a new `InstallPack { pack_id: "nodejs" }` request arrives
- **THEN** the Orchestrator rejects the request with an error: "Pack nodejs is currently busy"

### Requirement: Dependency-aware installation
When installing a Pack, the Orchestrator SHALL check if all dependencies are satisfied (in `Installed` or `Configured` state). Unsatisfied dependencies SHALL be installed first in topological order.

#### Scenario: Auto-install dependency
- **WHEN** the user requests to install Pack "npm" which depends on "nodejs", and "nodejs" is in `NotInstalled` state
- **THEN** the Orchestrator first installs "nodejs", and upon success, proceeds to install "npm"

#### Scenario: Dependency installation failure
- **WHEN** the Orchestrator auto-installs dependency "nodejs" for Pack "npm" but "nodejs" installation fails
- **THEN** the "npm" installation is aborted with an error: "Dependency nodejs failed to install"
