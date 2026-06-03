//! Task #1139: 문27 inline TAC Picture가 다음 줄까지 미리 렌더되어 중복 출력되던 회귀 방지.

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn hwpunit_to_mm(hu: i32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

fn hwpunit_to_px(hu: i32) -> f64 {
    hu as f64 * 96.0 / 7200.0
}

fn collect_small_bin5_images(node: &RenderNode, out: &mut Vec<(Option<usize>, Option<usize>)>) {
    if let RenderNodeType::Image(img) = &node.node_type {
        if img.bin_data_id == 5 && (node.bbox.width - 23.8).abs() < 0.1 {
            out.push((img.para_index, img.control_index));
        }
    }
    for child in &node.children {
        collect_small_bin5_images(child, out);
    }
}

fn collect_green_separator_lines(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if let RenderNodeType::Line(line) = &node.node_type {
        let width = (line.x2 - line.x1).abs();
        if line.style.color & 0x00ff_ffff == 0x0059_b859 && (width - 188.98).abs() < 1.0 {
            out.push((line.y1, width));
        }
    }
    for child in &node.children {
        collect_green_separator_lines(child, out);
    }
}

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

fn count_render_text_occurrences(node: &RenderNode, needle: &str) -> usize {
    let here = match &node.node_type {
        RenderNodeType::TextRun(run) => run.text.matches(needle).count(),
        RenderNodeType::FootnoteMarker(marker) => marker.text.matches(needle).count(),
        _ => 0,
    };
    here + node
        .children
        .iter()
        .map(|child| count_render_text_occurrences(child, needle))
        .sum::<usize>()
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

fn find_table_bbox(
    node: &RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para_index) && table.control_index == Some(control_index) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table_bbox(child, para_index, control_index))
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

fn find_rectangle_bbox(
    node: &RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<BoundingBox> {
    if let RenderNodeType::Rectangle(rect) = &node.node_type {
        if rect.para_index == Some(para_index) && rect.control_index == Some(control_index) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_rectangle_bbox(child, para_index, control_index))
}

fn collect_equation_bboxes_containing(node: &RenderNode, needle: &str, out: &mut Vec<BoundingBox>) {
    if let RenderNodeType::Equation(eq) = &node.node_type {
        if eq.svg_content.contains(needle) {
            out.push(node.bbox.clone());
        }
    }
    for child in &node.children {
        collect_equation_bboxes_containing(child, needle, out);
    }
}

fn find_equation_bbox(
    node: &RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<BoundingBox> {
    if let RenderNodeType::Equation(eq) = &node.node_type {
        if eq.para_index == Some(para_index) && eq.control_index == Some(control_index) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_equation_bbox(child, para_index, control_index))
}

fn count_table_nodes(node: &RenderNode, para_index: usize, control_index: usize) -> usize {
    let own = match &node.node_type {
        RenderNodeType::Table(table)
            if table.para_index == Some(para_index)
                && table.control_index == Some(control_index) =>
        {
            1
        }
        _ => 0,
    };
    own + node
        .children
        .iter()
        .map(|child| count_table_nodes(child, para_index, control_index))
        .sum::<usize>()
}

fn min_para_text_y(node: &RenderNode, para_index: usize) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::TextLine(line) if line.para_index == Some(para_index) => Some(node.bbox.y),
        RenderNodeType::TextRun(run) if run.para_index == Some(para_index) => Some(node.bbox.y),
        _ => None,
    };
    own.into_iter()
        .chain(
            node.children
                .iter()
                .filter_map(|child| min_para_text_y(child, para_index)),
        )
        .min_by(|a, b| a.partial_cmp(b).unwrap())
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

fn max_equation_bottom_in_region(
    node: &RenderNode,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::Equation(_)
            if node.bbox.x >= x_min
                && node.bbox.x < x_max
                && node.bbox.y >= y_min
                && node.bbox.y < y_max =>
        {
            Some(node.bbox.y + node.bbox.height)
        }
        _ => None,
    };
    own.into_iter()
        .chain(
            node.children.iter().filter_map(|child| {
                max_equation_bottom_in_region(child, x_min, x_max, y_min, y_max)
            }),
        )
        .max_by(|a, b| a.partial_cmp(b).unwrap())
}

fn max_para_content_bottom(node: &RenderNode, para_index: usize) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::TextLine(line) if line.para_index == Some(para_index) => {
            Some(node.bbox.y + node.bbox.height)
        }
        RenderNodeType::TextRun(run) if run.para_index == Some(para_index) => {
            Some(node.bbox.y + node.bbox.height)
        }
        RenderNodeType::Equation(eq) if eq.para_index == Some(para_index) => {
            Some(node.bbox.y + node.bbox.height)
        }
        RenderNodeType::Image(img) if img.para_index == Some(para_index) => {
            Some(node.bbox.y + node.bbox.height)
        }
        RenderNodeType::Table(table) if table.para_index == Some(para_index) => {
            Some(node.bbox.y + node.bbox.height)
        }
        _ => None,
    };
    own.into_iter()
        .chain(
            node.children
                .iter()
                .filter_map(|child| max_para_content_bottom(child, para_index)),
        )
        .max_by(|a, b| a.partial_cmp(b).unwrap())
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
fn issue_1189_2022_nov_page1_question1_marker_gap_matches_pdf() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(0).expect("page 1 render tree");

    let question1_eq = find_equation_bbox(&tree.root, 0, 4).expect("문1 수식");
    assert!(
        question1_eq.x <= 72.0,
        "문1 본문 미주 마커 앞 HWP5 placeholder가 0폭이어야 수식이 한컴/PDF처럼 문항 번호 바로 뒤에 붙음: {question1_eq:?}"
    );
    assert!(
        question1_eq.x >= 68.0,
        "문1 수식이 문항 번호와 겹치면 안 됨: {question1_eq:?}"
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
fn issue_1139_exam_2022_endnote_shape_matches_hancom_reference() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let shape = &doc.document().sections[0].section_def.endnote_shape;

    assert_eq!(shape.prefix_char, '문');
    assert_eq!(shape.suffix_char, '\u{ff09}');
    assert!((hwpunit_to_mm(shape.separator_length as i32) - 50.0).abs() < 0.05);
    assert_eq!(shape.separator_margin_top, 0);
    assert!(
        (hwpunit_to_mm(shape.note_spacing as i32) - 2.0).abs() < 0.05,
        "HWP5 binary field maps to Hancom '구분선 아래'"
    );
    assert!(
        (hwpunit_to_mm(shape.raw_unknown as i32) - 7.0).abs() < 0.05,
        "HWP5 raw_unknown preserves Hancom '미주 사이'"
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
    assert_eq!(shape.separator_margin_top, 100);
    assert_eq!(shape.note_spacing, 200, "구분선 아래 UI 값");
    assert_eq!(shape.raw_unknown, 300, "미주 사이 UI 값");
    assert_eq!(shape.separator_line_type, 1);
    assert_eq!(shape.separator_line_width, 2);
    assert_eq!(shape.separator_color, 0x0055_aa11);
    assert_eq!(shape.numbering, FootnoteNumbering::RestartSection);
    assert_eq!(shape.placement, FootnotePlacement::BelowText);

    let after = doc
        .get_endnote_shape_native(0)
        .expect("get_endnote_shape_native after apply");
    let after_json: Value = serde_json::from_str(&after).expect("shape json after apply");
    assert_eq!(after_json["numberFormat"], "hangulSyllable");
    assert_eq!(after_json["separatorColor"], "#11aa55");
    assert_eq!(after_json["noteSpacing"], 300);
    assert_eq!(after_json["placement"], "sectionEnd");
}

#[test]
fn issue_1139_exam_2022_page1_header_table_uses_page_border_spacing() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(0).expect("page 1 render tree");

    let header_table = find_table_bbox(&tree.root, 0, 0).expect("page 1 header title table");
    let expected_x = hwpunit_to_px(1984);
    let expected_y = hwpunit_to_px(1984);

    assert!(
        (header_table.x - expected_x).abs() < 0.5,
        "PDF/한컴 기준 머리말 제목 표는 paper-based 쪽 테두리 왼쪽 간격과 맞아야 함: table={header_table:?}, expected_x={expected_x}"
    );
    assert!(
        (header_table.y - expected_y).abs() < 0.5,
        "머리말 제목 표의 세로 위치는 기존 7mm 머리말 시작점을 유지해야 함: table={header_table:?}, expected_y={expected_y}"
    );
}

#[test]
fn issue_1139_endnote_spacing_reference_files_match_hancom_page_counts() {
    let below20 = std::fs::read("samples/3-09월_교육_통합_2024-구분선아래20.hwp").expect("below20");
    let below20_doc = HwpDocument::from_bytes(&below20).expect("parse below20");
    let below20_shape = &below20_doc.document().sections[0].section_def.endnote_shape;
    assert!(
        (hwpunit_to_mm(below20_shape.note_spacing as i32) - 20.0).abs() < 0.05,
        "note_spacing은 한컴 UI '구분선 아래' 값이어야 함"
    );
    assert!(
        (hwpunit_to_mm(below20_shape.raw_unknown as i32) - 7.0).abs() < 0.05,
        "raw_unknown은 한컴 UI '미주 사이' 값이어야 함"
    );
    assert_eq!(below20_doc.page_count(), 23, "구분선 아래 20mm 한컴 기준");
    let below20_page22 = below20_doc.dump_page_items(Some(21));
    let below20_page23 = below20_doc.dump_page_items(Some(22));
    assert!(
        below20_page22.contains("FullParagraph[미주]  pi=1163"),
        "구분선 아래 20mm PDF 기준 문30 제목은 22쪽 오른쪽 단에서 시작해야 함\n{below20_page22}"
    );
    assert!(
        !below20_page23.contains("FullParagraph[미주]  pi=1163"),
        "구분선 아래 20mm 23쪽은 문30 후반 꼬리만 남아야 함\n{below20_page23}"
    );

    let between20 =
        std::fs::read("samples/3-09월_교육_통합_2024-미주사이20.hwp").expect("between20");
    let between20_doc = HwpDocument::from_bytes(&between20).expect("parse between20");
    let between20_shape = &between20_doc.document().sections[0]
        .section_def
        .endnote_shape;
    assert!(
        (hwpunit_to_mm(between20_shape.note_spacing as i32) - 2.0).abs() < 0.05,
        "note_spacing은 한컴 UI '구분선 아래' 값이어야 함"
    );
    assert!(
        (hwpunit_to_mm(between20_shape.raw_unknown as i32) - 20.0).abs() < 0.05,
        "raw_unknown은 한컴 UI '미주 사이' 값이어야 함"
    );
    assert_eq!(between20_doc.page_count(), 24, "미주 사이 20mm 한컴 기준");
    let between20_page23 = between20_doc.dump_page_items(Some(22));
    let between20_page24 = between20_doc.dump_page_items(Some(23));
    assert!(
        between20_page23.contains("FullParagraph[미주]  pi=1129")
            && between20_page23.contains("FullParagraph[미주]  pi=1163"),
        "미주 사이 20mm PDF 기준 23쪽에는 문29 시작과 문30 시작이 함께 배치되어야 함\n{between20_page23}"
    );
    assert!(
        !between20_page24.contains("FullParagraph[미주]  pi=1163"),
        "미주 사이 20mm 24쪽은 문30 시작이 아니라 꼬리 문단만 남아야 함\n{between20_page24}"
    );
}

#[test]
fn issue_1139_small_inline_picture_rendered_once_per_control() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(4).expect("page 5 render tree");

    let mut images = Vec::new();
    collect_small_bin5_images(&tree.root, &mut images);
    images.sort();

    assert_eq!(
        images,
        vec![(Some(321), Some(10)), (Some(323), Some(4))],
        "문27 작은 inline Picture는 원본 컨트롤 2개만 렌더되어야 함"
    );
}

#[test]
fn issue_1139_exam_2022_page_count_matches_hancom_after_endnotes() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    assert_eq!(doc.page_count(), 23, "한컴오피스 기준 페이지 수");

    let page9 = doc.dump_page_items(Some(8));
    let page10 = doc.dump_page_items(Some(9));
    assert!(
        page9.contains("PartialParagraph  pi=522  lines=0..4"),
        "9쪽에는 문7 미주 마지막 문단의 앞부분 pi=522 lines=0..4가 남아야 함\n{page9}"
    );
    assert!(
        page9.contains("EndnoteSeparator"),
        "9쪽 미주 시작 앞에는 한컴 미주 구분선이 있어야 함\n{page9}"
    );
    assert!(
        !page9.contains("FullParagraph[미주]  pi=523"),
        "한컴오피스 기준 문8 미주 pi=523은 9쪽에 들어가면 안 됨\n{page9}"
    );
    assert!(
        page10.contains("PartialParagraph  pi=522  lines=4..5"),
        "한컴오피스 기준 문7 미주 마지막 수식 줄은 10쪽 첫 줄로 넘어가야 함\n{page10}"
    );
    assert!(
        page10.contains("FullParagraph[미주]  pi=523"),
        "한컴오피스 기준 문8 미주 pi=523은 10쪽에서 시작해야 함\n{page10}"
    );
    assert!(
        page10.contains("FullParagraph[미주]  pi=557"),
        "한컴오피스 기준 문11 미주는 10쪽 오른쪽 단에서 시작해야 함\n{page10}"
    );
    let page10_col1 = page10.find("  단 1").expect("page10 second column");
    let page10_q11 = page10
        .find("FullParagraph[미주]  pi=557")
        .expect("page10 question 11");
    assert!(
        page10_q11 > page10_col1,
        "문11(pi=557)이 10쪽 왼쪽 단 하단에 남으면 미주 사이가 깨지고 하단 overflow가 발생함\n{page10}"
    );
}

#[test]
fn issue_1139_page17_endnote_question30_starts_on_right_column() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page17 = doc.dump_page_items(Some(16));
    let page18 = doc.dump_page_items(Some(17));

    assert!(
        page17.contains("FullParagraph[미주]  pi=928"),
        "한컴오피스 기준 문30 첫 문단(pi=928)은 17쪽 우측 단 하단에서 시작해야 함\n{page17}"
    );
    assert!(
        page17.contains("FullParagraph[미주]  pi=900")
            && page17.contains("FullParagraph[미주]  pi=901"),
        "한컴오피스 기준 문29 시작은 17쪽 좌측 단 하단에 남아야 함\n{page17}"
    );
    assert!(
        page17.contains("FullParagraph[미주]  pi=929")
            && page17.contains("FullParagraph[미주]  pi=930"),
        "한컴오피스 기준 17쪽 우측 단에는 문30 풀이 본문 일부(pi=929, pi=930)도 이어져야 함\n{page17}"
    );
    assert!(
        !page18.contains("FullParagraph[미주]  pi=928")
            && !page18.contains("FullParagraph[미주]  pi=929")
            && !page18.contains("FullParagraph[미주]  pi=930"),
        "문30 앞부분(pi=928..930)이 18쪽으로 이월되면 17쪽 하단 배치가 한컴 기준보다 일찍 끊김\n{page18}"
    );
    assert!(
        page17.contains("PartialParagraph  pi=931  lines=0..4")
            && page18.contains("PartialParagraph  pi=931  lines=4..9")
            && !page17.contains("FullParagraph[미주]  pi=931")
            && !page18.contains("FullParagraph[미주]  pi=931"),
        "문30 본문은 17/18쪽에서 줄 단위로 이어져야 함\n{page17}\n{page18}"
    );
}

#[test]
fn issue_1139_page17_question30_followup_lines_do_not_overlap() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(16).expect("page 17 render tree");

    let title_y = min_para_text_y(&tree.root, 928).expect("문30 title");
    let condition_y = min_para_text_y(&tree.root, 929).expect("문30 condition line");
    let either_y = min_para_text_y(&tree.root, 930).expect("문30 n(A) line");
    let case_y = min_para_text_y(&tree.root, 931).expect("문30 case split");

    assert!(
        condition_y > title_y + 10.0,
        "문30 제목과 조건 줄이 겹치면 안 됨: title_y={title_y}, condition_y={condition_y}"
    );
    assert!(
        either_y > condition_y + 10.0,
        "문30 n(A)=2 또는 n(A)=3 줄이 조건 줄과 겹치면 안 됨: condition_y={condition_y}, either_y={either_y}"
    );
    assert!(
        case_y > either_y + 10.0,
        "문30 (i) 줄이 직전 줄과 겹치면 안 됨: either_y={either_y}, case_y={case_y}"
    );
}

#[test]
fn issue_1139_endnote_virtual_paragraph_selection_rects_are_available() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let line_info = doc
        .get_line_info(0, 868, 4)
        .unwrap_or_else(|e| panic!("미주 가상 문단 줄 정보 조회 실패: {e:?}"));
    let line_info: Value = serde_json::from_str(&line_info).expect("line info json");
    assert!(
        line_info["lineCount"].as_u64().unwrap_or(0) > 0,
        "미주 가상 문단도 줄 정보를 반환해야 함: {line_info}"
    );

    let cursor = doc
        .get_cursor_rect(0, 868, 4)
        .unwrap_or_else(|e| panic!("미주 가상 문단 커서 조회 실패: {e:?}"));
    let cursor: Value = serde_json::from_str(&cursor).expect("cursor rect json");
    assert_eq!(
        cursor["pageIndex"].as_u64(),
        Some(15),
        "문26 미주 가상 문단은 16쪽에서 커서 좌표를 찾아야 함: {cursor}"
    );

    let rects = doc
        .get_selection_rects(0, 868, 0, 868, 8)
        .unwrap_or_else(|e| panic!("미주 가상 문단 선택 사각형 조회 실패: {e:?}"));
    let rects: Value = serde_json::from_str(&rects).expect("selection rects json");
    let rects = rects.as_array().expect("selection rect array");
    assert!(
        rects.iter().any(|rect| {
            rect["pageIndex"].as_u64() == Some(15) && rect["width"].as_f64().unwrap_or(0.0) > 0.0
        }),
        "드래그 선택 하이라이트용 사각형이 16쪽 미주 문단에서 생성되어야 함: {rects:?}"
    );
}

#[test]
fn issue_1139_endnote_virtual_paragraph_para_shape_api_uses_source_note() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let props = doc
        .get_para_properties_at(0, 868)
        .unwrap_or_else(|e| panic!("미주 가상 문단 문단 모양 조회 실패: {e:?}"));
    let props: Value = serde_json::from_str(&props).expect("para props json");
    assert!(
        props["paraShapeId"].as_u64().is_some(),
        "미주 가상 문단 문단 모양이 조회되어야 함: {props}"
    );

    doc.apply_para_format(0, 868, r#"{"alignment":"center"}"#)
        .unwrap_or_else(|e| panic!("미주 가상 문단 문단 모양 적용 실패: {e:?}"));

    let after = doc
        .get_para_properties_at(0, 868)
        .unwrap_or_else(|e| panic!("미주 가상 문단 문단 모양 재조회 실패: {e:?}"));
    let after: Value = serde_json::from_str(&after).expect("para props json");
    assert_eq!(
        after["alignment"].as_str(),
        Some("center"),
        "미주 가상 문단 문단 모양 적용은 원본 Endnote 문단에 반영되어야 함: {after}"
    );
}

#[test]
fn issue_1139_page19_question29_starts_on_right_column() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page19 = doc.dump_page_items(Some(18));
    let page20 = doc.dump_page_items(Some(19));

    assert!(
        page19.contains("PartialParagraph  pi=992  lines=1..3"),
        "한컴오피스 기준 pi=992의 reset 이후 줄은 19쪽 우측 단으로 이어져야 함\n{page19}"
    );
    assert!(
        page19.contains("FullParagraph[미주]  pi=995"),
        "한컴오피스 기준 문29(pi=995)는 19쪽 우측 단에서 시작해야 함\n{page19}"
    );
    assert!(
        !page20.contains("FullParagraph[미주]  pi=995"),
        "문29 시작이 20쪽으로 밀리면 19쪽 우측 단이 한컴보다 비어 보임\n{page20}"
    );
}

#[test]
fn issue_1139_page20_starts_after_question29_tail() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page19 = doc.dump_page_items(Some(18));
    let page20 = doc.dump_page_items(Some(19));

    assert!(
        page19.contains("FullParagraph[미주]  pi=1020")
            && page19.contains("FullParagraph[미주]  pi=1021"),
        "PDF 기준 19쪽 오른쪽 단 하단에는 문29 풀이의 마지막 계산과 설명(pi=1020/1021)이 남아야 함\n{page19}"
    );
    assert!(
        !page20.contains("FullParagraph[미주]  pi=1020")
            && !page20.contains("FullParagraph[미주]  pi=1021"),
        "pi=1020/1021이 20쪽으로 밀리면 20쪽 시작이 PDF 기준보다 앞당겨짐\n{page20}"
    );
    assert!(
        page20.contains("FullParagraph[미주]  pi=1022")
            && page20.contains("FullParagraph[미주]  pi=1087"),
        "20쪽은 PDF 기준처럼 g'(2) 계산 이후부터 시작해 문27 제목까지 이어져야 함\n{page20}"
    );
}

#[test]
fn issue_1139_page23_split_endnote_empty_line_picture_is_rendered() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(22).expect("page 23 render tree");

    let picture = find_image_bbox(&tree.root, 1175, 20)
        .expect("문30) pi=1175 line 10 빈 줄 TAC Picture가 23쪽에 렌더되어야 함");

    assert!(
        picture.y < 140.0 && picture.width > 250.0 && picture.height > 220.0,
        "PDF 기준 23쪽 상단 그래프가 빠지거나 뒤쪽에 밀리면 안 됨: {picture:?}"
    );
}

#[test]
fn issue_1139_page22_question29_intro_moves_to_previous_page() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page21 = doc.dump_page_items(Some(20));
    let page22 = doc.dump_page_items(Some(21));

    assert!(
        page21.contains("FullParagraph[미주]  pi=1129")
            && page21.contains("FullParagraph[미주]  pi=1130"),
        "한컴오피스 기준 21쪽 하단에는 문29 제목과 [출제의도] 문단이 남아야 함\n{page21}"
    );
    assert!(
        !page22.contains("FullParagraph[미주]  pi=1129")
            && !page22.contains("FullParagraph[미주]  pi=1130"),
        "문29 제목/출제의도가 22쪽 첫머리에 남으면 22쪽 렌더링이 한컴보다 늦게 시작함\n{page22}"
    );
    assert!(
        page22.contains("Shape          pi=1131 ci=0  그림 tac=true"),
        "한컴오피스 기준 22쪽은 문29의 큰 구 그림(pi=1131)부터 시작해야 함\n{page22}"
    );
    assert!(
        page22.contains("Table          pi=1169 ci=0"),
        "한컴오피스 기준 문30 그래프 표(pi=1169)는 22쪽 우측 단에 렌더되어야 함\n{page22}"
    );
    assert!(
        page22.contains("PartialParagraph  pi=1175  lines=0..10"),
        "PDF 기준 22쪽 끝에는 문30 (i) 풀이의 마지막 텍스트 줄까지 남아야 함\n{page22}"
    );

    let tree = doc.build_page_render_tree(21).expect("page 22 render tree");
    let graph_table = find_table_bbox(&tree.root, 1169, 0).expect("문30 그래프 표");
    assert!(
        graph_table.width > 200.0 && graph_table.height > 170.0,
        "문30 그래프 표 bbox가 너무 작거나 누락됨: {:?}",
        graph_table
    );
}

#[test]
fn issue_1139_page23_question30_picture_line_is_rendered() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page23 = doc.dump_page_items(Some(22));
    assert!(
        page23.contains("PartialParagraph  pi=1175  lines=10..13"),
        "PDF 기준 23쪽은 문30 (ii) 그림 줄부터 시작해야 함\n{page23}"
    );

    let tree = doc.build_page_render_tree(22).expect("page 23 render tree");
    assert!(
        !render_tree_contains_text(&tree.root, "호이다"),
        "문30 (i)의 마지막 텍스트 줄은 22쪽에 남고 23쪽 첫머리에 반복되면 안 됨"
    );
    let picture = find_image_bbox(&tree.root, 1175, 20).expect("문30 (ii) 시작 그림");
    assert!(
        picture.width > 250.0 && picture.height > 220.0 && picture.y < 160.0,
        "23쪽 시작 그림 bbox가 PDF 기준 위치/크기에서 벗어남: {:?}",
        picture
    );
}

#[test]
fn issue_1209_2022_sep_page13_question19_square_picture_wraps_following_text() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(12).expect("page 13 render tree");

    let graph = find_image_bbox(&tree.root, 728, 0).expect("문19 그래프");
    let answer_line = find_text_line_bbox(&tree.root, 729, 0).expect("문19 그래프 옆 첫 문단");
    let explanation_line =
        find_text_line_bbox(&tree.root, 730, 0).expect("문19 그래프 옆 설명 첫 줄");
    let tail_formula_bottom = max_equation_bottom_in_region(
        &tree.root,
        answer_line.x - 1.0,
        graph.x,
        answer_line.y - 40.0,
        answer_line.y,
    )
    .expect("문19 f(2) 꼬리 수식");

    assert!(
        answer_line.y < graph.y + graph.height && explanation_line.y < graph.y + graph.height,
        "검증 대상 문단은 그래프 높이 범위 안에서 둘러싸기 배치되어야 함: graph={graph:?}, answer={answer_line:?}, explanation={explanation_line:?}"
    );
    assert!(
        answer_line.y >= tail_formula_bottom + 1.0,
        "문19 그래프 anchor 줄의 TAC 수식 높이를 예약해야 다음 문단과 겹치지 않음: formula_bottom={tail_formula_bottom}, answer={answer_line:?}"
    );
    assert!(
        answer_line.x + answer_line.width <= graph.x + 0.5,
        "문19 그래프 직후 문단은 한컴/PDF처럼 그래프 왼쪽 좁은 영역 안에 배치되어야 함: graph={graph:?}, line={answer_line:?}"
    );
    assert!(
        explanation_line.x + explanation_line.width <= graph.x + 0.5,
        "문19 설명 첫 줄이 그래프 영역을 침범하면 문단 겹침이 재발함: graph={graph:?}, line={explanation_line:?}"
    );
}

#[test]
fn issue_1209_2022_page8_question29_square_picture_starts_at_wrap_line() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(7).expect("page 8 render tree");

    let picture = find_image_bbox(&tree.root, 429, 21).expect("문29 어울림 그림");
    let first_full_line = find_text_line_bbox(&tree.root, 429, 0).expect("문29 첫 줄");
    let first_wrap_line = find_text_line_bbox(&tree.root, 429, 6).expect("문29 그림 옆 첫 좁은 줄");

    assert!(
        first_full_line.y + first_full_line.height <= picture.y + 1.0,
        "어울림 그림은 위쪽 full-width 본문 줄을 침범하면 안 됨: picture={picture:?}, first_line={first_full_line:?}"
    );
    assert!(
        (picture.y - first_wrap_line.y).abs() <= 2.0,
        "어울림 그림 상단은 HWP LINE_SEG가 처음 좁아지는 줄에 맞아야 함: picture={picture:?}, wrap_line={first_wrap_line:?}"
    );
    assert!(
        first_wrap_line.x + first_wrap_line.width <= picture.x + 1.0,
        "어울림 본문 줄은 그림 왼쪽 좁은 영역을 넘으면 안 됨: picture={picture:?}, wrap_line={first_wrap_line:?}"
    );
}

#[test]
fn issue_1245_2022_page7_square_pictures_use_relative_line_vpos() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(6).expect("page 7 render tree");

    let question25_picture = find_image_bbox(&tree.root, 386, 11).expect("문25 타원 그림");
    let question25_wrap_line =
        find_text_line_bbox(&tree.root, 386, 3).expect("문25 그림 옆 첫 좁은 줄");
    assert!(
        (question25_picture.y - question25_wrap_line.y).abs() <= 2.0,
        "문25 어울림 그림은 누적 LINE_SEG vpos를 중복 적용하지 않고 좁아지는 줄에 붙어야 함: picture={question25_picture:?}, wrap_line={question25_wrap_line:?}"
    );
    assert!(
        question25_picture.y + question25_picture.height < 920.0,
        "문25 그림이 페이지 하단 밖으로 밀리면 안 됨: picture={question25_picture:?}"
    );

    let question28_picture = find_image_bbox(&tree.root, 420, 9).expect("문28 포물선 그림");
    let question28_wrap_line =
        find_text_line_bbox(&tree.root, 420, 3).expect("문28 그림 옆 첫 좁은 줄");
    assert!(
        (question28_picture.y - question28_wrap_line.y).abs() <= 2.0,
        "문28 어울림 그림도 문단 첫 줄 대비 상대 LINE_SEG vpos로 배치되어야 함: picture={question28_picture:?}, wrap_line={question28_wrap_line:?}"
    );
}

#[test]
fn issue_1139_2023_page4_question26_square_table_uses_anchor_line() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2023.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(3).expect("page 4 render tree");

    let table = find_table_bbox(&tree.root, 258, 5).expect("문26 표준정규분포표");
    let first_text_y = min_para_text_y(&tree.root, 258).expect("문26 본문 텍스트");

    assert!(
        table.y > first_text_y + 70.0 && table.y < first_text_y + 120.0,
        "문26 Square wrap 표는 문단 첫 줄이 아니라 LineSeg가 좁아지는 후반 줄에 붙어야 함: table={table:?}, first_text_y={first_text_y}"
    );
}

#[test]
fn issue_1245_2023_page4_question26_endnote_marker_not_duplicated() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2023.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(3).expect("page 4 render tree");

    let count = count_render_text_occurrences(&tree.root, "문26");
    assert_eq!(
        count, 1,
        "미주 선두 번호는 일반 TextRun으로 한 번만 렌더되어야 하며 위첨자 마커로 중복되면 안 됨"
    );
}

#[test]
fn issue_1139_2023_pages12_13_endnote_boundary_matches_pdf() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2023.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page12 = doc.dump_page_items(Some(11));
    let page13 = doc.dump_page_items(Some(12));

    let page12_col1 = page12.find("  단 1").expect("page 12 second column");
    let q14_title = page12
        .find("FullParagraph[미주]  pi=611")
        .expect("page 12 question 14 title");
    let q14_graph_host = page12
        .find("FullParagraph[미주]  pi=613")
        .expect("page 12 question 14 graph host");
    assert!(
        q14_title < page12_col1,
        "PDF 기준 12쪽 왼쪽 단 하단에서 문14 제목이 시작해야 함\n{page12}"
    );
    assert!(
        q14_graph_host > page12_col1,
        "PDF 기준 12쪽 오른쪽 단은 문14 그래프 영역부터 이어져야 함\n{page12}"
    );
    assert!(
        page12.contains("FullParagraph[미주]  pi=635")
            && page12.contains("FullParagraph[미주]  pi=636")
            && !page12.contains("Shape          pi=637 ci=0"),
        "PDF 기준 문14 tail(pi=635/636)은 12쪽에 남고 그래프(pi=637)는 13쪽에서 시작해야 함\n{page12}"
    );
    assert!(
        !page13.contains("FullParagraph[미주]  pi=635")
            && !page13.contains("FullParagraph[미주]  pi=636"),
        "13쪽 첫머리에 이전 tail(pi=635/636)이 남으면 PDF보다 시작점이 늦음\n{page13}"
    );

    let graph = page13
        .find("Shape          pi=637 ci=0  그림 tac=true")
        .expect("page 13 starts with question 14 graph");
    let q15_title = page13
        .find("FullParagraph[미주]  pi=638")
        .expect("page 13 question 15 title");
    assert!(
        graph < q15_title,
        "PDF 기준 13쪽은 문15 제목 전에 문14 그래프가 먼저 보여야 함\n{page13}"
    );
}

#[test]
fn issue_1189_2023_page19_question29_tail_matches_pdf() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2023.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page19 = doc.dump_page_items(Some(18));
    assert!(
        page19.contains("PartialParagraph  pi=935  lines=0..2")
            && page19.contains("PartialParagraph  pi=935  lines=2..3")
            && page19.contains("FullParagraph[미주]  pi=946")
            && page19.contains("FullParagraph[미주]  pi=952")
            && page19.contains("PartialParagraph  pi=953  lines=0..1"),
        "PDF 기준 19쪽 우측 단에는 문29 제목, 첫 그림, 그림 아래 첫 문단이 함께 남고, 앞쪽 pi=935 분배는 기존 위치를 유지해야 함\n{page19}"
    );

    let tree = doc.build_page_render_tree(18).expect("page 19 render tree");
    let q29_title_y = min_para_text_y(&tree.root, 946).expect("문29 제목");
    let first_case_picture = find_image_bbox(&tree.root, 951, 0).expect("문29 첫 경우 그림");
    let picture_tail_y = min_para_text_y(&tree.root, 952).expect("문29 그림 아래 본문");

    assert!(
        q29_title_y < 725.0,
        "문29 제목이 PDF 기준보다 아래로 밀리면 뒤쪽 그림/본문이 페이지 하단에서 잘림: y={q29_title_y}"
    );
    assert!(
        first_case_picture.y < 880.0 && first_case_picture.y + first_case_picture.height < 1075.0,
        "문29 첫 그림이 PDF 기준 위치보다 아래로 밀리면 안 됨: {first_case_picture:?}"
    );
    assert!(
        picture_tail_y < 1080.0,
        "문29 그림 아래 첫 문단은 19쪽 하단 안쪽에 보여야 함: y={picture_tail_y}"
    );
}

#[test]
fn issue_1189_2022_nov_page17_internal_rewind_keeps_formula_tail_on_next_page() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page14 = doc.dump_page_items(Some(13));
    let page16 = doc.dump_page_items(Some(15));
    let page17 = doc.dump_page_items(Some(16));
    assert!(
        page14.contains("FullParagraph[미주]  pi=632")
            && page14.contains("FullParagraph[미주]  pi=650")
            && page14.contains("FullParagraph[미주]  pi=669")
            && page14.contains("PartialParagraph  pi=671  lines=0..2"),
        "PDF 기준 14쪽은 문22~문27 시작 흐름이 같은 페이지에 유지되어야 함\n{page14}"
    );
    assert!(
        page16.contains("PartialParagraph  pi=786  lines=0..1")
            && !page16.contains("PartialParagraph  pi=786  lines=0..2"),
        "한컴/PDF 기준 16쪽 하단에는 문26 수식 문단의 첫 줄만 남아야 함\n{page16}"
    );
    assert!(
        page17.contains("PartialParagraph  pi=786  lines=1..5")
            && page17.contains("FullParagraph[미주]  pi=787")
            && page17.contains("FullParagraph[미주]  pi=801"),
        "한컴/PDF 기준 17쪽은 문26 수식 나머지 줄 뒤에 문27/문28이 이어져야 함\n{page17}"
    );

    let tree = doc.build_page_render_tree(16).expect("page 17 render tree");
    let mut cqrt_lines = Vec::new();
    collect_equation_bboxes_containing(&tree.root, "CQRT", &mut cqrt_lines);
    let cqrt_line = cqrt_lines
        .into_iter()
        .find(|bbox| bbox.x > 400.0 && bbox.y > 440.0 && bbox.y < 500.0)
        .expect("문28 CQRT line");
    let mut g_theta_candidates = Vec::new();
    collect_equation_bboxes_containing(&tree.root, ">g</text>", &mut g_theta_candidates);
    let g_theta = g_theta_candidates
        .into_iter()
        .find(|bbox| {
            (bbox.y - cqrt_line.y).abs() < 2.0
                && bbox.x < cqrt_line.x
                && cqrt_line.x - bbox.x < 40.0
        })
        .expect("문28 g(theta)");
    assert!(
        (g_theta.y - cqrt_line.y).abs() < 2.0 && cqrt_line.x > g_theta.x,
        "문28의 g(theta)와 =□CQRT-△CST는 같은 줄에서 좌→우로 이어져야 함: g={g_theta:?}, cqrt={cqrt_line:?}"
    );
}

#[test]
fn issue_1209_2022_nov_page17_split_endnote_titles_keep_hancom_gap() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let tree = doc.build_page_render_tree(16).expect("page 17 render tree");
    let question27_y = min_para_text_y(&tree.root, 787).expect("문27 title");
    let question28_y = min_para_text_y(&tree.root, 801).expect("문28 title");

    assert!(
        (240.0..256.0).contains(&question27_y),
        "17쪽 왼쪽 단 문27 제목은 앞쪽에서 이어진 미주 본문 뒤 한컴/PDF 간격을 유지해야 함: y={question27_y}"
    );
    assert!(
        (204.0..216.0).contains(&question28_y),
        "17쪽 오른쪽 단 문28 제목도 앞쪽에서 이어진 미주 본문 뒤 한컴/PDF 간격을 유지해야 함: y={question28_y}"
    );
}

#[test]
fn issue_1209_2022_nov_page17_question29_keeps_hancom_gap_after_full_para() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page17 = doc.dump_page_items(Some(16));
    assert!(
        page17.contains("FullParagraph[미주]  pi=812"),
        "17쪽 오른쪽 단에는 문29 제목이 보여야 함\n{page17}"
    );

    let tree = doc.build_page_render_tree(16).expect("page 17 render tree");
    let question29_y = min_para_text_y(&tree.root, 812).expect("문29 title");

    assert!(
        (806.0..818.0).contains(&question29_y),
        "17쪽 문29 제목은 직전 full 미주 문단 뒤 PDF/한컴 기준 미주 사이 간격을 유지해야 함: y={question29_y}"
    );
}

#[test]
fn issue_1209_2022_nov_page14_question22_keeps_hancom_endnote_gap() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page14 = doc.dump_page_items(Some(13));
    assert!(
        page14.contains("FullParagraph[미주]  pi=631")
            && page14.contains("FullParagraph[미주]  pi=632"),
        "14쪽 문22 앞뒤 미주 문단이 같은 왼쪽 단 흐름에 있어야 함\n{page14}"
    );

    let tree = doc.build_page_render_tree(13).expect("page 14 render tree");
    let prev_bottom = max_para_content_bottom(&tree.root, 631).expect("문21 tail content");
    let question22_y = min_para_text_y(&tree.root, 632).expect("문22 title");
    let gap = question22_y - prev_bottom;

    assert!(
        (22.0..34.0).contains(&gap),
        "14쪽 문22 시작 전에는 한컴/PDF 기준 미주 사이 간격이 보존되어야 함: prev_bottom={prev_bottom}, question22_y={question22_y}, gap={gap}"
    );

    let question22_tail_bottom = max_para_content_bottom(&tree.root, 643).expect("문22 tail");
    let question23_y = min_para_text_y(&tree.root, 644).expect("문23 title");
    let question23_gap = question23_y - question22_tail_bottom;

    assert!(
        (20.0..34.0).contains(&question23_gap),
        "빈 spacer 뒤 문23 제목도 한컴/PDF 기준 미주 사이 간격을 유지해야 함: tail_bottom={question22_tail_bottom}, question23_y={question23_y}, gap={question23_gap}"
    );
}

#[test]
fn issue_1189_2022_oct_page17_endnote_drag_selection_covers_equation_tail_lines() {
    let bytes = std::fs::read("samples/3-10월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(16).expect("page 17 render tree");

    let rects = doc
        .get_selection_rects(0, 915, 0, 921, 3)
        .unwrap_or_else(|e| panic!("17쪽 문27 미주 드래그 선택 사각형 조회 실패: {e:?}"));
    let rects: Value = serde_json::from_str(&rects).expect("selection rects json");
    let rects = rects.as_array().expect("selection rect array");

    for para_idx in 915..=921 {
        let para_y = min_para_text_y(&tree.root, para_idx).expect("문27 미주 문단 text y");
        assert!(
            rects.iter().any(|rect| {
                rect["pageIndex"].as_u64() == Some(16)
                    && (rect["y"].as_f64().unwrap_or_default() - para_y).abs() < 0.8
                    && rect["width"].as_f64().unwrap_or_default() > 1.0
            }),
            "한컴오피스처럼 문27 미주 드래그 선택이 수식 꼬리 문단까지 연속으로 덮어야 함: para_idx={para_idx}, para_y={para_y}, rects={rects:?}"
        );
    }
}

#[test]
fn issue_1261_2022_oct_page5_question28_choices_stay_below_condition_box() {
    let bytes = std::fs::read("samples/3-10월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(4).expect("page 5 render tree");

    let condition_box = find_rectangle_bbox(&tree.root, 306, 0).expect("문28 조건 박스");
    let first_choice_line = find_text_line_bbox(&tree.root, 306, 1).expect("문28 ①②③ 선택지 줄");
    let box_bottom = condition_box.y + condition_box.height;

    assert!(
        first_choice_line.y > box_bottom + 2.0,
        "문28 선택지 줄은 조건 박스 아래에서 시작해야 하며 박스 내부 문장을 덮으면 안 됨: box={condition_box:?}, choice={first_choice_line:?}"
    );
}

#[test]
fn issue_1261_2024_sep_page10_question8_stays_below_previous_equation() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2024-미주사이20.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(9).expect("page 10 render tree");

    let previous_equation_bottom =
        max_para_content_bottom(&tree.root, 522).expect("문7 마지막 수식 문단");
    let question8_y = min_para_text_y(&tree.root, 523).expect("문8 제목");
    let between_notes_gap = question8_y - previous_equation_bottom;

    assert!(
        (70.0..82.0).contains(&between_notes_gap),
        "문8 제목은 직전 문7 마지막 수식 하단 뒤에 미주 사이 20mm 공통 간격을 유지해야 함: prev_bottom={previous_equation_bottom}, q8_y={question8_y}, gap={between_notes_gap}"
    );
}

#[test]
fn issue_1189_2022_oct_page11_endnote_question_gaps_match_pdf() {
    let bytes = std::fs::read("samples/3-10월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(10).expect("page 11 render tree");

    let question18_y = min_para_text_y(&tree.root, 569).expect("문18 title");
    let question19_y = min_para_text_y(&tree.root, 574).expect("문19 title");
    let question20_y = min_para_text_y(&tree.root, 582).expect("문20 title");
    let gap18_to_19 = question19_y - question18_y;
    let gap19_to_20 = question20_y - question19_y;

    assert!(
        (205.0..235.0).contains(&gap18_to_19),
        "11쪽 문18→문19 미주 간격이 PDF보다 넓어지면 안 됨: q18={question18_y}, q19={question19_y}, gap={gap18_to_19}"
    );
    assert!(
        (180.0..210.0).contains(&gap19_to_20),
        "11쪽 문19→문20 미주 간격도 한컴/PDF 흐름을 유지해야 함: q19={question19_y}, q20={question20_y}, gap={gap19_to_20}"
    );
}

#[test]
fn issue_1189_2022_nov_pages10_12_rewind_tail_and_equation_scale_match_pdf() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page10 = doc.dump_page_items(Some(9));
    let page11 = doc.dump_page_items(Some(10));
    let page12 = doc.dump_page_items(Some(11));
    assert!(
        page10.contains("FullParagraph[미주]  pi=475")
            && page10.contains("FullParagraph[미주]  pi=476"),
        "PDF 기준 10쪽 하단/우측 시작 미주 흐름을 유지해야 함\n{page10}"
    );
    let page10_tree = doc.build_page_render_tree(9).expect("page 10 render tree");
    let question6_tail_bottom =
        max_para_content_bottom(&page10_tree.root, 475).expect("문6 꼬리 수식");
    assert!(
        question6_tail_bottom <= 1092.8,
        "10쪽 문6 꼬리 수식은 본문 하단을 넘겨 문단끼리 겹치면 안 됨: bottom={question6_tail_bottom}"
    );
    assert!(
        page11.contains("PartialParagraph  pi=553  lines=0..8")
            && !page11.contains("FullParagraph[미주]  pi=553"),
        "문14 tail은 11쪽에서 내부 vpos 리셋 직전까지만 렌더되어야 함\n{page11}"
    );
    assert!(
        page12.contains("PartialParagraph  pi=553  lines=8..11")
            && page12.contains("Shape          pi=554 ci=0  그림 tac=true")
            && page12.contains("FullParagraph[미주]  pi=555"),
        "12쪽은 문14 tail 텍스트 뒤 그래프와 문15가 이어져야 함\n{page12}"
    );

    let svg = doc.render_page_svg_native(11).expect("page 12 svg");
    assert!(
        !svg.contains(">SEARROW</text>") && !svg.contains(">NEARROW</text>"),
        "문19 변화표의 HWP 대문자 화살표 토큰이 문자열 그대로 렌더되면 안 됨"
    );
    assert!(
        svg.contains(">↘</text>") && svg.contains(">↗</text>"),
        "문19 변화표의 감소/증가 방향은 한컴처럼 대각 화살표 기호로 렌더되어야 함"
    );
    let bad_eq_text = svg.find(">배수</text>").expect("문15 배수 수식");
    let group_start = svg[..bad_eq_text]
        .rfind("<g transform=")
        .expect("배수 수식 group");
    let group_end = bad_eq_text
        + svg[bad_eq_text..]
            .find("</g>")
            .expect("배수 수식 group end");
    let group = &svg[group_start..group_end];
    assert!(
        group.contains(",1.0000)"),
        "수식 bbox 높이로 Y축을 확대하면 12쪽 하단 주석 수식이 찌그러짐\n{group}"
    );
}

#[test]
fn issue_1256_2022_sep_page10_question12_keeps_between_notes_gap() {
    // [Task #1256] 문12 제목 위에는 미주 사이(between-notes, 7mm) 간격이 있어야 한다.
    // 한컴 PDF(pdf/3-09월_교육_통합_2022.pdf 10쪽) 기준 문11 풀이("k=9") 다음 빈 줄이
    // 들어가고 문12) 가 시작한다. 종전 #1209 는 저장 LINE_SEG 의 backtrack 위치
    // (cram, ~398px)를 단언했으나 이는 PDF 갭과 모순이라 #1256 에서 갭 포함 위치로 정정.
    // 수식 중앙정렬·꼬리 간격(아래 나머지 단언)은 #1209 그대로 유지된다.
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(9).expect("page 10 render tree");

    let question12_y = min_para_text_y(&tree.root, 567).expect("문12 title");
    let question12_body_y = min_para_text_y(&tree.root, 568).expect("문12 body");
    let question12_tail_y = min_para_text_y(&tree.root, 573).expect("문12 따라서");
    let question13_y = min_para_text_y(&tree.root, 575).expect("문13 title");
    let mut ah_formulas = Vec::new();
    collect_equation_bboxes_containing(&tree.root, ">AH</text>", &mut ah_formulas);
    let question12_formula = ah_formulas
        .into_iter()
        .filter(|bbox| bbox.y > question12_tail_y && bbox.y < question13_y)
        .min_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        .expect("문12 첫 수식");

    assert!(
        (410.0..=426.0).contains(&question12_y),
        "10쪽 문12 제목은 한컴 PDF처럼 between-notes(7mm) 갭 포함 위치여야 함(#1256): q12_y={question12_y}"
    );
    assert!(
        (12.0..=30.0).contains(&(question12_body_y - question12_y)),
        "문12 제목과 본문 첫 줄 사이 간격은 한컴/PDF 흐름을 유지해야 함: title={question12_y}, body={question12_body_y}"
    );
    assert!(
        question13_y <= 724.0,
        "문12 수식 블록이 아래로 밀려 문13을 늦게 시작시키면 안 됨: q13_y={question13_y}"
    );
    assert!(
        (398.0..=408.0).contains(&question12_formula.x),
        "문12 수식-only 문단은 배분 정렬 오프셋으로 중앙에 밀리면 안 됨: x={}",
        question12_formula.x
    );
    assert!(
        question12_formula.y - question12_tail_y <= 20.0,
        "문12 '따라서'와 수식-only 문단 사이 간격은 한컴/PDF처럼 촘촘해야 함: tail_y={question12_tail_y}, formula_y={}",
        question12_formula.y
    );
}

#[test]
fn issue_1139_page13_question20_table_is_not_duplicated() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page13 = doc.dump_page_items(Some(12));
    assert!(
        page13.contains("FullParagraph[미주]  pi=739"),
        "문20 변화표 host 문단은 13쪽에 있어야 함\n{page13}"
    );
    assert!(
        !page13.contains("Table          pi=739 ci=0"),
        "문20 변화표 TAC table은 host paragraph 안에서 렌더되므로 별도 Table PageItem으로 중복 배치되면 안 됨\n{page13}"
    );

    let tree = doc.build_page_render_tree(12).expect("page 13 render tree");
    assert_eq!(
        count_table_nodes(&tree.root, 739, 0),
        1,
        "문20 변화표 pi=739 ci=0은 13쪽에 한 번만 렌더되어야 함"
    );
}

#[test]
fn issue_1139_page9_endnote_table_does_not_overlap_header() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(8).expect("page 9 render tree");

    let answer_table = find_table_bbox(&tree.root, 466, 0).expect("page 9 answer table");
    let first_endnote_y = min_para_text_y(&tree.root, 468).expect("page 9 first endnote");
    let table_bottom = answer_table.y + answer_table.height;
    assert!(
        first_endnote_y >= table_bottom + 8.0,
        "9쪽 첫 미주가 상단 정답표 아래에서 시작해야 함: first_endnote_y={first_endnote_y}, table_bottom={table_bottom}, table={answer_table:?}"
    );
}

#[test]
fn issue_1139_page9_endnote_shape_textbox_is_rendered() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(8).expect("page 9 render tree");

    assert!(
        render_tree_contains_text(&tree.root, "다른 풀이"),
        "9쪽 문7 미주 내부 TAC Shape 그룹의 글상자 텍스트가 렌더되어야 함"
    );

    let mut lines = Vec::new();
    collect_green_separator_lines(&tree.root, &mut lines);
    assert!(
        !lines.is_empty(),
        "9쪽 미주 시작에는 50mm 녹색 구분선이 렌더되어야 함"
    );
}

#[test]
fn issue_1139_page9_endnote_shape_properties_resolve_virtual_para_index() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let layout = doc
        .get_page_control_layout_native(8)
        .expect("page 9 control layout");
    let parsed: Value = serde_json::from_str(&layout).expect("control layout json");

    let group = parsed["controls"]
        .as_array()
        .expect("controls array")
        .iter()
        .find(|ctrl| {
            ctrl["type"] == "group"
                && ctrl["paraIdx"].as_u64() == Some(518)
                && ctrl["controlIdx"].as_u64() == Some(0)
        })
        .expect("문7 [다른 풀이] group shape");

    assert_eq!(group["secIdx"].as_u64(), Some(0));

    let props = doc
        .get_shape_properties_native(0, 518, 0)
        .expect("미주 가상 문단 Shape 속성 조회");
    let props: Value = serde_json::from_str(&props).expect("shape props json");
    assert!(
        props["width"].as_u64().unwrap_or(0) > 0,
        "미주 내부 Shape 속성이 실제 값으로 조회되어야 함: {props}"
    );
}

#[test]
fn issue_1139_page12_endnote_shape_picture_properties_resolve_virtual_para_index() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let layout = doc
        .get_page_control_layout_native(11)
        .expect("page 12 control layout");
    let parsed: Value = serde_json::from_str(&layout).expect("control layout json");

    for (para_idx, control_idx) in [(651, 1), (652, 1)] {
        let image = parsed["controls"]
            .as_array()
            .expect("controls array")
            .iter()
            .find(|ctrl| {
                ctrl["type"] == "image"
                    && ctrl["paraIdx"].as_u64() == Some(para_idx)
                    && ctrl["controlIdx"].as_u64() == Some(control_idx)
            })
            .unwrap_or_else(|| {
                panic!("12쪽 그래프 image control 누락: para={para_idx}, ctrl={control_idx}")
            });

        assert_eq!(image["secIdx"].as_u64(), Some(0));

        let props = doc
            .get_picture_properties_native(0, para_idx as usize, control_idx as usize)
            .expect("미주 가상 문단 ShapeObject::Picture 속성 조회");
        let props: Value = serde_json::from_str(&props).expect("picture props json");
        assert_eq!(
            props["treatAsChar"].as_bool(),
            Some(true),
            "12쪽 그래프는 한컴 기준 '글자처럼 취급'이어야 함: {props}"
        );
        assert!(
            props["width"].as_u64().unwrap_or(0) > 0 && props["height"].as_u64().unwrap_or(0) > 0,
            "ShapeObject::Picture 속성이 실제 크기로 조회되어야 함: {props}"
        );
        assert_eq!(
            (
                props["cropLeft"].as_i64(),
                props["cropTop"].as_i64(),
                props["cropRight"].as_i64(),
                props["cropBottom"].as_i64(),
            ),
            (Some(0), Some(0), Some(0), Some(0)),
            "한컴 그림 탭 기준 원본 전체 crop rect는 자르기 0mm로 표시되어야 함: {props}"
        );
    }
}

#[test]
fn issue_1139_page12_question15_keeps_hancom_endnote_gap() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(11).expect("page 12 render tree");

    let prev_bottom = max_para_content_bottom(&tree.root, 664).expect("문14 final content");
    let question15_y = min_para_text_y(&tree.root, 665).expect("문15 title");
    let gap = question15_y - prev_bottom;

    assert!(
        (24.0..32.0).contains(&gap),
        "12쪽 우측 하단 문15 시작 전에는 한컴 미주 사이 7mm에 해당하는 gap이 보존되어야 함: prev_bottom={prev_bottom}, question15_y={question15_y}, gap={gap}"
    );
}

#[test]
fn issue_1139_endnote_equation_exposes_note_ref_and_properties() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let mut found = None;
    for page in 8..18 {
        let layout = doc
            .get_page_control_layout_native(page)
            .unwrap_or_else(|e| panic!("page {} control layout: {e}", page + 1));
        let parsed: Value = serde_json::from_str(&layout).expect("control layout json");
        if let Some(eq) = parsed["controls"]
            .as_array()
            .expect("controls array")
            .iter()
            .find(|ctrl| ctrl["type"] == "equation" && ctrl["noteRef"]["kind"] == "endnote")
        {
            found = Some(eq.clone());
            break;
        }
    }

    let eq = found.expect("미주 내부 수식은 noteRef와 함께 control layout에 노출되어야 함");
    let note = &eq["noteRef"];
    let props = doc
        .get_note_equation_properties_native(
            note["kind"].as_str().unwrap(),
            note["sectionIdx"].as_u64().unwrap() as usize,
            note["paraIdx"].as_u64().unwrap() as usize,
            note["controlIdx"].as_u64().unwrap() as usize,
            note["noteParaIdx"].as_u64().unwrap() as usize,
            note["innerControlIdx"].as_u64().unwrap() as usize,
        )
        .expect("미주 내부 수식 속성 조회");
    let props: Value = serde_json::from_str(&props).expect("equation props json");

    assert!(
        props["script"].as_str().is_some_and(|s| !s.is_empty()),
        "미주 내부 수식 script가 조회되어야 함: {props}"
    );
    assert!(
        props["fontSize"].as_u64().unwrap_or(0) > 0,
        "미주 내부 수식 fontSize가 조회되어야 함: {props}"
    );
    assert!(
        props["width"].as_u64().unwrap_or(0) > 0 && props["height"].as_u64().unwrap_or(0) > 0,
        "미주 내부 수식 기본 탭의 너비/높이가 실제 값으로 조회되어야 함: {props}"
    );
    assert_eq!(
        props["treatAsChar"].as_bool(),
        Some(true),
        "미주 내부 수식은 한컴 기준 '글자처럼 취급'이어야 함: {props}"
    );
    assert!(
        props["outerMarginLeft"].is_number()
            && props["outerMarginTop"].is_number()
            && props["outerMarginRight"].is_number()
            && props["outerMarginBottom"].is_number(),
        "수식 속성 여백/캡션 탭의 바깥 여백 값이 조회되어야 함: {props}"
    );
    assert_eq!(
        props["hasCaption"].as_bool(),
        Some(false),
        "캡션이 없는 수식은 수식 속성 여백/캡션 탭에서 위치 없음으로 표시되어야 함: {props}"
    );
}
