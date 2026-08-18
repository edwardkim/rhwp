//! 코퍼스 샤드 로더. 행 유일성(rowId · claimText)을 강제한다.

use crate::row::ClaimBindRow;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
    pub axis: String,
    pub record_count: u64,
    pub shard_count: u64,
    pub pass_count: u64,
    pub fail_count: u64,
    #[serde(default)]
    pub uniqueness: String,
    #[serde(default)]
    pub required_fields: Vec<String>,
    #[serde(default)]
    pub shards: Vec<ShardMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardMeta {
    pub path: String,
    pub count: u64,
    pub pass_count: u64,
    pub fail_count: u64,
}

pub fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn corpus_dir() -> PathBuf {
    crate_dir().join("fixtures").join("corpus")
}

pub fn load_manifest(dir: &Path) -> Result<CorpusManifest, LoadError> {
    let path = dir.join("manifest.json");
    let bytes = fs::read(&path).map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    serde_json::from_slice(&bytes).map_err(|e| LoadError::Json(format!("manifest: {e}")))
}

pub fn load_shard_rows(path: &Path) -> Result<Vec<ClaimBindRow>, LoadError> {
    let text =
        fs::read_to_string(path).map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    let mut rows = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let row: ClaimBindRow = serde_json::from_str(line)
            .map_err(|e| LoadError::Json(format!("{}:{}: {e}", path.display(), i + 1)))?;
        rows.push(row);
    }
    Ok(rows)
}

pub fn load_shards(dir: &Path) -> Result<Vec<ClaimBindRow>, LoadError> {
    let manifest = load_manifest(dir)?;
    let mut rows = Vec::with_capacity(manifest.record_count as usize);
    let mut ids = HashSet::new();
    let mut texts = HashSet::new();
    for shard in &manifest.shards {
        let path = dir.join(&shard.path);
        let shard_rows = load_shard_rows(&path)?;
        if shard_rows.len() as u64 != shard.count {
            return Err(LoadError::Json(format!(
                "{} count {} != manifest {}",
                shard.path,
                shard_rows.len(),
                shard.count
            )));
        }
        for row in shard_rows {
            if !ids.insert(row.row_id.clone()) {
                return Err(LoadError::Duplicate(format!("rowId {}", row.row_id)));
            }
            if !row.claim_text.trim().is_empty() && !texts.insert(row.claim_text.clone()) {
                return Err(LoadError::Duplicate(format!("claimText {}", row.row_id)));
            }
            rows.push(row);
        }
    }
    if rows.len() as u64 != manifest.record_count {
        return Err(LoadError::Json(format!(
            "loaded {} != manifest.recordCount {}",
            rows.len(),
            manifest.record_count
        )));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_dir_points_at_fixtures() {
        let dir = corpus_dir();
        assert!(dir.ends_with(Path::new("fixtures").join("corpus")));
    }
}
