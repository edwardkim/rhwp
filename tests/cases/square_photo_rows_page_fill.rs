//! 쪽을 넘는 사진 표에서 1쪽에 담는 행 수.
//!
//! 저장 줄(LINE_SEG)이 없는 칸에서 글자 없는 문단 하나가 문단 기준 어울림(Square) 사진을
//! 앵커하면, 그 칸은 `max(글줄, 사진 바닥)` 높이로 그려진다(사진 옆 자리 줄은 칸을 늘리지
//! 않는다). 종전 쪽 나눔 판정은 같은 칸을 자리 줄(400HU) + 사진 높이로 쌓아 행마다 약 5.3px
//! 를 더 예약했다. 그래서 그린 높이로는 1쪽에 들어가는 마지막 사진 행을 다음 쪽으로 넘겼다.
//!
//! 실문서(저장 줄 없는 생성계 주간 계획표) 정답(웹한글기안기 쪽 그림)에서 사진 행은 사진 높이 +
//! 안 여백으로 그려지고, 1쪽은 그 높이로 쪽 끝까지 행을 담았다. 종전 rhwp 는 쌓은 예약 때문에
//! 마지막 사진 행을 넘겨 쪽 수가 2 → 3 이 됐다(세 건).
//!
//! 재현물 `samples/square_photo_rows/square-photo-rows.hwp` 는 공개 샘플
//! (`samples/test-image.hwpx`, `samples/issue3738/tac_sibling_shape_line_advance.hwpx`)만 조합한
//! 합성본이다(저장 줄 없음, 원본 HWP5 경로). 자리차지 표 하나, 12행 x 2칸: 칸 0 은 "R{n}",
//! 칸 1 은 글자 없는 문단 하나에 어울림 사진(세로 오프셋 0, 7800HU, 겹침 허용 끔). 칸 선언 높이는
//! 사진 높이 + 안 여백 위아래다. 웹한글기안기 인쇄본
//! (`pdf/square_photo_rows/square-photo-rows-webhwp-print.pdf`)은 1쪽에 사진 행 8개, 2쪽에 4개를
//! 그린 높이(사진 + 안 여백)로 담는다. 기대값은 규칙으로 적는다: 행은 사진 높이 + 안 여백으로
//! 그려지고, 1쪽에는 다음 행 하나를 더 담을 자리가 남지 않는다. 절대 좌표는 핀하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/square_photo_rows/square-photo-rows.hwp";
const ROWS: usize = 12;
/// 사진 높이 7800HU + 안 여백 141HU x 2 (px).
const ROW_PX: f64 = (7800.0 + 2.0 * 141.0) * 96.0 / 7200.0;
/// 표 바깥 아래 여백 283HU (px).
const OUTER_BOTTOM_PX: f64 = 283.0 * 96.0 / 7200.0;
const TOL_PX: f64 = 0.5;

#[derive(Debug, Clone, Copy)]
struct Band {
    top: f64,
    bottom: f64,
}

struct PageRows {
    body_bottom: f64,
    rows: BTreeMap<u16, Band>,
}

fn page_rows(core: &DocumentCore, page: u32) -> PageRows {
    fn walk(n: &RenderNode, body_bottom: &mut f64, rows: &mut BTreeMap<u16, Band>) {
        match &n.node_type {
            RenderNodeType::Body { .. } => *body_bottom = n.bbox.y + n.bbox.height,
            RenderNodeType::TableCell(cell) => {
                let band = rows.entry(cell.row).or_insert(Band {
                    top: f64::MAX,
                    bottom: f64::MIN,
                });
                band.top = band.top.min(n.bbox.y);
                band.bottom = band.bottom.max(n.bbox.y + n.bbox.height);
                return;
            }
            _ => {}
        }
        for c in &n.children {
            walk(c, body_bottom, rows);
        }
    }
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut body_bottom = 0.0;
    let mut rows = BTreeMap::new();
    walk(&tree.root, &mut body_bottom, &mut rows);
    PageRows { body_bottom, rows }
}

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 사진 행은 사진 높이 + 안 여백으로 그려진다(자리 줄을 쌓지 않는다).
#[test]
fn square_photo_rows_paint_picture_plus_padding() {
    let core = load();
    let first = page_rows(&core, 0);
    let painted: Vec<(u16, f64)> = first
        .rows
        .iter()
        .take(first.rows.len().saturating_sub(1))
        .map(|(row, band)| (*row, band.bottom - band.top))
        .collect();
    assert!(!painted.is_empty(), "1쪽에 온전한 사진 행이 있어야 한다");
    for (row, height) in painted {
        assert!(
            (height - ROW_PX).abs() <= TOL_PX,
            "행 {row}: 그린 높이 {height:.2} 가 사진 + 안 여백 {ROW_PX:.2} 이어야 한다"
        );
    }
}

/// 1쪽은 그린 행 높이로 쪽 끝까지 행을 담는다: 다음 행 하나를 더 담을 자리가 남지 않는다.
#[test]
fn first_page_takes_every_photo_row_that_fits_by_painted_height() {
    let core = load();
    assert_eq!(core.page_count(), 2, "12행 사진 표는 두 쪽에 걸친다");
    let first = page_rows(&core, 0);
    let second = page_rows(&core, 1);
    assert_eq!(
        first.rows.len() + second.rows.len(),
        ROWS,
        "행이 두 쪽에 겹치지 않고 나뉘어야 한다: 1쪽 {:?}, 2쪽 {:?}",
        first.rows.keys().collect::<Vec<_>>(),
        second.rows.keys().collect::<Vec<_>>()
    );
    let table_top = first.rows.values().next().expect("1쪽 첫 행").top;
    let taken = first.rows.len();
    let room = first.body_bottom - OUTER_BOTTOM_PX - table_top;
    assert!(
        (taken + 1) as f64 * ROW_PX > room + TOL_PX,
        "1쪽에 {taken}행을 담았지만 그린 높이로 {}행이 들어간다(표 위 {table_top:.2}, 행 {ROW_PX:.2}, \
         본문 아래 {:.2}) — 쌓은 자리 줄 몫을 예약하면 들어가는 행이 다음 쪽으로 밀린다",
        taken + 1,
        first.body_bottom
    );
}
