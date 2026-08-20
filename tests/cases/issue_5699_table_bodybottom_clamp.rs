//! [Issue #5699 H3-J3] 자리차지(TopAndBottom) 표를 body_bottom 클램프가 흐름
//! 위치 위로 끌어올려, 이미 페인트된 직전 본문 줄과 겹쳐 그리던 결함의 회귀 가드.
//!
//! 근인: `compute_table_y_position`의 Para 기준 본문영역 클램프가, 쪽 말미에서
//! 표가 본문 하단에 다 안 들어갈 때 표 상단을 `body_bottom`(= 본문 하단 - 표높이)
//! 까지 상향 이동시킨다. rhwp 흐름이 저장 앵커보다 아래까지 이미 그린 문서에서는
//! 이 상향이 직전 줄을 침범한다(37787 규제영향분석서 p6 실측: 흐름 1017.6 → 클램프
//! 990.8, 직전 줄 983.6..1003.6 을 12px 침범). 자리차지는 텍스트가 겹칠 수 없는
//! 계약이므로, 클램프가 흐름 위로 끌어올리는 경우에 한해 클램프를 풀어 하단 여백
//! bleed 로 둔다. 같은 기전이 전면(full-band) 표에서는 쪽 상단부터 앞 문단 전체를
//! 덮는 형태(J2 가족)로 나타났다.
//!
//! 계약: 동봉 재현 문서 p6 에서 pi62 본문 줄과 pi63 자리차지 표가 세로로 겹치지
//! 않는다. 수정 전 실측: 겹침 12.0px (r38 검출기 v4 TLTB).
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue5699/37787_regulatory_impact.hwp";

/// (para_index, y0, y1, x0, x1)
type Band = (usize, f64, f64, f64, f64);

fn collect_body_bands(
    node: &RenderNode,
    lines: &mut Vec<Band>,
    tables: &mut Vec<Band>,
    in_cell: bool,
) {
    let cell_like = matches!(
        node.node_type,
        RenderNodeType::TableCell(_) | RenderNodeType::Header | RenderNodeType::Footer
    ) || in_cell;
    match &node.node_type {
        RenderNodeType::TextLine(tl) if !cell_like => {
            if let Some(pi) = tl.para_index {
                let b = &node.bbox;
                if b.height > 0.0 && b.height < 150.0 {
                    lines.push((pi, b.y, b.y + b.height, b.x, b.x + b.width));
                }
            }
        }
        RenderNodeType::Table(tn) if !cell_like => {
            if let Some(pi) = tn.para_index {
                let b = &node.bbox;
                tables.push((pi, b.y, b.y + b.height, b.x, b.x + b.width));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect_body_bands(child, lines, tables, cell_like);
    }
}

#[test]
fn issue_5699_topbottom_table_bodybottom_clamp_does_not_intrude_previous_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    // 수정 전 실측 결함: p6 에서 pi63 자리차지 표(991.1..1046.9)가 pi62 줄
    // (983.6..1003.6)을 12px 침범. 그 쌍에 한정해 잠근다 — 페이지 전수 스윕
    // 계약은 부속 플로우 잔재까지 걸어 위양성이 난다.
    let mut checked = false;
    for page in 0..doc.page_count() {
        let tree = doc
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render tree p{page}: {e}"));
        let mut lines = Vec::new();
        let mut tables = Vec::new();
        collect_body_bands(&tree.root, &mut lines, &mut tables, false);
        let line62: Vec<&Band> = lines.iter().filter(|l| l.0 == 62).collect();
        let table63: Vec<&Band> = tables.iter().filter(|t| t.0 == 63).collect();
        if line62.is_empty() || table63.is_empty() {
            continue;
        }
        checked = true;
        for a in &line62 {
            for t in &table63 {
                let dy = a.2.min(t.2) - a.1.max(t.1);
                let dx = a.4.min(t.4) - a.3.max(t.3);
                assert!(
                    !(dy >= 8.0 && dx > 0.0),
                    "p{} 에서 pi62 줄({:.0}..{:.0})과 pi63 자리차지 표({:.0}..{:.0})가 겹침 (#5699 J3 회귀)",
                    page + 1,
                    a.1,
                    a.2,
                    t.1,
                    t.2
                );
            }
        }
    }
    assert!(
        checked,
        "pi62 줄/pi63 표를 가진 쪽이 없다 — 샘플/계약 확인 필요"
    );
}
