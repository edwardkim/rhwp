//! Task #1139: 문27 inline TAC Picture가 다음 줄까지 미리 렌더되어 중복 출력되던 회귀 방지.

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn render_tree_contains_text(node: &RenderNode, needle: &str) -> bool {
    match &node.node_type {
        RenderNodeType::TextRun(run) if run.text.contains(needle) => return true,
        RenderNodeType::FootnoteMarker(marker) if marker.text.contains(needle) => return true,
        _ => {}
    }
    node.children
        .iter()
        .any(|child| render_tree_contains_text(child, needle))
}

fn svg_attr_f64(tag: &str, name: &str) -> Option<f64> {
    let pattern = format!("{name}=\"");
    let start = tag.find(&pattern)? + pattern.len();
    let end = tag[start..].find('"')?;
    tag[start..start + end].parse().ok()
}

fn sample16_page3_bottom_border_and_page_number(svg: &str) -> (f64, f64) {
    let mut bottom_lines = Vec::new();
    for tag in svg.match_indices("<line ").filter_map(|(start, _)| {
        let end = svg[start..].find('>')?;
        Some(&svg[start..start + end + 1])
    }) {
        let x1 = svg_attr_f64(tag, "x1").unwrap_or_default();
        let y1 = svg_attr_f64(tag, "y1").unwrap_or_default();
        let x2 = svg_attr_f64(tag, "x2").unwrap_or_default();
        let y2 = svg_attr_f64(tag, "y2").unwrap_or_default();
        if (y1 - y2).abs() < 0.1 && y1 > 1000.0 && (x2 - x1).abs() > 700.0 {
            bottom_lines.push(y1);
        }
    }

    let mut page_number_y = Vec::new();
    for (start, _) in svg.match_indices("<text ") {
        let Some(tag_end) = svg[start..].find('>') else {
            continue;
        };
        let tag = &svg[start..start + tag_end + 1];
        let text_start = start + tag_end + 1;
        let Some(text_end) = svg[text_start..].find("</text>") else {
            continue;
        };
        let text = &svg[text_start..text_start + text_end];
        let is_page_number = text == "-" || text.chars().all(|ch| ch.is_ascii_digit());
        if is_page_number {
            let y = svg_attr_f64(tag, "y").unwrap_or_default();
            if y > 1050.0 {
                page_number_y.push(y);
            }
        }
    }

    bottom_lines.sort_by(|a, b| a.partial_cmp(b).unwrap());
    page_number_y.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (
        bottom_lines
            .last()
            .copied()
            .expect("page bottom border line"),
        page_number_y
            .last()
            .copied()
            .expect("bottom page number baseline"),
    )
}

fn find_image_bbox(
    node: &RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<BoundingBox> {
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.para_index == Some(para_index) && image.control_index == Some(control_index) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_image_bbox(child, para_index, control_index))
}

fn find_text_line_bbox(
    node: &RenderNode,
    para_index: usize,
    line_index: u32,
) -> Option<BoundingBox> {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para_index) && line.line_index == Some(line_index) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_text_line_bbox(child, para_index, line_index))
}

fn count_equations_under_text_line(node: &RenderNode, para_index: usize) -> usize {
    let child_count = match &node.node_type {
        RenderNodeType::TextLine(line) if line.para_index == Some(para_index) => node
            .children
            .iter()
            .filter(|child| matches!(child.node_type, RenderNodeType::Equation(_)))
            .count(),
        _ => 0,
    };
    child_count
        + node
            .children
            .iter()
            .map(|child| count_equations_under_text_line(child, para_index))
            .sum::<usize>()
}

#[test]
fn issue_1209_test_image_topandbottom_picture_reserves_text_flow() {
    let bytes = std::fs::read("samples/test-image.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(0).expect("page 1 render tree");

    let image = find_image_bbox(&tree.root, 0, 2).expect("자리차지 그림");
    let text_line = find_text_line_bbox(&tree.root, 0, 0).expect("자리차지 텍스트 줄");
    let image_bottom = image.y + image.height;

    assert!(
        text_line.y + 0.1 >= image_bottom,
        "자리차지 그림은 한컴처럼 본문과 겹치면 안 됨: image={image:?}, text_line={text_line:?}"
    );
}

#[test]
fn issue_1293_equation_control_is_not_always_treat_as_char() {
    use rhwp::model::control::Control;

    let bytes = std::fs::read("samples/수식-문자처럼취급-아님.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let section = &doc.document().sections[0];
    let para = &section.paragraphs[0];

    let equation = para
        .controls
        .iter()
        .find_map(|ctrl| match ctrl {
            Control::Equation(eq) => Some(eq),
            _ => None,
        })
        .expect("수식 컨트롤");
    assert!(
        !equation.common.treat_as_char,
        "Control::Equation 타입만으로 TAC로 단정하면 안 됨"
    );

    let page_dump = doc.dump_page_items(Some(0));
    assert!(
        page_dump.contains("Shape          pi=0 ci=2  수식"),
        "비TAC 수식은 텍스트 줄 안 TAC가 아니라 별도 Shape item으로 라우팅되어야 함\n{page_dump}"
    );

    let tree = doc.build_page_render_tree(0).expect("page 1 render tree");
    assert_eq!(
        count_equations_under_text_line(&tree.root, 0),
        0,
        "비TAC 수식은 문단 TextLine 내부 인라인 Equation으로 중복 렌더되면 안 됨"
    );
}

#[test]
fn issue_1139_sample16_page3_page_number_stays_below_bottom_border() {
    let bytes = std::fs::read("samples/hwp3-sample16-hwp5.hwp").expect("sample16");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse sample16");
    let svg = doc.render_page_svg_native(2).expect("page 3 svg");
    let (bottom_line, page_number_y) = sample16_page3_bottom_border_and_page_number(&svg);

    let gap = page_number_y - bottom_line;
    assert!(
        (bottom_line - 1066.86).abs() < 0.5,
        "sample16 page-basis bottom border should not include extra double-line outset: {bottom_line}"
    );
    assert!(
        gap > 10.0,
        "page number should stay visibly below the bottom border: gap={gap}, border={bottom_line}, page_number={page_number_y}"
    );
}

#[test]
fn issue_1139_stage31_insert_endnote_on_blank_document_renders_marker() {
    use rhwp::model::control::Control;

    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native()
        .expect("create blank document");

    let result = doc
        .insert_endnote_native(0, 0, 0)
        .expect("insert_endnote_native");
    let parsed: Value = serde_json::from_str(&result).expect("insert result json");
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["endnoteNumber"], 1);

    let para = &doc.document().sections[0].paragraphs[0];
    let endnote = para
        .controls
        .iter()
        .find_map(|ctrl| match ctrl {
            Control::Endnote(en) => Some(en),
            _ => None,
        })
        .expect("본문 문단에 Endnote 컨트롤이 생성되어야 함");
    assert_eq!(endnote.number, 1);
    assert_eq!(endnote.paragraphs.len(), 1);
    assert_eq!(
        endnote.paragraphs[0].controls.len(),
        1,
        "미주 내용 문단은 AutoNumber anchor를 포함해야 함"
    );

    let tree = doc
        .build_page_render_tree(0)
        .expect("page render tree after endnote");
    assert!(
        render_tree_contains_text(&tree.root, "1)"),
        "본문에는 기본 미주 마커 형식인 '1)'이 렌더되어야 함"
    );
}

#[test]
fn issue_1139_stage31_insert_endnote_enters_editable_note_body() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native()
        .expect("create blank document");

    let result = doc
        .insert_endnote_native(0, 0, 0)
        .expect("insert_endnote_native");
    let parsed: Value = serde_json::from_str(&result).expect("insert result json");
    let control_idx = parsed["controlIdx"].as_u64().expect("controlIdx") as usize;

    let edit_info = doc
        .get_note_edit_info_native(0, 0, control_idx)
        .expect("get_note_edit_info_native");
    let edit_info: Value = serde_json::from_str(&edit_info).expect("edit info json");
    assert_eq!(edit_info["ok"], true);
    assert_eq!(edit_info["kind"], "endnote");
    assert_eq!(edit_info["fnParaIndex"], 0);
    assert_eq!(edit_info["charOffset"], 2);
    assert!(
        edit_info["virtualParaIndex"].as_u64().is_some(),
        "미주 편집 대상은 렌더링용 가상 문단을 제공해야 함"
    );

    let cursor_rect = doc
        .get_cursor_rect_in_note_native(0, 0, control_idx, 0, 2)
        .expect("get_cursor_rect_in_note_native");
    let cursor_rect: Value = serde_json::from_str(&cursor_rect).expect("cursor rect json");
    assert!(
        cursor_rect["x"].as_f64().is_some() && cursor_rect["y"].as_f64().is_some(),
        "미주 편집 위치의 캐럿 좌표를 계산해야 함"
    );

    let inserted = doc
        .insert_text_in_footnote_native(0, 0, control_idx, 0, 2, "test")
        .expect("insert text in endnote through note edit API");
    let inserted: Value = serde_json::from_str(&inserted).expect("insert text json");
    assert_eq!(inserted["ok"], true);
    assert_eq!(inserted["charOffset"], 6);

    let tree = doc
        .build_page_render_tree(0)
        .expect("page render tree after endnote text input");
    assert!(
        render_tree_contains_text(&tree.root, "1)"),
        "미주 내용 번호가 함께 렌더되어야 함"
    );
    assert!(
        render_tree_contains_text(&tree.root, "test"),
        "미주 내용 편집 API로 입력한 텍스트가 렌더되어야 함"
    );
}

#[test]
fn issue_1139_stage31_endnote_shape_prefix_renders_in_existing_endnote() {
    use rhwp::model::control::{AutoNumberType, Control};

    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native()
        .expect("create blank document");
    doc.insert_endnote_native(0, 0, 0)
        .expect("insert default endnote");

    doc.apply_endnote_shape_native(
        0,
        r##"{
            "numberFormat":"digit",
            "prefixChar":"(",
            "suffixChar":")",
            "startNumber":5,
            "separatorEnabled":true
        }"##,
    )
    .expect("apply endnote shape");

    let para = &doc.document().sections[0].paragraphs[0];
    let endnote = para
        .controls
        .iter()
        .find_map(|ctrl| match ctrl {
            Control::Endnote(en) => Some(en),
            _ => None,
        })
        .expect("Endnote control");
    assert_eq!(endnote.number, 5);
    assert_eq!(endnote.before_decoration_letter, '(' as u16);
    assert_eq!(endnote.after_decoration_letter, ')' as u16);

    let auto_num = endnote.paragraphs[0]
        .controls
        .iter()
        .find_map(|ctrl| match ctrl {
            Control::AutoNumber(an) if an.number_type == AutoNumberType::Endnote => Some(an),
            _ => None,
        })
        .expect("Endnote AutoNumber");
    assert_eq!(auto_num.assigned_number, 5);
    assert_eq!(auto_num.prefix_char, '(');
    assert_eq!(auto_num.suffix_char, ')');

    let tree = doc
        .build_page_render_tree(0)
        .expect("page render tree after shape apply");
    assert!(
        render_tree_contains_text(&tree.root, "(5)"),
        "앞 장식 문자와 시작 번호를 바꾼 뒤 본문/미주 번호가 '(5)'로 렌더되어야 함"
    );
    assert!(
        !render_tree_contains_text(&tree.root, "문1)"),
        "기존 미주도 새 장식 문자를 따라야 하므로 '문1)'이 남으면 안 됨"
    );
}

#[test]
fn issue_1139_stage31_endnote_shape_api_updates_section_shape() {
    use rhwp::model::footnote::{FootnoteNumbering, FootnotePlacement, NumberFormat};

    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native()
        .expect("create blank document");

    let before = doc
        .get_endnote_shape_native(0)
        .expect("get_endnote_shape_native");
    let before_json: Value = serde_json::from_str(&before).expect("shape json");
    assert_eq!(before_json["ok"], true);

    doc.apply_endnote_shape_native(
        0,
        r##"{
            "numberFormat":"hangulSyllable",
            "prefixChar":"[",
            "suffixChar":"]",
            "startNumber":3,
            "separatorEnabled":true,
            "separatorLength":1417,
            "separatorMarginTop":100,
            "separatorMarginBottom":200,
            "noteSpacing":300,
            "separatorLineType":1,
            "separatorLineWidth":2,
            "separatorColor":"#11aa55",
            "numbering":"restartSection",
            "placement":"sectionEnd"
        }"##,
    )
    .expect("apply_endnote_shape_native");

    let shape = &doc.document().sections[0].section_def.endnote_shape;
    assert_eq!(shape.number_format, NumberFormat::HangulSyllable);
    assert_eq!(shape.prefix_char, '[');
    assert_eq!(shape.suffix_char, ']');
    assert_eq!(shape.start_number, 3);
    assert_eq!(shape.separator_length, 1417);
    assert_eq!(shape.separator_above_margin_hu(), 100, "구분선 위 UI 값");
    assert_eq!(shape.separator_below_margin_hu(), 200, "구분선 아래 UI 값");
    assert_eq!(shape.between_notes_margin_hu(), 300, "미주 사이 UI 값");
    assert_eq!(shape.separator_line_type, 1);
    assert_eq!(shape.separator_line_width, 2);
    assert_eq!(shape.separator_color, 0x0055_aa11);
    assert_eq!(shape.numbering, FootnoteNumbering::RestartSection);
    assert_eq!(shape.placement, FootnotePlacement::BelowText);
    assert_eq!((shape.attr >> 8) & 0x03, 1, "placement bit 8~9");
    assert_eq!((shape.attr >> 10) & 0x03, 1, "numbering bit 10~11");

    let after = doc
        .get_endnote_shape_native(0)
        .expect("get_endnote_shape_native after apply");
    let after_json: Value = serde_json::from_str(&after).expect("shape json after apply");
    assert_eq!(after_json["numberFormat"], "hangulSyllable");
    assert_eq!(after_json["separatorColor"], "#11aa55");
    assert_eq!(after_json["noteSpacing"], 300);
    assert_eq!(after_json["placement"], "sectionEnd");
}
