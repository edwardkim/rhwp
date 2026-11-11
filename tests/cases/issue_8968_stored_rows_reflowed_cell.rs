//! [#8968] 관계형·고정형 회귀 — 칸 하나가 저장 줄 없이 재조판돼도 측정 행이 칸 선언과 같으면
//! 표를 개체 크기(`hp:sz`)로 늘리지 않는다.
//!
//! 주석 갱신: 2026-11-12 04:10 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8968/stored_rows_reflowed_cell.hwpx` — 서울특별시 영등포소방서 공개 결재문서
//!   「신길 구급차(영등포 6-7호) 동태변경 승인요청」(한컴오피스 2020 저장, 1쪽). 문단 4 의
//!   2×4 표(자리차지·문단 기준·RowBreak·`noAdjust`)는 개체 높이 7636HU, 칸 선언 행 합
//!   2414 + 4030 = 6444HU 이고, 1행 한 칸은 HWPX `linesegarray` 가 없어 rhwp 가 줄을 계산한다.
//! - 독립 기준: `pdf/issue8968/stored_rows_reflowed_cell-2020.pdf`(한/글 2022 12.0.0.4547 Print, 1쪽).
//!   한/글은 행을 칸 선언대로(32.2px + 53.7px) 그리고, 「끝.」을 그 표 바로 아래에 둔다.
//! - 수정 전 rhwp(devel `4c876654d8`)는 저장 줄 없는 칸 때문에 `stored_rows_own_body` 가 거짓이 되어
//!   표를 개체 높이 101.8px 로 늘렸다(둘째 행 69.6px).
//!
//! 측정: 2026-11-12, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣). 1쪽 Native 측정→게이트 기록.
//!
//! 검사: 원본 계약(개체 높이 > 칸 선언 행 합, 저장 줄 없는 칸 존재), 쪽수 1 = 한/글(고정형),
//! 칸마다 그린 높이 = 칸 선언 높이, 표 상자 높이 = 칸 선언 행 합, 「끝.」 줄이 표 아래끝 뒤
//! 한 줄 안에서 시작(관계형).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::LineSeg;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8968/stored_rows_reflowed_cell.hwpx";
/// 표를 품은 「끝.」 문단.
const HOST: usize = 4;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn find_table<'a>(node: &'a RenderNode, para: usize) -> Option<&'a RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) && table.cell_context.is_none() {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para))
}

fn body_lines(node: &RenderNode, out: &mut Vec<(Option<usize>, BoundingBox)>) {
    match &node.node_type {
        RenderNodeType::Table(_) => return,
        RenderNodeType::TextLine(line) => out.push((line.para_index, node.bbox)),
        _ => {}
    }
    for child in &node.children {
        body_lines(child, out);
    }
}

#[test]
fn measured_rows_matching_declared_cells_keep_table_out_of_object_cache() {
    let core = load();
    assert_eq!(core.page_count(), 1, "한/글 Print 와 같은 1쪽");

    let host = &core.document().sections[0].paragraphs[HOST];
    let table = host
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("문단 4 의 표");
    assert!(!table.common.treat_as_char, "자리차지(비-TAC) 표");

    // 원본 계약: 행마다 칸 선언 높이가 하나로 같고, 개체 높이는 그 합보다 크다.
    let mut declared_rows = vec![0u32; table.row_count as usize];
    for cell in &table.cells {
        assert_eq!(cell.row_span, 1, "병합 행 없음");
        let row = &mut declared_rows[cell.row as usize];
        assert!(
            *row == 0 || *row == cell.height,
            "행 {} 칸 선언 높이가 같음",
            cell.row
        );
        *row = cell.height;
    }
    let declared_sum: u32 = declared_rows.iter().sum();
    assert!(
        table.common.height > declared_sum,
        "개체 높이 {} > 칸 선언 행 합 {}",
        table.common.height,
        declared_sum
    );
    // 원본 계약: 저장 줄 없이 rhwp 가 줄을 계산한 칸이 있다.
    assert!(
        table
            .cells
            .iter()
            .any(|cell| cell.paragraphs.iter().any(|para| para
                .line_segs
                .iter()
                .all(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0))),
        "저장 줄 없는 칸 문단"
    );

    let page = core.build_page_render_tree(0).expect("1쪽");
    let node = find_table(&page.root, HOST).expect("표 상자");
    let mut checked = 0;
    for child in &node.children {
        let RenderNodeType::TableCell(cell_node) = &child.node_type else {
            continue;
        };
        let model = &table.cells[cell_node.model_cell_index.expect("칸 인덱스") as usize];
        let declared = hu_to_px(model.height as i32);
        assert!(
            (child.bbox.height - declared).abs() <= 0.5,
            "칸 r{} c{} 그린 높이 {:.1} = 칸 선언 {:.1}",
            cell_node.row,
            cell_node.col,
            child.bbox.height,
            declared
        );
        checked += 1;
    }
    assert_eq!(checked, table.cells.len(), "표 칸 전부 검사");

    let rows_px = hu_to_px(declared_sum as i32);
    assert!(
        (node.bbox.height - rows_px).abs() <= 1.0,
        "표 상자 높이 {:.1} = 칸 선언 행 합 {:.1}(개체 {:.1} 아님)",
        node.bbox.height,
        rows_px,
        hu_to_px(table.common.height as i32)
    );

    // 「끝.」 줄은 표 아래끝 뒤 한 줄 안에서 시작한다.
    let mut lines = Vec::new();
    body_lines(&page.root, &mut lines);
    let (_, end_line) = lines
        .iter()
        .find(|(para, _)| *para == Some(HOST))
        .expect("「끝.」 줄");
    let table_bottom = node.bbox.y + node.bbox.height;
    assert!(
        end_line.y >= table_bottom - 0.5 && end_line.y < table_bottom + end_line.height,
        "「끝.」 줄 위끝 {:.1} 은 표 아래끝 {:.1} 바로 아래",
        end_line.y,
        table_bottom
    );
}
