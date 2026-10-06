//! [Issue #4915 / #4889] 저장 reset 이 전혀 없는 **다쪽(≥2쪽) 1×1 중첩 표**가 legacy
//! 원자 하나로 뭉쳐, 조각 페인트가 용지 밖까지 괘선을 그리던 결함의 가드.
//!
//! `nested_table_mixed_fragment_heights` 의 canonical CellUnit 투영은 저장 프레임
//! 경계(reset)가 있어야 켜졌다. reset 0 인 이 문서는 legacy 로 물러나 손자 표가
//! **2095.6px 원자** 하나가 됐고, p2 조각(유닛 10..11 = 그 원자 하나)이 표 전체
//! 높이를 그렸다.
//!
//! 실측 `samples/issue4889/18098267_nested_fragment_origin.hwp`
//! (한글 2024 실측 3쪽 — COM 이 이 버전으로만 해석된다):
//!
//! ```text
//! 유닛 원장(legacy)  [... (33.6, 7), (2095.6, 7), (33.6, 7) ...]   ← 원자
//! canonical 원장     63 유닛                                        ← 분해됨
//!
//!                     수정 전                수정 후      한/글
//! p2 괘선 최하단       1603.2pt              841.9pt      (용지 841.9)
//! 쪽별 글자수          [118, 780, 33]        [627, 764, 402]   [627, 747, 419]
//! layout-anomaly      offcv 2 · 넘침 1      offcv 0 · 넘침 0   쪽수 3 불변
//! ```
//!
//! ⭐ 판별자는 `reset == 0 && 물리 높이 > 2쪽` 이다. 한 쪽 언저리 표(form-002 ·
//! 76076 계열, issue1891 의 깊은 wrapper — reset 0 전면 포함의 기지 반증)는 2쪽
//! 임계에 걸리지 않는다. `issue_1891` 4건과 `overflow_cell partition_12` 로 반대쪽을
//! 확인했다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue4889/18098267_nested_fragment_origin.hwp";

fn lowest_drawing_bottom(node: &serde_json::Value, out: &mut f64) {
    if let Some(bbox) = node.get("bbox") {
        let y = bbox.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let h = bbox.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0);
        if y + h > *out {
            *out = y + h;
        }
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        lowest_drawing_bottom(child, out);
    }
}

#[test]
fn reset_free_multi_page_nested_table_fragments_stay_inside_the_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    assert_eq!(
        document.page_count(),
        3,
        "쪽수는 한/글과 같은 3 이어야 한다"
    );

    // p2(0-기준 1)가 종전에 표 전체 높이(2137px)를 그리던 조각이다.
    let json = document
        .get_page_render_tree(1)
        .expect("render tree page 2");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let mut lowest = 0.0f64;
    lowest_drawing_bottom(&tree, &mut lowest);

    // 용지 1123px. 회귀하면 조각이 2137px 까지 그린다.
    assert!(
        lowest <= 1125.0,
        "p2 노드 최하단이 {lowest:.1}px 이다 — #4915 회귀. reset 0 다쪽 1×1 표가 legacy \
         원자로 뭉치면 조각이 표 전체 높이(2137px)를 그린다 (용지 1123px)"
    );
}
