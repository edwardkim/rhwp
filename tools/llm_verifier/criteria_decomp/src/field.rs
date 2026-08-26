//! 지식지도 §2 에 있는 기존 rhwp 봉투 필드만 허용한다.

/// 원자 기준이 붙을 수 있는 기존 봉투 키. 점 경로는 중첩 객체다.
pub const ALLOWED_ENVELOPE_FIELDS: &[&str] = &[
    "schemaVersion",
    "source",
    "untrustedContent",
    "untrustedFields",
    "format",
    "pageCount",
    "paraCount",
    "sectionCount",
    "paragraphCount",
    "charCount",
    "wordCount",
    "warnings",
    "encrypted",
    "identical",
    "diffCount",
    "status",
    "regression",
    "passCount",
    "failCount",
    "verdict",
    "overPages",
    "pageCountMismatch",
    "strict",
    "overflowCount",
    "offCanvasCount",
    "overlapCount",
    "textOverlapCount",
    "emptyPageCount",
    "hasSignal",
    "clean",
    "hiddenCharCount",
    "signalCount",
    "findingCount",
    "highestConfidence",
    "highestSeverity",
    "truncated",
    "fieldCount",
    "filledCount",
    "notFound",
    "ambiguous",
    "confusable",
    "dryRun",
    "verify",
    "verify.identical",
    "verify.diffCount",
    "verifyPages",
    "replacedCount",
    "redactedCount",
    "removedCount",
    "wasDistribution",
    "changedPages",
    "output",
    "outputFormat",
    "inPlace",
    "ok",
    "maxDisp",
    "worstPage",
    "lossCount",
    "renderedCount",
    "reproduced",
    "valid",
    "signatureOk",
    "capsuleShaMatches",
    "keyKnown",
    "matches",
    "items",
    "overflow",
    "keepPreview",
    "scannedChars",
];

/// LLM 이 지어내기 쉬운, 봉투에 없는 키.
pub const INVENTED_FIELDS: &[&str] = &[
    "holisticScore",
    "overall",
    "quality",
    "stars",
    "grade",
    "confidence",
    "vibe",
    "humanPage",
    "pdfPage",
    "bestOfN",
    "processReward",
    "stepReward",
    "rank",
    "winner",
    "llmScore",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnvelopeField(&'static str);

impl EnvelopeField {
    pub fn as_str(self) -> &'static str {
        self.0
    }
}

pub fn is_allowed_envelope_field(name: &str) -> bool {
    ALLOWED_ENVELOPE_FIELDS.contains(&name)
}

pub fn parse_envelope_field(name: &str) -> Option<EnvelopeField> {
    ALLOWED_ENVELOPE_FIELDS
        .iter()
        .copied()
        .find(|k| *k == name)
        .map(EnvelopeField)
}

pub fn is_invented_field(name: &str) -> bool {
    INVENTED_FIELDS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_set_has_no_invented_overlap() {
        for k in INVENTED_FIELDS {
            assert!(!is_allowed_envelope_field(k), "{k} leaked into allowed");
        }
    }

    #[test]
    fn dotted_verify_paths_are_first_class() {
        assert!(is_allowed_envelope_field("verify.identical"));
        assert!(is_allowed_envelope_field("verify.diffCount"));
        assert!(!is_allowed_envelope_field("verify.quality"));
    }
}
