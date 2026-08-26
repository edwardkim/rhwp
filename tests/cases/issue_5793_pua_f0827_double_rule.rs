//! [Issue #5793] PUA 0xF0827 을 ■ 로 그려 제목 밑 이중선이 검은 사각형 띠가 된다
//! — 길이 2배·제목과 겹침 (1776332).
//!
//! 한글 2022 시각 판정: `U+F000`(HWP5 사용자 정의 기호 0xA827) ×34 는 제목 폭에 맞는
//! **이중 가로선**(반각 6.66px/자, 280.0→506.5px)이다. rhwp 의 잠정 매핑 ■(전각
//! 12.9px/자)은 띠를 440px 로 늘여 제목 글자를 겹쳤다(layout-anomaly text-overlap
//! w=213 1위).
//!
//! 수정: 0xF0827 → ═(U+2550, 이웃 0xF0832 와 같은 이중선 계열).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5793/pua_f0827_double_rule.hwp";

#[test]
fn issue_5793_f0827_renders_as_double_horizontal_rule() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 이중선 34자, 반각 진행 — 마지막 글자 x ≈ 499.9 (한글 506.5 종점과 정합).
    let mut xs = Vec::new();
    for cap in svg.split("<text ").skip(1) {
        let Some(gt) = cap.find('>') else { continue };
        let head = &cap[..gt];
        let body = &cap[gt + 1..cap.find("</text>").unwrap_or(cap.len())];
        if body == "═" {
            if let Some(s) = head.find("x=\"") {
                let s = s + 3;
                if let Some(e) = head[s..].find('"') {
                    if let Ok(x) = head[s..s + e].parse::<f64>() {
                        xs.push(x);
                    }
                }
            }
        }
    }
    assert_eq!(xs.len(), 34, "이중선 문자 34자여야 한다: {}", xs.len());
    let max_x = xs.iter().cloned().fold(f64::MIN, f64::max);
    assert!(
        (450.0..520.0).contains(&max_x),
        "이중선이 제목 폭 안(마지막 자 x≈499.9)에서 끝나야 한다 — ■ 전각 결함 시 707: {max_x:.1}"
    );
    assert!(!svg.contains('■'), "■ 띠가 남아 있으면 안 된다");
}
