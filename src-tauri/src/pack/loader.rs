use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;

use super::types::{Manifest, PackId};

pub struct PackLoader;

impl PackLoader {
    pub fn scan(packs_dir: &Path) -> Result<Vec<Manifest>, String> {
        let mut paths = vec![];
        collect_manifests(packs_dir, &mut paths)?;
        paths.sort();

        let mut manifests = vec![];
        for path in paths {
            let src = fs::read_to_string(&path)
                .map_err(|e| format!("read `{}`: {e}", path.display()))?;

            let m: Manifest = match toml::from_str(&src) {
                Ok(m) => m,
                Err(e) => {
                    log::warn!("skip `{}`: {e}", path.display());
                    continue;
                }
            };

            if m.schema_version != "1.0" {
                log::warn!("skip `{}`: unsupported schema_version {}", path.display(), m.schema_version);
                continue;
            }
            if !supports_current_platform(&m) {
                continue;
            }

            manifests.push(m);
        }

        topological_sort(manifests)
    }
}

fn collect_manifests(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("read_dir `{}`: {e}", dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_manifests(&path, out)?;
        } else if entry.file_name().eq_ignore_ascii_case("manifest.toml") {
            out.push(path);
        }
    }
    Ok(())
}

fn supports_current_platform(m: &Manifest) -> bool {
    let os = std::env::consts::OS;
    m.platforms.iter().any(|p| p.eq_ignore_ascii_case(os))
}

fn topological_sort(manifests: Vec<Manifest>) -> Result<Vec<Manifest>, String> {
    let mut by_id: HashMap<PackId, Manifest> = HashMap::new();
    for m in manifests {
        let id = m.pack_id.clone();
        if by_id.insert(id.clone(), m).is_some() {
            return Err(format!("duplicate pack_id `{id}`"));
        }
    }

    let mut indegree: HashMap<PackId, usize> =
        by_id.keys().cloned().map(|id| (id, 0)).collect();
    let mut adjacency: HashMap<PackId, Vec<PackId>> = HashMap::new();

    for (id, manifest) in &by_id {
        for dep in &manifest.dependencies {
            if !indegree.contains_key(dep) {
                return Err(format!("unknown dependency `{dep}` required by `{id}`"));
            }
            adjacency.entry(dep.clone()).or_default().push(id.clone());
            *indegree.get_mut(id).unwrap() += 1;
        }
    }

    let mut ready: BTreeSet<PackId> = indegree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| id.clone())
        .collect();

    let mut ordered = Vec::with_capacity(by_id.len());
    while let Some(id) = ready.iter().next().cloned() {
        ready.remove(&id);
        ordered.push(id.clone());
        if let Some(dependents) = adjacency.get(&id) {
            let mut sorted = dependents.clone();
            sorted.sort();
            for dep in sorted {
                let deg = indegree.get_mut(&dep).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    ready.insert(dep);
                }
            }
        }
    }

    if ordered.len() != by_id.len() {
        return Err("circular dependency detected".to_string());
    }

    Ok(ordered.into_iter().filter_map(|id| by_id.remove(&id)).collect())
}
