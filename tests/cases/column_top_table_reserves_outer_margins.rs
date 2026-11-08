#![cfg(not(target_arch = "wasm32"))]
//! 단 맨 위 문단 기준 자리차지 표 뒤 문단은 표 아랫변 + 바깥 아래 여백에서 시작한다.
//!
//! 단 맨 위 표는 바깥 위 여백만큼 내려 그린다(#4068). 그 흐름은 저장 사다리가 바깥 여백
//! 상자를 담는다고 보고 위 여백을 되돌리는데, 편집 vpos 재계산은 빈 앵커 문단을 한 줄
//! (1000 + 600HU)로만 잇는다. 그래서 새 문서 첫 줄 표 뒤 문단이 표 아랫부분과 겹쳤다.
//! HWP·HWPX로 저장해 다시 열어도 같아야 한다.
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn hu_to_px(hu: i16) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 1쪽 render tree 의 (종류, 문단, 위끝, 아래끝). 표 안 글줄은 뺀다.
fn boxes(doc: &HwpDocument) -> Vec<(String, u64, f64, f64)> {
    fn walk(node: &Value, out: &mut Vec<(String, u64, f64, f64)>) {
        let kind = node["type"].as_str().unwrap_or_default();
        if matches!(kind, "Table" | "TextLine") {
            let y = node["bbox"]["y"].as_f64().unwrap();
            let bottom = y + node["bbox"]["h"].as_f64().unwrap();
            out.push((kind.to_string(), node["pi"].as_u64().unwrap(), y, bottom));
        }
        if kind != "Table" {
            for child in node["children"].as_array().into_iter().flatten() {
                walk(child, out);
            }
        }
    }
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let mut out = Vec::new();
    walk(&tree, &mut out);
    out
}

/// 문단 0 표 뒤 글줄이 모두 표 아랫변 + 바깥 아래 여백 아래에 있다.
fn assert_text_clears_table(doc: &HwpDocument, label: &str) {
    let Control::Table(table) = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find(|control| matches!(control, Control::Table(_)))
        .unwrap()
    else {
        unreachable!()
    };
    let boxes = boxes(doc);
    let (_, _, top, bottom) = boxes
        .iter()
        .find(|(kind, pi, _, _)| kind == "Table" && *pi == 0)
        .unwrap_or_else(|| panic!("{label}: 1쪽에 문단 0의 표가 없다"));
    let floor = bottom + hu_to_px(table.outer_margin_bottom);
    let after: Vec<_> = boxes
        .iter()
        .filter(|(kind, pi, _, _)| kind == "TextLine" && *pi > 0)
        .collect();
    assert!(!after.is_empty(), "{label}: 표 뒤 글줄이 없다");
    for (_, pi, y, _) in after {
        assert!(
            *y >= floor - 0.5,
            "{label}: 문단 {pi} 글줄 {y:.1}px가 표({top:.1}~{bottom:.1}px) 아래 여백 끝 {floor:.1}px보다 위다"
        );
    }
}

/// 새 문서 첫 줄 2×2 표. 표 아래 빈 문단 1에 `next` 를 쓰고 그 뒤에 글 문단을 하나 둔다.
fn first_line_table(next: &str) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.create_table(0, 0, 0, 2, 2).unwrap();
    if !next.is_empty() {
        doc.insert_text(0, 1, 0, next).unwrap();
    }
    doc.split_paragraph(0, 1, next.chars().count() as u32, None)
        .unwrap();
    doc.insert_text(0, 2, 0, "셋째 문단").unwrap();
    doc
}

/// 문단 0 표의 바깥 위·아래 여백을 바꾼 문서. 저장 원본 바이트는 쓰지 않으므로 화면만 본다.
fn with_outer_margins(mut doc: HwpDocument, margin: i16) -> HwpDocument {
    let mut source = doc.document().clone();
    for control in &mut source.sections[0].paragraphs[0].controls {
        if let Control::Table(table) = control {
            table.outer_margin_top = margin;
            table.outer_margin_bottom = margin;
            table.common.margin.top = margin;
            table.common.margin.bottom = margin;
        }
    }
    doc.set_document(source);
    doc
}

#[test]
fn paragraphs_after_first_line_table_start_below_its_bottom_margin() {
    for next in ["", "표 뒤 글"] {
        let doc = first_line_table(next);
        assert_text_clears_table(&doc, &format!("다음 문단 {next:?}"));
        for (format, bytes) in [
            ("HWP", doc.export_hwp().unwrap()),
            ("HWPX", doc.export_hwpx().unwrap()),
        ] {
            let reopened = HwpDocument::from_bytes(&bytes).unwrap();
            assert_text_clears_table(
                &reopened,
                &format!("다음 문단 {next:?}, {format} 다시 열기"),
            );
        }
        let doc = with_outer_margins(doc, 5000);
        assert_text_clears_table(&doc, &format!("다음 문단 {next:?}, 바깥 여백 5000HU"));
    }
}
