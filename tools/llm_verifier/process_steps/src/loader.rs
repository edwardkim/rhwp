//! 과정 추적 코퍼스 로더.

use crate::trace::{ProcessStep, UniquenessKey};
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
    pub step_kind_counts: HashMap<String, u64>,
    #[serde(default)]
    pub reward_pass_count: u64,
    #[serde(default)]
    pub reward_fail_count: u64,
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
    pub step_kinds: Vec<String>,
    #[serde(default)]
    pub reward_pass: u64,
    #[serde(default)]
    pub reward_fail: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorpusShard {
    pub schema_version: String,
    pub shard_id: String,
    pub records: Vec<ProcessStep>,
}

pub fn load_step_bytes(bytes: &[u8]) -> Result<ProcessStep, LoadError> {
    let mut step: ProcessStep =
        serde_json::from_slice(bytes).map_err(|e| LoadError::Json(format!("step: {e}")))?;
    step.refresh();
    Ok(step)
}

pub fn load_shard_bytes(bytes: &[u8]) -> Result<CorpusShard, LoadError> {
    let mut shard: CorpusShard =
        serde_json::from_slice(bytes).map_err(|e| LoadError::Json(format!("shard: {e}")))?;
    for rec in &mut shard.records {
        rec.refresh();
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

pub fn load_shards(paths: &[PathBuf]) -> Result<Vec<ProcessStep>, LoadError> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for path in paths {
        let shard = load_shard_path(path)?;
        for rec in shard.records {
            let key = UniquenessKey::from_step(&rec);
            if !seen.insert(key) {
                return Err(LoadError::Duplicate(format!(
                    "duplicate process step in {}: {}",
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
    fn load_step_from_bytes() {
        let raw = json!({
            "recordId": "vstep-000001",
            "episodeId": "ep-000001",
            "sourceTag": "gov/demo#fill-fields/s0",
            "stepIndex": 0,
            "stepKind": "fill-fields",
            "source": "demo.hwp",
            "argv": ["edit", "fill-fields", "demo.hwp", "--verify", "--json"],
            "editExitClass": 0,
            "checks": [{
                "check": "verify",
                "argv": ["verify", "demo.hwp", "--json"],
                "exitClass": 0,
                "pass": true,
                "failSignals": [],
                "envelope": {"verdict": "pass", "failCount": 0, "passCount": 1}
            }],
            "processReward": {
                "pass": true,
                "checkCount": 1,
                "passCount": 1,
                "failCount": 0,
                "failedChecks": [],
                "worstExitClass": 0,
                "consistent": true
            }
        });
        let step = load_step_bytes(&serde_json::to_vec(&raw).unwrap()).unwrap();
        assert_eq!(step.step_kind.as_str(), "fill-fields");
        assert_eq!(step.checks[0].fields.verdict.as_deref(), Some("pass"));
    }
}
