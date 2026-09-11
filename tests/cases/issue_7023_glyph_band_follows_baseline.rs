//! [Issue #7023] 글자겹침 검출이 em 상자를 줄 상자 **중앙**에 두는데 렌더러는
//! baseline 을 상자 아래쪽에 찍는다 — 부푼 줄 상자에서 띠가 잉크에서 멀어져
//! 진짜 겹침이 침묵한다.
//!
//! `glyph_band_bbox` 의 종전 주석은 중앙을 고른 이유로 "렌더 트리에 baseline
//! 필드가 없다" 를 적고, "방향이 한쪽(위양성 감소)이라 결함을 놓치는 쪽으로는
//! 틀리지 않는다" 고 덧붙였다. 둘 다 사실이 아니었다 — `TextRunNode::baseline`
//! 은 있고(SVG 가 그 값으로 글자를 찍는다), 중앙 가정은 줄 상자가 부풀수록 띠를
//! 잉크에서 `(h − em)/2` 만큼 떼어 놓아 **위음성을 만든다**.
//!
//! 이 시험이 잠그는 실물은 `1480000-201900042` 15쪽의 본문 ↔ 꼬리말 충돌이다.
//!
//! ```text
//!   본문   '□ 고시(안)에 대한 비용‧편익 분석 및 규제영향분석서 작성'
//!          baseline 1061.88   x  94.5 .. 515.3
//!   꼬리말 '- XI -'
//!          baseline 1064.69   x 374.9 .. 410.0
//! ```
//!
//! 세로로 2.8px 떨어져 있고 가로로는 꼬리말이 본문 줄 안에 통째로 들어간다 —
//! 본문이 쪽번호를 덮어 둘 다 읽기 어렵다(`#6920` 과 같은 결함 클래스).
//! 중앙 기준 띠에서는 이 짝이 잡히지 않았다.
//!
//! ⚠ 원 이슈가 든 `2769535` 2쪽은 이 수정으로도 신호가 나지 않는다. 그 쪽의
//! `마. 행정박물류` 잉크는 `x 91.6..199.8` 인데 같은 높이의 다른 글자는 `x >= 300`
//! 이라 **가로로 겹치지 않는다**. 그 이슈가 "진짜 겹침 7쌍" 이라고 센 것은 전부
//! 줄 상자 교차이고, 그건 `glyph_band_bbox` 가 없애려고 만든 아티팩트다. 그 쪽의
//! 실제 결함은 소제목이 165.4px 내려가는 `#7018` 이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::diagnostics::layout_anomaly::{scan_document, AnomalyOptions};
use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp";

/// 96쪽(0-based 95)의 두 본문 줄 충돌.
const PAGE: u32 = 95;

#[test]
fn issue_7023_crowded_body_lines_are_detected() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let report = scan_document(&core, &AnomalyOptions::default()).expect("scan");
    let page = report
        .pages
        .iter()
        .find(|p| p.page == PAGE)
        .expect("96쪽에 이상 신호가 있어야 한다");

    let body_pairs = page
        .text_overlap
        .iter()
        .filter(|o| o.path_a.contains("/Body/") && o.path_b.contains("/Body/"))
        .count();
    assert!(
        body_pairs >= 3,
        "96쪽의 붙은 본문 줄 짝 3건을 잡아야 한다 (중앙 기준 띠에서는 0건): {:?}",
        page.text_overlap
            .iter()
            .map(|o| (o.path_a.as_str(), o.path_b.as_str(), o.overlap_h))
            .collect::<Vec<_>>()
    );
}
