//! [#5696] HWP3 사다리 합성이 **제본 여백**을 뺀다 — 렌더 쪽 본문 상자와 같은 폭.
//!
//! 두 경로가 같은 문단의 폭을 다르게 냈다.
//!
//! | 경로 | 본문 왼끝 |
//! |---|---|
//! | `src/model/page.rs` `Rect::page_areas`(렌더) | `margin_left + margin_gutter` |
//! | `src/parser/hwp3/mod.rs`(사다리 합성) | `margin_left` — **제본 여백 누락** |
//!
//! `samples/hwp3-sample19.hwp` 는 제본 여백이 `8.0mm`(= `2268HU`) 라 두 경로가 정확히
//! 그만큼 갈렸다.
//!
//! ```text
//! 합성 sw          39688   (= 용지 59528 − 좌 9921 − 우 9921, 제본 미차감)
//! 렌더 본문 상자   37420   (= 39688 − 2268)
//! 한컴 변환본 sw   37420   ← 오라클
//! 비율 1.0606             ← 이슈가 보고한 1.06 · 119관측 100%
//! ```
//!
//! **오라클은 저장소 안에 있다** — 같은 문서의 한컴 변환본
//! `samples/hwp3-sample19-hwp5.hwp` 의 저장 `segment_width` 다. 이슈가 코퍼스 실측으로
//! "한컴이 낸 값과는 맞고 우리 HWP3 파서가 낸 값과는 안 맞는다"고 적은 그대로다.
//!
//! 나머지 문서는 무영향이다 — 제본 여백이 0 이면 `body_left_hu` 가 바뀌지 않는다.
//! `hwp3-sample16.hwp` 의 본문 문단 최다 폭은 수정 전후 모두 `46024` 로 같다. 그 문서는
//! 한컴 변환본(`51024`)과 여전히 다른데, 그 불일치는 제본 축이 **아니라** 어울림 구역·
//! 들여쓰기 축이라 이 이슈와 분리된다(본문의 `hwp3-sample16` 52.8%).
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::document_core::DocumentCore;

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

/// 제본 여백이 있는 HWP3 문서의 합성 폭이 한컴 변환본과 같다.
///
/// 종전에는 `39688`(제본 미차감)이라 상자보다 `1.06` 배 넓었다.
#[test]
fn sample19_synthesized_width_matches_hancom_conversion() {
    let ours = segment_width_histogram("samples/hwp3-sample19.hwp");
    let hancom = segment_width_histogram("samples/hwp3-sample19-hwp5.hwp");
    let ours_w = dominant_width(&ours);
    let hancom_w = dominant_width(&hancom);
    assert_eq!(
        ours_w, hancom_w,
        "HWP3 합성 폭이 한컴 변환본과 같아야 한다: ours={ours_w} hancom={hancom_w} \
         (종전 39688 = 제본 여백 2268HU 미차감)"
    );
    assert!(
        ours.keys().all(|w| *w <= hancom_w),
        "합성 폭이 한컴 최대 폭을 넘는 행이 없어야 한다: {:?}",
        ours.keys().filter(|w| **w > hancom_w).collect::<Vec<_>>()
    );
}

/// 제본 여백이 **0** 인 문서는 이 수정이 건드리지 않는다.
///
/// `hwp3-sample16.hwp` 의 본문 문단 최다 폭은 수정 전후 모두 `46024` 다. 이 문서는
/// 한컴 변환본(`51024`)과 여전히 다른데, 그 불일치는 제본 축이 아니라 어울림 구역·
/// 들여쓰기 축이라 이 이슈와 분리된다(#5696 본문의 `hwp3-sample16` 52.8%).
///
/// 이 시험이 깨지면 제본 차감이 제본 여백 0 인 문서까지 좁힌 것이다.
#[test]
fn sample16_dominant_width_is_untouched_by_the_gutter_subtraction() {
    /// 수정 전후 동일한 실측값.
    const SAMPLE16_DOMINANT_HU: i32 = 46024;
    let ours = dominant_width(&segment_width_histogram("samples/hwp3-sample16.hwp"));
    assert_eq!(
        ours, SAMPLE16_DOMINANT_HU,
        "제본 여백 0 인 문서의 최다 폭은 그대로여야 한다: {ours}"
    );
}
