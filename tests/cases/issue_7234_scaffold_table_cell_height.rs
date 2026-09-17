//! [#7234] scaffold 표 셀의 선언 높이는 한 줄 행 높이(안 여백 + 줄 높이)다.
//!
//! 종전 scaffold 는 셀 높이를 안 여백 합(`141 + 141 = 282 HU`)으로만 적었다. 그 값은 셀 상하
//! 여백 합과 **같아** `Cell::vertical_padding_is_abnormal`(#501) 이 발동하고, 조판의 행 컷
//! (`row_cut_content_height`)이 여백을 절반으로 줄여 행을 15.2px 로 셌다. 측정·렌더·한/글은
//! 17.09px(= 1000 + 282 HU)로 그리므로, 60행 표 첫 조각에 53행이 들어간다고 보고 본문 바닥을
//! +91.1px 넘겼다(한/글은 46행에서 나눈다).
//!
//! 셀 높이를 행 높이(1282 HU)로 적으면 여백이 비정상 판정을 받지 않아 컷·측정·렌더가 같은
//! 행 높이를 쓴다. 한/글 2020 MCP PDF 는 h=282 산출물과 h=1282 산출물이 60행·3행 표 모두
//! **화소 동일**이다(한/글에서 셀 높이는 최소 높이) — 같은 문서를 한/글로 HWP 재저장해도 셀
//! h=282 를 유지하므로 저장값 자체는 한/글과 호환되고, 이 변경은 rhwp 조판 정합을 위한 것이다.
//!
//! 엔진 쪽 규칙(여백 합 == 셀 높이에서 컷과 측정의 불일치)은 이 테스트 범위가 아니다 —
//! #7234 코멘트의 코퍼스 A/B 참조.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::scaffold::{build_scaffold, parse_scaffold_str};
use rhwp::serializer::serialize_hwpx;

fn spec(rows: usize) -> String {
    let mut table_rows = vec![r#"["No","항목","설명"]"#.to_string()];
    table_rows.extend((1..=rows).map(|i| format!(r#"["{i}","항목 {i}","설명 {i}"]"#)));
    format!(
        r#"{{"version":"1","title":"표 쪽 넘김","blocks":[
            {{"type":"paragraph","text":"{rows}행 표"}},
            {{"type":"table","rows":[{}]}},
            {{"type":"paragraph","text":"표 다음 문단"}}
        ]}}"#,
        table_rows.join(",")
    )
}

fn only_table(doc: &rhwp::model::document::Document) -> &Table {
    doc.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.controls.iter())
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .expect("표")
}

fn collect_runs(node: &RenderNode, out: &mut Vec<String>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push(run.text.trim().to_string());
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

fn body_bottom(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y + node.bbox.height);
    }
    node.children.iter().find_map(body_bottom)
}

fn max_bottom(node: &RenderNode, kind: fn(&RenderNodeType) -> bool, acc: &mut f64) {
    if kind(&node.node_type) {
        *acc = acc.max(node.bbox.y + node.bbox.height);
    }
    for child in &node.children {
        max_bottom(child, kind, acc);
    }
}

#[test]
fn issue_7234_scaffold_cell_height_is_one_line_row_height() {
    let doc = build_scaffold(&parse_scaffold_str(&spec(60)).expect("명세"));
    let table = only_table(&doc);
    let (pad_top, pad_bottom) = (table.padding.top as u32, table.padding.bottom as u32);
    for cell in &table.cells {
        assert_eq!(
            cell.height,
            pad_top + 1000 + pad_bottom,
            "셀({},{}) 선언 높이는 안 여백 + 한 줄 높이여야 한다",
            cell.row,
            cell.col
        );
        assert!(
            !rhwp::model::table::Cell::vertical_padding_is_abnormal(
                f64::from(cell.height),
                f64::from(pad_top + pad_bottom)
            ),
            "셀 선언 높이가 안 여백을 비정상으로 만들면 안 된다"
        );
    }
}

#[test]
fn issue_7234_scaffold_tall_table_first_fragment_stays_inside_body() {
    let doc = build_scaffold(&parse_scaffold_str(&spec(60)).expect("명세"));
    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let core = rhwp::document_core::DocumentCore::from_bytes(&hwpx).expect("열기");
    assert_eq!(core.page_count(), 2, "60행 표는 두 쪽이다(한/글 2쪽)");

    let page = core.build_page_render_tree(0).expect("1쪽");
    let bottom = body_bottom(&page.root).expect("Body");
    let mut cell_bottom = f64::MIN;
    max_bottom(
        &page.root,
        |t| matches!(t, RenderNodeType::TableCell(_)),
        &mut cell_bottom,
    );
    assert!(
        cell_bottom <= bottom + 0.5,
        "1쪽 표 조각의 칸 바닥 {cell_bottom:.1} 이 본문 바닥 {bottom:.1} 을 넘었다"
    );

    // 한/글 2020 PDF: 1쪽 마지막 행 `설명 46`. 첫 열 번호로 확인한다.
    let mut runs = Vec::new();
    collect_runs(&page.root, &mut runs);
    assert!(runs.iter().any(|t| t == "46"), "1쪽에 46행이 있어야 한다");
    assert!(
        !runs.iter().any(|t| t == "47"),
        "47행은 2쪽으로 가야 한다(한/글)"
    );
}
