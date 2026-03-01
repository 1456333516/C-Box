## ADDED Requirements

### Requirement: Command-based detection
The system SHALL detect installed tools by executing the `detect.command` from the Pack manifest via Shell plugin, parsing stdout with `detect.version_regex` to extract the version string.

#### Scenario: Detect installed Node.js
- **WHEN** the system runs `node --version` and stdout returns `v22.14.0`
- **THEN** the detected version is parsed as `22.14.0` and compared against `version_requirement`

#### Scenario: Detect tool not installed
- **WHEN** the system runs `node --version` and the command returns a non-zero exit code or command not found
- **THEN** the Pack is marked as `NotInstalled`

### Requirement: Fallback detection
If the primary `detect.command` fails, the system SHALL attempt `detect.fallback_command` (if defined) before marking detection as failed.

#### Scenario: Primary command fails but fallback succeeds
- **WHEN** `python --version` fails (command not found) but `python3 --version` returns `Python 3.12.0`
- **THEN** the version is parsed as `3.12.0` from the fallback command output

### Requirement: Windows stdout encoding normalization
On Windows, PowerShell's default stdout encoding is locale-dependent (e.g., GBK on Chinese Windows). The system SHALL prepend `[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; ` to every detection command before passing it to PowerShell's `-Command` argument. The Rust side SHALL decode stdout bytes with `String::from_utf8_lossy`, then normalize CRLF to LF and trim surrounding whitespace before regex matching.

#### Scenario: Detection on Chinese Windows with GBK locale
- **WHEN** running on Chinese Windows (default OEM codepage GBK) and `node --version` is executed
- **THEN** the prepended `[Console]::OutputEncoding` declaration forces UTF-8 byte output, and Rust decodes the result as `v22.14.0` with no garbled characters

### Requirement: Detection timeout
Each detection command SHALL have a 10-second timeout. Commands exceeding the timeout SHALL be treated as detection failures.

#### Scenario: Detection command hangs
- **WHEN** a detection command does not return within 10 seconds
- **THEN** the command is killed, and the Pack transitions to `DetectFailed` with a timeout error message

### Requirement: Version comparison
The system SHALL compare detected versions against `version_requirement` using semver range matching. Versions not satisfying the requirement SHALL be treated as `NotInstalled` with a warning showing the current version.

#### Scenario: Installed version too old
- **WHEN** Node.js `16.0.0` is detected but `version_requirement` is `>=18.0.0`
- **THEN** the Pack is marked as `NotInstalled` with a message: "Installed version 16.0.0 does not meet requirement >=18.0.0"

### Requirement: Batch detection
The system SHALL support detecting all loaded Packs in a single operation (`DetectAll`), executing detections in topological order and reporting results as a batch.

#### Scenario: Detect all Packs
- **WHEN** the user triggers "Detect All" with 6 loaded Packs
- **THEN** the system sequentially detects each Pack and emits individual state change events for each, completing with a summary report
