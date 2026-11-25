//! 관계형·고정형 검사. 사진끼리의 겹침과 칸 내부 배치, 독립 인쇄의 한 쪽 구성을 검사한다.
//! 2026-11-25 21:55 KST 갱신. 직전 변경은 2026-11-25 21:13:07 +0900, 6f3b11acfa.
//! 96 DPI, 2px raw. 원본·오른쪽·가운데·가로로 맞닿은 대조 HWP 각 1쪽 전부 Native와 fresh WASM
//! 100.00%. 독립 한컴 2020 PDF는 `pdf/pr9349-cell-floats/`, TSV는
//! `mydocs/pr/assets/hyunmyungjanelee-20261125/maintainer/`에 보존한다.
//! 용지 높이를 줄인 대조 HWP 두 개도 각 1쪽, Native·fresh WASM 모두 100.00%였다.
//! 한컴 2020 Print에서 용지 끝에 일부만 보이는 사진은 720pt 용지의 7행 둘째 사진과
//! 595pt 용지의 6행 첫째 사진이다. 원본에는 사진 15장이 모두 보인다.
//!
//! 저장 줄 없는 사진 칸에 나란히 놓인 자리차지 그림 두 장의 행 높이.
//!
//! 글자처럼 취급한 쪽 나눔(RowBreak) 표에서 글자 없는 문단 하나가 문단 기준 자리차지
//! (TopAndBottom) 그림을 둘 이상 앵커하면, 저장 줄이 있는 칸은 한컴이 그림마다 보존한 줄
//! 순서대로 높이를 쌓는다(#7333 아이콘 표). 저장 줄이 없는 칸에는 그런 순서가 없는데도 같은
//! 합산을 적용해, 가로로 나란히 놓인 사진 두 장 칸을 사진 두 장 높이로 쟀다. 사진 행이 두 배가
//! 되어 표가 다음 쪽으로 밀리고 문서가 한 쪽 길어졌다(생성계 활동 보고서 "활동 모습" 칸).
//!
//! 재현물 `samples/side_by_side_photo_cell/side-by-side-photo-rows.hwp` 는 공개 합성본
//! (`samples/title_flow_photo/title-flow-photo-row.hwp`)만 조합했다. 8행 x 2칸, 칸 선언 높이는
//! 그림 높이 + 안 여백보다 조금 크다(6616HU). 0~5행 사진 칸은 그림 두 장이 가로로 나란하고,
//! 6행(대조군)은 그림 한 장, 7행(대조군)은 그림 두 장이 가로로 겹친다. 글자 모양은 10pt 다.
//!
//! 웹한글기안기 인쇄본(`pdf/side_by_side_photo_cell/side-by-side-photo-rows-webhwp-print.pdf`)의
//! 행 높이는 97.0 / 89.0 / 177.0px 이다:
//! - 나란한 두 장: 그림 띠 하나 + 빈 앵커 줄(10pt) + 안 여백 = 80.0 + 13.3 + 3.8.
//! - 한 장: 칸 선언 높이(앵커 줄은 그림 자리로 흡수된다, #6660).
//! - 겹친 두 장: 그림 두 장 + 빈 앵커 줄 + 안 여백 = 160.0 + 13.3 + 3.8.
//!
//! 기대값은 이 규칙으로 적고, 절대 좌표는 핀하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/side_by_side_photo_cell/side-by-side-photo-rows.hwp";
const HU_PX: f64 = 96.0 / 7200.0;
/// 칸 선언 높이 6616HU (px).
const DECLARED_PX: f64 = 6616.0 * HU_PX;
/// 그림 높이 6000HU (px).
const PIC_PX: f64 = 6000.0 * HU_PX;
/// 빈 앵커 줄: 10pt 글자의 줄 높이 1000HU (px).
const ANCHOR_LINE_PX: f64 = 1000.0 * HU_PX;
/// 안 여백 위아래 141HU x 2 (px).
const PAD_PX: f64 = 282.0 * HU_PX;
const SIDE_ROWS: u16 = 6;
const SINGLE_ROW: u16 = 6;
const OVERLAP_ROW: u16 = 7;
const TOL_PX: f64 = 0.5;

fn load() -> DocumentCore {
    load_path(FIXTURE)
}

fn load_path(fixture: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(fixture);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 1쪽의 행 번호 → 그린 높이(행 안 칸 상자의 위·아래 끝).
fn first_page_rows(core: &DocumentCore) -> BTreeMap<u16, f64> {
    fn walk(n: &RenderNode, rows: &mut BTreeMap<u16, (f64, f64)>) {
        if let RenderNodeType::TableCell(cell) = &n.node_type {
            let band = rows.entry(cell.row).or_insert((f64::MAX, f64::MIN));
            band.0 = band.0.min(n.bbox.y);
            band.1 = band.1.max(n.bbox.y + n.bbox.height);
            return;
        }
        for c in &n.children {
            walk(c, rows);
        }
    }
    let tree = core.build_page_render_tree(0).expect("render tree");
    let mut rows = BTreeMap::new();
    walk(&tree.root, &mut rows);
    rows.into_iter()
        .map(|(row, (top, bottom))| (row, bottom - top))
        .collect()
}

fn row_height(rows: &BTreeMap<u16, f64>, row: u16) -> f64 {
    *rows
        .get(&row)
        .unwrap_or_else(|| panic!("1쪽에 행 {row} 이 있어야 한다: {:?}", rows.keys()))
}

/// 표 전체가 1쪽 본문 안에 들어간다(나란한 사진 행이 두 배로 부풀지 않는다).
#[test]
fn side_by_side_photo_table_fits_body() {
    fn walk(n: &RenderNode, body_bottom: &mut f64, table_bottom: &mut f64) {
        match &n.node_type {
            RenderNodeType::Body { .. } => *body_bottom = n.bbox.y + n.bbox.height,
            RenderNodeType::Table(_) => {
                *table_bottom = table_bottom.max(n.bbox.y + n.bbox.height);
                return;
            }
            _ => {}
        }
        for c in &n.children {
            walk(c, body_bottom, table_bottom);
        }
    }
    let core = load();
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).expect("render tree");
    let (mut body_bottom, mut table_bottom) = (0.0, f64::MIN);
    walk(&tree.root, &mut body_bottom, &mut table_bottom);
    assert!(
        table_bottom <= body_bottom + TOL_PX,
        "표 아래 끝 {table_bottom:.2} 가 본문 아래 끝 {body_bottom:.2} 을 넘는다 \
         (나란한 사진 칸을 그림 두 장 높이로 잰 것)"
    );
}

/// 나란한 사진 칸 행은 그림 띠 하나 + 빈 앵커 줄 + 안 여백으로 그려진다.
#[test]
fn side_by_side_photo_row_is_one_picture_band_plus_anchor_line() {
    let rows = first_page_rows(&load());
    let expected = PIC_PX + ANCHOR_LINE_PX + PAD_PX;
    for row in 0..SIDE_ROWS {
        let height = row_height(&rows, row);
        assert!(
            (height - expected).abs() <= TOL_PX,
            "행 {row}: 그린 높이 {height:.2} 가 그림 띠 + 앵커 줄 + 안 여백 {expected:.2} 이어야 한다 \
             (그림 두 장 합 {:.2} 이면 나란한 그림을 쌓은 것)",
            2.0 * PIC_PX + PAD_PX
        );
    }
}

/// 대조군: 그림 한 장 칸은 종전처럼 칸 선언 높이로 그려진다(앵커 줄을 더하지 않는다).
#[test]
fn single_picture_row_keeps_declared_height() {
    let height = row_height(&first_page_rows(&load()), SINGLE_ROW);
    assert!(
        (height - DECLARED_PX).abs() <= TOL_PX,
        "그림 한 장 행: 그린 높이 {height:.2} 가 선언 {DECLARED_PX:.2} 이어야 한다"
    );
}

/// 대조군: 가로로 겹치는 그림 두 장은 세로로 쌓이고, 빈 앵커 줄이 그 아래에 놓인다.
#[test]
fn overlapping_pictures_row_stacks_with_anchor_line() {
    let height = row_height(&first_page_rows(&load()), OVERLAP_ROW);
    let expected = 2.0 * PIC_PX + ANCHOR_LINE_PX + PAD_PX;
    assert!(
        (height - expected).abs() <= TOL_PX,
        "겹친 그림 행: 그린 높이 {height:.2} 가 그림 두 장 + 앵커 줄 + 안 여백 {expected:.2} 이어야 한다"
    );
}

/// 안 여백 위 141HU (px).
const PAD_TOP_PX: f64 = 141.0 * HU_PX;
/// 대조군 둘째 그림의 가로 오프셋 4000HU (px): 첫 그림과 가로로 겹친다.
const OVERLAP_X_PX: f64 = 4000.0 * HU_PX;

/// 1쪽 사진 칸(1열)의 행 번호 → (칸 위 끝, 그림 상자 (x, y) 목록).
fn first_page_photo_cells(core: &DocumentCore) -> BTreeMap<u16, (f64, Vec<(f64, f64)>)> {
    fn images(n: &RenderNode, out: &mut Vec<(f64, f64)>) {
        if let RenderNodeType::Image(_) = &n.node_type {
            out.push((n.bbox.x, n.bbox.y));
        }
        for c in &n.children {
            images(c, out);
        }
    }
    fn walk(n: &RenderNode, cells: &mut BTreeMap<u16, (f64, Vec<(f64, f64)>)>) {
        if let RenderNodeType::TableCell(cell) = &n.node_type {
            if cell.col == 1 {
                let mut boxes = Vec::new();
                images(n, &mut boxes);
                cells.insert(cell.row, (n.bbox.y, boxes));
            }
            return;
        }
        for c in &n.children {
            walk(c, cells);
        }
    }
    let tree = core.build_page_render_tree(0).expect("render tree");
    let mut cells = BTreeMap::new();
    walk(&tree.root, &mut cells);
    cells
}

#[test]
fn page_edge_clips_only_the_photos_outside_the_shortened_paper() {
    for (path, visible_count, partial_photo) in [
        (FIXTURE, 15, None),
        (
            "samples/pr9349-cell-floats/photo-short-page.hwp",
            15,
            Some((OVERLAP_ROW, 1)),
        ),
        (
            "samples/pr9349-cell-floats/photo-a5-height.hwp",
            13,
            Some((SINGLE_ROW, 0)),
        ),
    ] {
        let core = load_path(path);
        assert_eq!(core.page_count(), 1, "{path}");
        let tree = core.build_page_render_tree(0).expect("render tree");
        let page_bottom = tree.root.bbox.y + tree.root.bbox.height;
        let mut visible = 0;
        let mut partial = Vec::new();
        for (row, (_, photos)) in first_page_photo_cells(&core) {
            for (index, (_, top)) in photos.iter().enumerate() {
                if *top < page_bottom {
                    visible += 1;
                    if top + PIC_PX > page_bottom {
                        partial.push((row, index));
                    }
                }
            }
        }
        assert_eq!(visible, visible_count, "{path}: 용지에 보이는 사진 수");
        assert_eq!(
            partial.as_slice(),
            partial_photo.as_slice(),
            "{path}: 용지 끝 사진"
        );
    }
}

/// 나란한 사진 칸: 그림 두 장이 같은 높이에 놓이고, 그림 띠 + 글자 크기 앵커 줄이 칸을 채워
/// 가운데 정렬이어도 그림이 안 여백 바로 아래에서 시작한다(한컴 2020 인쇄본).
#[test]
fn side_by_side_pictures_start_at_cell_content_top() {
    let cells = first_page_photo_cells(&load());
    for row in 0..SIDE_ROWS {
        let (cell_top, boxes) = &cells[&row];
        assert_eq!(boxes.len(), 2, "행 {row}: 그림 두 장");
        for &(_, y) in boxes {
            assert!(
                (y - (cell_top + PAD_TOP_PX)).abs() <= TOL_PX,
                "행 {row}: 그림 위 {y:.2} 가 칸 위 {cell_top:.2} + 안 여백 {PAD_TOP_PX:.2} 이어야 한다 \
                 (앵커 줄을 400HU 로 세면 가운데 정렬이 그림을 약 4px 내린다)"
            );
        }
    }
}

/// 대조군: 겹침 허용 안 함인 자리차지 그림이 같은 문단 앞 그림과 가로로 겹치면 한컴은
/// 둘째 그림을 첫 그림 아래로 내린다. 가로 자리는 오프셋 그대로다.
#[test]
fn overlapping_no_overlap_pictures_stack_vertically() {
    let cells = first_page_photo_cells(&load());
    let (cell_top, boxes) = &cells[&OVERLAP_ROW];
    let [(x0, y0), (x1, y1)] = boxes.as_slice() else {
        panic!("겹친 그림 행: 그림 두 장이어야 한다: {boxes:?}");
    };
    assert!(
        (y0 - (cell_top + PAD_TOP_PX)).abs() <= TOL_PX,
        "첫 그림 위 {y0:.2} 가 칸 위 {cell_top:.2} + 안 여백이어야 한다"
    );
    assert!(
        (y1 - (y0 + PIC_PX)).abs() <= TOL_PX,
        "둘째 그림 위 {y1:.2} 가 첫 그림 아래 끝 {:.2} 이어야 한다 (같은 높이면 겹쳐 그린 것)",
        y0 + PIC_PX
    );
    assert!(
        (x1 - (x0 + OVERLAP_X_PX)).abs() <= TOL_PX,
        "둘째 그림 x {x1:.2} 는 첫 그림 x {x0:.2} + 오프셋 {OVERLAP_X_PX:.2} 그대로여야 한다"
    );
}

#[test]
fn picture_boxes_use_physical_overlap_across_alignment_origins() {
    let original = first_page_photo_cells(&load());
    for name in [
        "photo-right-overlap",
        "photo-center-overlap",
        "photo-left-touch",
    ] {
        let path = format!("samples/side_by_side_photo_cell/{name}.hwp");
        let core = load_path(&path);
        assert_eq!(core.page_count(), 1);
        let cells = first_page_photo_cells(&core);
        let (_, boxes) = &cells[&OVERLAP_ROW];
        let [(x0, y0), (x1, y1)] = boxes.as_slice() else {
            panic!("{name}: 사진 두 장")
        };
        if name == "photo-left-touch" {
            assert!(
                x1 > x0 && (y1 - y0).abs() <= TOL_PX,
                "{name}: 경계에서 맞닿은 사진"
            );
        } else {
            let (_, expected) = &original[&OVERLAP_ROW];
            for (actual, reference) in boxes.iter().zip(expected) {
                assert!(
                    (actual.0 - reference.0).abs() <= TOL_PX
                        && (actual.1 - reference.1).abs() <= TOL_PX,
                    "{name}: 같은 실물 위치를 표현하는 정렬에서 사진 배치 보존"
                );
            }
            assert!(
                (y1 - y0 - PIC_PX).abs() <= TOL_PX,
                "{name}: 뒤 사진이 앞 사진 아래에 놓인다"
            );
        }
    }
}
