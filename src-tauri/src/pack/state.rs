use std::collections::HashMap;
use std::sync::Mutex;

use super::types::{PackId, PackState};

pub struct StateStore(Mutex<HashMap<PackId, PackState>>);

impl StateStore {
    pub fn new(ids: impl IntoIterator<Item = PackId>) -> Self {
        let map = ids.into_iter().map(|id| (id, PackState::Undetected)).collect();
        Self(Mutex::new(map))
    }

    pub fn get(&self, id: &str) -> PackState {
        self.0
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .unwrap_or(PackState::Undetected)
    }

    pub fn set(&self, id: PackId, state: PackState) {
        self.0.lock().unwrap().insert(id, state);
    }

    pub fn snapshot(&self) -> HashMap<PackId, PackState> {
        self.0.lock().unwrap().clone()
    }
}
