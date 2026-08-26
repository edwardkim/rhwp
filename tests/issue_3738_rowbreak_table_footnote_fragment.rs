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
            text.push_str(&run.text);
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

#[test]
fn rowbreak_table_cell_footnotes_keep_the_pdf_fragment_boundary() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage9 HWP evidence fixture");

    assert!(
        doc.page_count() <= 224,
        "표 23의 전체 table-footnote 선예약이 되살아 HWP가 225쪽 이상으로 과페이지화됨: {}쪽",
        doc.page_count()
    );

    let p66 = page_text(&doc, PAGE_66);
    let p67 = page_text(&doc, PAGE_67);
    assert!(
        p66.contains("National Organ Transplant Act") && p66.contains("Organ Donation"),
        "p66에 기준 PDF의 표 23 0–4행이 함께 남아야 함: {p66}"
    );
    assert!(
        !p66.contains("Stephanie Tubbs Jones"),
        "p66은 기준 PDF처럼 Stephanie 행 이전에서 끝나야 함: {p66}"
    );
    assert!(
        p67.contains("Stephanie Tubbs Jones") && p67.contains("OPTN policy 14"),
        "p67은 기준 PDF처럼 표 23의 남은 5–6행에서 재개해야 함: {p67}"
    );

    let p66_tree = doc
        .build_page_render_tree(PAGE_66)
        .expect("render physical page 66");
    let mut p66_notes = String::new();
    footnote_text(&p66_tree.root, false, &mut p66_notes);
    assert!(
        p66_notes.contains("76)") && p66_notes.contains("77)"),
        "p66은 PDF처럼 table row 1의 note 77 첫 fragment를 note 76 뒤에 보여야 함: {p66_notes}"
    );

    let mut p66_table_bottom = None;
    let mut p66_separator_top = None;
    table_bottom(&p66_tree.root, 728, &mut p66_table_bottom);
    footnote_separator_top(&p66_tree.root, &mut p66_separator_top);
    assert!(
        p66_table_bottom.expect("p66 pi=728 table")
            <= p66_separator_top.expect("p66 footnote separator") + 0.5,
        "p66 table 23과 note 77 separator가 겹치면 안 됨"
    );

    let tree = doc
        .build_page_render_tree(PAGE_67)
        .unwrap_or_else(|e| panic!("render physical page 67: {e}"));
    let mut footnote_bottom = None;
    let mut footer_top = None;
    footnote_and_footer(&tree.root, &mut footnote_bottom, &mut footer_top);
    let footnote_bottom = footnote_bottom.expect("p67 footnote area");
    let footer_top = footer_top.expect("p67 footer");
    assert!(
        footnote_bottom <= footer_top + 1.0,
        "p67 각주 실제 하단({footnote_bottom:.1}px)이 footer 시작({footer_top:.1}px)을 넘어선다"
    );

    let mut p67_notes = String::new();
    footnote_text(&tree.root, false, &mut p67_notes);
    assert!(
        !p67_notes.contains("77)")
            && p67_notes.contains("Part 482(CONDITIONS OF PARTICIPATION")
            && p67_notes.contains("78)")
            && p67_notes.contains("85)"),
        "p67은 note 77의 번호 없는 tail과 78–85를 순서대로 이어야 함: {p67_notes}"
    );
    let mut p67_body_bottom = None;
    let mut p67_separator_top = None;
    paragraph_bottom(&tree.root, 736, &mut p67_body_bottom);
    footnote_separator_top(&tree.root, &mut p67_separator_top);
    assert!(
        p67_body_bottom.expect("p67 pi=736 body")
            <= p67_separator_top.expect("p67 footnote separator") + 0.5,
        "p67 본문과 table-cell note lane이 겹치면 안 됨"
    );
}

#[test]
fn native_hwp5_rowbreak_table_reclaims_only_the_actual_existing_footnote_boundary() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage31 HWP evidence fixture");

    // 한컴 PDF p90은 표 27의 "이식대상자와 관계" row를 note 141의 실제
    // FootnoteArea 바로 위에 둔다. 일반 40px safety margin은 이 30.7px row를
    // p91로 밀지만, p90의 물리 boundary 안에는 들어간다. 마지막 "기타" row는
    // 여전히 p91에서 시작해야 한다.
    let p90 = page_text(&doc, PAGE_90);
    let p91 = page_text(&doc, PAGE_91);
    assert!(
        p90.contains("이식대상자와") && p90.contains("형제만 가능") && p90.contains("친척만 가능"),
        "p90은 PDF처럼 표 27의 relationship row에서 끝나야 함: {p90}"
    );
    assert!(
        !p91.contains("이식대상자와") && p91.contains("기타"),
        "p91은 PDF처럼 표 27의 기타 row로 재개해야 함: {p91}"
    );
    assert!(
        doc.page_count() <= 219,
        "p90 표 27 row owner 보정은 extra native page를 만들면 안 됨: {}쪽",
        doc.page_count()
    );

    let p90_tree = doc
        .build_page_render_tree(PAGE_90)
        .expect("render physical page 90");
    let p91_tree = doc
        .build_page_render_tree(PAGE_91)
        .expect("render physical page 91");
    let p90_items = doc.dump_page_items(Some(PAGE_90));
    let host_pos = p90_items
        .find("PartialParagraph  pi=962")
        .expect("p90 must pre-emit table 27 host caption");
    let table_pos = p90_items
        .find("PartialTable   pi=962 ci=0  rows=0..6")
        .expect("p90 must own table 27 rows 0..6");
    assert!(
        host_pos < table_pos,
        "p90 table 27 host caption must precede its first fragment:\n{p90_items}"
    );
    let p91_items = doc.dump_page_items(Some(PAGE_91));
    assert!(
        p91_items.contains("PartialTable   pi=962 ci=0  rows=6..7")
            && !p91_items.contains("PartialParagraph  pi=962"),
        "p91 must contain only table 27's terminal row, not its host caption:\n{p91_items}"
    );

    let mut p90_caption_lines = Vec::new();
    let mut p91_caption_lines = Vec::new();
    paragraph_line_boxes(&p90_tree.root, 962, &mut p90_caption_lines);
    paragraph_line_boxes(&p91_tree.root, 962, &mut p91_caption_lines);
    let mut p90_table_top = None;
    table_top(&p90_tree.root, 962, &mut p90_table_top);
    assert_eq!(
        p90_caption_lines.len(),
        1,
        "p90 must render the single stored table 27 caption line"
    );
    assert!(
        p90_caption_lines[0].y + p90_caption_lines[0].height
            <= p90_table_top.expect("p90 pi=962 table top") + 0.5,
        "p90 table 27 caption must stay above its first table fragment"
    );
    assert!(
        p91_caption_lines.is_empty(),
        "p91 must not repeat or defer the pi=962 caption: {p91_caption_lines:?}"
    );
    let mut p90_table_bottom = None;
    let mut p90_separator_top = None;
    table_bottom(&p90_tree.root, 962, &mut p90_table_bottom);
    footnote_separator_top(&p90_tree.root, &mut p90_separator_top);
    assert!(
        p90_table_bottom.expect("p90 pi=962 table")
            <= p90_separator_top.expect("p90 note 141 separator") + 0.5,
        "p90 표 27은 note 141 separator 위에서 끝나야 함"
    );
}

/// #3820 Stage 7: p168의 표 44(`pi=1778`)는 p169로 통째 이월되는 표가 아니다.
/// 한컴 2020 PDF는 p168에 첫 fragment를 두고 p169에서 이어 그린 뒤 그림 65를 같은
/// 페이지에 둔다. 이 first fragment를 잃으면 그림 65와 `(라) 심혈관계 검사`가 한
/// 쪽씩 늦어져 p170 이후 문서 전체가 다른 논리 페이지와 대조된다.
#[test]
fn native_hwp5_rowbreak_table_starts_its_first_fragment_on_p168() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage7 HWP evidence fixture");

    let p168 = page_text(&doc, PAGE_168);
    let p169 = page_text(&doc, PAGE_169);
    let p170 = page_text(&doc, PAGE_170);
    assert!(
        p168.contains("기증자 평가 전") && p168.contains("이식대상자로부터 설명동의를 구함"),
        "p168은 PDF처럼 표 44(pi=1778)의 첫 fragment를 보유해야 함: {p168}"
    );
    assert!(
        p169.contains("전파 가능성을 염두에 둔 조심스러운 추적")
            && p169.contains("그림 65. 생존 기증자에 대한 결핵 스크리닝 권고안"),
        "p169은 표 44 continuation 뒤 그림 65를 함께 보유해야 함: {p169}"
    );
    assert!(
        p170.contains("(라) 심혈관계 검사") && !p170.contains("그림 65."),
        "p170은 PDF처럼 그림 65 전용 쪽이 아니라 심혈관계 검사 본문으로 시작해야 함: {p170}"
    );

    let p168_tree = doc
        .build_page_render_tree(PAGE_168)
        .expect("render physical page 168");
    let p169_tree = doc
        .build_page_render_tree(PAGE_169)
        .expect("render physical page 169");
    let mut p168_table = None;
    let mut p169_table = None;
    table_bottom(&p168_tree.root, 1778, &mut p168_table);
    table_bottom(&p169_tree.root, 1778, &mut p169_table);
    assert!(
        p168_table.is_some() && p169_table.is_some(),
        "표 44는 p168/p169 양쪽에 fragment를 렌더해야 함: p168={p168_table:?}, p169={p169_table:?}"
    );
}

/// #3820 Stage 11: p166의 `pi=1771`은 세 번째 source line이 `vpos=0`인
/// physical-page reset이다. RowBreak 표를 앞둔 일반 tail 보존은 한 줄을 되돌리지만,
/// 이 저장 reset 직전의 두 줄까지 되돌리면 PDF p166의 마지막 본문 줄이 p167로 밀린다.
#[test]
fn native_hwp5_rowbreak_table_keeps_pre_reset_tail_on_p166() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage11 HWP evidence fixture");

    let p166 = page_text(&doc, PAGE_166);
    let p167 = page_text(&doc, PAGE_167);
    assert!(
        p166.contains("이 제시하는 조건들임.") && !p166.contains("해야 함. 높은 위험"),
        "p166은 PDF처럼 pi=1771 reset 전 두 줄에서 끝나야 함: {p166}"
    );
    assert!(
        p167.contains("해야 함. 높은 위험") && !p167.contains("이 제시하는 조건들임."),
        "p167은 PDF처럼 pi=1771 reset tail부터 시작해야 함: {p167}"
    );

    let p166_tree = doc
        .build_page_render_tree(PAGE_166)
        .expect("render physical page 166");
    let p167_tree = doc
        .build_page_render_tree(PAGE_167)
        .expect("render physical page 167");
    let mut p166_lines = Vec::new();
    let mut p167_lines = Vec::new();
    paragraph_line_indices(&p166_tree.root, 1771, &mut p166_lines);
    paragraph_line_indices(&p167_tree.root, 1771, &mut p167_lines);
    assert_eq!(p166_lines, vec![0, 1], "p166 pi=1771 line owner");
    assert_eq!(p167_lines, vec![2], "p167 pi=1771 line owner");
}

/// #3820 Stage 11: p171의 `pi=1797`은 다음 저장 사다리가 표 선언 높이를 비우지
/// 않는 empty-host float이다. raw anchor를 흐름 높이로 쓰면 p1799 뒤의 본문이
/// 과대 계상되어 `pi=1800`과 `pi=1801` prefix가 통째로 p172로 밀린다. PDF는
/// p171에 pi=1800 전체와 pi=1801 reset 전 세 줄을 보존한다.
#[test]
fn native_hwp5_nonvacating_float_ladder_keeps_p171_text_owner() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage11 HWP evidence fixture");

    let p171 = page_text(&doc, PAGE_171);
    let p172 = page_text(&doc, PAGE_172);
    assert!(
        p171.contains("- EDQM에서는") && p171.contains("- BTS에서는"),
        "p171은 PDF처럼 pi=1800와 pi=1801 prefix를 보유해야 함: {p171}"
    );
    assert!(
        !p172.contains("- EDQM에서는") && p172.contains("편으로 사용할 때에는"),
        "p172는 PDF처럼 pi=1801 reset tail부터 이어져야 함: {p172}"
    );

    let p171_tree = doc
        .build_page_render_tree(PAGE_171)
        .expect("render physical page 171");
    let p172_tree = doc
        .build_page_render_tree(PAGE_172)
        .expect("render physical page 172");
    let mut p171_1800_lines = Vec::new();
    let mut p171_1801_lines = Vec::new();
    let mut p172_1801_lines = Vec::new();
    paragraph_line_indices(&p171_tree.root, 1800, &mut p171_1800_lines);
    paragraph_line_indices(&p171_tree.root, 1801, &mut p171_1801_lines);
    paragraph_line_indices(&p172_tree.root, 1801, &mut p172_1801_lines);
    assert_eq!(p171_1800_lines, vec![0, 1, 2], "p171 pi=1800 line owner");
    assert_eq!(p171_1801_lines, vec![0, 1, 2], "p171 pi=1801 prefix owner");
    assert_eq!(
        p172_1801_lines,
        vec![3, 4, 5],
        "p172 pi=1801 reset tail owner"
    );
}

/// #3820 Stage 11: p173의 기존 각주 222 바로 위에는 `pi=1816`의 reset 전
/// 두 줄이 남고, reset tail과 표 46(`pi=1822`)의 첫 fragment가 PDF p174에서
/// 이어져야 한다. p175에는 표 46 continuation과 그 표의 각주가 이어진다. 첫
/// fragment를 통째로 p175로 defer하면 p174의 큰 빈 영역과 이후 owner drift가 생긴다.
#[test]
fn native_hwp5_footnote_reset_keeps_rowbreak_table_on_p174() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage11 HWP evidence fixture");

    let p173_tree = doc
        .build_page_render_tree(PAGE_173)
        .expect("render physical page 173");
    let p174_tree = doc
        .build_page_render_tree(PAGE_174)
        .expect("render physical page 174");
    let p175_tree = doc
        .build_page_render_tree(PAGE_175)
        .expect("render physical page 175");
    let p176_tree = doc
        .build_page_render_tree(PAGE_175 + 1)
        .expect("render physical page 176");
    let mut p173_lines = Vec::new();
    let mut p174_lines = Vec::new();
    paragraph_line_indices(&p173_tree.root, 1816, &mut p173_lines);
    paragraph_line_indices(&p174_tree.root, 1816, &mut p174_lines);
    assert_eq!(p173_lines, vec![0, 1], "p173 pi=1816 reset 전 prefix");
    assert_eq!(p174_lines, vec![2, 3], "p174 pi=1816 reset tail");

    let mut p174_table = None;
    let mut p175_table = None;
    table_bottom(&p174_tree.root, 1822, &mut p174_table);
    table_bottom(&p175_tree.root, 1822, &mut p175_table);
    assert!(
        p174_table.is_some(),
        "p174는 PDF처럼 표 46(pi=1822)의 첫 fragment를 보유해야 함"
    );
    assert!(
        p175_table.is_some(),
        "p175는 PDF처럼 표 46(pi=1822)의 continuation을 보유해야 함"
    );

    let mut p174_images = Vec::new();
    let mut p175_images = Vec::new();
    images_for_table(&p174_tree.root, 1822, &mut p174_images);
    images_for_table(&p175_tree.root, 1822, &mut p175_images);
    assert_eq!(
        p174_images.len(),
        1,
        "p174는 PDF처럼 표 46 안의 그림 66을 정확히 한 번 포함해야 함: {p174_images:?}"
    );
    assert!(
        p175_images.is_empty(),
        "그림 66이 p175로 밀리면 안 됨: {p175_images:?}"
    );

    let mut p176_table = None;
    table_bottom(&p176_tree.root, 1822, &mut p176_table);
    assert!(
        p176_table.is_none(),
        "표 46 tail은 PDF처럼 p175에서 끝나야 하며 p176으로 밀리면 안 됨: {p176_table:?}"
    );

    // PDF p174에는 그림 66과 224)까지가 표 46 첫 fragment에, p175에는 Anderson
    // 문단과 223–231 각주가 있어야 한다. 단순히 표가 두 쪽에 모두 존재하는지만
    // 확인하면, 표 첫 조각을 너무 일찍 끊어 p175/p176으로 한 쪽씩 밀어도 통과한다.
    let p174_text = page_text(&doc, PAGE_174);
    let p175_text = page_text(&doc, PAGE_175);
    let p176_text = page_text(&doc, PAGE_175 + 1);
    assert!(
        p174_text.contains("그림 66") && p174_text.contains("사후 기증자의 연령 별 효과"),
        "p174는 PDF처럼 그림 66과 224) 전 본문까지 소유해야 함: {p174_text}"
    );
    assert!(
        p175_text.contains("Anderson") && p175_text.contains("223)") && p175_text.contains("231)"),
        "p175는 PDF처럼 표 46 tail 및 223–231 각주를 함께 소유해야 함: {p175_text}"
    );
    assert!(
        !p176_text.contains("Anderson"),
        "p176으로 표 46 tail이 밀리면 안 됨: {p176_text}"
    );

    let mut p175_footnotes = String::new();
    footnote_text(&p175_tree.root, false, &mut p175_footnotes);
    assert!(
        p175_footnotes.contains("223)") && p175_footnotes.contains("231)"),
        "p175 RenderTree FootnoteArea에는 223–231이 있어야 함: {p175_footnotes}"
    );
}

/// #3820 Stage 11: 그림 67(`pi=1904`)은 2×1 empty-host RowBreak 표 자체가
/// 그림+caption의 흐름 높이를 이미 예약한다. 표 직후 동일 PS의 빈 guide line 다섯
/// 개(`pi=1905..1909`)는 저장 vpos상 표의 paint span 안에 있으므로 다시 advance하면
/// `pi=1913`이 p183으로 밀린다. PDF p182에는 매독·기생충 문단이 모두 있고 p183은
/// 그림 68부터 시작한다.
#[test]
fn native_hwp5_figure_table_guides_keep_p182_paragraph_owner() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage11 HWP evidence fixture");

    let p182 = page_text(&doc, PAGE_182);
    let p183 = page_text(&doc, PAGE_183);
    assert!(
        p182.contains("매독 전파 사례") && p182.contains("기생충 질환"),
        "p182는 PDF처럼 그림 67 뒤의 두 문단을 모두 보유해야 함: {p182}"
    );
    assert!(
        !p183.contains("기생충 질환") && p183.contains("그림 68"),
        "p183은 PDF처럼 그림 68부터 시작해야 함: {p183}"
    );

    let p182_tree = doc
        .build_page_render_tree(PAGE_182)
        .expect("render physical page 182");
    let p183_tree = doc
        .build_page_render_tree(PAGE_183)
        .expect("render physical page 183");
    let mut p182_1913_lines = Vec::new();
    let mut p183_1913_lines = Vec::new();
    paragraph_line_indices(&p182_tree.root, 1913, &mut p182_1913_lines);
    paragraph_line_indices(&p183_tree.root, 1913, &mut p183_1913_lines);
    assert_eq!(
        p182_1913_lines,
        vec![0, 1, 2],
        "p182 pi=1913의 세 줄은 PDF owner와 같아야 함"
    );
    assert!(
        p183_1913_lines.is_empty(),
        "pi=1913이 p183으로 이월되면 이후 physical page owner가 연쇄적으로 밀림: {p183_1913_lines:?}"
    );

    let mut p182_figure_67 = None;
    let mut p183_figure_68 = None;
    table_bottom(&p182_tree.root, 1904, &mut p182_figure_67);
    table_bottom(&p183_tree.root, 1914, &mut p183_figure_68);
    assert!(p182_figure_67.is_some(), "p182 그림 67 table owner");
    assert!(p183_figure_68.is_some(), "p183 그림 68 table owner");
}

/// #3820 Stage 11: p199의 258) marker는 본문 tail에 있지만, 다음 문단이 raw
/// `vpos=0`으로 p200을 시작하고 기준 PDF의 258) FootnoteArea도 p200에 있다. 두 번째
/// note라는 이유만으로 p199에 붙이면 p200의 `pi=2310` reset tail이 footer 아래로
/// 그려져 p201 본문이 소실된다.
#[test]
fn native_hwp5_late_footnote_moves_to_next_reset_page_before_p200_tail() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage11 HWP evidence fixture");

    let p199_tree = doc
        .build_page_render_tree(PAGE_199)
        .expect("render physical page 199");
    let p200_tree = doc
        .build_page_render_tree(PAGE_200)
        .expect("render physical page 200");
    let p201_tree = doc
        .build_page_render_tree(PAGE_201)
        .expect("render physical page 201");
    let mut p199_footnotes = String::new();
    let mut p200_footnotes = String::new();
    footnote_text(&p199_tree.root, false, &mut p199_footnotes);
    footnote_text(&p200_tree.root, false, &mut p200_footnotes);
    assert!(
        !p199_footnotes.contains("258)"),
        "258) 각주는 PDF owner인 p200이 아니라 p199에 남으면 안 됨: {p199_footnotes}"
    );
    assert!(
        p200_footnotes.contains("258)"),
        "p200 FootnoteArea는 PDF처럼 258)을 보유해야 함: {p200_footnotes}"
    );

    let mut p200_2310_lines = Vec::new();
    let mut p201_2310_lines = Vec::new();
    paragraph_line_indices(&p200_tree.root, 2310, &mut p200_2310_lines);
    paragraph_line_indices(&p201_tree.root, 2310, &mut p201_2310_lines);
    assert_eq!(
        p200_2310_lines,
        vec![0],
        "p200은 reset 전 첫 줄만 두고 footer 아래로 tail을 그리면 안 됨"
    );
    assert_eq!(
        p201_2310_lines,
        vec![1, 2, 3, 4, 5],
        "p201은 PDF처럼 pi=2310 reset tail 다섯 줄부터 이어야 함"
    );
}

/// #3820 Stage 11: `pi=1806`의 1×1 RowBreak 표는 cell 안의 저장 vpos reset에서
/// 두 physical fragment로 나뉜다. PDF p172에는 `<BTS>`부터 `<OPTN>`까지가
/// 각주 219–221 바로 위에 있고, `간 특수 검사`부터는 p173에서 계속된다.
#[test]
fn native_hwp5_internal_reset_table_splits_at_p172_footnote_boundary() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage11 HWP evidence fixture");

    let p172 = page_text(&doc, PAGE_172);
    let p173 = page_text(&doc, PAGE_173);
    let p171 = page_text(&doc, PAGE_171);
    assert!(
        !p171.contains("<BTS>"),
        "p171에는 p172 소유의 pi=1806 first fragment가 앞당겨지면 안 됨: {p171}"
    );
    assert!(
        p172.contains("<BTS>") && p172.contains("<OPTN>") && !p172.contains("간 특수 검사"),
        "p172는 PDF처럼 pi=1806 reset 전 cell fragment를 보유해야 함: {p172}"
    );
    assert!(
        p173.contains("간 특수 검사") && p173.contains("지방변성의 여부"),
        "p173은 PDF처럼 pi=1806 reset tail부터 시작해야 함: {p173}"
    );

    let p172_tree = doc
        .build_page_render_tree(PAGE_172)
        .expect("render physical page 172");
    let p173_tree = doc
        .build_page_render_tree(PAGE_173)
        .expect("render physical page 173");
    let mut p172_table = None;
    let mut p173_table = None;
    let mut p172_preceding_table = None;
    table_bottom(&p172_tree.root, 1806, &mut p172_table);
    table_bottom(&p173_tree.root, 1806, &mut p173_table);
    table_bottom(&p172_tree.root, 1804, &mut p172_preceding_table);
    assert!(p172_table.is_some(), "p172 pi=1806 first fragment");
    assert!(p173_table.is_some(), "p173 pi=1806 continuation fragment");
    let mut p172_footnote_separator = None;
    footnote_separator_top(&p172_tree.root, &mut p172_footnote_separator);
    assert!(
        p172_table.is_some_and(|bottom| {
            p172_footnote_separator.is_some_and(|separator| bottom <= separator + 0.5)
        }),
        "p172 pi=1806 first fragment가 각주 영역과 겹치면 안 됨: previous={p172_preceding_table:?}, table={p172_table:?}, footnote={p172_footnote_separator:?}"
    );
}

#[test]
fn native_hwp5_footnote_reset_moves_only_the_overlapping_tail_to_the_next_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage12 HWP evidence fixture");

    let p30 = page_text(&doc, PAGE_30);
    let p31 = page_text(&doc, PAGE_31);
    let p32 = page_text(&doc, PAGE_32);
    assert!(
        p30.contains("10년 후 71.7%")
            && p30.contains("Dattani, Nikesh")
            && !p30.contains("문제가 나타남"),
        "p30은 각주 29와 그 위의 세 줄에서 끝나야 함: {p30}"
    );
    assert!(
        p31.contains("문제가 나타남")
            && p31.contains("5. 독일")
            && !p31.contains("Dattani, Nikesh"),
        "p31은 각주 29 없이 p30의 두 줄 tail 뒤에 독일 절로 이어져야 함: {p31}"
    );
    assert!(
        p32.contains("그림 35"),
        "각주 29를 p30으로 소급한 뒤에도 그림 35는 다음 페이지에 보존돼야 함: {p32}"
    );
}

#[test]
fn native_hwp5_existing_footnote_reset_moves_the_p43_tail_before_the_separator() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage23 HWP evidence fixture");

    let p43 = page_text(&doc, PAGE_43);
    let p44 = page_text(&doc, PAGE_44);
    assert!(
        p43.contains("여성이 1273명") && !p43.contains("(47.7%)이었음."),
        "p43은 PDF처럼 pi=512의 세 번째 줄에서 각주 전에 끝나야 함: {p43}"
    );
    assert!(
        p44.contains("(47.7%)이었음.") && p44.contains("이식대상자와의 관계는 다음 표와 같음"),
        "p44는 PDF처럼 pi=512 reset tail과 다음 본문으로 시작해야 함: {p44}"
    );

    let p43_tree = doc
        .build_page_render_tree(PAGE_43)
        .expect("render physical page 43");
    let mut p43_pi512_bottom = None;
    let mut p43_separator_top = None;
    paragraph_bottom(&p43_tree.root, 512, &mut p43_pi512_bottom);
    footnote_separator_top(&p43_tree.root, &mut p43_separator_top);
    assert!(
        p43_pi512_bottom.expect("p43 pi=512 body")
            <= p43_separator_top.expect("p43 footnote separator") + 0.5,
        "p43 pi=512 body tail must stay above the first footnote separator"
    );
    let mut p43_notes = String::new();
    footnote_text(&p43_tree.root, false, &mut p43_notes);
    for number in 39..=44 {
        assert!(
            p43_notes.contains(&format!("{number})")),
            "p43 must retain existing footnote {number}: {p43_notes}"
        );
    }

    let p44_tree = doc
        .build_page_render_tree(PAGE_44)
        .expect("render physical page 44");
    let mut p44_pi512_bottom = None;
    paragraph_bottom(&p44_tree.root, 512, &mut p44_pi512_bottom);
    assert!(p44_pi512_bottom.is_some(), "p44 must own pi=512 reset tail");
    let mut p44_notes = String::new();
    footnote_text(&p44_tree.root, false, &mut p44_notes);
    for number in 39..=44 {
        assert!(
            !p44_notes.contains(&format!("{number})")),
            "p44 must not inherit p43 footnote {number}: {p44_notes}"
        );
    }
}

#[test]
fn native_hwp5_current_marker_projects_the_p74_footnote_before_body_reset() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage99 HWP evidence fixture");

    assert_eq!(
        doc.page_count(),
        215,
        "정책연구 기준 PDF와 215쪽을 유지해야 함"
    );
    let p74 = page_text(&doc, PAGE_74);
    let p75 = page_text(&doc, PAGE_75);
    assert!(
        p74.contains("장기이식 환자 및 기증") && !p74.contains("자동화시스템"),
        "p74는 PDF처럼 para 839의 첫 줄에서 끝나야 함: {p74}"
    );
    assert!(
        p75.contains("자에 대한 정보를 관리하기 위한 자동화시스템"),
        "p75는 PDF처럼 para 839 reset 줄부터 시작해야 함: {p75}"
    );

    let p74_tree = doc
        .build_page_render_tree(PAGE_74)
        .expect("render physical page 74");
    let p75_tree = doc
        .build_page_render_tree(PAGE_75)
        .expect("render physical page 75");
    let mut p74_body_bottom = None;
    let mut p74_separator_top = None;
    paragraph_bottom(&p74_tree.root, 839, &mut p74_body_bottom);
    footnote_separator_top(&p74_tree.root, &mut p74_separator_top);
    assert!(
        p74_body_bottom.expect("p74 para 839 body")
            <= p74_separator_top.expect("p74 footnote separator") + 0.5,
        "p74 para 839는 projected FootnoteArea를 침범하면 안 됨"
    );
    let mut p75_body_bottom = None;
    paragraph_bottom(&p75_tree.root, 839, &mut p75_body_bottom);
    assert!(
        p75_body_bottom.is_some(),
        "p75가 para 839 reset tail을 소유해야 함"
    );

    let mut p74_notes = String::new();
    footnote_text(&p74_tree.root, false, &mut p74_notes);
    for number in [99, 100] {
        assert!(
            p74_notes.contains(&format!("{number})")),
            "p74 FootnoteArea가 note {number}를 소유해야 함: {p74_notes}"
        );
    }
}

#[test]
fn native_hwp5_earlier_marker_projects_the_p120_footnote_before_body_reset() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage102 HWP evidence fixture");

    assert_eq!(
        doc.page_count(),
        215,
        "정책연구 기준 PDF와 215쪽을 유지해야 함"
    );
    let p120 = page_text(&doc, PAGE_120);
    let p121 = page_text(&doc, PAGE_121);
    assert!(
        p120.contains("규정하고 있음.") && !p120.contains("A) 기증자가"),
        "p120은 PDF처럼 para 1293 reset 앞에서 끝나야 함: {p120}"
    );
    assert!(
        p121.contains("A) 기증자가 법적으로 가능한 연령이 되어야 하고"),
        "p121은 PDF처럼 para 1293 reset 줄부터 시작해야 함: {p121}"
    );

    let pages = [PAGE_120, PAGE_121, PAGE_121 + 1];
    let trees = pages.map(|page| {
        doc.build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render physical page {}: {e}", page + 1))
    });
    let expected_lines = [vec![0, 1, 2, 3], (4..14).collect::<Vec<_>>(), Vec::new()];
    for (index, tree) in trees.iter().enumerate() {
        let mut lines = Vec::new();
        paragraph_line_indices(&tree.root, 1293, &mut lines);
        lines.sort_unstable();
        assert_eq!(
            lines,
            expected_lines[index],
            "p{} para 1293 owner 또는 중복 line",
            pages[index] + 1
        );
    }

    let notes = trees.each_ref().map(|tree| {
        let mut text = String::new();
        footnote_text(&tree.root, false, &mut text);
        text
    });
    assert_footnote_owner(&notes, &pages, "158", 0, &["BOE-A-1979-26445"]);
    assert_footnote_owner(&notes, &pages, "159", 1, &["BOE-A-1980-5627"]);
    assert_footnote_owner(&notes, &pages, "160", 1, &["BOE-A-2000-79"]);
    assert!(
        notes[1].find("159)") < notes[1].find("160)"),
        "p121 각주 159가 160보다 먼저 와야 함: {}",
        notes[1]
    );

    let p120_tree = &trees[0];
    let p121_tree = &trees[1];
    let mut p120_body = None;
    body_bbox(&p120_tree.root, &mut p120_body);
    let p120_body = p120_body.expect("p120 body bbox");
    let mut p120_table = Vec::new();
    table_boxes_for_paragraph(&p120_tree.root, 1283, &mut p120_table);
    assert_eq!(p120_table.len(), 1, "p120 pi1283 whole table owner");
    let p120_table = p120_table[0];
    let outer_margin = hwpunit_to_px(283, DEFAULT_DPI);
    let expected_width = hwpunit_to_px(41_954, DEFAULT_DPI);
    let expected_height = hwpunit_to_px(23_790, DEFAULT_DPI);
    assert!(
        (p120_table.x - p120_body.x - outer_margin).abs() <= 0.2,
        "p120 pi1283 left는 body origin + outer-left 283HU여야 함: body={p120_body:?}, table={p120_table:?}"
    );
    assert!(
        (p120_table.y - p120_body.y - outer_margin).abs() <= 0.2,
        "p120 pi1283 top은 body origin + outer-top 283HU여야 함: body={p120_body:?}, table={p120_table:?}"
    );
    assert!(
        (p120_table.width - expected_width).abs() <= 0.2
            && (p120_table.height - expected_height).abs() <= 0.2,
        "p120 pi1283 declared size는 이동 뒤에도 불변이어야 함: {p120_table:?}"
    );
    let mut p1286_lines = Vec::new();
    paragraph_line_boxes(&p120_tree.root, 1286, &mut p1286_lines);
    assert_eq!(p1286_lines.len(), 1, "p120 pi1286 title line owner");
    assert!(
        (p1286_lines[0].y - 461.2).abs() <= 0.2,
        "표 paint inset이 다음 본문 flow를 이동시키면 안 됨: {:?}",
        p1286_lines[0]
    );

    let mut p120_body_bottom = None;
    let mut p120_separator_top = None;
    paragraph_bottom(&p120_tree.root, 1293, &mut p120_body_bottom);
    footnote_separator_top(&p120_tree.root, &mut p120_separator_top);
    assert!(
        p120_body_bottom.expect("p120 para 1293 body")
            <= p120_separator_top.expect("p120 footnote separator") + 0.5,
        "p120 para 1293은 projected FootnoteArea를 침범하면 안 됨"
    );
    let mut p121_body_bottom = None;
    let mut p121_separator_top = None;
    paragraph_bottom(&p121_tree.root, 1297, &mut p121_body_bottom);
    footnote_separator_top(&p121_tree.root, &mut p121_separator_top);
    assert!(
        p121_body_bottom.expect("p121 para 1297 body")
            <= p121_separator_top.expect("p121 footnote separator") + 0.5,
        "p121 para 1297은 FootnoteArea를 침범하면 안 됨"
    );
    for (physical_page, tree) in [(120, p120_tree), (121, p121_tree)] {
        let mut body = None;
        body_bbox(&tree.root, &mut body);
        let body = body.expect("body bbox");
        let mut separator = None;
        footnote_separator_bbox(&tree.root, &mut separator);
        let separator = separator.expect("footnote separator bbox");
        let expected_five_cm = DEFAULT_DPI * 5.0 / 2.54;
        assert!(
            (separator.x - body.x).abs() <= 0.05,
            "p{physical_page} footnote separator start x는 body와 같아야 함: body={body:?}, separator={separator:?}"
        );
        assert!(
            (separator.width - expected_five_cm).abs() <= 0.05,
            "p{physical_page} separatorLength=-1은 5cm여야 함: separator={separator:?}, expected={expected_five_cm}"
        );
        let mut footnote_bottom = None;
        let mut footer_top = None;
        footnote_and_footer(&tree.root, &mut footnote_bottom, &mut footer_top);
        assert!(
            footnote_bottom.expect("footnote bottom") <= footer_top.expect("footer top") + 1.0,
            "p{physical_page} FootnoteArea가 footer를 침범하면 안 됨"
        );
    }
}

#[test]
fn native_hwp5_body_footnotes_follow_the_p129_and_p131_reset_pages() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage103 HWP evidence fixture");

    assert_eq!(
        doc.page_count(),
        215,
        "정책연구 기준 PDF와 215쪽을 유지해야 함"
    );
    let pages = [PAGE_129, PAGE_130, PAGE_131, PAGE_132, PAGE_132 + 1];
    let trees = pages.map(|page| {
        doc.build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render physical page {}: {e}", page + 1))
    });

    let expected_1372 = [
        (0..6).collect::<Vec<_>>(),
        (6..9).collect::<Vec<_>>(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];
    let expected_1382 = [Vec::new(), Vec::new(), vec![0, 1], vec![2], Vec::new()];
    for (index, tree) in trees.iter().enumerate() {
        let mut lines_1372 = Vec::new();
        let mut lines_1382 = Vec::new();
        paragraph_line_indices(&tree.root, 1372, &mut lines_1372);
        paragraph_line_indices(&tree.root, 1382, &mut lines_1382);
        lines_1372.sort_unstable();
        lines_1382.sort_unstable();
        assert_eq!(
            lines_1372,
            expected_1372[index],
            "p{} para 1372 owner 또는 중복 line",
            pages[index] + 1
        );
        assert_eq!(
            lines_1382,
            expected_1382[index],
            "p{} para 1382 owner 또는 중복 line",
            pages[index] + 1
        );
    }

    let mut table_1377_boxes = Vec::new();
    let mut table_1379_boxes = Vec::new();
    for tree in &trees {
        let mut boxes_1377 = Vec::new();
        let mut boxes_1379 = Vec::new();
        table_boxes_for_paragraph(&tree.root, 1377, &mut boxes_1377);
        table_boxes_for_paragraph(&tree.root, 1379, &mut boxes_1379);
        table_1377_boxes.push(boxes_1377);
        table_1379_boxes.push(boxes_1379);
    }
    assert_eq!(
        table_1377_boxes.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![0, 0, 1, 0, 0],
        "pi1377 표는 p131에만 정확히 한 번 있어야 함"
    );
    assert_eq!(
        table_1379_boxes.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![0, 0, 1, 0, 0],
        "pi1379 표는 p131에만 정확히 한 번 있어야 함"
    );

    let p131_table_1377 = table_1377_boxes[2][0];
    let p131_table_1379 = table_1379_boxes[2][0];
    let mut p131_body_boxes = Vec::new();
    paragraph_line_boxes(&trees[2].root, 1382, &mut p131_body_boxes);
    let p131_body_top = p131_body_boxes
        .iter()
        .map(|bbox| bbox.y)
        .reduce(f64::min)
        .expect("p131 para 1382 lines");
    assert!(
        p131_table_1377.y + p131_table_1377.height <= p131_table_1379.y + 0.5,
        "p131 pi1377 표는 pi1379 표를 침범하면 안 됨"
    );
    assert!(
        p131_table_1379.y + p131_table_1379.height <= p131_body_top + 0.5,
        "p131 pi1379 표는 pi1382 본문을 침범하면 안 됨"
    );

    let mut notes = [
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];
    for (tree, text) in trees.iter().zip(notes.iter_mut()) {
        footnote_text(&tree.root, false, text);
    }
    assert!(
        notes[0].contains("176)") && notes[0].contains("일상생"),
        "p129는 각주 176의 번호와 reset 전 prefix를 소유해야 함: {}",
        notes[0]
    );
    assert!(
        !notes[0].contains("활이나 직업적 활동"),
        "p129는 각주 176 reset tail을 미리 소유하면 안 됨: {}",
        notes[0]
    );
    assert!(
        notes[1].contains("활이나 직업적 활동")
            && notes[1].contains("177)")
            && notes[1].contains("178)")
            && !notes[1].contains("176)"),
        "p130은 번호를 반복하지 않은 각주 176 tail과 177·178을 소유해야 함: {}",
        notes[1]
    );
    assert_footnote_owner(&notes, &pages, "179", 2, &["KAKENHI-PROJECT-24593293"]);
    assert_footnote_owner(
        &notes,
        &pages,
        "180",
        3,
        &["본인 확인뿐만 아니라", "대로 호적 등으로"],
    );
    assert_footnote_owner(&notes, &pages, "181", 3, &["hishinzoku.pdf"]);

    for index in [2, 3] {
        let physical_page = pages[index] + 1;
        let mut body_bottom = None;
        let mut separator_top = None;
        let mut footnote_bottom = None;
        let mut footer_top = None;
        paragraph_bottom(&trees[index].root, 1382, &mut body_bottom);
        footnote_separator_top(&trees[index].root, &mut separator_top);
        footnote_and_footer(&trees[index].root, &mut footnote_bottom, &mut footer_top);
        let separator_top = separator_top.expect("p131/p132 footnote separator");
        assert!(
            body_bottom.expect("p131/p132 para 1382 body") <= separator_top + 0.5,
            "p{physical_page} pi1382 본문은 각주 separator를 침범하면 안 됨"
        );
        assert!(
            footnote_bottom.expect("p131/p132 footnote bottom")
                <= footer_top.expect("p131/p132 footer top") + 1.0,
            "p{physical_page} FootnoteArea는 footer를 침범하면 안 됨"
        );
        if index == 2 {
            assert!(
                p131_table_1377.y + p131_table_1377.height <= separator_top + 0.5
                    && p131_table_1379.y + p131_table_1379.height <= separator_top + 0.5,
                "p131 표는 각주 separator를 침범하면 안 됨"
            );
        }
    }
}

#[test]
fn native_hwp5_repeated_zero_footnotes_continue_on_p177_and_p179() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage104 HWP evidence fixture");

    assert_eq!(
        doc.page_count(),
        215,
        "정책연구 기준 PDF와 215쪽을 유지해야 함"
    );
    let trees = [PAGE_176, PAGE_177, PAGE_178, PAGE_179].map(|page| {
        doc.build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render physical page {}: {e}", page + 1))
    });
    let mut notes = [String::new(), String::new(), String::new(), String::new()];
    for (tree, text) in trees.iter().zip(notes.iter_mut()) {
        footnote_text(&tree.root, false, text);
    }

    assert!(
        notes[0].contains("234)")
            && notes[0].contains("using moderately and")
            && !notes[0].contains("severely steatotic donor livers"),
        "p176은 table-cell 각주 234의 첫 stored line만 소유해야 함: {}",
        notes[0]
    );
    assert!(
        !notes[1].contains("234)")
            && notes[1].contains("severely steatotic donor livers")
            && notes[1].contains("235)"),
        "p177은 번호를 반복하지 않은 각주 234 tail과 235를 소유해야 함: {}",
        notes[1]
    );
    assert!(
        notes[2].contains("240)")
            && notes[2].contains("이식대상자도")
            && !notes[2].contains("HTLV-1")
            && !notes[2].contains("양성인 경우에는 별도로 검토함"),
        "p178은 body 각주 240의 첫 stored line만 소유해야 함: {}",
        notes[2]
    );
    assert!(
        !notes[3].contains("240)")
            && notes[3].contains("HTLV-1")
            && notes[3].contains("양성인 경우에는 별도로 검토함")
            && notes[3].contains("jikeisurgery.jp")
            && notes[3].contains("241)")
            && notes[3].contains("242)"),
        "p179는 번호를 반복하지 않은 각주 240 tail과 241·242를 소유해야 함: {}",
        notes[3]
    );

    let mut p178_body_bottom = None;
    let mut p178_separator_top = None;
    paragraph_bottom(&trees[2].root, 1865, &mut p178_body_bottom);
    footnote_separator_top(&trees[2].root, &mut p178_separator_top);
    assert!(
        p178_body_bottom.expect("p178 para 1865 body")
            <= p178_separator_top.expect("p178 footnote separator") + 0.5,
        "p178 para 1865 본문은 각주 236~240 prefix 영역을 침범하면 안 됨"
    );
}

#[test]
fn native_hwp5_final_marker_footnote_uses_the_next_reset_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage27 HWP evidence fixture");

    let p26 = page_text(&doc, PAGE_26);
    let p27 = page_text(&doc, PAGE_27);
    assert!(
        p26.contains("북부와 서부 지역이 동부 및 지중해 지역보다 더 빈번하게 수행됨.26)"),
        "p26 must retain the body tail and marker 26): {p26}"
    );
    assert!(
        !p26.contains("11번 참고문헌 내 Adam et al 논문"),
        "p26 must not own footnote 26 after its final marker: {p26}"
    );
    assert!(
        p27.contains("26)   11번 참고문헌 내 Adam et al 논문"),
        "p27 must own the complete footnote 26 before its following body: {p27}"
    );
    assert!(
        p27.contains("1991년부터 2013년까지의 ELTR 자료"),
        "p27 must retain its existing body restart after footnote 26: {p27}"
    );
    assert!(
        doc.page_count() <= 219,
        "p26 footnote owner는 extra native page를 만들면 안 됨: {}쪽",
        doc.page_count()
    );

    let p26_tree = doc
        .build_page_render_tree(PAGE_26)
        .expect("render physical page 26");
    let p27_tree = doc
        .build_page_render_tree(PAGE_27)
        .expect("render physical page 27");
    let mut p26_notes = String::new();
    let mut p27_notes = String::new();
    footnote_text(&p26_tree.root, false, &mut p26_notes);
    footnote_text(&p27_tree.root, false, &mut p27_notes);
    assert!(
        !p26_notes.contains("Adam et al"),
        "p26 FootnoteArea must be empty of note 26: {p26_notes}"
    );
    assert!(
        p27_notes.contains("Adam et al"),
        "p27 FootnoteArea must own note 26: {p27_notes}"
    );
}

#[test]
fn native_hwp5_split_body_footnotes_stay_with_their_marker_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage28 HWP evidence fixture");

    let p52 = page_text(&doc, PAGE_52);
    let p53 = page_text(&doc, PAGE_53);
    let p54 = page_text(&doc, PAGE_54);
    assert!(
        p52.contains("60)   http://www.who.int/transplantation/publications/ConsensusStatementShort.pdf?ua=1"),
        "p52 must retain footnote 60 with its split-body marker: {p52}"
    );
    assert!(
        !p53.contains("ConsensusStatementShort.pdf?ua=1"),
        "p53 must not inherit p52 footnote 60: {p53}"
    );
    assert!(
        p53.contains("62)   Lentine, Krista L., et al. \"KDIGO clinical practice guideline"),
        "p53 must retain footnote 62 with its split-body marker: {p53}"
    );
    assert!(
        !p54.contains("KDIGO clinical practice guideline"),
        "p54 must not inherit p53 footnote 62: {p54}"
    );
    assert!(
        doc.page_count() <= 219,
        "marker-page footnote routing은 extra native page를 만들면 안 됨: {}쪽",
        doc.page_count()
    );

    let p52_tree = doc
        .build_page_render_tree(PAGE_52)
        .expect("render physical page 52");
    let p53_tree = doc
        .build_page_render_tree(PAGE_53)
        .expect("render physical page 53");
    let p54_tree = doc
        .build_page_render_tree(PAGE_54)
        .expect("render physical page 54");
    let mut p52_notes = String::new();
    let mut p53_notes = String::new();
    let mut p54_notes = String::new();
    footnote_text(&p52_tree.root, false, &mut p52_notes);
    footnote_text(&p53_tree.root, false, &mut p53_notes);
    footnote_text(&p54_tree.root, false, &mut p54_notes);
    assert!(
        p52_notes.contains("ConsensusStatementShort.pdf?ua=1"),
        "p52 FootnoteArea must own note 60: {p52_notes}"
    );
    assert!(
        p53_notes.contains("KDIGO clinical practice guideline"),
        "p53 FootnoteArea must own note 62: {p53_notes}"
    );
    assert!(
        !p54_notes.contains("KDIGO clinical practice guideline"),
        "p54 FootnoteArea must not own note 62: {p54_notes}"
    );

    // completed page에 각주를 소급 등록하는 경로는 본문을 다시 paginate하지
    // 않으므로, marker가 든 마지막 body line과 새 FootnoteArea separator가
    // 실제로 겹치지 않는 것도 고정한다.
    for (page_name, tree, para_index) in [("p52", &p52_tree, 602), ("p53", &p53_tree, 605)] {
        let mut body_bottom = None;
        let mut separator_top = None;
        paragraph_bottom(&tree.root, para_index, &mut body_bottom);
        footnote_separator_top(&tree.root, &mut separator_top);
        assert!(
            body_bottom.expect("split body paragraph")
                <= separator_top.expect("footnote separator") + 0.5,
            "{page_name} marker body must remain above its retroactive FootnoteArea"
        );
    }
}

#[test]
fn native_hwp5_repeated_empty_guide_lines_emit_tac_picture_once() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage18 HWP evidence fixture");
    let tree = doc
        .build_page_render_tree(PAGE_37)
        .expect("render physical page 37");

    // pi=463 control 1은 그림 37이다. text-start가 같은 빈 guide 줄이 둘이지만
    // 이 control은 하나뿐이므로 첫 줄에만 귀속되어야 한다.
    let mut positions = Vec::new();
    images_for_control(&tree.root, 463, 1, &mut positions);
    assert_eq!(
        positions.len(),
        1,
        "p37 그림 37은 한 번만 방출되어야 한다: {positions:?}"
    );
    let (x, y) = positions[0];
    assert!(
        x < 350.0 && y < 800.0,
        "그림 37은 PDF처럼 좌측의 두-그림 band에 있어야 하며 페이지 하단 fallback으로 새면 안 된다: x={x:.1}, y={y:.1}"
    );
}

#[test]
fn native_hwp5_same_page_stale_rowbreak_picture_keeps_figure_25_visible() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage19 HWP evidence fixture");
    let p25 = page_text(&doc, PAGE_25);
    assert!(
        p25.contains("그림 25.") && p25.contains("그림 26."),
        "p25에는 PDF처럼 그림 25와 그림 26의 caption이 함께 있어야 한다: {p25}"
    );

    let tree = doc
        .build_page_render_tree(PAGE_25)
        .expect("render physical page 25");
    let mut positions = Vec::new();
    // pi=357은 그림 25를 담은 빈 1×1 RowBreak 표다. stale -50000 HU를 그대로
    // 적용하면 Image가 p25 위쪽 밖(y<0)으로 나가 PDF에 있는 첫 그림이 사라진다.
    images_for_table(&tree.root, 357, &mut positions);
    assert_eq!(
        positions.len(),
        1,
        "p25 그림 25 표는 Image를 정확히 하나 방출해야 한다: {positions:?}"
    );
    let (x, y) = positions[0];
    assert!(
        x > 100.0 && y >= 240.0 && y < 360.0,
        "그림 25는 PDF처럼 p25 표 frame 내부에 있어야 한다: x={x:.1}, y={y:.1}"
    );
}

#[test]
fn picture_caption_rowbreak_uses_the_actual_footnote_boundary_before_deferring() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage13 HWP evidence fixture");

    let p68 = page_text(&doc, PAGE_68);
    let p69 = page_text(&doc, PAGE_69);
    assert!(
        p68.contains("그림 49. OPTN 생존 장기기증 원칙"),
        "p68에는 그림 49와 caption이 각주 위에 남아야 함: {p68}"
    );
    assert!(
        !p69.contains("그림 49. OPTN 생존 장기기증 원칙")
            && p69.contains("나. 생존 장기기증 승인 절차"),
        "p69는 그림 49 없이 다음 본문으로 시작해야 함: {p69}"
    );
}

#[test]
fn native_hwp5_reset_tail_uses_the_actual_existing_footnote_boundary() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage14 HWP evidence fixture");

    let p58 = page_text(&doc, PAGE_58);
    let p59 = page_text(&doc, PAGE_59);
    assert!(
        p58.contains("호주 정부의 국민 건강 및 의료 연구 협의회")
            && p58.contains("Medical Research Council")
            && !p58.contains("독립적이며 적절한 지식과 기술"),
        "p58은 각주 70 위에 stored reset 전 세 줄을 보유해야 함: {p58}"
    );
    assert!(
        p59.contains("독립적이며 적절한 지식과 기술")
            && !p59.contains("호주 정부의 국민 건강 및 의료 연구 협의회"),
        "p59는 reset 뒤의 본문부터 재개해야 함: {p59}"
    );
}

#[test]
fn native_hwp5_rowbreak_tail_keeps_figure_51_with_its_pdf_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage15 HWP evidence fixture");

    let p76 = page_text(&doc, PAGE_76);
    let p77 = page_text(&doc, PAGE_77);
    let p78 = page_text(&doc, PAGE_78);
    let p79 = page_text(&doc, PAGE_79);
    assert!(
        p76.contains("생존 신장 기증자가") && p76.contains("위한 대기자 목록에 올라가거나,"),
        "p76은 표 24 row 4 reset 앞의 세 줄을 보유해야 함: {p76}"
    );
    assert!(
        p77.contains("투석을 시작하게 된 경우")
            && !p77.contains("후 2년 내에 신장 이식을 받기")
            && p77.contains("그림 51.")
            && !p77.contains("3. EU"),
        "p77은 표 24 row 4 tail 뒤에 그림 51을 각주 위에 포함해야 함: {p77}"
    );
    assert!(
        p78.contains("3. EU") && !p78.contains("그림 51."),
        "그림 51 단독 page가 제거되면 p78은 다음 본문으로 재개해야 함: {p78}"
    );
    assert!(
        !p79.trim().is_empty(),
        "p79은 연쇄 이월 때문에 빈 표 전용 page가 되어서는 안 됨"
    );
}

#[test]
fn native_hwp5_two_line_footnote_continues_after_the_reset_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage16 HWP evidence fixture");

    let p31 = page_text(&doc, PAGE_31);
    let p32 = page_text(&doc, PAGE_32);
    assert!(
        p31.contains("Aktuelle Entwicklungen") && !p31.contains("incentives"),
        "p31은 각주 30의 첫 줄만 보유해야 함: {p31}"
    );
    assert!(
        p32.contains("incentives") && !p32.contains("Aktuelle Entwicklungen"),
        "p32는 각주 30의 연속 tail만 보유해야 함: {p32}"
    );

    let tree = doc
        .build_page_render_tree(PAGE_31)
        .unwrap_or_else(|e| panic!("render physical page 31: {e}"));
    let mut body_bottom = None;
    let mut separator_top = None;
    paragraph_bottom(&tree.root, 421, &mut body_bottom);
    footnote_separator_top(&tree.root, &mut separator_top);
    assert!(
        body_bottom.expect("p31 para 421") <= separator_top.expect("p31 footnote separator") + 0.5,
        "p31 본문과 각주 separator가 겹치면 안 됨: body_bottom={body_bottom:?}, separator={separator_top:?}"
    );
}

#[test]
fn native_hwp5_large_rowbreak_table_keeps_its_first_fragment_before_cell_footnotes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage17 HWP evidence fixture");

    let p78 = page_text(&doc, PAGE_78);
    let p79 = page_text(&doc, PAGE_79);
    let p80 = page_text(&doc, PAGE_80);
    assert!(
        p78.contains("Convention") && p78.contains("Directive"),
        "p78은 표 25의 Convention·Directive first fragment를 보유해야 함: {p78}"
    );
    assert!(
        p79.contains("Recommendation") && p79.contains("CM/Res(2017)1"),
        "p79는 표 25의 Resolution/Recommendation continuation을 보유해야 함: {p79}"
    );
    assert!(
        p80.contains("유럽의회(European Parliament)") && !p80.contains("시행법은"),
        "p80은 표 25 continuation이 아니라 PDF처럼 본문으로 재개해야 함: {p80}"
    );

    // 표 25의 URL 각주는 source-cell 순서가 아니라 실제 물리 fragment page별로
    // 분할된다. p78의 기존 105·106, p79의 107–111, p80의 112–124 경계를
    // 고정해 한 fragment에 과예약해 다음 본문을 밀어내는 회귀를 막는다.
    let p78_tree = doc
        .build_page_render_tree(PAGE_78)
        .expect("render physical page 78");
    let p79_tree = doc
        .build_page_render_tree(PAGE_79)
        .expect("render physical page 79");
    let p80_tree = doc
        .build_page_render_tree(PAGE_80)
        .expect("render physical page 80");
    let mut p78_notes = String::new();
    let mut p79_notes = String::new();
    let mut p80_notes = String::new();
    footnote_text(&p78_tree.root, false, &mut p78_notes);
    footnote_text(&p79_tree.root, false, &mut p79_notes);
    footnote_text(&p80_tree.root, false, &mut p80_notes);
    for number in [105, 106] {
        assert!(
            p78_notes.contains(&format!("{number})")),
            "p78 각주 {number} 누락: {p78_notes}"
        );
    }
    assert!(
        !p78_notes.contains("107)"),
        "p78에는 표 cell 각주 107이 앞당겨지면 안 됨: {p78_notes}"
    );
    for number in 107..=111 {
        assert!(
            p79_notes.contains(&format!("{number})")),
            "p79 각주 {number} 누락: {p79_notes}"
        );
    }
    assert!(
        !p79_notes.contains("112)"),
        "p79에는 각주 112가 앞당겨지면 안 됨: {p79_notes}"
    );
    for number in 112..=124 {
        assert!(
            p80_notes.contains(&format!("{number})")),
            "p80 각주 {number} 누락: {p80_notes}"
        );
    }

    for (page, tree) in [(78, &p78_tree), (79, &p79_tree)] {
        let mut table = None;
        let mut separator = None;
        table_bottom(&tree.root, 885, &mut table);
        footnote_separator_top(&tree.root, &mut separator);
        assert!(
            table.expect("표 25") <= separator.expect("표 25 각주 separator") + 0.5,
            "p{page} 표 25 하단과 각주 separator가 겹치면 안 됨: table_bottom={table:?}, separator={separator:?}"
        );
    }
    assert!(
        p80.contains("유럽평의회는 2007년 5월 30일")
            && p80.contains("2007년 커뮤니케이션에 대한 대응으로"),
        "p80의 두 후속 본문이 각주 112–124 예약 때문에 p81로 밀리면 안 됨: {p80}"
    );
    let mut p80_body_bottom = None;
    let mut p80_separator = None;
    paragraph_bottom(&p80_tree.root, 889, &mut p80_body_bottom);
    footnote_separator_top(&p80_tree.root, &mut p80_separator);
    assert!(
        p80_body_bottom.expect("p80 para 889")
            <= p80_separator.expect("p80 footnote separator") + 0.5,
        "p80 표 25 뒤 본문과 각주 112 separator가 겹치면 안 됨: body_bottom={p80_body_bottom:?}, separator={p80_separator:?}"
    );
}

#[test]
fn native_hwp5_empty_rowbreak_table_uses_the_actual_existing_footnote_boundary() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage21 HWP evidence fixture");

    // pi=1682는 본문 각주 210 위에 통째로 들어간다. 40px safety margin을
    // 기계적으로 남기면 마지막 두 줄만 p155로 밀려 이후 물리 페이지가 전부 +1
    // shift 된다. Hancom PDF처럼 p154에서 표와 각주가 함께 끝나야 한다.
    let p154 = page_text(&doc, PAGE_154);
    let p155 = page_text(&doc, PAGE_155);
    assert!(
        p154.contains("생존 기증자가 모든 위험과 이익"),
        "p154에는 pi=1682의 마지막 셀 문단이 각주 210 위에 남아야 함: {p154}"
    );
    assert!(
        p155.trim_start().starts_with("(3) 평가 절차")
            && !p155.contains("생존 기증자가 모든 위험과 이익"),
        "p155는 pi=1682 tail 전용 페이지가 아니라 다음 절로 시작해야 함: {p155}"
    );

    let tree = doc
        .build_page_render_tree(PAGE_154)
        .expect("render physical page 154");
    let mut table = None;
    let mut separator = None;
    table_bottom(&tree.root, 1682, &mut table);
    footnote_separator_top(&tree.root, &mut separator);
    assert!(
        table.expect("p154 pi=1682") <= separator.expect("p154 footnote separator") + 0.5,
        "p154 pi=1682 하단과 기존 각주 separator가 겹치면 안 됨: table={table:?}, separator={separator:?}"
    );

    let p155_tree = doc
        .build_page_render_tree(PAGE_155)
        .expect("render physical page 155");
    let mut stale_tail = None;
    table_bottom(&p155_tree.root, 1682, &mut stale_tail);
    assert!(
        stale_tail.is_none(),
        "p155에는 pi=1682의 tail fragment가 남으면 안 됨: {stale_tail:?}"
    );
}

#[test]
fn native_hwp5_oversized_single_rowbreak_table_splits_inside_the_page_frame() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage21 HWP evidence fixture");

    // pi=1723은 선언 높이 363.8px보다 셀 본문 측정 높이가 1163.8px인 1×1
    // RowBreak 표다. 선언 높이만 예약하는 빈-anchor fast lane을 타면 p158
    // frame 밖으로 700px 이상 새므로, p157/p158의 두 fragment로 이어져야 한다.
    let p157 = page_text(&doc, PAGE_157);
    let p158 = page_text(&doc, PAGE_158);
    assert!(
        p157.contains("<BTS Guideline>") && p157.contains("<OPTN policy>"),
        "p157에는 표 37의 첫 fragment가 있어야 함: {p157}"
    );
    assert!(
        p158.contains("<BC Canada>") && p158.contains("신체 검진은 체중"),
        "p158에는 표 37의 continuation과 뒤 본문이 함께 있어야 함: {p158}"
    );

    for (page, label) in [(PAGE_157, "p157"), (PAGE_158, "p158")] {
        let tree = doc
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render physical {label}: {e}"));
        let mut table = None;
        let mut footnote = None;
        let mut footer = None;
        table_bottom(&tree.root, 1723, &mut table);
        footnote_and_footer(&tree.root, &mut footnote, &mut footer);
        assert!(
            table.expect("pi=1723 fragment") <= footer.expect("page footer") + 0.5,
            "{label} pi=1723 fragment가 footer 밖으로 넘으면 안 됨: table={table:?}, footer={footer:?}"
        );
    }
}

#[test]
fn native_hwp5_square_picture_uses_the_next_page_wrap_owner() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage22 HWP evidence fixture");

    // 그림 64의 anchor(pi=1692)와 p1693의 첫 두 줄은 p155에 남는다. 그러나
    // Square 그림+caption은 native HWP5의 다음 physical-page wrap owner(p156)에
    // 속한다. anchor 문단에서 즉시 PageItem을 만들면 p155의 표·본문·각주 211을
    // 덮는 회귀가 난다.
    let p155 = page_text(&doc, PAGE_155);
    let p156 = page_text(&doc, PAGE_156);
    assert!(
        p155.contains("일본 각 병원에서 일반적으로 진행되는 절차")
            && p155.contains("구마모토대는 문진과 진찰"),
        "p155에는 그림 anchor 본문과 p1693의 현재 쪽 두 줄이 남아야 함: {p155}"
    );
    assert!(
        !p155.contains("그림 64."),
        "p155에는 그림 64 caption이 남아 표·본문·각주를 덮으면 안 됨: {p155}"
    );
    assert!(
        p156.contains("상 금주 및 금연") && p156.contains("그림 64."),
        "p156은 p1693 narrow-wrap continuation과 그림 64 caption을 함께 가져야 함: {p156}"
    );

    let p155_tree = doc
        .build_page_render_tree(PAGE_155)
        .expect("render physical page 155");
    let p156_tree = doc
        .build_page_render_tree(PAGE_156)
        .expect("render physical page 156");
    let mut p155_images = Vec::new();
    let mut p156_images = Vec::new();
    images_for_control(&p155_tree.root, 1692, 1, &mut p155_images);
    images_for_control(&p156_tree.root, 1692, 1, &mut p156_images);
    assert!(
        p155_images.is_empty(),
        "p155 그림 64 Image가 표/각주 영역에 남으면 안 됨: {p155_images:?}"
    );
    assert_eq!(
        p156_images.len(),
        1,
        "p156에는 그림 64 Image가 정확히 하나 있어야 함: {p156_images:?}"
    );
    assert!(
        p156_images[0].0 > 400.0,
        "그림 64는 PDF처럼 p156 우측 Square band에 있어야 함: {:?}",
        p156_images[0]
    );
    assert!(
        (p156_images[0].1 - 90.1).abs() <= 1.0,
        "p156 그림 64는 full-width tail 뒤 reset contract의 518HU offset을 유지해야 함: {:?}",
        p156_images[0]
    );

    let mut p156_image_boxes = Vec::new();
    let mut p156_pi1693_lines = Vec::new();
    image_boxes_for_control(&p156_tree.root, 1692, 1, &mut p156_image_boxes);
    paragraph_line_boxes(&p156_tree.root, 1693, &mut p156_pi1693_lines);
    let image = p156_image_boxes
        .into_iter()
        .next()
        .expect("p156 그림 64 bbox");
    let overlapping_vertical_lines: Vec<_> = p156_pi1693_lines
        .into_iter()
        .filter(|line| vertically_intersects(*line, image))
        .collect();
    assert!(
        !overlapping_vertical_lines.is_empty(),
        "p156 pi=1693에는 그림 64와 같은 세로 band의 Square 본문이 있어야 함"
    );
    assert!(
        overlapping_vertical_lines
            .iter()
            .all(|line| does_not_overlap_horizontally(*line, image)),
        "p156 pi=1693 본문은 그림 64와 물리적으로 교차하면 안 됨: image={image:?}, lines={overlapping_vertical_lines:?}"
    );
}

#[test]
fn native_hwp5_text_tail_before_figure_55_keeps_the_pdf_page_owner() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse #3820 HWP evidence fixture");

    let p118_tree = doc
        .build_page_render_tree(PAGE_118)
        .expect("render physical page 118");
    let p119_tree = doc
        .build_page_render_tree(PAGE_119)
        .expect("render physical page 119");
    let mut p118_lines = Vec::new();
    let mut p119_lines = Vec::new();
    paragraph_line_indices(&p118_tree.root, 1275, &mut p118_lines);
    paragraph_line_indices(&p119_tree.root, 1275, &mut p119_lines);
    p118_lines.sort_unstable();
    p119_lines.sort_unstable();
    assert_eq!(
        p118_lines,
        (0..9).collect::<Vec<_>>(),
        "#3820 p118은 Figure 55 앞 pi=1275의 앞 9 stored lines에서 끝나야 함"
    );
    assert_eq!(
        p119_lines,
        vec![9, 10],
        "#3820 p119은 Figure 55보다 먼저 pi=1275 tail 두 줄을 이어야 함"
    );

    let mut p118_images = Vec::new();
    let mut p119_images = Vec::new();
    images_for_control(&p118_tree.root, 1276, 0, &mut p118_images);
    images_for_control(&p119_tree.root, 1276, 0, &mut p119_images);
    assert!(
        p118_images.is_empty(),
        "#3820 그림 55는 p118에 앞당겨지면 안 됨: {p118_images:?}"
    );
    assert_eq!(
        p119_images.len(),
        1,
        "#3820 p119은 pi=1275 tail 뒤 그림 55를 정확히 한 번 그려야 함: {p119_images:?}"
    );
}

#[test]
fn native_hwp5_square_picture_figure_56_uses_the_same_next_page_owner_contract() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc =
        HwpDocument::from_bytes(&bytes).expect("parse stage22 secondary HWP evidence fixture");

    // pi=1355와 p1356의 첫 vpos=0 narrow line은 그림 56에도 같은 HWP5 physical
    // owner contract가 있음을 보여 준다. PDF p126은 anchor 본문만, p127은 오른쪽
    // 그림 56과 좁은 본문 흐름을 가진다.
    let p126 = page_text(&doc, PAGE_126);
    let p127 = page_text(&doc, PAGE_127);
    assert!(
        p126.contains("한국의 장기이식관리센터") && !p126.contains("그림 56."),
        "p126에는 그림 56 caption이 남아 각주 170–172를 덮으면 안 됨: {p126}"
    );
    assert!(
        p127.contains("일반적으로 진행되는 절차는 오른쪽 그림") && p127.contains("그림 56."),
        "p127은 그림 56의 Square wrap 본문과 caption을 함께 가져야 함: {p127}"
    );

    let p126_tree = doc
        .build_page_render_tree(PAGE_126)
        .expect("render physical page 126");
    let p127_tree = doc
        .build_page_render_tree(PAGE_127)
        .expect("render physical page 127");
    let mut p126_images = Vec::new();
    let mut p127_images = Vec::new();
    images_for_control(&p126_tree.root, 1355, 0, &mut p126_images);
    images_for_control(&p127_tree.root, 1355, 0, &mut p127_images);
    assert!(
        p126_images.is_empty(),
        "p126 그림 56 Image가 anchor page에 남으면 안 됨: {p126_images:?}"
    );
    assert_eq!(
        p127_images.len(),
        1,
        "p127에는 그림 56 Image가 정확히 하나 있어야 함: {p127_images:?}"
    );
    assert!(
        p127_images[0].0 > 390.0,
        "그림 56은 PDF처럼 p127 우측 Square band에 있어야 함: {:?}",
        p127_images[0]
    );
    assert!(
        (p127_images[0].1 - 83.2).abs() <= 1.0,
        "p127 그림 56은 next-page owner body top에서 시작해야 함: {:?}",
        p127_images[0]
    );

    let mut p127_image_boxes = Vec::new();
    let mut p127_pi1356_lines = Vec::new();
    image_boxes_for_control(&p127_tree.root, 1355, 0, &mut p127_image_boxes);
    paragraph_line_boxes(&p127_tree.root, 1356, &mut p127_pi1356_lines);
    let image = p127_image_boxes
        .into_iter()
        .next()
        .expect("p127 그림 56 bbox");
    let overlapping_vertical_lines: Vec<_> = p127_pi1356_lines
        .into_iter()
        .filter(|line| vertically_intersects(*line, image))
        .collect();
    assert!(
        !overlapping_vertical_lines.is_empty(),
        "p127 pi=1356에는 그림 56과 같은 세로 band의 Square 본문이 있어야 함"
    );
    assert!(
        overlapping_vertical_lines
            .iter()
            .all(|line| does_not_overlap_horizontally(*line, image)),
        "p127 pi=1356 본문은 그림 56과 물리적으로 교차하면 안 됨: image={image:?}, lines={overlapping_vertical_lines:?}"
    );
}

#[test]
fn native_hwp5_table_host_footnotes_follow_the_terminal_fragment_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc = HwpDocument::from_bytes(&bytes).expect("parse stage107 HWP evidence fixture");

    assert_eq!(
        doc.page_count(),
        215,
        "정책연구 기준 PDF와 215쪽을 유지해야 함"
    );
    let pages = [PAGE_87, PAGE_88, PAGE_90, PAGE_91, PAGE_94, PAGE_95].map(|page| {
        doc.build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render physical page {}: {e}", page + 1))
    });
    let mut notes = [
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];
    for (tree, text) in pages.iter().zip(notes.iter_mut()) {
        footnote_text(&tree.root, false, text);
    }

    assert!(
        notes[0].contains("138)") && notes[0].contains("부록 내용 표로 정리"),
        "p87은 표 26 terminal fragment와 형제 각주 138을 같이 소유해야 함: {}",
        notes[0]
    );
    assert_eq!(
        notes[0].matches("138)").count(),
        1,
        "p87은 각주 138을 정확히 한 번만 렌더해야 함: {}",
        notes[0]
    );
    assert!(
        !notes[1].contains("138)") && notes[1].contains("139)"),
        "p88은 각주 138을 이월·중복 소유하지 않고 기존 139를 유지해야 함: {}",
        notes[1]
    );
    assert!(
        !notes[2].contains("142)")
            && notes[2].contains("141)")
            && notes[3].contains("142)")
            && notes[3].contains("유럽 28개국과 노르웨이 분석"),
        "각주 142는 표 27의 p90 first fragment가 아니라 p91 terminal fragment owner여야 함: p90={}, p91={}",
        notes[2],
        notes[3]
    );
    assert_eq!(
        notes[3].matches("142)").count(),
        1,
        "p91은 각주 142를 정확히 한 번만 렌더해야 함: {}",
        notes[3]
    );
    for number in 143..=145 {
        assert!(
            notes[3].contains(&format!("{number})")),
            "p91은 기존 후속 각주 {number}를 유지해야 함: {}",
            notes[3]
        );
    }
    assert!(
        !notes[4].contains("147)")
            && notes[5].contains("147)")
            && notes[5].contains("eutoolbox_living_kidney_donation_en.pdf"),
        "각주 147은 표 28의 p94 first fragment가 아니라 p95 terminal fragment owner여야 함: p94={}, p95={}",
        notes[4],
        notes[5]
    );
    assert_eq!(
        notes[5].matches("147)").count(),
        1,
        "p95는 각주 147을 정확히 한 번만 렌더해야 함: {}",
        notes[5]
    );

    for (tree, page, table_para, following_body_para) in [
        (&pages[0], "p87", 937, 940),
        (&pages[3], "p91", 962, 972),
        (&pages[5], "p95", 1000, 1009),
    ] {
        let mut table_end = None;
        let mut body_end = None;
        let mut separator = None;
        let mut footnote_end = None;
        let mut footer_start = None;
        table_bottom(&tree.root, table_para, &mut table_end);
        paragraph_bottom(&tree.root, following_body_para, &mut body_end);
        footnote_separator_top(&tree.root, &mut separator);
        footnote_and_footer(&tree.root, &mut footnote_end, &mut footer_start);
        let separator = separator.unwrap_or_else(|| panic!("{page} footnote separator"));
        assert!(
            table_end.is_some_and(|bottom| bottom <= separator + 0.5),
            "{page} terminal table은 각주 separator를 침범하면 안 됨: table={table_end:?}, separator={separator:.1}"
        );
        assert!(
            body_end.is_some_and(|bottom| bottom <= separator + 0.5),
            "{page} 후속 본문은 각주 separator를 침범하면 안 됨: body={body_end:?}, separator={separator:.1}"
        );
        assert!(
            footnote_end.is_some_and(|bottom| {
                footer_start.is_some_and(|footer| bottom <= footer + 1.0)
            }),
            "{page} 각주 영역은 footer를 침범하면 안 됨: footnote={footnote_end:?}, footer={footer_start:?}"
        );
    }
}

#[test]
fn native_hwp5_table_host_footnote_capacity_fallback_preserves_note_once() {
    use rhwp::model::control::Control;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse stage107 capacity fixture");
    let mut document = doc.document().clone();
    let footnote = document.sections[0].paragraphs[937]
        .controls
        .iter_mut()
        .find_map(|control| match control {
            Control::Footnote(footnote) if footnote.number == 138 => Some(footnote),
            _ => None,
        })
        .expect("pi937 table host sibling footnote 138");
    let template = footnote
        .paragraphs
        .first()
        .cloned()
        .expect("footnote 138 body paragraph");
    // 원본 terminal page의 잔여보다 크지만 fresh page에는 충분히 들어가는 각주를
    // 만든다. 후보+fit을 하나의 bool로 합치면 이 경우 has_table no-op으로 흘러
    // 각주가 문서 전체에서 사라진다.
    footnote
        .paragraphs
        .extend((0..58).map(|_| template.clone()));
    doc.set_document(document);

    let p87 = doc
        .build_page_render_tree(PAGE_87)
        .expect("render enlarged-note terminal page");
    let p88 = doc
        .build_page_render_tree(PAGE_88)
        .expect("render enlarged-note fallback page");
    let mut p87_notes = String::new();
    let mut p88_notes = String::new();
    footnote_text(&p87.root, false, &mut p87_notes);
    footnote_text(&p88.root, false, &mut p88_notes);

    assert!(
        !p87_notes.contains("138)"),
        "확대 각주 138은 공간이 모자란 terminal p87에 겹쳐 넣으면 안 됨: {p87_notes}"
    );
    assert_eq!(
        p88_notes.matches("138)").count(),
        1,
        "공간 부족 fallback은 새 physical page에 각주 138을 정확히 한 번 보존해야 함: {p88_notes}"
    );
    assert!(
        p88_notes.contains("부록 내용 표로 정리"),
        "fallback page는 각주 138 본문을 보존해야 함: {p88_notes}"
    );

    let mut footnote_end = None;
    let mut footer_start = None;
    footnote_and_footer(&p88.root, &mut footnote_end, &mut footer_start);
    assert!(
        footnote_end.is_some_and(|bottom| {
            footer_start.is_some_and(|footer| bottom <= footer + 1.0)
        }),
        "fallback 각주 영역은 footer를 침범하면 안 됨: footnote={footnote_end:?}, footer={footer_start:?}"
    );
}

#[test]
fn native_hwp5_table_host_footnote_survives_after_terminal_table_page_is_flushed() {
    use rhwp::model::control::Control;
    use rhwp::model::footnote::Footnote;

    fn find_nested_footnote(controls: &[Control], number: u16) -> Option<Box<Footnote>> {
        for control in controls {
            match control {
                Control::Footnote(footnote) if footnote.number == number => {
                    return Some(footnote.clone());
                }
                Control::Table(table) => {
                    for cell in &table.cells {
                        for paragraph in &cell.paragraphs {
                            if let Some(footnote) =
                                find_nested_footnote(&paragraph.controls, number)
                            {
                                return Some(footnote);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut doc = HwpDocument::from_bytes(&bytes).expect("parse stage107 terminal-flush fixture");
    let mut document = doc.document().clone();

    // 실제 fixture의 note 234는 첫 두 stored line이 vpos=0인 table-cell 각주다.
    // terminal fragment에서 prefix를 등록한 뒤 suffix용 fresh page를 만들므로,
    // 이어지는 표 host 형제 각주를 처리할 때 terminal 표는 current_items에 없다.
    let mut split_cell_footnote = document.sections[0]
        .paragraphs
        .iter()
        .find_map(|paragraph| find_nested_footnote(&paragraph.controls, 234))
        .expect("table-cell footnote 234 repeated-zero template");
    split_cell_footnote.number = 60_000;

    let table = document.sections[0].paragraphs[937]
        .controls
        .iter_mut()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("pi937 table 26");
    let terminal_row = table
        .cells
        .iter()
        .map(|cell| cell.row)
        .max()
        .expect("table 26 terminal row");
    let terminal_cell = table
        .cells
        .iter_mut()
        .find(|cell| cell.row == terminal_row && cell.row_span == 1)
        .expect("table 26 terminal row cell");
    terminal_cell
        .paragraphs
        .last_mut()
        .expect("table 26 terminal cell paragraph")
        .controls
        .push(Control::Footnote(split_cell_footnote));
    doc.set_document(document);

    let pages = [PAGE_87, PAGE_88, PAGE_88 + 1].map(|page| {
        doc.build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("render terminal-flush physical page {}: {e}", page + 1))
    });
    let mut notes = [String::new(), String::new(), String::new()];
    for (tree, text) in pages.iter().zip(notes.iter_mut()) {
        footnote_text(&tree.root, false, text);
    }

    assert!(
        notes[0].contains("60000)") && notes[0].contains("using moderately and"),
        "p87은 합성 table-cell 각주의 prefix를 소유해 terminal page flush를 일으켜야 함: {}",
        notes[0]
    );
    assert!(
        !notes[1].contains("60000)") && notes[1].contains("severely steatotic donor livers"),
        "p88은 번호를 반복하지 않은 합성 table-cell 각주 tail을 먼저 소유해야 함: {}",
        notes[1]
    );
    assert_eq!(
        notes.iter().map(|text| text.matches("138)").count()).sum::<usize>(),
        1,
        "terminal-not-current 경로에서도 표 host 형제 각주 138을 정확히 한 번 보존해야 함: {notes:?}"
    );
    assert!(
        notes[1].contains("138)") && notes[1].contains("부록 내용 표로 정리"),
        "표 host 형제 각주 138은 terminal 뒤 current footnote page의 기존 tail 다음에 등록돼야 함: {}",
        notes[1]
    );

    let mut footnote_end = None;
    let mut footer_start = None;
    footnote_and_footer(&pages[1].root, &mut footnote_end, &mut footer_start);
    assert!(
        footnote_end.is_some_and(|bottom| {
            footer_start.is_some_and(|footer| bottom <= footer + 1.0)
        }),
        "terminal-not-current fallback 각주 영역은 footer를 침범하면 안 됨: footnote={footnote_end:?}, footer={footer_start:?}"
    );
}
