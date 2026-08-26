//! 코퍼스 샤드·단일 관측 JSON 로더.

use crate::observation::{Observation, UniquenessKey};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Io(String),
    Json(String),
    Duplicate(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(s) | Self::Json(s) | Self::Duplicate(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for LoadError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorpusManifest {
    pub schema_version: String,
    pub generated_by: String,
    pub record_count: u64,
    pub shard_count: u64,
    #[serde(default)]
    pub exit_class_counts: HashMap<String, u64>,
    #[serde(default)]
    pub command_counts: HashMap<String, u64>,
    #[serde(default)]
    pub uniqueness: String,
    #[serde(default)]
    pub shards: Vec<ShardMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardMeta {
    pub path: String,
    pub count: u64,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub exit_classes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorpusShard {
    pub schema_version: String,
    pub shard_id: String,
    pub records: Vec<Observation>,
}

pub fn load_observation_bytes(bytes: &[u8]) -> Result<Observation, LoadError> {
    let mut obs: Observation =
        serde_json::from_slice(bytes).map_err(|e| LoadError::Json(format!("observation: {e}")))?;
    obs.refresh_judgment();
    Ok(obs)
}

pub fn load_observation_path(path: &Path) -> Result<Observation, LoadError> {
    let bytes = fs::read(path).map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    load_observation_bytes(&bytes)
}

pub fn load_shard_bytes(bytes: &[u8]) -> Result<CorpusShard, LoadError> {
    let mut shard: CorpusShard =
        serde_json::from_slice(bytes).map_err(|e| LoadError::Json(format!("shard: {e}")))?;
    for rec in &mut shard.records {
        rec.refresh_judgment();
    }
    Ok(shard)
}

pub fn load_shard_path(path: &Path) -> Result<CorpusShard, LoadError> {
    let bytes = fs::read(path).map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    load_shard_bytes(&bytes)
}

pub fn load_manifest(path: &Path) -> Result<CorpusManifest, LoadError> {
    let bytes = fs::read(path).map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    serde_json::from_slice(&bytes).map_err(|e| LoadError::Json(format!("manifest: {e}")))
}

pub fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn corpus_dir() -> PathBuf {
    crate_dir().join("corpus")
}

pub fn schema_dir() -> PathBuf {
    crate_dir().join("schema")
}

/// 샤드 일부만 읽어 유일키를 검사한다. 전량 로드는 호출자가 고른다.
pub fn load_shards(paths: &[PathBuf]) -> Result<Vec<Observation>, LoadError> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for path in paths {
        let shard = load_shard_path(path)?;
        for rec in shard.records {
            let key = UniquenessKey::from_observation(&rec);
            if !seen.insert(key) {
                return Err(LoadError::Duplicate(format!(
                    "duplicate (command, exitClass, judgment, sourceTag) in {}: {}",
                    path.display(),
                    rec.uniqueness_key()
                )));
            }
            out.push(rec);
        }
    }
    Ok(out)
}

pub fn list_shard_paths(corpus: &Path) -> Result<Vec<PathBuf>, LoadError> {
    let shards = corpus.join("shards");
    let mut paths = Vec::new();
    let rd =
        fs::read_dir(&shards).map_err(|e| LoadError::Io(format!("{}: {e}", shards.display())))?;
    for ent in rd {
        let ent = ent.map_err(|e| LoadError::Io(e.to_string()))?;
        let p = ent.path();
        if p.extension().and_then(|s| s.to_str()) == Some("json") {
            paths.push(p);
        }
    }
    paths.sort();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn load_observation_from_value_bytes() {
        let raw = json!({
            "recordId": "vproto-000001",
            "sourceTag": "gov/demo#info",
            "command": "info",
            "argv": ["info", "demo.hwp", "--json"],
            "exitClass": 0,
            "stdoutPresent": true,
            "envelope": {
                "schemaVersion": "1.0",
                "source": "demo.hwp",
                "format": "hwp5",
                "pageCount": 1
            }
        });
        let bytes = serde_json::to_vec(&raw).unwrap();
        let obs = load_observation_bytes(&bytes).unwrap();
        assert_eq!(obs.command.as_str(), "info");
        assert_eq!(obs.exit_class.code(), 0);
    }
}
