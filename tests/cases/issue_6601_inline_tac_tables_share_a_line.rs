//! [Issue #6601] 한 줄에 들어가는 인라인 TAC 표 두 개를 세로로 쌓아, 아래 내용이
//! 통째로 143pt 밀리던 결함의 가드.
//!
//! `layout_inline_table_paragraph` 안에서 **정렬 폭과 줄넘김 폭이 서로 다른 값**을
//! 썼다.
//!
//! - 정렬(`total_width`)은 `table_widths` — 열별 셀 폭(`col_span == 1` max 합)
//! - 줄넘김(`should_wrap_middle_anchored_table`)은 `table_footprint` — **선언 폭**
//!
//! 병합 셀이 많은 표에서 열 합산은 과소계상된다. 그러면 정렬이 줄 폭을 잘못 나눠
//! 시작 x 를 오른쪽으로 밀고, 그 밀린 만큼 줄넘김 검사가 넘쳐 둘째 표가 다음 줄로
//! 내려간다.
//!
//! 실측 `36331407_결재문서본문.hwpx` pi=0 (한글 2024 오라클 — 두 표가 나란하다):
//!
//! ```text
//! table_widths = [172.96, **124.95**]      둘째 표 선언 폭은 33920HU = 452.3px
//! total 350.3  →  Center 시작 x 가 +165.0  (선언 폭이면 +23.6)
//!   341.7 + 455.9 = 797.6 > 680.3  → 줄넘김
//! 선언 폭 사용 시  176.6 + 455.9 = 632.5 < 680.3  → 한 줄
//! ```
//!
//! `#5785` 가 `is_tac_table_inline` 에 세운 계약("판정 폭은 선언 폭을 우선한다")과 같다.
//!
//! ```text
//!                   수정 전                수정 후          한글 2024
//! 결재란   y  72.3 ~ 200.9  x0=168     72.3  x0=45        72.3  x0=45
//! 문서정보 y 215.4 ~ 343.8  x0= 44     71.5  x0=177       72.4  x0=210
//!
//! 1쪽 매칭 텍스트 편차  median 143.06pt → 1.41pt
//! layout-anomaly  off-canvas 1 → 0 · 넘침 2 → 1 · 쪽수 4 불변
//! ```
//!
//! ⚠ 둘째 표의 x 는 아직 한/글보다 33pt 왼쪽이다(177 vs 210). 세로 배치는 닫혔고
//! 가로 분배는 남는다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue6601/36331407_side_by_side_tac_tables.hwpx";

/// 두 표의 상단 y 차이가 이보다 작으면 "나란하다". 회귀하면 143pt 벌어진다.
const MAX_TOP_DELTA_PX: f64 = 20.0;

fn collect_tables(node: &serde_json::Value, out: &mut Vec<(f64, f64, f64)>) {
    if let (Some(ty), Some(bbox)) = (node.get("type").and_then(|t| t.as_str()), node.get("bbox")) {
        if ty == "Table" {
            let g = |k: &str| bbox.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
            out.push((g("y"), g("x"), g("w")));
        }
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        collect_tables(child, out);
    }
}

#[test]
fn two_inline_tac_tables_stay_on_one_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let json = document
        .get_page_render_tree(0)
        .expect("render tree page 1");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let mut tables = Vec::new();
    collect_tables(&tree, &mut tables);
    // 1쪽 상단 두 표(결재란·문서정보)만 본다 — 본문 표는 훨씬 아래다.
    tables.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    assert!(
        tables.len() >= 2,
        "1쪽에 표 노드가 둘 이상 있어야 한다 — 실측 {}개",
        tables.len()
    );
    let (y0, _, _) = tables[0];
    let (y1, _, _) = tables[1];

    assert!(
        (y1 - y0).abs() <= MAX_TOP_DELTA_PX,
        "1쪽 상단 두 TAC 표가 y={y0:.1} / {y1:.1} 로 갈라졌다 — #6601 회귀. \
         정렬 폭이 열 합산(과소)이고 줄넘김 폭이 선언 폭이면 둘째 표가 다음 줄로 내려가 \
         아래 내용이 143pt 밀린다 (허용 차 {MAX_TOP_DELTA_PX:.1}px)"
    );
}
