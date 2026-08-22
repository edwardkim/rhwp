//! [Issue #5785] 글자처럼 취급 중첩 표 3개만 오른쪽으로 22~27px 밀려 부모 칸을
//! 침범한다 (3049001 약장 패용방법, 12개 중 약장 2·5·11).
//!
//! 근인: 인라인 판정(`is_tac_table_inline`)의 폭이 `get_column_widths()` 합
//! (전역 그리드 col_span==1 max)이라, 행마다 열 구획이 다른 표(#5697)에서
//! 12,872 vs 17,299HU 로 표마다 흔들렸다 — 저장 속성이 12곳 전부 동일한데
//! 과소합산된 표만 90% 문턱을 우연히 통과해 인라인이 되고, 그 인라인 흐름이
//! 이웃 셀 폴백 기준 x 를 +22~27px 오염시켰다.
//!
//! 수정: 판정 폭을 **선언 폭 우선**으로(선언 0 인 합성 표만 colsum 폴백).
//! 한글 2022 PDF 오라클: 약장5 x=149.04 ↔ 수정 후 149.2, 약장2 x=186.78 ↔
//! 187.0. 10k COM-free 쪽수 A/B: 개선 1건(20731565 2→1=한글), 회귀 0.
//!
//! 픽스처는 원본 HWPX(12KB) 절단본 — 1쪽 별표 전체.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5785/medal_cells_ws_host_inline.hwpx";

#[test]
fn issue_5785_medal_nested_tables_stay_left_aligned() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 세로 괘선 x 수집.
    let mut xs: Vec<f64> = Vec::new();
    for cap in svg.split("<line ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let (Some(x1), Some(x2)) = (attr_f64(head, "x1=\""), attr_f64(head, "x2=\"")) else {
            continue;
        };
        if (x1 - x2).abs() < 0.1 {
            xs.push(x1);
        }
    }
    // 결함 시 밀린 중첩 표의 왼쪽 괘선이 176.1(약장5) / 208.9(약장2) /
    // 488.1(약장11)에 생긴다 — 정상은 149.2 / 187.0 / 461.2.
    let misplaced: Vec<f64> = xs
        .iter()
        .copied()
        .filter(|&x| {
            (172.0..=180.0).contains(&x)
                || (205.0..=212.0).contains(&x)
                || (484.0..=492.0).contains(&x)
        })
        .collect();
    assert!(
        misplaced.is_empty(),
        "중첩 표가 오른쪽으로 밀렸다 (한글 149.04/186.78/460.87): {misplaced:?}"
    );
    for want in [149.2, 187.0, 461.2] {
        assert!(
            xs.iter().any(|&x| (x - want).abs() < 1.5),
            "정상 위치({want})의 괘선이 있어야 검증이 유효하다"
        );
    }
}

fn attr_f64(head: &str, key: &str) -> Option<f64> {
    let rest = head.split_once(key)?.1;
    rest[..rest.find('"')?].parse().ok()
}
