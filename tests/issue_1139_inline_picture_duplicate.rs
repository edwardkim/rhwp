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

fn collect_equation_bboxes_in_region(
    node: &RenderNode,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    out: &mut Vec<BoundingBox>,
) {
    if let RenderNodeType::Equation(_) = &node.node_type {
        if node.bbox.x >= x_min
            && node.bbox.x < x_max
            && node.bbox.y >= y_min
            && node.bbox.y < y_max
        {
            out.push(node.bbox.clone());
        }
    }
    for child in &node.children {
        collect_equation_bboxes_in_region(child, x_min, x_max, y_min, y_max, out);
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

fn count_text_line_nodes(node: &RenderNode, para_index: usize) -> usize {
    let own = match &node.node_type {
        RenderNodeType::TextLine(line) if line.para_index == Some(para_index) => 1,
        _ => 0,
    };
    own + node
        .children
        .iter()
        .map(|child| count_text_line_nodes(child, para_index))
        .sum::<usize>()
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

fn max_equation_visual_bottom_in_region(
    node: &RenderNode,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::Equation(eq)
            if node.bbox.x >= x_min
                && node.bbox.x < x_max
                && node.bbox.y >= y_min
                && node.bbox.y < y_max =>
        {
            Some(node.bbox.y + eq.layout_box.height)
        }
        _ => None,
    };
    own.into_iter()
        .chain(node.children.iter().filter_map(|child| {
            max_equation_visual_bottom_in_region(child, x_min, x_max, y_min, y_max)
        }))
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
fn issue_1274_2022_nov_page11_empty_float_picture_host_has_no_phantom_overflow() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(10).expect("page 11 render tree");

    let picture = find_image_bbox(&tree.root, 537, 0).expect("문12 그래프 그림");
    assert!(
        (175.0..=190.0).contains(&picture.y),
        "빈 host 문단의 non-TAC 그림은 한컴/PDF처럼 우측 단 상단에 있어야 함: {picture:?}"
    );
    assert_eq!(
        count_text_line_nodes(&tree.root, 537),
        0,
        "빈 그림 host 문단 pi=537은 실제 Shape item으로만 렌더되어야 하며 phantom TextLine을 남기면 안 됨"
    );
    let bottom = max_para_content_bottom(&tree.root, 537).expect("pi=537 content bottom");
    assert!(
        bottom < 360.0,
        "pi=537 실제 콘텐츠 하단은 그림 bbox 하단이어야 하며 저장 vpos의 phantom line으로 페이지 밖을 가리키면 안 됨: bottom={bottom}, picture={picture:?}"
    );
}

#[test]
fn issue_1274_2022_nov_page11_partial_endnote_tail_stays_in_page_frame() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page11 = doc.dump_page_items(Some(10));
    let page12 = doc.dump_page_items(Some(11));
    assert!(
        page11.contains("PartialParagraph  pi=553  lines=0..8"),
        "한컴/PDF 기준 문14) 꼬리 첫 조각은 11쪽 끝에 남아야 함\n{page11}"
    );
    assert!(
        page12.contains("PartialParagraph  pi=553  lines=8..11"),
        "문14) 꼬리 나머지는 12쪽 첫머리에서 이어져야 함\n{page12}"
    );

    let tree = doc.build_page_render_tree(10).expect("page 11 render tree");
    let last_line = find_text_line_bbox(&tree.root, 553, 7).expect("pi=553 line 7");
    let last_line_bottom = last_line.y + last_line.height;
    assert!(
        last_line_bottom < 1113.0,
        "compact 미주 마지막 줄은 본문 하단을 조금 넘더라도 페이지 테두리 안에 남아야 함: {last_line:?}"
    );
    assert!(
        find_text_line_bbox(&tree.root, 553, 8).is_none(),
        "다음 줄까지 11쪽에 끌고 오면 12쪽 시작 분기가 한컴/PDF와 달라짐"
    );
}

/// [#1302] 다줄 미주 문단(pi=852, 분수 포함 키 큰 줄)의 마지막 줄 다음, 같은 문제(문30) 내
/// 연속 텍스트 문단(pi=853)이 컬럼 하단에서 줄간격이 좁아지면 안 된다.
/// page-path compact 미주 하단의 `page_tail_backtrack` 이 stored vpos 가 정상 한 줄 전진
/// (lh+ls)을 인코딩한 breakable 텍스트 연속에까지 발동해 trailing 줄간격(~6px)을 깎던 버그.
/// 18쪽 좌측 단: "극솟값…갖는다"(pi=852 끝줄) → "(나)를 고려하기…"(pi=853 첫줄).
#[test]
fn issue_1302_2022_nov_page18_multiline_endnote_continuation_keeps_line_spacing() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(17).expect("page 18 render tree");

    // pi=852 는 2줄(line_index 0,1) — 마지막 줄.
    let prev_last = find_text_line_bbox(&tree.root, 852, 1).expect("pi=852 마지막 줄");
    // pi=853 컬럼0 첫 줄.
    let cont_first = find_text_line_bbox(&tree.root, 853, 0).expect("pi=853 첫 줄");

    let gap = cont_first.y - prev_last.y;
    // 정상 한 줄 전진(lh+ls) ≈ 18~20px. 버그 시 lh 만(≈12~14px)으로 좁아짐.
    assert!(
        (16.0..=22.0).contains(&gap),
        "다줄 미주 문단 다음 같은 문제 연속 문단 줄간격이 trailing 누락으로 좁아지면 안 됨: \
         pi=852끝줄.y={}, pi=853첫줄.y={}, gap={gap}",
        prev_last.y,
        cont_first.y
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
    assert_eq!(shape.separator_above_margin_hu(), 0);
    assert!(
        (hwpunit_to_mm(shape.separator_below_margin_hu() as i32) - 2.0).abs() < 0.05,
        "HWP5 binary field maps to Hancom '구분선 아래'"
    );
    assert!(
        (hwpunit_to_mm(shape.between_notes_margin_hu() as i32) - 7.0).abs() < 0.05,
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

#[test]
fn issue_1139_endnote_shape_api_clears_hwp5_separator_above_fallback_slot() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp")
        .expect("below_above20");
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse below_above20");
    let before = &doc.document().sections[0].section_def.endnote_shape;
    assert_eq!(before.separator_margin_top, 0);
    assert!(
        (hwpunit_to_mm(before.separator_margin_bottom as i32) - 20.0).abs() < 0.05,
        "HWP5 조합 샘플은 fallback raw 슬롯에 구분선 위 20mm를 저장한다"
    );
    assert!(
        (hwpunit_to_mm(before.separator_above_margin_hu() as i32) - 20.0).abs() < 0.05,
        "적용 전 정규화 구분선 위는 20mm"
    );

    doc.apply_endnote_shape_native(0, r#"{"separatorMarginTop":0}"#)
        .expect("apply zero separator above");

    let after = &doc.document().sections[0].section_def.endnote_shape;
    assert_eq!(after.separator_margin_top, 0);
    assert_eq!(
        after.separator_margin_bottom, 0,
        "구분선 위를 0으로 적용하면 HWP5 fallback 슬롯도 지워야 한다"
    );
    assert_eq!(
        after.separator_above_margin_hu(),
        0,
        "UI/API 정규화 구분선 위 값도 0이어야 한다"
    );
    assert!(
        (hwpunit_to_mm(after.separator_below_margin_hu() as i32) - 20.0).abs() < 0.05,
        "구분선 아래 20mm는 그대로 유지되어야 한다"
    );

    let settings = doc
        .get_endnote_shape_native(0)
        .expect("get_endnote_shape_native");
    let settings_json: Value = serde_json::from_str(&settings).expect("shape json");
    assert_eq!(settings_json["separatorMarginTop"], 0);
    assert!(
        (hwpunit_to_mm(settings_json["separatorMarginBottom"].as_i64().unwrap() as i32) - 20.0)
            .abs()
            < 0.05,
        "API의 separatorMarginBottom은 공식 구분선 아래 값을 유지해야 한다"
    );
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
        (hwpunit_to_mm(below20_shape.separator_below_margin_hu() as i32) - 20.0).abs() < 0.05,
        "정규화된 구분선 아래 값은 한컴 UI '구분선 아래'여야 함"
    );
    assert!(
        (hwpunit_to_mm(below20_shape.between_notes_margin_hu() as i32) - 7.0).abs() < 0.05,
        "정규화된 미주 사이 값은 한컴 UI '미주 사이'여야 함"
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
        (hwpunit_to_mm(between20_shape.separator_below_margin_hu() as i32) - 2.0).abs() < 0.05,
        "정규화된 구분선 아래 값은 한컴 UI '구분선 아래'여야 함"
    );
    assert!(
        (hwpunit_to_mm(between20_shape.between_notes_margin_hu() as i32) - 20.0).abs() < 0.05,
        "정규화된 미주 사이 값은 한컴 UI '미주 사이'여야 함"
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

    let below_above20 = std::fs::read("samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp")
        .expect("below_above20");
    let below_above20_doc = HwpDocument::from_bytes(&below_above20).expect("parse below_above20");
    let below_above20_shape = &below_above20_doc.document().sections[0]
        .section_def
        .endnote_shape;
    assert!(
        (hwpunit_to_mm(below_above20_shape.separator_above_margin_hu() as i32) - 20.0).abs() < 0.05,
        "HWP5 raw separator_margin_bottom은 한컴 UI '구분선 위'로 정규화되어야 함"
    );
    assert!(
        (hwpunit_to_mm(below_above20_shape.separator_below_margin_hu() as i32) - 20.0).abs() < 0.05,
        "조합 샘플의 구분선 아래 값은 20mm여야 함"
    );
    assert!(
        (hwpunit_to_mm(below_above20_shape.between_notes_margin_hu() as i32) - 7.0).abs() < 0.05,
        "조합 샘플의 미주 사이 값은 7mm여야 함"
    );

    let below_above20_hwpx =
        std::fs::read("samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx")
            .expect("below_above20 hwpx");
    let below_above20_hwpx_doc =
        HwpDocument::from_bytes(&below_above20_hwpx).expect("parse below_above20 hwpx");
    let below_above20_hwpx_shape = &below_above20_hwpx_doc.document().sections[0]
        .section_def
        .endnote_shape;
    assert_eq!(
        below_above20_shape.separator_above_margin_hu(),
        below_above20_hwpx_shape.separator_above_margin_hu(),
        "HWP/HWPX 조합 샘플의 구분선 위 정규화 값은 같아야 함"
    );
    assert_eq!(
        below_above20_shape.separator_below_margin_hu(),
        below_above20_hwpx_shape.separator_below_margin_hu(),
        "HWP/HWPX 조합 샘플의 구분선 아래 정규화 값은 같아야 함"
    );
}

#[test]
fn issue_1293_clean_visual_sweep_targets_keep_page_counts_and_shape_profiles() {
    struct CleanTarget {
        key: &'static str,
        path: &'static str,
        pages: u32,
        separator_above_mm: f64,
        between_notes_mm: f64,
        separator_below_mm: f64,
        visible_separator: bool,
    }

    let targets = [
        CleanTarget {
            key: "2022-09",
            path: "samples/3-09월_교육_통합_2022.hwp",
            pages: 23,
            separator_above_mm: 0.0,
            between_notes_mm: 7.0,
            separator_below_mm: 2.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2023-09",
            path: "samples/3-09월_교육_통합_2023.hwp",
            pages: 20,
            separator_above_mm: 0.0,
            between_notes_mm: 7.0,
            separator_below_mm: 2.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2024-09-below20",
            path: "samples/3-09월_교육_통합_2024-구분선아래20.hwp",
            pages: 23,
            separator_above_mm: 0.0,
            between_notes_mm: 7.0,
            separator_below_mm: 20.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2024-09-between20",
            path: "samples/3-09월_교육_통합_2024-미주사이20.hwp",
            pages: 24,
            separator_above_mm: 0.0,
            between_notes_mm: 20.0,
            separator_below_mm: 2.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2022-11-practice",
            path: "samples/3-11월_실전_통합_2022.hwp",
            pages: 21,
            separator_above_mm: 0.0,
            between_notes_mm: 7.0,
            separator_below_mm: 2.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2024-11-practice-shape987",
            path: "samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp",
            pages: 21,
            separator_above_mm: 9.0,
            between_notes_mm: 8.0,
            separator_below_mm: 7.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2024-11-practice-above0-between0-below0",
            path: "samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp",
            pages: 21,
            separator_above_mm: 0.0,
            between_notes_mm: 0.0,
            separator_below_mm: 0.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2024-11-practice-above0-between7-below2",
            path: "samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래2.hwp",
            pages: 21,
            separator_above_mm: 0.0,
            between_notes_mm: 7.0,
            separator_below_mm: 2.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2024-11-practice-above0-between7-below20",
            path: "samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwp",
            pages: 21,
            separator_above_mm: 0.0,
            between_notes_mm: 7.0,
            separator_below_mm: 20.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2024-11-practice-above20-between0-below20",
            path: "samples/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwp",
            pages: 21,
            separator_above_mm: 20.0,
            between_notes_mm: 0.0,
            separator_below_mm: 20.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2024-11-practice-above20-between7-below2",
            path: "samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp",
            pages: 21,
            separator_above_mm: 20.0,
            between_notes_mm: 7.0,
            separator_below_mm: 2.0,
            visible_separator: true,
        },
        CleanTarget {
            key: "2024-11-practice-no-separator-above20-between20-below20",
            path: "samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp",
            pages: 23,
            separator_above_mm: 20.0,
            between_notes_mm: 20.0,
            separator_below_mm: 20.0,
            visible_separator: false,
        },
    ];

    for target in targets {
        let bytes = std::fs::read(target.path).unwrap_or_else(|err| {
            panic!("{} 샘플을 읽을 수 없음: {err}", target.key);
        });
        let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|err| {
            panic!("{} 샘플을 파싱할 수 없음: {err}", target.key);
        });
        assert_eq!(
            doc.page_count(),
            target.pages,
            "{} clean sweep target의 한컴/PDF 기준 page count가 바뀌면 안 됨",
            target.key
        );

        let shape = &doc.document().sections[0].section_def.endnote_shape;
        let visible_separator = shape.separator_line_type != 0 && shape.separator_line_width != 0;
        assert_eq!(
            visible_separator, target.visible_separator,
            "{} 구분선 표시 여부가 sweep 기준과 달라지면 안 됨",
            target.key
        );

        let above_mm = hwpunit_to_mm(shape.separator_above_margin_hu() as i32);
        let between_mm = hwpunit_to_mm(shape.between_notes_margin_hu() as i32);
        let below_mm = hwpunit_to_mm(shape.separator_below_margin_hu() as i32);
        assert!(
            (above_mm - target.separator_above_mm).abs() < 0.08,
            "{} 구분선 위 값이 달라짐: expected={}mm actual={}mm",
            target.key,
            target.separator_above_mm,
            above_mm
        );
        assert!(
            (between_mm - target.between_notes_mm).abs() < 0.08,
            "{} 미주 사이 값이 달라짐: expected={}mm actual={}mm",
            target.key,
            target.between_notes_mm,
            between_mm
        );
        assert!(
            (below_mm - target.separator_below_mm).abs() < 0.08,
            "{} 구분선 아래 값이 달라짐: expected={}mm actual={}mm",
            target.key,
            target.separator_below_mm,
            below_mm
        );
    }
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
fn issue_1139_endnote_virtual_paragraph_vertical_move_does_not_panic() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let moved = doc
        .move_vertical(0, 602, u32::MAX, 1, -1.0, u32::MAX, 0, 0, 0)
        .unwrap_or_else(|e| panic!("미주 가상 문단 아래 이동 실패: {e:?}"));
    let moved: Value = serde_json::from_str(&moved).expect("moveVertical json");

    assert!(
        moved["paragraphIndex"].as_u64().unwrap_or(0) >= 602,
        "미주 가상 문단의 아래 이동은 본문 문단 인덱스 공간으로 되감기면 안 됨: {moved}"
    );
    assert!(
        moved["pageIndex"].as_u64().is_some(),
        "미주 가상 문단 아래 이동도 커서 좌표를 반환해야 함: {moved}"
    );
}

#[test]
fn issue_1139_endnote_virtual_paragraph_right_arrow_moves_within_text() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let moved = doc.navigate_next_editable_wasm(0, 602, 0, 1, "[]");
    let moved: Value = serde_json::from_str(&moved).expect("navigateNextEditable json");

    assert_eq!(
        moved["type"].as_str(),
        Some("text"),
        "미주 가상 문단 오른쪽 이동이 문서 경계로 처리되면 안 됨: {moved}"
    );
    assert_eq!(
        moved["para"].as_u64(),
        Some(602),
        "미주 가상 문단 오른쪽 이동은 같은 렌더 문단에서 시작해야 함: {moved}"
    );
    assert!(
        moved["charOffset"].as_u64().unwrap_or(0) > 0,
        "미주 가상 문단 오른쪽 이동은 다음 편집 위치로 전진해야 함: {moved}"
    );
}

#[test]
fn issue_1139_endnote_equation_cursor_rects_do_not_rewind_to_line_start() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let cursor_x = |para: u32, offset: u32| -> f64 {
        let rect = doc
            .get_cursor_rect(0, para, offset)
            .unwrap_or_else(|e| panic!("cursor rect para={para} offset={offset}: {e:?}"));
        let rect: Value = serde_json::from_str(&rect).expect("cursor rect json");
        assert_eq!(
            rect["pageIndex"].as_u64(),
            Some(8),
            "9쪽 미주 커서여야 함: para={para} offset={offset} rect={rect}"
        );
        rect["x"].as_f64().expect("cursor rect x")
    };

    for (para, end_offset) in [(471u32, 11u32), (474, 8), (479, 7)] {
        let xs: Vec<f64> = (0..=end_offset)
            .map(|offset| cursor_x(para, offset))
            .collect();
        assert!(
            xs.windows(2).all(|pair| pair[1] + 0.1 >= pair[0]),
            "미주 수식 문단 커서 x가 오른쪽 이동 중 줄 시작으로 되감기면 안 됨: para={para} xs={xs:?}"
        );
        assert!(
            xs.last().copied().unwrap_or(0.0) > xs.first().copied().unwrap_or(0.0) + 20.0,
            "미주 수식 문단 오른쪽 이동은 실제로 수식/텍스트 뒤쪽으로 전진해야 함: para={para} xs={xs:?}"
        );
        for offset in 0..end_offset {
            let moved = doc.navigate_next_editable_wasm(0, para, offset, 1, "[]");
            let moved: Value = serde_json::from_str(&moved).expect("navigate json");
            assert_eq!(
                moved["type"].as_str(),
                Some("text"),
                "미주 수식 문단 오른쪽 이동은 boundary가 아니어야 함: para={para} offset={offset} moved={moved}"
            );
        }
    }
}

#[test]
fn issue_1139_endnote_equation_right_arrow_skips_duplicate_boundary_stop() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let char_offset_after_right = |para: u32, offset: u32| -> u64 {
        let moved = doc.navigate_next_editable_wasm(0, para, offset, 1, "[]");
        let moved: Value = serde_json::from_str(&moved).expect("navigate json");
        assert_eq!(
            moved["type"].as_str(),
            Some("text"),
            "미주 수식 문단 오른쪽 이동은 boundary가 아니어야 함: para={para} offset={offset} moved={moved}"
        );
        assert_eq!(
            moved["para"].as_u64(),
            Some(para as u64),
            "미주 수식 문단 오른쪽 이동은 같은 문단 안에서 진행되어야 함: para={para} offset={offset} moved={moved}"
        );
        moved["charOffset"].as_u64().expect("charOffset")
    };

    assert_eq!(
        char_offset_after_right(471, 1),
        2,
        "텍스트가 섞인 문단에서는 두 번째 수식 자체를 건너뛰면 안 됨"
    );
    assert_eq!(
        char_offset_after_right(479, 1),
        3,
        "수식만 연속된 문단에서는 이전 수식 끝과 다음 수식 시작의 같은 x 경계를 한 번 더 밟으면 안 됨"
    );
    assert_eq!(char_offset_after_right(479, 3), 5);
    assert_eq!(char_offset_after_right(479, 5), 7);
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
fn issue_1261_2024_sep_page10_question12_tail_stays_inside_column() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2024-미주사이20.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(9).expect("page 10 render tree");

    let question10_y = min_para_text_y(&tree.root, 550).expect("문10 제목");
    let question11_y = min_para_text_y(&tree.root, 557).expect("문11 제목");
    let question12_y = min_para_text_y(&tree.root, 567).expect("문12 제목");
    let question12_tail_bottom = max_para_content_bottom(&tree.root, 569).expect("문12 꼬리");

    assert!(
        (398.0..408.0).contains(&question10_y),
        "문10 제목은 한컴 PDF bbox(약 402.8px)와 맞아야 함: q10={question10_y}"
    );
    assert!(
        (614.0..624.0).contains(&question11_y),
        "문11 제목은 한컴 PDF bbox(약 618.5px)와 맞아야 함: q11={question11_y}"
    );
    assert!(
        (991.0..1001.0).contains(&question12_y),
        "문12 제목은 한컴 PDF bbox(약 995.6px)와 맞아야 함: q12={question12_y}"
    );
    assert!(
        question12_tail_bottom < 1092.5,
        "문12 꼬리는 10쪽 오른쪽 단 하단 안에 남아야 함: bottom={question12_tail_bottom}"
    );
}

#[test]
fn issue_1284_2024_between20_page13_question_flow_matches_pdf() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2024-미주사이20.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page12 = doc.dump_page_items(Some(11));
    let page13 = doc.dump_page_items(Some(12));
    let page14 = doc.dump_page_items(Some(13));
    assert!(
        !page12.contains("FullParagraph[미주]  pi=662"),
        "PDF 기준 page 12 하단에는 [알짜 풀이] 다음 ㄱ. [참] tail이 frame 밖에 남으면 안 됨\n{page12}"
    );
    let q14_tail = page13
        .find("FullParagraph[미주]  pi=662")
        .expect("page 13 starts with question 14 tail");
    let q15_title = page13
        .find("FullParagraph[미주]  pi=665")
        .expect("page 13 question 15 title");
    assert!(
        q14_tail < q15_title,
        "PDF 기준 page 13 첫머리의 문14 tail 뒤에 문15가 이어져야 함\n{page13}"
    );
    assert!(
        !page13.contains("FullParagraph[미주]  pi=713"),
        "PDF 기준 page 13에는 문18 제목 tail만 남고 첫 풀이 수식은 다음 쪽으로 넘어가야 함\n{page13}"
    );
    assert!(
        page14.contains("FullParagraph[미주]  pi=713"),
        "문18 첫 풀이 수식은 page 14 첫머리에서 이어져야 함\n{page14}"
    );

    let tree = doc.build_page_render_tree(12).expect("page 13 render tree");
    let question15_y = min_para_text_y(&tree.root, 665).expect("문15 제목");
    let question16_y = min_para_text_y(&tree.root, 696).expect("문16 제목");
    let question17_y = min_para_text_y(&tree.root, 708).expect("문17 제목");
    let question18_y = min_para_text_y(&tree.root, 712).expect("문18 제목");

    assert!(
        (615.0..=635.0).contains(&question15_y),
        "문15 제목은 PDF bbox(약 624.5px) 근처에서 시작해야 함: y={question15_y}"
    );
    assert!(
        (588.0..=608.0).contains(&question16_y),
        "문16 제목은 PDF bbox(약 597.7px) 근처에서 시작해야 함: y={question16_y}"
    );
    assert!(
        (890.0..=910.0).contains(&question17_y),
        "문17 제목은 PDF bbox(약 900.2px) 근처에서 시작해야 함: y={question17_y}"
    );
    assert!(
        (1056.0..=1082.0).contains(&question18_y),
        "문18 제목은 PDF bbox(약 1070.5px) 근처에서 drift 허용 범위 안에 있어야 함: y={question18_y}"
    );
}

#[test]
fn issue_1284_2024_between20_page17_question26_tail_starts_next_column() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2024-미주사이20.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page17 = doc.dump_page_items(Some(16));
    assert!(
        !page17.contains("PartialParagraph  pi=872  lines=0..2"),
        "PDF 기준 page 17 왼쪽 단 하단에는 문26 (ⅲ) 첫 두 줄이 남지 않아야 함\n{page17}"
    );
    let right_col = page17.find("  단 1").expect("page 17 right column");
    assert!(
        page17[right_col..].contains("FullParagraph[미주]  pi=872")
            || page17[right_col..].contains("PartialParagraph  pi=872  lines=0..6"),
        "문26 (ⅲ) 문단은 한컴/PDF처럼 page 17 오른쪽 단 첫머리에서 시작해야 함\n{page17}"
    );

    let tree = doc.build_page_render_tree(16).expect("page 17 render tree");
    let q26_tail = find_text_line_bbox(&tree.root, 872, 0).expect("문26 (ⅲ) 첫 줄");
    assert!(
        q26_tail.x > 390.0 && (84.0..=112.0).contains(&q26_tail.y),
        "문26 (ⅲ) 첫 줄은 PDF 기준 page 17 오른쪽 단 상단에서 시작해야 함: {:?}",
        q26_tail
    );
}

#[test]
fn issue_1284_2024_between20_page15_question22_graph_stays_in_frame() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2024-미주사이20.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(14).expect("page 15 render tree");

    let graph_intro = find_text_line_bbox(&tree.root, 794, 0).expect("문22 그래프 안내 문장");
    let graph = find_image_bbox(&tree.root, 795, 0).expect("문22 첫 그래프");
    let graph_bottom = graph.y + graph.height;

    assert!(
        (872.0..=904.0).contains(&graph_intro.y),
        "문22 그래프 안내 문장은 PDF page 15 왼쪽 단 하단 bbox(약 y=889px) 근처여야 함: {:?}",
        graph_intro
    );
    assert!(
        graph.y >= graph_intro.y + graph_intro.height - 2.0 && (895.0..=930.0).contains(&graph.y),
        "문22 첫 그래프는 안내 문장 직후 PDF 위치에서 시작해야 함: intro={:?}, graph={:?}",
        graph_intro,
        graph
    );
    assert!(
        graph_bottom <= 1092.3,
        "문22 첫 그래프가 frame 아래로 잘리면 안 됨: graph={:?}, bottom={graph_bottom}",
        graph
    );
}

#[test]
fn issue_1284_2023_sep_page14_question23_title_tail_matches_pdf() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2023.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page14 = doc.dump_page_items(Some(13));
    let page15 = doc.dump_page_items(Some(14));
    let q23_title = page14
        .find("FullParagraph[미주]  pi=759")
        .expect("page 14 question 23 title tail");
    let page14_col1 = page14.find("  단 1").expect("page 14 second column");
    assert!(
        q23_title > page14_col1,
        "PDF 기준 문23 제목은 page 14 오른쪽 단 하단에 남아야 함\n{page14}"
    );
    assert!(
        !page15.contains("FullParagraph[미주]  pi=759"),
        "문23 제목을 page 15 상단으로 넘기면 한컴/PDF보다 한 줄씩 밀림\n{page15}"
    );

    let page14_tree = doc.build_page_render_tree(13).expect("page 14 render tree");
    let q23_title_bbox = find_text_line_bbox(&page14_tree.root, 759, 0).expect("문23 제목");
    assert!(
        q23_title_bbox.x > 390.0 && (1058.0..=1084.0).contains(&q23_title_bbox.y),
        "문23 제목은 PDF page 14 오른쪽 단 하단 bbox(약 y=1069px)와 맞아야 함: {:?}",
        q23_title_bbox
    );

    let page15_tree = doc.build_page_render_tree(14).expect("page 15 render tree");
    let q23_body = find_text_line_bbox(&page15_tree.root, 760, 0).expect("문23 본문 첫 줄");
    let q24_title_y = min_para_text_y(&page15_tree.root, 762).expect("문24 제목");
    assert!(
        q23_body.x < 80.0 && (84.0..=110.0).contains(&q23_body.y),
        "문23 본문은 page 15 왼쪽 단 상단에서 이어져야 함: {:?}",
        q23_body
    );
    assert!(
        (168.0..=188.0).contains(&q24_title_y),
        "문23 제목 tail을 남기면 문24 제목은 PDF bbox(약 178.5px) 근처로 당겨져야 함: y={q24_title_y}"
    );
}

#[test]
fn issue_1284_2023_sep_page16_question27_title_matches_pdf_tail() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2023.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(15).expect("page 16 render tree");

    let q23_title = find_text_line_bbox(&tree.root, 812, 0).expect("문23 제목");
    let q24_title = find_text_line_bbox(&tree.root, 814, 0).expect("문24 제목");
    let q25_title = find_text_line_bbox(&tree.root, 820, 0).expect("문25 제목");
    let q25_tail_bottom = max_para_content_bottom(&tree.root, 822).expect("문25 꼬리 수식");
    let q26_title = find_text_line_bbox(&tree.root, 823, 0).expect("문26 제목");
    let q26_tail_bottom = max_para_content_bottom(&tree.root, 830).expect("문26 꼬리 수식");
    let q27_title = find_text_line_bbox(&tree.root, 831, 0).expect("문27 제목");
    let q27_first = find_text_line_bbox(&tree.root, 832, 0).expect("문27 첫 본문");

    assert!(
        (140.0..=155.0).contains(&q23_title.y),
        "문23 제목은 PDF page 16 왼쪽 단 상단 bbox(약 y=147px)에 있어야 함: {:?}",
        q23_title
    );
    assert!(
        (248.0..=266.0).contains(&q24_title.y),
        "문24 제목은 PDF page 16 왼쪽 단 중단 bbox(약 y=255px)에 있어야 함: {:?}",
        q24_title
    );
    assert!(
        (490.0..=520.0).contains(&q25_title.y),
        "문25 제목은 PDF page 16 왼쪽 단 중단 bbox(약 y=496px) 근처여야 함: {:?}",
        q25_title
    );
    assert!(
        (616.0..=638.0).contains(&q26_title.y),
        "문26 제목은 PDF page 16 왼쪽 단 중하단 bbox(약 y=625px)에 있어야 함: {:?}",
        q26_title
    );
    assert!(
        q25_tail_bottom <= q26_title.y - 6.0,
        "문25 마지막 수식이 문26 제목과 겹치면 안 됨: 문25 bottom={:.1}, 문26={:?}",
        q25_tail_bottom,
        q26_title
    );
    assert!(
        q26_tail_bottom <= q27_title.y - 6.0,
        "문26 마지막 수식이 문27 제목과 겹치면 안 됨: 문26 bottom={:.1}, 문27={:?}",
        q26_tail_bottom,
        q27_title
    );
    assert!(
        q27_title.x < 80.0 && (992.0..=1010.0).contains(&q27_title.y),
        "문27 제목은 PDF page 16 왼쪽 단 하단 bbox(약 y=1001px)에 있어야 함: {:?}",
        q27_title
    );
    assert!(
        q27_first.y > q27_title.y,
        "문27 첫 본문은 제목 아래에서 이어져야 함: title={:?}, first={:?}",
        q27_title,
        q27_first
    );
}

#[test]
fn issue_1284_2023_sep_page20_question30_title_stays_in_left_tail() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2023.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page20 = doc.dump_page_items(Some(19));
    let page20_col1 = page20.find("  단 1").expect("page 20 second column");
    let q30_title = page20
        .find("FullParagraph[미주]  pi=972")
        .expect("page 20 question 30 title tail");
    let q30_intro = page20
        .find("FullParagraph[미주]  pi=973")
        .expect("page 20 question 30 first body line");
    let q30_equation = page20
        .find("FullParagraph[미주]  pi=975")
        .expect("page 20 question 30 equation line");
    let q30_continuation = page20
        .find("FullParagraph[미주]  pi=976")
        .expect("page 20 question 30 right-column continuation");
    assert!(
        q30_title < page20_col1
            && q30_intro < page20_col1
            && q30_equation > page20_col1
            && q30_continuation > page20_col1,
        "현재 sweep 기준 문30은 page 20 왼쪽 단 하단에 제목과 첫 풀이 2줄을 남기고 오른쪽 단 수식으로 이어져야 함\n{page20}"
    );

    let tree = doc.build_page_render_tree(19).expect("page 20 render tree");
    let q30_title_bbox = find_text_line_bbox(&tree.root, 972, 0).expect("문30 제목");
    let q30_intro_bbox = find_text_line_bbox(&tree.root, 973, 0).expect("문30 본문 첫 줄");
    let q30_condition_bbox = find_text_line_bbox(&tree.root, 974, 0).expect("문30 조건 줄");
    let q30_equation_bbox = find_text_line_bbox(&tree.root, 975, 0).expect("문30 식 줄");
    let q30_continuation_bbox =
        find_text_line_bbox(&tree.root, 976, 0).expect("문30 오른쪽 단 이어짐");
    assert!(
        q30_title_bbox.x < 80.0 && (1010.0..=1034.0).contains(&q30_title_bbox.y),
        "문30 제목은 PDF page 20 왼쪽 단 하단 bbox(약 y=1022px)에 있어야 함: {:?}",
        q30_title_bbox
    );
    assert!(
        q30_intro_bbox.x < 80.0 && (1030.0..=1050.0).contains(&q30_intro_bbox.y),
        "문30 첫 본문 줄은 PDF처럼 page 20 왼쪽 단 하단 제목 바로 아래에 있어야 함: {:?}",
        q30_intro_bbox
    );
    assert!(
        q30_condition_bbox.x < 80.0 && (1050.0..=1070.0).contains(&q30_condition_bbox.y),
        "문30 조건 줄은 PDF처럼 왼쪽 단 하단에 이어져야 함: {:?}",
        q30_condition_bbox
    );
    assert!(
        q30_equation_bbox.x > 390.0 && (84.0..=110.0).contains(&q30_equation_bbox.y),
        "문30 식 줄은 현재 sweep처럼 오른쪽 단 상단에서 이어져야 함: {:?}",
        q30_equation_bbox
    );
    assert!(
        q30_continuation_bbox.x > 390.0 && (84.0..=116.0).contains(&q30_continuation_bbox.y),
        "문30 다음 줄은 page 20 오른쪽 단 상단에서 이어져야 함: {:?}",
        q30_continuation_bbox
    );
}

#[test]
fn issue_1284_2022_sep_page17_question27_starts_at_pdf_top() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page16 = doc.dump_page_items(Some(15));
    let page17 = doc.dump_page_items(Some(16));
    assert!(
        !page16.contains("FullParagraph[미주]  pi=875"),
        "문27 제목만 page 16 오른쪽 단 하단에 남기면 다음 빈/TAC 식이 frame 밖으로 넘침\n{page16}"
    );
    let q27_title = page17
        .find("FullParagraph[미주]  pi=875")
        .expect("page 17 question 27 title");
    let q28_title = page17
        .find("FullParagraph[미주]  pi=887")
        .expect("page 17 question 28 title");
    assert!(
        q27_title < q28_title,
        "PDF 기준 page 17 왼쪽 단은 문27 뒤 문28로 이어져야 함\n{page17}"
    );

    let tree = doc.build_page_render_tree(16).expect("page 17 render tree");
    let q27_title_bbox = find_text_line_bbox(&tree.root, 875, 0).expect("문27 제목");
    let q28_title_y = min_para_text_y(&tree.root, 887).expect("문28 제목");
    let q29_title_y = min_para_text_y(&tree.root, 900).expect("문29 제목");
    assert!(
        q27_title_bbox.x < 80.0 && (84.0..=108.0).contains(&q27_title_bbox.y),
        "문27 제목은 PDF page 17 왼쪽 단 상단 bbox(약 y=90.7px)에서 시작해야 함: {:?}",
        q27_title_bbox
    );
    assert!(
        (454.0..=480.0).contains(&q28_title_y),
        "문28 제목은 문27 전체가 page17에서 시작한 뒤 PDF bbox(약 y=467.3px) 근처여야 함: y={q28_title_y}"
    );
    assert!(
        (1012.0..=1054.0).contains(&q29_title_y),
        "문29 제목은 문27이 page17 상단으로 복귀한 뒤 PDF page17 하단 흐름 범위에 있어야 함: y={q29_title_y}"
    );
}

#[test]
fn issue_1284_2024_between20_page18_late_question_titles_match_pdf() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2024-미주사이20.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page17 = doc.dump_page_items(Some(16));
    let page18 = doc.dump_page_items(Some(17));
    assert!(
        page17.contains("PartialParagraph  pi=894  lines=0..4")
            && !page17.contains("FullParagraph[미주]  pi=894"),
        "PDF 기준 page17 오른쪽 단 하단에는 문28 (ⅰ) 풀이의 마지막 줄 직전까지 남아야 함\n{page17}"
    );
    assert!(
        page18.contains("PartialParagraph  pi=894  lines=4..5")
            && !page18.contains("FullParagraph[미주]  pi=894"),
        "PDF 기준 page18 왼쪽 단은 문28 (ⅰ) 풀이 마지막 줄부터 시작해야 함\n{page18}"
    );

    let tree = doc.build_page_render_tree(17).expect("page 18 render tree");
    let question29_y = min_para_text_y(&tree.root, 900).expect("문29 제목");
    let question30_y = min_para_text_y(&tree.root, 928).expect("문30 제목");
    let question23_y = min_para_text_y(&tree.root, 935).expect("문23 제목");

    assert!(
        (398.0..=414.0).contains(&question29_y),
        "page18 왼쪽 단 문29 제목은 PDF bbox(약 404.2px) 근처여야 함: y={question29_y}"
    );
    assert!(
        (330.0..=352.0).contains(&question30_y),
        "page18 오른쪽 단 문30 제목은 현재 sweep 기준 위치 근처여야 함: y={question30_y}"
    );
    assert!(
        (852.0..=878.0).contains(&question23_y),
        "page18 오른쪽 단 다음 회차 문23 제목은 q30 tail 뒤 현재 sweep 기준 위치여야 함: y={question23_y}"
    );
}

#[test]
fn issue_1284_2024_between20_page19_question24_continues_from_pdf_top() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2024-미주사이20.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page18 = doc.dump_page_items(Some(17));
    let page19 = doc.dump_page_items(Some(18));
    assert!(
        !page18.contains("FullParagraph[미주]  pi=937"),
        "PDF 기준 page 18 오른쪽 단에는 문23까지만 남고 문24는 frame 밖에 남으면 안 됨\n{page18}"
    );
    let q24_title = page19
        .find("FullParagraph[미주]  pi=937")
        .expect("page 19 question 24 title");
    let q25_title = page19
        .find("FullParagraph[미주]  pi=940")
        .expect("page 19 question 25 title");
    let q26_title = page19
        .find("FullParagraph[미주]  pi=945")
        .expect("page 19 question 26 title");
    assert!(
        q24_title < q25_title && q25_title < q26_title,
        "PDF 기준 page 19 왼쪽 단은 문24 -> 문25 -> 문26 순서로 이어져야 함\n{page19}"
    );

    let tree = doc.build_page_render_tree(18).expect("page 19 render tree");
    let question24_y = min_para_text_y(&tree.root, 937).expect("문24 제목");
    let question25_y = min_para_text_y(&tree.root, 940).expect("문25 제목");
    let question26_y = min_para_text_y(&tree.root, 945).expect("문26 제목");
    let question27_y = min_para_text_y(&tree.root, 956).expect("문27 제목");
    let question28_y = min_para_text_y(&tree.root, 975).expect("문28 제목");

    assert!(
        (84.0..=100.0).contains(&question24_y),
        "문24 제목은 PDF page 19 상단(약 90.7px)에서 시작해야 함: y={question24_y}"
    );
    assert!(
        (300.0..=320.0).contains(&question25_y),
        "문25 제목은 PDF bbox(약 307.8px) 근처에서 시작해야 함: y={question25_y}"
    );
    assert!(
        (570.0..=590.0).contains(&question26_y),
        "문26 제목은 PDF bbox(약 579.6px) 근처에서 시작해야 함: y={question26_y}"
    );
    assert!(
        (980.0..=1004.0).contains(&question27_y),
        "문27 제목은 PDF bbox(약 990.5px) 근처에서 시작해야 함: y={question27_y}"
    );
    assert!(
        (814.0..=836.0).contains(&question28_y),
        "문28 제목은 현재 sweep 기준 위치 근처에서 시작해야 함: y={question28_y}"
    );
}

#[test]
fn issue_1293_2024_zero_endnote_spacing_page19_tail_moves_to_page20() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp")
        .expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    assert_eq!(doc.page_count(), 21, "한컴/PDF 기준 페이지 수");

    let page18 = doc.dump_page_items(Some(17));
    let page19 = doc.dump_page_items(Some(18));
    let page20 = doc.dump_page_items(Some(19));

    assert!(
        page18.contains("FullParagraph[미주]  pi=907")
            && page18.contains("FullParagraph[미주]  pi=908"),
        "PDF 기준 page 18 오른쪽 단에는 문25 tail pi=907/908까지 남아야 함\n{page18}"
    );
    assert!(
        !page18.contains("FullParagraph[미주]  pi=909"),
        "문25의 다음 tail pi=909가 page 18에 남으면 page 19 시작 흐름이 한컴/PDF보다 빨라짐\n{page18}"
    );
    assert!(
        page19.contains("FullParagraph[미주]  pi=909")
            && page19.contains("FullParagraph[미주]  pi=955")
            && page19.contains("PartialParagraph  pi=960  lines=0..3"),
        "PDF 기준 page 19는 문25 tail pi=909에서 시작해 문29 pi=960 앞 3줄까지 이어져야 함\n{page19}"
    );
    assert!(
        !page19.contains("FullParagraph[미주]  pi=961"),
        "문29 다음 수식 문단 pi=961이 page 19 오른쪽 단 바닥에 남으면 frame 아래로 잘림\n{page19}"
    );

    let p20_tail = page20
        .find("PartialParagraph  pi=960  lines=3..4")
        .expect("page 20 starts with 문29 final text tail");
    let p20_formula = page20
        .find("FullParagraph[미주]  pi=961")
        .expect("page 20 문29 final formula");
    let p20_picture = page20
        .find("FullParagraph[미주]  pi=962")
        .expect("page 20 문29 picture");
    let p20_q30 = page20
        .find("FullParagraph[미주]  pi=976")
        .expect("page 20 문30 title");
    assert!(
        p20_tail < p20_formula && p20_formula < p20_picture && p20_picture < p20_q30,
        "PDF 기준 page 20은 문29 마지막 텍스트 줄 -> 마지막 수식 -> 그림 -> 문30 순으로 이어져야 함\n{page20}"
    );

    let tree = doc.build_page_render_tree(19).expect("page 20 render tree");
    let final_tail_bbox = find_text_line_bbox(&tree.root, 960, 3).expect("문29 마지막 텍스트 줄");
    assert!(
        final_tail_bbox.x < 80.0 && (84.0..=104.0).contains(&final_tail_bbox.y),
        "문29 마지막 텍스트 한 줄은 PDF처럼 page 20 왼쪽 단 상단에서 시작해야 함: {:?}",
        final_tail_bbox
    );
    let final_formula_bbox = find_text_line_bbox(&tree.root, 961, 0).expect("문29 마지막 수식 줄");
    assert!(
        final_formula_bbox.y > final_tail_bbox.y + final_tail_bbox.height,
        "문29 마지막 수식은 독립 tail 텍스트 줄 아래에서 시작해야 함: tail={:?}, formula={:?}",
        final_tail_bbox,
        final_formula_bbox
    );
}

#[test]
fn issue_1293_2024_zero_endnote_spacing_page10_question7_intro_stays_left_tail() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp")
        .expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page10 = doc.dump_page_items(Some(9));
    let page11 = doc.dump_page_items(Some(10));
    let q7_title = page10
        .find("FullParagraph[미주]  pi=480")
        .expect("page 10 문7 제목");
    let q7_intro = page10
        .find("FullParagraph[미주]  pi=481")
        .expect("page 10 문7 첫 설명 줄");
    let q7_formula = page10
        .find("FullParagraph[미주]  pi=482")
        .expect("page 10 문7 첫 수식 줄");
    let q7_rewind_formula = page10
        .find("FullParagraph[미주]  pi=483")
        .expect("page 10 오른쪽 단 문7 큰 수식 시작");
    assert!(
        q7_title < q7_intro && q7_intro < q7_formula && q7_formula < q7_rewind_formula,
        "PDF 기준 page 10은 문7 제목 뒤 첫 설명/수식 줄이 왼쪽 단 tail로 남고, 되감김 수식부터 오른쪽 단에서 이어져야 함\n{page10}"
    );
    assert!(
        page10.contains("FullParagraph[미주]  pi=512"),
        "문7 도입 tail이 왼쪽 단에 남아야 오른쪽 단 마지막에 문11 제목이 한컴/PDF처럼 올라옴\n{page10}"
    );
    assert!(
        page10.contains("FullParagraph[미주]  pi=513"),
        "PDF 기준 page 10 오른쪽 단 하단에는 문11 제목 아래 첫 본문 줄까지 남아야 함\n{page10}"
    );
    assert!(
        !page11.contains("FullParagraph[미주]  pi=512")
            && !page11.contains("FullParagraph[미주]  pi=513"),
        "문11 제목/첫 본문 줄이 page 11로 밀리면 page10 오른쪽 단 흐름이 한컴/PDF보다 늦음\n{page11}"
    );

    let tree = doc.build_page_render_tree(9).expect("page 10 render tree");
    let q7_title_bbox = find_text_line_bbox(&tree.root, 480, 0).expect("문7 제목");
    let q7_intro_bbox = find_text_line_bbox(&tree.root, 481, 0).expect("문7 첫 설명 줄");
    let q7_formula_bbox = find_equation_bbox(&tree.root, 482, 0).expect("문7 첫 수식 줄");
    let q11_y = min_para_text_y(&tree.root, 512).expect("문11 제목");
    let q11_body_bbox = find_text_line_bbox(&tree.root, 513, 0).expect("문11 첫 본문 줄");

    assert!(
        q7_title_bbox.x < 80.0 && q7_intro_bbox.x < 80.0 && q7_formula_bbox.x < 80.0,
        "문7 제목/도입 줄은 모두 page10 왼쪽 단에 있어야 함: title={:?}, intro={:?}, formula={:?}",
        q7_title_bbox,
        q7_intro_bbox,
        q7_formula_bbox
    );
    assert!(
        q7_intro_bbox.y > q7_title_bbox.y && q7_formula_bbox.y > q7_intro_bbox.y,
        "문7 도입 줄은 제목 아래 순서로 배치되어야 함: title={:?}, intro={:?}, formula={:?}",
        q7_title_bbox,
        q7_intro_bbox,
        q7_formula_bbox
    );
    assert!(
        (1040.0..=1095.0).contains(&q11_y),
        "문11 제목은 PDF page10 오른쪽 단 하단(약 1060px)에 있어야 함: y={q11_y}"
    );
    assert!(
        q11_body_bbox.x > 390.0 && q11_body_bbox.y > q11_y,
        "문11 첫 본문 줄은 PDF처럼 page10 오른쪽 단 제목 아래에 이어져야 함: {:?}",
        q11_body_bbox
    );
}

#[test]
fn issue_1293_2024_visible_separator_above20_between7_question6_body_starts_right_column() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp")
        .expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    assert_eq!(doc.page_count(), 21, "한컴/PDF 기준 페이지 수");

    let page10 = doc.dump_page_items(Some(9));
    let right_column = page10.find("  단 1").expect("page 10 right column");
    let q6_title = page10
        .find("FullParagraph[미주]  pi=471")
        .expect("page 10 문6 제목");
    let q6_body = page10
        .find("FullParagraph[미주]  pi=472")
        .expect("page 10 문6 첫 본문/수식");
    let q7_title = page10
        .find("FullParagraph[미주]  pi=480")
        .expect("page 10 문7 제목");

    assert!(
        q6_title < right_column && right_column < q6_body && q6_body < q7_title,
        "구분선 위 20mm/미주 사이 7mm에서는 문6 제목만 왼쪽 단 하단에 남고, 문6 본문부터 오른쪽 단 상단으로 이어져야 함\n{page10}"
    );

    let tree = doc.build_page_render_tree(9).expect("page 10 render tree");
    let q6_title_bbox = find_text_line_bbox(&tree.root, 471, 0).expect("문6 제목");
    let q6_body_bbox = find_text_line_bbox(&tree.root, 472, 0).expect("문6 첫 본문/수식");
    let q7_title_bbox = find_text_line_bbox(&tree.root, 480, 0).expect("문7 제목");

    assert!(
        q6_title_bbox.x < 80.0 && (1038.0..=1068.0).contains(&q6_title_bbox.y),
        "문6 제목은 PDF처럼 page10 왼쪽 단 하단에 남아야 함: {:?}",
        q6_title_bbox
    );
    assert!(
        q6_body_bbox.x > 390.0 && (84.0..=110.0).contains(&q6_body_bbox.y),
        "문6 첫 본문/수식은 PDF처럼 page10 오른쪽 단 상단에서 시작해야 함: {:?}",
        q6_body_bbox
    );
    assert!(
        q7_title_bbox.x > 390.0 && (265.0..=295.0).contains(&q7_title_bbox.y),
        "문7 제목은 문6 본문 tail 뒤 PDF 위치 근처에서 시작해야 함: {:?}",
        q7_title_bbox
    );
}

#[test]
fn issue_1293_2024_visible_separator_large_tac_picture_tail_starts_next_page() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwp")
        .expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    assert_eq!(doc.page_count(), 21, "한컴/PDF 기준 페이지 수");

    let page18 = doc.dump_page_items(Some(17));
    let page19 = doc.dump_page_items(Some(18));
    assert!(
        page18.contains("FullParagraph[미주]  pi=881")
            && !page18.contains("FullParagraph[미주]  pi=882"),
        "큰 TAC 그림 문단(pi=882)은 page18 오른쪽 단 하단 bleed로 남기면 안 됨\n{page18}"
    );
    assert!(
        page19.contains("FullParagraph[미주]  pi=882")
            && page19.find("FullParagraph[미주]  pi=882")
                < page19.find("FullParagraph[미주]  pi=883"),
        "PDF 기준 page19는 문30 그래프(pi=882)에서 시작한 뒤 후속 풀이(pi=883)로 이어져야 함\n{page19}"
    );

    let page19_tree = doc.build_page_render_tree(18).expect("page 19 render tree");
    let graph = find_image_bbox(&page19_tree.root, 882, 0).expect("문30 그래프 TAC 그림");
    assert!(
        graph.x < 80.0 && (84.0..=115.0).contains(&graph.y),
        "문30 그래프는 PDF처럼 page19 왼쪽 단 상단에서 시작해야 함: {:?}",
        graph
    );
}

#[test]
fn issue_1293_2024_visible_separator_shape987_page17_rewind_para_stays_whole() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp")
        .expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    assert_eq!(doc.page_count(), 21, "한컴/PDF 기준 페이지 수");

    let page17 = doc.dump_page_items(Some(16));
    let right_column = page17.find("  단 1").expect("page 17 right column");
    let q26_body = page17
        .find("FullParagraph[미주]  pi=786")
        .expect("page 17 문26 본문");
    let q27_title = page17
        .find("FullParagraph[미주]  pi=787")
        .expect("page 17 문27 제목");
    let q28_title = page17
        .find("FullParagraph[미주]  pi=801")
        .expect("page 17 문28 제목");

    assert!(
        !page17.contains("PartialParagraph  pi=786"),
        "새 쪽 왼쪽 단 상단에 들어온 문26 rewind 문단(pi=786)을 다시 2/3줄로 쪼개면 page 수가 22쪽으로 밀림\n{page17}"
    );
    assert!(
        q26_body < q27_title && q27_title < right_column && right_column < q28_title,
        "PDF 기준 page17은 왼쪽 단에 문26 본문과 문27이 이어지고, 오른쪽 단에서 문28이 시작해야 함\n{page17}"
    );

    let page18 = doc.dump_page_items(Some(17));
    let right_column = page18.find("  단 1").expect("page 18 right column");
    let q30_title = page18
        .find("FullParagraph[미주]  pi=849")
        .expect("page 18 문30 제목");
    let q30_intro = page18
        .find("FullParagraph[미주]  pi=850")
        .expect("page 18 문30 첫 줄");

    assert!(
        q30_title < right_column && q30_intro < right_column,
        "PDF 기준 page18은 문30 제목과 첫 줄이 왼쪽 단 하단에 남아야 함\n{page18}"
    );
}

#[test]
fn issue_1293_2024_no_separator_20mm_page10_question4_starts_right_column_with_body() {
    let bytes = std::fs::read(
        "samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp",
    )
    .expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    assert_eq!(doc.page_count(), 23, "한컴/PDF 기준 페이지 수");

    let page10 = doc.dump_page_items(Some(9));
    let page11 = doc.dump_page_items(Some(10));
    let right_column = page10.find("  단 1").expect("page 10 right column");
    let q4_title = page10
        .find("FullParagraph[미주]  pi=464")
        .expect("page 10 문4 제목");
    let q4_body = page10
        .find("FullParagraph[미주]  pi=465")
        .expect("page 10 문4 첫 본문");
    let q4_formula = page10
        .find("FullParagraph[미주]  pi=466")
        .expect("page 10 문4 다음 수식");
    let q5_title = page10
        .find("FullParagraph[미주]  pi=467")
        .expect("page 10 문5 제목");

    assert!(
        !page10[..right_column].contains("FullParagraph[미주]  pi=464"),
        "구분선 없음 20/20/20에서는 문4 제목만 왼쪽 단 하단에 고립되면 안 됨\n{page10}"
    );
    assert!(
        right_column < q4_title && q4_title < q4_body && q4_body < q4_formula && q4_formula < q5_title,
        "PDF 기준 page10 오른쪽 단은 문4 제목 -> 첫 본문 -> 다음 수식 -> 문5 순으로 시작해야 함\n{page10}"
    );
    assert!(
        !page11.contains("FullParagraph[미주]  pi=464")
            && !page11.contains("FullParagraph[미주]  pi=465"),
        "문4 제목/첫 본문은 page11로 밀리면 안 됨\n{page11}"
    );

    let tree = doc.build_page_render_tree(9).expect("page 10 render tree");
    let q4_title_bbox = find_text_line_bbox(&tree.root, 464, 0).expect("문4 제목");
    let q4_body_bbox = find_text_line_bbox(&tree.root, 465, 0).expect("문4 첫 본문");
    let q4_formula_bbox = find_text_line_bbox(&tree.root, 466, 0).expect("문4 다음 수식");
    let q5_y = min_para_text_y(&tree.root, 467).expect("문5 제목");

    assert!(
        q4_title_bbox.x > 390.0 && (84.0..=104.0).contains(&q4_title_bbox.y),
        "문4 제목은 PDF처럼 page10 오른쪽 단 상단에서 시작해야 함: {:?}",
        q4_title_bbox
    );
    assert!(
        q4_body_bbox.x > 390.0
            && q4_body_bbox.y > q4_title_bbox.y
            && (104.0..=130.0).contains(&q4_body_bbox.y),
        "문4 첫 본문은 제목 아래 같은 오른쪽 단에 이어져야 함: {:?}",
        q4_body_bbox
    );
    assert!(
        q4_formula_bbox.x > 390.0
            && q4_formula_bbox.y > q4_body_bbox.y
            && (126.0..=150.0).contains(&q4_formula_bbox.y),
        "문4 다음 수식도 본문 아래 오른쪽 단 상단 흐름에 남아야 함: {:?}",
        q4_formula_bbox
    );
    assert!(
        (210.0..=235.0).contains(&q5_y),
        "문5 제목은 문4 본문/수식 뒤 PDF page10 오른쪽 단 상단 흐름에 이어져야 함: y={q5_y}"
    );
}

#[test]
fn issue_1293_2024_no_separator_20mm_page11_question12_tail_stays_in_frame() {
    let bytes = std::fs::read(
        "samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp",
    )
    .expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    assert_eq!(doc.page_count(), 23, "한컴/PDF 기준 페이지 수");

    let page11 = doc.dump_page_items(Some(10));
    let page12 = doc.dump_page_items(Some(11));
    let right_column = page11.find("  단 1").expect("page 11 right column");
    let q11_last_calc = page11
        .find("FullParagraph[미주]  pi=515")
        .expect("page 11 문11 마지막 왼쪽 단 계산");
    let q12_title = page11
        .find("FullParagraph[미주]  pi=526")
        .expect("page 11 문12 제목");
    let q12_graph = page11
        .find("Shape          pi=537")
        .expect("page 11 문12 그래프");
    let q12_tail = page11
        .find("FullParagraph[미주]  pi=538")
        .expect("page 11 문12 마지막 수식 줄");

    assert!(
        q11_last_calc < right_column,
        "문11 pi=515 두 줄은 PDF처럼 page11 왼쪽 단 하단에 남아야 함\n{page11}"
    );
    assert!(
        !page11[right_column..].contains("pi=515"),
        "문11 pi=515가 오른쪽 단으로 넘어가면 문12가 PDF보다 내려감\n{page11}"
    );
    assert!(
        right_column < q12_title && q12_title < q12_graph && q12_graph < q12_tail,
        "page11 오른쪽 단은 문12 제목 -> 그래프 -> 마지막 수식 순서를 유지해야 함\n{page11}"
    );
    assert!(
        !page12.contains("FullParagraph[미주]  pi=538"),
        "문12 마지막 수식 줄은 page12로 넘어가면 안 됨\n{page12}"
    );
    assert!(
        page12.contains("FullParagraph[미주]  pi=539"),
        "page12는 PDF처럼 문13 제목으로 시작해야 함\n{page12}"
    );

    let page11_tree = doc.build_page_render_tree(10).expect("page 11 render tree");
    let q11_calc_line0 = find_text_line_bbox(&page11_tree.root, 515, 0).expect("문11 계산 첫 줄");
    let q11_calc_line1 = find_text_line_bbox(&page11_tree.root, 515, 1).expect("문11 계산 둘째 줄");
    let q12_title_bbox = find_text_line_bbox(&page11_tree.root, 526, 0).expect("문12 제목");
    let q12_tail_bbox = find_text_line_bbox(&page11_tree.root, 538, 0).expect("문12 마지막 수식");

    assert!(
        q11_calc_line0.x < 80.0
            && q11_calc_line1.x < 80.0
            && (1040.0..=1060.0).contains(&q11_calc_line0.y)
            && (1060.0..=1090.0).contains(&q11_calc_line1.y),
        "문11 pi=515 두 줄은 왼쪽 단 하단에 들어가야 함: {:?}, {:?}",
        q11_calc_line0,
        q11_calc_line1
    );
    assert!(
        q12_title_bbox.x > 390.0 && (450.0..=475.0).contains(&q12_title_bbox.y),
        "문12 제목은 PDF page11 오른쪽 단 중단(약 462px)에서 시작해야 함: {:?}",
        q12_title_bbox
    );
    assert!(
        q12_tail_bbox.x > 390.0
            && (1045.0..=1065.0).contains(&q12_tail_bbox.y)
            && q12_tail_bbox.y + q12_tail_bbox.height <= 1095.0,
        "문12 마지막 수식은 page11 frame 안쪽에 보여야 함: {:?}",
        q12_tail_bbox
    );

    let page12_tree = doc.build_page_render_tree(11).expect("page 12 render tree");
    let q13_title_bbox = find_text_line_bbox(&page12_tree.root, 539, 0).expect("문13 제목");
    assert!(
        q13_title_bbox.x < 80.0 && (84.0..=104.0).contains(&q13_title_bbox.y),
        "page12는 PDF처럼 문13 제목으로 시작해야 함: {:?}",
        q13_title_bbox
    );
}

#[test]
fn issue_1293_2024_no_separator_20mm_page12_question15_tail_keeps_page13_aligned() {
    let bytes = std::fs::read(
        "samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp",
    )
    .expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    assert_eq!(doc.page_count(), 23, "한컴/PDF 기준 페이지 수");

    let page12 = doc.dump_page_items(Some(11));
    let page13 = doc.dump_page_items(Some(12));
    let right_column = page12.find("  단 1").expect("page 12 right column");
    let q15_tail = page12
        .find("FullParagraph[미주]  pi=571")
        .expect("page 12 문15 마지막 다줄 tail");

    assert!(
        right_column < q15_tail,
        "문15 마지막 다줄 tail(pi=571)은 PDF처럼 page12 오른쪽 단 하단에 모두 남아야 함\n{page12}"
    );
    assert!(
        !page12.contains("PartialParagraph  pi=571")
            && !page13.contains("PartialParagraph  pi=571")
            && !page13.contains("FullParagraph[미주]  pi=571"),
        "pi=571을 4/2줄로 분할하면 page13 문16~문19가 PDF보다 약 54px 내려감\npage12:\n{page12}\npage13:\n{page13}"
    );
    assert!(
        page13.contains("FullParagraph[미주]  pi=572"),
        "page13은 PDF처럼 문15 다음 풀이(pi=572)부터 이어져야 함\n{page13}"
    );

    let page12_tree = doc.build_page_render_tree(11).expect("page 12 render tree");
    let q15_tail_last =
        find_text_line_bbox(&page12_tree.root, 571, 5).expect("문15 tail 마지막 줄");
    assert!(
        q15_tail_last.x > 390.0 && q15_tail_last.y + q15_tail_last.height <= 1100.0,
        "문15 pi=571 마지막 줄은 page12 오른쪽 단 frame 안쪽 또는 허용 bleed 안에 보여야 함: {:?}",
        q15_tail_last
    );

    let page13_tree = doc.build_page_render_tree(12).expect("page 13 render tree");
    let q16_title = find_text_line_bbox(&page13_tree.root, 575, 0).expect("문16 제목");
    let q19_tail = find_text_line_bbox(&page13_tree.root, 590, 1).expect("문19 마지막 줄");
    let q19_table = find_table_bbox(&page13_tree.root, 591, 0).expect("문19 도표");
    let q20_title = find_text_line_bbox(&page13_tree.root, 594, 0).expect("문20 제목");
    assert!(
        q16_title.x < 80.0 && (380.0..=405.0).contains(&q16_title.y),
        "문16 제목은 PDF page13 왼쪽 단 약 386px 위치에서 시작해야 함: {:?}",
        q16_title
    );
    assert!(
        q19_tail.y + q19_tail.height <= 1096.0,
        "문19 tail은 page13 frame 아래로 흘러나가면 안 됨: {:?}",
        q19_tail
    );
    assert!(
        q19_table.x > 390.0 && q19_table.y < 120.0,
        "문19 도표는 PDF처럼 page13 오른쪽 단 상단에서 이어져야 함: {:?}",
        q19_table
    );
    assert!(
        q20_title.x > 390.0 && (440.0..=490.0).contains(&q20_title.y),
        "문20 제목은 문19 도표/그래프 뒤 PDF 위치 근처에서 시작해야 함: {:?}",
        q20_title
    );

    let page15_tree = doc.build_page_render_tree(14).expect("page 15 render tree");
    let q26_body = find_text_line_bbox(&page15_tree.root, 659, 0).expect("문26 첫 본문 줄");
    let q27_title = find_text_line_bbox(&page15_tree.root, 669, 0).expect("문27 제목");
    let q27_tail = find_text_line_bbox(&page15_tree.root, 676, 0).expect("문27 끝부분 줄");
    let q27_last = find_text_line_bbox(&page15_tree.root, 677, 0).expect("문27 마지막 줄");
    assert!(
        q26_body.x > 390.0 && (210.0..=240.0).contains(&q26_body.y),
        "문26 본문은 제목 뒤에 미주 사이가 중복 적용되지 않아야 함: {:?}",
        q26_body
    );
    assert!(
        q27_title.x > 390.0 && (620.0..=660.0).contains(&q27_title.y),
        "문27 제목은 문26 본문 보정 후 PDF 위치 근처에서 시작해야 함: {:?}",
        q27_title
    );
    assert!(
        q27_tail.y + q27_tail.height <= 1096.0 && q27_last.y + q27_last.height <= 1096.0,
        "문27 마지막 줄은 page15 frame 아래로 흘러나가면 안 됨: {:?}",
        q27_last
    );

    let page16_tree = doc.build_page_render_tree(15).expect("page 16 render tree");
    let q28_title = find_text_line_bbox(&page16_tree.root, 678, 0).expect("문28 제목");
    assert!(
        q28_title.x < 80.0 && (84.0..=104.0).contains(&q28_title.y),
        "문27 마지막 줄이 page16으로 밀리면 문28 시작이 한컴/PDF보다 내려감: {:?}",
        q28_title
    );

    let page17_tree = doc.build_page_render_tree(16).expect("page 17 render tree");
    let q26_title = find_text_line_bbox(&page17_tree.root, 785, 0).expect("page17 문26 제목");
    let q26_first_formula =
        find_text_line_bbox(&page17_tree.root, 786, 0).expect("page17 문26 첫 수식 줄");
    assert!(
        q26_title.x > 390.0 && (1030.0..=1050.0).contains(&q26_title.y),
        "문26 제목은 PDF처럼 page17 오른쪽 단 하단에 남아야 함: {:?}",
        q26_title
    );
    assert!(
        q26_first_formula.x > 390.0
            && (1045.0..=1065.0).contains(&q26_first_formula.y)
            && q26_first_formula.y + q26_first_formula.height <= 1096.0,
        "문26 첫 수식은 page17 하단 frame 안에 보여야 함: {:?}",
        q26_first_formula
    );

    let page18_tree = doc.build_page_render_tree(17).expect("page 18 render tree");
    let q26_continuation =
        find_text_line_bbox(&page18_tree.root, 786, 1).expect("page18 문26 이어지는 수식");
    assert!(
        q26_continuation.x < 80.0 && (84.0..=104.0).contains(&q26_continuation.y),
        "문26 첫 수식 뒤 이어지는 줄은 PDF처럼 page18 왼쪽 단 상단에서 이어져야 함: {:?}",
        q26_continuation
    );

    let page18 = doc.dump_page_items(Some(17));
    let page19 = doc.dump_page_items(Some(18));
    assert!(
        page18.contains("FullParagraph[미주]  pi=812")
            && page18.contains("FullParagraph[미주]  pi=813")
            && page18.contains("FullParagraph[미주]  pi=815")
            && !page18.contains("FullParagraph[미주]  pi=816"),
        "PDF 기준 page18 오른쪽 단 하단에는 문29 제목과 첫 풀이 일부가 남아야 함\n{page18}"
    );
    assert!(
        !page19.contains("FullParagraph[미주]  pi=812")
            && !page19.contains("FullParagraph[미주]  pi=813")
            && page19.contains("FullParagraph[미주]  pi=816")
            && page19.contains("FullParagraph[미주]  pi=848")
            && page19.contains("FullParagraph[미주]  pi=849"),
        "문29 시작은 page18에 남기되, 다음 rewind 그룹 직전 계산(pi=816)은 PDF처럼 page19 상단으로 넘어가야 함\n{page19}"
    );
    let page19_right_col = page19.find("  단 1").expect("page19 right column");
    let q29_final_formula = page19
        .find("FullParagraph[미주]  pi=848")
        .expect("page19 문29 마지막 수식");
    let q30_title = page19
        .find("FullParagraph[미주]  pi=849")
        .expect("page19 문30 제목");
    assert!(
        q29_final_formula < page19_right_col && page19_right_col < q30_title,
        "PDF 기준 page19 왼쪽 단은 문29 마지막 수식(pi=848)까지 남고, 오른쪽 단은 문30 제목(pi=849)부터 시작해야 함\n{page19}"
    );
    let q29_title = find_text_line_bbox(&page18_tree.root, 812, 0).expect("page18 문29 제목");
    let q29_tail =
        find_text_line_bbox(&page18_tree.root, 815, 0).expect("page18 문29 첫 풀이 tail");
    assert!(
        q29_title.x > 390.0 && (955.0..=985.0).contains(&q29_title.y),
        "문29 제목은 PDF처럼 page18 오른쪽 단 하단에서 시작해야 함: {:?}",
        q29_title
    );
    assert!(
        q29_tail.x > 390.0 && q29_tail.y + q29_tail.height <= 1096.0,
        "문29 첫 풀이 tail은 page18 frame 아래로 흘러나가면 안 됨: {:?}",
        q29_tail
    );

    let page20 = doc.dump_page_items(Some(19));
    let page20_right_col = page20.find("  단 1").expect("page20 right column");
    let q24_final_tail = page20
        .find("FullParagraph[미주]  pi=902")
        .expect("page20 문24 마지막 tail");
    let q25_title = page20
        .find("FullParagraph[미주]  pi=903")
        .expect("page20 문25 제목");
    assert!(
        q24_final_tail < page20_right_col && page20_right_col < q25_title,
        "PDF 기준 page20 왼쪽 단은 문24 마지막 줄(pi=902)까지 남고, 오른쪽 단은 문25(pi=903)부터 시작해야 함\n{page20}"
    );

    let page21 = doc.dump_page_items(Some(20));
    let page22 = doc.dump_page_items(Some(21));
    assert!(
        page21.contains("FullParagraph[미주]  pi=965")
            && !page21.contains("FullParagraph[미주]  pi=966")
            && !page21.contains("FullParagraph[미주]  pi=967"),
        "PDF 기준 page21 오른쪽 단은 문29 큰 그림 뒤 첫 줄(pi=965)까지만 남아야 함\n{page21}"
    );
    assert!(
        page22.contains("FullParagraph[미주]  pi=966")
            && page22.contains("FullParagraph[미주]  pi=967")
            && page22.contains("FullParagraph[미주]  pi=976"),
        "PDF 기준 page22는 문29 그림 뒤 tail(pi=966..967)로 시작해 문30(pi=976)으로 이어져야 함\n{page22}"
    );
    let page22_tree = doc.build_page_render_tree(21).expect("page 22 render tree");
    let q30_title = find_text_line_bbox(&page22_tree.root, 976, 0).expect("page22 문30 제목");
    assert!(
        q30_title.x < 80.0 && (480.0..=500.0).contains(&q30_title.y),
        "문30 제목은 PDF처럼 page22 왼쪽 단 중단에서 시작해야 함: {:?}",
        q30_title
    );
}

#[test]
fn issue_1284_2024_between20_page21_question23_title_stays_in_left_tail() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2024-미주사이20.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page21 = doc.dump_page_items(Some(20));
    let page22 = doc.dump_page_items(Some(21));
    let q23_title = page21
        .find("FullParagraph[미주]  pi=1054")
        .expect("page 21 question 23 title tail");
    let q23_body = page21
        .find("FullParagraph[미주]  pi=1055")
        .expect("page 21 question 23 body continuation");
    assert!(
        q23_title < q23_body,
        "PDF 기준 page 21은 왼쪽 단 하단 문23 제목 뒤 오른쪽 단에서 본문이 이어져야 함\n{page21}"
    );
    assert!(
        !page22.contains("FullParagraph[미주]  pi=1054"),
        "문23 제목은 다음 쪽으로 넘어가면 안 됨\n{page22}"
    );

    let tree = doc.build_page_render_tree(20).expect("page 21 render tree");
    let q23_title_bbox = find_text_line_bbox(&tree.root, 1054, 0).expect("문23 제목");
    let q23_body_bbox = find_text_line_bbox(&tree.root, 1055, 0).expect("문23 본문 첫 줄");
    let q24_body_bbox = find_text_line_bbox(&tree.root, 1060, 0).expect("문24 본문 첫 줄");
    let question30_y = min_para_text_y(&tree.root, 1025).expect("문30 제목");
    let question24_y = min_para_text_y(&tree.root, 1059).expect("문24 제목");
    let question25_y = min_para_text_y(&tree.root, 1066).expect("문25 제목");
    let question26_y = min_para_text_y(&tree.root, 1076).expect("문26 제목");
    let question26_tail_bottom = max_para_content_bottom(&tree.root, 1083).expect("문26 tail");

    assert!(
        q23_title_bbox.x < 80.0 && (1064.0..=1084.0).contains(&q23_title_bbox.y),
        "문23 제목은 PDF page 21 왼쪽 단 하단(약 x=34, y=1073.2)에 있어야 함: {:?}",
        q23_title_bbox
    );
    assert!(
        q23_body_bbox.x > 390.0 && (84.0..=104.0).contains(&q23_body_bbox.y),
        "문23 본문은 PDF page 21 오른쪽 단 상단에서 이어져야 함: {:?}",
        q23_body_bbox
    );
    assert!(
        (242.0..=264.0).contains(&question30_y),
        "page21 왼쪽 단 문30 제목은 현재 sweep 기준 위치 근처에서 시작해야 함: y={question30_y}"
    );
    assert!(
        (256.0..=276.0).contains(&question24_y),
        "문24 제목은 PDF bbox(약 266.2px) 근처에서 시작해야 함: y={question24_y}"
    );
    assert!(
        (282.0..=304.0).contains(&q24_body_bbox.y),
        "문24 제목 뒤 본문 첫 줄은 PDF bbox(수식 상단 약 284.5px, 일반 글자 약 294px)처럼 과한 title-body gap 없이 이어져야 함: {:?}",
        q24_body_bbox
    );
    assert!(
        (528.0..=558.0).contains(&question25_y),
        "문25 제목은 PDF bbox(약 535.8px) 근처에서 시작해야 함: y={question25_y}"
    );
    assert!(
        (812.0..=842.0).contains(&question26_y),
        "문26 제목은 PDF bbox(약 818.5px) 근처에서 시작해야 함: y={question26_y}"
    );
    assert!(
        question26_tail_bottom <= 1092.3,
        "문26 tail은 page 21 오른쪽 단 frame 안에서 끝나야 함: bottom={question26_tail_bottom}"
    );
}

#[test]
fn issue_1284_2024_between20_page22_23_question_tail_matches_pdf() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2024-미주사이20.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");

    let page22 = doc.dump_page_items(Some(21));
    let page22_col1 = page22.find("  단 1").expect("page 22 second column");
    let q28_formula_tail = page22
        .find("FullParagraph[미주]  pi=1114")
        .expect("page 22 q28 formula tail");
    let q28_graph = page22
        .find("FullParagraph[미주]  pi=1115")
        .expect("page 22 q28 graph paragraph");
    assert!(
        q28_formula_tail < page22_col1 && page22_col1 < q28_graph,
        "현재 sweep 기준 page 22는 왼쪽 단 마지막 ㉡ 식(pi=1114) 뒤 오른쪽 단 그래프(pi=1115)로 이어져야 함\n{page22}"
    );

    let page23 = doc.dump_page_items(Some(22));
    let page23_col1 = page23.find("  단 1").expect("page 23 second column");
    let q29_projection_tail = page23
        .find("FullParagraph[미주]  pi=1159")
        .expect("page 23 q29 projection tail");
    let q29_projection_formula = page23
        .find("FullParagraph[미주]  pi=1160")
        .expect("page 23 q29 projection formula");
    let q30_title = page23
        .find("FullParagraph[미주]  pi=1163")
        .expect("page 23 q30 title");
    assert!(
        q29_projection_tail > page23_col1
            && q29_projection_formula > q29_projection_tail
            && q30_title > q29_projection_formula,
        "PDF 기준 page 23 오른쪽 단은 문29 tail(pi=1159,1160) 뒤 문30으로 이어져야 함\n{page23}"
    );

    let page22_tree = doc.build_page_render_tree(21).expect("page 22 render tree");
    let q28_y = min_para_text_y(&page22_tree.root, 1106).expect("문28 제목");
    let q28_formula_tail_bbox =
        find_text_line_bbox(&page22_tree.root, 1114, 0).expect("문28 ㉡ tail");
    assert!(
        (812.0..=836.0).contains(&q28_y),
        "문28 제목은 현재 sweep 기준 위치 근처에서 시작해야 함: y={q28_y}"
    );
    assert!(
        q28_formula_tail_bbox.x < 80.0 && (1052.0..=1084.0).contains(&q28_formula_tail_bbox.y),
        "문28 마지막 ㉡ 식은 현재 sweep처럼 page 22 왼쪽 단 하단에 남아야 함: {:?}",
        q28_formula_tail_bbox
    );

    let page23_tree = doc.build_page_render_tree(22).expect("page 23 render tree");
    let q29_tail_bbox = find_text_line_bbox(&page23_tree.root, 1159, 0).expect("문29 tail");
    let q30_y = min_para_text_y(&page23_tree.root, 1163).expect("문30 제목");
    assert!(
        q29_tail_bbox.x > 390.0 && (116.0..=140.0).contains(&q29_tail_bbox.y),
        "문29 마지막 정사영 tail은 현재 sweep 기준 page 23 오른쪽 단 상단 흐름에서 이어져야 함: {:?}",
        q29_tail_bbox
    );
    assert!(
        (226.0..=252.0).contains(&q30_y),
        "문30 제목은 문29 tail 뒤 현재 sweep 기준 위치 근처에서 시작해야 함: y={q30_y}"
    );
}

#[test]
fn issue_1274_2022_sep_page18_question26_equation_paragraph_reserves_height() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(17).expect("page 18 render tree");

    let equation_bottom =
        max_equation_visual_bottom_in_region(&tree.root, 20.0, 300.0, 1020.0, 1095.0)
            .expect("문26 하단 수식");
    let next_text = find_text_line_bbox(&tree.root, 949, 0).expect("문26 다음 본문");

    assert!(
        next_text.y >= equation_bottom + 0.5,
        "빈 TAC 수식 문단 pi=948은 실제 수식 하단과 다음 문단이 겹치지 않아야 함: equation_bottom={equation_bottom}, next={next_text:?}"
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
        (198.0..235.0).contains(&gap18_to_19),
        "11쪽 문18→문19 미주 간격은 tail frame을 유지하면서 한컴/PDF 흐름을 따라야 함: q18={question18_y}, q19={question19_y}, gap={gap18_to_19}"
    );
    assert!(
        (180.0..210.0).contains(&gap19_to_20),
        "11쪽 문19→문20 미주 간격도 한컴/PDF 흐름을 유지해야 함: q19={question19_y}, q20={question20_y}, gap={gap19_to_20}"
    );
}

#[test]
fn issue_1274_2022_oct_page11_question20_equation_tail_keeps_pdf_bleed() {
    let bytes = std::fs::read("samples/3-10월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(10).expect("page 11 render tree");

    let equation_bottom =
        max_equation_visual_bottom_in_region(&tree.root, 395.0, 700.0, 1020.0, 1100.0)
            .expect("문20 하단 수식");
    assert!(
        equation_bottom <= 1124.0,
        "문20 하단 수식-only tail은 현재 sweep 기준 허용 bleed 안에 남아야 함: bottom={equation_bottom}"
    );
    assert!(
        equation_bottom >= 1118.0,
        "문20 수식 tail을 과도하게 끌어올리면 현재 sweep의 하단 잔여 흐름과 달라짐: bottom={equation_bottom}"
    );
}

#[test]
fn issue_1284_2022_oct_page11_question20_formula_does_not_overlap_next_text() {
    let bytes = std::fs::read("samples/3-10월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(10).expect("page 11 render tree");

    let equation_bottom = max_para_content_bottom(&tree.root, 584).expect("문20 조건식 수식");
    let next_text = find_text_line_bbox(&tree.root, 585, 0).expect("문20 다음 본문");

    assert!(
        next_text.y >= equation_bottom + 0.1,
        "문20 조건식 수식과 다음 본문은 한컴/PDF처럼 겹치지 않아야 함: equation_bottom={equation_bottom}, next={next_text:?}"
    );
}

#[test]
fn issue_1284_2022_oct_page15_question28_formula_does_not_overlap_case_label() {
    let bytes = std::fs::read("samples/3-10월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(14).expect("page 15 render tree");

    let equation_bottom = max_equation_bottom_in_region(&tree.root, 390.0, 570.0, 740.0, 790.0)
        .expect("문28 중간 수식");
    let next_text = find_text_line_bbox(&tree.root, 817, 0).expect("문28 (ii) 본문");

    assert!(
        next_text.y >= equation_bottom + 0.1,
        "문28 중간 수식과 (ii) 본문은 PDF처럼 겹치지 않아야 함: equation_bottom={equation_bottom}, next={next_text:?}"
    );
}

#[test]
fn issue_1284_2022_oct_page14_question25_tail_matches_pdf_frame() {
    let bytes = std::fs::read("samples/3-10월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(13).expect("page 14 render tree");

    let question25_title = find_text_line_bbox(&tree.root, 776, 0).expect("문25 제목");
    let question25_tail = find_text_line_bbox(&tree.root, 779, 0).expect("문25 tail");
    let question25_tail_bottom = question25_tail.y + question25_tail.height;

    assert!(
        question25_title.x > 390.0 && (934.0..=948.0).contains(&question25_title.y),
        "문25 제목은 PDF page 14 오른쪽 단 bbox(약 y=939.7px)와 맞아야 함: {:?}",
        question25_title
    );
    assert!(
        question25_tail_bottom <= 1097.0,
        "문25 tail은 한컴/PDF처럼 page 14 frame 안에서 끝나야 함: tail={:?}, bottom={question25_tail_bottom}",
        question25_tail
    );
}

#[test]
fn issue_1284_2022_oct_page17_question29_tail_matches_pdf_frame() {
    let bytes = std::fs::read("samples/3-10월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(16).expect("page 17 render tree");

    let question28_title = find_text_line_bbox(&tree.root, 926, 0).expect("문28 제목");
    let question29_title = find_text_line_bbox(&tree.root, 948, 0).expect("문29 제목");
    let question29_tail = find_text_line_bbox(&tree.root, 962, 1).expect("문29 tail 마지막 줄");
    let question29_tail_bottom = question29_tail.y + question29_tail.height;

    assert!(
        question28_title.x > 390.0 && (200.0..=214.0).contains(&question28_title.y),
        "문28 제목은 PDF page 17 오른쪽 단 bbox(약 y=206.6px)와 맞아야 함: {:?}",
        question28_title
    );
    assert!(
        question29_title.x > 390.0 && (668.0..=684.0).contains(&question29_title.y),
        "문29 제목은 PDF page 17 오른쪽 단 bbox(약 y=675.1px)와 맞아야 함: {:?}",
        question29_title
    );
    assert!(
        question29_tail_bottom <= 1097.0,
        "문29 tail은 한컴/PDF처럼 page 17 frame 안에서 끝나야 함: tail={:?}, bottom={question29_tail_bottom}",
        question29_tail
    );
}

#[test]
fn issue_1284_2022_nov_practice_page11_question14_tail_matches_pdf_frame() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(10).expect("page 11 render tree");

    let question13_title = find_text_line_bbox(&tree.root, 539, 0).expect("문13 제목");
    let question14_title = find_text_line_bbox(&tree.root, 545, 0).expect("문14 제목");
    let question14_tail = find_text_line_bbox(&tree.root, 553, 7).expect("문14 tail 마지막 줄");
    let question14_tail_bottom = question14_tail.y + question14_tail.height;

    assert!(
        question13_title.x > 390.0 && (398.0..=408.0).contains(&question13_title.y),
        "문13 제목은 PDF page 11 오른쪽 단 bbox(약 y=401.8px)와 맞아야 함: {:?}",
        question13_title
    );
    assert!(
        question14_title.x > 390.0 && (614.0..=624.0).contains(&question14_title.y),
        "문14 제목은 PDF page 11 오른쪽 단 bbox(약 y=617.8px)와 맞아야 함: {:?}",
        question14_title
    );
    assert!(
        question14_tail_bottom <= 1097.0,
        "문14 tail은 한컴/PDF처럼 page 11 frame 안에서 끝나야 함: tail={:?}, bottom={question14_tail_bottom}",
        question14_tail
    );
}

#[test]
fn issue_1284_2022_nov_practice_page19_question25_tail_matches_pdf_frame() {
    let bytes = std::fs::read("samples/3-11월_실전_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(18).expect("page 19 render tree");

    let question24_title = find_text_line_bbox(&tree.root, 899, 0).expect("문24 제목");
    let question25_title = find_text_line_bbox(&tree.root, 903, 0).expect("문25 제목");
    let question25_tail = find_text_line_bbox(&tree.root, 906, 0).expect("문25 tail");
    let question25_tail_bottom = question25_tail.y + question25_tail.height;

    assert!(
        question24_title.x < 100.0 && (826.0..=838.0).contains(&question24_title.y),
        "문24 제목은 PDF page 19 왼쪽 단 bbox(약 y=832.0px)와 맞아야 함: {:?}",
        question24_title
    );
    assert!(
        question25_title.x < 100.0 && (965.0..=977.0).contains(&question25_title.y),
        "문25 제목은 PDF page 19 왼쪽 단 bbox(약 y=971.2px)와 맞아야 함: {:?}",
        question25_title
    );
    assert!(
        question25_tail_bottom <= 1097.0,
        "문25 tail은 한컴/PDF처럼 page 19 frame 안에서 끝나야 함: tail={:?}, bottom={question25_tail_bottom}",
        question25_tail
    );
}

#[test]
fn issue_1284_2023_sep_page19_question29_tail_matches_pdf_frame() {
    let bytes = std::fs::read("samples/3-09월_교육_통합_2023.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(18).expect("page 19 render tree");

    let question29_title = find_text_line_bbox(&tree.root, 946, 0).expect("문29 제목");
    let question29_tail = find_text_line_bbox(&tree.root, 953, 0).expect("문29 tail");
    let question29_tail_bottom = question29_tail.y + question29_tail.height;

    assert!(
        question29_title.x > 390.0 && (696.0..=708.0).contains(&question29_title.y),
        "문29 제목은 PDF page 19 오른쪽 단 bbox(약 y=701.5px)와 맞아야 함: {:?}",
        question29_title
    );
    assert!(
        question29_tail_bottom <= 1097.0,
        "문29 tail은 한컴/PDF처럼 page 19 frame 안에서 끝나야 함: tail={:?}, bottom={question29_tail_bottom}",
        question29_tail
    );
}

#[test]
fn issue_1274_2022_oct_page16_question30_title_tail_continues_next_column() {
    let bytes = std::fs::read("samples/3-10월_교육_통합_2022.hwp").expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let tree = doc.build_page_render_tree(15).expect("page 16 render tree");

    let title = find_text_line_bbox(&tree.root, 841, 0).expect("문30 제목");
    let first_line = find_text_line_bbox(&tree.root, 842, 0).expect("문30 첫 본문 줄");

    assert!(
        (1060.0..=1085.0).contains(&title.y),
        "문30 제목은 한컴/PDF처럼 16쪽 하단에 남아야 함: title={title:?}"
    );
    assert!(
        first_line.x > 390.0 && (84.0..=104.0).contains(&first_line.y),
        "문30 첫 본문 줄은 현재 sweep 기준 다음 단 상단에서 이어져야 함: title={title:?}, first_line={first_line:?}"
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
    // #1310: 문12의 수식-only 미주 흐름은 한컴처럼 연속 TAC 수식을 열 폭 기준으로
    // 자동 줄바꿈해야 한다. 따라서 문13 위치는 고정 상한이 아니라 수식 블록의 실제
    // visual bottom 이후 gap으로 검증한다.
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
        (398.0..=408.0).contains(&question12_formula.x),
        "문12 수식-only 문단의 첫 시각 줄은 한컴 문단 첫 줄 원점을 적용해야 함(#1310): x={}",
        question12_formula.x
    );
    assert!(
        question12_formula.y - question12_tail_y <= 20.0,
        "문12 '따라서'와 수식-only 문단 사이 간격은 한컴/PDF처럼 촘촘해야 함: tail_y={question12_tail_y}, formula_y={}",
        question12_formula.y
    );
    let mut question12_formulas = Vec::new();
    collect_equation_bboxes_in_region(
        &tree.root,
        398.0,
        760.5,
        question12_formula.y - 1.0,
        question13_y,
        &mut question12_formulas,
    );
    let max_formula_right = question12_formulas
        .iter()
        .map(|bbox| bbox.x + bbox.width)
        .fold(0.0f64, f64::max);
    assert!(
        max_formula_right <= 760.5,
        "문12 수식-only 흐름은 오른쪽 단을 넘지 않아야 함(#1310): max_right={max_formula_right}"
    );
    assert!(
        question12_formulas.iter().any(|bbox| {
            (bbox.x - (question12_formula.x + 80.7)).abs() <= 7.0
                && bbox.y > question12_formula.y + 25.0
                && bbox.width > 170.0
        }),
        "문12 첫 수식 줄에서 넘친 세 번째 TAC 수식은 다음 visual row로 줄바꿈되고 한컴 UI 내어쓰기 60.5pt 전체 x를 적용해야 함(#1310): {question12_formulas:?}"
    );
    let formula_bottom = question12_formulas
        .iter()
        .map(|bbox| bbox.y + bbox.height)
        .fold(0.0f64, f64::max);
    assert!(
        (12.0..=42.0).contains(&(question13_y - formula_bottom)),
        "문13은 wrapping된 문12 수식 블록 뒤에 자연스럽게 이어져야 함: formula_bottom={formula_bottom}, q13_y={question13_y}"
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
