//! Issue #1921 — 59043 규제영향분석서 페이지네이션 드리프트 핀.
//!
//! `samples/issue1921/59043_regulatory_analysis.hwp` — 부동(자리차지) 표·rowspan
//! 블록이 밀집한 규제영향분석서. PR #2092(RowBreak 블록컷 sliver 흡수)로
//! 48쪽 → 42쪽 (수정 전 pi=160 3×3 rowspan 블록에서 컷 진동 `+46,+1` 교대).
//!
//! 권위 정답지는 한글 2022 편집기 37쪽
//! (`pdf/issue1921/59043_regulatory_analysis-2022.pdf`, 편집기 PageCount=37 정합).
//! 현재 페이지 수는 권위 정답지와 같은 37쪽이다. 과거 중간 도달값 39쪽을 고정하던
//! provisional pin은 PDF 직접 대조 뒤 정답지 pin으로 승격했다.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue1921/59043_regulatory_analysis.hwp";
const PAGE_8: u32 = 7;
const PAGE_11: u32 = 10;
const PAGE_12: u32 = 11;
const PAGE_35: u32 = 34;
const PAGE_36: u32 = 35;
const TOLERANCE_PX: f64 = 0.75;

fn page_count_of(rel: &str) -> u32 {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.page_count()
}

fn load_document() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"))
}

fn find_table<'a>(
    node: &'a RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<&'a RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para_index) && table.control_index == Some(control_index) {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para_index, control_index))
}

fn find_cell(node: &RenderNode, row: u16, col: u16) -> Option<&RenderNode> {
    node.children.iter().find(|child| {
        matches!(
            child.node_type,
            RenderNodeType::TableCell(ref cell) if cell.row == row && cell.col == col
        )
    })
}

fn collect_images<'a>(node: &'a RenderNode, images: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        images.push(node);
    }
    for child in &node.children {
        collect_images(child, images);
    }
}

fn collect_visible_text(node: &RenderNode, clip_top: f64, clip_bottom: f64, text: &mut String) {
    let (next_top, next_bottom) = if matches!(&node.node_type, RenderNodeType::TableCell(_)) {
        (
            clip_top.max(node.bbox.y),
            clip_bottom.min(node.bbox.y + node.bbox.height),
        )
    } else {
        (clip_top, clip_bottom)
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let bottom = node.bbox.y + node.bbox.height;
        if node.bbox.y >= next_top - TOLERANCE_PX && bottom <= next_bottom + TOLERANCE_PX {
            text.push_str(&run.text);
        }
    }
    for child in &node.children {
        collect_visible_text(child, next_top, next_bottom, text);
    }
}

fn compact_text(node: &RenderNode, page_bottom: f64) -> String {
    let mut text = String::new();
    collect_visible_text(node, 0.0, page_bottom, &mut text);
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn regulatory_59043_page_count_pin() {
    let pages = page_count_of("samples/issue1921/59043_regulatory_analysis.hwp");
    assert_eq!(
        pages, 37,
        "issue1921 59043은 한글 2022 정답지와 같은 37쪽이어야 함. 실측 {}p — \
         페이지 수가 같더라도 p8/p11의 물리 cell containment를 별도 확인할 것.",
        pages
    );
}

/// 한컴 2022 PDF p8의 6행 왼쪽 셀에는 두 사진이 모두 들어간다. 특히 Square-wrap
/// 세로 사진은 cell bottom에서 다시 배치되면 다음 본문을 침범한다. 페이지 수만으로는
/// 이 소실/겹침을 감지하지 못하므로, 이미지의 물리 cell containment를 별도 고정한다.
#[test]
fn regulatory_59043_page8_square_picture_stays_in_its_table_cell() {
    let doc = load_document();
    let tree = doc
        .build_page_render_tree(PAGE_8)
        .unwrap_or_else(|e| panic!("render {SAMPLE} page 8: {e}"));
    let table = find_table(&tree.root, 73, 0).expect("p8 사진 표(pi=73, ci=0)");
    let cell = find_cell(table, 6, 0).expect("p8 사진 표의 6행 왼쪽 셀");
    let cell_bottom = cell.bbox.y + cell.bbox.height;
    let mut images = Vec::new();
    collect_images(cell, &mut images);
    assert_eq!(
        images.len(),
        2,
        "p8 6행 왼쪽 셀의 사진 두 개가 모두 렌더되어야 함"
    );
    for image in images {
        let bottom = image.bbox.y + image.bbox.height;
        assert!(
            bottom <= cell_bottom + 0.75,
            "p8 6행 왼쪽 셀의 사진이 cell 밖으로 돌출: image bottom={bottom:.1}, \
             cell bottom={cell_bottom:.1}"
        );
    }
}

/// 한컴 2022 PDF p11의 pi=98 row 0에는 동영상 1개가 있고, row 2에는 제품 사진
/// 3개와 제품 상자 1개가 해당 cell 안에 배치된다.
/// 저장 vpos가 있는 RowBreak cell에서 그림 높이를 다시 일반 flow로 더하면 다음 fragment가
/// 소유해야 할 그림이 앞 page의 cell 밖으로 올라간다. 페이지 수 핀만으로는 이 소유권
/// 역전을 감지하지 못하므로, p11의 물리 cell containment를 고정한다.
#[test]
fn regulatory_59043_page11_square_pictures_stay_in_row2_fragment() {
    let doc = load_document();
    let tree = doc
        .build_page_render_tree(PAGE_11)
        .unwrap_or_else(|e| panic!("render {SAMPLE} page 11: {e}"));
    let table = find_table(&tree.root, 98, 0).expect("p11 SNS 사례 표(pi=98, ci=0)");
    let cell = find_cell(table, 2, 0).expect("p11 SNS 사례 표의 2행");
    let cell_top = cell.bbox.y;
    let cell_bottom = cell.bbox.y + cell.bbox.height;
    let mut images = Vec::new();
    collect_images(cell, &mut images);
    assert_eq!(
        images.len(),
        4,
        "p11 row 2의 PDF 소유 이미지 네 개가 같은 fragment에서 렌더되어야 함"
    );
    for image in images {
        let bottom = image.bbox.y + image.bbox.height;
        assert!(
            image.bbox.y >= cell_top - 0.75 && bottom <= cell_bottom + 0.75,
            "p11 row 2 Square 사진의 owner가 cell fragment와 다름: image={:?}, cell=({cell_top:.1}..{cell_bottom:.1})",
            image.bbox
        );
    }
}

/// 한컴 2022 PDF p12의 pi=98 row 4에는 empty paragraph의 TAC 사진 두 개가 각자의
/// 저장 LINE_SEG slot에 위·아래로 들어간다. partial RowBreak fallback이 첫 slot의 x만
/// 누적하면 둘째 사진은 셀 우측 밖으로 나가므로, 셀 containment와 vertical order를 함께 고정한다.
#[test]
fn regulatory_59043_page12_empty_tac_pictures_follow_distinct_line_slots() {
    let doc = load_document();
    let tree = doc
        .build_page_render_tree(PAGE_12)
        .unwrap_or_else(|e| panic!("render {SAMPLE} page 12: {e}"));
    let table = find_table(&tree.root, 98, 0).expect("p12 SNS 사례 표(pi=98, ci=0)");
    let cell = find_cell(table, 4, 0).expect("p12 SNS 사례 표의 4행");
    let cell_left = cell.bbox.x;
    let cell_right = cell.bbox.x + cell.bbox.width;
    let cell_top = cell.bbox.y;
    let cell_bottom = cell.bbox.y + cell.bbox.height;
    let mut images = Vec::new();
    collect_images(cell, &mut images);
    assert_eq!(
        images.len(),
        2,
        "p12 row 4의 empty TAC 사진 두 개가 모두 같은 cell fragment에 있어야 함"
    );
    images.sort_by(|left, right| left.bbox.y.total_cmp(&right.bbox.y));
    for image in &images {
        let right = image.bbox.x + image.bbox.width;
        let bottom = image.bbox.y + image.bbox.height;
        assert!(
            image.bbox.x >= cell_left - 0.75
                && right <= cell_right + 0.75
                && image.bbox.y >= cell_top - 0.75
                && bottom <= cell_bottom + 0.75,
            "p12 row 4 TAC 사진이 cell 밖으로 돌출: image={:?}, cell=({cell_left:.1}..{cell_right:.1}, {cell_top:.1}..{cell_bottom:.1})",
            image.bbox
        );
    }
    assert!(
        images[0].bbox.y + images[0].bbox.height <= images[1].bbox.y + 0.75,
        "p12 row 4 TAC 사진은 각각의 LINE_SEG slot에 위·아래로 배치돼야 함: first={:?}, second={:?}",
        images[0].bbox,
        images[1].bbox
    );
}

/// 한컴 2022 PDF p35/p36은 마지막 6×2 RowBreak 행 안의 1×1 자식 표를
/// p35의 도입부와 p36의 근거·소표·계산으로 나눈다. 자식의 저장 fragment 합만
/// 보고 한 쪽짜리 atom으로 소비하면 p35 화면 밖에 전부 생성되고 p36은 빈 셀만 남는다.
#[test]
fn regulatory_59043_page36_keeps_nested_source_and_following_headings() {
    let doc = load_document();
    let p35 = doc
        .build_page_render_tree(PAGE_35)
        .unwrap_or_else(|e| panic!("render {SAMPLE} page 35: {e}"));
    let p36 = doc
        .build_page_render_tree(PAGE_36)
        .unwrap_or_else(|e| panic!("render {SAMPLE} page 36: {e}"));
    let p35_text = compact_text(&p35.root, p35.root.bbox.y + p35.root.bbox.height);
    let p36_text = compact_text(&p36.root, p36.root.bbox.y + p36.root.bbox.height);

    assert!(
        p35_text.contains("담배판촉행위금지시흡연율약1%감소효과"),
        "p35는 자식 표의 PDF 도입부를 소유해야 한다"
    );
    assert!(
        !p35_text.contains("흡연율감소효과추정근거"),
        "p36 source owner가 p35 Cell clip 안에 조기 노출됐다"
    );
    assert!(
        p36_text.contains("흡연율감소효과추정근거")
            && p36_text.contains("3,664백만갑")
            && p36_text.contains("228억원"),
        "p36은 PDF의 근거 본문·3×3 소표·계산 결과를 모두 보여야 한다"
    );

    let outer = find_table(&p36.root, 260, 0).expect("p36 6×2 RowBreak 표(pi=260, ci=0)");
    let source_cell = find_cell(outer, 5, 1).expect("p36 마지막 행의 오른쪽 source cell");
    let source_text = compact_text(source_cell, source_cell.bbox.y + source_cell.bbox.height);
    assert!(
        source_text.contains("흡연율감소효과추정근거"),
        "p36 source가 마지막 행 오른쪽 cell의 가시 clip 안에 있어야 한다"
    );

    let general_public = find_table(&p36.root, 270, 0).expect("p36 일반국민 제목(pi=270)");
    let benefit = find_table(&p36.root, 271, 0).expect("p36 편익 제목(pi=271)");
    assert!(
        general_public.bbox.y + general_public.bbox.height <= benefit.bbox.y + TOLERANCE_PX,
        "p36 후속 제목이 겹쳤다: 일반국민={:?}, 편익={:?}",
        general_public.bbox,
        benefit.bbox
    );
}
