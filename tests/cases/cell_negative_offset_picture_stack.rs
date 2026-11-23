//! 관계형·고정형 회귀 검사. 표 칸의 그림 순서·겹침·칸 높이·뒤 문단 위치와 독립 인쇄본의 한 쪽을 검사한다.
//! 주석 갱신 2026-11-23T16:33:20+09:00. 직전 파일 커밋 a864c91331bf18ab0980f79d1e17bf6e1a45b7a5 (2026-11-18 16:03:15 KST).
//!
//! 공개 HWPX와 그림 하나의 속성을 바꾼 각 대조군을 Windows 한컴 2024에서 직접 인쇄했다.
//! 2026-11-23, 96 DPI 전쪽, 2px 이웃 raw 실루엣. 아래는 각각 1쪽의 Native / fresh WASM Canvas 결과다.
//! grid2x2, row3-mixed, grid-right — 100.00% / 100.00%.
//! grid-right-same, grid-overlap-one, grid-center-in — 100.00% / 100.00%.
//! grid-center-out, grid-margin, grid-positive — 100.00% / 100.00%.
//! grid-center-aligned, grid-overlap-first, grid-outside — 100.00% / 100.00%.
//! 전 12쪽 최저 Native 100.00%, WASM 100.00%.
//! 입력·독립 Print의 SHA-256은 mydocs/pr/assets/pr9339/print-inputs.tsv,
//! 쪽별 실측과 출력 해시는 같은 폴더의 page-scores.tsv에 있다.
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
    fn right(self) -> f64 {
        self.x + self.w
    }

    fn bottom(self) -> f64 {
        self.y + self.h
    }
}

fn core(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

struct PicturePage {
    cell: Rect,
    pictures: Vec<Rect>,
    following_text: Rect,
}

/// 공개 표본에는 그림 칸 하나와 표 뒤에 오는 END 문단 하나가 있다.
fn picture_page(path: &str) -> PicturePage {
    fn rect(n: &RenderNode) -> Rect {
        Rect {
            x: n.bbox.x,
            y: n.bbox.y,
            w: n.bbox.width,
            h: n.bbox.height,
        }
    }
    fn walk(n: &RenderNode, cells: &mut Vec<(Rect, Vec<(usize, Rect)>)>, ends: &mut Vec<Rect>) {
        if matches!(n.node_type, RenderNodeType::TableCell(_)) {
            let mut pictures = Vec::new();
            collect_images(n, &mut pictures);
            if !pictures.is_empty() {
                cells.push((rect(n), pictures));
            }
            return;
        }
        if matches!(&n.node_type, RenderNodeType::TextRun(run) if run.text == "END") {
            ends.push(rect(n));
        }
        for c in &n.children {
            walk(c, cells, ends);
        }
    }
    fn collect_images(n: &RenderNode, out: &mut Vec<(usize, Rect)>) {
        if let RenderNodeType::Image(img) = &n.node_type {
            out.push((
                img.control_index.expect("원본 그림의 control index"),
                rect(n),
            ));
        }
        for c in &n.children {
            collect_images(c, out);
        }
    }
    let core = core(path);
    assert_eq!(
        core.page_count(),
        1,
        "{path}: 표와 뒤 문단은 같은 쪽에 남는다"
    );
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut cells = Vec::new();
    let mut ends = Vec::new();
    walk(&tree.root, &mut cells, &mut ends);
    let [(cell, mut pictures)]: [_; 1] = cells.try_into().expect("그림을 담은 칸 한 개");
    let [following_text]: [Rect; 1] = ends.try_into().expect("표 밖에 END 한 개");
    assert!(
        following_text.y >= cell.bottom() - TOL,
        "{path}: {cell:?}, {following_text:?}"
    );
    pictures.sort_by_key(|(control_idx, _)| *control_idx);
    assert!(
        pictures
            .iter()
            .map(|(index, _)| *index)
            .eq(0..pictures.len()),
        "{path}: 그림 누락 또는 중복"
    );
    PicturePage {
        cell,
        pictures: pictures.into_iter().map(|(_, r)| r).collect(),
        following_text,
    }
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
    let PicturePage {
        cell,
        pictures: p,
        following_text,
    } = picture_page(GRID);
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
    // 빈 그림 문단과 뒤 END 문단은 원본의 동일한 문단·글자 모양 0번을 쓴다.
    let top_gap = p[0].y - (cell.y + CELL_PAD);
    let bottom_gap = cell.bottom() - CELL_PAD - p[3].bottom();
    assert!(
        top_gap > 0.0 && top_gap < bottom_gap,
        "그림 묶음과 그 아래 빈 줄이 칸 가운데에 놓인다: 위 틈 {top_gap:.1}, 아래 틈 {bottom_gap:.1}"
    );
    assert!(
        close(bottom_gap - top_gap, following_text.h),
        "아래 틈과 위 틈의 차는 빈 줄 하나다: 위 틈 {top_gap:.1}, 아래 틈 {bottom_gap:.1}"
    );
}

/// 3열 2줄(자리차지 + 어울림): 위 줄은 칸 내용 위에서 시작하고, 아래 줄은 위 줄 바로 아래에
/// 붙는다. 묶음이 선언 칸보다 커서 칸이 늘어나되 표는 한 쪽 안에 남는다.
#[test]
fn mixed_wrap_pictures_stack_and_grow_the_cell() {
    let PicturePage {
        cell,
        pictures: p,
        following_text,
    } = picture_page(MIXED);
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
        close(
            cell.h - 2.0 * MIXED_PIC_H - 2.0 * CELL_PAD,
            following_text.h
        ),
        "칸은 그림 두 줄과 빈 줄 하나만큼만 늘어난다(그림 높이 합이 아니다): {cell:?}"
    );
}

fn grid_variant(name: &str) -> PicturePage {
    let page = picture_page(&format!(
        "samples/cell_negative_offset_stack/pr9339-review/{name}.hwpx"
    ));
    assert_eq!(
        page.pictures.len(),
        4,
        "{name}: 네 그림 모두 원래 칸에 속한다"
    );
    page
}

fn assert_same_rect(actual: Rect, expected: Rect) {
    assert!(
        close(actual.x, expected.x)
            && close(actual.y, expected.y)
            && close(actual.w, expected.w)
            && close(actual.h, expected.h),
        "배치가 달라졌다: {actual:?} / {expected:?}"
    );
}

#[test]
fn equivalent_horizontal_anchors_preserve_the_cell_pictures_and_following_paragraph() {
    let original = picture_page(GRID);
    for name in ["grid-center-aligned", "grid-right-same"] {
        let variant = grid_variant(name);
        assert_same_rect(variant.cell, original.cell);
        assert_same_rect(variant.following_text, original.following_text);
        for (&a, &b) in variant.pictures.iter().zip(&original.pictures) {
            assert_same_rect(a, b);
        }
    }
}

#[test]
fn crossing_the_cell_edge_keeps_the_other_three_pictures_in_their_rows() {
    let inside = grid_variant("grid-center-in");
    let outside = grid_variant("grid-center-out");
    let detached = grid_variant("grid-outside");
    for page in [&outside, &detached] {
        assert_same_rect(inside.cell, page.cell);
        assert_same_rect(inside.following_text, page.following_text);
        for i in 0..3 {
            assert_same_rect(inside.pictures[i], page.pictures[i]);
        }
    }
    let a = inside.pictures[3];
    let b = outside.pictures[3];
    assert!(a.x + a.w / 2.0 < inside.cell.right());
    assert!(b.x + b.w / 2.0 > outside.cell.right());
    assert!(b.x < outside.cell.right() && detached.pictures[3].x >= detached.cell.right());
    assert!(close(a.y, inside.pictures[1].bottom()) && close(b.y, outside.pictures[1].bottom()));
    assert!(b.x > a.x && close(a.w, b.w) && close(a.h, b.h));
}

#[test]
fn overlap_permission_on_either_picture_only_affects_its_intersecting_column() {
    let original = picture_page(GRID);
    for (name, overlapping, stacked) in [
        ("grid-overlap-first", [0, 2], [1, 3]),
        ("grid-overlap-one", [1, 3], [0, 2]),
    ] {
        let variant = grid_variant(name);
        let p = &variant.pictures;
        assert_same_rect(variant.cell, original.cell);
        assert_same_rect(variant.following_text, original.following_text);
        assert!(overlaps(p[overlapping[0]], p[overlapping[1]]));
        assert!(close(p[overlapping[0]].y, p[overlapping[1]].y));
        for i in stacked {
            assert_same_rect(p[i], original.pictures[i]);
        }
        assert!(!overlaps(p[stacked[0]], p[stacked[1]]));
        assert!(close(p[stacked[1]].y, p[stacked[0]].bottom()));
    }
}
