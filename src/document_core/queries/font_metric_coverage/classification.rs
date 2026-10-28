//! 실제 문자 측정 결과만 분류하고 미등록·모순된 출처는 오류로 처리한다.

use super::{coverage_error, HwpError};
use crate::renderer::layout::CharWidthDecision;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum CoverageClassification {
    Category(&'static str),
    NotApplicable,
}

pub(super) fn classify_decision(
    decision: &CharWidthDecision<'_>,
    relation_type: Option<&str>,
    relation_evidence_status: Option<&str>,
) -> Result<CoverageClassification, HwpError> {
    let metric_entry = decision.metric.map(|metric| metric.entry_index);
    match decision.width_source {
        "metricMiss" | "metricCharacterMiss" => Err(coverage_error(format!(
            "internal-only widthSource reached aggregate: {}",
            decision.width_source
        ))),
        "clusterContinuation"
        | "figureSpace"
        | "literalFigureSpace"
        | "hwpPuaFiller"
        | "inlineObjectPlaceholder"
        | "tabAdvance" => {
            if decision.character_match != "notApplicable" || metric_entry.is_some() {
                return Err(coverage_error(format!(
                    "non-applicable widthSource has metric state: {}",
                    decision.width_source
                )));
            }
            Ok(CoverageClassification::NotApplicable)
        }
        "measuredOriginalGlyph"
        | "kopubTable"
        | "metricHalfwidthPunctuationOverlay"
        | "metricNarrowPunctuationOverlay"
        | "metricSpaceOverlay" => {
            if decision.character_match != "hit" {
                return Err(coverage_error(format!(
                    "measured overlay must be a character hit: {}",
                    decision.width_source
                )));
            }
            Ok(CoverageClassification::Category("measured-overlay"))
        }
        "embeddedMetric"
        | "metricHalfSpace"
        | "canonicalHangulMetric"
        | "declaredGlyphFallbackMetric" => {
            if decision.character_match != "hit" || metric_entry.is_none() {
                return Err(coverage_error(format!(
                    "metric widthSource requires a metric character hit: {}",
                    decision.width_source
                )));
            }
            match (relation_type, relation_evidence_status) {
                (Some("identity-alias"), Some("verified-by-bytes")) => {
                    Ok(CoverageClassification::Category("identity-alias-hit"))
                }
                (Some("identity-alias"), _) => Err(coverage_error(
                    "identity-alias requires verified-by-bytes evidence",
                )),
                (Some("metric-surrogate"), _) => {
                    Ok(CoverageClassification::Category("metric-surrogate"))
                }
                _ => Ok(CoverageClassification::Category("exact-hit")),
            }
        }
        "heuristicFullwidth" | "heuristicHalfwidth" | "heuristicNarrow" => {
            if metric_entry.is_some() {
                if decision.character_match != "miss" {
                    return Err(coverage_error(format!(
                        "metric fallback requires characterMatch miss: {}",
                        decision.width_source
                    )));
                }
                Ok(CoverageClassification::Category("char-miss"))
            } else if decision.character_match == "notApplicable" {
                Ok(CoverageClassification::Category("face-miss"))
            } else {
                Err(coverage_error(format!(
                    "face miss has contradictory metric state: {}",
                    decision.width_source
                )))
            }
        }
        "areaDotFallback" => {
            if metric_entry.is_some() || decision.character_match != "notApplicable" {
                return Err(coverage_error(
                    "areaDotFallback has contradictory metric state",
                ));
            }
            Ok(CoverageClassification::Category("heuristic"))
        }
        other => Err(coverage_error(format!("unclassified widthSource: {other}"))),
    }
}
