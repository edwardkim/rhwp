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

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

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
fn issue_2007_nested_cell_content_paginates() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // 한컴 2020 기준 PDF는 17페이지다. `>= 12`는 24페이지 공백 회귀와
    // 23페이지 중복 회귀를 모두 통과시켜 #4069를 보호하지 못했다.
    let pages = doc.page_count();
    assert_eq!(
        pages, 17,
        "#4069 중첩 흐름 분할 회귀 — 페이지 수 {pages} (한컴 2020 기준 17)"
    );

    // p8(0-based 7)은 큰 1×1 RowBreak 표의 continuation이다. 이전에는 cell clip 밖에
    // 남아 있는 수천 px 자손까지 partial Table bbox/body clip을 확장해 Canvas/WASM
    // paint 후보가 현재 쪽을 벗어났다. 페이지 수 17만으로는 이 구조 결함을 못 잡는다.
    let tree = doc
        .build_page_render_tree(7)
        .expect("issue2007 p8 render tree");
    let fragment =
        find_table_fragment(&tree.root, 7, 1).expect("issue2007 p8의 원본 pi=7 ci=1 표 조각");
    let fragment_bottom = fragment.bbox.y + fragment.bbox.height;
    assert!(
        fragment_bottom <= tree.root.bbox.height + 0.5,
        "p8 RowBreak 표 조각 bbox가 쪽 밖으로 새어 Canvas/WASM paint 범위를 오염한다: \
         bottom={fragment_bottom:.1}, page_height={:.1}",
        tree.root.bbox.height
    );
    let fragment_cell =
        direct_table_cell(fragment).expect("issue2007 p8 RowBreak 표 조각의 직접 clipped cell");
    let clip_bottom = fragment_cell.bbox.y + fragment_cell.bbox.height;
    assert!(
        !has_painted_horizontal_line_in_bottom_residue(fragment, clip_bottom),
        "p8 paints the next fragment's top border at the preceding page bottom; \
         clip_bottom={clip_bottom:.1}"
    );
    assert!(
        !contains_painted_text(
            &tree.root,
            "및 정치부문에 존재하는 것으로 인식되는 부패의 정도를 측정",
            Some(ClipRect::from_node(&tree.root)),
        ),
        "p8 continuation이 p7 마지막 줄을 다시 paint한다 — mixed nested split의 콘텐츠 원점이 한 unit 앞서 있다"
    );

    // p7 끝에는 7×3 표의 제목이 남을 공간처럼 보이지만, 표 본체는 cell clip을
    // 통과하지 못한다. 한컴은 제목을 표와 분리하지 않고 p8에 함께 배치한다.
    let p7 = doc
        .build_page_render_tree(6)
        .expect("issue2007 p7 render tree");
    let p7_clip = Some(ClipRect::from_node(&p7.root));
    let carried_heading = "해외 반부패 전담기구 조사기능 현황";
    assert!(
        !contains_painted_text(&p7.root, carried_heading, p7_clip),
        "p7 must not paint a table heading whose table starts on p8"
    );
    assert!(
        contains_painted_text(
            &tree.root,
            carried_heading,
            Some(ClipRect::from_node(&tree.root))
        ),
        "p8 must retain the heading with its first visible table rows"
    );
    let heading_top =
        first_text_run_top(&tree.root, carried_heading).expect("p8 carried table heading text run");
    // 한컴 2020 PDF bbox의 제목 yMin=88.610521pt, 96dpi 환산 118.147px.
    assert!(
        (117.5..=119.0).contains(&heading_top),
        "p8 heading must match the Hancom PDF viewport, got y={heading_top}"
    );
}

#[test]
fn issue_2007_nested_cell_cursor_has_no_boundary_duplication() {
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
fn issue_2007_saved_frame_tail_nested_table_starts_before_next_frame() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&path).expect("fixture read");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture parse");
    let page15 = normalized_page_text(&core, 14);
    let page16 = normalized_page_text(&core, 15);

    const NESTED_TABLE_START: &str = "조달사업에관한법률";
    const NESTED_TABLE_TAIL: &str = "제4항에따라시정요구를받은계약상대자";
    const NEXT_FRAME: &str = "<이해관계자협의>:입법예고‧기관협의중";

    assert!(
        page15.contains(NESTED_TABLE_START),
        "15쪽 조달청 제목 뒤 자식 표가 다음 쪽으로 통째로 밀렸다"
    );
    assert!(
        page15.contains(NESTED_TABLE_TAIL),
        "15쪽 저장 프레임 말미까지 조달청 자식 표가 이어지지 않았다"
    );
    assert!(
        !page15.contains(NEXT_FRAME),
        "다음 저장 프레임의 이해관계자 협의 제목이 15쪽에 흡수됐다"
    );
    assert!(
        page16.contains(NEXT_FRAME),
        "16쪽이 이해관계자 협의 저장 프레임에서 재개하지 않았다"
    );
}

#[test]
fn issue_2007_nested_table_right_outer_border_is_not_clipped() {
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
fn issue_2007_single_cell_continuation_does_not_repaint_boundary_fragments() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // p11은 명시적 쪽 나누기 앞에서 끝나고 p12가 정확한 다음 제목
    // "중앙선거관리위원회"로 시작해야 한다. substring 검사는 p12 뒤쪽의
    // "중앙선거관리위원회규칙"에도 매치하므로 exact TextRun owner를 고정한다.
    let p11 = doc
        .build_page_render_tree(10)
        .expect("issue2007 p11 render tree");
    let p11_clip = Some(ClipRect::from_node(&p11.root));
    assert!(
        !contains_exact_painted_text(&p11.root, "중앙선거관리위원회", p11_clip),
        "p11 must not paint the p12-owned heading after an explicit page break"
    );
    let p12 = doc
        .build_page_render_tree(11)
        .expect("issue2007 p12 render tree");
    let p12_clip = Some(ClipRect::from_node(&p12.root));
    assert!(
        contains_exact_painted_text(&p12.root, "중앙선거관리위원회", p12_clip),
        "p12 must contain its exact first reference heading"
    );
    assert!(
        !contains_painted_text(
            &p12.root,
            "진술을 하거나 그 직무집행을 거부 또는 기피한 자",
            p12_clip,
        ),
        "p12 repaints the preceding continuation line instead of starting at its own fragment"
    );

    // 마지막 non-terminal fragment(p16)가 p17 소속 heading을 미리 paint하면,
    // terminal p17에도 같은 heading이 다시 나타난다. 한컴 기준은 p17만 보유한다.
    let p16 = doc
        .build_page_render_tree(15)
        .expect("issue2007 p16 render tree");
    assert!(
        !contains_painted_text(
            &p16.root,
            "선호된 대안의 기대효과",
            Some(ClipRect::from_node(&p16.root)),
        ),
        "p16 paints p17-owned heading before the terminal continuation"
    );
    let p17 = doc
        .build_page_render_tree(16)
        .expect("issue2007 p17 render tree");
    assert!(
        contains_painted_text(
            &p17.root,
            "선호된 대안의 기대효과",
            Some(ClipRect::from_node(&p17.root)),
        ),
        "p17 must retain its terminal heading"
    );
    assert!(
        contains_painted_text(
            &p17.root,
            "선호된 대안의 이해관계자 의견 및 조치",
            Some(ClipRect::from_node(&p17.root)),
        ),
        "p17 terminal nested-cell clip drops the source's final section 4"
    );

    // Canvas/SVG는 TextLine bbox보다 위로 나온 glyph ink도 ancestor cell clip으로
    // 자른다. p16/p17의 첫 visible heading이 cell top보다 2.5px 위였으므로 상단
    // 획이 잘렸다. 현재 fragment의 line box 자체를 inset 안으로 넣되, 이전 쪽
    // source line은 여전히 clip 밖에 남겨야 한다.
    for (tree, needle, page) in [
        (&p16, "이해관계자 협의", 16),
        (&p17, "선호된 대안의 기대효과", 17),
    ] {
        let line_top = first_text_run_top(&tree.root, needle)
            .unwrap_or_else(|| panic!("p{page} top continuation heading"));
        assert!(
            line_top >= 117.3,
            "p{page} first visible heading remains above its nested-cell paint clip: y={line_top}"
        );
    }
}

#[test]
fn issue_2007_completed_multiline_table_keeps_following_heading_in_next_viewport() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {e}", hwp_path.display()));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // p9는 직전 7×3 표가 끝난 뒤 빈 spacer를 거쳐 새 제목이 시작한다. 일반 1×1
    // continuation처럼 첫 가시 unit을 소비하면 제목이 ancestor clip 위(y=100)로
    // 밀려 SVG/Canvas에서 사라진다. 한컴 PDF의 물리 p9처럼 새 viewport 안에
    // `<국내 유사입법례 분석>`이 나타나야 한다.
    let p9 = doc
        .build_page_render_tree(8)
        .expect("issue2007 p9 render tree");
    let p9_clip = Some(ClipRect::from_node(&p9.root));
    let heading = "국내 유사입법례 분석";
    assert!(
        contains_painted_text(&p9.root, heading, p9_clip),
        "p9 completed-table boundary clips the following heading"
    );
    let heading_top = first_text_run_top(&p9.root, heading).expect("p9 continuation heading");
    assert!(
        (125.0..=140.0).contains(&heading_top),
        "p9 heading must start inside the new physical viewport, got y={heading_top}"
    );

    // p9의 8×4 표는 실제 하단선이 wrapper Cell clip 밖으로 4.95px 나가 있었다.
    // node가 존재하는지만 보면 SVG/Canvas에서 선이 완전히 잘린 결함을 놓친다.
    assert_eq!(
        nested_table_bottom_border_is_painted(&p9.root, "조달청", p9_clip),
        Some(true),
        "p9 completed 8×4 table bottom border must survive every enclosing TableCell clip"
    );
}

#[test]
fn issue_2007_continuation_frame_restarts_and_drops_previous_page_residual() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {e}", hwp_path.display()));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // p11 and p15 start in the middle of a dotted 1×1 nested table. Their
    // source top edge is on the preceding page, so SVG/Canvas must receive a
    // new full-width top edge *inside* the continuation viewport instead of
    // retaining a centerline that clipPath/Canvas cuts in half.
    for (page_index, needle) in [
        (
            10,
            "행하여야 하며, 다른 목적 등을 위하여 조사권을 남용하여서는 아니된다.",
        ),
        (14, "금융위원회는 관계자에 대한 조사실적"),
    ] {
        let tree = doc
            .build_page_render_tree(page_index)
            .unwrap_or_else(|e| panic!("issue2007 p{} render tree: {e}", page_index + 1));
        let table = find_innermost_table_containing_text(&tree.root, needle).unwrap_or_else(|| {
            panic!(
                "p{} continuation table containing {needle:?}",
                page_index + 1
            )
        });
        assert!(
            has_direct_full_width_horizontal_line_inside_top_clip(table, 117.0),
            "p{} continuation top frame is absent or still paint-clipped at the viewport edge",
            page_index + 1
        );
    }

    // p10 follows an 8×4 table which ends 4.95px into the new page; p13
    // likewise contains only a 2.5px tail.  Neither is current-page content,
    // so neither previous-page bottom edge may become a false top frame.
    let p10 = doc
        .build_page_render_tree(9)
        .expect("issue2007 p10 render tree");
    assert!(
        !has_visible_full_width_horizontal_line_near(&p10.root, 84.21, 719.16, 117.15),
        "p10 paints the previous fragment's residual bottom border as a false top frame"
    );
    let p10_heading_top = first_text_run_top(&p10.root, "조사기능 관련 타기관 입법례")
        .expect("p10 heading after residual table tail");
    assert!(
        (126.5..=128.5).contains(&p10_heading_top),
        "p10 must remove the previous fragment's retained empty-spacer reservation, got heading y={p10_heading_top}"
    );

    // p10의 새 1×1 frame은 다음 페이지까지 계속되므로 source의 실제 하단선은
    // viewport 밖에 있다. 현재 fragment의 기하 자체를 viewport 안으로 자른 뒤,
    // 한컴 PDF처럼 현재 조각의 하단 frame을 다시 그리되 stroke 전체가 clip 안에
    // 남아야 한다. 과거에는 overflow된 원 table bbox를 전제로 이 합성 frame 경로를
    // 검사했지만, viewport split 이후에는 그 overflow 자체가 회귀다.
    let p10_outer = find_table_fragment(&p10.root, 7, 1).expect("p10 outer pi=7 ci=1 continuation");
    let p10_outer_cell = direct_table_cell(p10_outer).expect("p10 outer direct Cell");
    let p10_inner =
        find_innermost_table_containing_text(&p10.root, "독점규제 및 공정거래에 관한 법률")
            .expect("p10 bordered table below the unbordered RowBreak wrapper");
    let p10_clip_bottom = p10_outer_cell.bbox.y + p10_outer_cell.bbox.height;
    let p10_inner_bottom = p10_inner.bbox.y + p10_inner.bbox.height;
    let p10_page_clip = Some(ClipRect::from_node(&p10.root));
    assert!(
        contains_painted_text(
            &p10.root,
            "조사공무원은 이 법의 시행을 위하여 필요한 최소한의 범위 안에서 조사를",
            p10_page_clip,
        ),
        "p10 must keep the first line of 제50조의2 like the Hancom PDF"
    );
    assert!(
        !contains_painted_text(
            &p10.root,
            "행하여야 하며, 다른 목적 등을 위하여 조사권을 남용하여서는 아니된다",
            p10_page_clip,
        ),
        "p10 must not paint the second line owned by p11"
    );
    assert!(
        p10_inner_bottom <= p10_clip_bottom + 0.75,
        "p10 continuation table must be clipped into its physical fragment: table bottom={:.1}, clip bottom={p10_clip_bottom:.1}",
        p10_inner_bottom
    );
    assert!(
        has_direct_bottom_frame_inside_clip(p10_inner, p10_clip_bottom),
        "p10 continuation frame must paint a full-width bottom edge inside its physical clip"
    );

    let p12 = doc
        .build_page_render_tree(11)
        .expect("issue2007 p12 render tree");
    let (_, election_heading_bottom) =
        first_text_run_vertical_bounds(&p12.root, " 중앙선거관리위원회")
            .expect("p12 중앙선거관리위원회 section heading");
    let election_table = find_innermost_table_containing_text(&p12.root, "공직선거법")
        .expect("p12 중앙선거관리위원회 하위 표");
    let election_heading_gap = election_table.bbox.y - election_heading_bottom;
    assert!(
        (10.2..=12.6).contains(&election_heading_gap),
        "p12 중앙선거관리위원회 제목 뒤의 저장 줄간격 780 HWPUNIT이 소실됐다: \
         heading_bottom={election_heading_bottom:.3}, table_top={:.3}, gap={election_heading_gap:.3}",
        election_table.bbox.y,
    );

    let p13 = doc
        .build_page_render_tree(12)
        .expect("issue2007 p13 render tree");
    assert!(
        !has_visible_full_width_horizontal_line_near(&p13.root, 84.23, 727.47, 119.68),
        "p13 paints the previous fragment's residual bottom border as a false top frame"
    );

    // p12의 빈 separator + 국가인권위원회 제목은 뒤의 1×1 block과 함께 p13으로
    // 넘어가야 한다. 그 결과 p13은 감사원 항목 1에서 끝나고 항목 2는 p14가 소유한다.
    let p13_clip = Some(ClipRect::from_node(&p13.root));
    let human_rights_table = find_innermost_table_containing_text(&p13.root, "국가인권위원회법")
        .expect("p13 국가인권위원회 하위 표");
    let human_rights_first_top =
        first_text_run_top(&p13.root, "국가인권위원회법").expect("p13 국가인권위원회 표 첫 줄");
    assert!(
        (7.0..=8.1).contains(&(human_rights_first_top - human_rights_table.bbox.y)),
        "p13 정상 신규 표의 top inset이 p14 보정에 영향받았다: \
         table_top={:.3}, line_top={human_rights_first_top:.3}",
        human_rights_table.bbox.y,
    );
    let audit_item_2 = "증명서, 변명서, 그 밖의 관계 문서 및 장부, 물품 등의 제출 요구";
    assert!(
        !contains_painted_text(&p13.root, audit_item_2, p13_clip),
        "p13 must not paint the p14-owned 감사원 item 2"
    );

    let p14 = doc
        .build_page_render_tree(13)
        .expect("issue2007 p14 render tree");
    let p14_clip = Some(ClipRect::from_node(&p14.root));
    assert!(
        contains_painted_text(&p14.root, audit_item_2, p14_clip),
        "p14 must begin with the carried 감사원 item 2"
    );
    let finance_heading = "자본시장과 금융투자업에 관한 법률";
    let finance_item_8 = "금융위원회는 관계자에 대한 조사실적";
    assert!(
        contains_painted_text(&p14.root, finance_heading, p14_clip),
        "p14 must own the 금융위원회 heading"
    );
    let audit_table =
        find_innermost_table_containing_text(&p14.root, "감사원법").expect("p14 감사원법 하위 표");
    let audit_first_top =
        first_text_run_top(&p14.root, "증명서, 변명서").expect("p14 감사원법 continuation 첫 줄");
    let (_, audit_last_bottom) = first_text_run_vertical_bounds(&p14.root, "위반한 자는")
        .expect("p14 감사원법 마지막 벌칙 줄");
    let audit_table_bottom = audit_table.bbox.y + audit_table.bbox.height;
    let finance_table = find_innermost_table_containing_text(&p14.root, finance_heading)
        .expect("p14 금융위원회 하위 표");
    let finance_section_heading_top =
        first_text_run_top(&p14.root, " 금융위원회").expect("p14 금융위원회 section heading");
    assert!(
        (3.8..=6.2).contains(&(audit_first_top - audit_table.bbox.y)),
        "p14 감사원법 continuation 첫 줄이 PDF의 top inset과 다르다: \
         table_top={:.3}, line_top={audit_first_top:.3}",
        audit_table.bbox.y,
    );
    assert!(
        (4.5..=6.8).contains(&(audit_table_bottom - audit_last_bottom)),
        "p14 감사원법 마지막 줄 뒤에 빈 terminal tail이 남았다: \
         line_bottom={audit_last_bottom:.3}, table_bottom={audit_table_bottom:.3}"
    );
    assert!(
        (22.5..=26.0).contains(&(finance_section_heading_top - audit_table_bottom)),
        "p14 감사원법 표와 금융위원회 제목 사이 간격이 PDF와 다르다: \
         table_bottom={audit_table_bottom:.3}, heading_top={finance_section_heading_top:.3}"
    );
    assert!(
        (496.0..=499.0).contains(&finance_section_heading_top)
            && (523.5..=527.0).contains(&finance_table.bbox.y),
        "p14 중간 block 절대 좌표가 PDF와 다르다: \
         heading_top={finance_section_heading_top:.3}, table_top={:.3}",
        finance_table.bbox.y,
    );
    let heading_to_table = finance_table.bbox.y - finance_section_heading_top;
    assert!(
        (27.6..=27.9).contains(&heading_to_table),
        "p14 금융위원회 제목 뒤의 저장 줄간격 780 HWPUNIT이 소실됐다: \
         heading_top={finance_section_heading_top:.3}, table_top={:.3}, delta={heading_to_table:.3}",
        finance_table.bbox.y,
    );
    let mut finance_item_7_tail = Vec::new();
    collect_exact_text_line_clips_in_subtree(
        &p14.root,
        finance_table,
        "한다.",
        p14_clip,
        false,
        &mut finance_item_7_tail,
    );
    assert_eq!(
        finance_item_7_tail.len(),
        1,
        "p14 금융위원회 항목 7의 마지막 TextLine `한다.`는 해당 하위 표 안에 정확히 하나여야 한다"
    );
    let (tail_line, effective_clip) = finance_item_7_tail[0];
    let effective_clip =
        effective_clip.expect("p14 금융위원회 항목 7에 유효한 조상 TableCell clip이 있어야 한다");
    assert!(
        effective_clip.fully_contains(tail_line),
        "p14 금융위원회 항목 7 마지막 줄이 조상 TableCell clip에 잘린다: \
         line_bottom={:.3}, clip_bottom={:.3}",
        tail_line.bottom,
        effective_clip.bottom,
    );
    assert!(
        !contains_painted_text(&p14.root, finance_item_8, p14_clip),
        "p14 must stop at the stored frame break before 금융위원회 item 8"
    );

    let p15 = doc
        .build_page_render_tree(14)
        .expect("issue2007 p15 render tree");
    let p15_clip = Some(ClipRect::from_node(&p15.root));
    assert!(
        contains_painted_text(&p15.root, finance_item_8, p15_clip),
        "p15 must begin with the carried 금융위원회 item 8"
    );
    let finance_continuation_table =
        find_innermost_table_containing_text(&p15.root, finance_item_8)
            .expect("p15 금융위원회 continuation 표");
    let finance_item_8_top =
        first_text_run_top(&p15.root, finance_item_8).expect("p15 금융위원회 continuation 첫 줄");
    assert!(
        (1.5..=2.3).contains(&(finance_item_8_top - finance_continuation_table.bbox.y)),
        "p15 정상 recursive continuation top inset이 p14 보정에 영향받았다: \
         table_top={:.3}, line_top={finance_item_8_top:.3}",
        finance_continuation_table.bbox.y,
    );
    let (_, procurement_heading_bottom) =
        first_text_run_vertical_bounds(&p15.root, " 조달청").expect("p15 조달청 section heading");
    let procurement_table = find_innermost_table_containing_text(&p15.root, "조달사업에 관한 법률")
        .expect("p15 조달청 하위 표");
    let procurement_heading_gap = procurement_table.bbox.y - procurement_heading_bottom;
    assert!(
        (10.2..=12.6).contains(&procurement_heading_gap),
        "p15 조달청 제목 뒤의 저장 줄간격 780 HWPUNIT이 소실됐다: \
         heading_bottom={procurement_heading_bottom:.3}, table_top={:.3}, gap={procurement_heading_gap:.3}",
        procurement_table.bbox.y,
    );
    assert!(
        !contains_painted_text(&p15.root, finance_heading, p15_clip),
        "p15 must not repeat the p14-owned 금융위원회 heading"
    );
    assert!(
        contains_painted_text(&p15.root, "제기할 수 있다.", p15_clip),
        "p15 recursive viewport must retain the final 조달청 line inside the cell clip"
    );
}

#[test]
fn issue_2007_cell_vpos_reset_does_not_overlap_following_paragraphs() {
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
    let line_bottom =
        svg_number_attr(bottom_line, "y1") + svg_number_attr(bottom_line, "stroke-width");
    assert!(
        clip_bottom + 0.01 >= line_bottom,
        "SVG bottom stroke가 outer cell clip에 잘린다: line_bottom={line_bottom:.3}, clip_bottom={clip_bottom:.3}\n{outer_clip}\n{bottom_line}"
    );
}

#[test]
fn issue_2007_continuation_viewport_does_not_center_nested_cell_content() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue2007_nested_cell_pagination_42065.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue2007_nested_cell_pagination_42065.hwp");

    // 한컴 PDF p10에서는 "독점규제 …"가 1×1 하위 표의 첫 줄로 상단 경계 바로
    // 뒤에 온다. 종전 rhwp는 부모 RowBreak continuation의 clip window를 모른 채
    // 이 하위 Center 셀을 원본 1,296px 높이에서 다시 중앙 정렬해 약 250px 빈
    // 영역을 만들었다. 이 위치 계약은 단순 TextLine overlap=0으로는 검출되지 않는다.
    let tree = doc
        .build_page_render_tree(9)
        .expect("issue2007 p10 render tree");
    let needle = "독점규제 및 공정거래에 관한 법률";
    let table = find_innermost_table_containing_text(&tree.root, needle)
        .expect("p10 nested table containing the 공정거래 law heading");
    let text_top = first_text_run_top(table, needle).expect("p10 공정거래 law heading text run");
    assert!(
        text_top <= table.bbox.y + 40.0,
        "p10 nested continuation viewport centered its first line instead of starting at the visible table top: \
         table_y={:.1}, text_y={text_top:.1}",
        table.bbox.y,
    );

    // p11은 같은 1×1 셀의 다음 viewport다. 셀의 위쪽이 parent viewport에 의해
    // 잘렸는데도 원본 1,296px 전체 높이를 Center 기준으로 쓰면, p10에서 이미
    // paint한 "사용목적" 문단군을 이 페이지 상단에 다시 끌어온다. p11 PDF는 그
    // 다음 문장("행하여야 하며 …")부터 시작한다. render tree에 clip 밖의 source
    // text가 남는 것은 허용하지만 SVG/Canvas가 실제 paint하면 안 된다.
    let p11 = doc
        .build_page_render_tree(10)
        .expect("issue2007 p11 render tree");
    let p11_clip = Some(ClipRect::from_node(&p11.root));
    assert!(
        !contains_painted_text(&p11.root, "사용목적", p11_clip),
        "p11 replays the p10-owned 사용목적 paragraph because an upper-clipped nested cell is centered"
    );
    assert!(
        contains_painted_text(&p11.root, "행하여야 하며, 다른 목적", p11_clip),
        "p11 must begin from its own visible continuation text after the p10-owned paragraph"
    );
    let p11_continuation_top = first_text_run_top(&p11.root, "행하여야 하며, 다른 목적")
        .expect("p11 first visible continuation line");
    assert!(
        (117.0..=140.0).contains(&p11_continuation_top),
        "p11 first continuation line must be positioned inside the physical nested-cell clip, not above it: {p11_continuation_top}"
    );

    // 마지막 조각도 첫 visible unit의 reservation을 다시 content origin에 남기면
    // 제목이 한 단위(32px) 아래로 내려간다. PDF p17의 첫 제목은 body top 직후에
    // 있으므로 terminal 여부와 무관하게 같은 보정을 적용해야 한다.
    let p17 = doc
        .build_page_render_tree(16)
        .expect("issue2007 p17 render tree");
    let terminal_table = find_innermost_table_containing_text(&p17.root, "선호된 대안의 기대효과")
        .expect("p17 nested table containing terminal heading");
    let terminal_text_top = first_text_run_top(terminal_table, "선호된 대안의 기대효과")
        .expect("p17 terminal heading text run");
    assert!(
        terminal_text_top <= terminal_table.bbox.y + 14.0,
        "p17 terminal continuation retained the first visible-unit reservation: \
         table_y={:.1}, text_y={terminal_text_top:.1}",
        terminal_table.bbox.y,
    );
}
