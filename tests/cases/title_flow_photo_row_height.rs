//! 제목 문단 + 글자 없는 마지막 문단의 자리차지(TopAndBottom) 그림 칸의 행 높이.
//!
//! 저장 줄(LINE_SEG)이 없는 칸에서 글자 없는 문단은 측정에 400HU 자리 줄을 싣는다. 그 문단에
//! 자리차지 그림이 있으면 그 줄은 개체를 담는 자리라, 개체 높이와 함께 세면 같은 자리를 두 번
//! 센다. 종전 측정은 이 빼기를 문단이 하나뿐인 칸에만 했다. 그래서 제목 문단 뒤 마지막 빈
//! 문단에 사진을 둔 칸은 사진 아래에 자리 줄(5.33px)이 남아 행이 그만큼 커졌다.
//!
//! 재현물 `samples/title_flow_photo/title-flow-photo-row.hwp` 는 공개 샘플
//! (`samples/test-image.hwpx`, `samples/issue3738/tac_sibling_shape_line_advance.hwpx`)만
//! 조합한 합성본이다(저장 줄 없음, 원본 HWP5 경로). 사진 칸 = "제목" + 글자 없는 문단의 문단
//! 기준 자리차지 그림(세로 오프셋 0, 겹침 허용 끔 — 실문서 사진과 같음):
//! - 표 1(1쪽): 사진 행 하나, 칸 선언 높이 1000HU(내용보다 낮음 — 행은 측정으로 늘어난다)
//! - 표 2(1쪽): 사진 행 하나, 칸 선언 높이 = 안 여백 + 제목 줄 + 그림
//! - 표 3(2쪽부터): 표 2 와 같은 사진 행 8개 + 본문 행 3개. 쪽을 넘으므로 행 나눔 장부(칸 유닛)로
//!   쪽을 가른다. 첫 쪽(2쪽)에 사진 행 8개와 본문 행 1개가 들고 3쪽에는 본문 행 2개가 남는다.
//!   행마다 자리 줄을 더 세면 여덟째 사진 행이 3쪽으로 밀린다(실문서에서 마지막 행이 다음 쪽으로
//!   밀린 증상과 같은 경로).
//!
//! 그림 자리도 함께 본다. 저장 줄 없는 빈 문단이 앞 내용 뒤에 놓이면 그 문단 원점이 곧 그림
//! 자리다(제목 줄 흐름 바로 아래). 종전 배치는 이 그림을 칸 세로 가운데로 옮겨 제목 줄과 겹쳐
//! 그렸다 — 실문서 정답에서는 음수 세로 오프셋 그림까지 제목 줄 바로 아래에 놓인다.
//!
//! 기대값은 같은 형상의 실문서 정답(웹한글기안기 쪽 그림)에서 끌어낸 관계로 적는다. 사진 칸
//! 두 행(한/글 저장 칸 높이 7194HU)이 정답에서 95.3·96.0px 로, `안 여백 + 제목 줄 + 그림 +
//! 안 여백`(95.9px)과 같았다 — 곧 그림 바닥 아래에는 안 여백만 남는다. 재현물의 웹한글기안기
//! 인쇄본(`pdf/title_flow_photo/title-flow-photo-row-webhwp-print.pdf`)도 사진 행을 모두 이
//! 높이(105.0px)로 그리고 2쪽에 사진 행 8개 + 본문 행 1개, 3쪽에 본문 행 2개를 둔다. 절대 좌표는
//! 핀하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/title_flow_photo/title-flow-photo-row.hwp";
/// 재현물 칸 안 여백 141HU(px).
const PAD_PX: f64 = 141.0 * 96.0 / 7200.0;
/// 제목 줄의 흐름 높이: 10pt 글자 × 줄간격 160% = 1600HU(px).
const TITLE_FLOW_PX: f64 = 1600.0 * 96.0 / 7200.0;
/// 표 1 칸 선언 높이 1000HU(px).
const LOW_DECLARED_PX: f64 = 1000.0 * 96.0 / 7200.0;
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

/// 사진 칸 하나: 칸 상자, 글줄, 그림.
#[derive(Debug)]
struct PhotoCell {
    cell: Rect,
    lines: Vec<Rect>,
    photo: Rect,
}

/// 쪽별 사진 칸(표 순서).
fn photo_cells_by_page() -> Vec<Vec<PhotoCell>> {
    fn rect(n: &RenderNode) -> Rect {
        Rect {
            y: n.bbox.y,
            h: n.bbox.height,
        }
    }
    fn collect(n: &RenderNode, lines: &mut Vec<Rect>, images: &mut Vec<Rect>) {
        match &n.node_type {
            RenderNodeType::TextLine(_) => lines.push(rect(n)),
            RenderNodeType::Image(_) => images.push(rect(n)),
            _ => {}
        }
        for c in &n.children {
            collect(c, lines, images);
        }
    }
    fn walk(n: &RenderNode, out: &mut Vec<PhotoCell>) {
        if matches!(n.node_type, RenderNodeType::TableCell(_)) {
            let mut lines = Vec::new();
            let mut images = Vec::new();
            collect(n, &mut lines, &mut images);
            if let [photo] = images.as_slice() {
                lines.sort_by(|a, b| a.y.total_cmp(&b.y));
                out.push(PhotoCell {
                    cell: rect(n),
                    lines,
                    photo: *photo,
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
        "표 3 은 2쪽에서 시작해 3쪽으로 넘는다"
    );
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("render tree");
            let mut cells = Vec::new();
            walk(&tree.root, &mut cells);
            cells
        })
        .collect()
}

fn photo_cells() -> Vec<PhotoCell> {
    let mut pages = photo_cells_by_page();
    let cells = pages.swap_remove(0);
    assert_eq!(
        cells.len(),
        2,
        "1쪽: 그림을 하나 담은 칸 두 개(표 순서): {cells:?}"
    );
    cells
}

/// 칸 높이는 안 여백 + 제목 줄 + 그림이고(빈 문단 자리 줄을 더 세지 않음), 그림은 제목 줄 흐름 아래다.
fn assert_photo_closes_cell(name: &str, c: &PhotoCell) {
    assert!(
        !c.lines.is_empty(),
        "{name}: 제목 글줄이 있어야 한다: {c:?}"
    );
    let expected = PAD_PX * 2.0 + TITLE_FLOW_PX + c.photo.h;
    assert!(
        (c.cell.h - expected).abs() <= TOL_PX,
        "{name}: 칸 높이({:.2})가 안 여백 + 제목 줄 + 그림({expected:.2})이어야 한다: {c:?}",
        c.cell.h
    );
    let photo_top = c.cell.y + PAD_PX + TITLE_FLOW_PX;
    assert!(
        (c.photo.y - photo_top).abs() <= TOL_PX,
        "{name}: 그림 위({:.2})가 제목 줄 흐름 아래({photo_top:.2})여야 한다: {c:?}",
        c.photo.y
    );
}

/// 표 1: 선언이 낮은 칸은 측정으로 늘어나되, 그림 아래 자리 줄 없이 닫힌다.
#[test]
fn low_declared_title_photo_cell_grows_without_anchor_line() {
    let cells = photo_cells();
    let c = &cells[0];
    assert!(
        c.cell.h > LOW_DECLARED_PX + TOL_PX,
        "칸({:.2})이 선언 높이({LOW_DECLARED_PX:.2})보다 늘어나야 한다",
        c.cell.h
    );
    assert_photo_closes_cell("표 1 낮은 선언", c);
}

/// 표 2: 선언이 내용과 같은 칸은 선언 높이 그대로다(자리 줄만큼 늘지 않는다).
#[test]
fn exact_declared_title_photo_cell_keeps_declared_height() {
    assert_photo_closes_cell("표 2 내용과 같은 선언", &photo_cells()[1]);
}

/// 표 3: 쪽을 넘는 표의 행 나눔 장부도 자리 줄을 더 세지 않는다 — 사진 행 8개가 첫 쪽에 든다.
#[test]
fn split_table_keeps_all_photo_rows_on_first_page() {
    let pages = photo_cells_by_page();
    let first = &pages[1];
    assert_eq!(
        first.len(),
        8,
        "2쪽: 사진 행 8개가 모두 들어야 한다(행마다 자리 줄 5.33px 을 더 세면 7개): {first:?}"
    );
    for (i, c) in first.iter().enumerate() {
        assert_photo_closes_cell(&format!("표 3 사진 행 {i}"), c);
    }
    assert!(
        pages[2].is_empty(),
        "3쪽에는 본문 행 2개만 남는다(사진 칸 없음): {:?}",
        pages[2]
    );
}
