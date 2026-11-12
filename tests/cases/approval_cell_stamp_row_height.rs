//! 결재 칸(이름을 한 글자씩 여러 문단으로 세로로 쓰고 도장 그림을 둔 칸)의 행 높이.
//!
//! 저장 줄(LINE_SEG)이 없는 칸에서 문단 기준 개체·글자 없는 문단의 글자처럼 취급한
//! 그림은 그 문단의 흐름 위치에서 시작한다. 종전 측정은 저장 vpos 가 없으면 문단 원점을
//! 0 으로 두어, 셋째·넷째 문단의 도장이 첫 줄 위에 있는 것으로 계산했다. 그래서 행이 선언
//! 높이에 머물고(도장이 칸 아래로 잘림), 세로 가운데 정렬도 도장 몫 없이 잡혔다.
//!
//! 재현물 `samples/approval_cell/approval-cell-stamp-rows.hwp` 는 공개 샘플
//! (`samples/test-image.hwpx`, `samples/issue3738/tac_sibling_shape_line_advance.hwpx`)만
//! 조합한 합성본이다(저장 줄 없음, 원본 HWP5 경로). 표 세 개:
//! - 표 1: "가|나|다" + 넷째(글자 없는) 문단의 글자처럼 취급한 도장
//! - 표 2: "라|마|바" + 셋째 문단의 문단 기준 어울림(Square) 도장(세로 오프셋 851HU). 이름 칸
//!   폭(4000HU)은 도장 오른쪽에 "바" 한 글자가 드는 폭이다(그보다 좁으면 한컴은 "바" 줄을 도장
//!   아래로 내린다 — 어울림 글 흐름, 이 검사와 무관).
//! - 표 3: 표 1 과 같은 결재 행 뒤에 본문 행 40개가 이어져 쪽을 넘는다. 첫 쪽 조각은
//!   쪽 나눔 표 경로로 그려지므로, 그 경로의 세로 정렬도 같은 원장을 쓰는지 본다.
//! - 표 4: 넉넉한 칸(선언 30000HU)의 둘째 문단 어울림 그림(세로 오프셋 3374HU, 높이 11498HU).
//!   행은 늘지 않고, 세로 가운데 정렬은 종전 회계(오프셋 + 그림 높이)로 잡힌다. 같은 형상의
//!   실문서 준비물 칸 정답에서 글 시작 여백이 (안 높이 − (오프셋 + 그림 높이)) / 2 와 같았다.
//!   흐름 원점까지 넣어 정렬하면 글과 그림이 위로 올라간다(수정 중 실측한 회귀).
//! - 표 5: 넉넉한 칸(선언 20000HU)의 제목 한 줄 + 글자 없는 문단의 글자처럼 취급한 사진.
//!   쪽 나눔으로 새 쪽(3쪽)에서 시작한다.
//!   사진 줄은 흐름 내용이라 제목 줄 + 사진이 함께 가운데 정렬된다(실문서 사진 칸 정답과 같은
//!   관계: 위 여유 = 아래 여유).
//! - 표 6: 표 5 와 같되 사진이 글자 없는 문단의 어울림 그림(세로 오프셋 0)이다. 글자 없는
//!   문단은 사진을 놓으려고 둔 자리라 같은 관계가 성립한다.
//!
//! 웹한글기안기 인쇄본(`pdf/approval_cell/approval-cell-stamp-rows-webhwp-print.pdf`)에서 표 1·3
//! 결재 행은 66.0px, 표 2 는 61.0px(= 안 여백 + 글줄 둘 + 세로 오프셋 + 도장 + 안 여백)이다.
//! 기대값은 같은 형상의 실문서 정답(웹한글기안기 쪽 그림)에서 끌어낸 관계로 적는다 — 실문서
//! 두 건에서 결재 행 높이는
//! `안 여백 위 + 글줄 + 도장 + 안 여백 아래` 와 0.3px 안에서 같았다(문서 A 62.1 대 61.9,
//! 문서 B 55.4 대 55.5). 곧 칸은 내용에 꼭 맞게 늘어나고, 세로 가운데 정렬 여유가 0 이라
//! 첫 글줄은 칸 위 + 안 여백에, 도장 바닥은 칸 아래 − 안 여백에 놓인다. 어울림 도장은
//! 셋째 글줄 위에서 세로 오프셋만큼 내려간 자리다. 절대 좌표는 핀하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/approval_cell/approval-cell-stamp-rows.hwp";
/// 재현물 칸 안 여백 141HU(px).
const PAD_PX: f64 = 141.0 * 96.0 / 7200.0;
/// 표 4 어울림 그림의 세로 오프셋 3374HU·높이 11498HU(px).
const ROOMY_PIC_EXTENT_PX: f64 = (3374.0 + 11498.0) * 96.0 / 7200.0;
const ROOMY_PIC_VOFF_PX: f64 = 3374.0 * 96.0 / 7200.0;
/// 표 4 칸 선언 높이 30000HU(px).
const ROOMY_CELL_H_PX: f64 = 30000.0 * 96.0 / 7200.0;
/// 재현물 칸 선언 높이 3992HU(px).
const DECLARED_CELL_H_PX: f64 = 3992.0 * 96.0 / 7200.0;
/// 어울림 도장의 세로 오프셋 851HU(px).
const SQUARE_VOFF_PX: f64 = 851.0 * 96.0 / 7200.0;
const TOL_PX: f64 = 0.5;

#[derive(Debug, Clone, Copy)]
struct Rect {
    y: f64,
    h: f64,
}

impl Rect {
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
}

/// 이름 칸 하나: 칸 상자, 글줄(문단 순서), 도장.
#[derive(Debug)]
struct NameCell {
    cell: Rect,
    lines: Vec<(usize, Rect)>,
    stamp: Rect,
}

fn name_cells() -> Vec<NameCell> {
    fn rect(n: &RenderNode) -> Rect {
        Rect {
            y: n.bbox.y,
            h: n.bbox.height,
        }
    }
    fn collect(n: &RenderNode, lines: &mut Vec<(usize, Rect)>, images: &mut Vec<Rect>) {
        match &n.node_type {
            RenderNodeType::TextLine(line) => {
                lines.push((line.para_index.unwrap_or(usize::MAX), rect(n)));
            }
            RenderNodeType::Image(_) => images.push(rect(n)),
            _ => {}
        }
        for c in &n.children {
            collect(c, lines, images);
        }
    }
    fn walk(n: &RenderNode, out: &mut Vec<NameCell>) {
        if matches!(n.node_type, RenderNodeType::TableCell(_)) {
            let mut lines = Vec::new();
            let mut images = Vec::new();
            collect(n, &mut lines, &mut images);
            if let [stamp] = images.as_slice() {
                lines.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
                out.push(NameCell {
                    cell: rect(n),
                    lines,
                    stamp: *stamp,
                });
            }
            return;
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        3,
        "표 3 이 쪽을 넘고 표 4~6 이 뒤 쪽에 놓인다"
    );
    let mut cells = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render tree");
        walk(&tree.root, &mut cells);
    }
    assert_eq!(
        cells.len(),
        6,
        "그림을 하나 담은 칸 여섯 개(표 순서): {cells:?}"
    );
    cells
}

/// 칸이 내용에 꼭 맞게 늘어난다: 첫 글줄은 칸 위 + 안 여백, 도장 바닥은 칸 아래 − 안 여백.
fn assert_content_fills_cell(name: &str, c: &NameCell) {
    let first = c.lines.first().expect("글줄").1;
    assert!(
        c.cell.h > DECLARED_CELL_H_PX + TOL_PX,
        "{name}: 칸({:.2})이 선언 높이({DECLARED_CELL_H_PX:.2})보다 늘어나야 한다",
        c.cell.h
    );
    assert!(
        c.stamp.bottom() + PAD_PX <= c.cell.bottom() + TOL_PX,
        "{name}: 도장 바닥({:.2}) + 안 여백이 칸 아래({:.2}) 안에 있어야 한다",
        c.stamp.bottom(),
        c.cell.bottom()
    );
    assert!(
        (first.y - (c.cell.y + PAD_PX)).abs() <= TOL_PX,
        "{name}: 첫 글줄 위({:.2})가 칸 위 + 안 여백({:.2})이어야 한다(정렬 여유 0)",
        first.y,
        c.cell.y + PAD_PX
    );
    assert!(
        (c.cell.bottom() - PAD_PX - c.stamp.bottom()).abs() <= TOL_PX,
        "{name}: 도장 바닥({:.2})이 칸 아래 − 안 여백({:.2})이어야 한다(칸이 내용에 꼭 맞음)",
        c.stamp.bottom(),
        c.cell.bottom() - PAD_PX
    );
}

/// 넷째(글자 없는) 문단의 글자처럼 취급한 도장은 셋째 글줄 바로 아래 줄을 차지한다.
fn assert_tac_stamp_cell(name: &str, c: &NameCell) {
    let text_lines: Vec<Rect> = c
        .lines
        .iter()
        .filter(|(_, r)| r.y + TOL_PX < c.stamp.y)
        .map(|(_, r)| *r)
        .collect();
    assert_eq!(text_lines.len(), 3, "{name}: 도장 위 글줄 세 개: {c:?}");
    let last_text = text_lines.last().unwrap();
    assert!(
        (c.stamp.y - last_text.bottom()).abs() <= TOL_PX,
        "{name}: 도장 위({:.2})가 셋째 글줄 아래({:.2})에 붙어야 한다",
        c.stamp.y,
        last_text.bottom()
    );
    assert_content_fills_cell(name, c);
}

/// 표 1: 글자처럼 취급한 도장 몫만큼 행이 늘고, 가운데 정렬 여유가 0 이다.
#[test]
fn tac_stamp_in_fourth_paragraph_grows_row() {
    assert_tac_stamp_cell("표 1 글자처럼 취급한 도장", &name_cells()[0]);
}

/// 표 3: 쪽을 넘는 표의 첫 조각(쪽 나눔 표 경로)도 같은 행 높이·정렬을 쓴다.
#[test]
fn tac_stamp_row_in_page_split_table_keeps_alignment() {
    assert_tac_stamp_cell("표 3 쪽 나눔 표 첫 조각", &name_cells()[2]);
}

/// 표 2: 셋째 문단의 문단 기준 어울림 도장은 그 문단 글줄 위에서 세로 오프셋만큼 내려간다.
#[test]
fn square_stamp_on_third_paragraph_grows_row() {
    let cells = name_cells();
    let c = &cells[1];
    assert_eq!(c.lines.len(), 3, "글줄 세 개: {c:?}");
    let third = c.lines[2].1;
    assert!(
        (c.stamp.y - (third.y + SQUARE_VOFF_PX)).abs() <= TOL_PX,
        "도장 위({:.2})가 셋째 글줄 위 + 세로 오프셋({:.2})이어야 한다",
        c.stamp.y,
        third.y + SQUARE_VOFF_PX
    );
    assert_content_fills_cell("어울림 도장", c);
}

/// 표 4: 넉넉한 칸의 어울림 그림은 행을 키우지 않고, 정렬은 종전 회계(오프셋 + 그림 높이)다.
#[test]
fn roomy_square_picture_keeps_object_extent_alignment() {
    let cells = name_cells();
    let c = &cells[3];
    assert!(
        (c.cell.h - ROOMY_CELL_H_PX).abs() <= TOL_PX,
        "칸({:.2})은 선언 높이({ROOMY_CELL_H_PX:.2})를 유지한다",
        c.cell.h
    );
    let first = c.lines.first().expect("글줄").1;
    let inner = c.cell.h - 2.0 * PAD_PX;
    let expected_gap = (inner - ROOMY_PIC_EXTENT_PX) / 2.0;
    assert!(
        (first.y - (c.cell.y + PAD_PX) - expected_gap).abs() <= TOL_PX,
        "글 시작 여백({:.2})이 (안 높이 − (오프셋 + 그림 높이)) / 2 ({expected_gap:.2}) 와 같아야 한다",
        first.y - (c.cell.y + PAD_PX)
    );
    let second = c.lines[1].1;
    assert!(
        (c.stamp.y - (second.y + ROOMY_PIC_VOFF_PX)).abs() <= TOL_PX,
        "그림 위({:.2})가 둘째 글줄 위 + 세로 오프셋({:.2})이어야 한다",
        c.stamp.y,
        second.y + ROOMY_PIC_VOFF_PX
    );
}

/// 제목 줄 + 그 아래 사진이 한 덩어리로 가운데 정렬된다(위 여유 = 아래 여유).
fn assert_title_and_photo_centered(name: &str, c: &NameCell) {
    let title = c
        .lines
        .iter()
        .map(|(_, r)| *r)
        .find(|r| r.y + TOL_PX < c.stamp.y)
        .expect("사진 위 제목 줄");
    assert!(
        (c.stamp.y - title.bottom()).abs() <= TOL_PX,
        "{name}: 사진 위({:.2})가 제목 줄 아래({:.2})에 붙어야 한다",
        c.stamp.y,
        title.bottom()
    );
    let top_gap = title.y - (c.cell.y + PAD_PX);
    let bottom_gap = c.cell.bottom() - PAD_PX - c.stamp.bottom();
    assert!(
        (top_gap - bottom_gap).abs() <= TOL_PX,
        "{name}: 위 여유({top_gap:.2})와 아래 여유({bottom_gap:.2})가 같아야 한다"
    );
}

/// 표 5: 글자처럼 취급한 사진.
#[test]
fn inline_photo_under_title_centers_with_title() {
    assert_title_and_photo_centered("표 5 글자처럼 취급한 사진", &name_cells()[4]);
}

/// 표 6: 글자 없는 문단의 어울림 사진.
#[test]
fn floating_photo_in_empty_paragraph_centers_with_title() {
    assert_title_and_photo_centered("표 6 글자 없는 문단의 어울림 사진", &name_cells()[5]);
}
