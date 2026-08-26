//! 코퍼스 샤드 로더.

use crate::row::RepeatRow;
use crate::schema::PROTOCOL_SCHEMA_VERSION;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Io(String),
    Json(String),
    Duplicate(String),
    Shape(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(s) | Self::Json(s) | Self::Duplicate(s) | Self::Shape(s) => f.write_str(s),
        }
    }
}

impl std::error::Error for LoadError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorpusManifest {
    pub schema_version: String,
    pub claim: String,
    pub generated_by: String,
    pub record_count: u64,
    pub shard_count: u64,
    pub uniqueness: String,
    #[serde(default)]
    pub check_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub command_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub k_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub shards: Vec<ShardMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardMeta {
    pub path: String,
    pub count: u64,
    #[serde(default)]
    pub checks: Vec<String>,
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorpusShard {
    pub schema_version: String,
    pub claim: String,
    pub shard_id: String,
    pub records: Vec<RepeatRow>,
}

pub fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus")
}

pub fn load_manifest(dir: &Path) -> Result<CorpusManifest, LoadError> {
    let path = dir.join("manifest.json");
    let bytes = fs::read(&path).map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    serde_json::from_slice(&bytes).map_err(|e| LoadError::Json(format!("manifest: {e}")))
}

pub fn load_shard_path(path: &Path) -> Result<CorpusShard, LoadError> {
    let bytes = fs::read(path).map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    let shard: CorpusShard =
        serde_json::from_slice(&bytes).map_err(|e| LoadError::Json(format!("shard: {e}")))?;
    if shard.schema_version != PROTOCOL_SCHEMA_VERSION {
        return Err(LoadError::Shape(format!(
            "{} schema {}",
            path.display(),
            shard.schema_version
        )));
    }
    Ok(shard)
}

pub fn load_shards(dir: &Path) -> Result<Vec<RepeatRow>, LoadError> {
    let shards_dir = dir.join("shards");
    let mut paths: Vec<PathBuf> = fs::read_dir(&shards_dir)
        .map_err(|e| LoadError::Io(format!("{}: {e}", shards_dir.display())))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    paths.sort();
    let mut rows = Vec::new();
    let mut seen_key = HashSet::new();
    let mut seen_id = HashSet::new();
    for path in paths {
        let shard = load_shard_path(&path)?;
        for row in shard.records {
            row.validate_shape()
                .map_err(|e| LoadError::Shape(format!("{}: {e}", row.record_id)))?;
            if !seen_id.insert(row.record_id.clone()) {
                return Err(LoadError::Duplicate(format!("recordId {}", row.record_id)));
            }
            let key = row.uniqueness().as_string();
            if !seen_key.insert(key.clone()) {
                return Err(LoadError::Duplicate(format!("uniqueness {key}")));
            }
            rows.push(row);
        }
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_dir_ends_with_corpus() {
        assert!(corpus_dir().ends_with("corpus"));
    }
}
