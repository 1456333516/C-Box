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

pub(crate) fn topological_sort(manifests: Vec<Manifest>) -> Result<Vec<Manifest>, String> {
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

// 9.1: Unit tests for Pack loader
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::pack::types::DetectConfig;

    fn mini(id: &str, deps: &[&str]) -> Manifest {
        Manifest {
            schema_version: "1.0".to_string(),
            pack_id: id.to_string(),
            name: id.to_string(),
            description: String::new(),
            category: "test".to_string(),
            platforms: vec!["windows".to_string(), "linux".to_string(), "macos".to_string()],
            version_requirement: None,
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            detect: DetectConfig {
                command: "true".to_string(),
                version_regex: r"(?P<version>\d+\.\d+\.\d+)".to_string(),
                fallback_command: None,
            },
            install: None,
        }
    }

    // Helpers for file-based scan tests
    fn write_manifest(dir: &Path, id: &str, content: &str) {
        let pack_dir = dir.join(id);
        std::fs::create_dir_all(&pack_dir).unwrap();
        std::fs::write(pack_dir.join("manifest.toml"), content).unwrap();
    }

    fn temp_dir(suffix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("c_box_test_{suffix}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn scan_valid_manifest() {
        let dir = temp_dir("scan_valid");
        write_manifest(&dir, "nodejs", r#"
schema_version = "1.0"
pack_id = "nodejs"
name = "Node.js"
description = "test"
category = "runtime"
platforms = ["windows", "linux", "macos"]
[detect]
command = "node --version"
version_regex = 'v?(?P<version>\d+\.\d+\.\d+)'
"#);
        let result = PackLoader::scan(&dir).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pack_id, "nodejs");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn scan_skips_invalid_schema_version() {
        let dir = temp_dir("scan_schema");
        write_manifest(&dir, "bad", r#"
schema_version = "99.0"
pack_id = "bad"
name = "Bad"
description = "test"
category = "test"
platforms = ["windows"]
[detect]
command = "x"
version_regex = '(?P<version>\d+)'
"#);
        let result = PackLoader::scan(&dir).unwrap();
        assert!(result.is_empty(), "unsupported schema_version should be skipped");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn scan_platform_filtering() {
        let dir = temp_dir("scan_platform");
        // Only has "linux" platform — on any OS, only matches that OS
        write_manifest(&dir, "linuxonly", &format!(r#"
schema_version = "1.0"
pack_id = "linuxonly"
name = "Linux Only"
description = "test"
category = "test"
platforms = ["{current_os}"]
[detect]
command = "x"
version_regex = '(?P<version>\d+)'
"#, current_os = std::env::consts::OS));
        write_manifest(&dir, "other", r#"
schema_version = "1.0"
pack_id = "other"
name = "Other OS Only"
description = "test"
category = "test"
platforms = ["__nonexistent_os__"]
[detect]
command = "x"
version_regex = '(?P<version>\d+)'
"#);
        let result = PackLoader::scan(&dir).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pack_id, "linuxonly");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn topological_sort_orders_deps_first() {
        // b depends on a → [a, b]
        let result = topological_sort(vec![mini("b", &["a"]), mini("a", &[])]).unwrap();
        let ids: Vec<_> = result.iter().map(|m| m.pack_id.as_str()).collect();
        let pos_a = ids.iter().position(|&s| s == "a").unwrap();
        let pos_b = ids.iter().position(|&s| s == "b").unwrap();
        assert!(pos_a < pos_b, "dependency 'a' must come before dependent 'b'");
    }

    #[test]
    fn topological_sort_detects_circular() {
        // a → b → a
        let a = mini("a", &["b"]);
        let b = mini("b", &["a"]);
        let err = topological_sort(vec![a, b]).unwrap_err();
        assert!(err.contains("circular"), "expected circular dependency error, got: {err}");
    }

    #[test]
    fn topological_sort_rejects_unknown_dep() {
        let m = mini("a", &["nonexistent"]);
        let err = topological_sort(vec![m]).unwrap_err();
        assert!(err.contains("unknown dependency"), "got: {err}");
    }

    #[test]
    fn topological_sort_rejects_duplicate_id() {
        let err = topological_sort(vec![mini("a", &[]), mini("a", &[])]).unwrap_err();
        assert!(err.contains("duplicate"), "got: {err}");
    }
}
