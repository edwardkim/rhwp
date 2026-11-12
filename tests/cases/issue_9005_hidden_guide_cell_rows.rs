//! [#9005] 관계형·고정형 회귀 — 미기입 누름틀 안내문을 담아 나눈 저장 줄과 저장 표 높이는 안내문을 숨긴 칸 배치의 근거가 아니다.
//!
//! 주석 갱신: 2026-11-12 23:48 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue-986-receipt.hwp`(진료비 계산서·영수증 서식, HWP5, 1쪽). 큰 표(문단 0 컨트롤 2,
//!   44행×13열, 칸 안 여백 미사용·표 안 여백 0)의 행 2·열 10 칸 문단은 「야간 □ 공휴일 □」 이며 미기입
//!   누름틀 두 개(안내문 「야간 진료 여부」 8자, 「공휴일 진료 여부」 9자)를 품는다. 한/글이 저장한 이 문단의
//!   LineSeg 는 안내문까지 넣고 나눈 2줄이고, 표 개체 높이(37866HU = 504.9px)도 그 2줄 행으로 잰 값이다.
//! - 독립 기준: `pdf/issue9005/issue-986-receipt-hwp-2020.pdf`(한/글 2022 Print, 로컬 오라클 engine 2020,
//!   1쪽). 안내문은 인쇄되지 않고, 그 칸은 1줄(행 높이 9.3px)이며 큰 표는 선언보다 15px 짧게 끝난다
//!   (표 위 18.9px, 아래 괘선 508.5px). 그 아래 표(컨트롤 3)는 큰 표 바로 아래 508.5px 에서 시작한다.
//! - 수정 전 rhwp(devel `80b1c354c2`)는 저장 2줄로 행 2 를 24.2px 로 그려 그 아래 모든 괘선이 15px
//!   내려갔다. 저장 줄만 다시 나누면 이번엔 측정 행을 낡은 표 선언 높이까지 비례로 늘려(#2070/#1510
//!   규칙) 행마다 0.3px, 표 끝에서 15px 가 다시 생겼다.
//!
//! 측정: 2026-11-12, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣). 전체 1쪽 = 한/글 1쪽.
//! 수정 후(devel `80b1c354c2` 기준) 1쪽 Native 98.69%, fresh WASM 99.99%(두 경로 전쪽 최저값도 같음).
//! 수정 전 Native 57.07%.
//! 증거: `mydocs/pr/assets/issue_9005/{native,wasm,base-native}/`.
//!
//! 검사: 원본 계약(칸 문단의 저장 줄 2개, 안내문 잔재 둘), 쪽수 1 = 한/글(고정형), 행 2 높이 = 같은
//! 1줄 행인 행 1 높이, 본문 행 높이 = max(칸 선언, 줄 + 유효 안 여백) 중 최댓값, 아래 표 위 = 큰 표 아래(관계형),
//! 큰 표 아래 괘선 = PDF 508.5px ±2px(고정형, 양쪽 raw 98% 이상·확대 판독 완료).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue-986-receipt.hwp";
/// 큰 표(44행×13열)를 품은 본문 문단과 컨트롤 번호.
const HOST: usize = 0;
const BIG_TABLE: usize = 2;
/// 큰 표 바로 아래 표.
const NEXT_TABLE: usize = 3;
/// 한/글 Print PDF 의 큰 표 아래 괘선(96dpi px).
const PDF_BIG_TABLE_BOTTOM: f64 = 508.5;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn table_node(node: &RenderNode, control: usize) -> Option<&RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(HOST)
            && table.control_index == Some(control)
            && table.cell_context.is_none()
        {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| table_node(child, control))
}

fn cell_box(table: &RenderNode, row: u16, col: u16) -> (f64, f64) {
    table
        .children
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TableCell(cell) if cell.row == row && cell.col == col => {
                Some((node.bbox.y, node.bbox.height))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("칸 ({row},{col})"))
}

fn big_table(core: &DocumentCore) -> &Table {
    match &core.document().sections[0].paragraphs[HOST].controls[BIG_TABLE] {
        Control::Table(table) => table,
        _ => panic!("문단 {HOST} 컨트롤 {BIG_TABLE} 은 큰 표"),
    }
}

#[test]
fn hidden_guide_cell_lays_out_one_line_and_table_ends_at_content() {
    let core = load();
    assert_eq!(core.page_count(), 1, "한/글 Print 와 같은 1쪽");

    let table = big_table(&core);
    assert_eq!((table.row_count, table.col_count), (44, 13));
    let guide_cell = table
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 10)
        .expect("「야간 □ 공휴일 □」 칸");
    let para = &guide_cell.paragraphs[0];
    // 원본 계약: 한/글이 안내문까지 넣고 나눈 저장 줄 2개, 적재 정규화가 남긴 안내문 잔재 2개.
    assert_eq!(para.line_segs.len(), 2, "저장 줄은 안내문을 담은 2줄");
    let residues: Vec<&str> = para
        .controls
        .iter()
        .filter_map(|control| match control {
            Control::Field(field) => field.guide_residue.as_ref().map(|r| r.text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        residues.len(),
        2,
        "미기입 누름틀 안내문 잔재 둘: {residues:?}"
    );
    assert!(table.common.height > 0);

    let page = core.build_page_render_tree(0).expect("1쪽");
    let big = table_node(&page.root, BIG_TABLE).expect("큰 표 상자");

    // 행 2(안내문 칸이 있는 행)는 같은 1줄 행인 행 1 과 높이가 같다.
    let (_, row1_h) = cell_box(big, 1, 0);
    let (_, row2_h) = cell_box(big, 2, 10);
    assert!(
        (row2_h - row1_h).abs() <= 0.1,
        "행 2 높이 {row2_h:.2} = 1줄 행 1 높이 {row1_h:.2}"
    );

    // 본문 행 높이 = 그 행 단일 칸들의 max(칸 선언 높이, 1줄 저장 줄 높이 + 유효 상하 안 여백) 중
    // 최댓값이다(한/글 행 높이 규칙). 낡은 표 선언 높이로 늘리지 않는다.
    for row in [8u16, 20, 30, 40] {
        let row_cells: Vec<_> = table
            .cells
            .iter()
            .filter(|cell| cell.row == row && cell.row_span == 1)
            .collect();
        assert!(
            row_cells
                .iter()
                .all(|cell| cell.paragraphs.len() == 1 && cell.paragraphs[0].line_segs.len() == 1),
            "행 {row} 의 단일 칸은 모두 1줄 문단 하나"
        );
        let expected_hu = row_cells
            .iter()
            .map(|cell| {
                let content = cell.paragraphs[0].line_segs[0].line_height
                    + cell.effective_vertical_padding_hu(&table.padding);
                content.max(cell.height as i32)
            })
            .max()
            .unwrap_or_else(|| panic!("행 {row} 의 단일 칸"));
        let (_, h) = cell_box(big, row, row_cells[0].col);
        assert!(
            (h - hu_to_px(expected_hu)).abs() <= 0.1,
            "행 {row} 높이 {h:.2} = max(칸 선언, 줄+안 여백) {:.2}",
            hu_to_px(expected_hu)
        );
    }

    // 큰 표 아래 괘선은 한/글 PDF 와 같고, 아래 표는 그 바로 아래에서 시작한다.
    let big_bottom = big.bbox.y + big.bbox.height;
    assert!(
        big_bottom < big.bbox.y + hu_to_px(table.common.height as i32) - 10.0,
        "큰 표 아래 {big_bottom:.1} 는 안내문을 담아 잰 선언 높이보다 위"
    );
    assert!(
        (big_bottom - PDF_BIG_TABLE_BOTTOM).abs() <= 2.0,
        "큰 표 아래 {big_bottom:.1} = 한/글 PDF {PDF_BIG_TABLE_BOTTOM} ±2"
    );
    let next = table_node(&page.root, NEXT_TABLE).expect("아래 표 상자");
    assert!(
        (next.bbox.y - big_bottom).abs() <= 0.5,
        "아래 표 위 {:.1} = 큰 표 아래 {big_bottom:.1}",
        next.bbox.y
    );
}
