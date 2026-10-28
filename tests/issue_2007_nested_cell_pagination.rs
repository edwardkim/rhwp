//! Issue #2007: 1×1 단일 셀 중첩 표의 셀 콘텐츠 페이지 분할(intra-cell pagination).
//!
//! `samples/basic/issue2007_nested_cell_pagination_42065.hwp` (규제영향분석서)는
//! 1×1 RowBreak 표(자리차지) 안에 중첩 1×1 표가 있고, 그 중첩 셀에 135+문단(약 8164px,
//! 8쪽 분량)이 담긴다.
//!
//! 회귀 (수정 전 버그, rhwp 6p vs 한글 17p):
//! - per-중첩행 유닛 분해(`cell_units`)는 중첩 표 `row_count >= 2` 에만 적용 →
//!   1×1(단일 행) 중첩 표는 atomic 유닛 1개로 취급 → 8164px 콘텐츠가 한 페이지에 통째
//!   배치(오버플로/크램) → under-pagination.
//!
//! 정정: 1×1 중첩 표의 셀 콘텐츠가 한 페이지를 명백히 초과(>1000px)하면 기존
//! `nested_table_mixed_fragment_heights`(텍스트+중첩표 문단에 쓰던 페이지 분할 fragment)
//! 를 빈-텍스트 문단에도 적용해 splittable 유닛으로 분해 → 페이지 경계로 분할.
//! 한컴 2020 PDF = 17페이지. #4069의 완료 계약은 중첩 표를 하위 행·셀
//! 흐름까지 분할해 빠짐·중복 없이 17페이지에 정확히 수렴하는 것이다.

use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

/// [#4207] 이 파일의 테스트는 같은 픽스처를 조판한다. 생성 suite 한 프로세스에서
/// 병렬이면 LayoutEngine 의 cell-units 포인터 캐시가 allocator 재사용으로
/// 저장 프레임 경계를 흔든다(shard 간헐 red, 무관 PR 오염).
/// 단언은 그대로 두고 조판만 직렬화한다. CI nextest 는 별도 프로세스가 기본이라
/// `.config/nextest.toml` 에서 해당 테스트를 exclusive 로 돌린다.
static ISSUE_2007_LAYOUT_LOCK: Mutex<()> = Mutex::new(());

fn lock_issue_2007_layout() -> MutexGuard<'static, ()> {
    ISSUE_2007_LAYOUT_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn page_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        page_text(child, out);
    }
}

fn normalized_page_text(core: &DocumentCore, page: u32) -> String {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("render tree p{}: {error:?}", page + 1));
    let mut text = String::new();
    page_text(&tree.root, &mut text);
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn terminal_bottom_lines_with_cell_clips(
    node: &RenderNode,
    clip_ancestors: &mut Vec<BoundingBox>,
    found: &mut Vec<(BoundingBox, Vec<BoundingBox>)>,
) {
    let pushes_clip = matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.clip);
    if pushes_clip {
        clip_ancestors.push(node.bbox);
    }

    if matches!(node.node_type, RenderNodeType::Line(_))
        && node.bbox.y > 820.0
        && node.bbox.width > 500.0
        && node.bbox.height <= 2.0
    {
        found.push((node.bbox, clip_ancestors.clone()));
    }
    for child in &node.children {
        terminal_bottom_lines_with_cell_clips(child, clip_ancestors, found);
    }

    if pushes_clip {
        clip_ancestors.pop();
    }
}

fn svg_number_attr(tag: &str, name: &str) -> f64 {
    let marker = format!("{name}=\"");
    let value = tag
        .split_once(&marker)
        .and_then(|(_, tail)| tail.split_once('"'))
        .map(|(value, _)| value)
        .unwrap_or_else(|| panic!("SVG attribute {name} missing: {tag}"));
    value
        .parse::<f64>()
        .unwrap_or_else(|error| panic!("SVG attribute {name}={value}: {error}"))
}

/// 지정한 원본 표 control의 렌더 조각을 깊이와 무관하게 찾는다.
fn find_table_fragment(
    node: &RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<&RenderNode> {
    if matches!(
        node.node_type,
        RenderNodeType::Table(ref table)
            if table.para_index == Some(para_index) && table.control_index == Some(control_index)
    ) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_table_fragment(child, para_index, control_index))
}

/// `needle`을 실제로 포함하는 가장 안쪽 table fragment를 찾는다.
///
/// p10의 결함은 표가 통째로 사라지는 문제가 아니라, continuation viewport에 걸친
/// 하위 1×1 표가 Center valign을 유지해 첫 본문을 표 상단에서 수백 px 아래로
/// 보내는 형태다. 따라서 source control만 찾는 기존 helper로는 해당 하위 표를
/// 특정할 수 없다.
fn find_innermost_table_containing_text<'a>(
    node: &'a RenderNode,
    needle: &str,
) -> Option<&'a RenderNode> {
    for child in &node.children {
        if let Some(found) = find_innermost_table_containing_text(child, needle) {
            return Some(found);
        }
    }
    (matches!(node.node_type, RenderNodeType::Table(_)) && contains_text(node, needle))
        .then_some(node)
}

fn contains_text(node: &RenderNode, needle: &str) -> bool {
    matches!(node.node_type, RenderNodeType::TextRun(ref run) if run.text.contains(needle))
        || node
            .children
            .iter()
            .any(|child| contains_text(child, needle))
}

fn first_text_run_top(node: &RenderNode, needle: &str) -> Option<f64> {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) if run.text.contains(needle) => Some(node.bbox.y),
        _ => None,
    };
    own.or_else(|| {
        node.children
            .iter()
            .filter_map(|child| first_text_run_top(child, needle))
            .min_by(|left, right| left.total_cmp(right))
    })
}

fn first_text_run_vertical_bounds(node: &RenderNode, needle: &str) -> Option<(f64, f64)> {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) if run.text.contains(needle) => {
            Some((node.bbox.y, node.bbox.y + node.bbox.height))
        }
        _ => None,
    };
    own.or_else(|| {
        node.children
            .iter()
            .filter_map(|child| first_text_run_vertical_bounds(child, needle))
            .min_by(|left, right| left.0.total_cmp(&right.0))
    })
}

#[derive(Clone, Copy)]
struct ClipRect {
    x: f64,
    y: f64,
    right: f64,
    bottom: f64,
}

impl ClipRect {
    fn from_node(node: &RenderNode) -> Self {
        Self {
            x: node.bbox.x,
            y: node.bbox.y,
            right: node.bbox.x + node.bbox.width,
            bottom: node.bbox.y + node.bbox.height,
        }
    }

    fn intersect(self, other: Self) -> Option<Self> {
        let clipped = Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        };
        (clipped.right > clipped.x && clipped.bottom > clipped.y).then_some(clipped)
    }

    fn intersects_node(self, node: &RenderNode) -> bool {
        self.intersect(Self::from_node(node)).is_some()
    }

    fn fully_contains(self, other: Self) -> bool {
        const EPSILON: f64 = 0.01;
        other.x + EPSILON >= self.x
            && other.y + EPSILON >= self.y
            && other.right <= self.right + EPSILON
            && other.bottom <= self.bottom + EPSILON
    }
}

/// SVG와 Canvas가 공통으로 지키는 TableCell clip을 적용한 뒤의 가시 text만 센다.
fn contains_painted_text(node: &RenderNode, needle: &str, clip: Option<ClipRect>) -> bool {
    // RenderNode::visible is honored by both the SVG and Canvas painters.
    // A source line deliberately suppressed at a pagination seam must not be
    // counted as painted merely because its layout bbox still intersects the
    // physical page.
    if !node.visible {
        return false;
    }
    let clip = match &node.node_type {
        RenderNodeType::TableCell(cell) if cell.clip => {
            clip.and_then(|active| active.intersect(ClipRect::from_node(node)))
        }
        _ => clip,
    };
    if matches!(
        node.node_type,
        RenderNodeType::TextRun(ref run) if run.text.contains(needle)
    ) && clip.is_some_and(|active| active.intersects_node(node))
    {
        return true;
    }
    node.children
        .iter()
        .any(|child| contains_painted_text(child, needle, clip))
}

/// Substring이 같은 페이지의 다른 본문에 나타나도 제목 소유권으로 오인하지 않도록,
/// trim한 TextRun 전체가 정확히 일치하는 가시 text만 센다.
fn contains_exact_painted_text(node: &RenderNode, expected: &str, clip: Option<ClipRect>) -> bool {
    if !node.visible {
        return false;
    }
    let clip = match &node.node_type {
        RenderNodeType::TableCell(cell) if cell.clip => {
            clip.and_then(|active| active.intersect(ClipRect::from_node(node)))
        }
        _ => clip,
    };
    if matches!(
        node.node_type,
        RenderNodeType::TextRun(ref run) if run.text.trim() == expected
    ) && clip.is_some_and(|active| active.intersects_node(node))
    {
        return true;
    }
    node.children
        .iter()
        .any(|child| contains_exact_painted_text(child, expected, clip))
}

/// `scope` 아래의 정확한 TextLine을 찾되, root부터 내려오며 적용되는 모든
/// `clip=true TableCell` 교집합도 함께 보존한다. TextRun 단위 교차만 검사하면
/// p14 하단처럼 render tree에는 있으나 paint 때 잘리는 줄을 놓친다.
fn collect_exact_text_line_clips_in_subtree(
    node: &RenderNode,
    scope: &RenderNode,
    expected: &str,
    clip: Option<ClipRect>,
    inside_scope: bool,
    found: &mut Vec<(ClipRect, Option<ClipRect>)>,
) {
    if !node.visible || node.editor_only {
        return;
    }
    let clip = match &node.node_type {
        RenderNodeType::TableCell(cell) if cell.clip => {
            clip.and_then(|active| active.intersect(ClipRect::from_node(node)))
        }
        _ => clip,
    };
    let inside_scope = inside_scope || std::ptr::eq(node, scope);
    if inside_scope && matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        page_text(node, &mut text);
        if text.trim() == expected {
            found.push((ClipRect::from_node(node), clip));
        }
    }
    for child in &node.children {
        collect_exact_text_line_clips_in_subtree(child, scope, expected, clip, inside_scope, found);
    }
}

/// Return the painted right extent of a nested table's own outer vertical
/// border.  `LineNode` stores its centerline, so account for half its stroke.
fn nested_table_right_border_paint_extent(table: &RenderNode) -> Option<f64> {
    let table_right = table.bbox.x + table.bbox.width;
    table
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::Line(line)
                if (line.x1 - line.x2).abs() < 0.01
                    && (line.y1 - line.y2).abs() > 1.0
                    && (line.x1 - table_right).abs() <= (line.style.width + 1.0).max(2.0) =>
            {
                Some(line.x1 + line.style.width / 2.0)
            }
            _ => None,
        })
        .max_by(|left, right| left.total_cmp(right))
}

/// A RowBreak host clips its direct nested table at the page boundary. The
/// continuation needs a newly painted top edge whose *whole stroke* is inside
/// that clip; a centerline exactly on the boundary is only a half-painted SVG
/// or Canvas rule.
fn has_direct_full_width_horizontal_line_inside_top_clip(
    table: &RenderNode,
    clip_top: f64,
) -> bool {
    let left = table.bbox.x;
    let right = table.bbox.x + table.bbox.width;
    table.children.iter().any(|child| {
        matches!(
            &child.node_type,
            RenderNodeType::Line(line)
                if child.visible
                    && (line.y1 - line.y2).abs() <= 0.1
                    && (line.x1.min(line.x2) - left).abs() <= 0.6
                    && (line.x1.max(line.x2) - right).abs() <= 0.6
                    && line.y1 - line.style.width / 2.0 >= clip_top + 0.001
                    && line.y1 - clip_top <= line.style.width.max(1.0) + 0.6
        )
    })
}

fn has_visible_full_width_horizontal_line_near(
    node: &RenderNode,
    left: f64,
    right: f64,
    y: f64,
) -> bool {
    matches!(
        &node.node_type,
        RenderNodeType::Line(line)
            if node.visible
                && (line.y1 - line.y2).abs() <= 0.1
                && (line.y1 - y).abs() <= 0.6
                && (line.x1.min(line.x2) - left).abs() <= 0.6
                && (line.x1.max(line.x2) - right).abs() <= 0.6
    ) || node
        .children
        .iter()
        .any(|child| has_visible_full_width_horizontal_line_near(child, left, right, y))
}

/// `visible` is inherited by the painter.  A table hidden as a future-page
/// residue can keep its child Line nodes structurally present, so this helper
/// must stop at an invisible ancestor rather than inspecting leaf visibility
/// alone.
fn has_painted_horizontal_line_in_bottom_residue(node: &RenderNode, clip_bottom: f64) -> bool {
    if !node.visible {
        return false;
    }
    matches!(
        &node.node_type,
        RenderNodeType::Line(line)
            if (line.y1 - line.y2).abs() <= 0.1
                && line.y1 >= clip_bottom - 0.5
                && line.y1 <= clip_bottom + 1.0
    ) || node
        .children
        .iter()
        .any(|child| has_painted_horizontal_line_in_bottom_residue(child, clip_bottom))
}

/// Find the innermost table containing `needle` and verify that its real
/// bottom border's full stroke survives every enclosing `TableCell` clip.
/// A line node alone is insufficient: SVG/Canvas clip paths can silently
/// erase the line after layout has emitted it (issue2007 p9).
fn nested_table_bottom_border_is_painted(
    node: &RenderNode,
    needle: &str,
    clip: Option<ClipRect>,
) -> Option<bool> {
    let clip = match &node.node_type {
        RenderNodeType::TableCell(cell) if cell.clip => {
            clip.and_then(|active| active.intersect(ClipRect::from_node(node)))
        }
        _ => clip,
    };
    for child in &node.children {
        if let Some(result) = nested_table_bottom_border_is_painted(child, needle, clip) {
            return Some(result);
        }
    }
    if !matches!(node.node_type, RenderNodeType::Table(_)) || !contains_text(node, needle) {
        return None;
    }
    let table_left = node.bbox.x;
    let table_right = table_left + node.bbox.width;
    let table_bottom = node.bbox.y + node.bbox.height;
    let Some(active_clip) = clip else {
        return Some(false);
    };
    Some(node.children.iter().any(|child| {
        matches!(
            &child.node_type,
            RenderNodeType::Line(line)
                if child.visible
                    && (line.y1 - line.y2).abs() <= 0.1
                    && (line.y1 - table_bottom).abs() <= 0.6
                    && (line.x1.min(line.x2) - table_left).abs() <= 0.6
                    && (line.x1.max(line.x2) - table_right).abs() <= 0.6
                    && line.y1 + line.style.width / 2.0 <= active_clip.bottom + 0.01
        )
    }))
}

/// A clipped continuation frame needs a bottom edge placed fully inside its
/// physical clip.  A centerline exactly on the clip bottom paints as a half
/// line in SVG/Canvas and can disappear at device scale.
fn has_direct_bottom_frame_inside_clip(table: &RenderNode, clip_bottom: f64) -> bool {
    let left = table.bbox.x;
    let right = table.bbox.x + table.bbox.width;
    let table_bottom = table.bbox.y + table.bbox.height;
    table.children.iter().any(|child| {
        matches!(
            &child.node_type,
            RenderNodeType::Line(line)
                if child.visible
                    && (line.y1 - line.y2).abs() <= 0.1
                    && (line.x1.min(line.x2) - left).abs() <= 0.6
                    && (line.x1.max(line.x2) - right).abs() <= 0.6
                    && (line.y1 - table_bottom).abs() <= 0.6
                    && line.y1 + line.style.width / 2.0 <= clip_bottom + 0.01
        )
    })
}

/// Wrapper Cell이 직접 포함한 중첩 표의 바깥 우측선을 모두 검사한다.
///
/// issue2007 p2에는 4×2와 9×2 표가 한 wrapper Cell 안에 연달아 있고, p3에는
/// 같은 9×2 표의 continuation만 남는다. 둘 다 stored width가 wrapper의 논리
/// clip보다 조금 넓어, 표가 완성되기 전에 clip 범위를 계산하면 우측선이 통째로
/// 사라진다.
fn direct_nested_table_right_borders(cell: &RenderNode) -> Vec<f64> {
    cell.children
        .iter()
        .filter(|child| matches!(child.node_type, RenderNodeType::Table(_)))
        .filter_map(nested_table_right_border_paint_extent)
        .collect()
}

/// A table fragment's direct host Cell is the ancestor SVG/Canvas clip for
/// all of its nested descendants.  The deepest nested table can have a valid
/// border `Line` while that ancestor still silently clips the stroke.
fn direct_table_cell(table: &RenderNode) -> Option<&RenderNode> {
    table
        .children
        .iter()
        .find(|child| matches!(child.node_type, RenderNodeType::TableCell(_)))
}

/// 한 TableCell의 직접 콘텐츠에서만 실제 TextLine 상자들을 수집한다. 중첩 셀은
/// 별도 좌표계이므로 여기서 섞으면 정상적인 열/중첩 표를 거짓 양성으로 판정한다.
fn collect_direct_cell_text_lines(node: &RenderNode, lines: &mut Vec<ClipRect>) {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        return;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_))
        && node.visible
        && !node.editor_only
        && node.bbox.width > 0.0
        && node.bbox.height > 0.0
    {
        lines.push(ClipRect::from_node(node));
    }
    for child in &node.children {
        collect_direct_cell_text_lines(child, lines);
    }
}

/// 같은 셀의 실제 TextLine 두 줄이 충분히 큰 면적으로 겹치는지 검사한다.
///
/// 이 문서 p10--p16의 결함은 nested 1×1 continuation 안에서 LINE_SEG `vpos=0`
/// 재시작을 새 셀의 원점으로 오인해, 앞 문단 위로 뒤 문단을 재배치한 경우였다.
/// 단순 bbox 교차만으로는 정상적인 인접 줄 간 anti-aliasing까지 잡으므로,
/// `fidelity_compare.py`와 같은 문턱(세로 3px 또는 작은 줄의 35%, 가로 24px 또는
/// 작은 줄의 45%)을 쓴다.
fn has_substantial_direct_text_line_overlap(cell: &RenderNode) -> bool {
    let mut lines = Vec::new();
    for child in &cell.children {
        collect_direct_cell_text_lines(child, &mut lines);
    }
    lines.iter().enumerate().any(|(index, first)| {
        lines[index + 1..].iter().any(|second| {
            let overlap_x = (first.right.min(second.right) - first.x.max(second.x)).max(0.0);
            let overlap_y = (first.bottom.min(second.bottom) - first.y.max(second.y)).max(0.0);
            let min_width = (first.right - first.x).min(second.right - second.x);
            let min_height = (first.bottom - first.y).min(second.bottom - second.y);
            overlap_x >= 24.0_f64.max(min_width * 0.45)
                && overlap_y >= 3.0_f64.max(min_height * 0.35)
        })
    })
}

/// 표 조각 아래 어느 nested TableCell에서도 같은 셀 내부의 줄 겹침이 없어야 한다.
fn has_nested_cell_text_overlap(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::TableCell(_))
        && has_substantial_direct_text_line_overlap(node)
        || node.children.iter().any(has_nested_cell_text_overlap)
}

#[test]
fn issue_2007_nested_cell_cursor_has_no_boundary_duplication() {
    let _issue_2007_layout = lock_issue_2007_layout();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture parse");
    let page2 = normalized_page_text(&core, 1);
    let page3 = normalized_page_text(&core, 2);

    const FIRST_ITEM: &str = "1.출석요구및진술청취또는진술서제출요구";
    const SECOND_ITEM: &str = "2.신고사항과관련이있다고인정되는자료등의제출요구";
    assert!(
        page2.contains(FIRST_ITEM),
        "2쪽에 조문 대비표 제1호가 없다 — 첫 child cursor 누락"
    );
    assert!(
        !page2.contains(SECOND_ITEM),
        "3쪽 소속 조문 대비표 제2호가 2쪽에 미리 노출됐다 — 비종료 clip 회귀"
    );
    assert!(
        !page3.contains(FIRST_ITEM),
        "3쪽에 조문 대비표 제1호가 반복됐다 — continuation cursor 중복"
    );
    assert!(
        page3.contains(SECOND_ITEM),
        "3쪽에 조문 대비표 제2호가 없다 — continuation cursor 누락"
    );
    assert!(
        page3.contains("④제1항부터제3항까지"),
        "3쪽에 조문 대비표 마지막 개정 조항이 없다 — terminal cursor 누락"
    );
}

#[test]
fn issue_2007_recursive_partial_render_is_page_order_independent() {
    let _issue_2007_layout = lock_issue_2007_layout();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");

    // 재귀 partial-table 렌더는 같은 문서 원본을 사용하므로 앞 페이지 렌더 순서가
    // 마지막 페이지의 cell-unit identity를 바꾸면 안 된다. 과거에는 매 페이지 만든
    // 임시 Table clone의 cell 주소가 재사용되어, p1→p17 순차 렌더와 p17 단독 렌더가
    // 서로 다른 캐시 entry를 적중했다.
    let sequential = DocumentCore::from_bytes(&bytes).expect("sequential fixture parse");
    for page_index in 0..16 {
        let _ = normalized_page_text(&sequential, page_index);
    }
    let sequential_p17 = normalized_page_text(&sequential, 16);

    let direct = DocumentCore::from_bytes(&bytes).expect("direct fixture parse");
    let direct_p17 = normalized_page_text(&direct, 16);

    assert_eq!(
        sequential_p17, direct_p17,
        "p17 render text changed after warming p1-p16; recursive partial tables must use stable model identity"
    );
}

#[test]
fn issue_2007_intra_paragraph_saved_frame_break_is_preserved() {
    let _issue_2007_layout = lock_issue_2007_layout();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture parse");
    let page10 = normalized_page_text(&core, 9);
    let page11 = normalized_page_text(&core, 10);

    const FRAME_START: &str = "제50조의2(조사권의남용금지)";
    const FRAME_CONTINUATION: &str = "행하여야하며,다른목적등을위하여조사권을남용하여서는아니된다.";
    const NEXT_ARTICLE: &str = "제50조의4(이행강제금등)";

    assert!(
        page10.contains(FRAME_START),
        "10쪽에 저장 프레임 말미 조항이 없다"
    );
    assert!(
        !page10.contains(FRAME_CONTINUATION),
        "10쪽에 다음 저장 프레임이 겹쳤다 — 문단 내부 vpos reset 소실"
    );
    assert!(
        page11.contains(FRAME_CONTINUATION),
        "11쪽에 문단 내부 vpos reset 이후 줄이 없다"
    );
    assert!(
        page11.contains(NEXT_ARTICLE),
        "11쪽에 후속 조항이 없다 — child cursor 누락"
    );
}

#[test]
fn issue_2007_nested_table_right_outer_border_is_not_clipped() {
    let _issue_2007_layout = lock_issue_2007_layout();
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // p4의 outer 1×1 RowBreak 표(pi=6, ci=0) 안에는 stored width를 유지하는
    // 12×5 nested table이 있다. 종전에는 parent TableCell/Body clip이 nested
    // table의 우측 vertical border보다 좁아 SVG/Canvas에서 선 전체가 사라졌다.
    let tree = doc
        .build_page_render_tree(3)
        .expect("issue2007 p4 render tree");
    let outer = find_table_fragment(&tree.root, 6, 0).expect("issue2007 p4의 outer pi=6 ci=0 표");
    let cell = outer
        .children
        .iter()
        .find(|child| matches!(child.node_type, RenderNodeType::TableCell(_)))
        .expect("outer table's clipped cell");
    let nested = cell
        .children
        .iter()
        .find(|child| matches!(child.node_type, RenderNodeType::Table(_)))
        .expect("outer cell's nested table");
    let border_right =
        nested_table_right_border_paint_extent(nested).expect("nested table right outer border");
    let cell_clip_right = cell.bbox.x + cell.bbox.width;

    assert!(
        cell_clip_right + 0.01 >= border_right,
        "p4 nested table right border is outside its parent cell clip: \
         clip_right={cell_clip_right:.2}, border_right={border_right:.2}"
    );
}

#[test]
fn issue_2007_wrapper_clip_keeps_completed_nested_table_right_borders() {
    let _issue_2007_layout = lock_issue_2007_layout();
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // p2의 4×2·9×2 표와 p3의 9×2 continuation은 모두 outer wrapper(pi=2,
    // ci=1)의 오른쪽 logical clip보다 넓다. 기준 PDF에는 세 outer vertical
    // stroke가 보인다. 종전 p4 단일 보정은 child table의 edge가 아직 emit되기 전
    // cell loop에서 실행돼 이 경로를 놓쳤다.
    for (page_index, expected_borders) in [(1, 2), (2, 1)] {
        let tree = doc
            .build_page_render_tree(page_index)
            .unwrap_or_else(|e| panic!("issue2007 p{} render tree: {e}", page_index + 1));
        let outer = find_table_fragment(&tree.root, 2, 1).unwrap_or_else(|| {
            panic!(
                "issue2007 p{}의 outer wrapper pi=2 ci=1 표 조각",
                page_index + 1
            )
        });
        let wrapper = outer
            .children
            .iter()
            .find(|child| !direct_nested_table_right_borders(child).is_empty())
            .unwrap_or_else(|| {
                panic!(
                    "issue2007 p{} outer wrapper가 completed nested table을 직접 포함해야 함",
                    page_index + 1
                )
            });
        let right_borders = direct_nested_table_right_borders(wrapper);
        assert_eq!(
            right_borders.len(),
            expected_borders,
            "p{} direct nested table right border count",
            page_index + 1
        );
        let clip_right = wrapper.bbox.x + wrapper.bbox.width;
        for border_right in right_borders {
            assert!(
                clip_right + 0.01 >= border_right,
                "p{} completed nested table right border is outside its wrapper clip: \
                 clip_right={clip_right:.2}, border_right={border_right:.2}",
                page_index + 1,
            );
        }
    }
}

#[test]
fn issue_2007_continuation_ancestor_clip_keeps_deep_right_border() {
    let _issue_2007_layout = lock_issue_2007_layout();
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // p10's pi=7/ci=1 RowBreak wrapper contains the long 1x1 continuation.
    // Its deepest table emits the right border, but every enclosing Cell clip
    // must include that stroke for both SVG and Canvas paint.
    let tree = doc
        .build_page_render_tree(9)
        .expect("issue2007 p10 render tree");
    let outer =
        find_table_fragment(&tree.root, 7, 1).expect("issue2007 p10 outer pi=7 ci=1 continuation");
    let outer_cell = direct_table_cell(outer).expect("p10 outer direct Cell");
    let deepest = find_innermost_table_containing_text(outer, "독점규제 및 공정거래에 관한 법률")
        .expect("p10 nested table containing first visible law heading");
    let right_border =
        nested_table_right_border_paint_extent(deepest).expect("p10 nested table right border");
    let outer_clip_right = outer_cell.bbox.x + outer_cell.bbox.width;
    assert!(
        outer_clip_right + 0.01 >= right_border,
        "p10 continuation ancestor Cell clips its deep nested right border: \\
         ancestor_right={outer_clip_right:.2}, border_right={right_border:.2}"
    );
}

#[test]
fn issue_2007_cell_vpos_reset_does_not_overlap_following_paragraphs() {
    let _issue_2007_layout = lock_issue_2007_layout();
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // p2(0-based 1)의 pi=2, ci=1에는 일반 nested 9×2 표의 우측 cell이 있다. 세 번째
    // paragraph가 다시 vpos=0으로 시작한 뒤의 positive vpos를 cell-top anchor로 쓰면
    // 5쌍의 본문 줄이 겹친다. continuation만의 예외가 아니라 일반 셀에도 같은 저장
    // 형식이 있으므로 먼저 이 구간을 고정한다.
    let p2_tree = doc
        .build_page_render_tree(1)
        .expect("issue2007 p2 render tree");
    let p2_fragment =
        find_table_fragment(&p2_tree.root, 2, 1).expect("issue2007 p2의 원본 pi=2 ci=1 표 조각");
    assert!(
        !has_nested_cell_text_overlap(p2_fragment),
        "p2 nested 9×2 table has overlapping painted text lines after a cell-local vpos reset"
    );

    // p10--p16(0-based 9--15)은 원본 pi=7, ci=1의 1×1 RowBreak 표가 계속되는
    // 구간이다. 과거에는 손자 셀의 중간 LINE_SEG `vpos=0`을 새 셀 시작으로 해석해
    // 각 쪽마다 최대 28쌍의 본문 줄을 겹쳐 paint했다. 쪽수/clip만으로는 이를 못
    // 잡으므로 같은 TableCell 내부의 가시 TextLine 기하를 직접 고정한다.
    for page_index in 9..=15 {
        let tree = doc
            .build_page_render_tree(page_index)
            .unwrap_or_else(|e| panic!("issue2007 p{} render tree: {e}", page_index + 1));
        let fragment = find_table_fragment(&tree.root, 7, 1).unwrap_or_else(|| {
            panic!(
                "issue2007 p{}의 원본 pi=7 ci=1 continuation 표 조각",
                page_index + 1
            )
        });
        assert!(
            !has_nested_cell_text_overlap(fragment),
            "p{} nested-cell continuation has overlapping painted text lines; \
             descendant LINE_SEG vpos reset must not rebase to the cell top",
            page_index + 1
        );
    }
}

#[test]
fn issue_4159_terminal_nested_bottom_border_is_inside_all_cell_clips() {
    let _issue_2007_layout = lock_issue_2007_layout();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture parse");
    let preceding_tree = core
        .build_page_render_tree(1)
        .expect("render physical page 2");
    let mut premature = Vec::new();
    terminal_bottom_lines_with_cell_clips(&preceding_tree.root, &mut Vec::new(), &mut premature);
    assert!(
        premature.is_empty(),
        "비종료 물리 2쪽에 종료 bottom 선이 미리 노출됐다: {premature:?}"
    );

    let tree = core
        .build_page_render_tree(2)
        .expect("render physical page 3");

    let mut found = Vec::new();
    terminal_bottom_lines_with_cell_clips(&tree.root, &mut Vec::new(), &mut found);
    assert_eq!(
        found.len(),
        1,
        "물리 3쪽의 폭 500px 이상 종료 bottom 선을 하나만 찾아야 한다: {found:?}"
    );

    let (line, clips) = &found[0];
    assert!(
        !clips.is_empty(),
        "종료 nested bottom 선에 clip=true TableCell 조상이 없다"
    );
    let line_bottom = line.y + line.height;
    for clip in clips {
        let clip_bottom = clip.y + clip.height;
        assert!(
            clip_bottom + 0.01 >= line_bottom,
            "종료 nested bottom stroke가 조상 셀 clip에 잘린다: line_bottom={line_bottom:.3}, clip_bottom={clip_bottom:.3}, line={line:?}, clip={clip:?}"
        );
    }
}

#[test]
fn issue_4159_svg_terminal_bottom_border_is_visible_inside_outer_cell_clip() {
    let _issue_2007_layout = lock_issue_2007_layout();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("fixture parse");
    let svg = doc
        .render_page_svg_native(2)
        .expect("render physical page 3 SVG");

    let outer_clip = svg
        .lines()
        .filter(|line| line.contains("<clipPath id=\"cell-clip"))
        .find(|line| {
            let x = svg_number_attr(line, "x");
            let width = svg_number_attr(line, "width");
            x < 80.0 && width > 650.0
        })
        .expect("physical page 3 outer split cell clip");
    let bottom_line = svg
        .lines()
        .filter(|line| line.starts_with("<line "))
        .find(|line| {
            let x1 = svg_number_attr(line, "x1");
            let x2 = svg_number_attr(line, "x2");
            let y1 = svg_number_attr(line, "y1");
            let y2 = svg_number_attr(line, "y2");
            y1 > 820.0 && (y1 - y2).abs() < 0.01 && x2 - x1 > 500.0
        })
        .expect("physical page 3 terminal nested bottom SVG line");

    let clip_bottom = svg_number_attr(outer_clip, "y") + svg_number_attr(outer_clip, "height");
    // [#6269] 획은 경로에 **중심 정렬**로 칠해지므로 잉크 하단은 `y1 + 획/2` 다.
    // 종전에는 전체 획을 더해 잉크를 반 획 아래로 잡았고, clip 도 같은 만큼 헐겁게
    // 잡혀 있어 우연히 맞아떨어졌다. 잉크 정의를 바로잡아 실제 경계를 잠근다.
    let line_bottom =
        svg_number_attr(bottom_line, "y1") + svg_number_attr(bottom_line, "stroke-width") / 2.0;
    assert!(
        clip_bottom + 0.01 >= line_bottom,
        "SVG bottom stroke가 outer cell clip에 잘린다: line_bottom={line_bottom:.3}, clip_bottom={clip_bottom:.3}\n{outer_clip}\n{bottom_line}"
    );
}
