//! 주장-좌표 결속. 미결속 주장은 실패다.

use crate::claim::NaturalClaim;
use crate::coords::DocumentCoords;
use crate::envelope::SearchExtractEnvelope;
use crate::row::ClaimBindRow;
use crate::verdict::{BindDecision, FailKind};
use serde_json::Value;

/// 자연어 주장 + (선택) 부착 좌표. 봉투 없이 좌표 완전성만 본다.
pub fn bind_claim(claim: &NaturalClaim) -> BindDecision {
    classify(
        &claim.id,
        &claim.claim_text,
        claim.locator.as_ref(),
        &claim.invented_keys,
        claim.envelope_kind.map(|k| k.as_str()),
        None,
    )
}

/// 주장 좌표가 기존 search/extract-data 봉투 매치에 실제로 있는지 본다.
pub fn bind_claim_to_envelope(claim: &NaturalClaim, envelope: &Value) -> BindDecision {
    let parsed = match SearchExtractEnvelope::from_value(envelope) {
        Ok(env) => env,
        Err(_) => {
            let mut d = BindDecision::fail(
                &claim.id,
                &claim.claim_text,
                claim.locator.as_ref(),
                FailKind::UnknownEnvelopeKind,
            );
            d.invented_keys = claim.invented_keys.clone();
            return d;
        }
    };
    classify(
        &claim.id,
        &claim.claim_text,
        claim.locator.as_ref(),
        &claim.invented_keys,
        Some(parsed.kind.as_str()),
        Some(&parsed),
    )
}

/// 코퍼스 행을 다시 판정한다. 픽스처 `verdict` 를 믿지 않는다.
pub fn bind_row(row: &ClaimBindRow) -> BindDecision {
    let locator = row.locator();
    classify(
        &row.row_id,
        &row.claim_text,
        locator.as_ref(),
        &row.invented_keys,
        Some(row.envelope_kind.as_str()),
        None,
    )
}

fn classify(
    id: &str,
    text: &str,
    locator: Option<&DocumentCoords>,
    invented: &[String],
    envelope_kind: Option<&str>,
    envelope: Option<&SearchExtractEnvelope>,
) -> BindDecision {
    if text.trim().is_empty() {
        let mut d = BindDecision::fail(id, text, locator, FailKind::EmptyClaim);
        d.invented_keys = invented.to_vec();
        return d;
    }
    if !invented.is_empty() {
        let mut d = BindDecision::fail(id, text, locator, FailKind::InventedKey);
        d.invented_keys = invented.to_vec();
        return d;
    }
    if let Some(kind) = envelope_kind {
        if crate::envelope::EnvelopeKind::parse(kind).is_none() {
            return BindDecision::fail(id, text, locator, FailKind::UnknownEnvelopeKind);
        }
    }
    match locator {
        None => BindDecision::fail(id, text, None, FailKind::Unbound),
        Some(coords) if coords.is_empty() => {
            BindDecision::fail(id, text, Some(coords), FailKind::Unbound)
        }
        Some(coords) if !coords.coords_present() => {
            BindDecision::fail(id, text, Some(coords), FailKind::IncompleteCoords)
        }
        Some(coords) => {
            if let Some(env) = envelope {
                if env.hit_with_required(coords).is_none() {
                    return BindDecision::fail(id, text, Some(coords), FailKind::EnvelopeMismatch);
                }
            }
            BindDecision::pass(id, text, coords)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords::DocumentCoords;
    use crate::envelope::EnvelopeKind;
    use crate::verdict::{FailKind, Verdict};
    use serde_json::json;

    fn full_coords() -> DocumentCoords {
        DocumentCoords {
            section: Some(0),
            paragraph: Some(12),
            page: Some(2),
            char_offset: Some(48),
            length: Some(18),
            ..DocumentCoords::default()
        }
    }

    fn claim(text: &str, locator: Option<DocumentCoords>) -> NaturalClaim {
        NaturalClaim {
            id: "CB-T".into(),
            claim_text: text.into(),
            locator,
            envelope_kind: Some(EnvelopeKind::Search),
            quote: None,
            file: None,
            invented_keys: Vec::new(),
        }
    }

    #[test]
    fn bound_claim_passes() {
        let d = bind_claim(&claim(
            "과업지시서 제3조는 표준 API 연계를 필수기능으로 정한다.",
            Some(full_coords()),
        ));
        assert!(d.is_pass());
        assert!(d.coords_present);
        assert!(d.field_set.contains(&"section".into()));
        assert!(d.field_set.contains(&"paragraph".into()));
        assert!(d.field_set.contains(&"page".into()));
        assert!(d.field_set.contains(&"charOffset".into()));
    }

    #[test]
    fn unbound_claim_fails() {
        let d = bind_claim(&claim("시장이 성장할 것이다.", None));
        assert_eq!(d.verdict, Verdict::Fail);
        assert_eq!(d.fail_kind, Some(FailKind::Unbound));
        assert!(!d.coords_present);
        assert!(d.field_set.is_empty());
    }

    #[test]
    fn missing_page_fails() {
        let mut c = full_coords();
        c.page = None;
        let d = bind_claim(&claim("쪽 좌표 없이 과업을 인용한다.", Some(c)));
        assert_eq!(d.fail_kind, Some(FailKind::IncompleteCoords));
        assert!(d.missing_fields.iter().any(|k| k == "page"));
    }

    #[test]
    fn invented_key_fails_even_if_four_coords_exist() {
        let mut c = claim(
            "사람이 읽기 쉬운 1쪽이라고 고쳐 적었다.",
            Some(full_coords()),
        );
        c.invented_keys = vec!["humanPage".into()];
        let d = bind_claim(&c);
        assert_eq!(d.fail_kind, Some(FailKind::InventedKey));
        assert!(!d.is_pass());
    }

    #[test]
    fn empty_claim_fails() {
        let d = bind_claim(&claim("   ", Some(full_coords())));
        assert_eq!(d.fail_kind, Some(FailKind::EmptyClaim));
    }

    #[test]
    fn envelope_mismatch_fails() {
        let env = json!({
            "matches": [{
                "text": "다른 문단",
                "section": 0,
                "paragraph": 1,
                "page": 0,
                "charOffset": 0
            }]
        });
        let d = bind_claim_to_envelope(
            &claim(
                "과업지시서 제3조는 표준 API 연계를 필수기능으로 정한다.",
                Some(full_coords()),
            ),
            &env,
        );
        assert_eq!(d.fail_kind, Some(FailKind::EnvelopeMismatch));
    }

    #[test]
    fn envelope_hit_with_same_required_coords_passes() {
        let env = json!({
            "matches": [{
                "text": "표준 API 연계를 필수기능으로 정한다",
                "section": 0,
                "paragraph": 12,
                "page": 2,
                "charOffset": 48,
                "length": 18
            }]
        });
        let d = bind_claim_to_envelope(
            &claim(
                "과업지시서 제3조는 표준 API 연계를 필수기능으로 정한다.",
                Some(full_coords()),
            ),
            &env,
        );
        assert!(d.is_pass(), "{d:?}");
    }

    #[test]
    fn extract_data_hit_binds() {
        let env = json!({
            "items": [{
                "kind": "amount",
                "raw": "318,000,000원",
                "normalized": 318000000,
                "currency": "KRW",
                "section": 0,
                "paragraph": 7,
                "page": 0,
                "charOffset": 55,
                "length": 11
            }]
        });
        let mut loc = full_coords();
        loc.paragraph = Some(7);
        loc.page = Some(0);
        loc.char_offset = Some(55);
        loc.length = Some(11);
        let mut c = claim("계약 대금은 318,000,000원이다.", Some(loc));
        c.envelope_kind = Some(EnvelopeKind::ExtractData);
        let d = bind_claim_to_envelope(&c, &env);
        assert!(d.is_pass(), "{d:?}");
    }
}
