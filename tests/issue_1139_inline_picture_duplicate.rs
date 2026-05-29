//! Task #1139: 문27 inline TAC Picture가 다음 줄까지 미리 렌더되어 중복 출력되던 회귀 방지.

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn hwpunit_to_mm(hu: i32) -> f64 {
    hu as f64 * 25.4 / 7200.0
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
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
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
}
