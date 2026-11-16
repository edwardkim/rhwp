//! 쪽 끝에 걸린 사진 행을 가를 때 사진 칸이 차지하는 높이.
//!
//! 저장 줄(LINE_SEG)이 없는 칸에서 글자 없는 문단 하나가 문단 기준 어울림(Square) 사진을
//! 앵커하면, 그 칸은 `max(자리 줄, 사진 바닥)` 높이로 그려진다(자리 줄은 사진 옆에 놓인다).
//! 선언 높이가 쪽에 남은 자리보다 큰 행은 온전한 행으로 들어가지 못해 칸 내용 단위로 갈린다.
//! 종전 행 나눔 장부는 같은 칸을 자리 줄(400HU) + 사진 높이로 쌓아, 그리는 내용(사진 + 안 여백)이
//! 남은 자리에 드는데도 사진을 다음 쪽으로 넘겼다.
//!
//! 실문서(저장 줄 없는 생성계 주간 계획표) 정답(웹한글기안기 쪽 그림)에서 이런 행은 사진까지
//! 그 쪽에 담겼다. 종전 rhwp 는 사진만 다음 쪽으로 넘겨 쪽 수가 2 → 3 이 됐다.
//!
//! 재현물 `samples/square_photo_rows/square-photo-row-split.hwp` 는 공개 샘플
//! (`samples/test-image.hwpx`, `samples/issue3738/tac_sibling_shape_line_advance.hwpx`)만 조합한
//! 합성본이다(저장 줄 없음, 원본 HWP5 경로). 자리차지 표 하나, 10행 x 2칸: 칸 0 은 "R{n}",
//! 칸 1 은 글자 없는 문단 하나에 어울림 사진(세로 오프셋 0, 겹침 허용 끔). 행 7 만 사진이
//! 더 크고(9490HU) 칸 선언이 사진 + 안 여백보다 1000HU 높으며, 칸 0 에 글 문단이 셋이다(실문서
//! 처럼 여러 문단인 칸이 있어 내용 단위로 갈리는 행). 웹한글기안기 인쇄본
//! (`pdf/square_photo_rows/square-photo-row-split-webhwp-print.pdf`)은 행 7 을 1쪽 끝까지
//! 사진째 담고(선언보다 낮게), 2쪽은 행 8·9 로 시작한다. 기대값은 규칙으로 적는다: 행 7 은 선언
//! 높이로는 1쪽에 남은 자리를 넘고, 사진 + 안 여백은 남은 자리에 든다. 그때 사진은 1쪽에
//! 그려진다. 절대 좌표는 핀하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/square_photo_rows/square-photo-row-split.hwp";
const SPLIT_ROW: u16 = 7;
const HU_PX: f64 = 96.0 / 7200.0;
/// 행 7 의 사진 높이 9490HU (px).
const PHOTO_PX: f64 = 9490.0 * HU_PX;
/// 안 여백 141HU x 2 (px).
const PADDING_PX: f64 = 2.0 * 141.0 * HU_PX;
/// 행 7 의 칸 선언 높이: 사진 + 안 여백 + 1000HU (px).
const DECLARED_PX: f64 = PHOTO_PX + PADDING_PX + 1000.0 * HU_PX;
/// 표 바깥 아래 여백 283HU (px).
const OUTER_BOTTOM_PX: f64 = 283.0 * HU_PX;

#[derive(Default)]
struct PageCells {
    body_bottom: f64,
    /// (행, 위, 아래)
    cells: Vec<(u16, f64, f64)>,
    /// 사진 칸 안 그림의 (행, 위, 아래)
    photos: Vec<(u16, f64, f64)>,
}

fn page_cells(core: &DocumentCore, page: u32) -> PageCells {
    fn walk(n: &RenderNode, row: Option<u16>, out: &mut PageCells) {
        let row = match &n.node_type {
            RenderNodeType::Body { .. } => {
                out.body_bottom = n.bbox.y + n.bbox.height;
                row
            }
            RenderNodeType::TableCell(cell) => {
                out.cells
                    .push((cell.row, n.bbox.y, n.bbox.y + n.bbox.height));
                Some(cell.row)
            }
            RenderNodeType::Image(_) => {
                if let Some(row) = row {
                    out.photos.push((row, n.bbox.y, n.bbox.y + n.bbox.height));
                }
                row
            }
            _ => row,
        };
        for c in &n.children {
            walk(c, row, out);
        }
    }
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut out = PageCells::default();
    walk(&tree.root, None, &mut out);
    out
}

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 재현물이 겨냥한 자리인지: 행 7 은 선언 높이로는 넘고, 그리는 내용은 든다.
#[test]
fn split_row_content_fits_the_room_that_its_declared_height_exceeds() {
    let core = load();
    let first = page_cells(&core, 0);
    let prior_bottom = first
        .cells
        .iter()
        .filter(|(row, _, _)| *row + 1 == SPLIT_ROW)
        .map(|(_, _, bottom)| *bottom)
        .fold(f64::MIN, f64::max);
    assert!(prior_bottom > 0.0, "1쪽에 행 6 이 있어야 한다");
    let room = first.body_bottom - OUTER_BOTTOM_PX - prior_bottom;
    assert!(
        room < DECLARED_PX,
        "행 7 의 선언 높이 {DECLARED_PX:.2} 가 남은 자리 {room:.2} 를 넘어야 한다"
    );
    assert!(
        PHOTO_PX + PADDING_PX <= room,
        "사진 + 안 여백 {:.2} 이 남은 자리 {room:.2} 에 들어야 한다",
        PHOTO_PX + PADDING_PX
    );
}

/// 그리는 내용이 남은 자리에 드는 사진은 그 쪽에 그려진다(자리 줄을 사진 위에 쌓지 않는다).
#[test]
fn split_row_keeps_the_photo_that_fits_on_the_first_page() {
    let core = load();
    let first = page_cells(&core, 0);
    let second = page_cells(&core, 1);
    let on_first: Vec<_> = first
        .photos
        .iter()
        .filter(|(row, _, _)| *row == SPLIT_ROW)
        .collect();
    let on_second: Vec<_> = second
        .photos
        .iter()
        .filter(|(row, _, _)| *row == SPLIT_ROW)
        .collect();
    assert_eq!(
        (on_first.len(), on_second.len()),
        (1, 0),
        "행 7 의 사진은 1쪽에 한 번 그려져야 한다: 1쪽 {on_first:?}, 2쪽 {on_second:?}"
    );
    let (_, _, photo_bottom) = on_first[0];
    assert!(
        *photo_bottom <= first.body_bottom + 0.5,
        "사진 바닥 {photo_bottom:.2} 이 본문 아래 {:.2} 안에 있어야 한다",
        first.body_bottom
    );
}
