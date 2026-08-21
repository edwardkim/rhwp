//! [Issue #5729] 연달아 놓인 자리차지(TAC) 표가 4.2px 겹쳐 앞 표 괘선이 뒷 표
//! 글자를 가로지른다 — 최상위 표 경로 om_top 누락 (156505870 1쪽 머리 표).
//!
//! 근인: 저장 줄 밴드가 정확히 `om_top+선언높이+om_bottom`(283+h+283HU)인 TAC
//! 표 4개가 쌓여 있는데, 흐름이 직전 TAC 의 저장 seg 로 전진해 온 경우
//! (`prev_tac_seg_applied`) om_top 가산을 건너뛰어 2~4번째 표가 3.8px 위에
//! 앉았다 — 한글 이중 괘선 간격 0.4px vs rhwp 4.3px, 글자가 위 괘선 관통.
//!
//! 수정: 저장 밴드=바깥 상자 증거(`tac_stored_band_is_outer_box`)가 있으면 그
//! 경우에도 om_top 을 가산. 한글 오라클 159.3/159.7 ↔ 수정 후 159.5/160.0.
//! 10k COM-free 쪽수 A/B 회귀 0.
//!
//! 픽스처는 원본 HWPX 구역0 문단 0..8(머리 표 4개) 절단 + 스텁(62KB).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5729/stacked_tac_band_om_top.hwpx";

#[test]
fn issue_5729_stacked_tac_tables_keep_outer_margin_top() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 폭 300px 이상의 가로 괘선 y 를 수집한다.
    let mut rules: Vec<f64> = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let (Some(x1), Some(x2), Some(y1), Some(y2)) = (
            attr_f64(head, "x1=\""),
            attr_f64(head, "x2=\""),
            attr_f64(head, "y1=\""),
            attr_f64(head, "y2=\""),
        ) else {
            continue;
        };
        if (y1 - y2).abs() < 0.1 && (x2 - x1).abs() > 300.0 {
            rules.push(y1);
        }
    }
    // 표2 상단 괘선은 표1 하단(160.0) 바로 위 159.5 — om_top 누락 결함이면
    // 3.8px 위(155.7)로 앉아 [154.5, 158.5] 구간에 괘선이 생긴다.
    let misplaced: Vec<f64> = rules
        .iter()
        .copied()
        .filter(|&y| (154.5..=158.5).contains(&y))
        .collect();
    assert!(
        misplaced.is_empty(),
        "쌓인 TAC 표의 위 괘선이 om_top 없이 3.8px 위에 앉았다 (한글 159.3): {misplaced:?}"
    );
    assert!(
        rules.iter().any(|&y| (158.5..=161.0).contains(&y)),
        "표2 상단 괘선(≈159.5)이 있어야 검증이 유효하다"
    );
}

fn attr_f64(head: &str, key: &str) -> Option<f64> {
    let rest = head.split_once(key)?.1;
    rest[..rest.find('"')?].parse().ok()
}
