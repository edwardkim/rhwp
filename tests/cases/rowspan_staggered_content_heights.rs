//! 검사 유형: 관계형·고정형(독립 PDF 쪽수); 절대 페이지 좌표 핀 없음.
//! 주석 갱신: 2026-11-05T02:21:40+09:00; Git 직전 파일 변경: 19aed211d178996cf83aae89388ed91d6ec93bcf 2026-10-29T01:16:35+09:00.
//! 선행 측정 source: 5c540c6c3e. 96 DPI·Print·PDFium exact physical DPI v2·2px raw 실루엣.
//! 측정 실행 UTC: Native 2026-11-04T16:51:22.176956+00:00 ~ 2026-11-04T16:52:21.959195+00:00; WASM 2026-11-04T17:01:38.785488+00:00 ~ 2026-11-04T17:02:01.289862+00:00.
//! rowspan_staggered_heights: 전7쪽 Native 최저 99.32% / fresh WASM 최저 99.98%; 누락 없음.
//! 증거: mydocs/pr/assets/semanticist21-20261105/validation/candidate/{native,wasm}/rowspan_staggered_heights.tsv.
//! 대상 2쪽 N/W 99.50/99.99%, 이어받는 3쪽 99.99/100.00%; 같은 영역의 독립 PDF를 직접 판독했다.
//!
//! 엇갈린 병합 칸들의 행 높이를 칸 내용으로 푼다.
//!
//! 한컴 한글 12.30(Mac)에서 표 일부 블록에 "셀 높이를 같게"를 적용해 저장한 문서다.
//! 한컴은 14행을 27행으로 다시 나눈다. 선택한 오른쪽 칸은 모두 1500HU(20px)이고,
//! 고르지 않은 왼쪽 칸은 두 행씩 걸치며 원래 선언 282HU를 그대로 둔다. 왼쪽 칸의
//! 실제 높이는 글줄과 안 여백(18.4px)이 정하고, 두 열은 한 행씩 엇갈려 있다.
//!
//! 한컴 인쇄 PDF(`pdf/rowspan_staggered_heights/`)에서 왼쪽 칸은 18.4px,
//! 오른쪽 칸은 20px 간격이고 두 열의 표 끝이 같다. 선언 높이만으로 행을 풀면
//! 사슬이 282HU에서 끊겨 남은 행을 똑같이 나누므로 왼쪽 칸이 20px로 늘고
//! 표가 약 20px 길어졌다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "tests/fixtures/rowspan_staggered_heights/equal-height-partial-block.hwp";
const PAGE: u32 = 1;
const TABLE_PARA: usize = 25;

#[derive(Debug)]
struct Cell {
    row: u16,
    col: u16,
    row_span: u16,
    bbox: BoundingBox,
}

fn table_cells(node: &RenderNode, out: &mut Vec<Cell>) -> bool {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(TABLE_PARA) && table.cell_context.is_none() {
            for child in &node.children {
                if let RenderNodeType::TableCell(cell) = &child.node_type {
                    out.push(Cell {
                        row: cell.row,
                        col: cell.col,
                        row_span: cell.row_span,
                        bbox: child.bbox,
                    });
                }
            }
            return true;
        }
    }
    node.children.iter().any(|child| table_cells(child, out))
}

fn cell_at(cells: &[Cell], row: u16, col: u16) -> &Cell {
    cells
        .iter()
        .find(|cell| cell.row == row && cell.col == col)
        .unwrap_or_else(|| panic!("r{row} c{col} 칸"))
}

#[test]
fn staggered_span_cells_keep_their_own_heights() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 2 render tree");
    let mut cells = Vec::new();
    assert!(table_cells(&page.root, &mut cells), "27행 표");

    // 한 행짜리 칸이 각 열의 기준 높이다: 왼쪽 끝 행은 내용 높이, 오른쪽 r12는 1500HU.
    let left = cell_at(&cells, 26, 0).bbox.height;
    let right = cell_at(&cells, 12, 3).bbox.height;
    assert!(
        (right - 1500.0 / 75.0).abs() < 0.05,
        "선택한 칸 높이 1500HU: {right:.2}"
    );

    for cell in cells
        .iter()
        .filter(|cell| cell.row_span == 2 && cell.row >= 2)
    {
        let expected = if cell.col <= 2 { left } else { right };
        assert!(
            (cell.bbox.height - expected).abs() < 0.05,
            "r{} c{} 칸 높이 {:.2}, 같은 열의 한 행 칸 {:.2}",
            cell.row,
            cell.col,
            cell.bbox.height,
            expected
        );
    }

    // 두 열의 사슬이 같은 표 끝에서 닫히고, 오른쪽 열은 머리 칸 두 개와
    // 1500HU 칸 열둘이 그대로 쌓인다.
    let top = cell_at(&cells, 0, 3).bbox.y;
    let left_bottom = cell_at(&cells, 26, 0).bbox.y + left;
    let right_end = cell_at(&cells, 25, 3);
    let right_bottom = right_end.bbox.y + right_end.bbox.height;
    assert!(
        (left_bottom - right_bottom).abs() < 0.05,
        "열 끝: 왼쪽 {left_bottom:.2}, 오른쪽 {right_bottom:.2}"
    );
    assert!(
        (right_bottom - top - 14.0 * right).abs() < 0.05,
        "표 높이 {:.2}, 1500HU 칸 14개 {:.2}",
        right_bottom - top,
        14.0 * right
    );
}

fn outer_table(node: &RenderNode, paragraph: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table)
        if table.para_index == Some(paragraph) && table.cell_context.is_none())
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| outer_table(child, paragraph))
}

fn paragraph_lines<'a>(node: &'a RenderNode, paragraph: usize, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextLine(line)
        if line.para_index == Some(paragraph))
    {
        out.push(node);
        return;
    }
    for child in &node.children {
        paragraph_lines(child, paragraph, out);
    }
}

#[test]
fn staggered_table_preserves_following_content_and_paragraph_continuation() {
    let bytes = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 7, "독립 한컴 Print의 전체 쪽수");
    let second = core.build_page_render_tree(1).unwrap();
    let third = core.build_page_render_tree(2).unwrap();
    let target = outer_table(&second.root, TABLE_PARA).expect("엇갈린 병합 표");
    let following = outer_table(&second.root, 27).expect("분배 비율 안내 표");
    let mut body = Vec::new();
    paragraph_lines(&second.root, 28, &mut body);
    let first_body = body.first().expect("후속 이용허락 본문");
    assert!(target.bbox.y + target.bbox.height <= following.bbox.y);
    assert!(following.bbox.y + following.bbox.height <= first_body.bbox.y);

    let mut cells = Vec::new();
    table_cells(&second.root, &mut cells);
    for cell in cells {
        assert!(cell.bbox.y >= target.bbox.y - 1e-6);
        assert!(cell.bbox.y + cell.bbox.height <= target.bbox.y + target.bbox.height + 1e-6);
    }
    let mut before = Vec::new();
    let mut after = Vec::new();
    paragraph_lines(&second.root, 29, &mut before);
    paragraph_lines(&third.root, 29, &mut after);
    assert!(
        !before.is_empty() && !after.is_empty(),
        "본문 29는 두 쪽에 이어진다"
    );
    let normalize = |value: &str| -> String {
        value
            .chars()
            .filter(|c| !c.is_whitespace() && !c.is_control() && *c != '\u{fffc}')
            .collect()
    };
    let mut actual = String::new();
    for line in before.iter().chain(&after) {
        for child in &line.children {
            if let RenderNodeType::TextRun(run) = &child.node_type {
                actual.push_str(run.display_or_text());
            }
        }
    }
    assert_eq!(
        normalize(&actual),
        normalize(&core.document().sections[0].paragraphs[29].text),
        "분할 본문의 순서·내용을 누락이나 중복 없이 보존한다"
    );
    let mut next = Vec::new();
    paragraph_lines(&third.root, 30, &mut next);
    assert!(after.last().unwrap().bbox.y < next.first().expect("후속 ④ 문단").bbox.y);
}
