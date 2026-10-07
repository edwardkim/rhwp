//! Issue #3738 Stage 9: 작은 RowBreak 표의 cell-footnote 전체 선예약이
//! 첫 fragment를 통째로 다음 쪽으로 미는 회귀를 실제 HWP로 고정한다.
//!
//! 한컴오피스 2020 기준 PDF p66에는 표 23의 0–4행(Organ Donation까지)과
//! 각주 76·77이 있고, p67은 Stephanie 행부터 이어진다. 표 전체 각주를
//! 첫 행 전부터 예약하면 p66 표가 전부 이월되어 이후 문단까지 한 쪽씩 밀린다.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp";
const PAGE_66: u32 = 65;
const PAGE_67: u32 = 66;
const PAGE_30: u32 = 29;
const PAGE_31: u32 = 30;
const PAGE_32: u32 = 31;
const PAGE_68: u32 = 67;
const PAGE_69: u32 = 68;
const PAGE_74: u32 = 73;
const PAGE_75: u32 = 74;
const PAGE_58: u32 = 57;
const PAGE_59: u32 = 58;
const PAGE_76: u32 = 75;
const PAGE_77: u32 = 76;
const PAGE_78: u32 = 77;
const PAGE_79: u32 = 78;
const PAGE_80: u32 = 79;
const PAGE_87: u32 = 86;
const PAGE_88: u32 = 87;
const PAGE_90: u32 = 89;
const PAGE_91: u32 = 90;
const PAGE_94: u32 = 93;
const PAGE_95: u32 = 94;
const PAGE_118: u32 = 117;
const PAGE_119: u32 = 118;
const PAGE_120: u32 = 119;
const PAGE_121: u32 = 120;
const PAGE_129: u32 = 128;
const PAGE_130: u32 = 129;
const PAGE_131: u32 = 130;
const PAGE_132: u32 = 131;
const PAGE_126: u32 = 125;
const PAGE_127: u32 = 126;
const PAGE_37: u32 = 36;
const PAGE_43: u32 = 42;
const PAGE_44: u32 = 43;
const PAGE_25: u32 = 24;
const PAGE_26: u32 = 25;
const PAGE_27: u32 = 26;
const PAGE_52: u32 = 51;
const PAGE_53: u32 = 52;
const PAGE_54: u32 = 53;
const PAGE_154: u32 = 153;
const PAGE_155: u32 = 154;
const PAGE_156: u32 = 155;
const PAGE_157: u32 = 156;
const PAGE_158: u32 = 157;
const PAGE_166: u32 = 165;
const PAGE_167: u32 = 166;
const PAGE_168: u32 = 167;
const PAGE_169: u32 = 168;
const PAGE_170: u32 = 169;
const PAGE_171: u32 = 170;
const PAGE_172: u32 = 171;
const PAGE_173: u32 = 172;
const PAGE_174: u32 = 173;
const PAGE_175: u32 = 174;
const PAGE_176: u32 = 175;
const PAGE_177: u32 = 176;
const PAGE_178: u32 = 177;
const PAGE_179: u32 = 178;
const PAGE_182: u32 = 181;
const PAGE_183: u32 = 182;
const PAGE_199: u32 = 198;
const PAGE_200: u32 = 199;
const PAGE_201: u32 = 200;

fn page_text(doc: &HwpDocument, page: u32) -> String {
    doc.extract_page_text_native(page)
        .unwrap_or_else(|e| panic!("extract physical page {}: {e}", page + 1))
}

fn subtree_bottom(node: &RenderNode) -> f64 {
    node.children
        .iter()
        .fold(node.bbox.y + node.bbox.height, |bottom, child| {
            bottom.max(subtree_bottom(child))
        })
}

fn footnote_and_footer(
    node: &RenderNode,
    footnote_bottom: &mut Option<f64>,
    footer_top: &mut Option<f64>,
) {
    match node.node_type {
        RenderNodeType::FootnoteArea => *footnote_bottom = Some(subtree_bottom(node)),
        RenderNodeType::Footer => *footer_top = Some(node.bbox.y),
        _ => {}
    }
    for child in &node.children {
        footnote_and_footer(child, footnote_bottom, footer_top);
    }
}

fn body_bbox(node: &RenderNode, bbox: &mut Option<BoundingBox>) {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        *bbox = Some(node.bbox);
        return;
    }
    for child in &node.children {
        body_bbox(child, bbox);
    }
}

fn paragraph_bottom(node: &RenderNode, para_index: usize, bottom: &mut Option<f64>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para_index) {
            let candidate = node.bbox.y + node.bbox.height;
            *bottom = Some(bottom.map_or(candidate, |current| current.max(candidate)));
        }
    }
    for child in &node.children {
        paragraph_bottom(child, para_index, bottom);
    }
}

fn footnote_separator_top(node: &RenderNode, top: &mut Option<f64>) {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        for child in &node.children {
            if matches!(child.node_type, RenderNodeType::Line(_)) {
                *top = Some(child.bbox.y);
                return;
            }
        }
    }
    for child in &node.children {
        footnote_separator_top(child, top);
    }
}

fn footnote_separator_bbox(node: &RenderNode, bbox: &mut Option<BoundingBox>) {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        if let Some(line) = node
            .children
            .iter()
            .find(|child| matches!(child.node_type, RenderNodeType::Line(_)))
        {
            *bbox = Some(line.bbox);
            return;
        }
    }
    for child in &node.children {
        footnote_separator_bbox(child, bbox);
    }
}

fn table_bottom(node: &RenderNode, para_index: usize, bottom: &mut Option<f64>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para_index) {
            let candidate = node.bbox.y + node.bbox.height;
            *bottom = Some(bottom.map_or(candidate, |current| current.max(candidate)));
        }
    }
    for child in &node.children {
        table_bottom(child, para_index, bottom);
    }
}

fn table_boxes_for_paragraph(node: &RenderNode, para_index: usize, boxes: &mut Vec<BoundingBox>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para_index) {
            boxes.push(node.bbox);
        }
    }
    for child in &node.children {
        table_boxes_for_paragraph(child, para_index, boxes);
    }
}

fn table_top(node: &RenderNode, para_index: usize, top: &mut Option<f64>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para_index) {
            let candidate = node.bbox.y;
            *top = Some(top.map_or(candidate, |current| current.min(candidate)));
        }
    }
    for child in &node.children {
        table_top(child, para_index, top);
    }
}

fn images_for_control(
    node: &RenderNode,
    para_index: usize,
    control_index: usize,
    positions: &mut Vec<(f64, f64)>,
) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.para_index == Some(para_index) && image.control_index == Some(control_index) {
            positions.push((node.bbox.x, node.bbox.y));
        }
    }
    for child in &node.children {
        images_for_control(child, para_index, control_index, positions);
    }
}

fn image_boxes_for_control(
    node: &RenderNode,
    para_index: usize,
    control_index: usize,
    boxes: &mut Vec<BoundingBox>,
) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.para_index == Some(para_index) && image.control_index == Some(control_index) {
            boxes.push(node.bbox);
        }
    }
    for child in &node.children {
        image_boxes_for_control(child, para_index, control_index, boxes);
    }
}

fn paragraph_line_boxes(node: &RenderNode, para_index: usize, boxes: &mut Vec<BoundingBox>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para_index) {
            boxes.push(node.bbox);
        }
    }
    for child in &node.children {
        paragraph_line_boxes(child, para_index, boxes);
    }
}

fn paragraph_line_indices(node: &RenderNode, para_index: usize, out: &mut Vec<u32>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para_index) {
            if let Some(line_index) = line.line_index {
                out.push(line_index);
            }
        }
    }
    for child in &node.children {
        paragraph_line_indices(child, para_index, out);
    }
}

fn vertically_intersects(left: BoundingBox, right: BoundingBox) -> bool {
    left.y < right.y + right.height && right.y < left.y + left.height
}

fn does_not_overlap_horizontally(left: BoundingBox, right: BoundingBox) -> bool {
    left.x + left.width <= right.x + 0.5 || right.x + right.width <= left.x + 0.5
}

fn images_for_table(node: &RenderNode, para_index: usize, positions: &mut Vec<(f64, f64)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para_index) {
            fn collect_images(node: &RenderNode, positions: &mut Vec<(f64, f64)>) {
                if matches!(node.node_type, RenderNodeType::Image(_)) {
                    positions.push((node.bbox.x, node.bbox.y));
                }
                for child in &node.children {
                    collect_images(child, positions);
                }
            }
            collect_images(node, positions);
            return;
        }
    }
    for child in &node.children {
        images_for_table(child, para_index, positions);
    }
}

fn footnote_text(node: &RenderNode, in_footnote: bool, text: &mut String) {
    let in_footnote = in_footnote || matches!(node.node_type, RenderNodeType::FootnoteArea);
    if in_footnote {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            text.push_str(run.display_or_text());
        }
    }
    for child in &node.children {
        footnote_text(child, in_footnote, text);
    }
}

fn assert_footnote_owner<const N: usize>(
    notes: &[String; N],
    pages: &[u32; N],
    number: &str,
    expected_page_index: usize,
    needles: &[&str],
) {
    let marker = format!("{number})");
    for (index, text) in notes.iter().enumerate() {
        let physical_page = pages[index] + 1;
        if index == expected_page_index {
            assert_eq!(
                text.matches(&marker).count(),
                1,
                "p{physical_page}는 각주 {number} 번호를 정확히 한 번 소유해야 함: {text}"
            );
            for needle in needles {
                assert!(
                    text.contains(needle),
                    "p{physical_page} 각주 {number}에 고유 본문이 누락됨 ({needle}): {text}"
                );
            }
        } else {
            assert_eq!(
                text.matches(&marker).count(),
                0,
                "p{physical_page}는 각주 {number} 번호를 소유하면 안 됨: {text}"
            );
            for needle in needles {
                assert!(
                    !text.contains(needle),
                    "p{physical_page}에 각주 {number}의 marker 없는 fragment가 남으면 안 됨 ({needle}): {text}"
                );
            }
        }
    }
}

fn footnote_line_count(node: &RenderNode, in_footnote: bool) -> usize {
    let in_footnote = in_footnote || matches!(node.node_type, RenderNodeType::FootnoteArea);
    let here = usize::from(in_footnote && matches!(node.node_type, RenderNodeType::TextLine(_)));
    here + node
        .children
        .iter()
        .map(|child| footnote_line_count(child, in_footnote))
        .sum::<usize>()
}

/// Stage 29: fragment queue는 빈 각주 문단도 가상 한 줄로 예약한다. 실제 composer
/// 결과가 0줄일 때, 번호를 그리는 첫 fragment가 그 가상 범위를 그대로 slice하면
/// range-end 1이 실제 len 0을 넘어 panic 난다. 빈 문단 fallback line을 보존한다.
#[test]
fn empty_footnote_virtual_fragment_uses_fallback_without_slice_panic() {
    use rhwp::model::control::Control;
    use rhwp::renderer::composer::compose_paragraph;

    let mut doc = HwpDocument::create_empty();
    doc.insert_text_native(0, 0, 0, "본문")
        .expect("seed body text for a footnote marker");
    doc.insert_footnote_native(0, 0, 2)
        .expect("insert initially blank footnote");

    // 공개 편집 API가 만든 각주 contract(AutoNumber 포함)는 유지하고, 사용자 편집
    // 뒤 lineSeg와 표시 텍스트가 모두 비어 있는 실제 renderer 입력만 만든다.
    let mut document = doc.document().clone();
    let footnote = document.sections[0].paragraphs[0]
        .controls
        .iter_mut()
        .find_map(|control| match control {
            Control::Footnote(footnote) => Some(footnote),
            _ => None,
        })
        .expect("inserted body footnote");
    let empty_para = footnote
        .paragraphs
        .first_mut()
        .expect("inserted footnote paragraph");
    empty_para.text.clear();
    empty_para.char_offsets.clear();
    empty_para.line_segs.clear();
    empty_para.char_count = 0;
    empty_para.has_para_text = false;
    assert!(
        compose_paragraph(empty_para).lines.is_empty(),
        "regression setup requires a 0-line composed footnote paragraph"
    );
    doc.set_document(document);

    assert!(
        doc.page_has_footnote_footholds_native(0),
        "pagination must retain the footnote so the layout path is exercised"
    );
    let tree = doc
        .build_page_render_tree(0)
        .expect("empty footnote virtual fragment must render without a slice panic");
    assert_eq!(
        footnote_line_count(&tree.root, false),
        1,
        "the 0-line footnote must keep the one-line fallback reserved by pagination"
    );
}
