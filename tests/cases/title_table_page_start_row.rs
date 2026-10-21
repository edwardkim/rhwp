#![cfg(not(target_arch = "wasm32"))]
//! 쪽 위를 쪽·종이 기준 어울림 제목 표가 차지하면 한글은 그 쪽의 첫 줄을 제목 표 바깥
//! 상자 아래에서 시작하고, 저장 첫 줄 vpos 가 그 상자 바닥과 같다. 그 쪽은 저장 위치를
//! 그대로 쓴다: 제목 표만 든 문단의 줄은 상자 바닥에, 다음 문단의 글자처럼 취급 표는 그
//! 저장 위치에 놓인다.
//!
//! 고치기 전에는 쪽 첫 vpos 를 쪽 원점으로 빼서 첫 줄을 본문 위끝에 두었고, 다음 문단의
//! 표가 제목 표를 덮었다(85px). 표본은 `paper-square-table-tac-host.hwpx` 의 글자처럼 취급
//! 표를 다음 문단으로 옮겨 한글 2024 로 다시 저장한 HWPX 다.
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/title-table-then-inline-table.hwpx";

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 쪽 render tree 의 (종류, 문단, 컨트롤, 위끝, 아래끝). 표 안 글줄은 뺀다.
fn boxes(doc: &HwpDocument, page: u32) -> Vec<(String, u64, Option<u64>, f64, f64)> {
    fn walk(node: &Value, out: &mut Vec<(String, u64, Option<u64>, f64, f64)>) {
        let kind = node["type"].as_str().unwrap_or_default();
        if matches!(kind, "Table" | "TextLine") {
            if let Some(pi) = node["pi"].as_u64() {
                let y = node["bbox"]["y"].as_f64().unwrap();
                let bottom = y + node["bbox"]["h"].as_f64().unwrap();
                out.push((kind.to_string(), pi, node["ci"].as_u64(), y, bottom));
            }
        }
        if kind != "Table" {
            for child in node["children"].as_array().into_iter().flatten() {
                walk(child, out);
            }
        }
    }
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
    let mut out = Vec::new();
    walk(&tree, &mut out);
    out
}

#[test]
fn page_opened_below_title_table_keeps_stored_positions() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(doc.page_count(), 1, "독립 한글 출력과 같은 쪽 수");

    let paragraphs = &doc.document().sections[0].paragraphs;
    let (square, square_outer_bottom) = paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .find_map(|(index, control)| match control {
            Control::Table(table) if !table.common.treat_as_char => {
                Some((index as u64, table.outer_margin_bottom))
            }
            _ => None,
        })
        .expect("첫 문단의 쪽·종이 기준 어울림 제목 표");
    let inline = paragraphs[1]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Table(table) if table.common.treat_as_char))
        .expect("다음 문단의 글자처럼 취급 표") as u64;
    // 제목 표만 든 문단의 저장 줄 하나만큼 다음 문단이 아래에 있다.
    let host_line =
        paragraphs[1].line_segs[0].vertical_pos - paragraphs[0].line_segs[0].vertical_pos;
    assert!(host_line > 0, "다음 문단의 저장 위치가 제목 문단 줄 아래다");

    let boxes = boxes(&doc, 0);
    let table = |pi: u64, ci: u64| {
        boxes
            .iter()
            .find(|(kind, p, c, _, _)| kind == "Table" && *p == pi && *c == Some(ci))
            .map(|(_, _, _, top, bottom)| (*top, *bottom))
            .unwrap_or_else(|| panic!("문단 {pi} 컨트롤 {ci} 표가 1쪽에 없다"))
    };
    let (_, square_bottom) = table(0, square);
    let (inline_top, inline_bottom) = table(1, inline);
    let floor = square_bottom + hu_to_px(i32::from(square_outer_bottom));
    let expected = floor + hu_to_px(host_line);
    assert!(
        (inline_top - expected).abs() <= 0.5,
        "글자처럼 취급 표({inline_top:.1})가 저장 위치({expected:.1}, 제목 표 바깥 상자 {floor:.1} + 제목 문단 줄)에 있지 않다"
    );

    // 그 뒤 문단의 표와 글줄은 모두 그 표 아래에서 시작하고 서로 겹치지 않는다.
    let mut successors: Vec<_> = boxes
        .iter()
        .filter(|(_, pi, _, _, _)| *pi > 1)
        .map(|(kind, pi, _, top, bottom)| (kind.as_str(), *pi, *top, *bottom))
        .collect();
    assert!(!successors.is_empty(), "뒤 문단이 1쪽에 있다");
    for (kind, pi, top, _) in &successors {
        assert!(
            *top >= inline_bottom - 0.5,
            "문단 {pi} {kind}({top:.1})가 표 아래끝({inline_bottom:.1}) 위로 올라왔다"
        );
    }
    successors.sort_by(|a, b| a.2.total_cmp(&b.2));
    for pair in successors.windows(2) {
        let (kind_a, pi_a, _, bottom_a) = pair[0];
        let (kind_b, pi_b, top_b, _) = pair[1];
        assert!(
            top_b >= bottom_a - 0.5,
            "문단 {pi_a} {kind_a} 와 문단 {pi_b} {kind_b} 가 겹친다"
        );
    }
}
