//! 관계형·고정형(쪽수): 셀의 유일한 문단에 담긴 표가 그 셀 안에서 셀 내용 위부터 선언 높이로 배치되고 다음 행과 겹치지 않는지, 전체 쪽수가 독립 PDF 와 같은지 검사한다.
//! 주석 갱신: 2026-11-06T14:27:56+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-06 KST, source = upstream/devel 5716a226b5 + #8677 수정(이 PR 의 src 변경과 동일).
//! 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw, 독립 한컴 2020 Print
//! `pdf/issue8677/1411000-201000007_final_report-2020.pdf`(Hwp 2022 생성, 111쪽).
//! 입력 `samples/issue8677/1411000-201000007_final_report.hwp`(10k 공개 연구용역 최종보고서), 전체 111쪽=111쪽.
//! 수정 전 Native: 88쪽 37.72·89쪽 32.91(전쪽 최저 32.91) / fresh WASM: 88쪽 38.28·89쪽 32.77.
//! 수정 후 Native: 88쪽 99.84·89쪽 99.90, 전쪽 최저 86.07(28쪽, 수정 전과 동일)
//! / fresh WASM: 88쪽 99.83·89쪽 99.87, 전쪽 최저 86.99(28쪽, 27쪽 89.41 — 둘 다 수정 전과 동일, 표 셀 각주 #8685).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8677/{base,head}/{native,wasm}/`, `png/`.
//! 확대 판독: 89쪽 16×16 표가 5행(PDF 356~715px) 윗변에서 시작해 행을 채우고, 6·7행 9×10 표와 행 경계가 PDF 와 맞는다.
//! 같은 assertion 이 수정 전 FAIL(1행 3열 1×8 표 drawn 19.5 / declared 27.1) / 수정 후 PASS.
//! 독립 절대 px 좌표를 고정하지 않는다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8677/1411000-201000007_final_report.hwp";
/// 독립 한/글 2022 Print 의 전체 쪽수.
const PDF_PAGES: u32 = 111;
const SECTION: usize = 6;
/// 「월별 전기/열 수급 실적」 서식 8×3 표를 가진 문단.
const HOST_PARA: usize = 274;
const DPI: f64 = 96.0;

fn hu_to_px(hu: f64) -> f64 {
    hu * DPI / 7200.0
}

fn open() -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력")).expect("기준 문서 열기")
}

fn table_of(controls: &[Control]) -> &Table {
    match controls {
        [Control::Table(table)] => table,
        other => panic!("표 하나만 담은 문단이어야 한다: {} 컨트롤", other.len()),
    }
}

/// 바깥 표에서 (행, 열) 셀이 유일한 문단에 담은 안쪽 표.
fn sole_para_nested(outer: &Table, row: u16, col: u16) -> &Table {
    let cell = outer
        .cells
        .iter()
        .find(|cell| cell.row == row && cell.col == col)
        .expect("셀");
    assert_eq!(cell.paragraphs.len(), 1, "셀의 유일한 문단");
    table_of(&cell.paragraphs[0].controls)
}

fn find_outer(node: &RenderNode) -> Option<&RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.section_index == Some(SECTION)
            && table.para_index == Some(HOST_PARA)
            && table.cell_context.is_none()
        {
            return Some(node);
        }
    }
    node.children.iter().find_map(find_outer)
}

fn cell_node(outer: &RenderNode, row: u16, col: u16) -> &RenderNode {
    fn walk<'a>(node: &'a RenderNode, row: u16, col: u16) -> Option<&'a RenderNode> {
        if let RenderNodeType::TableCell(cell) = &node.node_type {
            return (cell.row == row && cell.col == col).then_some(node);
        }
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            return None;
        }
        node.children.iter().find_map(|child| walk(child, row, col))
    }
    outer
        .children
        .iter()
        .find_map(|child| walk(child, row, col))
        .expect("바깥 표 셀 노드")
}

/// 셀 노드 아래 첫 안쪽 표 노드.
fn nested_in(cell: &RenderNode) -> (u16, u16, BoundingBox) {
    fn walk(node: &RenderNode) -> Option<(u16, u16, BoundingBox)> {
        if let RenderNodeType::Table(table) = &node.node_type {
            return Some((table.row_count, table.col_count, node.bbox));
        }
        node.children.iter().find_map(walk)
    }
    cell.children.iter().find_map(walk).expect("셀 안 표")
}

fn text_of(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        text_of(child, out);
    }
}

#[test]
fn sole_cell_paragraph_table_fills_its_cell_from_the_top() {
    let core = open();
    assert_eq!(core.page_count(), PDF_PAGES, "독립 Print 와 같은 쪽수");

    let section = &core.document().sections[SECTION];
    let outer_model = table_of(&section.paragraphs[HOST_PARA].controls);

    let (page, tree) = (0..core.page_count())
        .map(|page| (page, core.build_page_render_tree(page).expect("쪽 배치")))
        .find(|(_, tree)| find_outer(&tree.root).is_some())
        .expect("서식 표가 배치된 쪽");
    let outer = find_outer(&tree.root).unwrap();

    // 1행 3열(「년 월」 1×8 표)과 5행(16×16 표): 셀의 유일한 문단이 표 하나만 담는다.
    for (row, col) in [(0u16, 2u16), (4, 0)] {
        let model = sole_para_nested(outer_model, row, col);
        let cell = cell_node(outer, row, col);
        let (rows, cols, nested) = nested_in(cell);
        assert_eq!(
            (rows, cols),
            (model.row_count, model.col_count),
            "{page}쪽 ({row},{col}) 셀의 안쪽 표"
        );
        let declared = hu_to_px(f64::from(model.common.height));
        assert!(
            (nested.height - declared).abs() <= 1.0,
            "({row},{col}) 안쪽 표는 선언 높이로 그린다(눌리지 않는다): drawn={:.1} declared={declared:.1}",
            nested.height
        );
        // 셀 내용 위 = 셀 윗변 + 셀 안 위 여백, 그 아래 표 바깥 위 여백 + 세로 오프셋.
        let cell_model = outer_model
            .cells
            .iter()
            .find(|c| c.row == row && c.col == col)
            .unwrap();
        let content_top = cell.bbox.y + hu_to_px(f64::from(cell_model.padding.top));
        let expected_top = content_top
            + hu_to_px(f64::from(model.outer_margin_top))
            + hu_to_px(f64::from(model.common.vertical_offset as i32));
        assert!(
            (nested.y - expected_top).abs() <= 1.0,
            "({row},{col}) 셀의 유일한 문단 표는 셀 내용 위에서 시작한다: nested.y={:.1} expected={expected_top:.1} cell={:?}",
            nested.y,
            cell.bbox
        );
        assert!(
            nested.y + nested.height <= cell.bbox.y + cell.bbox.height + 0.5,
            "({row},{col}) 안쪽 표는 자기 셀 안에 끝난다: nested={nested:?} cell={:?}",
            cell.bbox
        );
    }

    // 16×16 표의 머리 칸 글자가 보존되고, 다음 행(6행)은 5행 아래에서 시작한다.
    let row4 = cell_node(outer, 4, 0);
    let mut text = String::new();
    text_of(row4, &mut text);
    for label in ["일련번호", "발전기"] {
        assert!(text.contains(label), "16×16 표 머리 칸 「{label}」 보존");
    }
    let row5 = cell_node(outer, 5, 0);
    assert!(
        row4.bbox.y + row4.bbox.height <= row5.bbox.y + 0.5,
        "5행과 6행은 겹치지 않는다: row4={:?} row5={:?}",
        row4.bbox,
        row5.bbox
    );
}
