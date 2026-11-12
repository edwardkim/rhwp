//! 사진 칸(글자 없는 문단 하나 + 자리차지 그림) 아래 캡션 칸 행이 쪽 끝에 놓인 쪽 나눔 표.
//!
//! 놀이 계획표의 사진 줄 꼴이다. 사진 칸은 저장 줄 없는 빈 문단 하나뿐이고 그 문단에 문단
//! 기준 자리차지(TopAndBottom) 그림이 있다. 설명 글은 다음 행의 별도 칸(캡션 칸)에 있다.
//! 왼쪽 열 칸이 표 끝까지 세로로 걸쳐 쪽 경계를 지나고, 오른쪽 열은 사진 행 + 캡션 행에 걸친
//! 빈 칸이다.
//!
//! 저장 줄 없는 빈 문단의 400HU 자리 줄을 개체 유닛에서 빼는 규칙(제목 문단 + 사진 칸)은 앞에
//! 내용이 있는 마지막 빈 문단에만 쓴다. 문단이 하나뿐인 사진 칸까지 넓히면 행 나눔 장부가 첫
//! 쪽에 남는 공간을 다르게 세고, 다음 사진 행이 근소하게 못 들어가 남은 띠가 걸침 칸을 따라
//! 마지막 캡션 행에 붙는다 — 캡션 행이 선언보다 커지고 오른쪽 걸침 칸도 같이 늘어난다(실문서에서
//! 쪽 끝 사진 줄이 정답보다 약 15px 커진 증상과 같은 경로).
//!
//! 재현물 `samples/photo_caption_rows/photo-caption-rows-page-end.hwp` 는 생성기
//! `tools/fixture_generators/build_photo_caption_rows_fixture.rs` 가 공개 합성본
//! `samples/title_flow_photo/title-flow-photo-row.hwp` 의 그림 문단·글자 문단만으로 만든다.
//! 행: 0 = 여백 행, 1..=12 = 사진·캡션 행 묶음 6개(홀수 = 사진, 짝수 = 캡션), 13.. = 본문 행.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/photo_caption_rows/photo-caption-rows-page-end.hwp";
/// 칸 안 여백 141HU(px).
const PAD_PX: f64 = 141.0 * 96.0 / 7200.0;
/// 그림 높이 6000HU(px).
const PHOTO_PX: f64 = 6000.0 * 96.0 / 7200.0;
/// 캡션 행 선언 높이 1989HU(px).
const CAPTION_DECLARED_PX: f64 = 1989.0 * 96.0 / 7200.0;
const TOL_PX: f64 = 0.5;

#[derive(Debug, Clone, Copy)]
struct CellBox {
    row: u16,
    col: u16,
    h: f64,
}

fn cells_by_page() -> Vec<Vec<CellBox>> {
    fn walk(n: &RenderNode, out: &mut Vec<CellBox>) {
        if let RenderNodeType::TableCell(cell) = &n.node_type {
            out.push(CellBox {
                row: cell.row,
                col: cell.col,
                h: n.bbox.height,
            });
            return;
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("render tree");
            let mut cells = Vec::new();
            walk(&tree.root, &mut cells);
            cells
        })
        .collect()
}

/// 첫 쪽은 다섯째 묶음의 캡션 행(10)에서 끝나고 여섯째 사진 행(11)은 다음 쪽에서 시작한다.
#[test]
fn first_page_ends_after_fifth_caption_row() {
    let pages = cells_by_page();
    assert!(pages.len() >= 2, "표가 두 쪽에 걸친다: {} 쪽", pages.len());
    let last_photo_col_row = pages[0].iter().filter(|c| c.col == 1).map(|c| c.row).max();
    assert_eq!(
        last_photo_col_row,
        Some(10),
        "1쪽 사진 열의 마지막 행은 다섯째 캡션 행이다: {:?}",
        pages[0]
    );
    assert!(
        pages[1].iter().any(|c| c.row == 11 && c.col == 1),
        "여섯째 사진 행은 2쪽에 있다: {:?}",
        pages[1]
    );
}

/// 사진 행 높이는 안 여백 + 그림이다(문단 하나뿐인 사진 칸에 자리 줄을 더하지 않는다).
#[test]
fn photo_rows_close_at_picture_bottom() {
    for c in cells_by_page()[0]
        .iter()
        .filter(|c| c.col == 1 && c.row % 2 == 1)
    {
        let expected = PAD_PX * 2.0 + PHOTO_PX;
        assert!(
            (c.h - expected).abs() <= TOL_PX,
            "사진 행 {} 높이({:.2})가 안 여백 + 그림({expected:.2})이어야 한다",
            c.row,
            c.h
        );
    }
}

/// 쪽 끝 캡션 행과 그 행까지 걸친 오른쪽 칸은 남은 띠로 늘어나지 않는다.
#[test]
fn page_end_caption_row_is_not_stretched_by_remaining_band() {
    let pages = cells_by_page();
    let first = &pages[0];
    for c in first
        .iter()
        .filter(|c| c.col == 1 && c.row % 2 == 0 && c.row > 0)
    {
        assert!(
            c.h <= CAPTION_DECLARED_PX + TOL_PX,
            "1쪽 캡션 행 {} 높이({:.2})가 선언({CAPTION_DECLARED_PX:.2})을 넘지 않아야 한다",
            c.row,
            c.h
        );
    }
    let photo_row_px = PAD_PX * 2.0 + PHOTO_PX;
    for c in first.iter().filter(|c| c.col == 2 && c.row % 2 == 1) {
        assert!(
            c.h <= photo_row_px + CAPTION_DECLARED_PX + TOL_PX,
            "1쪽 오른쪽 걸침 칸(행 {}) 높이({:.2})가 사진 행 + 캡션 선언({:.2})을 넘지 않아야 한다",
            c.row,
            c.h,
            photo_row_px + CAPTION_DECLARED_PX
        );
    }
}
