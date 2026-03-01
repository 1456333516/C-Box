export type PackId = string

// Matches Rust PackState serde(tag = "type", content = "data")
export type PackState =
  | { type: 'undetected' }
  | { type: 'detecting' }
  | { type: 'installed'; data: { version: string } }
  | { type: 'not_installed' }
  | { type: 'detect_failed'; data: { reason: string } }

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
