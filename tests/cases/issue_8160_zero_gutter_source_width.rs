//! #8160: sample16 전용 회귀. 이전 등록: tests/cases/issue_5696_hwp3_lineseg_subtracts_binding_margin.rs
use rhwp::document_core::DocumentCore;
use std::{collections::BTreeMap, path::Path};

/// 사다리에 실린 `segment_width` 의 빈도표. 미설정 센티널(`i32::MAX / 2`)은 뺀다.
fn segment_width_histogram(rel: &str) -> BTreeMap<i32, usize> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"));
    let mut hist = BTreeMap::new();
    for section in &core.document().sections {
        for para in &section.paragraphs {
            for seg in &para.line_segs {
                if seg.segment_width <= 0 || seg.segment_width >= i32::MAX / 2 {
                    continue;
                }
                *hist.entry(seg.segment_width).or_insert(0) += 1;
            }
        }
    }
    hist
}

fn dominant_width(hist: &BTreeMap<i32, usize>) -> i32 {
    *hist
        .iter()
        .max_by_key(|(_, n)| **n)
        .expect("segment_width 가 하나도 없다")
        .0
}

/// 제본 여백이 **0** 인 문서는 이 수정이 건드리지 않는다.
///
/// `hwp3-sample16.hwp` 의 본문 문단 최다 폭은 수정 전후 모두 `46024` 다. 이 문서는
/// 한컴 변환본(`51024`)과 여전히 다른데, 그 불일치는 제본 축이 아니라 어울림 구역·
/// 들여쓰기 축이라 이 이슈와 분리된다(#5696 본문의 `hwp3-sample16` 52.8%).
///
/// 이 시험이 깨지면 제본 차감이 제본 여백 0 인 문서까지 좁힌 것이다.
#[test]
fn issue_8160_sample16_dominant_width_is_untouched_by_the_gutter_subtraction() {
    /// 수정 전후 동일한 실측값.
    const SAMPLE16_DOMINANT_HU: i32 = 46024;
    let ours = dominant_width(&segment_width_histogram("samples/hwp3-sample16.hwp"));
    assert_eq!(
        ours, SAMPLE16_DOMINANT_HU,
        "제본 여백 0 인 문서의 최다 폭은 그대로여야 한다: {ours}"
    );
}
