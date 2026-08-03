//! Issue #3738 Stage 9: 작은 RowBreak 표의 cell-footnote 전체 선예약이
//! 첫 fragment를 통째로 다음 쪽으로 미는 회귀를 실제 HWP로 고정한다.
//!
//! 한컴오피스 2020 기준 PDF p66에는 표 23의 0–4행(Organ Donation까지)과
//! 각주 76·77이 있고, p67은 Stephanie 행부터 이어진다. 표 전체 각주를
//! 첫 행 전부터 예약하면 p66 표가 전부 이월되어 이후 문단까지 한 쪽씩 밀린다.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
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
const PAGE_58: u32 = 57;
const PAGE_59: u32 = 58;
const PAGE_76: u32 = 75;
const PAGE_77: u32 = 76;
const PAGE_78: u32 = 77;
const PAGE_79: u32 = 78;
const PAGE_80: u32 = 79;
const PAGE_90: u32 = 89;
const PAGE_91: u32 = 90;
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
    assert_eq!(
        doc.page_count(),
        219,
        "p90 표 27 row owner 보정이 전체 native page count를 바꾸면 안 됨"
    );

    let p90_tree = doc
        .build_page_render_tree(PAGE_90)
        .expect("render physical page 90");
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
    assert_eq!(
        doc.page_count(),
        219,
        "p26 footnote owner must not change total page count"
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
    assert_eq!(
        doc.page_count(),
        219,
        "marker-page footnote routing must not introduce a new physical page"
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
