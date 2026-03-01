## ADDED Requirements

### Requirement: Pack manifest loading
The system SHALL scan the `packs/` directory on startup, locate all `manifest.toml` files, and parse them into typed Pack definitions. Invalid manifests SHALL be logged and skipped without blocking other Packs.

#### Scenario: Load valid manifests on startup
- **WHEN** the application starts and `packs/` contains 6 valid `manifest.toml` files
- **THEN** the system loads all 6 Pack definitions into memory and reports them as available

#### Scenario: Skip invalid manifest
- **WHEN** a `manifest.toml` file has invalid TOML syntax or missing required fields
- **THEN** the system logs a warning with the file path and error detail, skips that Pack, and continues loading the remaining Packs

### Requirement: Schema version validation
The system SHALL validate the `schema_version` field in each `manifest.toml`. If the schema version is higher than the supported version, the Pack SHALL be skipped with a warning.

#### Scenario: Reject unsupported schema version
- **WHEN** a manifest declares `schema_version = "2.0"` but the engine only supports `"1.0"`
- **THEN** the system logs a warning indicating version incompatibility and skips the Pack

### Requirement: Platform filtering
The system SHALL filter Packs by the current platform. Only Packs that include the current OS in their `platforms` array SHALL be loaded.

#### Scenario: Filter out non-Windows Pack on Windows
- **WHEN** running on Windows and a Pack declares `platforms = ["linux", "macos"]`
- **THEN** the Pack is excluded from the loaded Pack list

### Requirement: Dependency graph construction
The system SHALL construct a DAG (Directed Acyclic Graph) from the `dependencies` fields of all loaded Packs. Circular dependencies SHALL be detected and reported as errors.

#### Scenario: Build valid dependency graph
- **WHEN** Pack "npm" declares `dependencies = ["nodejs"]` and Pack "nodejs" declares `dependencies = []`
- **THEN** the system constructs a DAG where "nodejs" must be processed before "npm"

#### Scenario: Detect circular dependency
- **WHEN** Pack "A" depends on "B" and Pack "B" depends on "A"
- **THEN** the system reports a circular dependency error listing the cycle path and refuses to proceed with installation

### Requirement: Topological sort for installation order
The system SHALL perform topological sort on the dependency DAG to determine installation order. Packs with no dependencies SHALL be processed first.

#### Scenario: Produce correct installation order
- **WHEN** the dependency graph is: npm → nodejs, nodejs → (none), git → (none), claude-code → nodejs
- **THEN** the topological sort produces an order where nodejs appears before npm and claude-code, and git can appear at any position
