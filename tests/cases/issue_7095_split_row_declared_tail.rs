#![cfg(not(target_arch = "wasm32"))]

//! [Issue #7095] 쪽을 넘는 다행 표 행의 **끝 조각 상자**와 그 안의 배치.
//!
//! `36308670` 결재문서본문(원본 HWPX)의 본문 표 `pi=5`(8×5, RowBreak) 4행 `내용` 칸은
//! 선언 높이 35618HU(474.9px)이고 1쪽 아래에서 나뉜다. 한/글 2024 PDF 괘선 실측:
//!
//! ```text
//!   1쪽 조각  668.1 .. 1062.7  (394.6px)
//!   2쪽 조각  115.3 ..  195.7  ( 80.4px)   394.6 + 80.4 = 475.0 ≈ 선언 474.9
//! ```
//!
//! 한/글은 나뉜 칸의 저장 높이를 조각 상자 합으로 적는다. 끝 조각은 내용(세 줄 65.3px)이
//! 아니라 **선언 높이의 잔여**를 상자로 쓰고, 칸 `valign=Center` 로 세 줄을 그 안에서
//! 가운데 둔다(첫 줄이 상자 위에서 7.5px 아래). 아래 `관련 사진` 행의 Square 그림 다섯은
//! 저장 세로 오프셋(−71·−663·0·0·−561HU)과 무관하게 각 칸 가운데에 놓인다(0.2px 안).
//!
//! 수정 전 rhwp 는 끝 조각을 내용 높이로 접어 아래 행 전체를 15.1px 올렸고, 그림은 빈
//! 문단의 줄 높이(14.67px)만큼 아래로 그려 우연히 상쇄됐다. 표본은 원본의 BinData 그림
//! 바이트만 1×1 PNG 로 바꿨다(조판 XML 은 바이트 그대로 — 원본과 SVG 괘선·글자·그림 좌표가
//! 같음을 확인했다). 검사는 절대 좌표가 아니라 저장 높이·대칭·소속 관계로 한다.

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7095/36308670_split_row_declared_tail.hwpx";
const HU_PER_PX: f64 = 75.0;

fn load() -> DocumentCore {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&p).expect("표본 읽기")).expect("문서 로드")
}

struct Cell {
    row: u16,
    col: u16,
    bbox: BoundingBox,
    lines: Vec<BoundingBox>,
    images: Vec<BoundingBox>,
}

/// 본문 최상위 표 `pi=5` 의 칸들(행·열·상자·칸 안 글줄·그림).
fn body_table_cells(core: &DocumentCore, page: u32) -> Vec<Cell> {
    let tree = core.build_page_render_tree(page).expect("render tree");
    fn collect(n: &RenderNode, lines: &mut Vec<BoundingBox>, images: &mut Vec<BoundingBox>) {
        match &n.node_type {
            RenderNodeType::TextLine(_) => lines.push(n.bbox),
            RenderNodeType::Image(_) => images.push(n.bbox),
            _ => {}
        }
        for c in &n.children {
            collect(c, lines, images);
        }
    }
    fn walk(n: &RenderNode, out: &mut Vec<Cell>) {
        if let RenderNodeType::Table(t) = &n.node_type {
            if t.cell_context.is_none() && t.para_index == Some(5) {
                for c in &n.children {
                    if let RenderNodeType::TableCell(meta) = &c.node_type {
                        let (mut lines, mut images) = (Vec::new(), Vec::new());
                        for g in &c.children {
                            collect(g, &mut lines, &mut images);
                        }
                        out.push(Cell {
                            row: meta.row,
                            col: meta.col,
                            bbox: c.bbox,
                            lines,
                            images,
                        });
                    }
                }
                return;
            }
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    let mut out = Vec::new();
    walk(&tree.root, &mut out);
    out
}

fn cell(cells: &[Cell], row: u16, col: u16) -> &Cell {
    cells
        .iter()
        .find(|c| c.row == row && c.col == col)
        .unwrap_or_else(|| panic!("r{row}c{col} 칸이 없다"))
}

fn source_cell(
    core: &DocumentCore,
    row: u16,
    col: u16,
) -> (rhwp::model::table::Cell, rhwp::model::Padding) {
    let para = &core.document().sections[0].paragraphs[5];
    let table = para
        .controls
        .iter()
        .find_map(|c| match c {
            rhwp::model::control::Control::Table(t) => Some(t),
            _ => None,
        })
        .expect("표본 전제: pi=5 본문 표");
    let cell = table
        .cells
        .iter()
        .find(|c| c.row == row && c.col == col)
        .cloned()
        .expect("원본 칸");
    let padding = cell.effective_padding(&table.padding);
    (cell, padding)
}

#[test]
fn split_row_tail_fragment_keeps_the_declared_row_remainder() {
    let core = load();
    assert_eq!(core.page_count(), 2, "한/글 2024 출력과 같은 2쪽");

    let declared = f64::from(source_cell(&core, 4, 1).0.height) / HU_PER_PX;
    let p1 = body_table_cells(&core, 0);
    let p2 = body_table_cells(&core, 1);
    let first = cell(&p1, 4, 1).bbox;
    let tail = cell(&p2, 4, 1).bbox;
    assert!(
        (first.height + tail.height - declared).abs() <= 0.5,
        "나뉜 `내용` 행의 두 조각 상자 합 {:.2}+{:.2} 이 저장 칸 높이 {declared:.2}px 와 다르다",
        first.height,
        tail.height
    );
    // 같은 행의 이름 칸도 같은 끝 조각 상자를 쓴다.
    assert!((cell(&p2, 4, 0).bbox.height - tail.height).abs() <= 0.05);

    // 다음 행(관련 사진)은 끝 조각 바로 아래에서 시작한다 — 앞 조각 높이가 그대로 흐름을 민다.
    let photo = cell(&p2, 5, 0).bbox;
    assert!(
        (photo.y - (tail.y + tail.height)).abs() <= 0.05,
        "관련 사진 행 {:.2} 이 끝 조각 아래 {:.2} 에서 시작하지 않는다",
        photo.y,
        tail.y + tail.height
    );
}

#[test]
fn split_row_tail_fragment_centers_its_remaining_lines() {
    let core = load();
    let (src, padding) = source_cell(&core, 4, 1);
    assert!(
        matches!(
            src.vertical_align,
            rhwp::model::table::VerticalAlign::Center
        ),
        "표본 전제: `내용` 칸은 가운데 정렬"
    );
    let p2 = body_table_cells(&core, 1);
    let c = cell(&p2, 4, 1);
    assert_eq!(c.lines.len(), 3, "끝 조각은 남은 세 줄을 소유한다");
    let pad_top = f64::from(padding.top) / HU_PER_PX;
    let pad_bottom = f64::from(padding.bottom) / HU_PER_PX;
    let content_top = c.lines.iter().map(|l| l.y).fold(f64::MAX, f64::min);
    let content_bottom = c
        .lines
        .iter()
        .map(|l| l.y + l.height)
        .fold(f64::MIN, f64::max);
    let above = content_top - (c.bbox.y + pad_top);
    let below = (c.bbox.y + c.bbox.height - pad_bottom) - content_bottom;
    assert!(
        above > 1.0 && (above - below).abs() <= 0.5,
        "끝 조각 세 줄이 상자 안에서 가운데가 아니다: 위 {above:.2} / 아래 {below:.2}"
    );
}

#[test]
fn square_pictures_sit_centered_in_their_photo_cells() {
    let core = load();
    let p2 = body_table_cells(&core, 1);
    let mut seen = 0;
    for c in p2.iter().filter(|c| (5..=6).contains(&c.row) && c.col >= 1) {
        for img in &c.images {
            seen += 1;
            let above = img.y - c.bbox.y;
            let below = (c.bbox.y + c.bbox.height) - (img.y + img.height);
            assert!(
                above >= 0.0 && below >= 0.0,
                "r{}c{} 그림이 칸 밖이다: 위 {above:.2} / 아래 {below:.2}",
                c.row,
                c.col
            );
            assert!(
                (above - below).abs() <= 0.5,
                "r{}c{} 그림이 칸 가운데가 아니다: 위 {above:.2} / 아래 {below:.2}",
                c.row,
                c.col
            );
        }
    }
    assert_eq!(seen, 5, "관련 사진 칸의 그림 다섯");
}
