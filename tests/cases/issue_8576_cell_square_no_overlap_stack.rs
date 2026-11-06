//! 관계형·고정형 회귀: 그림/칸의 포함·겹침·정렬·후속 본문과 독립 Print 쪽수.
//! 주석 갱신: 2026-11-06T18:23:51+09:00 (Asia/Seoul).
//! 직전 파일 변경: 78b0c6fab987aa53071005045d5f2e74e2f18ecc,
//! 2026-11-06T10:23:43+09:00 (Git commit 시각; 아래 실측 시각과 구분).
//! 실측: 2026-11-06 18:22 KST, 96dpi, 색상 경계 보정 전 2px raw 실루엣.
//! 작은 그림/큰 그림/CENTER/선언 높이 경계/겹침 허용/가로 비충돌은 각 1쪽,
//! split-row HWP와 남은 공간 경계 HWPX는 각각 2쪽이다.
//! 원본별 Native/fresh optimized WASM의 모든 해당 쪽은 각각 100.00%이며,
//! 8입력 전체 10쪽의 두 경로 최저값도 각각 100.00%다. 원문/PDF 쪽수 일치와
//! 괘선·그림 묶음·분할 전후 글줄·END의 동일 영역 직접 판독을 완료했다.
//! 근거: mydocs/pr/assets/pr8658/final/corrected-native/primary/*.tsv 및
//! mydocs/pr/assets/pr8658/final/corrected-wasm/primary/*.tsv, 같은 key의 compare/
//! overlay/review PNG, mydocs/pr/assets/pr8658/correction-consumers.md.
//! 기준: pdf/issue8576/*-2024.pdf 및 split-row-picture-only-para-2020.pdf.
//! 기대값은 원본 그림/안 여백과 독립 Print의 소유·순서·포함·겹침 관계다.
//! 절대 px 좌표·SVG 해시를 핀하지 않는다. 쪽수는 내용 보존과 함께 검사한다.
//!
//! [#8576] 칸 한 문단의 '겹침 허용 안 함' 어울림 그림은 서로 겹치지 않게 아래로 쌓인다.
//!
//! 표 칸의 글자 없는 문단에 글자처럼 취급하지 않는 Square 그림 6장이 모두 같은 자리
//! (세로·가로 오프셋 0)에 저장되어 있고 `allowOverlap=0` 이다. 웹한글기안기 인쇄본은
//! 뒤 그림을 앞 그림 바로 아래에 놓아 그림 높이 간격으로 쌓고, 쌓인 범위가 칸보다
//! 크면 칸을 늘린다(60×30mm 판은 선언 150mm 칸이 쌓인 180mm 를 담도록 늘어나고, 표는 한 쪽 안에 남는다).
//! rhwp 는 6장을 한 자리에 겹쳐 그렸다.
//!
//! 재현물은 공개 샘플(`samples/test-image.hwpx`, `samples/issue3738/…`)을 조합한
//! 합성본이고, 기준 인쇄본은 `pdf/issue8576/<재현물>-webhwp-print.pdf` 에 있다.
//! 기대값은 픽셀 좌표가 아니라 관계(겹치지 않음, 앞 그림 아래 = 뒤 그림 위, 칸이
//! 쌓인 범위를 담음)로 적는다.
//!
//! 같은 수정은 쪽을 넘어 나뉘는 행도 다룬다. 칸 안에 글자 없이 그림만 든 문단(HWP5,
//! 저장 줄 없음)은 빈 줄을 가진 조각이 없어 쪽 나눔 표 경로가 어느 조각에서도 그리지
//! 않았다. 웹한글기안기 인쇄본(`pdf/issue8576/split-row-picture-only-para-webhwp-print.pdf`)은
//! 1쪽에서 시작하는 그 행의 칸 위에 그림 2장을 가로로 나란히 그린다. 재현물
//! `split-row-picture-only-para.hwp` 도 공개 샘플을 조합한 합성본이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SMALL: &str = "samples/issue8576/cell-square-six-pictures-35x25.hwpx";
const SPLIT_ROW: &str = "samples/issue8576/split-row-picture-only-para.hwp";
const LARGE: &str = "samples/issue8576/cell-square-six-pictures-60x30-grows.hwpx";
/// 재현물 칸의 선언 높이 150mm(px).
const LARGE_DECLARED_CELL_H_PX: f64 = 150.0 / 25.4 * 96.0;
const TOL_PX: f64 = 0.5;

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn core(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 첫 쪽의 (칸 상자, 그 칸 안 그림들을 control 순서로) 를 모은다.
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
    a.x < b.x + b.w - TOL_PX
        && b.x < a.x + a.w - TOL_PX
        && a.y < b.y + b.h - TOL_PX
        && b.y < a.y + a.h - TOL_PX
}

/// 6장이 서로 겹치지 않고, 뒤 그림 위가 바로 앞 그림 아래에 붙는다.
fn assert_stacked(pictures: &[Rect]) {
    assert_eq!(pictures.len(), 6, "칸 그림 6장: {pictures:?}");
    for (i, a) in pictures.iter().enumerate() {
        for b in &pictures[i + 1..] {
            assert!(!overlaps(*a, *b), "그림이 겹친다: {a:?} / {b:?}");
        }
    }
    for pair in pictures.windows(2) {
        let (prev, next) = (pair[0], pair[1]);
        assert!(
            (next.y - (prev.y + prev.h)).abs() <= TOL_PX,
            "뒤 그림 위({:.2})가 앞 그림 아래({:.2})에 붙어야 한다",
            next.y,
            prev.y + prev.h
        );
        assert!(
            (next.x - prev.x).abs() <= TOL_PX,
            "가로 자리는 그대로다: {prev:?} / {next:?}"
        );
    }
}

/// 35×25mm 6장: 칸 안에서 그림 높이 간격으로 쌓인다.
#[test]
fn small_pictures_stack_without_overlap() {
    let core = core(SMALL);
    let (_, pictures) = first_page_cell_and_pictures(&core);
    assert_stacked(&pictures);
    assert_end_after_table(&core);
}

/// 60×30mm 6장: 쌓인 범위가 선언 칸보다 커서 칸이 늘어난다.
#[test]
fn large_pictures_stack_and_grow_cell() {
    let core = core(LARGE);
    let (cell, pictures) = first_page_cell_and_pictures(&core);
    assert_stacked(&pictures);
    let stack_top = pictures.first().unwrap().y;
    let stack_bottom = pictures.last().unwrap().y + pictures.last().unwrap().h;
    assert!(
        cell.h + TOL_PX >= stack_bottom - stack_top,
        "칸 높이({:.2})가 쌓인 범위({:.2})를 담아야 한다",
        cell.h,
        stack_bottom - stack_top
    );
    assert!(
        cell.y + cell.h + TOL_PX >= stack_bottom,
        "마지막 그림 아래({stack_bottom:.2})가 칸 아래({:.2}) 안에 있어야 한다",
        cell.y + cell.h
    );
    assert!(
        stack_bottom - stack_top > LARGE_DECLARED_CELL_H_PX + TOL_PX,
        "쌓인 범위({:.2})가 선언 칸 높이({LARGE_DECLARED_CELL_H_PX:.2})보다 커야 재현물이 성립한다",
        stack_bottom - stack_top
    );
    assert!(
        cell.h > LARGE_DECLARED_CELL_H_PX + TOL_PX,
        "칸이 선언 높이({LARGE_DECLARED_CELL_H_PX:.2})보다 늘어나야 한다: {:.2}",
        cell.h
    );
    assert_eq!(
        core.page_count(),
        1,
        "기준 인쇄본처럼 표와 뒤 문단이 한 쪽에 있다"
    );
    assert_end_after_table(&core);
}

/// 모든 쪽의 (칸 상자, 그 칸 안 그림들) 을 모은다. 그림 없는 칸은 뺀다.
fn all_pages_cells_with_pictures(core: &DocumentCore) -> Vec<(u32, Rect, Vec<Rect>)> {
    fn rect(n: &RenderNode) -> Rect {
        Rect {
            x: n.bbox.x,
            y: n.bbox.y,
            w: n.bbox.width,
            h: n.bbox.height,
        }
    }
    fn collect_images(n: &RenderNode, out: &mut Vec<Rect>) {
        if matches!(n.node_type, RenderNodeType::Image(_)) {
            out.push(rect(n));
        }
        for c in &n.children {
            collect_images(c, out);
        }
    }
    fn walk(page: u32, n: &RenderNode, out: &mut Vec<(u32, Rect, Vec<Rect>)>) {
        if matches!(n.node_type, RenderNodeType::TableCell(_)) {
            let mut pictures = Vec::new();
            collect_images(n, &mut pictures);
            if !pictures.is_empty() {
                out.push((page, rect(n), pictures));
            }
            return;
        }
        for c in &n.children {
            walk(page, c, out);
        }
    }
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render tree");
        walk(page, &tree.root, &mut out);
    }
    out
}

/// 쪽을 넘어 나뉘는 행의 그림만 든 문단: 그림 2장이 한 조각의 칸 안에 가로로 나란히 그려진다.
#[test]
fn split_row_picture_only_paragraph_draws_pictures_side_by_side_inside_cell() {
    let core = core(SPLIT_ROW);
    assert_eq!(core.page_count(), 2, "기준 인쇄본처럼 행이 2쪽으로 나뉜다");
    let mut lines = Vec::new();
    for page in 0..core.page_count() {
        text_lines(&core.build_page_render_tree(page).unwrap().root, &mut lines);
    }
    let expected: Vec<String> = (1..=6)
        .map(|n| format!("ROW {n}"))
        .chain((1..=40).map(|n| format!("LINE {n}")))
        .chain(std::iter::once("END".to_owned()))
        .collect();
    assert_eq!(
        lines, expected,
        "전체 행/글줄/뒤 문단의 순서·누락·중복 보존"
    );
    assert_end_after_table(&core);
    let cells = all_pages_cells_with_pictures(&core);
    let total: usize = cells.iter().map(|(_, _, pictures)| pictures.len()).sum();
    assert_eq!(
        total, 2,
        "나뉜 행의 그림 2장이 어느 조각에든 그려져야 한다: {cells:?}"
    );
    assert_eq!(
        cells.len(),
        1,
        "두 그림은 같은 조각의 같은 칸에 있다: {cells:?}"
    );
    let (_, cell, pictures) = &cells[0];
    for picture in pictures {
        assert!(
            picture.x + TOL_PX >= cell.x
                && picture.x + picture.w <= cell.x + cell.w + TOL_PX
                && picture.y + TOL_PX >= cell.y
                && picture.y + picture.h <= cell.y + cell.h + TOL_PX,
            "그림이 칸 안에 있어야 한다: {picture:?} / 칸 {cell:?}"
        );
    }
    let mut pictures = pictures.clone();
    pictures.sort_by(|a, b| a.x.total_cmp(&b.x));
    let (left, right) = (pictures[0], pictures[1]);
    assert!(
        (left.y - right.y).abs() <= TOL_PX,
        "같은 세로 오프셋의 두 그림은 같은 높이에서 시작한다: {left:?} / {right:?}"
    );
    assert!(
        right.x >= left.x + left.w - TOL_PX,
        "가로 오프셋대로 나란히 놓여 겹치지 않는다: {left:?} / {right:?}"
    );
}

fn text_lines(node: &RenderNode, out: &mut Vec<String>) {
    fn line_text(node: &RenderNode, text: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            text.push_str(&run.text);
        }
        for child in &node.children {
            line_text(child, text);
        }
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        line_text(node, &mut text);
        if !text.trim().is_empty() {
            out.push(text.trim().to_owned());
        }
        return;
    }
    for child in &node.children {
        text_lines(child, out);
    }
}

fn assert_end_after_table(core: &DocumentCore) {
    fn walk(node: &RenderNode, tables: &mut Vec<f64>, end: &mut Vec<f64>) {
        match &node.node_type {
            RenderNodeType::Table(_) => tables.push(node.bbox.y + node.bbox.height),
            RenderNodeType::TextRun(run) if run.text.trim() == "END" => end.push(node.bbox.y),
            _ => {}
        }
        for child in &node.children {
            walk(child, tables, end);
        }
    }
    let tree = core
        .build_page_render_tree(core.page_count() - 1)
        .expect("마지막 쪽");
    let (mut tables, mut end) = (Vec::new(), Vec::new());
    walk(&tree.root, &mut tables, &mut end);
    assert_eq!(end.len(), 1, "뒤 문단 END가 마지막 쪽에 한 번 있다");
    let table_bottom = tables.into_iter().reduce(f64::max).expect("마지막 표 조각");
    assert!(
        end[0] + TOL_PX >= table_bottom,
        "END는 마지막 괘선 아래에 있다"
    );
}

#[test]
fn exact_declared_height_preserves_picture_occupancy_and_padding() {
    use rhwp::model::control::Control;
    let core = core("tests/fixtures/pr8658/cell-square-six-pictures-exact-fit.hwpx");
    let (cell, pictures) = first_page_cell_and_pictures(&core);
    assert_stacked(&pictures);
    let table = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .expect("원본 표");
    let padding = table.cells[0].effective_padding(&table.padding);
    let top = f64::from(padding.top) / 75.0;
    let bottom = f64::from(padding.bottom) / 75.0;
    let first = pictures.first().unwrap();
    let last = pictures.last().unwrap();
    assert!(first.y + TOL_PX >= cell.y + top, "위 안 여백 보존");
    assert!(
        last.y + last.h + bottom <= cell.y + cell.h + TOL_PX,
        "실제 그림 점유와 아래 안 여백이 칸 안에 있다: {cell:?}, {last:?}"
    );
    assert!(
        cell.h > f64::from(table.cells[0].height) / 75.0 + TOL_PX,
        "그림 높이 합과 같은 선언은 여백을 없애는 상한이 아니다"
    );
    assert_eq!(core.page_count(), 1, "독립 Print의 전체 쪽수와 같다");
    assert_end_after_table(&core);
}

#[test]
fn centered_stack_uses_one_content_band() {
    let core = core("tests/fixtures/pr8658/cell-square-six-pictures-center.hwpx");
    let (cell, pictures) = first_page_cell_and_pictures(&core);
    assert_stacked(&pictures);
    let top = pictures.first().unwrap().y - cell.y;
    let last = pictures.last().unwrap();
    let bottom = cell.y + cell.h - last.y - last.h;
    assert!(
        (top - bottom).abs() <= TOL_PX,
        "묶음 전체의 위아래 여유가 같다"
    );
    assert_end_after_table(&core);
}

#[test]
fn overlap_permission_keeps_coincident_picture_origins() {
    let core = core("tests/fixtures/pr8658/cell-square-six-pictures-overlap-allowed.hwpx");
    let (_, pictures) = first_page_cell_and_pictures(&core);
    assert_eq!(pictures.len(), 6);
    for pair in pictures.windows(2) {
        assert!((pair[0].x - pair[1].x).abs() <= TOL_PX);
        assert!((pair[0].y - pair[1].y).abs() <= TOL_PX);
    }
    assert_end_after_table(&core);
}

#[test]
fn horizontally_disjoint_pictures_keep_their_shared_top() {
    let core = core("tests/fixtures/pr8658/cell-square-six-pictures-side-by-side.hwpx");
    let (_, pictures) = first_page_cell_and_pictures(&core);
    assert_eq!(pictures.len(), 3);
    for pair in pictures.windows(2) {
        assert!(!overlaps(pair[0], pair[1]));
        assert!((pair[0].y - pair[1].y).abs() <= TOL_PX);
        assert!(pair[1].x >= pair[0].x + pair[0].w - TOL_PX);
    }
    assert_end_after_table(&core);
}

#[test]
fn grown_stack_moves_with_its_cell_when_only_declared_height_fits() {
    let core = core("tests/fixtures/pr8658/cell-square-stack-after-leading-lines.hwpx");
    assert_eq!(
        core.page_count(),
        2,
        "독립 Print는 앞 문단과 성장한 표를 서로 다른 쪽에 둔다"
    );
    let cells = all_pages_cells_with_pictures(&core);
    assert_eq!(cells.len(), 1, "그림 묶음의 칸 소유가 한 번이다");
    let (page, cell, pictures) = &cells[0];
    assert_eq!(*page, 1, "늘어난 칸이 들어가는 다음 쪽이 그림을 소유한다");
    assert_stacked(pictures);
    let last = pictures.last().unwrap();
    assert!(last.y + last.h <= cell.y + cell.h + TOL_PX);
    let mut first = Vec::new();
    text_lines(&core.build_page_render_tree(0).unwrap().root, &mut first);
    assert_eq!(
        first,
        ["BEFORE"],
        "앞 문단을 보존하고 그림·뒤 문단을 앞 쪽에 중복하지 않는다"
    );
    assert_end_after_table(&core);
}
