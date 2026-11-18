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
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
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
