use std::collections::HashMap;
use std::sync::Mutex;

use super::types::{PackId, PackState};

pub struct StateStore(Mutex<HashMap<PackId, PackState>>);

impl StateStore {
    #[allow(dead_code)]
    pub fn new(ids: impl IntoIterator<Item = PackId>) -> Self {
        let map = ids.into_iter().map(|id| (id, PackState::Undetected)).collect();
        Self(Mutex::new(map))
    }

    /// 3.3: Initialize with pre-loaded states from lock file.
    /// 3.4: States not present in `initial` start as Undetected — crash recovery is implicit
    ///      since transient states (Installing, Detecting) are never persisted to the lock file.
    pub fn new_with_states(
        ids: impl IntoIterator<Item = PackId>,
        initial: HashMap<PackId, PackState>,
    ) -> Self {
        let map = ids
            .into_iter()
            .map(|id| {
                let state = initial.get(&id).cloned().unwrap_or(PackState::Undetected);
                (id, state)
            })
            .collect();
        Self(Mutex::new(map))
    }

    pub fn get(&self, id: &str) -> PackState {
        self.0.lock().unwrap().get(id).cloned().unwrap_or(PackState::Undetected)
    }

    pub fn set(&self, id: PackId, state: PackState) {
        self.0.lock().unwrap().insert(id, state);
    }

    pub fn snapshot(&self) -> HashMap<PackId, PackState> {
        self.0.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(v: &[&str]) -> Vec<PackId> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn new_defaults_to_undetected() {
        let store = StateStore::new(ids(&["a", "b"]));
        assert!(matches!(store.get("a"), PackState::Undetected));
        assert!(matches!(store.get("b"), PackState::Undetected));
    }

    #[test]
    fn new_with_states_uses_initial() {
        let mut initial = HashMap::new();
        initial.insert("a".to_string(), PackState::Installed { version: "1.0.0".to_string(), pending_reboot: false });
        let store = StateStore::new_with_states(ids(&["a", "b"]), initial);
        assert!(matches!(store.get("a"), PackState::Installed { .. }));
        assert!(matches!(store.get("b"), PackState::Undetected)); // not in initial
    }

    #[test]
    fn set_and_get_round_trips() {
        let store = StateStore::new(ids(&["x"]));
        store.set("x".to_string(), PackState::NotInstalled);
        assert!(matches!(store.get("x"), PackState::NotInstalled));
    }

    #[test]
    fn snapshot_clones_all_states() {
        let store = StateStore::new(ids(&["a", "b"]));
        store.set("a".to_string(), PackState::Detecting);
        let snap = store.snapshot();
        assert_eq!(snap.len(), 2);
        assert!(matches!(snap["a"], PackState::Detecting));
    }

    /// 3.4: Crash recovery — transient states not in lock file start as Undetected
    #[test]
    fn crash_recovery_transient_states_start_fresh() {
        // Simulate: lock file only has Installed states
        let mut initial = HashMap::new();
        initial.insert("nodejs".to_string(), PackState::Installed { version: "20.0.0".to_string(), pending_reboot: false });
        // "python" was in Installing at crash — not in lock file
        let store = StateStore::new_with_states(ids(&["nodejs", "python"]), initial);
        assert!(matches!(store.get("nodejs"), PackState::Installed { .. }));
        assert!(matches!(store.get("python"), PackState::Undetected)); // reset to retryable
    }
}
