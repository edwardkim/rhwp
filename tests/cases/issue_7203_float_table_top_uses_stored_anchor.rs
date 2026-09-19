//! [#7203] 자리차지 표의 윗변이 **앵커 문단의 저장 자리**에 놓인다 — 앞 문단 글자를 뚫지 않는다.
//!
//! `samples/hwpctl_API_v2.4.hwp` 28쪽(0-based 27)의 코드 상자 표는 빈 host 문단
//! `pi=574`(저장 `vpos=40693`)에 매달린 `wrap=자리차지 · vert=문단` 표다. 이 쪽의 글줄은
//! 전부 저장 사다리를 따른다 — 예로 `pi=573` 은 `vpos=38893` → `y=650.84`(±0.1px).
//! 같은 사다리로 환산하면 host 의 자리는 **674.8px** 이다.
//!
//! ```text
//!   수정 전  표 윗변 661.51  = 앵커 저장 자리 − 줄 높이(1000HU = 13.33px)
//!                            앞 문단 pi=573 줄 상자 650.8 .. 664.1 을 관통한다
//!   한/글    표 윗변 671.27  (pdf/hwpctl_API_v2.4-hwp-2020.pdf 가로 괘선 실측)
//!   수정 후  표 윗변 671.03  = 앵커 저장 자리 − 위여백 283HU (정본 괘선과 0.24px)
//! ```
//!
//! 막고 있던 것은 `stored_ladder_leaves_object_room` 의 필요 공간 산식이었다. 앵커 아래에
//! `높이 + 위여백 + 아래여백` 을 요구했는데, 저장 사다리의 간격은 `10948HU` 로 그보다
//! `501HU` 작아 저장 anchor 경로가 통째로 꺼졌다. 정본은 이 표의 윗변을 앵커보다 **위여백
//! 한 개 위**에 두므로 앵커 아래로 필요한 공간은 `높이 + 아래여백 − 위여백`(= 대칭 여백이면
//! 높이)이고, 그 값으로는 `10882 ≤ 10948` 로 들어간다.
//!
//! 이 검사는 앞 문단 비침범과 정본 괘선 0.5px 이내 정렬, 비대칭 여백을 고정한다.
//! 다른 페이지의 별도 표 배치는 이 시험으로 해결했다고 주장하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/hwpctl_API_v2.4.hwp";
/// 한/글 2020 PDF 실측 (가로 괘선, 폭 427.7px).
const ORACLE_TABLE_TOP: f64 = 671.27;
/// 수정 전 관측값 — 앵커 저장 자리에서 줄 높이만큼 위.
const BEFORE_FIX_TABLE_TOP: f64 = 661.51;

fn collect(node: &RenderNode, tables: &mut Vec<(f64, f64)>, host_line: &mut Option<(f64, f64)>) {
    match &node.node_type {
        RenderNodeType::Table(_) if (node.bbox.width - 427.7).abs() < 1.5 => {
            tables.push((node.bbox.y, node.bbox.height));
        }
        RenderNodeType::TextLine(line) => {
            if line.para_index == Some(573) && host_line.is_none() {
                *host_line = Some((node.bbox.y, node.bbox.y + node.bbox.height));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, tables, host_line);
    }
}

#[test]
fn para_float_table_top_sits_at_the_stored_anchor_not_a_line_above() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {FIXTURE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {FIXTURE}: {e:?}"));
    let tree = doc
        .build_page_render_tree(27)
        .unwrap_or_else(|e| panic!("render tree 28쪽: {e:?}"));

    let mut tables = Vec::new();
    let mut host_line = None;
    collect(&tree.root, &mut tables, &mut host_line);

    let (line_top, line_bottom) = host_line.expect("앞 문단(pi=573) 줄을 찾지 못했다");
    let table_top = tables
        .iter()
        .map(|(y, _)| *y)
        .find(|y| *y > line_top)
        .expect("28쪽에서 코드 상자 표를 찾지 못했다");

    assert!(
        table_top >= line_bottom,
        "표 윗변 {table_top:.2} 가 앞 문단 줄 상자({line_top:.2}..{line_bottom:.2})를 뚫는다 \
         (수정 전 {BEFORE_FIX_TABLE_TOP:.2})"
    );
    // 한/글은 이 표의 윗변을 앵커보다 바깥여백(283HU = 3.77px) 한 개 위에 둔다.
    // 실제 배치도 같은 위여백을 빼야 하며, 허용치는 PDF stroke와 box 경계 차이뿐이다.
    let delta = table_top - ORACLE_TABLE_TOP;
    assert!(
        delta.abs() <= 0.5,
        "표 윗변 {table_top:.2} 가 정본 {ORACLE_TABLE_TOP:.2} 에서 stroke 허용치를 넘어 \
         어긋난다 (차 {delta:+.2}px, 수정 전 {:+.2}px)",
        BEFORE_FIX_TABLE_TOP - ORACLE_TABLE_TOP
    );
}

#[test]
fn saved_anchor_subtracts_only_top_margin_with_asymmetric_margins() {
    use rhwp::document_core::DocumentCore;
    use rhwp::model::control::Control;
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)).unwrap();
    for (top_margin, bottom_margin) in [(283, 0), (0, 0), (141, 0)] {
        let mut core = DocumentCore::from_bytes(&bytes).unwrap();
        let mut document = core.document().clone();
        let host = &mut document.sections[0].paragraphs[574];
        let table = host
            .controls
            .iter_mut()
            .find_map(|control| match control {
                Control::Table(table) => Some(table),
                _ => None,
            })
            .expect("stored anchor table");
        table.outer_margin_top = top_margin;
        table.outer_margin_bottom = bottom_margin;
        core.set_document(document);
        let tree = core.build_page_render_tree(27).unwrap();
        let mut tables = Vec::new();
        let mut line = None;
        collect(&tree.root, &mut tables, &mut line);
        let top = tables
            .iter()
            .map(|(y, _)| *y)
            .find(|y| *y > line.unwrap().0)
            .unwrap();
        // Saved host vpos=40693 HU, body origin=132.2266667 px. Only the
        // upper outer margin belongs above this anchor; bottom is reservation.
        let expected = 674.8 - f64::from(top_margin) * 96.0 / 7200.0;
        assert!(
            (top - expected).abs() < 0.05,
            "top={top}, expected={expected}, margins={top_margin}/{bottom_margin}"
        );
    }
}

#[test]
fn saved_outer_box_anchor_keeps_following_tables_inside_the_body() {
    // The stored 1x1 anchor ladder advances by declaration + both margins.
    // It denotes a flow origin, not an anchor after the upper margin. Mixing
    // those origins inflated the following table reservation by 19.5px.
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue6111/56345_regulatory_impact_analysis.hwp"),
    )
    .unwrap();
    let core = rhwp::document_core::DocumentCore::from_bytes(&bytes).unwrap();
    let tree = core.build_page_render_tree(19).unwrap();
    fn check(node: &RenderNode, body_bottom: Option<f64>, found: &mut bool) {
        let bottom = if matches!(node.node_type, RenderNodeType::Column(_)) {
            Some(node.bbox.y + node.bbox.height)
        } else {
            body_bottom
        };
        if let RenderNodeType::Table(meta) = &node.node_type {
            if let Some(vpos) = match meta.para_index {
                Some(357) => Some(43556),
                Some(358) => Some(45988),
                _ => None,
            } {
                let expected = 75.6 + f64::from(vpos) * 96.0 / 7200.0;
                assert!(
                    (node.bbox.y - expected).abs() < 0.05,
                    "stored flow origin: actual={}, expected={expected}",
                    node.bbox.y
                );
            }
            if meta.para_index == Some(359) {
                *found = true;
                let table_bottom = node.bbox.y + node.bbox.height;
                assert!(
                    table_bottom <= bottom.expect("column") + 0.5,
                    "measured table bottom {table_bottom} exceeds body {bottom:?}"
                );
            }
        }
        for child in &node.children {
            check(child, bottom, found);
        }
    }
    let mut found = false;
    check(&tree.root, None, &mut found);
    assert!(found, "target picture/caption table must remain present");
}
