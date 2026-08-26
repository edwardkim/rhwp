//! 코퍼스 TSV 샤드 로더.

use crate::gate::{parse_reproduced, reproduced_token};
use crate::reason::Reason;
use crate::row::GateRow;
use crate::schema::{CLAIM_ID, KIND, PROTOCOL_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const TSV_COLUMNS: [&str; 7] = [
    "id",
    "attest_version",
    "verify_version",
    "reproduced",
    "accepted",
    "reason",
    "family",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Io(String),
    Json(String),
    Tsv(String),
    Duplicate(String),
    Shape(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(s) | Self::Json(s) | Self::Tsv(s) | Self::Duplicate(s) | Self::Shape(s) => {
                f.write_str(s)
            }
        }
    }
}

impl std::error::Error for LoadError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorpusManifest {
    pub schema_version: String,
    pub claim: String,
    pub kind: String,
    pub generated_by: String,
    pub record_count: u64,
    pub shard_count: u64,
    pub uniqueness: String,
    pub tuple_fields: Vec<String>,
    pub accepted_count: u64,
    pub rejected_count: u64,
    pub stale_tool_count: u64,
    pub min_line_floor: u64,
    #[serde(default)]
    pub reason_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub family_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub shards: Vec<ShardMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardMeta {
    pub path: String,
    pub count: u64,
}

pub fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus")
}

pub fn load_manifest(dir: &Path) -> Result<CorpusManifest, LoadError> {
    let path = dir.join("manifest.json");
    let bytes = fs::read(&path).map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    serde_json::from_slice(&bytes).map_err(|e| LoadError::Json(format!("manifest: {e}")))
}

fn parse_accepted(raw: &str) -> Result<bool, LoadError> {
    match raw.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(LoadError::Tsv(format!("accepted {other}"))),
    }
}

pub fn parse_tsv_line(line: &str, line_no: usize) -> Result<GateRow, LoadError> {
    let cols: Vec<&str> = line.split('\t').collect();
    if cols.len() != TSV_COLUMNS.len() {
        return Err(LoadError::Tsv(format!(
            "line {line_no}: expected {} columns, got {}",
            TSV_COLUMNS.len(),
            cols.len()
        )));
    }
    let reproduced = parse_reproduced(cols[3]).map_err(LoadError::Tsv)?;
    let accepted = parse_accepted(cols[4])?;
    let reason = Reason::parse(cols[5])
        .ok_or_else(|| LoadError::Tsv(format!("line {line_no}: reason {}", cols[5])))?;
    let uniqueness_key = format!(
        "{}|{}|{}|{}",
        cols[1],
        cols[2],
        reproduced_token(reproduced),
        accepted
    );
    Ok(GateRow {
        schema_version: PROTOCOL_SCHEMA_VERSION.to_string(),
        claim: CLAIM_ID.to_string(),
        kind: KIND.to_string(),
        record_id: cols[0].to_string(),
        uniqueness_key,
        attest_version: cols[1].to_string(),
        verify_version: cols[2].to_string(),
        reproduced,
        accepted,
        reason,
        family: cols[6].to_string(),
    })
}

pub fn load_shard_path(path: &Path) -> Result<Vec<GateRow>, LoadError> {
    let text =
        fs::read_to_string(path).map_err(|e| LoadError::Io(format!("{}: {e}", path.display())))?;
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| LoadError::Tsv(format!("{} empty", path.display())))?;
    let expect = TSV_COLUMNS.join("\t");
    if header != expect {
        return Err(LoadError::Tsv(format!(
            "{} header {header} != {expect}",
            path.display()
        )));
    }
    let mut rows = Vec::new();
    for (i, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        rows.push(parse_tsv_line(line, i + 2)?);
    }
    Ok(rows)
}

pub fn load_shards(dir: &Path) -> Result<Vec<GateRow>, LoadError> {
    let shards_dir = dir.join("shards");
    let mut paths: Vec<PathBuf> = fs::read_dir(&shards_dir)
        .map_err(|e| LoadError::Io(format!("{}: {e}", shards_dir.display())))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("tsv"))
        .collect();
    paths.sort();
    let mut rows = Vec::new();
    let mut seen_key = HashSet::new();
    let mut seen_id = HashSet::new();
    for path in paths {
        for row in load_shard_path(&path)? {
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

    #[test]
    fn parse_one_stale_line() {
        let line = "tvg-1\t0.8.3\t0.8.4\ttrue\tfalse\tSTALE_TOOL\tpatch_drift";
        let row = parse_tsv_line(line, 2).expect("line");
        assert_eq!(row.attest_version, "0.8.3");
        assert_eq!(row.verify_version, "0.8.4");
        assert_eq!(row.reproduced, Some(true));
        assert!(!row.accepted);
        assert_eq!(row.reason, Reason::StaleTool);
        row.validate_shape().expect("shape");
    }
}
