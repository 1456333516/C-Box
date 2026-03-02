export type PackId = string

// Matches Rust PackState serde(tag = "type", content = "data")
export type PackState =
  | { type: 'undetected' }
  | { type: 'detecting' }
  | { type: 'not_installed' }
  | { type: 'downloading' }
  | { type: 'installing' }
  | { type: 'installed'; data: { version: string; pending_reboot: boolean } }
  | { type: 'configured' }
  | { type: 'detect_failed'; data: { reason: string } }
  | { type: 'download_failed'; data: { reason: string } }
  | { type: 'install_failed'; data: { reason: string } }

export interface PackSummary {
  pack_id: PackId
  name: string
  description: string
  category: string
  state: PackState
  installed_version: string | null
}

export interface StateChangedPayload {
  pack_id: PackId
  state: PackState
}

export interface InstallOutputPayload {
  pack_id: PackId
  stream: 'stdout' | 'stderr'
  line: string
}
