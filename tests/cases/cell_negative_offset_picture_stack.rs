//! 저장 줄 없는 칸 문단의 음수 세로 오프셋 자리차지 그림 무리는 겹치지 않고 줄지어 쌓인다.
//!
//! 사진 칸 실문서는 표 한 칸의 글 없는 문단(저장 줄 없음)에 문단 기준 자리차지 그림 여러 장을
//! 걸고, 각 그림의 세로 오프셋을 음수로 저장한다. 그 값은 앞 그림들에 밀려 내려간 줄 위치를
//! 기준으로 한 값이다. 웹한글기안기 인쇄본은 그림을 차례대로 놓는다. 위 줄 그림은 칸 내용
//! 위에서 시작하고, 아래 줄 그림은 위 그림 바로 아래에 붙으며, 묶음과 마지막 빈 줄을 칸 세로
//! 정렬로 놓는다. 묶음이 선언 칸보다 크면 칸이 늘어난다.
//!
//! rhwp 는 이 무리의 오프셋을 모두 버리고 같은 문단 원점에 겹쳐 그려, 위 줄 사진이 아래 줄
//! 사진에 덮여 보이지 않았다. 행 높이는 그림 높이를 모두 더해 칸이 쪽 밖으로 커졌다.
//!
//! 재현물은 공개 샘플(`samples/test-image.hwpx`, `samples/issue3738/…`)을 조합한 합성본이고,
//! 기준 인쇄본은 `pdf/cell_negative_offset_stack/<재현물>-webhwp-print.pdf` 에 있다.
//! 기대값은 픽셀 좌표가 아니라 관계(겹치지 않음, 앞 그림 아래 = 뒤 그림 위, 칸 안 포함,
//! 정렬 방향)로 적는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const GRID: &str = "samples/cell_negative_offset_stack/grid2x2.hwpx";
const MIXED: &str = "samples/cell_negative_offset_stack/row3-mixed.hwpx";
/// 재현물 그림 높이(px): grid2x2 40mm, row3-mixed 50mm.
const GRID_PIC_H: f64 = 40.0 / 25.4 * 96.0;
const MIXED_PIC_H: f64 = 50.0 / 25.4 * 96.0;
/// 재현물 칸 선언 높이(px): row3-mixed 95mm.
const MIXED_DECLARED_CELL_H: f64 = 95.0 / 25.4 * 96.0;
/// 칸 안 여백 141HU(px).
const CELL_PAD: f64 = 141.0 / 7200.0 * 96.0;
const TOL: f64 = 1.0;

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn bottom(self) -> f64 {
        self.y + self.h
    }
}

fn core(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 첫 쪽의 (그림을 담은 칸 상자, 그 칸 그림들을 control 순서로).
fn first_page_cell_and_pictures(core: &DocumentCore) -> (Rect, Vec<Rect>) {
    fn rect(n: &RenderNode) -> Rect {
        Rect {
            x: n.bbox.x,
            y: n.bbox.y,
            w: n.bbox.width,
            h: n.bbox.height,
        }
    }
    fn walk(n: &RenderNode, cells: &mut Vec<(Rect, Vec<(usize, Rect)>)>) {
        if matches!(n.node_type, RenderNodeType::TableCell(_)) {
            let mut pictures = Vec::new();
            collect_images(n, &mut pictures);
            cells.push((rect(n), pictures));
            return;
        }
        for c in &n.children {
            walk(c, cells);
        }
    }
    fn collect_images(n: &RenderNode, out: &mut Vec<(usize, Rect)>) {
        if let RenderNodeType::Image(img) = &n.node_type {
            out.push((img.control_index.unwrap_or(usize::MAX), rect(n)));
        }
        for c in &n.children {
            collect_images(c, out);
        }
    }
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut cells = Vec::new();
    walk(&tree.root, &mut cells);
    let (cell, mut pictures) = cells
        .into_iter()
        .find(|(_, pictures)| !pictures.is_empty())
        .expect("그림을 담은 칸");
    pictures.sort_by_key(|(control_idx, _)| *control_idx);
    (cell, pictures.into_iter().map(|(_, r)| r).collect())
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w - TOL && b.x < a.x + a.w - TOL && a.y < b.y + b.h - TOL && b.y < a.y + a.h - TOL
}

fn assert_no_overlap(pictures: &[Rect]) {
    for (i, a) in pictures.iter().enumerate() {
        for b in &pictures[i + 1..] {
            assert!(!overlaps(*a, *b), "그림이 겹친다: {a:?} / {b:?}");
        }
    }
}

fn assert_inside(cell: Rect, pictures: &[Rect]) {
    for p in pictures {
        assert!(
            p.y >= cell.y + CELL_PAD - TOL && p.bottom() <= cell.bottom() - CELL_PAD + TOL,
            "그림이 칸 밖으로 나간다: 칸 {cell:?}, 그림 {p:?}"
        );
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOL
}

/// 2열 2줄: 아래 줄 그림은 자기 오프셋(앞 그림과 30px 쯤 틈)이 아니라 위 그림 바로 아래에 붙고,
/// 묶음은 칸 가운데 정렬로 놓인다.
#[test]
fn grid_pictures_stack_in_two_rows_and_center_in_cell() {
    let core = core(GRID);
    let (cell, p) = first_page_cell_and_pictures(&core);
    assert_eq!(p.len(), 4, "칸 그림 4장: {p:?}");
    assert_no_overlap(&p);
    assert_inside(cell, &p);
    for r in &p {
        assert!(close(r.h, GRID_PIC_H), "그림 높이: {r:?}");
    }
    assert!(close(p[0].y, p[1].y), "위 줄 두 장은 같은 높이: {p:?}");
    assert!(
        close(p[2].y, p[0].bottom()),
        "아래 왼쪽은 위 왼쪽 바로 아래: {p:?}"
    );
    assert!(
        close(p[3].y, p[1].bottom()),
        "아래 오른쪽은 위 오른쪽 바로 아래: {p:?}"
    );
    let top_gap = p[0].y - (cell.y + CELL_PAD);
    let bottom_gap = cell.bottom() - CELL_PAD - p[3].bottom();
    assert!(
        top_gap > 20.0 && top_gap < bottom_gap,
        "그림 묶음과 그 아래 빈 줄이 칸 가운데에 놓인다: 위 틈 {top_gap:.1}, 아래 틈 {bottom_gap:.1}"
    );
    assert!(
        bottom_gap - top_gap < 30.0,
        "아래 틈과 위 틈의 차는 빈 줄 하나다: 위 틈 {top_gap:.1}, 아래 틈 {bottom_gap:.1}"
    );
}

/// 3열 2줄(자리차지 + 어울림): 위 줄은 칸 내용 위에서 시작하고, 아래 줄은 위 줄 바로 아래에
/// 붙는다. 묶음이 선언 칸보다 커서 칸이 늘어나되 표는 한 쪽 안에 남는다.
#[test]
fn mixed_wrap_pictures_stack_and_grow_the_cell() {
    let core = core(MIXED);
    assert_eq!(core.page_count(), 1, "표는 한 쪽 안에 남는다");
    let (cell, p) = first_page_cell_and_pictures(&core);
    assert_eq!(p.len(), 6, "칸 그림 6장: {p:?}");
    assert_no_overlap(&p);
    assert_inside(cell, &p);
    let content_top = cell.y + CELL_PAD;
    for r in &p[..3] {
        assert!(
            close(r.y, content_top),
            "위 줄은 칸 내용 위에서 시작: {r:?}, 칸 {cell:?}"
        );
    }
    for r in &p[3..] {
        assert!(
            close(r.y, content_top + MIXED_PIC_H),
            "아래 줄은 위 줄 바로 아래: {r:?}, 칸 {cell:?}"
        );
    }
    assert!(
        cell.h > MIXED_DECLARED_CELL_H + TOL,
        "그림 묶음이 선언 칸보다 커서 칸이 늘어난다: {cell:?}"
    );
    assert!(
        cell.h < 2.0 * MIXED_PIC_H + 2.0 * CELL_PAD + 30.0,
        "칸은 그림 두 줄과 빈 줄 하나만큼만 늘어난다(그림 높이 합이 아니다): {cell:?}"
    );
}
