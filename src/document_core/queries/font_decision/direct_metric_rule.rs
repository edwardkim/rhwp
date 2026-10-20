//! 직접 등록된 메트릭 규칙의 실제 ID와 선택 조건을 추적에 보존한다.

use crate::error::HwpError;
use crate::renderer::font_decision::{candidate_id, ProvenanceDecision};
use crate::renderer::font_rule_layout_metric_projection::find_font_rule_layout_metric;

pub(super) fn provenance(
    source: &str,
    target: &str,
    generated_rule_id: &str,
    identity: &serde_json::Value,
) -> Result<Option<ProvenanceDecision>, HwpError> {
    let Some(rule) = find_font_rule_layout_metric(source).filter(|rule| rule.mode == "direct")
    else {
        return Ok(None);
    };
    if rule.rule_id != generated_rule_id
        || rule.source_face != Some(source)
        || rule.target_face_or_policy != target
        || rule.source_boundary_id != "rust-metric.metric-alias"
        || rule.decision_plane != "layout-metric"
        || rule.order.is_some()
        || rule.conditions.language_slot.is_some()
        || rule.conditions.alt_type.is_some()
        || rule.conditions.availability.is_some()
        || rule.conditions.profile.is_some()
    {
        return Err(HwpError::RenderError(format!(
            "font trace provenance: direct metric selection mismatch for {source}"
        )));
    }
    // 선택 튜플의 결정적 식별자는 유지하되, W1에 이 규칙의 근거가 있다고
    // 주장하지 않는다. ID와 관계는 실제 활성 레지스트리 투영에서 가져온다.
    let candidate_id = candidate_id(identity.clone())
        .map_err(|error| HwpError::RenderError(format!("font trace provenance: {error}")))?;
    Ok(Some(ProvenanceDecision {
        candidate_id,
        rule_id: Some(rule.rule_id.into()),
        evidence_anchor: format!(
            "assets/font-rules/font_rule_registry_v2.json#{}",
            rule.rule_id
        ),
        source_owner: "rust-metric".into(),
        relation_type: Some(rule.relation_type.into()),
        evidence_status: Some("unknown".into()),
        known_limitations: vec![
            "Direct registry rule has no W1 candidate evidence link.".into(),
            "Metric mapping does not prove glyph-outline identity.".into(),
        ],
        reason: None,
    }))
}
