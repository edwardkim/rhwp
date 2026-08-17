//! Issue #4961: layout font decision trace의 직렬화와 결정적 해시 계약.

use serde::Serialize;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

const EVIDENCE_PREFIX: &str = "mydocs/tech/investigations/issue-4939/font_rule_candidates.json#";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TraceEnvelope {
    pub(crate) schema_version: u8,
    pub(crate) status: String,
    pub(crate) scope: TraceScope,
    pub(crate) counts: TraceCounts,
    pub(crate) records: Vec<TraceRecord>,
    pub(crate) backend_summary: BackendSummary,
    pub(crate) reasons: Vec<TraceReason>,
    pub(crate) layout_hash: TraceHash,
    pub(crate) normalized_hash: TraceHash,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TraceScope {
    pub(crate) page_index: u32,
    pub(crate) requested_limits: TraceLimits,
    pub(crate) applied_limits: TraceLimits,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TraceLimits {
    pub(crate) max_characters: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TraceCounts {
    pub(crate) runs_seen: usize,
    pub(crate) characters_seen: usize,
    pub(crate) records_emitted: usize,
    pub(crate) records_omitted: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TraceReason {
    pub(crate) code: String,
    pub(crate) detail: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct TraceHash {
    pub(crate) algorithm: &'static str,
    pub(crate) value: Option<String>,
}

impl TraceHash {
    pub(crate) fn pending() -> Self {
        Self {
            algorithm: "sha256",
            value: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TraceRecord {
    pub(crate) record_id: String,
    pub(crate) source: SourceDecision,
    pub(crate) document: DocumentDecision,
    pub(crate) layout_name: LayoutNameDecision,
    pub(crate) layout_metric: LayoutMetricDecision,
    pub(crate) paint: PaintDecision,
    pub(crate) provenance: Vec<ProvenanceDecision>,
    pub(crate) oracle: OracleDecision,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourceDecision {
    pub(crate) status: String,
    pub(crate) section_index: Option<usize>,
    pub(crate) paragraph_index: Option<usize>,
    pub(crate) nested_path: Vec<usize>,
    pub(crate) run_index: Option<usize>,
    pub(crate) char_offset: Option<usize>,
    pub(crate) character: String,
    pub(crate) code_point: u32,
    pub(crate) char_shape_id: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocumentDecision {
    pub(crate) language_slot: Option<usize>,
    pub(crate) inherited_language_slot: Option<usize>,
    pub(crate) face: Option<String>,
    pub(crate) alt_type: Option<u8>,
    pub(crate) embedded: Option<bool>,
    pub(crate) subst_font: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DecisionStep {
    pub(crate) kind: String,
    pub(crate) input: Option<String>,
    pub(crate) output: Option<String>,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LayoutNameDecision {
    pub(crate) requested_face: Option<String>,
    pub(crate) normalized_face: Option<String>,
    pub(crate) css_family_chain: Vec<String>,
    pub(crate) steps: Vec<DecisionStep>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LayoutMetricDecision {
    pub(crate) requested_face: Option<String>,
    pub(crate) alias_resolved_face: Option<String>,
    pub(crate) match_kind: String,
    pub(crate) metric_entry: Option<usize>,
    pub(crate) character_match: String,
    pub(crate) width_source: String,
    pub(crate) base_advance_hwpunit: Option<i32>,
    pub(crate) transforms: Vec<DecisionStep>,
    pub(crate) final_advance_hwpunit: Option<i32>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PaintDecision {
    pub(crate) native: BackendDecision,
    pub(crate) canvas2d: BackendDecision,
    pub(crate) canvaskit: BackendDecision,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BackendDecision {
    pub(crate) status: String,
    pub(crate) certainty: String,
    pub(crate) requested: Option<String>,
    pub(crate) candidates: Vec<String>,
    pub(crate) resolved: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) capabilities: Vec<String>,
    pub(crate) failures: Vec<String>,
}

impl BackendDecision {
    pub(crate) fn unsupported(requested: Option<String>, reason: &str) -> Self {
        Self {
            status: "unsupported".into(),
            certainty: "unsupported".into(),
            requested,
            candidates: Vec::new(),
            resolved: None,
            source: None,
            capabilities: Vec::new(),
            failures: vec![reason.into()],
        }
    }
}

impl PaintDecision {
    pub(crate) fn stage3(requested: Option<String>, native: Option<BackendDecision>) -> Self {
        Self {
            native: native.unwrap_or_else(|| {
                BackendDecision::unsupported(requested.clone(), "nativeSkiaFeatureUnavailable")
            }),
            canvas2d: BackendDecision::unsupported(requested.clone(), "studioSnapshotRequired"),
            canvaskit: BackendDecision::unsupported(requested, "studioSnapshotRequired"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProvenanceDecision {
    pub(crate) candidate_id: String,
    pub(crate) rule_id: Option<String>,
    pub(crate) evidence_anchor: String,
    pub(crate) source_owner: String,
    pub(crate) relation_type: Option<String>,
    pub(crate) evidence_status: Option<String>,
    pub(crate) known_limitations: Vec<String>,
    pub(crate) reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OracleDecision {
    pub(crate) status: String,
    pub(crate) profile_id: Option<String>,
    pub(crate) known_limitations: Vec<String>,
}

impl OracleDecision {
    pub(crate) fn not_provided() -> Self {
        Self {
            status: "notProvided".into(),
            profile_id: None,
            known_limitations: vec![
                "No oracle profile was supplied to this read-only query.".into()
            ],
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct BackendSummary {
    pub(crate) layout: BackendSummaryEntry,
    pub(crate) native: BackendSummaryEntry,
    pub(crate) canvas2d: BackendSummaryEntry,
    pub(crate) canvaskit: BackendSummaryEntry,
}

#[derive(Debug, Serialize)]
pub(crate) struct BackendSummaryEntry {
    pub(crate) status: String,
    pub(crate) reasons: Vec<String>,
}

impl BackendSummary {
    pub(crate) fn stage3(native_available: bool) -> Self {
        let studio = || BackendSummaryEntry {
            status: "unsupported".into(),
            reasons: vec!["studioSnapshotRequired".into()],
        };
        Self {
            layout: BackendSummaryEntry {
                status: "complete".into(),
                reasons: Vec::new(),
            },
            native: if native_available {
                BackendSummaryEntry {
                    status: "complete".into(),
                    reasons: Vec::new(),
                }
            } else {
                BackendSummaryEntry {
                    status: "unsupported".into(),
                    reasons: vec!["nativeSkiaFeatureUnavailable".into()],
                }
            },
            canvas2d: studio(),
            canvaskit: studio(),
        }
    }
}

fn canonicalize(value: Value, parent_key: Option<&str>, normalized_trace: bool) -> Value {
    match value {
        Value::Array(values) => {
            let mut values: Vec<Value> = values
                .into_iter()
                .map(|value| canonicalize(value, parent_key, normalized_trace))
                .collect();
            if matches!(
                parent_key,
                Some("capabilities" | "failures" | "knownLimitations")
            ) && values.iter().all(Value::is_string)
            {
                values.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
                values.dedup();
            }
            Value::Array(values)
        }
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            for key in keys {
                if normalized_trace
                    && matches!(
                        key.as_str(),
                        "layoutHash"
                            | "normalizedHash"
                            | "timestamp"
                            | "generatedAt"
                            | "elapsedMs"
                            | "durationMs"
                            | "stack"
                    )
                {
                    continue;
                }
                let child = map.get(&key).cloned().expect("키 목록은 map에서 생성됨");
                sorted.insert(
                    key.clone(),
                    canonicalize(child, Some(&key), normalized_trace),
                );
            }
            Value::Object(sorted)
        }
        scalar => scalar,
    }
}

fn sha256_canonical(value: Value, normalized_trace: bool) -> Result<String, serde_json::Error> {
    let canonical = canonicalize(value, None, normalized_trace);
    let mut bytes = serde_json::to_string_pretty(&canonical)?;
    bytes.push('\n');
    let digest = Sha256::digest(bytes.as_bytes());
    let mut hex = String::with_capacity(64);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("String formatting cannot fail");
    }
    Ok(hex)
}

pub(crate) fn finalize_hashes(trace: &mut TraceEnvelope) -> Result<(), serde_json::Error> {
    let value = serde_json::to_value(&*trace)?;
    let layout = json!({
        "schemaVersion": value["schemaVersion"].clone(),
        "scope": value["scope"].clone(),
        "counts": value["counts"].clone(),
        "records": value["records"].as_array().map(|records| records.iter().map(|record| json!({
            "recordId": record["recordId"].clone(),
            "source": record["source"].clone(),
            "document": record["document"].clone(),
            "layoutName": record["layoutName"].clone(),
            "layoutMetric": record["layoutMetric"].clone(),
            "provenance": record["provenance"].clone(),
        })).collect::<Vec<_>>()).unwrap_or_default(),
    });
    trace.layout_hash.value = Some(sha256_canonical(layout, true)?);
    let value = serde_json::to_value(&*trace)?;
    trace.normalized_hash.value = Some(sha256_canonical(value, true)?);
    Ok(())
}

pub(crate) fn candidate_id(identity: Value) -> Result<String, serde_json::Error> {
    Ok(format!(
        "candidate.{}",
        &sha256_canonical(identity, false)?[..20]
    ))
}

pub(crate) fn linked_provenance(
    identity: Value,
    source_owner: &str,
    relation_type: &str,
    evidence_status: &str,
    known_limitations: Vec<String>,
) -> Result<ProvenanceDecision, serde_json::Error> {
    let candidate_id = candidate_id(identity)?;
    let mut known_limitations = known_limitations;
    known_limitations.push(
        "W1 source digest predates the Stage 2 trace-only refactor; rerun the collector before promoting evidence status."
            .into(),
    );
    Ok(ProvenanceDecision {
        rule_id: Some(format!("rule.{source_owner}.{}", &candidate_id[10..])),
        evidence_anchor: format!("{EVIDENCE_PREFIX}{candidate_id}"),
        candidate_id,
        source_owner: source_owner.into(),
        relation_type: Some(relation_type.into()),
        evidence_status: Some(evidence_status.into()),
        known_limitations,
        reason: Some("ledgerSourceDrift".into()),
    })
}

pub(crate) fn unlinked_provenance(
    identity: Value,
    source_owner: &str,
) -> Result<ProvenanceDecision, serde_json::Error> {
    let candidate_id = candidate_id(identity)?;
    Ok(ProvenanceDecision {
        rule_id: None,
        evidence_anchor: format!("{EVIDENCE_PREFIX}{candidate_id}"),
        candidate_id,
        source_owner: source_owner.into(),
        relation_type: None,
        evidence_status: None,
        known_limitations: Vec::new(),
        reason: Some("ledgerRuleMissing".into()),
    })
}
