//! 관계형·고정형 검사. 쪽별 개체의 소속과 칸의 포함 관계, 표 괘선과 도형의 그리기 순서를 검사한다.
//! 2026-11-25 21:12 KST 갱신. 직전 변경은 2026-11-23 16:19:02 +0900, 47bcc944e8.
//! 96 DPI, 독립 한컴 PDF 대비 2px raw. 바깥 괘선과 겹치는 square/front/behind HWPX는
//! 각 2쪽 전부 Native·fresh WASM 100.00%다. 한컴 직접 인쇄와 측정 TSV는
//! `pdf/pr9349-cell-floats/`, `mydocs/pr/assets/hyunmyungjanelee-20261125/maintainer/`에 둔다.
//! 영문 글꼴 표가 비어 있는 분할 HWP의 Native 최저 95.98%·WASM 96.56%는 별도 검토 중이다.
//!
//! 표 칸 안 문단에 매였지만 세로 기준이 종이인 떠 있는 사각형(어울림·글 앞으로)은 칸의 흐름에
//! 들지 않는다. 사각형은 종이 좌표에 그려지고, 칸 높이는 저장 높이 그대로다.
//!
//! 재현물 `samples/pk40991_cell_paper_shape/*.{hwpx,hwp}`: 공개 샘플
//! `samples/issue3738/tac_sibling_shape_line_advance.hwpx` 의 머리·구역 정의와 사각형 XML 을
//! 조합한 합성본이다. 칸 문단에는 저장 줄이 없다. HWP 는 같은 HWPX 를 rhwp 로 변환한 것이다.
//!
//! - `paper_square`: 1열 표. 둘째 행 칸(저장 높이 7500 HU, 100px)의 둘째 문단에 종이 기준
//!   어울림 사각형(세로 오프셋 40000 HU, 높이 12000 HU). 뒤로 행 32개.
//! - `paper_front`: 같은 표, 사각형만 글 앞으로.
//! - `paper_square_split`: 둘째 행 칸에 문단 70개(칸이 쪽을 넘어 나뉜다). 60번째 문단에 종이
//!   기준 어울림 사각형(세로 오프셋 20000 HU, 가로 오프셋 36000 HU, 글줄과 겹치지 않는 자리). 뒤로
//!   행 4개.
//! - `paper_square_nested`: `paper_square` 의 사각형 글상자 문단에 종이 기준 사각형(세로 오프셋
//!   60000 HU, 높이 6000 HU)을 하나 더 둔다.
//!
//! 참고(웹한글기안기 인쇄, HWP 판): `paper_square`·`paper_front` 1쪽, 둘째 행은 저장 높이
//! 그대로이고 사각형은 종이 위끝에서 533px 에 표 행들 위로 겹친다. `paper_square_nested` 도
//! 1쪽이고, 안쪽 사각형은 바깥 사각형 아래 종이 위끝에서 800px 에 놓이며 바깥 사각형은 선언
//! 높이 그대로다. `paper_square_split` 2쪽,
//! 1쪽은 `MID LINE 42` 까지, 2쪽에 `MID LINE 43` 부터 `TAIL ROW`·`END`. 데스크톱 한컴 2020 HWPX와 2024 HWP 직접 인쇄에서
//! 사각형은 1쪽이며 흰 채움과 표의 세로 괘선은 감싸기 속성이 정한 순서로 그려진다.
//!
//! 수정 전 rhwp 는 사각형 높이를 칸 문단 원점부터 세어 둘째 행을 185px 로 키웠고(사각형은 칸
//! clip 안에 그려 칸 밖 몫이 잘렸다), `paper_square`·`paper_front` 가 2쪽이 되었다. 글상자 안
//! 종이 기준 사각형은 글상자 위끝부터 세어 놓여 바깥 글상자를 884px 로 키웠다.
//!
//! `paper_square_split` 의 나뉜 칸은 세로 정렬이 위다. 기준 PDF 는 같은 입력의 한컴 2020
//! Print PDF(`pdf/paper_square_split-2020.pdf`)다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn load(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/pk40991_cell_paper_shape")
        .join(name);
    let bytes = std::fs::read(&path).expect("재현물");
    DocumentCore::from_bytes(&bytes).unwrap()
}

fn has_text(node: &RenderNode, text: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(text) {
            return true;
        }
    }
    node.children.iter().any(|child| has_text(child, text))
}

/// 글 `text` 를 품은 사각형들의 (y, 높이, 칸 clip 안에 있는지).
fn rects_with_text(
    node: &RenderNode,
    text: &str,
    in_cell_clip: bool,
    out: &mut Vec<(f64, f64, bool)>,
) {
    let before = out.len();
    let child_clip =
        in_cell_clip || matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.clip);
    for child in &node.children {
        rects_with_text(child, text, child_clip, out);
    }
    // 글을 품은 가장 안쪽 사각형만 센다(바깥 글상자는 안쪽 사각형을 자손으로 둘 수 있다).
    if out.len() == before
        && matches!(node.node_type, RenderNodeType::Rectangle(_))
        && has_text(node, text)
    {
        out.push((node.bbox.y, node.bbox.height, in_cell_clip));
    }
}

/// 바깥 표(깊이 1) `row` 행 칸들의 (y, 높이).
fn outer_row_cells(node: &RenderNode, depth: usize, row: u16, out: &mut Vec<(f64, f64)>) {
    let depth = match node.node_type {
        RenderNodeType::Table(_) => depth + 1,
        _ => depth,
    };
    if depth == 1 {
        if let RenderNodeType::TableCell(cell) = &node.node_type {
            if cell.row == row {
                out.push((node.bbox.y, node.bbox.height));
            }
        }
    }
    for child in &node.children {
        outer_row_cells(child, depth, row, out);
    }
}

/// 글 `text` 를 담은 첫 글줄의 y.
fn text_line_y(node: &RenderNode, text: &str) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && has_text(node, text) {
        return Some(node.bbox.y);
    }
    node.children
        .iter()
        .find_map(|child| text_line_y(child, text))
}

fn text_line_box(node: &RenderNode, text: &str) -> Option<(f64, f64)> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && has_text(node, text) {
        return Some((node.bbox.y, node.bbox.height));
    }
    node.children
        .iter()
        .find_map(|child| text_line_box(child, text))
}

fn outer_table_top(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(outer_table_top)
}

fn check_single_page(name: &str) {
    let core = load(name);
    assert_eq!(core.page_count(), 1, "{name}: 쪽 수");
    let page = core.build_page_render_tree(0).unwrap();
    assert!(has_text(&page.root, "TAIL ROW"), "{name}: 끝 행은 1쪽");
    assert!(has_text(&page.root, "END"), "{name}: 표 뒤 문단은 1쪽");

    let mut cells = Vec::new();
    outer_row_cells(&page.root, 0, 1, &mut cells);
    assert_eq!(cells.len(), 1, "{name}: 둘째 행 칸 하나");
    let (cell_y, cell_h) = cells[0];
    // 저장 높이 7500 HU = 100px. 사각형 높이(160px)를 더하지 않는다.
    assert!(
        (cell_h - 100.0).abs() < 1.0,
        "{name}: 둘째 행 높이 {cell_h:.1}px"
    );

    // 칸은 세로 가운데 정렬이다. 두 글줄은 칸 안에 놓이고, 위아래 여유가 거의 같다.
    let first = text_line_y(&page.root, "MID LINE 1").expect("첫 글줄");
    let second = text_line_y(&page.root, "MID LINE 2").expect("둘째 글줄");
    assert!(
        first > cell_y && second < cell_y + cell_h,
        "{name}: 글줄은 칸 안"
    );
    let top_gap = first - cell_y;
    let bottom_gap = cell_y + cell_h - (second + (second - first));
    assert!(
        (top_gap - bottom_gap).abs() < 12.0,
        "{name}: 가운데 정렬 위 {top_gap:.1}px 아래 {bottom_gap:.1}px"
    );

    // 사각형은 종이 기준 40000 HU(533.3px)에 한 번 그려진다.
    let mut rects = Vec::new();
    rects_with_text(&page.root, "PAPER RECT", false, &mut rects);
    assert_eq!(rects.len(), 1, "{name}: 사각형 한 번");
    let (rect_y, rect_h, clipped) = rects[0];
    // 사각형은 칸 밖(둘째 행 아래)에 놓이므로 칸 clip 에 들면 보이지 않는다.
    assert!(!clipped, "{name}: 사각형은 칸 clip 밖에 그린다");
    assert!((rect_y - 533.3).abs() < 1.0, "{name}: 사각형 y {rect_y:.1}");
    assert!(
        (rect_h - 160.0).abs() < 1.0,
        "{name}: 사각형 높이 {rect_h:.1}"
    );
}

fn check_split(name: &str) {
    let core = load(name);
    assert_eq!(core.page_count(), 2, "{name}: 쪽 수");
    let first = core.build_page_render_tree(0).unwrap();
    let last = core.build_page_render_tree(1).unwrap();
    assert!(
        has_text(&first.root, "MID LINE 42"),
        "{name}: 1쪽 칸 조각 끝"
    );
    assert!(
        !has_text(&first.root, "MID LINE 43"),
        "{name}: 43번째 줄은 2쪽"
    );
    for text in ["MID LINE 43", "MID LINE 70", "ROW 1", "TAIL ROW", "END"] {
        assert!(has_text(&last.root, text), "{name}: {text} 는 2쪽");
    }
    // 두 조각은 같은 본문 상단과 같은 표 바깥 위 여백을 소유한다.
    let first_top = outer_table_top(&first.root).expect("첫 표 조각");
    let last_top = outer_table_top(&last.root).expect("이어받은 표 조각");
    assert!(
        (first_top - last_top).abs() < 0.5,
        "{name}: 표 조각 위끝 {first_top:.2} / {last_top:.2}"
    );
    // 재현물의 나뉜 칸은 세로 정렬이 위다. 첫 조각의 가시 줄은 칸 안 위 여백 바로
    // 아래에서 시작한다(나뉜 칸의 가운데 정렬은 이 시험 범위 밖).
    let mut cells = Vec::new();
    outer_row_cells(&first.root, 0, 1, &mut cells);
    assert_eq!(cells.len(), 1, "{name}: 첫 가운데 칸 조각");
    let (cell_y, cell_h) = cells[0];
    let (first_y, _) = text_line_box(&first.root, "MID LINE 1").expect("첫 가시 줄");
    let (last_y, last_h) = text_line_box(&first.root, "MID LINE 42").expect("끝 가시 줄");
    let top_gap = first_y - cell_y;
    assert!(
        top_gap < 4.0 && last_y + last_h <= cell_y + cell_h + 0.5,
        "{name}: 위 정렬 위 {top_gap:.2}px, 끝 줄 아래 {:.2} / 칸 아래 {:.2}",
        last_y + last_h,
        cell_y + cell_h
    );
    let (next_y, _) = text_line_box(&last.root, "MID LINE 43").expect("이어받은 첫 줄");
    assert!(next_y > last_top, "{name}: 이어받은 줄이 표 위끝 아래");
    // 같은 입력의 한컴 2020 Print PDF는 앵커 문단이 2쪽이어도 사각형을 1쪽에 인쇄한다.
    let mut first_rects = Vec::new();
    rects_with_text(&first.root, "PAPER RECT", false, &mut first_rects);
    assert_eq!(
        first_rects.len(),
        1,
        "{name}: 종이 기준 사각형은 첫 칸 조각 쪽"
    );
    let mut last_rects = Vec::new();
    rects_with_text(&last.root, "PAPER RECT", false, &mut last_rects);
    assert!(
        last_rects.is_empty(),
        "{name}: 다음 쪽에는 사각형 중복 없음"
    );
    let mut rects = Vec::new();
    for page in [&first, &last] {
        rects_with_text(&page.root, "PAPER RECT", false, &mut rects);
    }
    assert_eq!(rects.len(), 1, "{name}: 사각형 한 번");
    assert!(!rects[0].2, "{name}: 사각형은 칸 clip 밖에 그린다");
}

/// 글상자 안 종이 기준 사각형은 종이 좌표에 놓이고 바깥 글상자를 키우지 않는다.
fn check_nested(name: &str) {
    check_single_page(name);
    let core = load(name);
    let page = core.build_page_render_tree(0).unwrap();
    let mut nested = Vec::new();
    rects_with_text(&page.root, "NESTED RECT", false, &mut nested);
    assert_eq!(nested.len(), 1, "{name}: 안쪽 사각형 한 번");
    let (nested_y, nested_h, clipped) = nested[0];
    assert!(
        (nested_y - 800.0).abs() < 1.0,
        "{name}: 안쪽 사각형 y {nested_y:.1}"
    );
    assert!(
        (nested_h - 80.0).abs() < 1.0,
        "{name}: 안쪽 사각형 높이 {nested_h:.1}"
    );
    assert!(!clipped, "{name}: 안쪽 사각형은 칸 clip 밖");
}

#[test]
fn cell_paper_square_nested_paper_rect_hwpx() {
    check_nested("paper_square_nested.hwpx");
}

#[test]
fn cell_paper_square_nested_paper_rect_hwp5() {
    check_nested("paper_square_nested.hwp");
}

#[test]
fn cell_paper_square_does_not_grow_row_hwpx() {
    check_single_page("paper_square.hwpx");
}

#[test]
fn cell_paper_square_does_not_grow_row_hwp5() {
    check_single_page("paper_square.hwp");
}

#[test]
fn cell_paper_front_does_not_grow_row_hwpx() {
    check_single_page("paper_front.hwpx");
}

#[test]
fn cell_paper_front_does_not_grow_row_hwp5() {
    check_single_page("paper_front.hwp");
}

#[test]
fn cell_paper_square_split_keeps_fragments_hwpx() {
    check_split("paper_square_split.hwpx");
}

#[test]
fn cell_paper_square_split_keeps_fragments_hwp5() {
    check_split("paper_square_split.hwp");
}

/// 같은 부모가 그리는 괘선과 도형의 실제 순서를 확인한다. 위아래로 사각형을
/// 가로지르는 세로 괘선만 고르므로 단순히 선이 존재하는지만 검사하지 않는다.
fn rule_and_rectangle_order(parent: &RenderNode) -> Option<(usize, usize)> {
    if let Some((shape_index, shape)) = parent.children.iter().enumerate().find(|(_, child)| {
        matches!(child.node_type, RenderNodeType::Rectangle(_)) && has_text(child, "PAPER RECT")
    }) {
        let rule = parent.children.iter().position(|line| {
            matches!(line.node_type, RenderNodeType::Line(_))
                && line.bbox.height > line.bbox.width
                && line.bbox.intersects(&shape.bbox)
                && line.bbox.y < shape.bbox.y
                && line.bbox.y + line.bbox.height > shape.bbox.y + shape.bbox.height
        })?;
        return Some((rule, shape_index));
    }
    parent.children.iter().find_map(rule_and_rectangle_order)
}

#[test]
fn overlapping_cell_rule_respects_foreground_and_background_shape_wrap() {
    for (name, foreground) in [
        ("paper_square_split.hwpx", true),
        ("paper-split-front.hwpx", true),
        ("paper-split-behind.hwpx", false),
    ] {
        let core = load(name);
        let first = core.build_page_render_tree(0).unwrap();
        let second = core.build_page_render_tree(1).unwrap();
        let (rule, shape) =
            rule_and_rectangle_order(&first.root).expect("1쪽의 도형을 가로지르는 괘선");
        assert_eq!(shape > rule, foreground, "{name}");
        assert!(
            !has_text(&second.root, "PAPER RECT"),
            "{name}: 이월한 칸에 도형 중복"
        );
    }
}

#[test]
fn paper_anchored_child_preserves_parent_text_vertical_alignment() {
    for ext in ["hwpx", "hwp"] {
        let plain = load(&format!("paper_square.{ext}"));
        let nested = load(&format!("paper_square_nested.{ext}"));
        let before = plain.build_page_render_tree(0).unwrap();
        let after = nested.build_page_render_tree(0).unwrap();
        let plain_line = text_line_box(&before.root, "PAPER RECT").unwrap();
        let nested_line = text_line_box(&after.root, "PAPER RECT").unwrap();
        assert_eq!(
            plain_line, nested_line,
            "{ext}: 종이 기준 자식은 부모의 글줄을 밀지 않는다"
        );
        let child = text_line_y(&after.root, "NESTED RECT").unwrap();
        assert!(child > nested_line.0 + nested_line.1);
    }
}
