//! 표 레이아웃 (layout_table + 셀 높이/줄범위 계산)

use super::super::composer::{compose_paragraph, ComposedLine, ComposedParagraph};
use super::super::height_measurer::{
    fit_measured_table_nested_tail_to_declared_height, MeasuredTable,
};
use super::super::page_layout::LayoutRect;
use super::super::render_tree::*;
use super::super::style_resolver::{ResolvedBorderStyle, ResolvedStyleSet};
use crate::model::bin_data::BinDataContent;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::style::{Alignment, BorderLine, CenterLine};
use crate::model::table::{TablePageBreak, VerticalAlign};
use crate::renderer::float_placement::signed_hwpunit;

const ROWBREAK_OBJECT_BOTTOM_BLEED_TOLERANCE_PX: f64 = 64.0;
/// [#3738 Stage 19] native HWP5가 빈 1×1 RowBreak picture table에 남기는 stale page
/// origin은 일반적인 local offset보다 한 페이지 단위로 크다. 이 값보다 작은 음수는
/// 일반 그림 위치일 수 있으므로 절대 보정하지 않는다.
const ROWBREAK_STALE_PAGE_SCALE_PICTURE_OFFSET_MIN_HU: i32 = -40_000;

/// [#2424 프로파일] 분할 표 컷 프리미티브 실측 카운터 — `RHWP_2424_PROFILE` 전용, 동작 불변.
/// 프로세스 누적이며 `RHWP_2424_STEP_PROFILE` 출력(typeset.rs)이 스냅샷을 읽는다.
pub(crate) static ISSUE2424_ADVANCE_ROW_CUT_CALLS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
pub(crate) static ISSUE2424_ADVANCE_ROW_CUT_NANOS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
pub(crate) static ISSUE2424_CELL_UNITS_HITS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
pub(crate) static ISSUE2424_CELL_UNITS_MISSES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
pub(crate) static ISSUE2424_CELL_UNITS_MISS_NANOS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// [#2424 프로파일] env 게이트 1회 판정. wasm 은 항상 false 라 `Instant::now` 가
/// 호출되지 않는다 (`paginate_pass` 의 게이트 패턴과 동일 규약).
pub(crate) fn issue2424_profile_enabled() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        false
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ENABLED.get_or_init(|| {
            std::env::var("RHWP_2424_PROFILE").is_ok_and(|value| !value.is_empty() && value != "0")
        })
    }
}

/// [Task #548] paragraph 의 line N 에 적용되는 effective margin_left.
/// paragraph_layout.rs 의 line_indent 산식과 동일 (단일 룰).
/// - positive indent: line 0 에만 +indent 적용 (첫줄 들여쓰기)
/// - negative indent (hanging): line N≥1 에 +|indent| 적용
/// - indent=0: 모든 line 에 margin_left 만 적용
pub(super) fn effective_margin_left_line(margin_left: f64, indent: f64, line_n: usize) -> f64 {
    let line_indent = if indent > 0.0 {
        if line_n == 0 {
            indent
        } else {
            0.0
        }
    } else if indent < 0.0 {
        if line_n == 0 {
            0.0
        } else {
            indent.abs()
        }
    } else {
        0.0
    };
    margin_left + line_indent
}

fn cell_para_line_anchor_y(
    base_y: f64,
    content_cell_y: f64,
    pad_top: f64,
    vertical_pos_hu: i32,
    dpi: f64,
    use_top_vpos_anchor: bool,
    upper_clip_line_reservation: f64,
) -> f64 {
    if use_top_vpos_anchor {
        // Top/vpos 앵커는 평상시 `base_y`의 vertical-align offset을 의도적으로
        // 무시한다. 단, RowBreak continuation의 page-clip 보정은 바로 이
        // absolute vpos 경로에도 적용해야 한다. 그렇지 않으면 text_y_start에
        // 예약값을 더해도 실제 문단은 종전 y에 남아 첫 줄이 clip 밖에서 사라진다.
        content_cell_y + pad_top + hwpunit_to_px(vertical_pos_hu, dpi) + upper_clip_line_reservation
    } else {
        base_y + hwpunit_to_px(vertical_pos_hu, dpi)
    }
}

fn has_initial_tac_shape_host(paragraphs: &[Paragraph]) -> bool {
    paragraphs.first().is_some_and(|para| {
        para.text.trim().is_empty()
            && para
                .controls
                .iter()
                .any(|ctrl| matches!(ctrl, Control::Shape(shape) if shape.common().treat_as_char))
    })
}

/// native HWP5와 original HWPX의 빈 RowBreak 그림 표가 fresh page로 이월된 뒤에도, 내부 picture가
/// 이월 전 outer host 좌표를 상쇄하지 않도록 하는 정확한 형상 판정이다.
///
/// `host_stored_vpos_hu`는 table의 소유 문단에서만 얻을 수 있으며 셀 paragraph의
/// vpos와 다르다. 이 값을 table-cell 경로까지 명시적으로 전달해, page-scale 음수
/// picture offset이 의도한 일반 음수 위치인지와 page boundary 상쇄인지 구분한다.
fn stored_layout_relocated_empty_rowbreak_picture_resets_offset(
    stored_layout: bool,
    native_hwp5_layout: bool,
    host_stored_vpos_hu: Option<i32>,
    table: &crate::model::table::Table,
    cell: &crate::model::table::Cell,
    para: &Paragraph,
    picture: &crate::model::image::Picture,
) -> bool {
    let Some(host_vpos) = host_stored_vpos_hu else {
        return false;
    };
    if !stored_layout
        || host_vpos <= 0
        || table.page_break != TablePageBreak::RowBreak
        || table.common.treat_as_char
        || !matches!(table.common.text_wrap, TextWrap::TopAndBottom)
        || !matches!(table.common.vert_rel_to, VertRelTo::Para)
        || table.row_count != 1
        || table.col_count != 1
        || table.cells.len() != 1
        || cell.row != 0
        || cell.col != 0
        || cell.row_span != 1
        || cell.col_span != 1
        || cell.paragraphs.len() != 1
        || !para.text.trim().is_empty()
        || para.controls.len() != 1
        || para.line_segs.len() != 1
        || para.line_segs[0].vertical_pos != 0
        || !matches!(picture.common.text_wrap, TextWrap::TopAndBottom)
        || picture.common.treat_as_char
        || !picture.common.flow_with_text
        || !matches!(picture.common.vert_rel_to, VertRelTo::Para)
    {
        return false;
    }

    let table_offset = signed_hwpunit(table.common.vertical_offset);
    let picture_offset = signed_hwpunit(picture.common.vertical_offset);
    let relocated_page_ladder = table_offset > 0
        && picture_offset < 0
        && (host_vpos as i64 + table_offset as i64 + picture_offset as i64).abs() <= 8;
    // HWP p25 (pi=357)는 table vOffset=0인데 picture에만 stale -50000 HU가 남았다.
    // 이는 다음 쪽 ladder가 아니라 같은 물리 쪽의 page-scale stale origin이다. HWPX
    // stored-layout에는 이 HWP5 직렬화 서명이 없으므로 native 경로에만 한정한다.
    let same_page_stale_hwp5_picture = native_hwp5_layout
        && table_offset == 0
        && picture_offset <= ROWBREAK_STALE_PAGE_SCALE_PICTURE_OFFSET_MIN_HU;
    relocated_page_ladder || same_page_stale_hwp5_picture
}

use super::super::composer::effective_text_for_metrics;
use super::super::{hwpunit_to_px, ShapeStyle};
use super::border_rendering::{
    build_row_col_x, collect_cell_borders, create_border_line_nodes, render_cell_diagonal,
    render_edge_borders, render_transparent_borders,
};
use super::text_measurement::{estimate_text_width, resolved_to_text_style};
use super::utils::find_bin_data_bytes;
use super::{CellContext, CellPathEntry, LayoutEngine};

// 표 수평 정렬: model::shape 타입 사용
use crate::model::shape::{
    Caption, CaptionDirection, CommonObjAttr, HorzAlign, HorzRelTo, TextWrap, VertRelTo,
};

fn caption_has_topbottom_picture(caption: &Caption) -> bool {
    caption.paragraphs.iter().any(|para| {
        para.controls.iter().any(|ctrl| {
            matches!(
                ctrl,
                Control::Picture(pic) if matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
            )
        })
    })
}

/// A clipped table cell still has to expose an immediately nested table's
/// *outer border*. A nested table can begin after the host cell's left padding
/// while retaining its stored width, which puts that right border just beyond
/// the host cell's logical content rectangle. A completed nested table can
/// likewise end one border-width below an ancestor wrapper clip. Clipping at
/// the logical rectangle then removes the entire border even though the table
/// layout emitted it (issue2007 p2-p4, p9).
///
/// Do not expand to every descendant: a RowBreak continuation deliberately
/// keeps future-page text below its physical cell clip. This is restricted to
/// direct nested `Table` outer vertical `Line`s. When the outer clip expands,
/// direct nested `TableCell` content remains bounded by the host's original
/// horizontal viewport so the border exception cannot reveal a text tail.
/// The vertical correction is separately bounded to a terminal border that
/// misses the clip by at most six pixels.
fn extend_clipped_cell_horizontal_clip_to_nested_table_borders(cell_node: &mut RenderNode) {
    let RenderNodeType::TableCell(cell_meta) = &cell_node.node_type else {
        return;
    };
    if !cell_meta.clip {
        return;
    }

    let host_clip_left = cell_node.bbox.x;
    let host_clip_right = cell_node.bbox.x + cell_node.bbox.width;
    let mut clip_left = host_clip_left;
    let mut clip_right = host_clip_right;

    for table_node in &mut cell_node.children {
        if !matches!(table_node.node_type, RenderNodeType::Table(_)) {
            continue;
        }
        let table_left = table_node.bbox.x;
        let table_right = table_node.bbox.x + table_node.bbox.width;
        let mut found_outer_vertical_border = false;

        for border_node in &table_node.children {
            let RenderNodeType::Line(line) = &border_node.node_type else {
                continue;
            };
            // Cell-content lines can be arbitrary.  Only a near-vertical
            // table edge that sits on the nested table's left/right boundary
            // is eligible to enlarge the clipping viewport.
            if (line.x1 - line.x2).abs() > 0.01 || (line.y1 - line.y2).abs() < 1.0 {
                continue;
            }
            let x = line.x1;
            let outer_edge_tolerance = (line.style.width + 1.0).max(2.0);
            if (x - table_left).abs() > outer_edge_tolerance
                && (x - table_right).abs() > outer_edge_tolerance
            {
                continue;
            }
            let half_stroke = line.style.width / 2.0;
            clip_left = clip_left.min(x - half_stroke);
            clip_right = clip_right.max(x + half_stroke);
            found_outer_vertical_border = true;
        }

        if !found_outer_vertical_border {
            // 일부 normal/partial 표는 현재 부모 subtree가 최종 edge `Line`을
            // 붙이기 전에도 직접 child Table bbox를 완성한다(42065 p2-p3). 이
            // bbox는 table의 물리 stored-width 경계이므로 작은 stroke 여유만
            // 포함해 가로 clip의 fallback으로 쓸 수 있다. 세로 bbox는 전혀
            // 확장하지 않아 다음 쪽 continuation tail은 계속 가려진다.
            const FALLBACK_BORDER_HALF_STROKE_PX: f64 = 1.0;
            clip_left = clip_left.min(table_left - FALLBACK_BORDER_HALF_STROKE_PX);
            clip_right = clip_right.max(table_right + FALLBACK_BORDER_HALF_STROKE_PX);
        }

        if table_left < host_clip_left - NESTED_FRAGMENT_EDGE_EPSILON_PX
            || table_right > host_clip_right + NESTED_FRAGMENT_EDGE_EPSILON_PX
        {
            // Keep the direct child table's frame visible through the expanded
            // host clip, but never let its cell content paint past the parent
            // grid edge. The table's `Line` children stay outside this clamp.
            for nested_cell in &mut table_node.children {
                let RenderNodeType::TableCell(nested_meta) = &nested_cell.node_type else {
                    continue;
                };
                if !nested_meta.clip {
                    continue;
                }
                let content_left = nested_cell.bbox.x.max(host_clip_left);
                let content_right = (nested_cell.bbox.x + nested_cell.bbox.width)
                    .min(host_clip_right)
                    .max(content_left);
                nested_cell.bbox.x = content_left;
                nested_cell.bbox.width = content_right - content_left;
            }
        }
    }

    cell_node.bbox.x = clip_left;
    cell_node.bbox.width = (clip_right - clip_left).max(0.0);
}

/// A terminal table border may be just outside a *wrapper ancestor* clip even
/// though its direct host cell contains it. This is distinct from a real
/// continuation: allowing an arbitrary descendant's vertical extent would
/// reveal future-page text, but a completed outer horizontal border within a
/// few pixels is paint-only. Preserve exactly that stroke interval (42065 p9).
const NESTED_COMPLETED_BORDER_CLIP_OVERFLOW_PX: f64 = 6.0;

fn extend_clipped_cell_vertical_clip_to_nearby_nested_table_borders(cell_node: &mut RenderNode) {
    let RenderNodeType::TableCell(cell_meta) = &cell_node.node_type else {
        return;
    };
    if !cell_meta.clip {
        return;
    }

    fn scan_table_borders(
        node: &RenderNode,
        clip_top: f64,
        clip_bottom: f64,
        extended_top: &mut f64,
        extended_bottom: &mut f64,
    ) {
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            let table_left = node.bbox.x;
            let table_right = table_left + node.bbox.width;
            let table_top = node.bbox.y;
            let table_bottom = table_top + node.bbox.height;
            for child in &node.children {
                let RenderNodeType::Line(line) = &child.node_type else {
                    continue;
                };
                if (line.y1 - line.y2).abs() > NESTED_FRAGMENT_EDGE_EPSILON_PX
                    || (line.x1.min(line.x2) - table_left).abs() > NESTED_FRAGMENT_EDGE_EPSILON_PX
                    || (line.x1.max(line.x2) - table_right).abs() > NESTED_FRAGMENT_EDGE_EPSILON_PX
                {
                    continue;
                }
                let is_outer_top = (line.y1 - table_top).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX;
                let is_outer_bottom =
                    (line.y1 - table_bottom).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX;
                if !is_outer_top && !is_outer_bottom {
                    continue;
                }
                let half_stroke = line.style.width.max(NESTED_FRAGMENT_EDGE_EPSILON_PX) / 2.0;
                let paint_top = line.y1 - half_stroke;
                let paint_bottom = line.y1 + half_stroke;
                if paint_top < clip_top
                    && clip_top - paint_top <= NESTED_COMPLETED_BORDER_CLIP_OVERFLOW_PX
                {
                    *extended_top =
                        (*extended_top).min(paint_top - NESTED_FRAGMENT_FRAME_INSET_EPSILON_PX);
                }
                if paint_bottom > clip_bottom
                    && paint_bottom - clip_bottom <= NESTED_COMPLETED_BORDER_CLIP_OVERFLOW_PX
                {
                    *extended_bottom = (*extended_bottom)
                        .max(paint_bottom + NESTED_FRAGMENT_FRAME_INSET_EPSILON_PX);
                }
            }
        }
        for child in &node.children {
            scan_table_borders(child, clip_top, clip_bottom, extended_top, extended_bottom);
        }
    }

    let clip_top = cell_node.bbox.y;
    let clip_bottom = clip_top + cell_node.bbox.height;
    let mut extended_top = clip_top;
    let mut extended_bottom = clip_bottom;
    for child in &cell_node.children {
        scan_table_borders(
            child,
            clip_top,
            clip_bottom,
            &mut extended_top,
            &mut extended_bottom,
        );
    }
    cell_node.bbox.y = extended_top;
    cell_node.bbox.height = (extended_bottom - extended_top).max(0.0);
}

/// A table's logical stored width can end just inside a direct cell whose
/// horizontal paint extent was widened for a nested table border.  Preserve
/// that direct-cell extent on the table node before its parent cell computes
/// the next clip.  Without the post-order propagation, p10-p16's 1x1
/// continuation chain stops at the first table: the grandchild right edge is
/// correct, but the outer Cell and Body clips still cut it away.
///
/// Only direct `TableCell` children participate.  This deliberately does not
/// union arbitrary descendants (which could include next-page continuation
/// tails); it forwards the same horizontal paint boundary that the immediate
/// table already owns.
fn extend_table_horizontal_bbox_to_direct_cell_paint(table_node: &mut RenderNode) {
    if !matches!(table_node.node_type, RenderNodeType::Table(_)) {
        return;
    }

    let mut left = table_node.bbox.x;
    let mut right = table_node.bbox.x + table_node.bbox.width;
    for child in &table_node.children {
        if matches!(child.node_type, RenderNodeType::TableCell(_)) {
            left = left.min(child.bbox.x);
            right = right.max(child.bbox.x + child.bbox.width);
        }
    }
    table_node.bbox.x = left;
    table_node.bbox.width = (right - left).max(0.0);
}

const NESTED_FRAGMENT_EDGE_EPSILON_PX: f64 = 0.5;
/// A table that leaks less than this distance into a clipped continuation cell
/// is the terminal border of the previous fragment, not content for this page.
/// Keeping it paints a stray horizontal line at the next page's top (42065
/// p10/p13), while the corresponding text is already correctly clipped away.
///
/// A native 1px border may be rasterized just below the logical clip by up to
/// roughly 5px at the renderer's layout scale.  Six pixels remains below the
/// smallest real continuation fragment in this fixture, so it only suppresses
/// that paint residue rather than current-page table content.
const NESTED_FRAGMENT_RESIDUAL_BORDER_PX: f64 = 6.0;

/// SVG and Canvas both clip a stroke by its painted area, rather than by the
/// centerline.  Keep a reconstructed frame's whole stroke a hair inside the
/// viewport: a centreline exactly on the clip boundary loses its anti-aliased
/// outer half, and can disappear entirely at a fractional device scale.
const NESTED_FRAGMENT_FRAME_INSET_EPSILON_PX: f64 = 0.05;
const NESTED_FRAGMENT_FRAME_TARGET_EPSILON_PX: f64 = 0.05;

fn push_fragment_border_line(
    tree: &mut PageLayoutContext,
    table_node: &mut RenderNode,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    style: crate::renderer::LineStyle,
) {
    let width = style.width.max(NESTED_FRAGMENT_EDGE_EPSILON_PX);
    let bbox = BoundingBox::new(
        x1.min(x2),
        y1.min(y2),
        (x2 - x1).abs().max(width),
        (y2 - y1).abs().max(width),
    );
    table_node.children.push(RenderNode::new(
        tree.next_id(),
        RenderNodeType::Line(LineNode::new(x1, y1, x2, y2, style)),
        bbox,
    ));
}

fn has_fragment_border_line(table_node: &RenderNode, x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
    table_node.children.iter().any(|child| {
        matches!(
            &child.node_type,
            RenderNodeType::Line(line)
                if (line.x1 - x1).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                    && (line.y1 - y1).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                    && (line.x2 - x2).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                    && (line.y2 - y2).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
        )
    })
}

/// Return the reconstructed frame coordinate whose full painted stroke stays
/// inside the active clip. `at_top` selects the clip's upper or lower edge.
fn fragment_horizontal_frame_y(
    clip_top: f64,
    clip_bottom: f64,
    style: &crate::renderer::LineStyle,
    at_top: bool,
) -> f64 {
    let inset = style.width.max(NESTED_FRAGMENT_EDGE_EPSILON_PX) / 2.0
        + NESTED_FRAGMENT_FRAME_INSET_EPSILON_PX;
    if at_top {
        clip_top + inset
    } else {
        clip_bottom - inset
    }
}

/// Only the borderless 1×1 RowBreak continuation contract needs a synthetic
/// bottom edge at a physical page clip.  A multi-cell nested table owns real
/// row boundaries on later fragments; turning a purely geometric
/// `ends_after_clip` result into a full-width rule exposes a premature terminal
/// border on the preceding page (#4159, issue2007 p2).
fn reconstructs_clipped_fragment_bottom(table_node: &RenderNode) -> bool {
    matches!(
        &table_node.node_type,
        RenderNodeType::Table(TableNode {
            row_count: 1,
            col_count: 1,
            ..
        })
    )
}

/// Make one full-width fragment edge paint-safe without producing a double
/// rule. Native table layout commonly leaves the source border exactly on the
/// clip edge.  The old broad `has_fragment_border_line` tolerance treated that
/// clipped source line as equivalent to the reconstructed one, so no usable
/// frame was emitted (issue2007 p11/p14).  Prefer moving that exact source
/// line inward; add a line only when the source did not retain one.
fn ensure_fragment_horizontal_frame_inside_clip(
    tree: &mut PageLayoutContext,
    table_node: &mut RenderNode,
    table_left: f64,
    table_right: f64,
    clip_edge_y: f64,
    frame_y: f64,
    style: crate::renderer::LineStyle,
) {
    let has_target = table_node.children.iter().any(|child| {
        matches!(
            &child.node_type,
            RenderNodeType::Line(line)
                if child.visible
                    && (line.y1 - line.y2).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                    && (line.x1.min(line.x2) - table_left).abs()
                        <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                    && (line.x1.max(line.x2) - table_right).abs()
                        <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                    && (line.y1 - frame_y).abs()
                        <= NESTED_FRAGMENT_FRAME_TARGET_EPSILON_PX
        )
    });
    if has_target {
        return;
    }

    if let Some(source_line) = table_node.children.iter_mut().find(|child| {
        matches!(
            &child.node_type,
            RenderNodeType::Line(line)
                if child.visible
                    && (line.y1 - line.y2).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                    && (line.x1.min(line.x2) - table_left).abs()
                        <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                    && (line.x1.max(line.x2) - table_right).abs()
                        <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                    && (line.y1 - clip_edge_y).abs()
                        <= NESTED_FRAGMENT_EDGE_EPSILON_PX
        )
    }) {
        let delta_y = frame_y - source_line.bbox.y;
        if let RenderNodeType::Line(line) = &mut source_line.node_type {
            line.y1 += delta_y;
            line.y2 += delta_y;
        }
        source_line.bbox.y += delta_y;
        return;
    }

    push_fragment_border_line(
        tree,
        table_node,
        table_left,
        frame_y,
        table_right,
        frame_y,
        style,
    );
}

fn translate_render_subtree_y(node: &mut RenderNode, delta_y: f64) {
    node.bbox.y += delta_y;
    if let RenderNodeType::Line(line) = &mut node.node_type {
        line.y1 += delta_y;
        line.y2 += delta_y;
    }
    for child in &mut node.children {
        translate_render_subtree_y(child, delta_y);
    }
}

/// A `vpos=0` line with no text is the explicit empty spacer stored between
/// a completed nested table and its following source block.
fn is_empty_vpos_spacer_line(node: &RenderNode) -> bool {
    matches!(&node.node_type, RenderNodeType::TextLine(line) if line.vpos == Some(0))
        && node.children.iter().all(|child| {
            matches!(&child.node_type, RenderNodeType::TextRun(run) if run.text.trim().is_empty())
        })
}

/// A non-empty direct text line immediately followed by a nested table is a
/// heading/table group, not independent bottom-of-page prose.  HWP5 can keep
/// the heading in the preceding RowBreak fragment while putting the table's
/// first usable row only in the next fragment.  That paints the heading twice:
/// once at the prior page bottom and once above the next cell clip
/// (issue2007 p7--p8).  Keep the group together at the actual viewport that
/// can paint the table.
const NESTED_HEADING_WITH_TABLE_MAX_GAP_PX: f64 = 32.0;
const NESTED_HEADING_WITH_TABLE_TOP_INSET_PX: f64 = 4.0;

/// A text bbox is the layout line box.  Canvas glyph ink can extend slightly
/// above it, so the first line of a clipped cell needs a small paint-safe
/// inset rather than a centreline exactly on the clip boundary.
const CLIPPED_TEXT_INK_TOP_OVERFLOW_PX: f64 = 4.0;
const CLIPPED_TEXT_INK_TOP_INSET_PX: f64 = 0.25;

fn text_line_has_non_whitespace_text(node: &RenderNode) -> bool {
    matches!(node.node_type, RenderNodeType::TextLine(_))
        && node.children.iter().any(|child| {
            matches!(&child.node_type, RenderNodeType::TextRun(run) if !run.text.trim().is_empty())
        })
}

/// A complete source line belongs to the next RowBreak fragment when only a
/// sub-glyph sliver reaches the current clipped cell.  SVG/Canvas would still
/// paint that sliver because clipping works on ink geometry, producing a
/// duplicate heading at the preceding page bottom (issue2007 p16 -> p17).
///
/// Limit this to the same six-pixel residue window used for a future table
/// border: it never hides a usable line, and the successor fragment remains
/// responsible for the full line.
fn suppress_bottom_clipped_text_residue(node: &mut RenderNode, clip_bottom: f64) {
    for child in &mut node.children {
        if !child.visible || !text_line_has_non_whitespace_text(child) {
            continue;
        }
        let line_top = child.bbox.y;
        let line_bottom = line_top + child.bbox.height;
        let visible_sliver = clip_bottom - line_top;
        if line_top < clip_bottom - NESTED_FRAGMENT_EDGE_EPSILON_PX
            && line_bottom > clip_bottom + NESTED_FRAGMENT_EDGE_EPSILON_PX
            && visible_sliver < NESTED_FRAGMENT_RESIDUAL_BORDER_PX
        {
            child.visible = false;
        }
    }
}

/// A continued nested-cell line can retain its prior-page absolute y while a
/// one-pixel ink tail reaches this physical cell.  The outer page clip then
/// hides the full line even though the source owner is this fragment
/// (`rowbreak-problem-pages.hwpx` p8).  Rebase only that crossing line to the
/// cell top; older, wholly off-page sibling lines remain hidden.
///
/// This is deliberately independent of the cell's `clip` flag.  A nested
/// table can inherit its effective viewport from an ancestor RowBreak cell,
/// leaving the inner cell itself unclipped in the render tree.
fn rebase_nested_cell_top_residue_line(node: &mut RenderNode) {
    if !matches!(node.node_type, RenderNodeType::TableCell(_)) {
        return;
    }

    let cell_top = node.bbox.y;
    let has_wholly_off_top_predecessor = node.children.iter().any(|child| {
        child.visible
            && text_line_has_non_whitespace_text(child)
            && child.bbox.y + child.bbox.height < cell_top - NESTED_FRAGMENT_EDGE_EPSILON_PX
    });
    if !has_wholly_off_top_predecessor {
        return;
    }

    for child in &mut node.children {
        if !child.visible || !text_line_has_non_whitespace_text(child) {
            continue;
        }
        let line_top = child.bbox.y;
        let line_bottom = line_top + child.bbox.height;
        if line_top < cell_top
            && cell_top - line_top <= CLIPPED_TEXT_INK_TOP_OVERFLOW_PX * 4.0
            && line_bottom >= cell_top - NESTED_FRAGMENT_EDGE_EPSILON_PX
        {
            translate_render_subtree_y(child, cell_top + CLIPPED_TEXT_INK_TOP_INSET_PX - line_top);
        }
    }
}

/// Preserve source ownership at a clipped cell's title/table seam and keep a
/// first visible glyph out of the ancestor Canvas/SVG clip.
///
/// This operates only on direct source siblings.  It never grows the clip:
/// prior-page text remains hidden and a future-page tail cannot be exposed.
fn repair_clipped_cell_text_table_seam(node: &mut RenderNode, suppress_bottom_text_residue: bool) {
    let is_clipped_cell = matches!(
        &node.node_type,
        RenderNodeType::TableCell(TableCellNode { clip: true, .. })
    );
    if !is_clipped_cell {
        return;
    }

    let clip_top = node.bbox.y;
    let clip_bottom = clip_top + node.bbox.height;

    for table_index in 0..node.children.len() {
        if !node.children[table_index].visible
            || !matches!(
                node.children[table_index].node_type,
                RenderNodeType::Table(_)
            )
        {
            continue;
        }
        let table_top = node.children[table_index].bbox.y;
        let title_index = (0..table_index).rev().find(|&index| {
            node.children[index].visible && text_line_has_non_whitespace_text(&node.children[index])
        });
        let Some(title_index) = title_index else {
            continue;
        };
        // A real intervening text paragraph owns its own page boundary.  Only
        // a title followed by empty host lines and the next table is movable.
        if node.children[title_index + 1..table_index]
            .iter()
            .any(text_line_has_non_whitespace_text)
        {
            continue;
        }
        let title_top = node.children[title_index].bbox.y;
        let title_bottom = title_top + node.children[title_index].bbox.height;
        if table_top + NESTED_FRAGMENT_EDGE_EPSILON_PX < title_bottom
            || table_top - title_bottom > NESTED_HEADING_WITH_TABLE_MAX_GAP_PX
        {
            continue;
        }

        if table_top >= clip_bottom - NESTED_FRAGMENT_EDGE_EPSILON_PX
            && title_top >= clip_top - NESTED_FRAGMENT_EDGE_EPSILON_PX
            && title_bottom <= clip_bottom + NESTED_FRAGMENT_EDGE_EPSILON_PX
        {
            // The table has no paintable content in this fragment.  Its title
            // belongs to the next fragment with the table rather than to this
            // page's last line.
            for child in &mut node.children[title_index..=table_index] {
                child.visible = false;
            }
            continue;
        }

        if title_top < clip_top
            && clip_top - title_top <= NESTED_HEADING_WITH_TABLE_MAX_GAP_PX
            && table_top >= clip_top - NESTED_FRAGMENT_EDGE_EPSILON_PX
        {
            // Move the complete source group, retaining its title-to-table
            // spacing.  Moving only the text would detach it from the table;
            // expanding the cell clip would replay preceding-page content.
            let target_top = clip_top + NESTED_HEADING_WITH_TABLE_TOP_INSET_PX;
            let delta_y = target_top - title_top;
            for child in &mut node.children[title_index..=table_index] {
                translate_render_subtree_y(child, delta_y);
            }
        }
    }

    for child in &mut node.children {
        if !child.visible || !text_line_has_non_whitespace_text(child) {
            continue;
        }
        let line_top = child.bbox.y;
        if line_top < clip_top && clip_top - line_top <= CLIPPED_TEXT_INK_TOP_OVERFLOW_PX {
            translate_render_subtree_y(child, clip_top + CLIPPED_TEXT_INK_TOP_INSET_PX - line_top);
        }
    }
    if suppress_bottom_text_residue {
        suppress_bottom_clipped_text_residue(node, clip_bottom);
    }
}

/// Suppress a next-fragment table whose first border only grazes the current
/// clipped cell.  A Canvas clip can still anti-alias a fraction of that border
/// even when the table's logical top is just below the clip, leaving a stray
/// horizontal rule at the preceding page's bottom (issue2007 p8).
///
/// This is deliberately limited to a table beginning no more than one border
/// residue window below the physical clip.  A separate page render owns that
/// table with a fresh viewport; no current-page text or usable table area is
/// discarded here.
fn suppress_future_nested_table_border_residue(node: &mut RenderNode, clip_bottom: f64) {
    for child in &mut node.children {
        if !child.visible {
            continue;
        }
        if matches!(child.node_type, RenderNodeType::Table(_)) {
            let table_top = child.bbox.y;
            if table_top >= clip_bottom - NESTED_FRAGMENT_EDGE_EPSILON_PX
                && table_top - clip_bottom <= NESTED_FRAGMENT_RESIDUAL_BORDER_PX
            {
                child.visible = false;
                continue;
            }
        }
        suppress_future_nested_table_border_residue(child, clip_bottom);
    }
}

/// Reconstruct one table's physical fragment frame inside an ancestor
/// `TableCell` clip.  A 1×1 RowBreak wrapper often has no border of its own:
/// the paintable frame belongs to a deeper table in its cell.  Consequently
/// this helper intentionally accepts the ancestor clip rather than requiring
/// the table itself to own it (42065 p10-p14).
fn reconstruct_nested_table_fragment_frame(
    tree: &mut PageLayoutContext,
    table_node: &mut RenderNode,
    clip_top: f64,
    clip_bottom: f64,
) {
    let table_top = table_node.bbox.y;
    let table_bottom = table_top + table_node.bbox.height;
    if table_bottom <= clip_top + NESTED_FRAGMENT_EDGE_EPSILON_PX
        || table_top >= clip_bottom - NESTED_FRAGMENT_EDGE_EPSILON_PX
    {
        return;
    }
    let starts_before_clip = table_top < clip_top - NESTED_FRAGMENT_EDGE_EPSILON_PX;
    let ends_after_clip = table_bottom > clip_bottom + NESTED_FRAGMENT_EDGE_EPSILON_PX;
    if !starts_before_clip && !ends_after_clip {
        return;
    }

    let fragment_top = table_top.max(clip_top);
    let fragment_bottom = table_bottom.min(clip_bottom);
    let fragment_height = fragment_bottom - fragment_top;
    if fragment_height <= NESTED_FRAGMENT_EDGE_EPSILON_PX {
        return;
    }
    // Preserve the source-flow seam rule from the direct wrapper repair. A
    // sub-line tail belongs to the preceding page, even when its visible
    // border is owned by a deeper descendant table; rebuilding it here would
    // turn that tail into a false top frame on p10/p13.
    if starts_before_clip && fragment_height < NESTED_FRAGMENT_RESIDUAL_BORDER_PX {
        return;
    }

    let table_left = table_node.bbox.x;
    let table_right = table_left + table_node.bbox.width;
    let mut horizontal_style = None;
    let mut left_style = None;
    let mut right_style = None;
    for child in &table_node.children {
        let RenderNodeType::Line(line) = &child.node_type else {
            continue;
        };
        let horizontal = (line.y2 - line.y1).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX;
        let vertical = (line.x2 - line.x1).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX;
        if horizontal
            && (line.x1.min(line.x2) - table_left).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
            && (line.x1.max(line.x2) - table_right).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
            && horizontal_style.is_none()
        {
            horizontal_style = Some(line.style.clone());
        }
        if vertical && (line.x1 - table_left).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX {
            left_style.get_or_insert_with(|| line.style.clone());
        }
        if vertical && (line.x1 - table_right).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX {
            right_style.get_or_insert_with(|| line.style.clone());
        }
    }

    // Keep the horizontal centreline inside the clip by half a stroke.  A
    // centreline exactly at the SVG/Canvas clip edge otherwise loses its
    // anti-aliased outer half, and at some device scales the entire rule.
    let frame_top = horizontal_style.as_ref().map_or(fragment_top, |style| {
        if starts_before_clip {
            fragment_horizontal_frame_y(clip_top, clip_bottom, style, true)
        } else {
            fragment_top
        }
    });
    let frame_bottom = horizontal_style.as_ref().map_or(fragment_bottom, |style| {
        if ends_after_clip {
            fragment_horizontal_frame_y(clip_top, clip_bottom, style, false)
        } else {
            fragment_bottom
        }
    });

    if let Some(style) = horizontal_style.as_ref() {
        if starts_before_clip {
            ensure_fragment_horizontal_frame_inside_clip(
                tree,
                table_node,
                table_left,
                table_right,
                clip_top,
                frame_top,
                style.clone(),
            );
        }
        if ends_after_clip && reconstructs_clipped_fragment_bottom(table_node) {
            ensure_fragment_horizontal_frame_inside_clip(
                tree,
                table_node,
                table_left,
                table_right,
                clip_bottom,
                frame_bottom,
                style.clone(),
            );
        }
    }
    if let Some(style) = left_style {
        if !has_fragment_border_line(table_node, table_left, frame_top, table_left, frame_bottom) {
            push_fragment_border_line(
                tree,
                table_node,
                table_left,
                frame_top,
                table_left,
                frame_bottom,
                style,
            );
        }
    }
    if let Some(style) = right_style {
        if !has_fragment_border_line(
            table_node,
            table_right,
            frame_top,
            table_right,
            frame_bottom,
        ) {
            push_fragment_border_line(
                tree,
                table_node,
                table_right,
                frame_top,
                table_right,
                frame_bottom,
                style,
            );
        }
    }
}

/// Visit only descendants: the direct child is handled by the original seam
/// repair first, while this pass reaches the real bordered table below an
/// unbordered RowBreak wrapper.
fn reconstruct_nested_table_descendant_fragment_frames(
    tree: &mut PageLayoutContext,
    node: &mut RenderNode,
    clip_top: f64,
    clip_bottom: f64,
) {
    for child in &mut node.children {
        if matches!(child.node_type, RenderNodeType::Table(_)) {
            reconstruct_nested_table_fragment_frame(tree, child, clip_top, clip_bottom);
        }
        reconstruct_nested_table_descendant_fragment_frames(tree, child, clip_top, clip_bottom);
    }
}

/// Repair the frame and source-flow seam of a true nested-table continuation.
///
/// A direct nested table keeps its document-global coordinates even when its
/// owning 1×1 RowBreak cell is a clipped page fragment. SVG/Canvas therefore
/// naturally retains old table geometry, but loses the new fragment's frame:
/// the source top or bottom can lie beyond the physical clip rectangle. A
/// few-pixel terminal remnant is the inverse case and must be suppressed.
///
/// The source also retains the completed table's following empty `vpos=0`
/// spacer.  If only a sub-line table tail reaches the new viewport, that
/// consumed spacer otherwise starts a second time in the new cell and moves
/// the next real source block down by one line advance (42065 p10/p13).
/// Normalize that exact two-line seam after layout; it changes neither the
/// pagination cut nor a non-empty text line's ownership.
///
/// This runs after native table edges are emitted. It never changes a table
/// with current-page content, and the small source-spacer translation is
/// limited to the direct siblings following a suppressed residual tail.
fn repair_clipped_nested_table_fragment_frame(
    tree: &mut PageLayoutContext,
    node: &mut RenderNode,
    suppress_bottom_text_residue: bool,
    repair_unclipped_hwpx_top_residue: bool,
) {
    if repair_unclipped_hwpx_top_residue {
        rebase_nested_cell_top_residue_line(node);
    }
    let is_clipped_cell = matches!(
        &node.node_type,
        RenderNodeType::TableCell(TableCellNode { clip: true, .. })
    );
    if !is_clipped_cell {
        return;
    }

    let clip_top = node.bbox.y;
    let clip_bottom = node.bbox.y + node.bbox.height;
    repair_clipped_cell_text_table_seam(node, suppress_bottom_text_residue);
    suppress_future_nested_table_border_residue(node, clip_bottom);
    for table_index in 0..node.children.len() {
        if !node.children[table_index].visible
            || !matches!(
                node.children[table_index].node_type,
                RenderNodeType::Table(_)
            )
        {
            continue;
        }

        let table_node = &mut node.children[table_index];

        let table_top = table_node.bbox.y;
        let table_bottom = table_top + table_node.bbox.height;
        if table_bottom <= clip_top + NESTED_FRAGMENT_EDGE_EPSILON_PX
            || table_top >= clip_bottom - NESTED_FRAGMENT_EDGE_EPSILON_PX
        {
            continue;
        }
        let starts_before_clip = table_top < clip_top - NESTED_FRAGMENT_EDGE_EPSILON_PX;
        let ends_after_clip = table_bottom > clip_bottom + NESTED_FRAGMENT_EDGE_EPSILON_PX;
        if !starts_before_clip && !ends_after_clip {
            continue;
        }
        let fragment_top = table_top.max(clip_top);
        let fragment_bottom = table_bottom.min(clip_bottom);
        let fragment_height = fragment_bottom - fragment_top;
        if fragment_height <= NESTED_FRAGMENT_EDGE_EPSILON_PX {
            continue;
        }

        if starts_before_clip && fragment_height < NESTED_FRAGMENT_RESIDUAL_BORDER_PX {
            // Only a sub-line tail reaches this page. Hiding its source edge
            // prevents the previous table's bottom border from appearing as a
            // false top border without affecting any current-page content.
            table_node.visible = false;

            // The tail's following empty `vpos=0` source line was consumed on
            // the preceding fragment.  The renderer still lays it out at the
            // new cell's top, so its line box and following advance are
            // incorrectly paid twice. Drop that empty spacer and shift its
            // following source siblings by precisely those two stored
            // advances. This is
            // deliberately tighter than a global continuation offset: p9 has
            // a real spacer at its new block and must retain it, while p10/p13
            // have an actual table tail in this viewport.
            let following_lines: Vec<(f64, f64)> = node
                .children
                .iter()
                .skip(table_index + 1)
                .filter(|child| child.visible && child.bbox.y >= clip_top)
                .filter(|child| matches!(child.node_type, RenderNodeType::TextLine(_)))
                .map(|child| (child.bbox.y, child.bbox.height))
                .collect();
            if let (Some((spacer_y, _)), Some((next_line_y, _))) =
                (following_lines.first(), following_lines.get(1))
            {
                let spacer_index = node
                    .children
                    .iter()
                    .enumerate()
                    .skip(table_index + 1)
                    .find(|(_, child)| {
                        child.visible
                            && (child.bbox.y - *spacer_y).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                    })
                    .map(|(index, _)| index);
                let line_advance = next_line_y - spacer_y;
                if spacer_index
                    .is_some_and(|index| is_empty_vpos_spacer_line(&node.children[index]))
                    && spacer_y - clip_top <= 24.0
                    && line_advance > NESTED_FRAGMENT_EDGE_EPSILON_PX
                    && line_advance <= 16.0
                {
                    let source_seam_height = line_advance * 2.0;
                    let spacer_index = spacer_index.expect("checked empty spacer index");
                    node.children[spacer_index].visible = false;
                    for child in node.children.iter_mut().skip(spacer_index + 1) {
                        translate_render_subtree_y(child, -source_seam_height);
                    }
                }
            }
            continue;
        }

        let table_left = table_node.bbox.x;
        let table_right = table_left + table_node.bbox.width;
        let mut horizontal_style = None;
        let mut left_style = None;
        let mut right_style = None;
        for child in &table_node.children {
            let RenderNodeType::Line(line) = &child.node_type else {
                continue;
            };
            let horizontal = (line.y2 - line.y1).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX;
            let vertical = (line.x2 - line.x1).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX;
            if horizontal
                && (line.x1.min(line.x2) - table_left).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                && (line.x1.max(line.x2) - table_right).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX
                && horizontal_style.is_none()
            {
                horizontal_style = Some(line.style.clone());
            }
            if vertical && (line.x1 - table_left).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX {
                left_style.get_or_insert_with(|| line.style.clone());
            }
            if vertical && (line.x1 - table_right).abs() <= NESTED_FRAGMENT_EDGE_EPSILON_PX {
                right_style.get_or_insert_with(|| line.style.clone());
            }
        }

        // Place reconstructed horizontal centerlines half a stroke inside the
        // clip. SVG clipPath and Canvas clip both otherwise remove half (and
        // at some device scales all) of a line placed exactly on the boundary.
        // The sub-pixel inset retains the physical edge while keeping paint
        // independent of browser anti-aliasing rules.
        let frame_top = horizontal_style.as_ref().map_or(fragment_top, |style| {
            if starts_before_clip {
                fragment_horizontal_frame_y(clip_top, clip_bottom, style, true)
            } else {
                fragment_top
            }
        });
        let frame_bottom = horizontal_style.as_ref().map_or(fragment_bottom, |style| {
            if ends_after_clip {
                fragment_horizontal_frame_y(clip_top, clip_bottom, style, false)
            } else {
                fragment_bottom
            }
        });

        // The original vertical sides already intersect the cell clip in most
        // renderers. Emit them again at fragment boundaries nevertheless: an
        // SVG/Canvas clip exactly on the edge otherwise yields an open frame.
        if let Some(style) = horizontal_style.as_ref() {
            if starts_before_clip {
                ensure_fragment_horizontal_frame_inside_clip(
                    tree,
                    table_node,
                    table_left,
                    table_right,
                    clip_top,
                    frame_top,
                    style.clone(),
                );
            }
            if ends_after_clip && reconstructs_clipped_fragment_bottom(table_node) {
                ensure_fragment_horizontal_frame_inside_clip(
                    tree,
                    table_node,
                    table_left,
                    table_right,
                    clip_bottom,
                    frame_bottom,
                    style.clone(),
                );
            }
        }
        if let Some(style) = left_style {
            if !has_fragment_border_line(
                table_node,
                table_left,
                frame_top,
                table_left,
                frame_bottom,
            ) {
                push_fragment_border_line(
                    tree,
                    table_node,
                    table_left,
                    frame_top,
                    table_left,
                    frame_bottom,
                    style,
                );
            }
        }
        if let Some(style) = right_style {
            if !has_fragment_border_line(
                table_node,
                table_right,
                frame_top,
                table_right,
                frame_bottom,
            ) {
                push_fragment_border_line(
                    tree,
                    table_node,
                    table_right,
                    frame_top,
                    table_right,
                    frame_bottom,
                    style,
                );
            }
        }
    }

    // The immediate 1×1 RowBreak table can be an unbordered structural
    // wrapper.  Its descendant table owns the visible rule, so apply the same
    // physical fragment-frame contract below that wrapper as well.
    reconstruct_nested_table_descendant_fragment_frames(tree, node, clip_top, clip_bottom);
}

/// Run the narrow horizontal clip correction only after every nested table in
/// the current subtree has emitted its border edges. Calling the single-cell
/// helper during the parent cell loop is too early for normal edge rendering:
/// p2-p3's 4×2/9×2 tables append their `Line`s after that loop and therefore
/// retained the undersized wrapper clip. The traversal stays post-order and
/// only delegates to the direct-child-table helper above, so it cannot widen a
/// continuation's vertical viewport or reveal a future-page text tail.
pub(super) fn extend_completed_nested_table_border_clips(
    tree: &mut PageLayoutContext,
    node: &mut RenderNode,
    suppress_bottom_text_residue: bool,
    repair_unclipped_hwpx_top_residue: bool,
) {
    for child in &mut node.children {
        extend_completed_nested_table_border_clips(
            tree,
            child,
            suppress_bottom_text_residue,
            repair_unclipped_hwpx_top_residue,
        );
    }
    extend_table_horizontal_bbox_to_direct_cell_paint(node);
    extend_clipped_cell_horizontal_clip_to_nested_table_borders(node);
    extend_clipped_cell_vertical_clip_to_nearby_nested_table_borders(node);
    repair_clipped_nested_table_fragment_frame(
        tree,
        node,
        suppress_bottom_text_residue,
        repair_unclipped_hwpx_top_residue,
    );
}

fn should_render_table_caption(table: &crate::model::table::Table, depth: usize) -> bool {
    depth == 0
        || (depth == 1
            && table
                .caption
                .as_ref()
                .is_some_and(caption_has_topbottom_picture))
}

fn caption_flow_extra(caption: &Option<Caption>, caption_height: f64, caption_spacing: f64) -> f64 {
    let is_lr_caption = caption.as_ref().is_some_and(|cap| {
        matches!(
            cap.direction,
            CaptionDirection::Left | CaptionDirection::Right
        )
    });
    if is_lr_caption || caption_height <= 0.0 {
        0.0
    } else {
        caption_height + caption_spacing
    }
}

fn top_caption_flow_extra(
    caption: &Option<Caption>,
    caption_height: f64,
    caption_spacing: f64,
) -> f64 {
    if caption
        .as_ref()
        .is_some_and(|cap| matches!(cap.direction, CaptionDirection::Top))
    {
        caption_flow_extra(caption, caption_height, caption_spacing)
    } else {
        0.0
    }
}

fn build_col_row_y_from_cell_heights(
    table: &crate::model::table::Table,
    row_heights: &[f64],
    row_y: &[f64],
    col_count: usize,
    row_count: usize,
    cell_spacing: f64,
    dpi: f64,
) -> Vec<Vec<f64>> {
    let mut cell_height_grid = vec![vec![None::<f64>; row_count]; col_count];
    for (cell_idx, cell) in table.cells.iter().enumerate() {
        if cell.row_span == 1
            && cell.col_span == 1
            && cell.height < 0x8000_0000
            && (cell.col as usize) < col_count
            && (cell.row as usize) < row_count
        {
            let render_height = table
                .local_resize_cell_heights
                .iter()
                .find(|(idx, _)| *idx == cell_idx)
                .map(|(_, height)| *height)
                .unwrap_or(cell.height);
            cell_height_grid[cell.col as usize][cell.row as usize] =
                Some(hwpunit_to_px(render_height as i32, dpi));
        }
    }

    let fallback_h = hwpunit_to_px(400, dpi);
    let target_total = if table.common.height > 0 {
        hwpunit_to_px(table.common.height as i32, dpi)
            + cell_spacing * row_count.saturating_sub(1) as f64
    } else {
        row_y.last().copied().unwrap_or(0.0)
    };
    let mut col_row_y = vec![vec![0.0f64; row_count + 1]; col_count];
    for c in 0..col_count {
        let col_idx = c as u16;
        if !table.local_resize_cols.contains(&col_idx) {
            col_row_y[c].clone_from_slice(row_y);
            continue;
        }
        for r in 0..row_count {
            let h = cell_height_grid[c][r]
                .or_else(|| row_heights.get(r).copied())
                .unwrap_or(fallback_h);
            col_row_y[c][r + 1] =
                col_row_y[c][r] + h + if r + 1 < row_count { cell_spacing } else { 0.0 };
        }
        // 저장 파일의 cell.height는 표 전체 높이와 맞지 않는 보조값일 수 있다.
        // 열별 누적 높이가 표 외곽과 맞을 때만 독립 horizontal segment로 해석한다.
        if (col_row_y[c][row_count] - target_total).abs() > 0.5 && row_y.len() == row_count + 1 {
            col_row_y[c].clone_from_slice(row_y);
        }
    }
    col_row_y
}

fn has_independent_col_row_y(col_row_y: &[Vec<f64>], row_y: &[f64]) -> bool {
    col_row_y.iter().any(|cy| {
        cy.iter()
            .zip(row_y.iter())
            .any(|(a, b)| (a - b).abs() > 0.01)
    })
}

fn render_cell_box_borders(
    tree: &mut PageLayoutContext,
    bs: &ResolvedBorderStyle,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> Vec<RenderNode> {
    let mut nodes = Vec::new();
    nodes.extend(create_border_line_nodes(
        tree,
        &bs.borders[2],
        x,
        y,
        x + w,
        y,
    ));
    nodes.extend(create_border_line_nodes(
        tree,
        &bs.borders[3],
        x,
        y + h,
        x + w,
        y + h,
    ));
    nodes.extend(create_border_line_nodes(
        tree,
        &bs.borders[0],
        x,
        y,
        x,
        y + h,
    ));
    nodes.extend(create_border_line_nodes(
        tree,
        &bs.borders[1],
        x + w,
        y,
        x + w,
        y + h,
    ));
    nodes
}

pub(crate) fn border_style_has_diagonal(bs: &ResolvedBorderStyle) -> bool {
    let slash_bits = (bs.diagonal_attr >> 2) & 0x07;
    let backslash_bits = (bs.diagonal_attr >> 5) & 0x07;
    (slash_bits != 0 || backslash_bits != 0 || bs.center_line != CenterLine::None)
        && bs.diagonal.diagonal_type != 0
}

fn border_style_has_center_line_only(bs: &ResolvedBorderStyle) -> bool {
    let slash_bits = (bs.diagonal_attr >> 2) & 0x07;
    let backslash_bits = (bs.diagonal_attr >> 5) & 0x07;
    bs.diagonal.diagonal_type != 0
        && bs.center_line != CenterLine::None
        && slash_bits == 0
        && backslash_bits == 0
}

/// cellzone 대각선은 영역 전체에 한 번 그리고, 원본 중복 BF가 붙는 시작 셀만 숨긴다.
fn mark_cellzone_diagonal_origin_coverage(
    covered: &mut [Vec<bool>],
    start_row: usize,
    start_col: usize,
) {
    if let Some(row) = covered.get_mut(start_row) {
        if let Some(cell) = row.get_mut(start_col) {
            *cell = true;
        }
    }
}

fn cell_span_has_cellzone_diagonal(
    covered: &[Vec<bool>],
    row: usize,
    col: usize,
    row_span: usize,
    col_span: usize,
    row_count: usize,
    col_count: usize,
) -> bool {
    let end_row = (row + row_span).min(row_count);
    let end_col = (col + col_span).min(col_count);
    (row..end_row).any(|rr| {
        (col..end_col).any(|cc| {
            covered
                .get(rr)
                .and_then(|cells| cells.get(cc))
                .copied()
                .unwrap_or(false)
        })
    })
}

fn border_style_has_center_line(bs: &ResolvedBorderStyle) -> bool {
    bs.center_line != CenterLine::None && bs.diagonal.diagonal_type != 0
}

fn table_grid_cell_has_own_diagonal(
    table: &crate::model::table::Table,
    styles: &ResolvedStyleSet,
    row: usize,
    col: usize,
    zone_border_fill_id: u16,
) -> bool {
    table.cells.iter().any(|cell| {
        let start_row = cell.row as usize;
        let end_row = start_row + cell.row_span as usize;
        let start_col = cell.col as usize;
        let end_col = start_col + cell.col_span as usize;
        if row < start_row
            || row >= end_row
            || col < start_col
            || col >= end_col
            || cell.border_fill_id == 0
            || cell.border_fill_id == zone_border_fill_id
        {
            return false;
        }
        styles
            .border_styles
            .get((cell.border_fill_id as usize).saturating_sub(1))
            .is_some_and(border_style_has_diagonal)
    })
}

fn cellzone_diagonal_fully_overridden_by_cells(
    table: &crate::model::table::Table,
    styles: &ResolvedStyleSet,
    start_row: usize,
    end_row: usize,
    start_col: usize,
    end_col: usize,
    zone_border_fill_id: u16,
) -> bool {
    start_row < end_row
        && start_col < end_col
        && (start_row..end_row).all(|row| {
            (start_col..end_col).all(|col| {
                table_grid_cell_has_own_diagonal(table, styles, row, col, zone_border_fill_id)
            })
        })
}

/// [Task #993] 분할 표 행 컷 — 행에 속한 셀(col 오름차순)별 "소비한 콘텐츠 유닛 수".
/// 빈 Vec = 처음부터(아무것도 소비 안 함).
pub(crate) type RowCut = Vec<usize>;

/// [Task #993] `advance_row_cut` 결과.
#[derive(Debug, Clone)]
pub(crate) struct RowCutResult {
    /// 셀별 소비 유닛 수 (전진 후).
    pub end_cut: RowCut,
    /// 어느 셀이든 vpos 리셋(hard break)에서 멈췄는가.
    pub hit_hard_break: bool,
    /// 모든 셀이 모든 유닛을 소비했는가.
    pub fully_consumed: bool,
    /// 이 프래그먼트의 콘텐츠 높이 (셀별 표시 높이의 최댓값, 패딩 제외).
    pub consumed_height: f64,
}

/// 재귀 1×1 block 앞의 source 문단 묶음 역할.
///
/// 부모 `RowCut`으로 투영한 뒤에는 자식 문단 인덱스가 사라지므로, 빈 구분 문단과
/// 바로 뒤 한 줄 제목을 높이나 텍스트 휴리스틱 없이 함께 넘기기 위해 보존한다.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum RecursiveBlockPreludeRole {
    #[default]
    None,
    EmptySeparator,
    ExplicitPageBreakSeparator,
    OneLineHeadingBeforeSingleCellTable,
}

/// [Task #993] 한 셀의 콘텐츠 유닛 — 합성 줄 1개 또는 중첩 표 atom 1개.
pub(super) struct CellUnit {
    /// 유닛 높이 (px).
    height: f64,
    /// 이 유닛 앞에 vpos 리셋(셀 내부 페이지 분할)이 있는가.
    hard_break_before: bool,
    /// 같은 문단의 줄 사이에서 페이지 하단까지 진행한 저장 vpos가
    /// 상단으로 되돌아가며 프레임이 바뀌는 경계인가.
    /// 이 경계를 흡수하면 렌더러의 줄 좌표가 역행하므로 부모 컷에서도
    /// 반드시 보존한다. 문단 사이 reset의 orphan/sliver 완화 계약과 구분한다.
    stored_frame_break_before: bool,
    vpos_gap_before: bool,
    /// 이 유닛이 속한 문단 인덱스 (셀 내). [#4149] 커서 프로브 계획이 창 문단
    /// 범위를 계산할 때 형제 모듈(table_partial)에서 읽는다.
    pub(super) para_idx: usize,
    /// 이 유닛이 visible 일 때 기여하는 문단 내 줄 범위 `[vis_start, vis_end)`.
    /// 텍스트 줄 유닛 = `(li, li+1)`, 중첩/빈 atom = `(0, line_count.max(1))`.
    vis_start: usize,
    vis_end: usize,
    /// [Task #1073] 이 유닛이 중첩 표의 한 행을 표현하면 그 행 인덱스. 텍스트/일반 유닛은 None.
    /// 분할 행에서 컷 → `NestedTableSplit`(중첩행 범위) 매핑에 사용.
    nested_row: Option<usize>,
    /// [#4069] `CELL` 분할 중첩 표의 한 행을 셀별 cursor로 더 잘게 나눈 조각.
    /// 바깥 CellUnit 컷이 이 조각을 선택하면 렌더러도 같은 자식 컷을 사용한다.
    nested_table_fragment: Option<NestedTableUnitCut>,
    mixed_nested_fragment: bool,
    mixed_nested_trailing: bool,
    mixed_nested_content_height: f64,
    /// [#4069] 이 mixed fragment가 자식 1×1 표의 canonical CellUnit을 그대로
    /// 투영한 것인지 표시한다. true이면 렌더도 같은 자식 컷 범위를 재귀 사용한다.
    mixed_nested_recursive: bool,
    /// 이 fragment의 첫 실제 콘텐츠가 직전 중첩 표 다음 문단에서 시작하는가.
    /// 새 block은 이전 viewport의 예약 줄이 아니므로 continuation origin에서
    /// 건너뛰지 않는다(42065 p9).
    mixed_nested_starts_after_table: bool,
    /// `mixed_nested_fragment`를 만든 immediate child cell의 source paragraph.
    /// 상위 host의 `para_idx`와 분리해 중첩 표 control의 정확한 원본 경계를 보존한다.
    mixed_nested_source_para_idx: Option<usize>,
    /// 자식 source 문단의 재귀 block prelude 역할. 재귀 투영 단계에서도 보존한다.
    recursive_block_prelude_role: RecursiveBlockPreludeRole,
    top_and_bottom_flow: bool,
    empty_spacer: bool,
    /// Square/Tight/Through non-inline flow fragment가 걸친 원 문단 control index 범위
    /// (inclusive). 높이·unit 경계는 바꾸지 않고 partial renderer의 page owner 판단에만 쓴다.
    non_inline_control_range: Option<(usize, usize)>,
}

/// mixed nested unit의 source-owner 판정에 필요한 최소 의미 정보.
///
/// `CellUnit` 전체를 helper에 노출하지 않아도 viewport reservation 규칙을 독립적으로
/// 회귀 고정할 수 있게 한다.
#[derive(Debug, Clone, Copy)]
struct MixedNestedOwnerMarker {
    para_idx: usize,
    fragment: bool,
    trailing: bool,
    content_height: f64,
    height: f64,
}

impl From<&CellUnit> for MixedNestedOwnerMarker {
    fn from(unit: &CellUnit) -> Self {
        Self {
            para_idx: unit.para_idx,
            fragment: unit.mixed_nested_fragment,
            trailing: unit.mixed_nested_trailing,
            content_height: unit.mixed_nested_content_height,
            height: unit.height,
        }
    }
}

/// 현재 cut 바로 뒤의 빈 trailing reservation이 mixed stream의 최종 source owner
/// 다음에 놓였을 때만 그 높이를 반환한다.
///
/// 뒤에 실제 source unit이 하나라도 남아 있으면 scalar child renderer는 명시적인
/// end-cut이 없으므로 viewport 확장이 미래 콘텐츠를 현재 쪽에 노출할 수 있다.
fn trailing_reservation_after_final_source_owner(
    para_idx: usize,
    successor: Option<MixedNestedOwnerMarker>,
    later_units: impl IntoIterator<Item = MixedNestedOwnerMarker>,
) -> f64 {
    let Some(successor) = successor.filter(|unit| {
        unit.para_idx == para_idx && unit.fragment && unit.trailing && unit.content_height <= 0.5
    }) else {
        return 0.0;
    };

    let has_later_source_owner = later_units.into_iter().any(|unit| {
        unit.para_idx == para_idx && unit.fragment && (!unit.trailing || unit.content_height > 0.5)
    });
    if has_later_source_owner {
        0.0
    } else {
        successor.height
    }
}

/// [#4069] 중첩 표의 셀 흐름을 바깥 셀 컷 원장으로 투영한 한 조각.
///
/// 기존 `(height, trailing, content_height)` 튜플은 내부 셀의 강제 쪽 경계를
/// 잃어버렸다. 특히 42065의 1×1 중첩 셀은 저장된 vpos가 쪽마다 0으로
/// 리셋되므로, 그 경계까지 함께 투영해야 첫 조각과 continuation이 같은 원장을 쓴다.
#[derive(Debug, Clone, Copy)]
struct NestedFlowFragment {
    height: f64,
    hard_break_before: bool,
    stored_frame_break_before: bool,
    trailing: bool,
    content_height: f64,
    recursive: bool,
    starts_after_table: bool,
    /// Immediate child cell의 source paragraph. 1×1 host를 outer cell unit으로
    /// 평탄화할 때 table-control 경계(p20 등)를 잃지 않기 위한 provenance다.
    /// `None`은 기존 synthetic/row aggregate fragment이며 layout 값은 바꾸지 않는다.
    source_para_idx: Option<usize>,
    recursive_block_prelude_role: RecursiveBlockPreludeRole,
}

#[derive(Debug, Clone)]
struct NestedTableUnitCut {
    start_cut: RowCut,
    end_cut: RowCut,
    terminal: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct NestedTableCut {
    pub start_row: usize,
    pub end_row: usize,
    pub start_cut: RowCut,
    pub end_cut: RowCut,
    pub is_block_split: bool,
}

/// 중첩 표 부분 렌더링을 위한 행 범위 정보
pub(crate) struct NestedTableSplit {
    pub start_row: usize,
    pub end_row: usize,
    /// 실제 표시할 높이 (마지막 행이 부분적으로 보일 때 전체 행 높이 대신 사용)
    pub visible_height: f64,
    /// 다음 셀 내용의 흐름 위치를 전진시킬 높이. 일반 split 에서는 visible_height 와 같고,
    /// mixed nested tail 에서는 표시 bbox 보다 큰 원래 flow slice 를 유지할 수 있다.
    pub flow_height: f64,
    /// start_row 내부 오프셋: 이미 이전 페이지에 렌더링된 start_row 상단 부분의 높이
    pub offset_within_start: f64,
    /// 셀 유닛 스트림에서 이 조각이 시작하는 원본 소비 위치. mixed nested tail은
    /// 물리 y 보정 때문에 `offset_within_start`에 첫 가시 유닛을 더할 수 있으므로,
    /// 다음 깊이의 동일한 유닛 컷을 재구성할 때는 이 원본 값을 사용한다.
    pub content_offset: f64,
    /// 부모 native short-parent-child fragment가 이미 소비한 source unit을 terminal
    /// child viewport에서도 다시 그리지 않도록 한다. 일반 terminal tail은 종전처럼
    /// source cut을 끈다; native short-parent-child와 p33→34류 terminal rowbreak
    /// source cursor를 함께 OR한 값이므로 두 형상 모두 unit 컷 자체는 켠다.
    pub force_source_start_cut: bool,
    /// native short-parent-child(76076 p81→82)에서만 true. `force_source_start_cut`이
    /// p33→34류 terminal rowbreak source cursor만으로 true인 경우에는 false를 유지해
    /// 이미 소유가 끝난 마지막 unit을 다음 조각에서 중복 페인트하지 않는다.
    pub replay_terminal_boundary_unit: bool,
    /// [#3658] 이 조각이 해당 셀 콘텐츠의 **마지막** 조각인가 (컷이 마지막 유닛까지
    /// 포함 — end_cut 종료). true 면 이어받을 continuation 이 없으므로 셀 하단 초과
    /// 줄 드롭(다음 쪽 소속 줄 제외용)을 적용하지 않는다 — 꼬리 문단 유실 방지.
    pub terminal: bool,
    /// [#4069] 중첩 표의 자식 행·CellUnit 범위. Some이면 scalar clip 대신
    /// `layout_partial_table`에 동일 컷을 넘겨 측정과 렌더의 fragment 권위를 통일한다.
    pub recursive_cut: Option<NestedTableCut>,
}

/// [#4698] 셀 안 문단을 한컴이 저장한 **조각(fragment)** 단위로 묶는다.
///
/// 병합 셀이 쪽 경계에 걸리면 한컴은 그 셀의 문단을 쪽별로 나눠 놓고, 각 조각의
/// 첫 문단 `LINE_SEG.vpos` 를 다시 0 부터 기록한다 (kps-ai 셀[39]:
/// `[0, 1920, 3840, 0, 1920]` = 앞쪽 3 문단 + 다음 쪽 2 문단). 즉 두 번째 이후
/// 문단의 `vpos == 0` 은 "조각이 여기서 시작한다"는 저장된 사실이다.
///
/// 모든 문단이 `vpos == 0` 인 셀(앵커 미기록 센티널)은 조각 정보가 아니므로
/// 제외한다 — 각 조각의 저장 extent 가 0 인지로 가른다.
fn stored_cell_fragment_groups(cell: &crate::model::table::Cell) -> Vec<(usize, usize)> {
    let mut groups: Vec<(usize, usize)> = Vec::new();
    let mut start = 0usize;
    for (idx, para) in cell.paragraphs.iter().enumerate() {
        let restarts = idx > 0
            && para
                .line_segs
                .first()
                .is_some_and(|seg| seg.vertical_pos == 0);
        if restarts {
            groups.push((start, idx));
            start = idx;
        }
    }
    if start < cell.paragraphs.len() {
        groups.push((start, cell.paragraphs.len()));
    }
    groups
}

/// 조각의 저장 높이(HWPUNIT) — 조각 안 줄들의 `vpos + line_height` 최댓값.
fn stored_fragment_extent_hu(cell: &crate::model::table::Cell, group: (usize, usize)) -> i32 {
    cell.paragraphs[group.0..group.1]
        .iter()
        .flat_map(|para| para.line_segs.iter())
        .filter(|seg| seg.vertical_pos >= 0)
        .map(|seg| seg.vertical_pos.saturating_add(seg.line_height.max(0)))
        .max()
        .unwrap_or(0)
}

/// 중첩 표에서 pixel offset/space를 행 범위로 변환한다.
/// 공간이 부족한 마지막 행은 제외하여 다음 페이지에서 렌더링되도록 한다.
pub(crate) fn calc_nested_split_rows(
    row_heights: &[f64],
    cell_spacing: f64,
    offset: f64,
    space: f64,
) -> NestedTableSplit {
    let row_count = row_heights.len();
    if row_count == 0 {
        return NestedTableSplit {
            start_row: 0,
            end_row: 0,
            visible_height: 0.0,
            flow_height: 0.0,
            offset_within_start: 0.0,
            content_offset: 0.0,
            force_source_start_cut: false,
            replay_terminal_boundary_unit: false,
            terminal: false,
            recursive_cut: None,
        };
    }

    // row_y 누적 배열 (layout_table과 동일 방식)
    let mut row_y = vec![0.0f64; row_count + 1];
    for i in 0..row_count {
        row_y[i + 1] =
            row_y[i] + row_heights[i] + if i + 1 < row_count { cell_spacing } else { 0.0 };
    }

    // offset에 해당하는 시작 행 찾기
    let mut start_row = 0;
    if offset > 0.0 {
        start_row = row_count;
        for r in 0..row_count {
            if row_y[r] + row_heights[r] > offset {
                start_row = r;
                break;
            }
        }
    }

    // space에 해당하는 끝 행 찾기
    let visible_end = offset + space;
    let mut end_row = row_count;
    if space > 0.0 && space < f64::MAX {
        for r in 0..row_count {
            if row_y[r] + row_heights[r] >= visible_end {
                end_row = r + 1;
                break;
            }
        }
    }

    // 마지막 행이 거의 들어가지 않으면 제외하여 다음 페이지에서 온전하게 렌더링
    if end_row > start_row {
        let last_r = end_row - 1;
        let last_row_top = row_y[last_r];
        let available_for_last = visible_end - last_row_top;
        let last_h = row_heights[last_r];
        let min_threshold = (last_h * 0.5).min(10.0);
        if available_for_last < last_h && available_for_last < min_threshold {
            end_row -= 1;
        }
    }

    // visible_height: 포함된 행의 실제 높이 (start_row 전체 포함)
    let range_height = if end_row > start_row {
        row_y[end_row] - row_y[start_row]
    } else {
        0.0
    };
    // 연속 페이지(offset>0): start_row를 처음부터 완전히 렌더링하므로
    // offset_within_start=0, visible_height=range_height (포함된 행 전체 높이)
    // 첫 페이지(offset==0): 가용 공간으로 캡
    let visible_height = if offset > 0.0 {
        range_height
    } else {
        space.min(range_height)
    };

    NestedTableSplit {
        start_row,
        end_row,
        visible_height,
        flow_height: visible_height,
        offset_within_start: 0.0,
        content_offset: offset.max(0.0),
        force_source_start_cut: false,
        replay_terminal_boundary_unit: false,
        terminal: false,
        recursive_cut: None,
    }
}

/// [#2089] 가로쓰기 셀 본문 배치의 셀-스코프 스칼라 묶음.
#[derive(Clone, Copy)]
struct HorizontalCellVars {
    cell_idx: usize,
    r: usize,
    cell_y: f64,
    cell_h: f64,
    content_cell_y: f64,
    pad_top: f64,
    inner_x: f64,
    inner_width: f64,
    inner_height: f64,
    text_y_start: f64,
    use_top_vpos_anchor: bool,
    /// Physical page-clip continuation이 저장 vpos 앵커도 함께 내려야 하는 높이.
    upper_clip_line_reservation: f64,
    /// [Task #2211] 저장 LINE_SEG 흐름이 자체 스택 합보다 압축된 셀 —
    /// 문단 배치를 저장 vpos 스냅으로 강제한다 (valign 무관).
    trust_stored_cell_flow: bool,
    has_nested_table: bool,
    section_index: usize,
    outline_numbering_id: u16,
    depth: usize,
    clamp_header_negative_para_offset: bool,
    /// root-body table owner의 첫 저장 LINE_SEG vpos. nested/header/footer 호출은 None.
    outer_host_stored_vpos_hu: Option<i32>,
    inline_table_flow_y_shift: f64,
    /// 이 1×1 표가 앞 페이지에서 일부 흐름을 이미 소비한 continuation인가.
    /// 단순 row filter의 첫 조각과 구분해, 미래 중첩 표의 위치 상한을 풀 수 있는
    /// 유일한 문맥으로 쓴다 (#2007/#3637).
    single_row_continuation: bool,
    /// 부모 RowBreak 조각이 이 1×1 표에 명시적으로 전달한 물리 viewport인가.
    /// 첫 조각(offset=0)도 포함한다. 표 셀의 유닛 컷을 재구성할 때 continuation과
    /// 구분하지 않고 같은 소유 범위를 적용해야 한다.
    single_row_fragment: bool,
    /// 현재 1행 continuation이 앞 조각에서 이미 소비한 높이. 중첩 표도 같은
    /// 물리 viewport를 이어 그리려면 이 누적 오프셋을 그대로 받아야 한다.
    single_row_continuation_offset: Option<f64>,
    /// 같은 조각의 원본 unit 소비 위치. 물리 y 보정과 분리해 다음 중첩 표의
    /// unit 경계를 계산한다.
    single_row_fragment_content_offset: Option<f64>,
    /// native short-parent child의 terminal continuation도 이미 소비한 source
    /// prefix를 건너뛰게 하는 명시 신호. 일반 terminal tail에는 false다.
    force_source_start_cut: bool,
    /// true면 마지막 source unit을 다음 조각에서 재생한다 (76076 p81→82 형상만 해당).
    replay_terminal_boundary_unit: bool,
    /// [#3658] 분할 렌더(row_filter)가 이 셀 콘텐츠의 마지막 조각인가.
    /// true 면 셀 하단 초과 줄 드롭(다음 쪽 소속 줄 제외)을 적용하지 않는다 —
    /// 이어받을 continuation 이 없어 드롭된 꼬리 줄은 영구 유실되기 때문.
    split_terminal: bool,
}

impl LayoutEngine {
    /// 셀 안 비-TAC 자리차지 개체가 표 흐름에 요구하는 세로 범위.
    ///
    /// 한컴의 `쪽 영역 안으로 제한`은 세로 기준이 문단일 때 개체를 쪽 영역 안에
    /// 남기도록 흐름 높이에 반영된다. 반대로 제한이 꺼진 문단 기준 floating
    /// 개체는 표 행 높이를 밀지 않는다.
    pub(crate) fn non_inline_control_flow_height(&self, common: &CommonObjAttr) -> f64 {
        if common.treat_as_char || !matches!(common.text_wrap, TextWrap::TopAndBottom) {
            return 0.0;
        }
        let object_height = hwpunit_to_px(common.height as i32, self.dpi)
            + hwpunit_to_px(common.margin.top as i32, self.dpi)
            + hwpunit_to_px(common.margin.bottom as i32, self.dpi);
        if matches!(common.vert_rel_to, VertRelTo::Para) {
            if common.flow_with_text {
                hwpunit_to_px((common.vertical_offset as i32).max(0), self.dpi) + object_height
            } else {
                0.0
            }
        } else {
            object_height
        }
    }

    pub(crate) fn cell_non_inline_control_flow_height(&self, common: &CommonObjAttr) -> f64 {
        let top_and_bottom_height = self.non_inline_control_flow_height(common);
        if top_and_bottom_height > 0.0 || common.treat_as_char {
            return top_and_bottom_height;
        }

        if !matches!(
            common.text_wrap,
            TextWrap::Square | TextWrap::Tight | TextWrap::Through
        ) {
            return 0.0;
        }

        hwpunit_to_px(common.height as i32, self.dpi)
            + hwpunit_to_px(common.margin.top as i32, self.dpi)
            + hwpunit_to_px(common.margin.bottom as i32, self.dpi)
    }

    pub(crate) fn paragraph_top_and_bottom_non_inline_flow_height(
        &self,
        controls: &[Control],
    ) -> f64 {
        controls
            .iter()
            .map(|ctrl| match ctrl {
                Control::Picture(pic) => self.non_inline_control_flow_height(&pic.common),
                Control::Shape(shape) => self.non_inline_control_flow_height(shape.common()),
                _ => 0.0,
            })
            .fold(0.0, f64::max)
    }

    pub(crate) fn paragraph_cell_non_inline_controls_flow_height(
        &self,
        controls: &[Control],
    ) -> f64 {
        let (top_and_bottom_h, other_h) =
            self.paragraph_cell_non_inline_control_flow_parts(controls);
        top_and_bottom_h + other_h
    }

    fn paragraph_cell_non_inline_control_flow_parts(&self, controls: &[Control]) -> (f64, f64) {
        let mut top_and_bottom_h = 0.0f64;
        let mut other_h = 0.0f64;
        for ctrl in controls {
            let Some(common) = (match ctrl {
                Control::Picture(pic) => Some(&pic.common),
                Control::Shape(shape) => Some(shape.common()),
                _ => None,
            }) else {
                continue;
            };
            if common.treat_as_char {
                continue;
            }
            if matches!(common.text_wrap, TextWrap::TopAndBottom) {
                top_and_bottom_h =
                    top_and_bottom_h.max(self.non_inline_control_flow_height(common));
            } else {
                other_h += self.cell_non_inline_control_flow_height(common);
            }
        }
        (top_and_bottom_h, other_h)
    }

    /// 텍스트 없는 legacy HWP5 host 문단에 수평으로 나란히 놓인 Square/Tight/Through
    /// 개체들의 vertical flow band. 일반 경로는 control 높이를 합산한다. 여기서는
    /// 모든 개체가 paragraph-relative nonnegative offset이고 interval이 공통으로
    /// 겹친다는 좁은 증거가 있을 때만, paragraph origin부터 가장 먼 bottom까지의
    /// physical band를 반환한다. 서로 다른 세로 band나 stale negative offset을 가진
    /// 개체는 `None`으로 돌려 기존 합산 계약을 그대로 보존한다.
    fn paragraph_parallel_other_non_inline_flow_band_height(
        &self,
        controls: &[Control],
    ) -> Option<f64> {
        if controls.len() < 2 {
            return None;
        }

        let mut latest_start = 0.0f64;
        let mut earliest_end = f64::INFINITY;
        let mut furthest_bottom = 0.0f64;
        for control in controls {
            let common = match control {
                Control::Picture(picture) => &picture.common,
                Control::Shape(shape) => shape.common(),
                _ => return None,
            };
            if common.treat_as_char
                || !matches!(
                    common.text_wrap,
                    TextWrap::Square | TextWrap::Tight | TextWrap::Through
                )
                || !matches!(common.vert_rel_to, VertRelTo::Para)
            {
                return None;
            }
            let offset_hu = signed_hwpunit(common.vertical_offset);
            if offset_hu < 0 {
                return None;
            }
            let start = hwpunit_to_px(offset_hu, self.dpi);
            let height = self.cell_non_inline_control_flow_height(common);
            if height <= 0.5 {
                return None;
            }
            let end = start + height;
            latest_start = latest_start.max(start);
            earliest_end = earliest_end.min(end);
            furthest_bottom = furthest_bottom.max(end);
        }

        (latest_start + 0.5 < earliest_end).then_some(furthest_bottom)
    }

    /// Square/Tight/Through cell-flow의 control별 높이. 기존 aggregate 높이 계산과 같은
    /// contract를 유지하되, 16px fragment unit이 어떤 source control에 해당하는지 복원한다.
    fn paragraph_cell_other_non_inline_control_heights(
        &self,
        controls: &[Control],
    ) -> Vec<(usize, f64)> {
        controls
            .iter()
            .enumerate()
            .filter_map(|(control_idx, control)| {
                let common = match control {
                    Control::Picture(picture) => &picture.common,
                    Control::Shape(shape) => shape.common(),
                    _ => return None,
                };
                if common.treat_as_char || matches!(common.text_wrap, TextWrap::TopAndBottom) {
                    return None;
                }
                let height = self.cell_non_inline_control_flow_height(common);
                (height > 0.5).then_some((control_idx, height))
            })
            .collect()
    }

    fn cell_has_top_and_bottom_non_inline_flow(&self, cell: &crate::model::table::Cell) -> bool {
        cell.paragraphs
            .iter()
            .any(|para| self.paragraph_top_and_bottom_non_inline_flow_height(&para.controls) > 0.5)
    }

    pub(crate) fn calc_non_inline_controls_flow_height(&self, paragraphs: &[Paragraph]) -> f64 {
        paragraphs
            .iter()
            .map(|p| self.paragraph_top_and_bottom_non_inline_flow_height(&p.controls))
            .sum()
    }

    fn cell_wrap_object_visual_bottom(&self, common: &CommonObjAttr) -> f64 {
        if common.treat_as_char {
            return 0.0;
        }
        if !matches!(
            common.text_wrap,
            TextWrap::Square | TextWrap::Tight | TextWrap::Through
        ) {
            return 0.0;
        }

        let object_height = hwpunit_to_px(common.height as i32, self.dpi);
        let top_offset = if matches!(common.vert_rel_to, VertRelTo::Para) {
            hwpunit_to_px((common.vertical_offset as i32).max(0), self.dpi)
        } else {
            0.0
        };
        top_offset + object_height
    }

    pub(crate) fn calc_cell_wrap_objects_bottom_height(&self, paragraphs: &[Paragraph]) -> f64 {
        // [Task #2226] TopAndBottom flow 개체 보유 문단의 para_top 은 사다리 기반
        // 문단 시작 — height_measurer::cell_wrap_objects_bottom_height 와 동일 정정.
        let mut prev_extent = 0.0f64;
        paragraphs
            .iter()
            .map(|p| {
                let first_vpos = p
                    .line_segs
                    .first()
                    .map(|s| hwpunit_to_px(s.vertical_pos, self.dpi))
                    .unwrap_or(0.0);
                // 개체가 문단 시작~줄 상단 구간을 채우는 배치(줄이 개체 아래로
                // 밀림)면 first_vpos 는 문단 시작이 아니다 — 기하 판정으로 전환.
                let probe_object_bottom = p
                    .controls
                    .iter()
                    .map(|ctrl| match ctrl {
                        Control::Picture(pic) => self.cell_wrap_object_visual_bottom(&pic.common),
                        Control::Shape(shape) => {
                            self.cell_wrap_object_visual_bottom(shape.common())
                        }
                        _ => 0.0,
                    })
                    .fold(0.0f64, f64::max);
                let objects_above_line = probe_object_bottom > 0.0
                    && prev_extent + probe_object_bottom <= first_vpos + 0.5;
                let para_top = if objects_above_line {
                    prev_extent
                } else {
                    first_vpos
                };
                let para_extent = p
                    .line_segs
                    .iter()
                    .map(|s| {
                        hwpunit_to_px(
                            s.vertical_pos.saturating_add(s.line_height.max(0)),
                            self.dpi,
                        )
                    })
                    .fold(prev_extent, f64::max);
                prev_extent = para_extent;
                let object_bottom = p
                    .controls
                    .iter()
                    .map(|ctrl| match ctrl {
                        Control::Picture(pic) => self.cell_wrap_object_visual_bottom(&pic.common),
                        Control::Shape(shape) => {
                            self.cell_wrap_object_visual_bottom(shape.common())
                        }
                        _ => 0.0,
                    })
                    .fold(0.0f64, f64::max);
                if object_bottom > 0.0 {
                    para_top + object_bottom
                } else {
                    0.0
                }
            })
            .fold(0.0f64, f64::max)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_table(
        &self,
        tree: &mut PageLayoutContext,
        col_node: &mut RenderNode,
        table: &crate::model::table::Table,
        section_index: usize,
        styles: &ResolvedStyleSet,
        outline_numbering_id: u16,
        col_area: &LayoutRect,
        y_start: f64,
        bin_data_content: &[BinDataContent],
        measured_table: Option<&MeasuredTable>,
        depth: usize,
        table_meta: Option<(usize, usize)>,
        host_alignment: Alignment,
        enclosing_cell_ctx: Option<CellContext>,
        host_margin_left: f64,
        host_margin_right: f64,
        inline_x_override: Option<f64>,
        nested_split: Option<&NestedTableSplit>,
        para_y: Option<f64>,
        outer_host_stored_vpos_hu: Option<i32>,
        allow_para_top_bleed: bool,
        clamp_header_negative_para_offset: bool,
        physical_outer_box_paint_inset: bool,
    ) -> f64 {
        if table.cells.is_empty() {
            if depth == 0 {
                return y_start;
            } else {
                return 0.0;
            }
        }
        // 1x1 래퍼 표 감지: 외곽 표를 무시하고 내부 표를 직접 렌더링.
        // (Task #688) 셀 paragraphs 가 2개 이상이면 첫 nested 표만 unwrap 시 나머지
        // paragraph 의 nested 표가 누락되므로 paragraphs.len() == 1 가드를 둔다.
        // controls.len() == 1 가드는 두지 않는다 — exam_social.hwp pi=15 (PR #681)
        // 처럼 정렬 마커 등 다른 control 이 동거하는 케이스에서 unwrap + 외곽선 분기를
        // 모두 보존해야 하므로 find_map 으로 첫 nested table 만 추출한다.
        if table.row_count == 1 && table.col_count == 1 && table.cells.len() == 1 {
            let cell = &table.cells[0];
            if cell.paragraphs.len() == 1 {
                let p = &cell.paragraphs[0];
                let has_visible_text = p
                    .text
                    .chars()
                    .any(|ch| !ch.is_whitespace() && ch != '\r' && ch != '\n');
                if !has_visible_text {
                    if let Some(nested) = p.controls.iter().find_map(|c| {
                        if let Control::Table(t) = c {
                            Some(t.as_ref())
                        } else {
                            None
                        }
                    }) {
                        // [Task #1658 v3] 외곽 1×1 래퍼가 페이지/용지 앵커 자리차지
                        // (절대배치) 표면, unwrap 이 외곽의 절대 y 를 소실시키고 내부 표를
                        // flow 커서(y_start)에 렌더하던 결함 교정 — 외곽 표 속성으로 절대
                        // y 를 계산해 내부 표 시작점으로 사용한다 (하단 고정 결재/서명 틀이
                        // 본문 상단에 그려지던 문제, #1653 RCA 패턴 B).
                        let y_start = if depth == 0
                            && !table.common.treat_as_char
                            && matches!(
                                table.common.text_wrap,
                                crate::model::shape::TextWrap::TopAndBottom
                            )
                            && matches!(
                                table.common.vert_rel_to,
                                crate::model::shape::VertRelTo::Page
                                    | crate::model::shape::VertRelTo::Paper
                            ) {
                            let outer_h = hwpunit_to_px(
                                crate::renderer::float_placement::signed_hwpunit(
                                    table.common.height,
                                )
                                .max(0),
                                self.dpi,
                            );
                            // [Issue #1858] valign=Bottom 하단앵커는 한컴이 **실측
                            // 내용 높이**로 박스 하단을 anchor 하단에 밀착시킨다.
                            // 선언높이(common.height)가 실측보다 크면(stale) 선언
                            // 기준 top 이 위로 떠서 결재/발신명의 코퍼스 전반이
                            // −30.5pt 상향(36389312 계열, 18건 중 13건 동일 상수).
                            // MeasuredTable(캡션 제외 행높이 합) 사용, 부재 시 선언 유지.
                            let effective_h = if matches!(
                                table.common.vert_align,
                                crate::model::shape::VertAlign::Bottom
                                    | crate::model::shape::VertAlign::Outside
                            ) {
                                measured_table
                                    .map(|mt| (mt.total_height - mt.caption_height).max(0.0))
                                    .filter(|h| *h > 0.0)
                                    .unwrap_or(outer_h)
                            } else {
                                outer_h
                            };
                            self.compute_table_y_position(
                                table,
                                effective_h,
                                y_start,
                                col_area,
                                depth,
                                0.0,
                                0.0,
                                para_y,
                                allow_para_top_bleed,
                            )
                        } else {
                            y_start
                        };
                        // [Task: nested-table-border] 자료 박스 외곽 테두리 추가:
                        // 외부 1x1 표가 wrapper 라도 padding + border_fill 에 테두리선이
                        // 정의된 경우 (자료 박스 외곽), 외곽 4개 라인을 별도 추가하여 시각 정합.
                        // 외곽 박스의 size 는 nested layout 의 실제 결과 (y_end - y_start) 와
                        // nested 표의 측정 width 를 사용하여 내부 표 영역과 정확히 정합.
                        // (exam_social.hwp pi=15 4번 자료 박스: 외부 1x1 padding=(850,850,850,850)
                        //  border_fill_id=6, 내부 6x3 대화체 셀.)
                        let outer_y = y_start;
                        let outer_border_meta = if depth == 0 {
                            let has_outer_padding = cell.padding.left != 0
                                || cell.padding.right != 0
                                || cell.padding.top != 0
                                || cell.padding.bottom != 0;
                            if has_outer_padding {
                                // border_fill_id 는 1-based(borderFillIDRef), border_styles 는
                                // 0-based Vec 이므로 -1 변환한다. (일반 셀/표/zone lookup 과 동일)
                                if let Some(bs) = styles
                                    .border_styles
                                    .get((cell.border_fill_id as usize).saturating_sub(1))
                                {
                                    let any_border = bs.borders.iter().any(|b| {
                                        b.line_type != crate::model::style::BorderLineType::None
                                    });
                                    if any_border {
                                        Some(bs.borders)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        // nested 표 위치/size 미리 결정 (nested layout 의 위치 결정 logic 동일)
                        let pw_now = self.current_paper_width.get();
                        let paper_w = if pw_now > 0.0 { Some(pw_now) } else { None };
                        let nested_w = hwpunit_to_px(nested.common.width as i32, self.dpi)
                            * self.render_table_width_scale(nested);
                        let outer_w_for_box = nested_w;
                        let outer_x_for_box = self.compute_table_x_position(
                            nested,
                            nested_w,
                            col_area,
                            depth,
                            host_alignment,
                            host_margin_left,
                            host_margin_right,
                            inline_x_override,
                            paper_w,
                        );

                        let y_end = self.layout_table(
                            tree,
                            col_node,
                            nested,
                            section_index,
                            styles,
                            outline_numbering_id,
                            col_area,
                            y_start,
                            bin_data_content,
                            None,
                            depth,
                            table_meta,
                            host_alignment,
                            enclosing_cell_ctx,
                            host_margin_left,
                            host_margin_right,
                            inline_x_override,
                            nested_split,
                            para_y,
                            None,
                            allow_para_top_bleed,
                            clamp_header_negative_para_offset,
                            false,
                        );

                        if let Some(bs_borders) = outer_border_meta {
                            let outer_h_actual = (y_end - outer_y).max(0.0);
                            if outer_h_actual > 0.0 {
                                use super::border_rendering::create_border_line_nodes;
                                // 좌
                                col_node.children.extend(create_border_line_nodes(
                                    tree,
                                    &bs_borders[0],
                                    outer_x_for_box,
                                    outer_y,
                                    outer_x_for_box,
                                    outer_y + outer_h_actual,
                                ));
                                // 우
                                col_node.children.extend(create_border_line_nodes(
                                    tree,
                                    &bs_borders[1],
                                    outer_x_for_box + outer_w_for_box,
                                    outer_y,
                                    outer_x_for_box + outer_w_for_box,
                                    outer_y + outer_h_actual,
                                ));
                                // 상
                                col_node.children.extend(create_border_line_nodes(
                                    tree,
                                    &bs_borders[2],
                                    outer_x_for_box,
                                    outer_y,
                                    outer_x_for_box + outer_w_for_box,
                                    outer_y,
                                ));
                                // 하
                                col_node.children.extend(create_border_line_nodes(
                                    tree,
                                    &bs_borders[3],
                                    outer_x_for_box,
                                    outer_y + outer_h_actual,
                                    outer_x_for_box + outer_w_for_box,
                                    outer_y + outer_h_actual,
                                ));
                            }
                        }
                        return y_end;
                    }
                }
            }
        }

        let col_count = table.col_count as usize;
        let row_count = table.row_count as usize;
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);

        // ── 1. 열 폭 + 행 높이 계산 ──
        let mut col_widths = self.resolve_column_widths(table, col_count);
        let row_heights = self.resolve_row_heights(
            table,
            col_count,
            row_count,
            measured_table,
            styles,
            depth > 0 || table.common.treat_as_char,
        );
        if std::env::var("RHWP_DIAG_TAC").is_ok() {
            let decl: Vec<f64> = table
                .cells
                .iter()
                .filter(|c| c.row_span == 1)
                .map(|c| hwpunit_to_px(c.height as i32, self.dpi))
                .collect();
            eprintln!(
                "[DIAG_TAC layout_table] tac={} depth={} mt={} meta={:?} rows={} y_start={:.1} decl_common_h={:.1} row_heights={:?} decl_cell_h={:?}",
                table.common.treat_as_char,
                depth,
                measured_table.is_some(),
                table_meta,
                row_count,
                y_start,
                hwpunit_to_px(table.common.height as i32, self.dpi),
                row_heights.iter().map(|h| (h * 10.0).round() / 10.0).collect::<Vec<_>>(),
                decl.iter().map(|h| (h * 10.0).round() / 10.0).collect::<Vec<_>>(),
            );
        }

        // ── 2. 누적 위치 계산 ──
        let mut col_x = vec![0.0f64; col_count + 1];
        for i in 0..col_count {
            col_x[i + 1] =
                col_x[i] + col_widths[i] + if i + 1 < col_count { cell_spacing } else { 0.0 };
        }
        let mut row_y = vec![0.0f64; row_count + 1];
        for i in 0..row_count {
            row_y[i + 1] =
                row_y[i] + row_heights[i] + if i + 1 < row_count { cell_spacing } else { 0.0 };
        }

        // 부모 셀 조각이 전달한 viewport보다 큰 손자 표는, source-unit split이 따로
        // 없는 경우에도 이 조각에서 실제 보이는 행까지만 생성한다. 종전에는 이 경우
        // 전체 행을 RenderTree에 넣은 다음 조상 Cell clip으로만 숨겼다. clip은 SVG
        // 잉크는 가리지만 쪽 하단 밖 TextLine까지 없애지는 않아, 다음 쪽 소유인 줄이
        // 현재 쪽의 `LAYOUT_OVERFLOW_CELL`로 계상됐다(#3637 p28).
        //
        // 저장된 split은 source 소유를 표현하므로 항상 우선한다. 여기의 geometry
        // fallback은 split 부재 + 실제 중첩 표(depth>0)로 한정한다. 저장 파일은 이
        // 크기의 표에도 `treat_as_char` 비트를 남길 수 있으므로 그 비트로 제외하지
        // 않으며, 다음 조각은 부모 RowBreak viewport가 다시 호출해 소유한다.
        let inferred_viewport_split = if nested_split.is_none()
            && depth > 0
            && col_area.height > 0.0
            && row_y.last().copied().unwrap_or(0.0) > col_area.height + 0.5
        {
            Some(calc_nested_split_rows(
                &row_heights,
                cell_spacing,
                0.0,
                col_area.height,
            ))
        } else {
            None
        };
        let nested_split = nested_split.or(inferred_viewport_split.as_ref());

        // 중첩 표 부분 렌더링: row_y를 시프트하여 보이는 행만 표시
        let (row_y_shift, split_row_range, split_y_offset) = if let Some(split) = nested_split {
            let sr = split.start_row.min(row_count);
            let er = split.end_row.min(row_count);
            let shift = row_y[sr];
            // row_y를 시프트하여 start_row가 0에서 시작하도록 함
            for y in row_y.iter_mut() {
                *y -= shift;
            }
            // end_row 이후의 모든 row_y를 캡하여 spanning 셀이 보이는 영역을 초과하지 않도록 함
            let cap_y = if split.visible_height > 0.0 {
                split.visible_height.min(row_y[er])
            } else {
                row_y[er]
            };
            for i in er..=row_count {
                row_y[i] = cap_y;
            }
            // start_row 내부 오프셋: 이미 이전 페이지에 표시된 부분만큼 위로 올림
            (shift, Some((sr, er)), split.offset_within_start)
        } else {
            (0.0, None, 0.0)
        };
        // [#3658] 종료 조각 여부 — 셀 하단 초과 줄 드롭 예외 판정에 사용.
        let split_terminal = nested_split.is_some_and(|s| s.terminal);
        // PR #4122의 재귀 child cursor가 있으면 자식 RowCut이 continuation 소유권을
        // 직접 결정한다. 기존 1×1 위치/정렬 보정은 그 cursor가 없는 scalar fallback
        // continuation에서만 적용해 같은 흐름 오프셋을 두 번 소비하지 않는다.
        let scalar_single_row_fragment = nested_split.is_some_and(|split| {
            row_count == 1
                && col_count == 1
                && split.start_row == 0
                && split.end_row >= 1
                && split.recursive_cut.is_none()
        });
        let scalar_single_row_continuation_offset = nested_split.and_then(|split| {
            (scalar_single_row_fragment && split.offset_within_start > 0.5)
                .then_some(split.offset_within_start)
        });
        let scalar_single_row_fragment_content_offset = nested_split
            .filter(|_| scalar_single_row_fragment)
            .map(|split| split.content_offset);
        let scalar_force_source_start_cut = nested_split
            .is_some_and(|split| scalar_single_row_fragment && split.force_source_start_cut);
        let scalar_replay_terminal_boundary_unit = nested_split
            .is_some_and(|split| scalar_single_row_fragment && split.replay_terminal_boundary_unit);
        let scalar_single_row_continuation = scalar_single_row_continuation_offset.is_some();
        let mut row_col_x = build_row_col_x(
            table,
            &col_widths,
            col_count,
            row_count,
            cell_spacing,
            self.dpi,
            self.render_table_width_scale(table),
        );
        let independent_col_row_y = if split_row_range.is_none() && !table.common.treat_as_char {
            let col_row_y = build_col_row_y_from_cell_heights(
                table,
                &row_heights,
                &row_y,
                col_count,
                row_count,
                cell_spacing,
                self.dpi,
            );
            if has_independent_col_row_y(&col_row_y, &row_y) {
                Some(col_row_y)
            } else {
                None
            }
        } else {
            None
        };

        let mut table_width = row_col_x
            .iter()
            .map(|rx| rx.last().copied().unwrap_or(0.0))
            .fold(col_x.last().copied().unwrap_or(0.0), f64::max);
        // [#4042 버그 A] 셀 안 중첩 표(depth>0, 비-TAC)의 렌더 폭은 표 선언 폭(=부모 셀
        // full 폭)으로 결정되는데, 호출자(table_partial.rs:1309 continuation, table_layout.rs
        // 정상 비-TAC 셀 경로)는 이미 패딩을 뺀 col_area(inner_width)를 넘기고 원점도
        // compute_table_x_position 이 패딩 반영(inner_x)해 잡는다. 빠진 단계는 폭을 그
        // 안쪽 내용 상자(col_area.width)에 맞춰 clamp 하는 것뿐이라, 원점은 패딩만큼 우측
        // 이동했는데 폭은 full 이라 우측이 pad_left 만큼 셀 밖으로 넘쳐 클립됐다. col_area
        // 에 맞춰 균일 축소해 좌우 원점·폭을 정합시킨다. table_width < col_area.width 인
        // 정상/좁은 표(#3308 가운데 배치)와 TAC 표는 조건상 no-op. 지역변수 스케일링뿐이라
        // cell_units/projection 캐시를 재계산하지 않아 단일 패스 성능 불변.
        // row_col_x 를 함께 축소하지 않으면 셀 내용만 줄고 테두리 세로선이 full 로 남아
        // 우측 세로선이 어긋나므로 col_widths·col_x·row_col_x·table_width 를 동일 fit 로 축소.
        // fit 타깃은 col_area.width 가 아니라 원점 로직(compute_table_x_position depth>0
        // 분기, 2573-2575)이 실제로 쓰는 가용 폭 `area_w = col_area.width - om_left` 와
        // 정확히 일치시킨다. 표 원점이 col_area.x + om_left 로 밀리므로, 폭을 col_area.width
        // 로 맞추면 om_left(예: 조문대비표 ≈1.9px)만큼 우측이 여전히 초과한다. area_w 로
        // 맞추면 표 우측 = (col_area.x + om_left) + area_w = col_area.x + col_area.width 로
        // 셀 내용 우측에 정확히 flush 된다.
        let fit_om_left = hwpunit_to_px(table.outer_margin_left as i32, self.dpi);
        let fit_avail_w = (col_area.width - fit_om_left).max(0.0);
        // [#4042 버그 A] 다중열(col_count>1) 중첩 표만 대상. 1×1(단일 셀) 중첩 표는
        // 셀 자체가 표 폭이라 render_normalization 이 부모 셀에 맞춰 스트레치(#2195/#4058
        // 76076)하는 것이 정답 기하이며, 여기서 col_area.width 로 되축소하면 그 스트레치
        // 를 되돌려 nested fragment 기하가 어긋난다(issue_2308 회귀). 우측 클립 defect 는
        // 열 경계 합이 셀 내용 상자를 넘는 다중열 표의 증상이므로 col_count>1 로 한정한다
        // (케이스별 구조 가드).
        if depth > 0
            && table.col_count > 1
            && !table.common.treat_as_char
            && table_width > fit_avail_w + 0.5
        {
            let fit = fit_avail_w / table_width;
            for w in col_widths.iter_mut() {
                *w *= fit;
            }
            for x in col_x.iter_mut() {
                *x *= fit;
            }
            for rx in row_col_x.iter_mut() {
                for x in rx.iter_mut() {
                    *x *= fit;
                }
            }
            table_width *= fit;
        }
        let table_height = if let Some(col_row_y) = independent_col_row_y.as_ref() {
            col_row_y
                .iter()
                .filter_map(|cy| cy.last().copied())
                .fold(row_y.last().copied().unwrap_or(0.0), f64::max)
        } else if let Some((_, er)) = split_row_range {
            row_y[er].max(0.0)
        } else {
            row_y.last().copied().unwrap_or(0.0)
        };

        // ── 3. 위치 결정 ──
        let pw = self.current_paper_width.get();
        let paper_w = if pw > 0.0 { Some(pw) } else { None };
        let mut table_x = self.compute_table_x_position(
            table,
            table_width,
            col_area,
            depth,
            host_alignment,
            host_margin_left,
            host_margin_right,
            inline_x_override,
            paper_w,
        );

        let render_caption = should_render_table_caption(table, depth);
        let (caption_height, caption_spacing) = if render_caption {
            let ch = self.calculate_caption_height(&table.caption, styles);
            let cs = table
                .caption
                .as_ref()
                .map(|c| hwpunit_to_px(c.spacing as i32, self.dpi))
                .unwrap_or(0.0);
            (ch, cs)
        } else {
            (0.0, 0.0)
        };

        // Left 캡션: 표를 캡션 크기만큼 오른쪽으로 이동
        if render_caption {
            if let Some(ref cap) = table.caption {
                if matches!(cap.direction, crate::model::shape::CaptionDirection::Left) {
                    let cap_w = hwpunit_to_px(cap.width as i32, self.dpi);
                    table_x += cap_w + caption_spacing;
                }
            }
        }
        if physical_outer_box_paint_inset {
            table_x += hwpunit_to_px(table.outer_margin_left as i32, self.dpi);
        }

        let table_text_wrap = if depth == 0 {
            table.common.text_wrap
        } else {
            crate::model::shape::TextWrap::Square
        };
        let inline_top_caption_offset = if inline_x_override.is_some() && render_caption {
            top_caption_flow_extra(&table.caption, caption_height, caption_spacing)
        } else {
            0.0
        };

        // inline_x_override가 있으면 외부에서 inline 위치를 계산했으므로 x/y 기준은 유지한다.
        // 단, Top 캡션은 표 본문 위의 별도 영역이므로 표 본문 y 에 캡션 높이만큼 반영한다.
        let flow_table_y = if inline_x_override.is_some() {
            y_start + inline_top_caption_offset
        } else {
            let computed_y = self.compute_table_y_position(
                table,
                table_height,
                y_start,
                col_area,
                depth,
                caption_height,
                caption_spacing,
                para_y,
                allow_para_top_bleed,
            );
            if depth > 0 && render_caption {
                computed_y + top_caption_flow_extra(&table.caption, caption_height, caption_spacing)
            } else {
                computed_y
            }
        };
        let table_y = flow_table_y
            + if physical_outer_box_paint_inset {
                hwpunit_to_px(table.outer_margin_top as i32, self.dpi)
            } else {
                0.0
            };
        let inline_table_flow_y_shift = if inline_x_override.is_some() {
            para_y
                .map(|anchor_y| (flow_table_y - anchor_y).max(0.0))
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // ── 4. 표 노드 생성 ──
        let table_id = tree.next_id();
        let mut table_node = RenderNode::new(
            table_id,
            RenderNodeType::Table(TableNode {
                row_count: table.row_count,
                col_count: table.col_count,
                border_fill_id: table.border_fill_id,
                section_index: Some(section_index),
                para_index: table_meta.map(|(pi, _)| pi),
                control_index: table_meta.map(|(_, ci)| ci),
                // [#4334] 셀 안에 중첩된 표(nested table)의 문서 경로 — 최외곽 표는 None.
                cell_context: enclosing_cell_ctx.clone(),
            }),
            BoundingBox::new(table_x, table_y, table_width, table_height),
        );

        // ── 4-1. 표 배경 렌더링 (표 > 배경 > 색 > 면색) ──
        if table.border_fill_id > 0 {
            let tbl_idx = (table.border_fill_id as usize).saturating_sub(1);
            if let Some(tbl_bs) = styles.border_styles.get(tbl_idx) {
                self.render_cell_background(
                    tree,
                    &mut table_node,
                    Some(tbl_bs),
                    table_x,
                    table_y,
                    table_width,
                    table_height,
                    bin_data_content,
                );
            }
        }

        // ── 4-2. cellzone 배경 렌더링 (zone 전체 영역에 한 번) ──
        let mut cellzone_diagonal_nodes = Vec::new();
        let mut cellzone_diagonal_origin_covered = vec![vec![false; col_count]; row_count];
        for zone in &table.zones {
            if zone.border_fill_id == 0 {
                continue;
            }
            let zone_idx = (zone.border_fill_id as usize).saturating_sub(1);
            if let Some(zone_bs) = styles.border_styles.get(zone_idx) {
                // zone 영역의 좌표 계산
                let sc = zone.start_col as usize;
                let ec = (zone.end_col as usize + 1).min(col_count);
                let sr = zone.start_row as usize;
                let er = (zone.end_row as usize + 1).min(row_count);
                if sc < col_count && sr < row_count {
                    let zone_x = table_x
                        + row_col_x
                            .get(sr)
                            .and_then(|r| r.get(sc))
                            .copied()
                            .unwrap_or(0.0);
                    let zone_y = table_y + row_y.get(sr).copied().unwrap_or(0.0);
                    let zone_x_end = table_x
                        + row_col_x
                            .get(sr)
                            .and_then(|r| {
                                if ec < r.len() {
                                    Some(r[ec])
                                } else {
                                    r.last().map(|&last_x| {
                                        // 마지막 열 끝 = 마지막 열 시작 + 해당 셀 너비
                                        let last_col = r.len() - 1;
                                        table
                                            .cells
                                            .iter()
                                            .find(|c| {
                                                c.row as usize == sr && c.col as usize == last_col
                                            })
                                            .map(|c| {
                                                last_x + hwpunit_to_px(c.width as i32, self.dpi)
                                            })
                                            .unwrap_or(last_x)
                                    })
                                }
                            })
                            .unwrap_or(0.0);
                    let zone_y_end = table_y
                        + row_y.get(er).copied().unwrap_or_else(|| {
                            // 마지막 행 끝 = 마지막 행 시작 + 해당 행 높이
                            row_y.get(er - 1).copied().unwrap_or(0.0)
                                + table
                                    .row_sizes
                                    .get(er - 1)
                                    .map(|&h| hwpunit_to_px(h as i32, self.dpi))
                                    .unwrap_or(0.0)
                        });
                    let zone_w = (zone_x_end - zone_x).max(0.0);
                    let zone_h = (zone_y_end - zone_y).max(0.0);
                    // [Task #429] 단색/패턴/그라데이션 + 이미지 채우기 (zone 의 별도 image fill 처리는
                    // render_cell_background 가 통합 처리하므로 제거)
                    self.render_cell_background(
                        tree,
                        &mut table_node,
                        Some(zone_bs),
                        zone_x,
                        zone_y,
                        zone_w,
                        zone_h,
                        bin_data_content,
                    );
                    if border_style_has_diagonal(zone_bs)
                        && !cellzone_diagonal_fully_overridden_by_cells(
                            table,
                            styles,
                            sr,
                            er,
                            sc,
                            ec,
                            zone.border_fill_id,
                        )
                    {
                        mark_cellzone_diagonal_origin_coverage(
                            &mut cellzone_diagonal_origin_covered,
                            sr,
                            sc,
                        );
                        cellzone_diagonal_nodes.extend(render_cell_diagonal(
                            tree, zone_bs, zone_x, zone_y, zone_w, zone_h,
                        ));
                    }
                }
            }
        }

        // ── 5. 셀 레이아웃 ──
        let mut h_edges: Vec<Vec<Option<BorderLine>>> = vec![vec![None; col_count]; row_count + 1];
        let mut v_edges: Vec<Vec<Option<BorderLine>>> = vec![vec![None; row_count]; col_count + 1];

        self.layout_table_cells(
            tree,
            &mut table_node,
            table,
            section_index,
            styles,
            outline_numbering_id,
            col_area,
            bin_data_content,
            depth,
            table_meta,
            outer_host_stored_vpos_hu,
            enclosing_cell_ctx.clone(),
            &row_col_x,
            &row_y,
            independent_col_row_y.as_deref(),
            col_count,
            row_count,
            table_x,
            table_y,
            &mut h_edges,
            &mut v_edges,
            split_row_range,
            row_y_shift,
            split_y_offset,
            scalar_single_row_continuation,
            scalar_single_row_continuation_offset,
            scalar_single_row_fragment,
            scalar_single_row_fragment_content_offset,
            scalar_force_source_start_cut,
            scalar_replay_terminal_boundary_unit,
            split_terminal,
            clamp_header_negative_para_offset,
            inline_table_flow_y_shift,
            // HWP5에서는 표 안의 비글자 1×1 표가 `inMargin=(0,0,141,141)`를
            // 갖더라도 셀의 작은 좌우 저장 margin을 계속 적용하는 형상이 있다.
            // 일반 최상위 표의 #2195 pad 사다리는 유지하고, 실제 셀 내부 중첩에만
            // 문맥을 제한한다 (#2308 HWP 2024 PDF p34). MasterPage/footer의
            // 중첩 표는 #2195의 일반 aim=false 표 여백을 써야 하므로 제외한다
            // (exam_social p2 footer 번호 회귀).
            depth > 0
                && !table.common.treat_as_char
                // p34의 일반 non-TAC nested table은 saved 510HU margin을 유지해
                // 우측 테두리 침범을 막는다. 반면 RenderNormalizationOverlay가
                // 표시한 native short RowBreak child는 parent owner content box를
                // 그대로 써야 한컴 PDF의 p81 `… 등의 사고` wrap이 재현된다.
                && !self
                    .render_normalization_overlay()
                    .uses_owner_content_box(table)
                && !matches!(
                    col_node.node_type,
                    RenderNodeType::Header | RenderNodeType::Footer | RenderNodeType::MasterPage
                ),
            &cellzone_diagonal_origin_covered,
        );

        if !cellzone_diagonal_nodes.is_empty() {
            table_node.children.extend(cellzone_diagonal_nodes);
        }

        // ── 5-1. 표 전체 외곽 테두리 보충 ──
        // 셀 테두리만으로는 표 외곽이 비어있을 수 있음.
        // 셀이 해당 외곽 엣지를 커버하지 않는 곳에만 table.border_fill_id fallback 적용.
        // (셀이 존재하지만 의도적으로 테두리를 없앤 곳에는 적용하지 않음)
        if table.border_fill_id > 0 {
            let tbl_idx = (table.border_fill_id as usize).saturating_sub(1);
            if let Some(tbl_bs) = styles.border_styles.get(tbl_idx) {
                let borders = &tbl_bs.borders; // [left, right, top, bottom]

                // 셀이 커버하는 외곽 엣지 맵 구축
                let mut h_covered = vec![vec![false; col_count]; row_count + 1];
                let mut v_covered = vec![vec![false; row_count]; col_count + 1];
                for cell in &table.cells {
                    let c = cell.col as usize;
                    let r = cell.row as usize;
                    if c >= col_count || r >= row_count {
                        continue;
                    }
                    let ec = (c + cell.col_span as usize).min(col_count);
                    let er = (r + cell.row_span as usize).min(row_count);
                    // 상단
                    if r == 0 {
                        for cc in c..ec {
                            h_covered[0][cc] = true;
                        }
                    }
                    // 하단
                    if er == row_count {
                        for cc in c..ec {
                            h_covered[row_count][cc] = true;
                        }
                    }
                    // 좌측
                    if c == 0 {
                        for rr in r..er {
                            v_covered[0][rr] = true;
                        }
                    }
                    // 우측
                    if ec == col_count {
                        for rr in r..er {
                            v_covered[col_count][rr] = true;
                        }
                    }
                }

                // 셀이 커버하지 않는 외곽 엣지에만 fallback 적용
                for c in 0..col_count {
                    if h_edges[0][c].is_none() && !h_covered[0][c] {
                        let b = &borders[2];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) {
                            h_edges[0][c] = Some(*b);
                        }
                    }
                    if h_edges[row_count][c].is_none() && !h_covered[row_count][c] {
                        let b = &borders[3];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) {
                            h_edges[row_count][c] = Some(*b);
                        }
                    }
                }
                for r in 0..row_count {
                    if v_edges[0][r].is_none() && !v_covered[0][r] {
                        let b = &borders[0];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) {
                            v_edges[0][r] = Some(*b);
                        }
                    }
                    if v_edges[col_count][r].is_none() && !v_covered[col_count][r] {
                        let b = &borders[1];
                        if !matches!(b.line_type, crate::model::style::BorderLineType::None) {
                            v_edges[col_count][r] = Some(*b);
                        }
                    }
                }
            }
        }

        // ── 6. 테두리 렌더링 ──
        if independent_col_row_y.is_none() {
            let body_top_clip = (depth == 0
                && self.is_body_flow_col_area(col_area)
                && (table_y - col_area.y).abs() <= 0.5)
                .then_some(col_area.y);
            table_node.children.extend(render_edge_borders(
                tree,
                &h_edges,
                &v_edges,
                &row_col_x,
                &row_y,
                table_x,
                table_y,
                body_top_clip,
            ));
            if self.show_transparent_borders.get() {
                table_node.children.extend(render_transparent_borders(
                    tree, &h_edges, &v_edges, &row_col_x, &row_y, table_x, table_y,
                ));
            }
        }

        // Cell children may complete normal table-edge rendering only after
        // the parent cell loop. Correct their horizontal clip at this point
        // without changing the vertical continuation viewport.
        extend_completed_nested_table_border_clips(
            tree,
            &mut table_node,
            self.profile.get().native_hwp5_layout() || self.profile.get().hwp5_origin_hwpx(),
            self.profile.get().hwpx_container(),
        );

        col_node.children.push(table_node);

        // ── 7. 캡션 렌더링 ──
        if render_caption {
            if let Some(ref caption) = table.caption {
                use crate::model::shape::{CaptionDirection, CaptionVertAlign};
                let (cap_x, cap_w, cap_y) = match caption.direction {
                    CaptionDirection::Top => (table_x, table_width, y_start),
                    CaptionDirection::Bottom => (
                        table_x,
                        table_width,
                        table_y + table_height + caption_spacing,
                    ),
                    CaptionDirection::Left | CaptionDirection::Right => {
                        let cw = hwpunit_to_px(caption.width as i32, self.dpi);
                        let cx = if caption.direction == CaptionDirection::Left {
                            table_x - cw - caption_spacing
                        } else {
                            table_x + table_width + caption_spacing
                        };
                        let cy = match caption.vert_align {
                            CaptionVertAlign::Top => table_y,
                            CaptionVertAlign::Center => {
                                table_y + (table_height - caption_height).max(0.0) / 2.0
                            }
                            CaptionVertAlign::Bottom => {
                                table_y + (table_height - caption_height).max(0.0)
                            }
                        };
                        (cx, cw, cy)
                    }
                };
                let cap_cell_ctx = table_meta
                    .map(|(pi, ci)| CellContext {
                        parent_para_index: pi,
                        path: vec![CellPathEntry {
                            control_index: ci,
                            cell_index: 65534, // 캡션 식별 센티널
                            cell_para_index: 0,
                            text_direction: 0,
                        }],
                    })
                    .or_else(|| {
                        enclosing_cell_ctx.as_ref().map(|ctx| {
                            let mut cc = ctx.clone();
                            if let Some(last) = cc.path.last_mut() {
                                last.cell_index = 65534;
                                last.cell_para_index = 0;
                            }
                            cc
                        })
                    });
                self.layout_caption(
                    tree,
                    col_node,
                    caption,
                    styles,
                    col_area,
                    cap_x,
                    cap_w,
                    cap_y,
                    &mut self.auto_counter.borrow_mut(),
                    bin_data_content,
                    cap_cell_ctx,
                );
            }
        }

        // ── 8. 반환값 ──
        if depth == 0 {
            // Left/Right 캡션은 표 높이에 영향 없음
            let is_lr_cap = table.caption.as_ref().map_or(false, |c| {
                use crate::model::shape::CaptionDirection;
                matches!(
                    c.direction,
                    CaptionDirection::Left | CaptionDirection::Right
                )
            });
            let caption_extra = if is_lr_cap {
                0.0
            } else {
                caption_height
                    + if caption_height > 0.0 {
                        caption_spacing
                    } else {
                        0.0
                    }
            };
            if matches!(
                table_text_wrap,
                crate::model::shape::TextWrap::BehindText
                    | crate::model::shape::TextWrap::InFrontOfText
            ) {
                // 글뒤로/글앞으로: y_offset 변경 없음
                y_start
            } else if matches!(table_text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                && !table.common.treat_as_char
            {
                // 자리차지: 표 아래쪽까지 y_offset 진행 (절대 위치 기준)
                let table_bottom = table_y + table_height + caption_extra;
                table_bottom.max(y_start)
            } else {
                let total_height = table_height + caption_extra;
                y_start + total_height
            }
        } else {
            // 중첩 표: outer_margin 포함 높이 반환
            let om_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
            let om_bottom = hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi);
            (table_height
                + caption_flow_extra(&table.caption, caption_height, caption_spacing)
                + om_top
                + om_bottom)
                .max(0.0)
        }
    }

    /// 열 폭 계산 (단일 셀 + 병합 셀 해결)
    pub(crate) fn resolve_column_widths(
        &self,
        table: &crate::model::table::Table,
        col_count: usize,
    ) -> Vec<f64> {
        let width_scale = self.render_table_width_scale(table);
        // 1단계: col_span==1인 셀에서 개별 열 폭 추출
        let base_grid_outlier_rows = table.base_grid_outlier_rows();
        let mut col_widths = vec![0.0f64; col_count];
        for cell in &table.cells {
            if table.local_resize_rows.contains(&cell.row)
                || base_grid_outlier_rows.contains(&cell.row)
            {
                continue;
            }
            if cell.col_span == 1 && (cell.col as usize) < col_count {
                let w = hwpunit_to_px(cell.width as i32, self.dpi) * width_scale;
                if w > col_widths[cell.col as usize] {
                    col_widths[cell.col as usize] = w;
                }
            }
        }

        // 2단계: 병합 셀에서 미지 열 폭을 반복적으로 해결
        {
            let mut constraints: Vec<(usize, usize, f64)> = Vec::new();
            for cell in &table.cells {
                if table.local_resize_rows.contains(&cell.row)
                    || base_grid_outlier_rows.contains(&cell.row)
                {
                    continue;
                }
                let c = cell.col as usize;
                let span = cell.col_span as usize;
                if span > 1 && c + span <= col_count {
                    let total_w = hwpunit_to_px(cell.width as i32, self.dpi) * width_scale;
                    if let Some(existing) = constraints.iter_mut().find(|x| x.0 == c && x.1 == span)
                    {
                        if total_w > existing.2 {
                            existing.2 = total_w;
                        }
                    } else {
                        constraints.push((c, span, total_w));
                    }
                }
            }
            constraints.sort_by_key(|&(_, span, _)| span);

            let max_iter = col_count + constraints.len();
            for _ in 0..max_iter {
                let mut progress = false;
                for &(c, span, total_w) in &constraints {
                    let known_sum: f64 = (c..c + span).map(|i| col_widths[i]).sum();
                    let unknown_cols: Vec<usize> =
                        (c..c + span).filter(|&i| col_widths[i] == 0.0).collect();
                    if unknown_cols.len() == 1 {
                        let remaining = (total_w - known_sum).max(0.0);
                        col_widths[unknown_cols[0]] = remaining;
                        progress = true;
                    }
                }
                if !progress {
                    break;
                }
            }

            for &(c, span, total_w) in &constraints {
                let known_sum: f64 = (c..c + span).map(|i| col_widths[i]).sum();
                let unknown_cols: Vec<usize> =
                    (c..c + span).filter(|&i| col_widths[i] == 0.0).collect();
                if !unknown_cols.is_empty() {
                    let remaining = (total_w - known_sum).max(0.0);
                    let per_col = remaining / unknown_cols.len() as f64;
                    for i in unknown_cols {
                        col_widths[i] = per_col;
                    }
                }
            }

            // 병합 셀 제약이 이미 값이 있는 열들로만 구성되어도 총합이 더 클 수 있다.
            // 한컴은 이 경우 뒤쪽 열을 확장해 병합 셀 폭을 만족시킨다.
            for &(c, span, total_w) in &constraints {
                let known_sum: f64 = (c..c + span).map(|i| col_widths[i]).sum();
                let deficit = total_w - known_sum;
                if deficit > 0.5 {
                    let target_col = c + span - 1;
                    if target_col < col_widths.len() {
                        col_widths[target_col] += deficit;
                    }
                }
            }
        }

        // 3단계: 여전히 폭이 0인 열에 기본값 할당
        for c in 0..col_count {
            if col_widths[c] <= 0.0 {
                col_widths[c] = hwpunit_to_px(1800, self.dpi);
            }
        }
        let target_width = if table.common.width > 0 {
            hwpunit_to_px(table.common.width as i32, self.dpi) * width_scale
        } else {
            0.0
        };
        if target_width > 0.0 {
            let current: f64 = col_widths.iter().sum();
            let residual = target_width - current;
            if residual > 0.5 {
                if let Some(last) = col_widths.last_mut() {
                    *last += residual;
                }
            }
        }
        col_widths
    }

    /// 행 높이 계산 (MeasuredTable 우선, 없으면 셀/병합/컨텐츠 기반)
    pub(crate) fn resolve_row_heights(
        &self,
        table: &crate::model::table::Table,
        col_count: usize,
        row_count: usize,
        measured_table: Option<&MeasuredTable>,
        styles: &ResolvedStyleSet,
        relaxed_pad: bool,
    ) -> Vec<f64> {
        self.resolve_row_heights_with_common_fit(
            table,
            col_count,
            row_count,
            measured_table,
            styles,
            true,
            relaxed_pad,
        )
    }

    fn resolve_row_heights_for_content(
        &self,
        table: &crate::model::table::Table,
        col_count: usize,
        row_count: usize,
        measured_table: Option<&MeasuredTable>,
        styles: &ResolvedStyleSet,
        relaxed_pad: bool,
    ) -> Vec<f64> {
        self.resolve_row_heights_with_common_fit(
            table,
            col_count,
            row_count,
            measured_table,
            styles,
            false,
            relaxed_pad,
        )
    }

    /// [Task #2211] 셀의 전 문단이 저장 LINE_SEG 를 보유하는지 — 보유 셀은
    /// 한컴이 저장 시 셀 h 를 콘텐츠에 맞춰 확정했으므로 행 성장 판정에서
    /// 저장 지오메트리를 그대로 신뢰한다 (#2112 계보). 합성 seg(tag bit31)는
    /// 저장으로 치지 않는다 — height_measurer 와 동일 술어.
    fn cell_has_stored_line_segs(cell: &crate::model::table::Cell) -> bool {
        !cell.paragraphs.is_empty()
            && cell
                .paragraphs
                .iter()
                .all(|p| !crate::renderer::para_has_no_stored_line_segs(p))
    }

    /// [#3386] MeasuredTable 행높이를 행별 저장 선언(cellSz)으로 교정한다.
    /// 발동 조건(전부 충족 시에만):
    /// - 모든 셀이 row_span==1 이고 저장 LINE_SEG 를 보유(#2211 술어)
    /// - 모든 행에 유효 선언 높이 존재(cell.height < 0x8000_0000)
    /// - 선언 합 == 측정 합 (±1.5px; 총높이 보존 → 쪽수·후속 흐름 불변)
    /// - 행별 |선언-측정| <= max(12px, 선언의 15%) (실콘텐츠 성장 행 보호)
    fn trust_declared_row_heights(
        &self,
        table: &crate::model::table::Table,
        row_count: usize,
        rh: &mut [f64],
    ) {
        if row_count == 0 || rh.len() < row_count {
            return;
        }
        let mut decl = vec![f64::NAN; row_count];
        for cell in &table.cells {
            if cell.row_span != 1 {
                return;
            }
            if !Self::cell_has_stored_line_segs(cell) {
                return;
            }
            let r = cell.row as usize;
            if r >= row_count || cell.height >= 0x8000_0000 {
                return;
            }
            let h = hwpunit_to_px(cell.height as i32, self.dpi);
            if decl[r].is_nan() || h > decl[r] {
                decl[r] = h;
            }
        }
        if decl.iter().any(|d| d.is_nan()) {
            return;
        }
        let decl_sum: f64 = decl.iter().sum();
        let measured_sum: f64 = rh[..row_count].iter().sum();
        if (decl_sum - measured_sum).abs() > 1.5 {
            return;
        }
        for r in 0..row_count {
            if (decl[r] - rh[r]).abs() > (decl[r] * 0.15).max(12.0) {
                return;
            }
        }
        rh[..row_count].copy_from_slice(&decl);
    }

    fn resolve_row_heights_with_common_fit(
        &self,
        table: &crate::model::table::Table,
        col_count: usize,
        row_count: usize,
        measured_table: Option<&MeasuredTable>,
        styles: &ResolvedStyleSet,
        fit_common_height: bool,
        relaxed_pad: bool,
    ) -> Vec<f64> {
        if let Some(mt) = measured_table {
            // `TypesetEngine::format_table` uses this same narrow replacement for
            // native HWP5 empty RowBreak hosts.  Layout must consume the identical
            // row geometry: otherwise pagination reserves the declared tail height
            // but the SVG layout paints the old over-measured table and moves every
            // following table back down (76076 p81→82).  The helper verifies the
            // actual last-row 1×1 block-child shape and the bounded drift; this
            // outer gate confines it to the native TopAndBottom RowBreak contract.
            let native_rowbreak_nested_tail = self.profile.get().hwp5_stored_pagination_layout()
                && !table.common.treat_as_char
                && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
                && matches!(table.common.vert_rel_to, VertRelTo::Para)
                && matches!(table.page_break, TablePageBreak::RowBreak)
                && table.row_count > 1
                && table.cells.iter().all(|cell| cell.row_span == 1);
            let tail_fitted = native_rowbreak_nested_tail
                .then(|| fit_measured_table_nested_tail_to_declared_height(mt, table, self.dpi))
                .flatten();
            let measured = tail_fitted.as_ref().unwrap_or(mt);
            let mut rh = measured.row_heights.clone();
            rh.resize(row_count, hwpunit_to_px(400, self.dpi));
            // [#3386] 행별 저장-선언 신뢰(렌더 전용): 전 셀이 rowspan 없이 저장
            // LINE_SEG 를 보유하고 행별 선언(cellSz)이 모두 존재하며, 선언 합이
            // 측정 합과 일치(±1.5px)하고 행별 편차가 max(12px,15%) 이내면 행
            // 경계는 선언을 따른다. 한글 PDF 실측(156678235 p5 pi=46): 한글
            // 행경계 == cellSz [22.4,41.9,69.4], rhwp 측정 재분배 [19.8,44.8,
            // 69.1] 은 폰트 메트릭 차로 행 경계만 드리프트 — 총높이 동일.
            self.trust_declared_row_heights(table, row_count, &mut rh);
            if fit_common_height {
                self.fit_row_heights_to_common_height(table, &mut rh);
            }
            return rh;
        }

        // 1단계: row_span==1인 셀에서 개별 행 높이 추출
        let mut row_heights = vec![0.0f64; row_count];
        for cell in &table.cells {
            if table.local_resize_cols.contains(&cell.col) {
                continue;
            }
            if cell.row_span == 1 && (cell.row as usize) < row_count {
                let r = cell.row as usize;
                if cell.height < 0x80000000 {
                    let h = hwpunit_to_px(cell.height as i32, self.dpi);
                    if h > row_heights[r] {
                        row_heights[r] = h;
                    }
                }
            }
        }

        // 1-b단계: 셀 내 실제 컨텐츠 높이 계산
        for cell in &table.cells {
            if table.local_resize_cols.contains(&cell.col) {
                continue;
            }
            if cell.row_span == 1 && (cell.row as usize) < row_count {
                let r = cell.row as usize;
                let (pad_left, pad_right, pad_top, pad_bottom) =
                    self.resolve_cell_padding(cell, table);

                // LINE_SEG의 line_height에 이미 셀 내 중첩 표 높이가 반영되어 있으므로
                // controls_height를 별도로 더하면 이중 계산됨
                // [Task #2211] 저장 LINE_SEG 보유 셀의 줄 흐름은 성장 판정에 pad 를
                // 더하지 않는다 — 한컴 저장 h 는 콘텐츠에 꽉 맞게 저장되며(빈 셀
                // lh=h), pad 가산 시 그런 행마다 +pad 상하합(주보 p1: 행당 +282HU)씩
                // 부풀어 하단이 절단된다. 개체 기반 지오메트리(Square bottom 등,
                // #1486 p19)와 LINE_SEG 부재(합성 줄) 셀은 pad 포함 유지.
                let required_height = if cell.text_direction != 0 {
                    // 세로쓰기: line_seg.segment_width가 열의 세로 길이
                    self.calc_vertical_cell_content_height(&cell.paragraphs) + pad_top + pad_bottom
                } else {
                    let cell_w_px = hwpunit_to_px(cell.width as i32, self.dpi)
                        * self.render_table_width_scale(table);
                    let inner_width = (cell_w_px - pad_left - pad_right).max(0.0);
                    let (line_based, object_based) = self.calc_cell_paragraphs_content_parts(
                        &cell.paragraphs,
                        styles,
                        inner_width,
                    );
                    // [#3386] 저장 cellSz 가 저장 줄 흐름보다 작은 모순 셀은 한글이
                    // 줄 흐름 + 상하 여백으로 재성장한다 (156678235 p5 내부 표 r0:
                    // cellSz 3.8px·lineseg 14.7px → 한글 PDF 실측 18.4px = 14.7+1.9×2).
                    // 선언이 줄 흐름을 수용하는 셀은 종전대로 pad 미가산 (#2211 유지).
                    let line_req = if relaxed_pad && Self::cell_has_stored_line_segs(cell) {
                        let decl_h = if cell.height < 0x8000_0000 {
                            hwpunit_to_px(cell.height as i32, self.dpi)
                        } else {
                            f64::MAX
                        };
                        if line_based > decl_h + 1.5 {
                            // 한글 실좌표는 원(cellMargin) 상하 여백 가산 — resolve
                            // 축소 pad(0.9×2)가 아니라 저장 1.9×2 로 18.4px 재현.
                            let raw_pad_v = hwpunit_to_px(cell.padding.top as i32, self.dpi)
                                + hwpunit_to_px(cell.padding.bottom as i32, self.dpi);
                            line_based + (pad_top + pad_bottom).max(raw_pad_v)
                        } else {
                            line_based
                        }
                    } else {
                        line_based + pad_top + pad_bottom
                    };
                    let object_req = if object_based > 0.0 {
                        object_based + pad_top + pad_bottom
                    } else {
                        0.0
                    };
                    line_req.max(object_req)
                };
                if required_height > row_heights[r] {
                    row_heights[r] = required_height;
                }
            }
        }

        // 2단계: 병합 셀에서 미지 행 높이를 반복적으로 해결
        {
            let mut constraints: Vec<(usize, usize, f64)> = Vec::new();
            for cell in &table.cells {
                if table.local_resize_cols.contains(&cell.col) {
                    continue;
                }
                let r = cell.row as usize;
                let span = cell.row_span as usize;
                if span > 1 && r + span <= row_count && cell.height < 0x80000000 {
                    let total_h = hwpunit_to_px(cell.height as i32, self.dpi);
                    if let Some(existing) = constraints.iter_mut().find(|x| x.0 == r && x.1 == span)
                    {
                        if total_h > existing.2 {
                            existing.2 = total_h;
                        }
                    } else {
                        constraints.push((r, span, total_h));
                    }
                }
            }
            constraints.sort_by_key(|&(_, span, _)| span);
            let max_iter = row_count + constraints.len();
            for _ in 0..max_iter {
                let mut progress = false;
                for &(r, span, total_h) in &constraints {
                    let known_sum: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                    let unknown_rows: Vec<usize> =
                        (r..r + span).filter(|&i| row_heights[i] == 0.0).collect();
                    if unknown_rows.len() == 1 {
                        let remaining = (total_h - known_sum).max(0.0);
                        row_heights[unknown_rows[0]] = remaining;
                        progress = true;
                    }
                }
                if !progress {
                    break;
                }
            }
            for &(r, span, total_h) in &constraints {
                let known_sum: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                let unknown_rows: Vec<usize> =
                    (r..r + span).filter(|&i| row_heights[i] == 0.0).collect();
                if !unknown_rows.is_empty() {
                    let remaining = (total_h - known_sum).max(0.0);
                    let per_row = remaining / unknown_rows.len() as f64;
                    for i in unknown_rows {
                        row_heights[i] = per_row;
                    }
                }
            }
            // [#2291/#2237] 병합 셀 **선언** 높이가 걸친 행합을 초과하면 잔여를
            // 마지막 걸침 행에 가산한다 — 한글 관례 실측(연결맵 244×10 r183:
            // c3 rs=4 선언 217.8px vs 행합 201.3px, 한글 행 괘선 실측 r183 =
            // 39.8+16.5 = 56.3px 정확 일치). 종전에는 모든 행이 rs=1 선언으로
            // 채워진(미지 행 없음) 표에서 이 잔여가 지면에서 소실되어, rowspan
            // 중첩 문서가 한글보다 쪽당 +15% 조밀해졌다(연결맵 −35쪽의 지배
            // 성분). 콘텐츠 기반 확장(2-b)과 별개의 선언 기반 규칙이다.
            for &(r, span, total_h) in &constraints {
                let known_sum: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                if total_h > known_sum + 0.5 {
                    row_heights[r + span - 1] += total_h - known_sum;
                }
            }
        }

        // 2-b단계: 병합 셀 컨텐츠 높이 > 결합 행 높이이면 마지막 행 확장
        for cell in &table.cells {
            if table.local_resize_cols.contains(&cell.col) {
                continue;
            }
            let r = cell.row as usize;
            let span = cell.row_span as usize;
            if span > 1 && r + span <= row_count {
                let (pad_left, pad_right, pad_top, pad_bottom) =
                    self.resolve_cell_padding(cell, table);
                let cell_w_px = hwpunit_to_px(cell.width as i32, self.dpi)
                    * self.render_table_width_scale(table);
                let inner_width = (cell_w_px - pad_left - pad_right).max(0.0);
                // LINE_SEG의 line_height에 이미 셀 내 중첩 표 높이가 반영되어 있으므로
                // controls_height를 별도로 더하면 이중 계산됨
                // [Task #2211] 1-b 와 동일 — 저장 LINE_SEG 줄 흐름은 pad 미가산,
                // 개체 기반 지오메트리는 pad 가산 유지.
                let (line_based, object_based) =
                    self.calc_cell_paragraphs_content_parts(&cell.paragraphs, styles, inner_width);
                // [#3386] 1-b 와 동일 — 모순 선언(span 합) 초과 성장 시 여백 가산.
                let line_req = if relaxed_pad && Self::cell_has_stored_line_segs(cell) {
                    let decl_h = if cell.height < 0x8000_0000 {
                        hwpunit_to_px(cell.height as i32, self.dpi)
                    } else {
                        f64::MAX
                    };
                    if line_based > decl_h + 1.5 {
                        let raw_pad_v = hwpunit_to_px(cell.padding.top as i32, self.dpi)
                            + hwpunit_to_px(cell.padding.bottom as i32, self.dpi);
                        line_based + (pad_top + pad_bottom).max(raw_pad_v)
                    } else {
                        line_based
                    }
                } else {
                    line_based + pad_top + pad_bottom
                };
                let object_req = if object_based > 0.0 {
                    object_based + pad_top + pad_bottom
                } else {
                    0.0
                };
                let required_height = line_req.max(object_req);
                let combined: f64 = (r..r + span).map(|i| row_heights[i]).sum();
                if required_height > combined {
                    let deficit = required_height - combined;
                    row_heights[r + span - 1] += deficit;
                }
            }
        }

        // 3단계: 높이 0인 행에 기본값
        for r in 0..row_count {
            if row_heights[r] <= 0.0 {
                row_heights[r] = hwpunit_to_px(400, self.dpi);
            }
        }
        if fit_common_height {
            self.fit_row_heights_to_common_height(table, &mut row_heights);
        }
        row_heights
    }

    fn fit_row_heights_to_common_height(
        &self,
        table: &crate::model::table::Table,
        row_heights: &mut [f64],
    ) {
        if row_heights.is_empty() {
            return;
        }
        let target_height = if table.common.height > 0 {
            hwpunit_to_px(table.common.height as i32, self.dpi)
        } else {
            0.0
        };
        if target_height > 0.0 {
            // `common.height` is the stored outer table height: it already
            // contains the gaps between adjacent rows.  `row_heights`, on the
            // other hand, is consumed together with `cell_spacing` when row_y
            // is built.  Giving the full common height to the row sum adds the
            // same gaps a second time, making every multi-row stored table
            // taller by `(row_count - 1) * cell_spacing` at paint time.
            let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
            let target_row_sum = (target_height
                - cell_spacing * row_heights.len().saturating_sub(1) as f64)
                .max(0.0);
            let current: f64 = row_heights.iter().sum();
            let residual = target_row_sum - current;
            if residual > 0.5 {
                if let Some(last) = row_heights.last_mut() {
                    *last += residual;
                }
            }
        }
    }

    /// 셀 문단들의 콘텐츠 높이 합산 (spacing + line_height + line_spacing)
    pub(crate) fn calc_cell_paragraphs_content_height(
        &self,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        cell_inner_width_px: f64,
    ) -> f64 {
        let (line_based, object_based) =
            self.calc_cell_paragraphs_content_parts(paragraphs, styles, cell_inner_width_px);
        line_based.max(object_based)
    }

    /// [Task #2211] 셀 콘텐츠 높이를 (줄 기반, 개체 기반)으로 분리 반환.
    /// 행 성장 판정에서 저장 LINE_SEG 줄 흐름은 pad 미가산, 개체(중첩 표·
    /// TopAndBottom flow·Square bottom) 지오메트리는 pad 가산이 한컴 정합 —
    /// 두 축의 pad 취급이 다르다 (#1486 p19 Square 그림 캘리브레이션).
    pub(crate) fn calc_cell_paragraphs_content_parts(
        &self,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        cell_inner_width_px: f64,
    ) -> (f64, f64) {
        let cell_para_count = paragraphs.len();
        let line_based_height: f64 = paragraphs
            .iter()
            .enumerate()
            .map(|(pidx, p)| {
                let mut comp = compose_paragraph(p);
                // [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine
                // 압축 결과를 셀 가용 너비에 맞춰 다중 ComposedLine 으로 재분할.
                // 측정/렌더링 일관성 보장 (table_layout.rs:1226 의 렌더링 경로와 동일).
                crate::renderer::composer::recompose_for_cell_width(
                    &mut comp,
                    p,
                    cell_inner_width_px,
                    styles,
                );
                self.calc_para_lines_height(
                    &comp.lines,
                    p,
                    self.profile.get().hwp3_layout()
                        && p.line_segs.is_empty()
                        && !p.text.is_empty(),
                    !p.line_segs.is_empty(),
                    pidx,
                    cell_para_count,
                    styles.para_styles.get(p.para_shape_id as usize),
                    styles,
                )
            })
            .sum();
        let object_based = self
            .calc_nested_controls_bottom_height(paragraphs, styles)
            .max(self.calc_non_inline_controls_flow_height(paragraphs))
            .max(self.calc_cell_wrap_objects_bottom_height(paragraphs));
        (line_based_height, object_based)
    }

    /// pre-composed 문단들의 콘텐츠 높이 합산 (compose 생략)
    pub(crate) fn calc_composed_paras_content_height(
        &self,
        composed_paras: &[ComposedParagraph],
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let cell_para_count = paragraphs.len();
        composed_paras
            .iter()
            .zip(paragraphs.iter())
            .enumerate()
            .map(|(pidx, (comp, para))| {
                self.calc_para_lines_height(
                    &comp.lines,
                    para,
                    self.profile.get().hwp3_layout()
                        && para.line_segs.is_empty()
                        && !para.text.is_empty(),
                    !para.line_segs.is_empty(),
                    pidx,
                    cell_para_count,
                    styles.para_styles.get(para.para_shape_id as usize),
                    styles,
                )
            })
            .sum()
    }

    /// 단일 문단의 줄 높이 합산 (공통 로직)
    ///
    /// [Task #674] line_height 측정에 corrected_line_height 보정 적용.
    /// line_segs 부재 paragraph 의 fallback line_height (400 HU = 5.33 px) 가
    /// max_fs 보다 작은 경우 ParaShape 의 line_spacing_type + line_spacing 으로
    /// 보정. height_measurer.rs:570-587 와 동일 로직 — 측정/layout 일관성 보장.
    /// [#2112] `trust_stored_lh`: 실제 저장 LINE_SEG 를 보유한 문단은 저장 줄높이를
    /// 그대로 신뢰한다. 한글은 압축 줄높이(lh < 글자크기)를 저장값대로 렌더하는데,
    /// #674 보정(fs×줄간격% 대체)이 저장 줄에도 적용되어 셀 행높이가 부풀었다
    /// (39607: 행별 +3.8~+76.8px, 표 합계 +335.5px → 다쪽 표 쪽수 밀림).
    /// 보정은 line_segs 부재 폴백(400HU 합성 줄, #671/#674 원 목적)에만 유지.
    fn calc_para_lines_height(
        &self,
        lines: &[crate::renderer::composer::ComposedLine],
        para: &Paragraph,
        hwp3_variant_synthetic: bool,
        trust_stored_lh: bool,
        pidx: usize,
        total_para_count: usize,
        para_style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let is_last_para = pidx + 1 == total_para_count;
        let spacing_before = if pidx > 0 {
            para_style.map(|s| s.spacing_before).unwrap_or(0.0)
        } else {
            0.0
        };
        let spacing_after = if !is_last_para {
            para_style.map(|s| s.spacing_after).unwrap_or(0.0)
        } else {
            0.0
        };
        if lines.is_empty() {
            // [#2169] NO_LS 순수 빈 문단 = em 줄박스 (한글 공식).
            let h = if crate::renderer::para_has_no_stored_line_segs(para)
                && para.controls.is_empty()
            {
                let fs = para
                    .char_shapes
                    .first()
                    .and_then(|cs| styles.char_styles.get(cs.char_shape_id as usize))
                    .map(|cs| cs.font_size)
                    .unwrap_or(0.0);
                if fs <= 0.0 {
                    hwpunit_to_px(400, self.dpi)
                } else if is_last_para {
                    fs
                } else {
                    match para_style {
                        Some(ps) => crate::renderer::corrected_line_height(
                            hwpunit_to_px(400, self.dpi),
                            fs,
                            ps.line_spacing_type,
                            ps.line_spacing,
                        ),
                        None => fs,
                    }
                }
            } else {
                hwpunit_to_px(400, self.dpi)
            };
            spacing_before + h + spacing_after
        } else {
            let cell_ls_val = para_style.map(|s| s.line_spacing).unwrap_or(160.0);
            let cell_ls_type = para_style
                .map(|s| s.line_spacing_type)
                .unwrap_or(crate::model::style::LineSpacingType::Percent);
            let line_count = lines.len();
            let lines_total: f64 = lines
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                    let max_fs = line
                        .runs
                        .iter()
                        .map(|r| {
                            styles
                                .char_styles
                                .get(r.char_style_id as usize)
                                .map(|cs| cs.font_size)
                                .unwrap_or(0.0)
                        })
                        .fold(0.0f64, f64::max);
                    // [#2169] NO_LS 순수 빈 문단 — 문단 char shape fs 폴백 (em 줄박스).
                    let max_fs = if max_fs <= 0.0
                        && crate::renderer::para_has_no_stored_line_segs(para)
                        && para.controls.is_empty()
                    {
                        para.char_shapes
                            .first()
                            .and_then(|cs| styles.char_styles.get(cs.char_shape_id as usize))
                            .map(|cs| cs.font_size)
                            .unwrap_or(0.0)
                    } else {
                        max_fs
                    };
                    let is_cell_last_line = is_last_para && i + 1 == line_count;
                    let h = if trust_stored_lh {
                        raw_lh
                    } else {
                        // [#2150/#2148] 셀 마지막 줄 em 공식 — #2195 축 정합 세트로
                        // 정식화됨 (종전 "[정식화 보류]" 주석은 stage3 실험기 잔재).
                        // [#2070] NO_LS 단일 문단·단일 줄 셀 = em — 한글은 1줄 셀에서
                        // 줄간격(Percent/Fixed)을 완전 무시 (fixed_ladder 실측).
                        crate::renderer::corrected_line_height_for_variant_synthetic(
                            raw_lh,
                            max_fs,
                            cell_ls_type,
                            cell_ls_val,
                            hwp3_variant_synthetic || is_cell_last_line,
                        )
                    };
                    if !is_cell_last_line {
                        h + hwpunit_to_px(line.line_spacing, self.dpi)
                    } else {
                        h
                    }
                })
                .sum();
            spacing_before + lines_total + spacing_after
        }
    }

    /// 세로쓰기 셀의 콘텐츠 높이 계산
    /// 세로쓰기에서 line_seg.segment_width = 열의 세로 길이 (HWPUNIT)
    /// 셀 높이 = 최대 segment_width
    fn calc_vertical_cell_content_height(&self, paragraphs: &[Paragraph]) -> f64 {
        let mut max_seg_height: f64 = 0.0;
        for para in paragraphs {
            for ls in &para.line_segs {
                let h = hwpunit_to_px(ls.segment_width, self.dpi);
                if h > max_seg_height {
                    max_seg_height = h;
                }
            }
        }
        if max_seg_height <= 0.0 {
            // fallback: 기본 높이
            hwpunit_to_px(400, self.dpi)
        } else {
            max_seg_height
        }
    }

    /// 셀 패딩 계산
    pub(crate) fn resolve_cell_padding(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
    ) -> (f64, f64, f64, f64) {
        self.resolve_cell_padding_for_context(cell, table, false)
    }

    fn resolve_cell_padding_for_context(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        allow_saved_small_cell_margin: bool,
    ) -> (f64, f64, f64, f64) {
        // HWP 스펙: aim(apply_inner_margin)=true → cell.padding,
        //           aim=false → table.padding 우선.
        // 한컴은 aim=false일 때 cell.padding 원값을 파일에 보존하더라도 렌더에는 쓰지 않는다.
        // aim=true에서는 0mm도 사용자가 지정한 셀 고유 안 여백으로 존중한다.
        // [#2195 stage50] 표 기본 전축 0 = 미지정 → 셀 pad (Cell::table_padding_unspecified).
        let table_pad_unspec = !cell.apply_inner_margin
            && crate::model::table::Cell::table_padding_unspecified(&table.padding);
        let use_cell_left = (table_pad_unspec && cell.padding.left < 2500)
            || Self::should_use_cell_padding_axis_for_context(
                cell,
                cell.padding.left,
                table.padding.left,
                allow_saved_small_cell_margin,
            );
        let use_cell_right = (table_pad_unspec && cell.padding.right < 2500)
            || Self::should_use_cell_padding_axis_for_context(
                cell,
                cell.padding.right,
                table.padding.right,
                allow_saved_small_cell_margin,
            );
        let use_cell_top = (table_pad_unspec && cell.padding.top < 2500)
            || Self::should_use_cell_padding_axis_for_context(
                cell,
                cell.padding.top,
                table.padding.top,
                allow_saved_small_cell_margin,
            );
        let use_cell_bottom = (table_pad_unspec && cell.padding.bottom < 2500)
            || Self::should_use_cell_padding_axis_for_context(
                cell,
                cell.padding.bottom,
                table.padding.bottom,
                allow_saved_small_cell_margin,
            );

        let pad_left = if use_cell_left {
            hwpunit_to_px(cell.padding.left as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.left as i32, self.dpi)
        };
        let pad_right = if use_cell_right {
            hwpunit_to_px(cell.padding.right as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.right as i32, self.dpi)
        };
        let pad_top = if use_cell_top {
            hwpunit_to_px(cell.padding.top as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.top as i32, self.dpi)
        };
        let pad_bottom = if use_cell_bottom {
            hwpunit_to_px(cell.padding.bottom as i32, self.dpi)
        } else {
            hwpunit_to_px(table.padding.bottom as i32, self.dpi)
        };
        // [Task #501] 한컴 방어 로직 모방 — cell.padding.top + bottom 합산이
        // cell.height 자체를 초과하면 (mel-001 p2 셀[21]: pad=1700 HU 두 축, h=1280 HU)
        // 한컴은 자체 가드로 cell 안에 콘텐츠가 들어가도록 처리. cell.height 의 절반까지
        // 비례 축소 (HWP 스펙 외 한컴 동작 모방).
        let (pad_top, pad_bottom) = if cell.height < 0x80000000 {
            let cell_h_px = hwpunit_to_px(cell.height as i32, self.dpi);
            let total_v_pad = pad_top + pad_bottom;
            if cell_h_px > 0.0 && total_v_pad >= cell_h_px {
                let max_v_pad = cell_h_px * 0.5;
                let scale = max_v_pad / total_v_pad;
                (pad_top * scale, pad_bottom * scale)
            } else {
                (pad_top, pad_bottom)
            }
        } else {
            (pad_top, pad_bottom)
        };
        (pad_left, pad_right, pad_top, pad_bottom)
    }

    fn should_use_cell_padding_axis_for_context(
        cell: &crate::model::table::Cell,
        cell_padding: i16,
        table_padding: i16,
        allow_saved_small_cell_margin: bool,
    ) -> bool {
        // [Task #1785] 규칙 본체는 Cell::use_cell_padding_axis 로 이동 — height_measurer
        // 와 단일 출처 공유 (규칙이 갈리면 예약 높이와 실제 렌더가 어긋난다).
        cell.use_cell_padding_axis(cell_padding, table_padding, allow_saved_small_cell_margin)
    }

    /// 셀 텍스트가 오버플로우할 때 좌우 패딩을 축소하여 공간을 확보한다.
    /// composed 문단의 각 줄 텍스트 폭을 측정하여 최대값이 가용 폭을 초과하면
    /// 패딩을 비례 축소한다 (최소 1px 보장).
    ///
    /// [Task #617] 다중 줄(2 줄 이상) 단락이 있는 셀은 HWP 가 가용 폭에 자간을
    /// 분배·줄바꿈을 확정한 상태이므로 padding 을 보존한다 (자연 폭 추정으로
    /// 다시 깎으면 본문이 테두리에 닿는 시각 오류 발생 — exam_kor.hwp
    /// 16/27/36번 보기 박스). 단일 줄 셀(좁은 수치 셀에서 오버플로우 가능성
    /// 있음) 은 종전 휴리스틱으로 보호한다.
    pub(crate) fn shrink_cell_padding_for_overflow(
        &self,
        pad_left: f64,
        pad_right: f64,
        cell_w: f64,
        composed_paras: &[ComposedParagraph],
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        preserve_cell_padding: bool,
    ) -> (f64, f64) {
        // [#2279 axis B] 규칙 본체는 composer::shrunk_cell_horizontal_padding 로 이동 —
        // cut(cell_units)/mt(HeightMeasurer) 측정과 단일 출처 공유 (규칙이 갈리면
        // 측정 줄수와 실제 렌더 줄수가 어긋난다).
        crate::renderer::composer::shrunk_cell_horizontal_padding(
            pad_left,
            pad_right,
            cell_w,
            composed_paras,
            paragraphs,
            styles,
            preserve_cell_padding,
        )
    }

    /// 셀 배경 렌더링 (fill_color + pattern + gradient)
    pub(crate) fn render_cell_background(
        &self,
        tree: &mut PageLayoutContext,
        cell_node: &mut RenderNode,
        border_style: Option<&crate::renderer::style_resolver::ResolvedBorderStyle>,
        cell_x: f64,
        cell_y: f64,
        cell_w: f64,
        cell_h: f64,
        bin_data_content: &[BinDataContent],
    ) {
        let fill_color = border_style.and_then(|bs| bs.fill_color);
        let pattern = border_style.and_then(|bs| bs.pattern);
        let gradient = border_style.and_then(|bs| bs.gradient.clone());
        if fill_color.is_some() || gradient.is_some() || pattern.is_some() {
            let rect_id = tree.next_id();
            let rect_node = RenderNode::new(
                rect_id,
                RenderNodeType::Rectangle(RectangleNode::new(
                    0.0,
                    ShapeStyle {
                        fill_color,
                        pattern,
                        stroke_color: None,
                        stroke_width: 0.0,
                        ..Default::default()
                    },
                    gradient,
                )),
                BoundingBox::new(cell_x, cell_y, cell_w, cell_h),
            );
            cell_node.children.push(rect_node);
        }
        // [Task #429] image fill 처리 — zone 처리와 동일 패턴
        if let Some(img_fill) = border_style.and_then(|bs| bs.image_fill.as_ref()) {
            if let Some(img_bytes) = find_bin_data_bytes(bin_data_content, img_fill.bin_data_id) {
                let img_id = tree.next_id();
                let img_node = RenderNode::new(
                    img_id,
                    RenderNodeType::Image(ImageNode {
                        fill_mode: Some(img_fill.fill_mode),
                        brightness: img_fill.brightness,
                        contrast: img_fill.contrast,
                        effect: img_fill.effect,
                        ..ImageNode::new(img_fill.bin_data_id, Some(img_bytes))
                    }),
                    BoundingBox::new(cell_x, cell_y, cell_w, cell_h),
                );
                cell_node.children.push(img_node);
            }
        }
    }

    /// 표 수평 위치 결정
    pub(crate) fn compute_table_x_position(
        &self,
        table: &crate::model::table::Table,
        table_width: f64,
        col_area: &LayoutRect,
        depth: usize,
        host_alignment: Alignment,
        host_margin_left: f64,
        host_margin_right: f64,
        inline_x_override: Option<f64>,
        paper_width: Option<f64>,
    ) -> f64 {
        if let Some(ix) = inline_x_override {
            // inline_x_override: 外部(テキストフロー)で既に正しい位置が計算済み
            // TAC表のh_offsetはテキストフロー位置には不要 (非TAC表のみ加算)
            if table.common.treat_as_char {
                ix
            } else {
                let h_offset = hwpunit_to_px(table.common.horizontal_offset as i32, self.dpi);
                ix + h_offset
            }
        } else if depth == 0 && table.common.treat_as_char {
            // 글자처럼 취급(treat_as_char)
            // TAC 표의 위치는 텍스트 플로우에 의해 결정되므로 h_offset 미적용
            let ref_x = col_area.x + host_margin_left;
            let ref_w = col_area.width - host_margin_left - host_margin_right;
            match host_alignment {
                Alignment::Center | Alignment::Distribute => {
                    ref_x + (ref_w - table_width).max(0.0) / 2.0
                }
                Alignment::Right => ref_x + (ref_w - table_width).max(0.0),
                _ => ref_x,
            }
        } else if depth == 0 {
            // 표 자체 위치 속성
            let horz_rel_to = table.common.horz_rel_to;
            let horz_align = table.common.horz_align;
            let h_offset = hwpunit_to_px(table.common.horizontal_offset as i32, self.dpi);
            let (ref_x, ref_w) = match horz_rel_to {
                HorzRelTo::Paper => {
                    let paper_w = paper_width.unwrap_or({
                        // fallback: col_area 기반 추정 (paper_width 미전달 시)
                        if table_width > col_area.width {
                            col_area.x * 2.0 + table_width
                        } else {
                            col_area.x * 2.0 + col_area.width
                        }
                    });
                    (0.0, paper_w)
                }
                HorzRelTo::Page => {
                    // Task #347: 본문 영역(body_area) 기준. 미설정 시 col_area 폴백.
                    let body = self.current_body_area.get();
                    if body.2 > 0.0 {
                        (body.0, body.2)
                    } else {
                        (col_area.x, col_area.width)
                    }
                }
                HorzRelTo::Para => (
                    col_area.x + host_margin_left,
                    col_area.width - host_margin_left,
                ),
                _ => (col_area.x, col_area.width),
            };
            match horz_align {
                HorzAlign::Left | HorzAlign::Inside => ref_x + h_offset,
                HorzAlign::Center => ref_x + (ref_w - table_width).max(0.0) / 2.0 + h_offset,
                // Task #347: picture_footnote.rs:185와 동일하게 - h_offset (오른쪽 끝에서 안쪽으로 오프셋).
                HorzAlign::Right | HorzAlign::Outside => {
                    ref_x + (ref_w - table_width).max(0.0) - h_offset
                }
            }
        } else {
            // 중첩 표: outer_margin_left 적용 + host_alignment에 따라 셀 내에서 정렬
            let om_left = hwpunit_to_px(table.outer_margin_left as i32, self.dpi);
            let area_x = col_area.x + om_left;
            let area_w = (col_area.width - om_left).max(0.0);
            // [#3308/#3820] 비-TAC 중첩 표는 저장 폭을 유지하고, 부모 셀보다 좁으면
            // 저장 h_offset(편집기 대화상자 표시값)과 무관하게 셀 안 가운데에 배치한다.
            // 76076 p34의 near-fit 1×1 표도 이 계약을 따른다. 부모 폭으로의 확장은
            // PDF 줄바꿈·조각 높이를 바꾸므로 적용하지 않는다.
            if !table.common.treat_as_char
                && table_width
                    < area_w * crate::renderer::render_normalization::NESTED_STRETCH_MIN_RATIO
            {
                return area_x + (area_w - table_width).max(0.0) / 2.0;
            }
            match host_alignment {
                Alignment::Center | Alignment::Distribute => {
                    area_x + (area_w - table_width).max(0.0) / 2.0
                }
                Alignment::Right => area_x + (area_w - table_width).max(0.0),
                _ => area_x,
            }
        }
    }

    /// 표 세로 위치 결정 (text_wrap + v_offset + 캡션)
    fn compute_table_y_position(
        &self,
        table: &crate::model::table::Table,
        table_height: f64,
        y_start: f64,
        col_area: &LayoutRect,
        depth: usize,
        caption_height: f64,
        caption_spacing: f64,
        para_y: Option<f64>,
        allow_para_top_bleed: bool,
    ) -> f64 {
        let table_treat_as_char = table.common.treat_as_char;
        let table_text_wrap = if depth == 0 {
            table.common.text_wrap
        } else {
            crate::model::shape::TextWrap::Square
        };

        if depth == 0
            && !table_treat_as_char
            && matches!(
                table_text_wrap,
                crate::model::shape::TextWrap::TopAndBottom
                    | crate::model::shape::TextWrap::BehindText
                    | crate::model::shape::TextWrap::InFrontOfText
            )
        {
            // 자리차지(1) / 글뒤로(2) / 글앞으로(3): v_offset 기반 절대 위치

            let v_offset = hwpunit_to_px(table.common.vertical_offset as i32, self.dpi);
            // 문단 기준일 때 para_y 사용 (같은 문단의 여러 표가 동일 기준점 공유)
            let anchor_y = para_y.unwrap_or(y_start);
            // bit 13: VertRelTo가 'para'일 때 본문 영역으로 제한

            let page_h_approx = col_area.y * 2.0 + col_area.height;
            let vert_rel_to = table.common.vert_rel_to;
            // Task #297: Page는 본문 영역(body area) 기준, Paper는 용지 전체 기준
            // (HWP 스펙: Page=쪽 본문, Paper=용지 전체). 바탕쪽 문맥에서는
            // col_area = paper_area이므로 두 경로 결과가 동일하여 회귀 없음.
            let (ref_y, ref_h) = match vert_rel_to {
                crate::model::shape::VertRelTo::Page => {
                    // Task #347: 본문 영역(body_area) 기준. 미설정 시 col_area 폴백.
                    let body = self.current_body_area.get();
                    if body.3 > 0.0 {
                        (body.1, body.3)
                    } else {
                        (col_area.y, col_area.height)
                    }
                }
                crate::model::shape::VertRelTo::Para => {
                    (anchor_y, col_area.height - (anchor_y - col_area.y).max(0.0))
                }
                crate::model::shape::VertRelTo::Paper => (0.0, page_h_approx),
            };
            // Top 캡션: 표 위치를 캡션 높이만큼 아래로 이동
            let caption_top_offset = if let Some(ref cap) = table.caption {
                use crate::model::shape::CaptionDirection;
                if matches!(cap.direction, CaptionDirection::Top) {
                    caption_height
                        + if caption_height > 0.0 {
                            caption_spacing
                        } else {
                            0.0
                        }
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let vert_align = table.common.vert_align;
            // [Task #898] Paper-relative 표는 v_offset 이 외곽 박스 (outer_margin 포함) 기준이므로
            // 가시 표 상단 = v_offset + outer_margin_top. 한컴 PDF (exam_math.hwp 바탕쪽 쪽번호 박스) 정합.
            let om_top_px = if matches!(vert_rel_to, crate::model::shape::VertRelTo::Paper) {
                hwpunit_to_px(table.outer_margin_top as i32, self.dpi)
            } else {
                0.0
            };
            let om_bottom_px = if matches!(vert_rel_to, crate::model::shape::VertRelTo::Paper) {
                hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi)
            } else {
                0.0
            };
            let raw_y = match vert_align {
                crate::model::shape::VertAlign::Top | crate::model::shape::VertAlign::Inside => {
                    ref_y + v_offset + caption_top_offset + om_top_px
                }
                crate::model::shape::VertAlign::Center => {
                    ref_y + (ref_h - table_height) / 2.0 + v_offset + caption_top_offset
                }
                crate::model::shape::VertAlign::Bottom
                | crate::model::shape::VertAlign::Outside => {
                    ref_y + ref_h - table_height - v_offset + caption_top_offset - om_bottom_px
                }
            };
            // Para 기준 + bit 13: 본문 영역으로 제한
            // 앞선 표/텍스트가 차지한 영역(y_start) 아래로 밀어내고, 본문 영역 내로 클램핑
            // Task #347: TopAndBottom 만 y_start 이하로 밀어냄. 글뒤로(BehindText) /
            // 글앞으로(InFrontOfText) 표는 절대 위치 오버레이이므로 push-down 미적용.
            if matches!(vert_rel_to, crate::model::shape::VertRelTo::Para) {
                let body_top = col_area.y;
                let body_bottom = col_area.y + col_area.height - table_height;
                let declared_height = hwpunit_to_px(table.common.height as i32, self.dpi).max(0.0);
                let allow_rowbreak_object_bottom_bleed =
                    matches!(table.page_break, TablePageBreak::RowBreak)
                        && !table.common.treat_as_char
                        && table.row_count == 1
                        && table.col_count == 1
                        && table.cells.len() == 1
                        && signed_hwpunit(table.common.vertical_offset) <= 0
                        && declared_height > 0.0
                        && table_height
                            > declared_height + ROWBREAK_OBJECT_BOTTOM_BLEED_TOLERANCE_PX;
                let pushed =
                    if matches!(table_text_wrap, crate::model::shape::TextWrap::TopAndBottom) {
                        raw_y.max(y_start)
                    } else {
                        raw_y
                    };
                let min_y = if allow_para_top_bleed && v_offset < 0.0 {
                    body_top + v_offset
                } else {
                    body_top
                };
                // [#4514] 문단 기준 다행 RowBreak overlay(글앞/글뒤) 표는 상향 클램프를
                // 걸지 않는다. 앵커가 쪽 하단 부근이면 body_bottom 클램프가 표를 수백
                // px 위로 끌어올려 선행 표 위에 겹쳐 그렸다(8쪽: 880→491.4, 555.5px
                // 겹침 — 판독 불가). 한컴은 이 표를 쪽 경계에서 행 분할한다. 분할
                // 페인트 전 단계로, 앵커 위치를 보존하고 하단 bleed 는 쪽에서 잘리게
                // 둔다(겹침 해소가 우선). 1×1 장식 래퍼는 종전 클램프 유지.
                let overlay_multirow_rowbreak = matches!(
                    table_text_wrap,
                    crate::model::shape::TextWrap::InFrontOfText
                        | crate::model::shape::TextWrap::BehindText
                ) && table.row_count > 1
                    && matches!(table.page_break, TablePageBreak::RowBreak);
                if allow_rowbreak_object_bottom_bleed || overlay_multirow_rowbreak {
                    pushed.max(min_y)
                } else {
                    pushed.clamp(min_y, body_bottom.max(min_y))
                }
            } else {
                raw_y
            }
        } else if depth == 0 {
            let v_offset = if table_treat_as_char {
                hwpunit_to_px(table.common.vertical_offset as i32, self.dpi)
            } else {
                0.0
            };
            if let Some(ref caption) = table.caption {
                use crate::model::shape::CaptionDirection;
                if matches!(caption.direction, CaptionDirection::Top) {
                    y_start + caption_height + caption_spacing + v_offset
                } else {
                    y_start + v_offset
                }
            } else {
                y_start + v_offset
            }
        } else {
            // 중첩 표: outer_margin_top 적용
            let om_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
            y_start + om_top
        }
    }

    /// [Task #2089] 가로쓰기 셀 본문 배치 — 셀 문단/TAC/수식/중첩표 방출.
    /// 원본 무변경 통이동 (탈출은 전부 내부 루프 소속).
    #[allow(clippy::too_many_arguments)]
    fn layout_horizontal_cell_paragraphs(
        &self,
        tree: &mut PageLayoutContext,
        table_node: &mut RenderNode,
        cell_node: &mut RenderNode,
        cell: &crate::model::table::Cell,
        composed_paras: &[ComposedParagraph],
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        bin_data_content: &[BinDataContent],
        table_meta: Option<(usize, usize)>,
        enclosing_cell_ctx: &Option<CellContext>,
        row_filter: Option<(usize, usize)>,
        row_y: &[f64],
        effective_valign: VerticalAlign,
        v: HorizontalCellVars,
    ) {
        let HorizontalCellVars {
            cell_idx,
            r,
            cell_y,
            cell_h,
            content_cell_y,
            pad_top,
            inner_x,
            inner_width,
            inner_height,
            text_y_start,
            use_top_vpos_anchor,
            upper_clip_line_reservation,
            trust_stored_cell_flow,
            has_nested_table,
            section_index,
            outline_numbering_id,
            depth,
            clamp_header_negative_para_offset,
            outer_host_stored_vpos_hu,
            inline_table_flow_y_shift,
            single_row_continuation,
            single_row_continuation_offset,
            single_row_fragment,
            single_row_fragment_content_offset,
            force_source_start_cut,
            replay_terminal_boundary_unit,
            split_terminal,
        } = v;
        let inner_area = LayoutRect {
            x: inner_x,
            y: text_y_start,
            width: inner_width,
            height: inner_height,
        };
        // 1×1 RowBreak 표는 행을 다시 자를 수 없으므로, 부모가 넘긴 픽셀
        // viewport를 이 셀 자신의 유닛 경계로 되돌린다. 여기서 얻은 범위는
        // `layout_partial_table`의 start_cut/end_cut와 같은 의미다. 단순히 현재
        // 셀 하단에서 줄을 버리면 중첩 표/비인라인 컨트롤은 이후 쪽 소유를 잃고
        // SVG clip에만 가려진다 (#3637 HWP 2020 p25–p30).
        // 마지막 조각은 다음 페이지로 넘길 소유자가 없다. 이때 viewport cut을
        // 적용하면 한컴이 같은 쪽에 보존하는 꼬리 문단/중첩 표를 영구 유실한다.
        // 기존 line-fit 경로도 `split_terminal`에서 같은 예외를 두었다 (#3658).
        // Stage 42 diagnostic: make the same source-unit viewport available to
        // native HWP5 RowBreak fragments as to stored HWPX fragments.  The
        // selected overflow fixtures and issue2007 ownership tests determine
        // the final, narrower eligibility predicate.
        let fragment_cut_units = if single_row_fragment
            && row_filter.is_some()
            && (!split_terminal || force_source_start_cut)
        {
            let offset = single_row_fragment_content_offset
                .unwrap_or_else(|| single_row_continuation_offset.unwrap_or(0.0))
                .max(0.0);
            // 앞 조각의 콘텐츠는 `content_cell_y`가 이미 음수 방향으로 옮겨 놓고
            // 물리 Cell clip이 위쪽을 제거한다. 여기서 start까지 다시 버리면 현
            // 페이지 상단에 이어져야 할 줄이 사라진다. 따라서 현재 페이지 **하단**
            // 까지만 정확히 자르고, 앞부분은 같은 논리 원점에서 배치시킨다.
            let start = if force_source_start_cut {
                let units = self.cell_units_fitting_height(cell, table, styles, offset);
                if replay_terminal_boundary_unit {
                    // Native HWP5 short-parent child fragments can end the preceding
                    // viewport inside the final source unit.  That unit is physically
                    // clipped on the preceding page, so treating it as fully consumed
                    // loses its first visible line on the terminal continuation
                    // (76076 p81 -> p82).  Keep the outer fragment geometry intact and
                    // replay exactly that boundary unit on the next page.
                    units.saturating_sub(1)
                } else {
                    // terminal_rowbreak_source_cursor-only (76076 p33 -> p34) already
                    // owns every unit through `units`; replaying one back here would
                    // repaint its last source line again.
                    units
                }
            } else {
                0
            };
            let end = self
                .cell_units_fitting_height(cell, table, styles, offset + cell_h.max(0.0))
                .max(start);
            Some((start, end))
        } else {
            None
        };
        let fragment_line_ranges = fragment_cut_units
            .map(|(start, end)| self.cell_line_ranges_from_cut(cell, table, styles, start, end));
        // 셀 내 문단 + 컨트롤 통합 레이아웃
        let mut para_y = text_y_start;
        let mut has_preceding_text = false;
        for (cp_idx, (composed, para)) in composed_paras
            .iter()
            .zip(cell.paragraphs.iter())
            .enumerate()
        {
            let (start_line, end_line) = fragment_line_ranges
                .as_ref()
                .and_then(|ranges| ranges.get(cp_idx).copied())
                .unwrap_or((0, composed.lines.len()));
            let mixed_nested_split = fragment_cut_units.and_then(|(start, end)| {
                self.mixed_nested_split_from_cut(cell, table, styles, start, end, cp_idx)
            });
            let visible_non_inline_controls = fragment_cut_units.is_some_and(|(start, end)| {
                self.cell_cut_contains_non_inline_control_units(
                    cell, table, styles, start, end, cp_idx,
                )
            });
            // 빈 host 문단은 블록 중첩 표만 담을 수 있다. 그러므로 이 조각이
            // 실제 unit cut을 가진 경우에만 빈 범위를 건너뛴다. 일반 표까지
            // 건너뛰면 `근거설명`처럼 text_len=0인 host의 Table control 자체가
            // 방출되지 않는다 (76076 regulatory analysis p34).
            if fragment_cut_units.is_some()
                && start_line >= end_line
                && mixed_nested_split.is_none()
                && !visible_non_inline_controls
            {
                continue;
            }
            let cell_context = if let Some(ref ctx) = enclosing_cell_ctx {
                let mut new_ctx = ctx.clone();
                if let Some(last) = new_ctx.path.last_mut() {
                    last.cell_index = cell_idx;
                    last.cell_para_index = cp_idx;
                    last.text_direction = cell.text_direction;
                }
                Some(new_ctx)
            } else {
                table_meta.map(|(pi, ci)| CellContext {
                    parent_para_index: pi,
                    path: vec![CellPathEntry {
                        control_index: ci,
                        cell_index: cell_idx,
                        cell_para_index: cp_idx,
                        text_direction: cell.text_direction,
                    }],
                })
            };

            let has_table_ctrl = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
            // [Task #573] inline TAC 표(treat_as_char=true) 와 block 표(treat_as_char=false)
            // 를 분리. 인라인 TAC 표가 있는 셀 paragraph 의 surrounding text (예: "ㄷ. ",
            // "이다.") 가 layout_composed_paragraph 호출 미진입으로 미렌더되던 결함 정정.
            // block 표는 별도 layout_table 호출로 배치되므로 텍스트 흐름 외부 — 기존
            // ELSE 분기 로직 유지. inline TAC 표는 layout_composed_paragraph 의 run_tacs
            // 에서 텍스트와 함께 배치되어야 함.
            let has_block_table_ctrl = para
                .controls
                .iter()
                .any(|c| matches!(c, Control::Table(t) if !t.common.treat_as_char));

            // HWP/HWPX가 셀 내부 문단의 LINE_SEG.vpos를 제공하는 경우에는
            // 누적 y 대신 그 절대 위치를 우선한다. 조직도형 표처럼 셀 하나에
            // 여러 짧은 문단이 있고 paraPr spacing/lineSpacing이 함께 지정된
            // 문서는 한컴이 각 문단 top을 vpos로 고정해 둔다. 누적 y만 쓰면
            // spacing_before가 중복되거나 음수 line_spacing이 누적되어 줄 위치가
            // 점점 어긋난다.
            //
            // 단, vpos == 0 은 "앵커 없음"의 센티널이기도 하다. 셀 안 문단이 전부
            // vpos == 0 으로 저장된 문서(중첩 표 안쪽 셀에서 흔하다)에서 이를 절대
            // 위치로 받아들이면 모든 문단이 셀 상단이라는 같은 y 로 리셋되어 서로
            // 겹쳐 그려진다. 첫 문단의 vpos == 0 은 "셀 상단"이라는 유효한 값이므로
            // 그대로 두고, 두 번째 이후 문단은 양수 vpos 가 저장돼 있을 때만 앵커로
            // 쓴다. (같은 파일의 text_y_start 계산도 `v > 0.0` 을 앵커 조건으로 쓴다)
            // 셀 안의 문단 또는 문단 내부 줄이 중간에 `vpos=0`으로 다시 시작한 뒤,
            // 그 다음 양수 vpos를 cell top 기준 절대 좌표로 해석하면 앞 문단 위로
            // 되감겨 겹친다. 이 reset은 RowBreak continuation에만 한정되지 않는다.
            // 예컨대 42065 p2의 일반 9×2 표 우측 셀과 p10--p16의 손자 1×1 셀은
            // 모두 같은 저장 형식이다. reset 뒤에는 저장 anchor 대신 누적 flow를
            // 쓴다. 첫 문단 첫 줄의 0은 정상적인 cell-top anchor이므로 제외한다.
            let local_vpos_restart_seen = cell
                .paragraphs
                .iter()
                .take(cp_idx.saturating_add(1))
                .enumerate()
                .any(|(prior_para_idx, prior)| {
                    prior.line_segs.iter().enumerate().any(|(line_idx, seg)| {
                        seg.vertical_pos == 0 && (prior_para_idx > 0 || line_idx > 0)
                    })
                });
            let has_stored_para_anchor =
                !local_vpos_restart_seen && crate::renderer::first_seg_vpos_is_anchor(para, cp_idx);
            let use_saved_cell_para_vpos = use_top_vpos_anchor
                || trust_stored_cell_flow
                || has_initial_tac_shape_host(&cell.paragraphs);
            if use_saved_cell_para_vpos
                && (!has_nested_table || trust_stored_cell_flow)
                && has_stored_para_anchor
            {
                if let Some(first_seg) = para.line_segs.first() {
                    if first_seg.vertical_pos >= 0 {
                        let spacing_before = styles
                            .para_styles
                            .get(para.para_shape_id as usize)
                            .map(|s| s.spacing_before)
                            .unwrap_or(0.0);
                        let anchored_y = cell_para_line_anchor_y(
                            text_y_start,
                            content_cell_y,
                            pad_top,
                            first_seg.vertical_pos,
                            self.dpi,
                            use_top_vpos_anchor,
                            upper_clip_line_reservation,
                        );
                        // layout_composed_paragraph()가 spacing_before를 더하므로
                        // 호출 전에 그 값을 빼서 최종 line top이 vpos와 일치하게 한다.
                        para_y = anchored_y - spacing_before;
                    }
                }
            }

            let para_y_before_compose = para_y;

            // 줄별 TAC 컨트롤 너비 합산: 각 TAC가 속한 줄을 판별하여 줄별 최대 너비 계산
            let tac_line_widths: Vec<f64> = {
                // 줄별 너비 합산 벡터
                let mut line_widths = vec![0.0f64; composed.lines.len().max(1)];
                for ctrl in &para.controls {
                    let (is_tac, w) = match ctrl {
                        Control::Picture(pic) if pic.common.treat_as_char => {
                            (true, hwpunit_to_px(pic.common.width as i32, self.dpi))
                        }
                        Control::Shape(shape) if shape.common().treat_as_char => {
                            (true, hwpunit_to_px(shape.common().width as i32, self.dpi))
                        }
                        Control::Equation(eq) => {
                            (true, hwpunit_to_px(eq.common.width as i32, self.dpi))
                        }
                        Control::Table(t) if t.common.treat_as_char => {
                            // [Issue #3396] 한글은 TAC 표의 문자 폭에 outMargin
                            // 좌/우를 포함한다 (정렬·전진 폭 공히).
                            (
                                true,
                                hwpunit_to_px(
                                    t.common.width as i32
                                        + t.outer_margin_left as i32
                                        + t.outer_margin_right as i32,
                                    self.dpi,
                                ),
                            )
                        }
                        _ => (false, 0.0),
                    };
                    if !is_tac {
                        continue;
                    }
                    // 줄이 1개이면 무조건 0번 줄
                    if composed.lines.len() <= 1 {
                        line_widths[0] += w;
                    } else {
                        // 아직 줄 분배 전이므로 순서대로 채워넣기:
                        // 현재 줄 너비 + 이 컨트롤 너비 > 셀 너비이면 다음 줄로
                        let mut placed = false;
                        for lw in line_widths.iter_mut() {
                            if *lw == 0.0 || *lw + w <= inner_width + 0.5 {
                                *lw += w;
                                placed = true;
                                break;
                            }
                        }
                        if !placed {
                            if let Some(last) = line_widths.last_mut() {
                                *last += w;
                            }
                        }
                    }
                }
                line_widths
            };
            let total_inline_width: f64 = tac_line_widths.iter().cloned().fold(0.0f64, f64::max);

            if !has_block_table_ctrl {
                let is_last_para = cp_idx + 1 == composed_paras.len();
                let numbered_comp = if start_line == 0 && end_line > start_line {
                    self.apply_paragraph_numbering(
                        Some(composed),
                        para,
                        styles,
                        outline_numbering_id,
                    )
                } else {
                    None
                };
                let composed_for_layout = numbered_comp.as_ref().unwrap_or(composed);
                para_y = self.layout_composed_paragraph(
                    tree,
                    cell_node,
                    composed_for_layout,
                    styles,
                    &inner_area,
                    para_y,
                    start_line,
                    end_line,
                    section_index,
                    cp_idx,
                    cell_context.clone(),
                    !use_top_vpos_anchor,
                    is_last_para,
                    0.0,
                    None,
                    Some(para),
                    Some(bin_data_content),
                    None, // 셀 컨텍스트 — wrap zone 무관
                );

                let has_visible_text = composed
                    .lines
                    .iter()
                    .any(|line| line.runs.iter().any(|run| !run.text.trim().is_empty()));
                if has_visible_text {
                    has_preceding_text = true;
                }
            } else {
                // has_table_ctrl: 표가 포함된 문단
                // LINE_SEG vpos가 문단 위치를 정확히 지정하므로,
                // 추가 spacing 없이 para_y를 그대로 사용.
                // (leading spacing은 LINE_SEG vpos에 이미 반영되어 있음)
            }

            let para_alignment = styles
                .para_styles
                .get(para.para_shape_id as usize)
                .map(|s| s.alignment)
                .unwrap_or(Alignment::Left);
            // [Task #548] paragraph margin_left + first-line indent 를 inline shape
            // 위치에 반영. paragraph_layout 텍스트 경로와 동일한 effective_margin_left
            // 산식을 적용해 텍스트와 shape 위치 일관성 보장.
            let para_margin_left_px = styles
                .para_styles
                .get(para.para_shape_id as usize)
                .map(|s| s.margin_left)
                .unwrap_or(0.0);
            let para_indent_px = styles
                .para_styles
                .get(para.para_shape_id as usize)
                .map(|s| s.indent)
                .unwrap_or(0.0);

            let mut prev_tac_text_pos: usize = 0;
            // LINE_SEG 기반 줄별 TAC 이미지 배치를 위한 상태
            // 빈 문단(runs 없음)에서 TAC 컨트롤을 LINE_SEG에 순서대로 매핑
            let all_runs_empty = composed.lines.iter().all(|l| l.runs.is_empty());
            let mut tac_seq_index: usize = 0; // TAC 컨트롤 순번 (빈 문단용)
            let mut current_tac_line: usize = 0;
            let mut inline_x = {
                let line_w = tac_line_widths
                    .first()
                    .copied()
                    .unwrap_or(total_inline_width);
                let line_margin =
                    effective_margin_left_line(para_margin_left_px, para_indent_px, 0);
                match para_alignment {
                    Alignment::Center | Alignment::Distribute => {
                        inner_area.x + (inner_area.width - line_w).max(0.0) / 2.0
                    }
                    Alignment::Right => inner_area.x + (inner_area.width - line_w).max(0.0),
                    _ => inner_area.x + line_margin,
                }
            };
            let mut tac_img_y = para_y_before_compose;
            let mut rendered_top_and_bottom_non_inline = false;

            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                match ctrl {
                    Control::Picture(pic) => {
                        let visible_non_inline_control =
                            fragment_cut_units.map_or(true, |(su, eu)| {
                                self.cell_cut_starts_non_inline_control(
                                    cell, table, styles, su, eu, cp_idx, ctrl_idx,
                                )
                            });
                        let fragment_owned_square_flow = self.profile.get().native_hwp5_layout()
                            && fragment_cut_units.is_some()
                            && visible_non_inline_control
                            && pic.common.flow_with_text
                            && matches!(pic.common.text_wrap, TextWrap::Square);
                        if !pic.common.treat_as_char
                            && fragment_cut_units.is_some()
                            && !visible_non_inline_control
                        {
                            continue;
                        }
                        if pic.common.treat_as_char {
                            let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                            // [Task #928] paragraph_layout 이 inline picture 를 emit 한
                            // 경우 set_inline_shape_position 을 호출하므로 (paragraph_layout.rs
                            // 라인 2019-2022), 본 가드는 inline_shape_position 등록 여부로
                            // 판정한다. 기존 tac_controls + line_chars 기반 가드는 boundary
                            // 케이스 (abs_pos == line_chars) 를 빠뜨려 exam_kor 5p ㉢
                            // 그림 중복 emit 회귀가 있었다.
                            let will_render_inline = tree
                                .get_inline_shape_position(
                                    section_index,
                                    cp_idx,
                                    ctrl_idx,
                                    cell_context.as_ref(),
                                )
                                .is_some();
                            if !will_render_inline {
                                // LINE_SEG 기반 줄 판별
                                let target_line = if all_runs_empty && para.line_segs.len() > 1 {
                                    // 빈 문단: TAC 순번으로 LINE_SEG에 1:1 매핑
                                    let li = tac_seq_index.min(para.line_segs.len() - 1);
                                    tac_seq_index += 1;
                                    li
                                } else {
                                    // 텍스트 있는 문단: char position으로 줄 판별
                                    composed
                                        .tac_controls
                                        .iter()
                                        .find(|&&(_, _, ci)| ci == ctrl_idx)
                                        .map(|&(abs_pos, _, _)| {
                                            composed
                                                .lines
                                                .iter()
                                                .enumerate()
                                                .rev()
                                                .find(|(_, line)| abs_pos >= line.char_start)
                                                .map(|(li, _)| li)
                                                .unwrap_or(0)
                                        })
                                        .unwrap_or(0)
                                };

                                if target_line > current_tac_line {
                                    // 줄이 바뀜: inline_x 리셋, y를 LINE_SEG vpos 기준으로 이동
                                    current_tac_line = target_line;
                                    let line_w =
                                        tac_line_widths.get(target_line).copied().unwrap_or(0.0);
                                    // [Task #548] target_line 의 effective_margin_left 적용
                                    let line_margin = effective_margin_left_line(
                                        para_margin_left_px,
                                        para_indent_px,
                                        target_line,
                                    );
                                    inline_x = match para_alignment {
                                        Alignment::Center | Alignment::Distribute => {
                                            inner_area.x
                                                + (inner_area.width - line_w).max(0.0) / 2.0
                                        }
                                        Alignment::Right => {
                                            inner_area.x + (inner_area.width - line_w).max(0.0)
                                        }
                                        _ => inner_area.x + line_margin,
                                    };
                                    if let Some(seg) = para.line_segs.get(target_line) {
                                        // [Task #520 / #624 복원] LineSeg.vertical_pos 는 셀 origin 기준 절대값.
                                        // para_y_before_compose 에 이미 ls[0].vpos 가 누적되어 있어
                                        // 상대 오프셋(seg.vpos - ls[0].vpos)만 더해야 이중 합산을 피한다.
                                        let first_vpos = para
                                            .line_segs
                                            .first()
                                            .map(|f| f.vertical_pos)
                                            .unwrap_or(0);
                                        tac_img_y = para_y_before_compose
                                            + hwpunit_to_px(
                                                seg.vertical_pos - first_vpos,
                                                self.dpi,
                                            );
                                    }
                                }

                                let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                // [Task #477] 셀 폭 초과 시 비율 유지 클램프
                                let clamped_w = pic_w.min(inner_area.width);
                                let clamped_h = if pic_w > 0.0 {
                                    pic_h * (clamped_w / pic_w)
                                } else {
                                    pic_h
                                };
                                let pic_area = LayoutRect {
                                    x: inline_x,
                                    y: tac_img_y,
                                    width: clamped_w,
                                    height: clamped_h,
                                };
                                // [Task #1151 v4] 셀 안 inline picture (tac=true):
                                // outer paragraph idx + inner picture ctrl idx +
                                // cell_ctx 전달 → ImageNode cell_index + cursor_rect
                                // hit-test 정합.
                                self.layout_picture(
                                    tree,
                                    cell_node,
                                    pic,
                                    &pic_area,
                                    bin_data_content,
                                    Alignment::Left,
                                    Some(section_index),
                                    cell_context.as_ref().map(|c| c.parent_para_index),
                                    Some(ctrl_idx),
                                    cell_context.as_ref(),
                                );
                                inline_x += clamped_w;
                                continue;
                            }
                            inline_x += pic_w;
                        } else {
                            // 비-인라인(자리차지/글뒤로/글앞으로) 이미지:
                            // 본문배치 속성(가로/세로 기준, 정렬, 오프셋) 적용
                            let pic_w = hwpunit_to_px(pic.common.width as i32, self.dpi);
                            let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                            // vert_rel_to=Para 인 셀 내부 비인라인 이미지의 앵커 기준점.
                            // `para_y` 는 `layout_composed_paragraph` 가 advance 시킨 뒤의
                            // 값이라 한 줄 아래를 가리킨다 — 그대로 쓰면 그림이 줄 높이만큼
                            // 내려가 셀 경계에 잘린다.
                            //
                            // 이 자리는 wrap 종류를 하나씩 열거하며 고쳐 왔다 —
                            // [Task #577] TopAndBottom(exam_science 2번 보기 ⑤ 등 5개가
                            // line_height 약 15.32px 만큼 밀려 잘림), [Task #2207] 글뒤로·
                            // 글앞으로(오버레이는 텍스트 플로우를 밀지 않아 같은 원리).
                            // [#4059] 그 열거에 Square·Tight·Through 가 빠져 있었다 — 관세청
                            // 보도자료 1쪽 "한국판뉴딜" 로고가 줄 높이(17.3px)만큼 밀려 잘렸다.
                            //
                            // 다만 **두 무리는 기준점이 다르다.** wrap 무관하게 #577 공식
                            // (`content_cell_y + pad_top + seg.vpos`)으로 통일해 보았더니
                            // `pic-in-table-with-toggle` 이 한글 대비 +8.6px 에서 −43.8px 로
                            // 더 어긋났다. 그 셀은 valign=Center 라 문단이 셀 상단이 아니라
                            // 가운데에 놓이는데, 저 공식은 셀 콘텐츠 상단을 가리키기 때문이다.
                            // Square 계열은 **실제 문단 top**(`para_y_before_compose`)이 맞다.
                            //
                            // 한글 PDF 오라클 실측 (그림 top, px):
                            //   문서                        한글     종전      정정 후
                            //   관세청 한국판뉴딜           191.9   208.3    191.0
                            //   pic-in-table-with-toggle    249.5   258.1    244.8
                            //   hwpx_sample2 p19            970.2   978.4    965.1
                            // 잔여 약 5px 는 별개 축이다 — toggle 은 x 도 같은 크기로 어긋난다
                            // (한글 170.1 vs 166.4). 앵커 원점(셀 padding 해석) 쪽으로 보인다.
                            //
                            // 이 분기는 이미 `treat_as_char == false` 안이므로 wrap 조건 없이
                            // vert_rel_to 만 본다.
                            let non_inline_para = matches!(
                                pic.common.vert_rel_to,
                                crate::model::shape::VertRelTo::Para
                            );
                            // #2071 셀 valign 강제 + 앵커 분기 판정용. 그쪽은 한글 2024
                            // 오라클로 TopAndBottom 한정 검증된 **별개 계약**이라 위 앵커
                            // 정정과 함께 넓히지 않는다.
                            let top_and_bottom_para = non_inline_para
                                && matches!(
                                    pic.common.text_wrap,
                                    crate::model::shape::TextWrap::TopAndBottom
                                );
                            let overlay_para = non_inline_para
                                && matches!(
                                    pic.common.text_wrap,
                                    crate::model::shape::TextWrap::BehindText
                                        | crate::model::shape::TextWrap::InFrontOfText
                                );
                            // [Task #2226] 텍스트 없는 문단에서 seg.vpos > 0 이면 그
                            // 줄은 flow 그림에 밀려난 위치다 — 그림 오프셋의 원점은
                            // 문단 시작이므로 앵커에 vpos 를 더하면 그림이 셀 아래로
                            // 이탈한다 (주보 p2 로고 표 붓글씨 셀: line vpos 51.3px).
                            let displaced_empty_line_para = para.text.trim().is_empty()
                                && para
                                    .line_segs
                                    .first()
                                    .is_some_and(|seg| seg.vertical_pos > 0);
                            let anchor_y = if displaced_empty_line_para {
                                // Square 포함 모든 비인라인 그림 — 원점은 문단 시작.
                                content_cell_y + pad_top
                            } else if non_inline_para && !top_and_bottom_para && !overlay_para {
                                // Square·Tight·Through — 흐름을 미는 wrap. 기준점은 셀
                                // 콘텐츠 상단이 아니라 **실제 문단 top** 이다(valign 반영).
                                para_y_before_compose
                            } else if non_inline_para {
                                para.line_segs
                                    .first()
                                    .filter(|seg| seg.vertical_pos >= 0)
                                    .map(|seg| {
                                        content_cell_y
                                            + pad_top
                                            + hwpunit_to_px(seg.vertical_pos, self.dpi)
                                    })
                                    .unwrap_or(para_y_before_compose)
                            } else {
                                para_y
                            };
                            let unrestricted_take_place_cell_float = !pic.common.flow_with_text
                                && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
                                && matches!(pic.common.vert_rel_to, VertRelTo::Para);
                            let reset_relocated_stored_picture_offset =
                                stored_layout_relocated_empty_rowbreak_picture_resets_offset(
                                    self.profile.get().native_hwp5_layout()
                                        || self.profile.get().hwpx_stored_layout(),
                                    self.profile.get().native_hwp5_layout(),
                                    outer_host_stored_vpos_hu,
                                    table,
                                    cell,
                                    para,
                                    pic,
                                );
                            let detached_from_inline_table_flow = inline_table_flow_y_shift > 0.0
                                && unrestricted_take_place_cell_float;
                            let picture_anchor_y = if detached_from_inline_table_flow {
                                anchor_y - inline_table_flow_y_shift - row_y[r].max(0.0)
                            } else if unrestricted_take_place_cell_float {
                                // 한컴의 셀 내부 자리차지 그림은 제한이 꺼지면
                                // offset 지점에 그림 하단이 걸리도록 위로 빠진다.
                                // compute_object_position 이 아래에서 vOffset 을
                                // 다시 더하므로 여기서는 미리 vOffset+높이를 뺀다.
                                anchor_y
                                    - pic_h
                                    - hwpunit_to_px(pic.common.vertical_offset as i32, self.dpi)
                            } else {
                                anchor_y
                            };
                            let cell_area = LayoutRect {
                                y: picture_anchor_y,
                                height: (inner_area.height - (picture_anchor_y - inner_area.y))
                                    .max(0.0),
                                ..inner_area
                            };
                            let (pic_x, pic_y) = self.compute_object_position(
                                &pic.common,
                                pic_w,
                                pic_h,
                                &cell_area,
                                &inner_area,
                                &inner_area,
                                &inner_area,
                                picture_anchor_y,
                                para_alignment,
                            );
                            // [Issue #2071] 셀 앵커 floating 그림(restrict-ON,
                            // TopAndBottom+Para)은 한컴이 **셀 vertical_align 으로만**
                            // 배치하고 그림 자체 pos vert_align 은 무시한다. 위
                            // compute_object_position 은 그림 pos vert_align 을 따르므로
                            // pic≠Top 이거나 셀 valign≠Top 이면 어긋난다.
                            // 한글 2024 편집기 오라클(ta-pic pos/cell vertAlign 변형 실측):
                            //   셀=Center × pic=Top/Center/Bottom → 모두 362.5(셀 중앙)
                            //   셀=Top × pic=Center → 153.8(셀 상단)  [pic 무시 확인]
                            // 콘텐츠 box·그림 높이 기준으로 셀 valign 위치를 강제:
                            //   TOP    = content_top + vOffset
                            //   CENTER = content_top + (content_h − pic_h + vOffset)/2
                            //   BOTTOM = content_bottom − pic_h − vOffset
                            let pic_y = if fragment_owned_square_flow {
                                // partial-table와 같은 source-owner 계약: 현재 cut이
                                // 소유한 Square flow picture는 page-local flow anchor를
                                // 쓴다. 이전 source ladder의 negative vOffset을 다시
                                // 적용하면 같은 paragraph의 후속 control이 fragment 위로
                                // 빠진다.
                                picture_anchor_y
                            } else if reset_relocated_stored_picture_offset {
                                // 이 형상은 cell의 Center 값이 현 물리 페이지의 정렬 계약이
                                // 아니라 stale 음수 offset과 짝을 이룬 이전 페이지 ladder다.
                                // page-local content top이 한컴 PDF의 그림 상단이다.
                                content_cell_y + pad_top
                            } else if top_and_bottom_para
                                && pic.common.flow_with_text
                                && !unrestricted_take_place_cell_float
                                && !detached_from_inline_table_flow
                            {
                                let v_off = hwpunit_to_px(
                                    signed_hwpunit(pic.common.vertical_offset),
                                    self.dpi,
                                );
                                // [#3738 Stage 8] Bottom caption은 그림과 하나의
                                // 시각 블록이다. 이를 빼고 Center/Bottom을 계산하면
                                // 그림 본체만 셀 중앙에 놓이고, caption이 셀 밖으로
                                // 넘쳐 뒤 본문과 겹친다. Top caption은 그림 위쪽
                                // 좌표 계약이 달라 이 보정 대상이 아니다.
                                let bottom_caption_h =
                                    pic.caption.as_ref().map_or(0.0, |caption| {
                                        if matches!(
                                            caption.direction,
                                            crate::model::shape::CaptionDirection::Bottom
                                        ) {
                                            self.calculate_caption_height(&pic.caption, styles)
                                                + hwpunit_to_px(caption.spacing as i32, self.dpi)
                                        } else {
                                            0.0
                                        }
                                    });
                                let aligned_visual_h = pic_h + bottom_caption_h;
                                let content_top = content_cell_y + pad_top;
                                match effective_valign {
                                    VerticalAlign::Top => content_top + v_off,
                                    VerticalAlign::Center => {
                                        content_top
                                            + (inner_height - aligned_visual_h + v_off) / 2.0
                                    }
                                    VerticalAlign::Bottom => {
                                        content_top + inner_height - aligned_visual_h - v_off
                                    }
                                }
                            } else {
                                pic_y
                            };
                            let pic_area = LayoutRect {
                                x: pic_x,
                                y: pic_y,
                                width: pic_w,
                                height: pic_h,
                            };
                            let mut pic_for_layout = pic.clone();
                            pic_for_layout.common.horizontal_offset = 0;
                            pic_for_layout.common.vertical_offset = 0;
                            pic_for_layout.common.horz_align = crate::model::shape::HorzAlign::Left;
                            pic_for_layout.common.vert_align = crate::model::shape::VertAlign::Top;
                            // [Task #1151 v4] 셀 안 non-inline picture (tac=false 자리차지 등):
                            // outer paragraph idx + inner picture ctrl idx +
                            // cell_ctx 전달.
                            let picture_parent = if detached_from_inline_table_flow
                                || unrestricted_take_place_cell_float
                            {
                                &mut *table_node
                            } else {
                                &mut *cell_node
                            };
                            self.layout_picture(
                                tree,
                                picture_parent,
                                &pic_for_layout,
                                &pic_area,
                                bin_data_content,
                                Alignment::Left,
                                Some(section_index),
                                cell_context.as_ref().map(|c| c.parent_para_index),
                                Some(ctrl_idx),
                                cell_context.as_ref(),
                            );
                            // 셀 안 부동 그림도 본문/각주 그림과 마찬가지로 자체 caption을
                            // 방출해야 한다. 이 경로가 빠져 있으면 HWP5 LIST_HEADER의
                            // 그림 caption은 파싱돼도 화면에는 사라지고 후속 flow의 기준도
                            // 달라진다. 현재는 image frame 아래에 놓이는 Bottom caption만
                            // 이 경로의 cell-local placement와 같은 좌표계로 렌더한다.
                            if let Some(caption) = pic.caption.as_ref().filter(|caption| {
                                matches!(caption.direction, CaptionDirection::Bottom)
                                    && !caption.paragraphs.is_empty()
                            }) {
                                let caption_spacing =
                                    hwpunit_to_px(caption.spacing as i32, self.dpi);
                                self.layout_caption(
                                    tree,
                                    picture_parent,
                                    caption,
                                    styles,
                                    &inner_area,
                                    pic_x,
                                    pic_w,
                                    pic_y + pic_h + caption_spacing,
                                    &mut self.auto_counter.borrow_mut(),
                                    bin_data_content,
                                    cell_context.clone(),
                                );
                            }
                            if matches!(pic.common.text_wrap, TextWrap::TopAndBottom) {
                                rendered_top_and_bottom_non_inline = true;
                            } else if fragment_owned_square_flow {
                                para_y += self.cell_non_inline_control_flow_height(&pic.common);
                            } else {
                                para_y += self.non_inline_control_flow_height(&pic.common);
                            }
                        }
                        has_preceding_text = true;
                    }
                    Control::Shape(shape) => {
                        // Shape/TextBox는 control entry 뒤의 physical fragment에서도
                        // 잔여 내부 문단을 계속 렌더한다. entry-only 판정은 원자적으로
                        // 소유하는 Picture에만 적용한다.
                        if !shape.common().treat_as_char
                            && fragment_cut_units.is_some()
                            && !visible_non_inline_controls
                        {
                            continue;
                        }
                        if shape.common().treat_as_char {
                            let shape_w = hwpunit_to_px(shape.common().width as i32, self.dpi);
                            // [Task #928] paragraph_layout 의 run_tacs 처리 (라인 2026-2034)
                            // 가 inline Shape 위치를 set_inline_shape_position 으로 등록
                            // 하므로, 본 가드는 등록 여부로 판정한다. Picture 분기와 동일
                            // 패턴이며 boundary 케이스에 안전.
                            let will_render_inline = tree
                                .get_inline_shape_position(
                                    section_index,
                                    cp_idx,
                                    ctrl_idx,
                                    cell_context.as_ref(),
                                )
                                .is_some();
                            // [Task #500] Picture 분기와 정합: target_line 산출 + 줄 변경 시
                            // inline_x/tac_img_y 리셋. multi-line paragraph 에서 사각형이
                            // ls[1]+ 에 있을 때 paragraph 첫 줄 좌표가 잘못 사용되던 결함 정정.
                            let target_line = if all_runs_empty && para.line_segs.len() > 1 {
                                let li = tac_seq_index.min(para.line_segs.len() - 1);
                                tac_seq_index += 1;
                                li
                            } else {
                                composed
                                    .tac_controls
                                    .iter()
                                    .find(|&&(_, _, ci)| ci == ctrl_idx)
                                    .map(|&(abs_pos, _, _)| {
                                        composed
                                            .lines
                                            .iter()
                                            .enumerate()
                                            .rev()
                                            .find(|(_, line)| abs_pos >= line.char_start)
                                            .map(|(li, _)| li)
                                            .unwrap_or(0)
                                    })
                                    .unwrap_or(0)
                            };
                            if target_line > current_tac_line {
                                current_tac_line = target_line;
                                let line_w =
                                    tac_line_widths.get(target_line).copied().unwrap_or(0.0);
                                // [Task #548] target_line 의 effective_margin_left 적용
                                let line_margin = effective_margin_left_line(
                                    para_margin_left_px,
                                    para_indent_px,
                                    target_line,
                                );
                                inline_x = match para_alignment {
                                    Alignment::Center | Alignment::Distribute => {
                                        inner_area.x + (inner_area.width - line_w).max(0.0) / 2.0
                                    }
                                    Alignment::Right => {
                                        inner_area.x + (inner_area.width - line_w).max(0.0)
                                    }
                                    _ => inner_area.x + line_margin,
                                };
                                if let Some(seg) = para.line_segs.get(target_line) {
                                    // [Task #520] LineSeg.vertical_pos 는 셀 origin 기준 절대값.
                                    // para_y_before_compose 에 이미 ls[0].vpos 가 누적되어 있어
                                    // 상대 오프셋만 더해야 한다 (Picture 분기와 동일).
                                    let first_vpos =
                                        para.line_segs.first().map(|f| f.vertical_pos).unwrap_or(0);
                                    tac_img_y = para_y_before_compose
                                        + hwpunit_to_px(seg.vertical_pos - first_vpos, self.dpi);
                                }
                            }
                            if !will_render_inline {
                                // Shape 앞의 텍스트 너비 계산: tac_controls에서 이 Shape의 text_pos와
                                // 이전 Shape의 text_pos 차이에 해당하는 텍스트 너비를 inline_x에 반영
                                if let Some(&(tac_pos, _, _)) = composed
                                    .tac_controls
                                    .iter()
                                    .find(|&&(_, _, ci)| ci == ctrl_idx)
                                {
                                    // [Task #495] 가드: 사각형이 paragraph 첫 줄(ls[0]) 범위 안에 있을 때만
                                    // text_before 추출/발행. multi-line paragraph 에서 사각형이 ls[1]+ 에
                                    // 있는 경우 composed.lines.first() 만 보던 기존 코드는 첫 줄 전체
                                    // 텍스트를 잘못 추출해 paragraph_layout 결과와 중복 발행했음.
                                    let in_first_line = composed
                                        .lines
                                        .first()
                                        .map(|line| {
                                            let line_chars: usize = line
                                                .runs
                                                .iter()
                                                .map(|r| r.text.chars().count())
                                                .sum();
                                            tac_pos >= line.char_start
                                                && tac_pos < line.char_start + line_chars
                                        })
                                        .unwrap_or(false);
                                    // 이 Shape 앞에 아직 inline_x에 반영되지 않은 텍스트가 있는지 계산
                                    let text_before: String = if in_first_line {
                                        composed
                                            .lines
                                            .first()
                                            .map(|line| {
                                                let mut chars_so_far = 0usize;
                                                let mut result = String::new();
                                                for run in &line.runs {
                                                    for ch in run.text.chars() {
                                                        if chars_so_far >= prev_tac_text_pos
                                                            && chars_so_far < tac_pos
                                                        {
                                                            result.push(ch);
                                                        }
                                                        chars_so_far += 1;
                                                    }
                                                }
                                                result
                                            })
                                            .unwrap_or_default()
                                    } else {
                                        String::new()
                                    };
                                    if !text_before.is_empty() {
                                        let char_style_id = composed
                                            .lines
                                            .first()
                                            .and_then(|l| l.runs.first())
                                            .map(|r| r.char_style_id)
                                            .unwrap_or(0);
                                        let lang_index = composed
                                            .lines
                                            .first()
                                            .and_then(|l| l.runs.first())
                                            .map(|r| r.lang_index)
                                            .unwrap_or(0);
                                        let ts = resolved_to_text_style(
                                            styles,
                                            char_style_id,
                                            lang_index,
                                        );
                                        // [Task #555] PUA 옛한글 char 은 자모 시퀀스로 변환 후 폭 측정.
                                        let text_before_metrics: String = {
                                            use super::super::pua_oldhangul::map_pua_old_hangul;
                                            text_before
                                                .chars()
                                                .flat_map(|ch| {
                                                    if let Some(jamos) = map_pua_old_hangul(ch) {
                                                        jamos.iter().copied().collect::<Vec<_>>()
                                                    } else {
                                                        vec![ch]
                                                    }
                                                })
                                                .collect()
                                        };
                                        let text_w = estimate_text_width(&text_before_metrics, &ts);
                                        let text_font_size = ts.font_size;
                                        // 텍스트 렌더링: Shape 사이에 배치
                                        // 텍스트 y를 Shape 하단 baseline에 맞춤
                                        // (Shape 높이 - 폰트 줄 높이)만큼 아래로 이동
                                        let text_baseline = text_font_size * 0.85;
                                        let font_line_h = text_font_size * 1.2;
                                        // 인접 Shape의 높이를 사용하여 텍스트 y를 baseline 정렬
                                        let adjacent_shape_h = para
                                            .controls
                                            .iter()
                                            .find_map(|c| {
                                                if let Control::Shape(s) = c {
                                                    if s.common().treat_as_char {
                                                        Some(hwpunit_to_px(
                                                            s.common().height as i32,
                                                            self.dpi,
                                                        ))
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                            })
                                            .unwrap_or(0.0);
                                        let text_y = para_y_before_compose
                                            + (adjacent_shape_h - font_line_h).max(0.0);
                                        let text_node_id = tree.next_id();
                                        let text_node = RenderNode::new(
                                            text_node_id,
                                            RenderNodeType::TextRun(TextRunNode {
                                                text: text_before,
                                                style: ts,
                                                char_shape_id: Some(char_style_id),
                                                para_shape_id: Some(composed.para_style_id),
                                                section_index: Some(section_index),
                                                para_index: None,
                                                char_start: None,
                                                cell_context: None,
                                                is_para_end: false,
                                                is_line_break_end: false,
                                                rotation: 0.0,
                                                is_vertical: false,
                                                char_overlap: None,
                                                border_fill_id: 0,
                                                baseline: text_baseline,
                                                field_marker: FieldMarkerType::None,
                                                display_text: None,
                                            }),
                                            BoundingBox::new(inline_x, text_y, text_w, font_line_h),
                                        );
                                        cell_node.children.push(text_node);
                                        inline_x += text_w;
                                    }
                                    prev_tac_text_pos = tac_pos;
                                }
                            }
                            // [Task #520 / #624 복원] target_line 기반 tac_img_y 사용 (Picture 분기와 동일).
                            // para_y_before_compose 사용 시 multi-line paragraph 의 ls[1]+ inline TAC Shape 가
                            // 항상 line 0 좌표에 떨어져 본문 텍스트와 겹친다 (exam_science p2 7번 글상자 ㉠).
                            // [Task #928] will_render_inline=true 인 경우 paragraph_layout 이
                            // 등록한 inline_shape_position 좌표를 사용해 도형 위치를
                            // run_tacs split 에서 reserve 한 gap 과 정확히 정합시킨다.
                            let (shape_x, shape_y) = if will_render_inline {
                                tree.get_inline_shape_position(
                                    section_index,
                                    cp_idx,
                                    ctrl_idx,
                                    cell_context.as_ref(),
                                )
                                .unwrap_or((inline_x, tac_img_y))
                            } else {
                                (inline_x, tac_img_y)
                            };
                            let shape_area = LayoutRect {
                                x: shape_x,
                                y: shape_y,
                                width: shape_w,
                                height: inner_area.height,
                            };
                            // [Task #1138] 셀 컨텍스트 (section, outer_para, outer_table_ctrl, cell, cell_para, inner_ctrl)
                            let table_cell_ctx = table_meta.map(|(opi, otci)| {
                                (section_index, opi, otci, cell_idx, cp_idx, ctrl_idx)
                            });
                            self.layout_cell_shape(
                                tree,
                                cell_node,
                                shape,
                                &shape_area,
                                shape_y,
                                Alignment::Left,
                                styles,
                                bin_data_content,
                                clamp_header_negative_para_offset,
                                table_cell_ctx,
                            );
                            inline_x += shape_w;
                        } else {
                            let shape_anchor_y = if matches!(
                                shape.common().vert_rel_to,
                                crate::model::shape::VertRelTo::Para
                            ) {
                                para_y_before_compose
                            } else {
                                para_y
                            };
                            // [Task #1138] 셀 컨텍스트
                            let table_cell_ctx = table_meta.map(|(opi, otci)| {
                                (section_index, opi, otci, cell_idx, cp_idx, ctrl_idx)
                            });
                            self.layout_cell_shape(
                                tree,
                                cell_node,
                                shape,
                                &inner_area,
                                shape_anchor_y,
                                para_alignment,
                                styles,
                                bin_data_content,
                                clamp_header_negative_para_offset,
                                table_cell_ctx,
                            );
                            if matches!(shape.common().text_wrap, TextWrap::TopAndBottom) {
                                rendered_top_and_bottom_non_inline = true;
                            }
                        }
                    }
                    Control::Equation(eq) => {
                        // 수식 컨트롤: 글자처럼 인라인 배치
                        let eq_w = hwpunit_to_px(eq.common.width as i32, self.dpi);

                        // 수식이 텍스트 run 사이에 인라인으로 배치되는 경우
                        // layout_composed_paragraph에서 이미 렌더링됨 → 건너뛰기
                        let has_text_in_para =
                            para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                        // 빈 runs 셀 + TAC 수식: paragraph_layout(Task #287 경로)이 이미
                        // 렌더 후 set_inline_shape_position 호출. 중복 emit 방지(Issue #301).
                        let already_rendered_inline = tree
                            .get_inline_shape_position(
                                section_index,
                                cp_idx,
                                ctrl_idx,
                                cell_context.as_ref(),
                            )
                            .is_some();
                        if has_text_in_para || already_rendered_inline {
                            // paragraph_layout 경로에서 이미 렌더됨
                            inline_x += eq_w;
                        } else {
                            // 수식만 있는 문단: 여기서 직접 렌더링
                            let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
                            let eq_x = {
                                let x = inline_x;
                                inline_x += eq_w;
                                x
                            };
                            let eq_y = para_y_before_compose;

                            let tokens = super::super::equation::tokenizer::tokenize(&eq.script);
                            let ast = super::super::equation::parser::EqParser::new(tokens).parse();
                            let font_size_px = hwpunit_to_px(eq.font_size as i32, self.dpi);
                            let layout_box =
                                super::super::equation::layout::EqLayout::new(font_size_px)
                                    .layout(&ast);
                            let color_str =
                                super::super::equation::svg_render::eq_color_to_svg(eq.color);
                            let svg_content =
                                super::super::equation::svg_render::render_equation_svg(
                                    &layout_box,
                                    &color_str,
                                    font_size_px,
                                );

                            let eq_node = RenderNode::new(
                                tree.next_id(),
                                RenderNodeType::Equation(EquationNode {
                                    svg_content,
                                    layout_box,
                                    color_str,
                                    color: eq.color,
                                    script: eq.script.clone(),
                                    font_size: font_size_px,
                                    section_index: Some(section_index),
                                    para_index: table_meta.map(|(pi, _)| pi),
                                    control_index: Some(ctrl_idx),
                                    cell_index: Some(cell_idx),
                                    cell_para_index: Some(cp_idx),
                                    note_ref: None,
                                }),
                                BoundingBox::new(eq_x, eq_y, eq_w, eq_h),
                            );
                            cell_node.children.push(eq_node);
                        }
                    }
                    Control::Table(nested_table) => {
                        let is_tac_table = nested_table.common.treat_as_char;
                        // HWPX의 같은 빈 host 문단 안에 있는 `글 뒤로` 1×1 표는
                        // 문단 흐름을 차지하지 않는 overlay control이다. 특히 자동날인
                        // 안내처럼 세 control이 같은 `vpos`에 있고 horzOffset만 다른
                        // 경우, 일반 nested-table 경로처럼 table_h만큼 para_y를 전진하면
                        // PDF의 가로 3개 상자가 세로로 쌓인다 (#3820 p144).
                        //
                        // HWP5의 legacy non-TAC 정렬이나 HWPX TopAndBottom 표까지
                        // horizontal offset을 강제하면 기존 셀 레이아웃을 바꾼다. stored
                        // HWPX의 paragraph-relative BehindText + Column anchor에만
                        // 한정해 parent cell x를 explicit anchor로 넘긴다. 기존
                        // compute_table_x_position은 이 override에 non-TAC horzOffset을
                        // 더하므로 offset의 부호/단위 규칙은 한 곳에 유지된다.
                        let hwpx_nested_behind_text_overlay =
                            self.profile.get().hwpx_stored_layout()
                                && !is_tac_table
                                && matches!(nested_table.common.text_wrap, TextWrap::BehindText)
                                && nested_table.common.flow_with_text
                                && matches!(nested_table.common.vert_rel_to, VertRelTo::Para)
                                && matches!(nested_table.common.horz_rel_to, HorzRelTo::Column);
                        let nested_y = if has_preceding_text {
                            para_y
                        } else {
                            inner_area.y
                        };
                        // [#3637] 중첩 표는 부모 셀 안에서 시작해야 한다. 앞 텍스트가 셀
                        // 밖으로 밀린 `para_y` 를 그대로 쓰면 컨테이너가 통째로 셀 아래에
                        // 놓여 쪽 밖으로 나간다(80550 29쪽: 셀 310~889 인데 중첩2가
                        // 889~1193). PR #3666 이 문단에 건 상한과 같은 계열의 한 단계 깊은
                        // 경로다.
                        // 누적 오프셋을 가진 1×1 RowBreak continuation은 현재 clip보다
                        // 뒤에 있는 다음 내부 표까지 하단으로 clamp하면 안 된다. 원래
                        // 다음 페이지에 속할 표들이 모두 같은 셀 하단으로 재배치되어
                        // 겹치기 때문이다(issue2007 42065 p7–p12). 이때만 원래 y를
                        // 유지해 부모 Cell clip이 미래 표를 제외하고, 앞쪽 표는 음수 y로
                        // 이어서 그릴 수 있다.
                        //
                        // 단순 `row_filter + 1×1`은 continuation의 첫 조각(offset=0)도
                        // 포함한다. 그 형상까지 상한을 풀면 #3637처럼 실제 셀 밖으로
                        // 새는 중첩 표가 다시 허용된다. 따라서 페이지 간에 이미 소비된
                        // 행 높이가 있는 실제 continuation으로 문맥을 좁힌다.
                        // native HWP5 RowBreak 1×1 wrapper는 offset=0인 첫 조각도
                        // 바깥 Cell을 continuation viewport로 사용한다. 이 조각의
                        // 하단으로 미래 descendant를 clamp하면 42065 p17 제목이
                        // p16에 미리 들어온다. HWPX는 실제로 셀 밖으로 빠진 중첩 표만
                        // 막기 위해 아래의 좁은 누적-offset guard를 계속 적용한다.
                        let hwp5_rowbreak_fragment = (self.profile.get().native_hwp5_layout()
                            || self.profile.get().hwp5_origin_hwpx())
                            && row_filter.is_some()
                            && table.row_count == 1
                            && table.col_count == 1;
                        let nested_y = if single_row_continuation || hwp5_rowbreak_fragment {
                            nested_y
                        } else {
                            nested_y.min(inner_area.y + inner_area.height)
                        };
                        let nested_ctx = cell_context.as_ref().map(|ctx| {
                            let mut new_ctx = ctx.clone();
                            new_ctx.path.push(CellPathEntry {
                                control_index: ctrl_idx,
                                cell_index: 0,
                                cell_para_index: 0,
                                text_direction: 0,
                            });
                            new_ctx
                        });
                        // [#4334] 아래 재귀 `layout_table` 호출 두 곳이 `table_meta: None`
                        // 을 넘겨 TableNode.para_index/control_index 가 항상 비었다 —
                        // 방금 확장한 `nested_ctx` 에서 이 중첩 표 자신의 좌표를 읽는다.
                        let derived_table_meta =
                            nested_ctx.as_ref().and_then(CellContext::nested_table_meta);
                        if is_tac_table {
                            // TAC 표: inline_x를 사용하여 수평 배치
                            // [Task #573] layout_composed_paragraph 의 run_tacs 가
                            // 인라인 TAC 표를 이미 렌더하고 set_inline_shape_position
                            // 등록했다면 중복 emit 방지 (Equation 의 L1800 가드와 동일 패턴).
                            let already_rendered_inline = tree
                                .get_inline_shape_position(
                                    section_index,
                                    cp_idx,
                                    ctrl_idx,
                                    cell_context.as_ref(),
                                )
                                .is_some();
                            let tac_w = hwpunit_to_px(nested_table.common.width as i32, self.dpi);
                            // [Issue #3396] 한글 TAC 표 문자 규칙: 괘선은
                            // pen + outMargin.left, 전진 폭은 outMargin 좌/우 포함.
                            let tac_om_l =
                                hwpunit_to_px(nested_table.outer_margin_left as i32, self.dpi);
                            let tac_om_r =
                                hwpunit_to_px(nested_table.outer_margin_right as i32, self.dpi);
                            if already_rendered_inline {
                                inline_x += tac_om_l + tac_w + tac_om_r;
                            } else {
                                // [Task #1195] 표 앞에 텍스트(공백 등)가 선행하면, 한컴은
                                // 그 textRun 너비 다음에 표를 놓되 잔여 너비가 부족하면
                                // 다음 줄(line feed)에 조판한다. 즉 표는 문단 첫 줄이 아니라
                                // 표가 속한 line_seg(표 앞 빈 줄 다음)에 위치한다.
                                // 이미지 TAC 분기(L2231)와 동일하게 para_y_before_compose 에
                                // (표 line_seg.vpos − 첫 line_seg.vpos) 상대 오프셋을 더한다.
                                // (para_y_before_compose 에 이미 ls[0].vpos 가 누적되어 있음.)
                                let table_anchor_y = if has_preceding_text
                                    && para.line_segs.len() > 1
                                {
                                    let first_vpos =
                                        para.line_segs.first().map(|f| f.vertical_pos).unwrap_or(0);
                                    let tbl_vpos = para
                                        .line_segs
                                        .last()
                                        .map(|s| s.vertical_pos)
                                        .unwrap_or(first_vpos);
                                    para_y_before_compose
                                        + hwpunit_to_px(tbl_vpos - first_vpos, self.dpi)
                                } else {
                                    para_y_before_compose
                                };
                                // [#3386] 표 전용 줄(저장 lh == h + om_top + om_bottom)
                                // 은 표 상단 = 줄 상단 + om_top 이 한글 실좌표다
                                // (156678235 p5: 저장 vpos+om_top == 한글 PDF 상단
                                // 536.7px, 종전 anchor 는 om_top 소실로 3.8px 상향).
                                let host_seg_lh = if has_preceding_text && para.line_segs.len() > 1
                                {
                                    para.line_segs.last().map(|s| s.line_height).unwrap_or(0)
                                } else {
                                    para.line_segs.first().map(|s| s.line_height).unwrap_or(0)
                                };
                                let om_top_hu = i64::from(nested_table.outer_margin_top);
                                let om_bottom_hu = i64::from(nested_table.outer_margin_bottom);
                                let table_anchor_y = if nested_table.common.height < 0x8000_0000
                                    && om_top_hu + om_bottom_hu > 0
                                    && i64::from(host_seg_lh)
                                        >= i64::from(nested_table.common.height)
                                            + om_top_hu
                                            + om_bottom_hu
                                            - 10
                                {
                                    table_anchor_y
                                        + hwpunit_to_px(
                                            nested_table.outer_margin_top as i32,
                                            self.dpi,
                                        )
                                } else {
                                    table_anchor_y
                                };
                                let ctrl_area = LayoutRect {
                                    x: inline_x + tac_om_l,
                                    y: table_anchor_y,
                                    width: tac_w,
                                    height: (inner_area.height - (table_anchor_y - inner_area.y))
                                        .max(0.0),
                                };
                                let table_h = self.layout_table(
                                    tree,
                                    cell_node,
                                    nested_table,
                                    section_index,
                                    styles,
                                    outline_numbering_id,
                                    &ctrl_area,
                                    table_anchor_y,
                                    bin_data_content,
                                    None,
                                    depth + 1,
                                    derived_table_meta,
                                    para_alignment,
                                    nested_ctx,
                                    0.0,
                                    0.0,
                                    Some(inline_x + tac_om_l),
                                    None,
                                    None,
                                    None,
                                    false,
                                    clamp_header_negative_para_offset,
                                    false,
                                );
                                inline_x += tac_om_l + tac_w + tac_om_r;
                                // para_y는 TAC 표 높이만큼 갱신 (같은 문단 내 다음 표도 같은 y)
                                let new_bottom = para_y_before_compose + table_h;
                                if new_bottom > para_y {
                                    para_y = new_bottom;
                                }
                            }
                        } else {
                            // 비-TAC 표: 기존 수직 배치
                            // 앞 텍스트 너비만큼 x 오프셋 적용
                            let tac_text_offset = if nested_table.attr & 0x01 != 0 {
                                let mut text_w = 0.0;
                                for line in &composed.lines {
                                    for run in &line.runs {
                                        if !run.text.is_empty() {
                                            let ts = resolved_to_text_style(
                                                styles,
                                                run.char_style_id,
                                                run.lang_index,
                                            );
                                            // [Task #555] PUA 옛한글 변환 후 자모 시퀀스 폭.
                                            text_w += estimate_text_width(
                                                effective_text_for_metrics(run),
                                                &ts,
                                            );
                                        }
                                    }
                                }
                                text_w
                            } else {
                                0.0
                            };
                            // TAC 표 앞 텍스트 렌더링 (문단부호 등 표시용)
                            if tac_text_offset > 0.0 {
                                let line_h = composed
                                    .lines
                                    .first()
                                    .map(|l| hwpunit_to_px(l.line_height, self.dpi))
                                    .unwrap_or(12.0);
                                let baseline = line_h * 0.85;
                                let line_id = tree.next_id();
                                let mut line_node = RenderNode::new(
                                    line_id,
                                    RenderNodeType::TextLine(TextLineNode::new(line_h, baseline)),
                                    BoundingBox::new(
                                        inner_area.x,
                                        nested_y,
                                        tac_text_offset,
                                        line_h,
                                    ),
                                );
                                let mut run_x = inner_area.x;
                                for line in &composed.lines {
                                    for run in &line.runs {
                                        if run.text.is_empty() {
                                            continue;
                                        }
                                        let ts = resolved_to_text_style(
                                            styles,
                                            run.char_style_id,
                                            run.lang_index,
                                        );
                                        // [Task #555] PUA 옛한글 변환 후 자모 시퀀스 폭.
                                        let run_w = estimate_text_width(
                                            effective_text_for_metrics(run),
                                            &ts,
                                        );
                                        let run_id = tree.next_id();
                                        let run_node = RenderNode::new(
                                            run_id,
                                            RenderNodeType::TextRun(TextRunNode {
                                                text: run.text.clone(),
                                                style: ts,
                                                char_shape_id: Some(run.char_style_id),
                                                para_shape_id: Some(para.para_shape_id),
                                                section_index: Some(section_index),
                                                para_index: None,
                                                char_start: None,
                                                cell_context: cell_context.clone(),
                                                is_para_end: false,
                                                is_line_break_end: false,
                                                rotation: 0.0,
                                                is_vertical: false,
                                                char_overlap: None,
                                                border_fill_id: 0,
                                                baseline,
                                                field_marker: FieldMarkerType::None,
                                                display_text: None,
                                            }),
                                            BoundingBox::new(run_x, nested_y, run_w, line_h),
                                        );
                                        line_node.children.push(run_node);
                                        run_x += run_w;
                                    }
                                }
                                cell_node.children.push(line_node);
                            }
                            let ctrl_area = LayoutRect {
                                x: inner_area.x + tac_text_offset,
                                y: nested_y,
                                width: (inner_area.width - tac_text_offset).max(0.0),
                                height: (inner_area.height - (nested_y - inner_area.y)).max(0.0),
                            };
                            // 이 셀 조각의 unit cut이 만든 중첩 표 slice를 다음 깊이에도
                            // 그대로 넘긴다. 픽셀 높이만으로 다시 행을 추정하면 첫 조각과
                            // continuation이 같은 행을 각각 재렌더해 쪽 소유가 깨진다.
                            // Source-unit viewport ownership is normally encoded in HWPX
                            // wrapper layouts. Native HWP5 RowBreak wrappers normally carry
                            // the equivalent physical cell clip and cumulative vpos;
                            // forwarding a general mixed split there advances the child early
                            // (42065 p16 -> p17).  The narrow short-parent child contract is
                            // different: its child was expanded to CellUnits specifically for
                            // this parent fragment, so omitting the cursor paints the first
                            // source line again on the continuation (76076 p81 -> p82).
                            let native_short_parent_child_split = self
                                .profile
                                .get()
                                .hwp5_stored_pagination_layout()
                                && self.native_short_parent_child_fragment_eligible(
                                    table,
                                    cell,
                                    nested_table,
                                    self.nested_table_mixed_fragment_heights(nested_table, styles)
                                        .iter()
                                        .map(|fragment| fragment.height)
                                        .sum(),
                                );
                            let nested_split = (self.profile.get().hwpx_stored_layout()
                                || native_short_parent_child_split)
                                .then_some(mixed_nested_split.as_ref())
                                .flatten();
                            let table_h = self.layout_table(
                                tree,
                                cell_node,
                                nested_table,
                                section_index,
                                styles,
                                outline_numbering_id,
                                &ctrl_area,
                                nested_y,
                                bin_data_content,
                                None,
                                depth + 1,
                                derived_table_meta,
                                para_alignment,
                                nested_ctx,
                                0.0,
                                0.0,
                                hwpx_nested_behind_text_overlay.then_some(inner_area.x),
                                nested_split,
                                None,
                                None,
                                false,
                                clamp_header_negative_para_offset,
                                false,
                            );
                            if !hwpx_nested_behind_text_overlay {
                                para_y = nested_y
                                    + nested_split
                                        .map(|split| split.flow_height)
                                        .unwrap_or(table_h);
                            }
                        }
                        has_preceding_text = true;
                    }
                    _ => {}
                }
            }
            if rendered_top_and_bottom_non_inline {
                para_y += self.paragraph_top_and_bottom_non_inline_flow_height(&para.controls);
            }

            // 마지막 인라인 Shape 이후의 남은 텍스트 렌더링 (예: "일")
            if prev_tac_text_pos > 0 {
                let total_text_chars = composed
                    .lines
                    .first()
                    .map(|line| {
                        line.runs
                            .iter()
                            .map(|r| r.text.chars().count())
                            .sum::<usize>()
                    })
                    .unwrap_or(0);
                if prev_tac_text_pos < total_text_chars {
                    let remaining_text: String = composed
                        .lines
                        .first()
                        .map(|line| {
                            let mut chars_so_far = 0usize;
                            let mut result = String::new();
                            for run in &line.runs {
                                for ch in run.text.chars() {
                                    if chars_so_far >= prev_tac_text_pos {
                                        result.push(ch);
                                    }
                                    chars_so_far += 1;
                                }
                            }
                            result
                        })
                        .unwrap_or_default();
                    let remaining_trimmed = remaining_text.trim_end();
                    if !remaining_trimmed.is_empty() {
                        let char_style_id = composed
                            .lines
                            .first()
                            .and_then(|l| l.runs.last())
                            .map(|r| r.char_style_id)
                            .unwrap_or(0);
                        let lang_index = composed
                            .lines
                            .first()
                            .and_then(|l| l.runs.last())
                            .map(|r| r.lang_index)
                            .unwrap_or(0);
                        let ts = resolved_to_text_style(styles, char_style_id, lang_index);
                        // [Task #555] PUA 옛한글 char 은 자모 시퀀스로 변환 후 폭 측정.
                        let remaining_metrics: String = {
                            use super::super::pua_oldhangul::map_pua_old_hangul;
                            remaining_trimmed
                                .chars()
                                .flat_map(|ch| {
                                    if let Some(jamos) = map_pua_old_hangul(ch) {
                                        jamos.iter().copied().collect::<Vec<_>>()
                                    } else {
                                        vec![ch]
                                    }
                                })
                                .collect()
                        };
                        let text_w = estimate_text_width(&remaining_metrics, &ts);
                        let text_baseline = ts.font_size * 0.85;
                        let text_h = ts.font_size * 1.2;
                        // 마지막 Shape 높이 기준으로 텍스트 y 계산
                        let last_shape_h = para
                            .controls
                            .iter()
                            .rev()
                            .find_map(|c| {
                                if let Control::Shape(s) = c {
                                    if s.common().treat_as_char {
                                        Some(hwpunit_to_px(s.common().height as i32, self.dpi))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(0.0);
                        let text_y = para_y_before_compose + (last_shape_h - text_h).max(0.0);
                        let text_node_id = tree.next_id();
                        let text_node = RenderNode::new(
                            text_node_id,
                            RenderNodeType::TextRun(TextRunNode {
                                text: remaining_trimmed.to_string(),
                                style: ts,
                                char_shape_id: Some(char_style_id),
                                para_shape_id: Some(composed.para_style_id),
                                section_index: Some(section_index),
                                para_index: None,
                                char_start: None,
                                cell_context: None,
                                is_para_end: false,
                                is_line_break_end: false,
                                rotation: 0.0,
                                is_vertical: false,
                                char_overlap: None,
                                border_fill_id: 0,
                                baseline: text_baseline,
                                field_marker: FieldMarkerType::None,
                                display_text: None,
                            }),
                            BoundingBox::new(inline_x, text_y, text_w, text_h),
                        );
                        cell_node.children.push(text_node);
                    }
                }
            }

            if has_table_ctrl {
                // LINE_SEG vpos 기반으로 para_y 보정.
                // LINE_SEG.line_height에는 중첩 표 높이가 미포함될 수 있으므로
                // layout_table 반환값과 vpos 기반 중 적절한 값을 선택한다.
                let is_last_para = cp_idx + 1 == composed_paras.len();
                // 다음 문단의 vpos가 있으면 그것을 기준으로 para_y 보정
                if !is_last_para {
                    if let Some(next_para) = cell.paragraphs.get(cp_idx + 1) {
                        if let Some(next_seg) = next_para.line_segs.first() {
                            let next_vpos_y =
                                text_y_start + hwpunit_to_px(next_seg.vertical_pos, self.dpi);
                            // layout_table 기반 para_y와 다음 문단 vpos 중
                            // 더 큰 값 사용 (표가 LINE_SEG보다 클 수 있으므로)
                            para_y = para_y.max(next_vpos_y);
                        }
                    }
                }
                // 음수 line_spacing 처리 (중첩 구조에서 para_y 되돌리기)
                if !(is_last_para && enclosing_cell_ctx.is_some()) {
                    if let Some(last_line) = composed.lines.last() {
                        let ls = hwpunit_to_px(last_line.line_spacing, self.dpi);
                        if ls < -0.01 {
                            para_y += ls;
                        }
                    }
                }
            }
        }
    }

    /// 각 셀 레이아웃 (배경, 패딩, 텍스트, 컨트롤, 테두리)
    #[allow(clippy::too_many_arguments)]
    fn layout_table_cells(
        &self,
        tree: &mut PageLayoutContext,
        table_node: &mut RenderNode,
        table: &crate::model::table::Table,
        section_index: usize,
        styles: &ResolvedStyleSet,
        outline_numbering_id: u16,
        col_area: &LayoutRect,
        bin_data_content: &[BinDataContent],
        depth: usize,
        table_meta: Option<(usize, usize)>,
        outer_host_stored_vpos_hu: Option<i32>,
        enclosing_cell_ctx: Option<CellContext>,
        row_col_x: &[Vec<f64>],
        row_y: &[f64],
        independent_col_row_y: Option<&[Vec<f64>]>,
        col_count: usize,
        row_count: usize,
        table_x: f64,
        table_y: f64,
        h_edges: &mut Vec<Vec<Option<BorderLine>>>,
        v_edges: &mut Vec<Vec<Option<BorderLine>>>,
        row_filter: Option<(usize, usize)>,
        row_y_shift: f64,
        split_y_offset: f64,
        scalar_single_row_continuation: bool,
        single_row_continuation_offset: Option<f64>,
        single_row_fragment: bool,
        single_row_fragment_content_offset: Option<f64>,
        force_source_start_cut: bool,
        replay_terminal_boundary_unit: bool,
        split_terminal: bool,
        clamp_header_negative_para_offset: bool,
        inline_table_flow_y_shift: f64,
        nested_non_tac_cell_margin_compat: bool,
        cellzone_diagonal_origin_covered: &[Vec<bool>],
    ) {
        let mut independent_border_nodes: Vec<RenderNode> = Vec::new();
        for (cell_idx, cell) in table.cells.iter().enumerate() {
            let c = cell.col as usize;
            let r = cell.row as usize;
            if c >= col_count || r >= row_count {
                continue;
            }

            // 행 범위 필터: 보이는 행에 겹치지 않는 셀은 스킵
            let cell_end_row = (r + cell.row_span as usize).min(row_count);
            if let Some((sr, er)) = row_filter {
                if cell_end_row <= sr || r >= er {
                    continue;
                }
            }

            let cell_x = table_x + row_col_x[r][c];
            let cell_col_y = independent_col_row_y.and_then(|col_y| col_y.get(c));
            // row_y는 이미 시프트된 상태이므로 음수일 수 있음 (start_row 이전 행).
            // 독립 셀 높이가 있는 표는 해당 열의 누적 y를 사용한다.
            let raw_cell_y = table_y
                + cell_col_y
                    .and_then(|cy| cy.get(r).copied())
                    .unwrap_or(row_y[r]);
            let cell_y = if row_filter.is_some() {
                raw_cell_y.max(table_y)
            } else {
                raw_cell_y
            };
            let end_col = (c + cell.col_span as usize).min(col_count);
            let end_row = (r + cell.row_span as usize).min(row_count);
            let cell_w = row_col_x[r][end_col] - row_col_x[r][c];
            let raw_cell_h = cell_col_y
                .and_then(|cy| {
                    let start = cy.get(r).copied()?;
                    let end = cy.get(end_row).copied()?;
                    Some(end - start)
                })
                .unwrap_or_else(|| row_y[end_row] - row_y[r]);
            let cell_h = if row_filter.is_some() {
                // 클램프된 y에 맞게 높이도 조정
                (raw_cell_h - (cell_y - raw_cell_y)).max(0.0)
            } else {
                raw_cell_h
            };
            let content_cell_y = if row_filter.is_some() {
                cell_y - split_y_offset
            } else {
                cell_y
            };

            // [#4698] 쪽 경계에 걸친 병합(rowspan) 라벨 셀은 한컴이 이미 문단을
            // 쪽별 조각으로 나눠 저장해 두었다(조각 첫 문단의 vpos 가 0 으로 재시작).
            // 종전에는 앞 조각이 셀 문단 전부를 받아 조각 하단을 넘긴 줄이 Cell clip
            // 으로 사라지고, 뒤 조각은 [#1073] 로 통째로 비워져 그 줄이 어느 쪽에도
            // 남지 않았다(kps-ai p65 `시장침해` 4자 소실). 이 조각이 소유하는 문단
            // 범위를 저장 조각 그대로 골라 셀을 잘라 넘긴다.
            let fragment_para_range = row_filter.and_then(|(sr, er)| {
                let straddles = cell.row_span > 1 && (r < sr || cell_end_row > er);
                if !straddles {
                    return None;
                }
                let groups = stored_cell_fragment_groups(cell);
                if groups.len() < 2
                    || groups
                        .iter()
                        .any(|&g| stored_fragment_extent_hu(cell, g) <= 0)
                {
                    return None;
                }
                // 이 조각 위쪽으로 이미 앞 쪽들이 소비한 셀 높이
                let consumed = (cell_y - raw_cell_y).max(0.0);
                let mut used = 0.0f64;
                let mut selected: Option<(usize, usize)> = None;
                for &g in &groups {
                    let h = hwpunit_to_px(stored_fragment_extent_hu(cell, g), self.dpi);
                    match selected {
                        None => {
                            if used + h <= consumed + 0.5 {
                                used += h;
                                continue;
                            }
                            selected = Some(g);
                            used = h;
                        }
                        Some((start, _)) => {
                            if used + h > cell_h + 0.5 {
                                break;
                            }
                            used += h;
                            selected = Some((start, g.1));
                        }
                    }
                }
                selected
            });
            let fragment_cell = fragment_para_range.map(|(start, end)| {
                let mut fragment = cell.clone();
                fragment.paragraphs = cell.paragraphs[start..end].to_vec();
                fragment
            });
            let cell = fragment_cell.as_ref().unwrap_or(cell);

            let cell_id = tree.next_id();
            let mut cell_node = RenderNode::new(
                cell_id,
                RenderNodeType::TableCell(TableCellNode {
                    col: cell.col,
                    row: cell.row,
                    col_span: cell.col_span,
                    row_span: cell.row_span,
                    border_fill_id: cell.border_fill_id,
                    text_direction: cell.text_direction,
                    clip: true,
                    model_cell_index: Some(cell_idx as u32),
                }),
                BoundingBox::new(cell_x, cell_y, cell_w, cell_h),
            );

            // 셀 BorderFill 조회
            let border_style = if cell.border_fill_id > 0 {
                let idx = (cell.border_fill_id as usize).saturating_sub(1);
                styles.border_styles.get(idx)
            } else {
                None
            };

            // (a) 셀 배경
            self.render_cell_background(
                tree,
                &mut cell_node,
                border_style,
                cell_x,
                cell_y,
                cell_w,
                cell_h,
                bin_data_content,
            );

            // 셀 패딩 (cell.padding이 0이면 table.padding fallback)
            let (mut pad_left, mut pad_right, pad_top, pad_bottom) = self
                .resolve_cell_padding_for_context(cell, table, nested_non_tac_cell_margin_compat);

            let mut composed_paras: Vec<_> = cell
                .paragraphs
                .iter()
                .map(|p| compose_paragraph(p))
                .collect();

            // [Task #1073] 중첩 표 분할 연속 페이지(row_filter sr>0)에서 분할 시작 행보다
            // 먼저 시작한 rowspan 셀(r < sr)은 라벨이 이전 페이지에 이미 렌더됨 → 연속
            // 페이지에선 공란(영역/배경만, 텍스트 미렌더). 외부 표 advance_row_block_cut 의
            // rs>1 라벨 공란 정합. row_filter 는 중첩 표 분할 전용(외부 표는 별도 경로).
            // 저장 조각을 골라 넘긴 셀(#4698)은 이 조각이 소유하는 문단만 담고 있으므로
            // 비우지 않는다 — 비우면 그 문단이 어느 쪽에도 남지 않는다.
            if let Some((sr, _)) = row_filter {
                if sr > 0 && r < sr && fragment_para_range.is_none() {
                    composed_paras.clear();
                }
            }

            // 텍스트 오버플로우 시 좌우 패딩 축소.
            // 1443 셀 안여백 샘플처럼 큰 명시 좌우 여백은 한컴과 같이 보존하되,
            // 기존 문서의 1~4mm급 일반 셀 여백은 종전 오버플로우 방어를 유지한다.
            let preserve_explicit_horizontal_padding = (cell.apply_inner_margin && cell.padding.left.max(cell.padding.right) >= 1700)
                    // #2308 p34: 중첩 비글자표의 510HU 저장 margin은 일반 overflow
                    // 추정보다 한컴 PDF가 우선한다. 이 여백을 다시 1px까지 깎으면
                    // 문단이 우측 테두리와 맞닿는다.
                    || (nested_non_tac_cell_margin_compat
                        && cell.padding.left.max(cell.padding.right)
                            > table.padding.left.max(table.padding.right)
                        && cell.padding.left.max(cell.padding.right) < 2500);
            let (new_pl, new_pr) = self.shrink_cell_padding_for_overflow(
                pad_left,
                pad_right,
                cell_w,
                &composed_paras,
                &cell.paragraphs,
                styles,
                preserve_explicit_horizontal_padding,
            );
            pad_left = new_pl;
            pad_right = new_pr;

            let inner_x = cell_x + pad_left;
            let inner_width = (cell_w - pad_left - pad_right).max(0.0);
            let inner_height = (cell_h - pad_top - pad_bottom).max(0.0);

            // [Task #671] line_segs 비어 있는 셀 paragraph 의 단일 ComposedLine 압축
            // 결과를 셀 가용 너비 (inner_width) 에 맞춰 다중 ComposedLine 으로 재분할.
            // 한컴이 PARA_LINE_SEG 를 인코딩하지 않은 케이스 (samples/계획서.hwp) 의
            // 줄겹침 시각 결함 정정. 정상 line_segs 인코딩된 paragraph 는 무영향.
            for (cpi, para) in cell.paragraphs.iter().enumerate() {
                if let Some(comp) = composed_paras.get_mut(cpi) {
                    crate::renderer::composer::recompose_for_cell_width(
                        comp,
                        para,
                        inner_width,
                        styles,
                    );
                }
            }

            // AutoNumber(Page) 치환: 셀 내 쪽번호 필드를 현재 페이지 번호로 변환
            let current_pn = self.current_page_number.get();
            if current_pn > 0 {
                for (cpi, para) in cell.paragraphs.iter().enumerate() {
                    if para.controls.iter().any(|c| {
                        matches!(c, Control::AutoNumber(an)
                            if an.number_type == crate::model::control::AutoNumberType::Page)
                    }) {
                        if let Some(comp) = composed_paras.get_mut(cpi) {
                            self.substitute_page_auto_numbers_in_composed(para, comp, current_pn);
                        }
                    }
                }
            }

            // AutoNumber(TotalPage) 치환: 셀 내 총쪽수 필드를 문서 전체 쪽수로 변환.
            // exam_eng.hwp 같은 시험지의 꼬리말 쪽번호 상자는 현재쪽/총쪽수 두 atno를
            // 같은 셀 안에 서로 다른 문단으로 둔다 — Page만 치환하면 총쪽수 자리에도
            // 현재 쪽번호가 그려진다 (Task: 꼬리말 총쪽수 필드 미치환 버그).
            let total_pages = self.total_pages.get();
            if total_pages > 0 {
                for (cpi, para) in cell.paragraphs.iter().enumerate() {
                    if para.controls.iter().any(|c| {
                        matches!(c, Control::AutoNumber(an)
                            if an.number_type == crate::model::control::AutoNumberType::TotalPage)
                    }) {
                        if let Some(comp) = composed_paras.get_mut(cpi) {
                            self.substitute_total_page_auto_numbers_in_composed(
                                para,
                                comp,
                                total_pages,
                            );
                        }
                    }
                }
            }

            // 인라인 이미지/도형 최대 높이
            let mut max_inline_height: f64 = 0.0;

            // 수직 정렬용 콘텐츠 높이
            // (A) composed 기반: LINE_SEG line_height 합산 + 비인라인 도형/그림
            let total_content_height: f64 = {
                let mut text_height: f64 = self.calc_composed_paras_content_height(
                    &composed_paras,
                    &cell.paragraphs,
                    styles,
                );
                for para in &cell.paragraphs {
                    text_height +=
                        self.paragraph_top_and_bottom_non_inline_flow_height(&para.controls);
                    for ctrl in &para.controls {
                        match ctrl {
                            Control::Picture(pic) => {
                                let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                if pic.common.treat_as_char {
                                    if pic_h > max_inline_height {
                                        max_inline_height = pic_h;
                                    }
                                }
                            }
                            Control::Shape(shape) => {
                                let shape_h = hwpunit_to_px(shape.common().height as i32, self.dpi);
                                if shape.common().treat_as_char {
                                    if shape_h > max_inline_height {
                                        max_inline_height = shape_h;
                                    }
                                }
                            }
                            Control::Equation(eq) => {
                                let eq_h = hwpunit_to_px(eq.common.height as i32, self.dpi);
                                if eq.common.treat_as_char {
                                    if eq_h > max_inline_height {
                                        max_inline_height = eq_h;
                                    }
                                } else {
                                    text_height += eq_h;
                                }
                            }
                            // [Task #1658] 중첩 표 높이를 composed(text_height)에 가산하지 않는다.
                            // 가산하면 stored vpos(last_seg_end, nested 포함) 및 아래 nested_bottom
                            // 과 double-count 되어 total_content_height 가 ~2× 과대 → Center/Bottom
                            // offset≈0 → 상단정렬(valign over-count, kkyu8925 제보). 중첩 표 기여는
                            // final max 의 vpos_height(B)·nested_bottom 이 담당하며, composed 의
                            // line_height 가 중첩을 반영하는 케이스는 composed 가, 미반영(과소)
                            // 케이스는 nested_bottom 이 max 로 보정한다(#44 under-count 가드 보존).
                            Control::Table(_) => {}
                            _ => {}
                        }
                    }
                }
                let composed_height = text_height.max(max_inline_height);

                // (B) vpos 기반: 마지막 문단의 vpos_end + 중첩 표 보정
                // LINE_SEG lh에 중첩 표 높이가 미반영된 경우를 보정
                let vpos_height = if cell.paragraphs.len() > 1 {
                    let last_para = cell.paragraphs.last().unwrap();
                    if let Some(seg) = last_para.line_segs.last() {
                        let mut last_end = seg.vertical_pos.saturating_add(seg.line_height);
                        // 마지막 문단에 중첩 표가 있고 lh가 표 높이보다 작으면 보정
                        for ctrl in &last_para.controls {
                            if let Control::Table(t) = ctrl {
                                let table_h = t.common.height as i32;
                                if table_h > seg.line_height {
                                    last_end += table_h - seg.line_height;
                                }
                            }
                        }
                        hwpunit_to_px(last_end, self.dpi)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                let nested_bottom =
                    self.calc_nested_controls_bottom_height(&cell.paragraphs, styles);
                let wrap_object_bottom =
                    self.calc_cell_wrap_objects_bottom_height(&cell.paragraphs);
                composed_height
                    .max(vpos_height)
                    .max(nested_bottom)
                    .max(wrap_object_bottom)
            };

            // 수직 정렬 (분할 표에서는 Top 강제 — 보이는 영역이 전체 셀보다 작음)
            // 중첩 표 행 범위 부분 렌더에서, **셀이 실제로 잘릴 때만** Top 을 강제한다.
            //
            // `row_filter` 는 행 단위로 자르므로 필터 안에 온전히 들어가는 셀은 잘리지
            // 않는다. 그런 셀까지 Top 으로 덮으면 세로로 긴 병합 라벨이 정중앙이 아니라
            // 맨 위에 붙는다 (한컴 pdf/kps-ai-2022.pdf p65 실측 = 정중앙. rhwp p66 은
            // 상단). 실측: kps-ai 의 Center 지정 셀 57건 중 55건이 안 잘리는데도 Top
            // 강제를 받았다.
            //
            // 잘리는 두 경우는 이 조건과 무관하게 결과가 Top 으로 수렴한다 —
            // 상단 잘림(r < sr)은 라벨이 앞 페이지에 이미 렌더돼 문단을 비우고(아래 #1073
            // 처리), 하단 잘림은 콘텐츠가 가시 높이를 넘어 정렬 오프셋이 0 으로 클램프된다.
            // 그래도 종전 동작을 그 두 경우에 한해 그대로 남긴다.
            let cell_clipped_by_row_filter = row_filter.is_some_and(|(sr, er)| {
                let cell_end_row = (r + cell.row_span as usize).min(row_count);
                r < sr || cell_end_row > er
            });
            // 이 표 자신은 `nested_split`을 받지 않아도, 현재 표가 부모 1×1
            // RowBreak continuation의 남은 viewport 안에서 호출될 수 있다. 이때 큰
            // 하위 셀은 부모 Cell clip에 의해 위나 아래가 잘린다. 원래 Center/Bottom
            // 정렬을 유지하면 잘린 반대쪽의 보이지 않는 공간을 기준으로 본문이 다시
            // 밀린다. 특히 위쪽만 잘린 p11에서는 앞 쪽에 이미 그린 문단군을 다시
            // 현재 페이지로 끌어내려 중복했다(42065). `col_area`는 호출자가 넘긴
            // 물리 viewport이므로, 그와 교차하면서 한쪽이라도 벗어난 중첩 셀만
            // Top으로 수렴시킨다. 다만 p11처럼 호출자가 전한 `col_area`가 직전
            // 조각까지 포함할 수 있으므로, 실제 페이지 viewport에서도 같은 판정을
            // 한다. 일반 완전 셀 및 최상위 표(depth=0)는 영향이 없다.
            let parent_view_top = col_area.y;
            let parent_view_bottom = col_area.y + col_area.height;
            let cell_intersects_parent_viewport =
                cell_y < parent_view_bottom - 0.5 && cell_y + cell_h > parent_view_top + 0.5;
            let cell_clipped_by_parent_viewport = depth > 0
                && !table.common.treat_as_char
                && col_area.height > 0.5
                && cell_intersects_parent_viewport
                && (cell_y < parent_view_top - 0.5 || cell_y + cell_h > parent_view_bottom + 0.5);
            // nested continuation은 부모 `col_area`가 이전 페이지의 logical
            // viewport를 포함한 채 호출될 수 있다. 렌더 트리의 page bbox는 실제
            // SVG/Canvas clip이므로, 그 밖으로 나간 셀은 그 logical viewport 안에
            // 있더라도 Center/Bottom 기준으로 배치하면 안 된다.
            let page_bbox = tree.page_bbox();
            let page_view_top = page_bbox.y;
            let page_view_bottom = page_bbox.y + page_bbox.height;
            let cell_intersects_page_viewport =
                cell_y < page_view_bottom - 0.5 && cell_y + cell_h > page_view_top + 0.5;
            let cell_clipped_by_page_viewport = depth > 0
                && !table.common.treat_as_char
                && cell_intersects_page_viewport
                && (cell_y < page_view_top - 0.5 || cell_y + cell_h > page_view_bottom + 0.5);
            // 위쪽 continuation에서는 source의 첫 가시 줄이 page clip 직전까지
            // 내려와 있다. Top을 셀의 논리 원점에 그대로 맞추면 그 줄의 잉크가
            // clip 바로 위에서 잘리고 다음 줄부터 나타난다(42065 p11: PDF의
            // "행하여야 하며 …"가 사라지고 "제50조의4"부터 시작). 첫 *유효*
            // 저장 줄의 물리 line-height만큼 예약해 그 줄을 clip 안으로 되돌린다.
            // HWP는 표 안의 빈 anchor line을 line_height=0으로 먼저 저장할 수 있어
            // 단순 first()를 쓰면 이 보정이 무효가 된다. 아래쪽 잘림이나 정상 완전
            // 셀에는 적용하지 않는다.
            let upper_page_clip_line_reservation =
                if cell_clipped_by_page_viewport && cell_y < page_view_top - 0.5 {
                    cell.paragraphs
                        .iter()
                        .flat_map(|para| para.line_segs.iter())
                        .find(|seg| seg.line_height > 0)
                        .map(|seg| hwpunit_to_px(seg.line_height, self.dpi))
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
            let effective_valign = if cell_clipped_by_row_filter
                || scalar_single_row_continuation
                || cell_clipped_by_parent_viewport
                || cell_clipped_by_page_viewport
            {
                VerticalAlign::Top
            } else {
                cell.vertical_align
            };
            // Task #347: HWP는 LineSeg.vertical_pos에 첫 줄의 절대 위치(셀 내부 컨텐츠 상단부터)
            // 를 기록한다. 다만 이 값을 모든 vertical_align에 곧바로 적용하면 Center/Bottom
            // 지정 셀도 Top처럼 배치된다. vpos 앵커링은 Top 셀의 세부 줄 위치 보정으로만
            // 사용하고, Center/Bottom은 전체 콘텐츠 높이 기반의 기존 정렬 계산을 유지한다.
            // 단, line_segs가 비어있는 Top 케이스는 기존 폴백 유지.
            // [Task #362] 셀 안에 nested table 이 있는 경우 vpos 적용 제외.
            // nested table 케이스에서 LineSeg.vpos 가 셀 콘텐츠 시작 오프셋 의미가 아니라
            // 셀 안의 누적 위치로 사용되어, vpos 를 추가하면 콘텐츠가 표 높이를 초과하여 클립 발생.
            // (kps-ai p56 case: 외부 셀 vpos=2000HU 가 추가되어 19.5px 클립.)
            let has_nested_table = cell
                .paragraphs
                .iter()
                .any(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))));
            // HWPX block-TAC 셀의 nested table은 예외다. 이 형상은 모든 문단의
            // 연속된 LineSeg.vpos가 셀 기준 좌표를 보존하며, 이를 무시하면
            // Center 정렬의 순차 flow가 누적되어 실제로 fit하는 하위 표가 다음
            // 쪽으로 clip된다 (#3820 production HWPX p144). 아래의 stored-flow
            // 신뢰 조건(연속 anchor, extent, 비-flow 객체)을 통과한 경우에만 허용해
            // Task #362의 일반 nested-table 누적-vpos 차단은 유지한다.
            let hwpx_noninline_tac_nested_stored_flow = self.profile.get().hwpx_stored_layout()
                && table.common.treat_as_char
                && !table.common.flow_with_text
                && matches!(table.page_break, TablePageBreak::None)
                && has_nested_table;
            let first_line_vpos = cell
                .paragraphs
                .first()
                .and_then(|p| p.line_segs.first())
                .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi));
            // [Task #2211] 저장 LINE_SEG 흐름 extent(각 seg 의 vpos+lh 최댓값)가
            // 자체 스택 합(total_content_height)보다 작으면 — 예: 악보 셀처럼
            // 빈 앵커 줄이 TopAndBottom 그림 높이에 흡수된 문서 — 한컴 저장
            // 지오메트리를 신뢰한다: 정렬 기준 콘텐츠 높이를 저장 extent 로
            // 바꾸고, 문단 배치도 저장 vpos 스냅을 강제한다 (한컴 실측:
            // 가사 top = 셀 top + pad + 센터 오프셋(저장 extent 기준) + vpos).
            let (stored_flow_extent, stored_flow_line_sum) = if (!has_nested_table
                || hwpx_noninline_tac_nested_stored_flow)
                && !cell.paragraphs.is_empty()
                && cell.paragraphs.iter().all(|p| !p.line_segs.is_empty())
            {
                cell.paragraphs
                    .iter()
                    .flat_map(|p| p.line_segs.iter())
                    .filter(|s| s.vertical_pos >= 0 && s.line_height > 0)
                    .map(|s| {
                        (
                            hwpunit_to_px(s.vertical_pos.saturating_add(s.line_height), self.dpi),
                            hwpunit_to_px(s.line_height, self.dpi),
                        )
                    })
                    .fold((0.0f64, 0.0f64), |(ext, sum), (e, h)| (ext.max(e), sum + h))
            } else {
                (0.0, 0.0)
            };
            // Square/중첩 표 등 비-flow 개체의 시각 bottom 은 저장 LINE_SEG 흐름에
            // 포함되지 않으므로(#1486 p19 Square 그림), 그런 개체가 저장 extent 를
            // 넘는 셀은 저장 흐름 신뢰 대상이 아니다 — TopAndBottom flow 개체만
            // 저장 vpos 에 흡수된다(악보 셀).
            let non_flow_object_extent = self
                .calc_nested_controls_bottom_height(&cell.paragraphs, styles)
                .max(self.calc_cell_wrap_objects_bottom_height(&cell.paragraphs));
            // [#2148 #2279] 저장 vpos 흐름이 물리적으로 줄들을 담지 못하는 퇴화
            // 형상(다문단 전부 vpos=0 등, 36399374 pi=79 병합 셀: extent 35px vs
            // 줄높이 합 260px)은 신뢰 대상이 아니다 — 전 문단이 셀 상단 한 y 에
            // 겹쳐 그려진다(한글은 fresh 재적층). 음수 line_spacing 누적 보정용
            // 정상 vpos 스냅(조직도형·악보 셀)은 extent ≈ 줄높이 합이므로 0.5
            // 비율 가드에 걸리지 않는다.
            // 위 비율 가드는 문단이 2개인 셀에서 경계에 정확히 걸려 통과한다
            // (전 문단 vpos=0, lh 동일 → extent = 줄높이합/2). 그래서 문단 단위
            // 앵커 유무를 직접 본다: 둘째 이후 문단의 first seg vpos == 0 은
            // "앵커 없음" 센티널이므로, 그런 문단이 있으면 저장 흐름은 문단 위치를
            // 구분해 담고 있지 않다. 이때 extent 를 콘텐츠 높이로 받아들이면 세로
            // 정렬 오프셋과 담을 줄 수가 1줄분으로 굳어 뒤 문단이 셀 밖으로 밀려
            // 잘리거나 아예 렌더되지 않는다. 배치 쪽 first_seg_vpos_is_anchor 와
            // 같은 규약을 측정에도 적용한다.
            let stored_flow_has_para_anchors = cell
                .paragraphs
                .iter()
                .enumerate()
                .all(|(idx, para)| crate::renderer::first_seg_vpos_is_anchor(para, idx));
            let stored_flow_shape_is_trusted = (depth > 0 || table.common.treat_as_char)
                && stored_flow_extent > 0.0
                && non_flow_object_extent <= stored_flow_extent + 0.5
                && stored_flow_extent + 0.5 >= 0.5 * stored_flow_line_sum
                && stored_flow_has_para_anchors;
            // 일반 셀은 저장 extent가 자체 측정값보다 실제로 압축된 경우에만
            // anchor를 신뢰한다. 다만 위의 좁은 HWPX block-TAC nested-table 형상은
            // extent와 자체 측정값이 같아도 문단별 vpos가 하위 표의 실제 위치를
            // 담고 있다. 이 경우에는 total height는 변하지 않지만 순차 배치만
            // 저장 anchor로 복원해야 한다 (#3820 p144).
            let trust_stored_cell_flow = stored_flow_shape_is_trusted
                && (stored_flow_extent + 0.5 < total_content_height
                    || (hwpx_noninline_tac_nested_stored_flow
                        && (stored_flow_extent - total_content_height).abs() <= 0.5));
            let total_content_height = if trust_stored_cell_flow {
                stored_flow_extent
            } else {
                total_content_height
            };
            let use_top_vpos_anchor = matches!(effective_valign, VerticalAlign::Top);
            let text_y_start = if use_top_vpos_anchor
                && !has_nested_table
                && first_line_vpos.filter(|&v| v > 0.0).is_some()
            {
                // vpos는 셀 컨텐츠 상단(=cell_y+pad_top)으로부터의 첫 줄 top y 오프셋
                content_cell_y + pad_top + first_line_vpos.unwrap()
            } else {
                match effective_valign {
                    VerticalAlign::Top => content_cell_y + pad_top,
                    VerticalAlign::Center => {
                        let mechanical_offset =
                            (inner_height - total_content_height).max(0.0) / 2.0;
                        content_cell_y + pad_top + mechanical_offset
                    }
                    VerticalAlign::Bottom => {
                        content_cell_y + pad_top + (inner_height - total_content_height).max(0.0)
                    }
                }
            };
            let text_y_start = text_y_start + upper_page_clip_line_reservation;
            // 세로쓰기 셀
            if cell.text_direction != 0 {
                let vert_inner_area = LayoutRect {
                    x: inner_x,
                    y: content_cell_y + pad_top,
                    width: inner_width,
                    height: inner_height,
                };
                self.layout_vertical_cell_text(
                    tree,
                    &mut cell_node,
                    &composed_paras,
                    &cell.paragraphs,
                    styles,
                    &vert_inner_area,
                    cell.vertical_align,
                    cell.text_direction,
                    section_index,
                    table_meta,
                    cell_idx,
                    enclosing_cell_ctx.clone(),
                );
            } else {
                self.layout_horizontal_cell_paragraphs(
                    tree,
                    table_node,
                    &mut cell_node,
                    cell,
                    &composed_paras,
                    table,
                    styles,
                    bin_data_content,
                    table_meta,
                    &enclosing_cell_ctx,
                    row_filter,
                    row_y,
                    effective_valign,
                    HorizontalCellVars {
                        cell_idx,
                        r,
                        cell_y,
                        cell_h,
                        content_cell_y,
                        pad_top,
                        inner_x,
                        inner_width,
                        inner_height,
                        text_y_start,
                        use_top_vpos_anchor,
                        upper_clip_line_reservation: upper_page_clip_line_reservation,
                        trust_stored_cell_flow,
                        has_nested_table,
                        section_index,
                        outline_numbering_id,
                        depth,
                        clamp_header_negative_para_offset,
                        outer_host_stored_vpos_hu,
                        inline_table_flow_y_shift,
                        single_row_continuation: scalar_single_row_continuation,
                        single_row_continuation_offset,
                        single_row_fragment,
                        single_row_fragment_content_offset,
                        force_source_start_cut,
                        replay_terminal_boundary_unit,
                        split_terminal,
                    },
                );
            } // else (가로쓰기)

            // 셀 내 각주 참조 번호 윗첨자
            for para in &cell.paragraphs {
                self.add_footnote_superscripts(tree, &mut cell_node, para, styles);
            }

            // (b) 셀 테두리를 수집한다. 열별 높이가 다른 표는 row_y 격자로
            // 테두리를 그릴 수 없으므로 셀 bbox 기준 라인을 별도로 생성한다.
            if let Some(bs) = border_style {
                if independent_col_row_y.is_some() {
                    independent_border_nodes.extend(render_cell_box_borders(
                        tree, bs, cell_x, cell_y, cell_w, cell_h,
                    ));
                } else {
                    collect_cell_borders(
                        h_edges,
                        v_edges,
                        c,
                        r,
                        cell.col_span as usize,
                        cell.row_span as usize,
                        &bs.borders,
                    );
                }
            }

            table_node.children.push(cell_node);

            // (c) 셀 대각선 렌더링 (셀 콘텐츠 위에 그림)
            let suppress_cell_diagonal = cell_span_has_cellzone_diagonal(
                cellzone_diagonal_origin_covered,
                r,
                c,
                cell.row_span as usize,
                cell.col_span as usize,
                row_count,
                col_count,
            );
            if let Some(bs) = border_style {
                if !suppress_cell_diagonal || border_style_has_center_line_only(bs) {
                    table_node.children.extend(render_cell_diagonal(
                        tree, bs, cell_x, cell_y, cell_w, cell_h,
                    ));
                }
            }
        }
        if !independent_border_nodes.is_empty() {
            table_node.children.extend(independent_border_nodes);
        }
    }

    pub(crate) fn calc_cell_controls_height(
        &self,
        cell: &crate::model::table::Cell,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let measurer = super::super::height_measurer::HeightMeasurer::new(self.dpi)
            .with_hwp3_variant(self.profile.get().hwp3_layout())
            .with_native_hwp5(self.profile.get().native_hwp5_layout())
            .with_render_normalization(self.render_normalization_overlay());
        measurer.cell_controls_height(&cell.paragraphs, styles, 0, 0.0)
    }

    /// 중첩 표의 총 높이를 계산한다 (행 높이 합 + cell_spacing).
    /// MeasuredCell.line_heights에서 중첩 표가 추가 줄로 포함될 때의 높이와 일관되게 계산.
    pub(crate) fn calc_nested_table_height(
        &self,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let col_count = table.col_count as usize;
        let row_count = table.row_count as usize;
        let row_heights = self.resolve_row_heights(table, col_count, row_count, None, styles, true);
        let cell_spacing = hwpunit_to_px(table.cell_spacing as i32, self.dpi);
        let om_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
        let om_bottom = hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi);
        row_heights.iter().sum::<f64>()
            + cell_spacing * (row_count.saturating_sub(1) as f64)
            + om_top
            + om_bottom
    }

    /// 셀 내 중첩 표가 실제로 차지하는 하단 위치를 계산한다.
    ///
    /// 일부 HWP/HWPX는 중첩 표 문단의 LINE_SEG.line_height에 내부 표의 실제
    /// 높이를 반영하지 않는다. 렌더링/측정은 해당 문단의 vertical_pos에 중첩 표
    /// 측정 높이를 더한 값을 셀 콘텐츠 끝점 후보로 사용한다.
    pub(crate) fn calc_nested_controls_bottom_height(
        &self,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        // [#4533] `para_top + nested_h` 는 "중첩 표가 앵커 문단 아래로 흐른다"는
        // 가정이다. 앵커 줄이 셀 하단에 있고 표가 셀 상단에 절대배치되는 서식
        // 문서(기장군 20420347: para_top 740.9 + 601.8 = 1342.6 vs 선언 794.1)
        // 에서는 이 가정이 셀을 548px 부풀려 후속 문단을 쪽 밖으로 민다.
        // 호스트 **뒤에** 저장 사다리가 이어지면(뒤 문단 저장 vpos 존재) 그
        // 사다리가 흐름-표 높이까지 이미 증명하므로 사다리 끝점으로 캡한다.
        // 호스트가 마지막 문단이면 기존 휴리스틱 유지(lh 미반영 문서의 원 목적).
        let native_hwp5 = self.profile.get().native_hwp5_layout();
        let ladder_end: f64 = if native_hwp5
            && paragraphs
                .iter()
                .all(|p| !crate::renderer::para_has_no_stored_line_segs(p))
        {
            paragraphs
                .iter()
                .flat_map(|p| p.line_segs.iter())
                .map(|s| hwpunit_to_px(s.vertical_pos.saturating_add(s.line_height), self.dpi))
                .fold(0.0f64, f64::max)
        } else {
            0.0
        };
        paragraphs
            .iter()
            .enumerate()
            .map(|(pidx, p)| {
                let nested_h: f64 = p
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                if nested_h <= 0.0 {
                    0.0
                } else {
                    let para_top = p
                        .line_segs
                        .first()
                        .map(|s| hwpunit_to_px(s.vertical_pos, self.dpi))
                        .unwrap_or(0.0);
                    let candidate = para_top + nested_h;
                    // 절대배치의 직접 증거는 "표의 공간이 호스트 줄 **위**에 이미
                    // 예약됨"이다 — 직전 저장 줄 끝→호스트 vpos 갭이 표 높이만큼
                    // 벌어진다(기장군 612 vs 표 598 · 수면 작성례 563 vs 536.5 —
                    // 호스트가 셀 마지막 문단인 변형도 같은 식으로 갈린다). 흐름형
                    // (lh 미흡수 포함)은 직전 갭이 평범한 줄간격이라 배제되고,
                    // 호스트가 셀 첫 문단이면(49308 조각 셀) 직전 줄이 없어 배제된다.
                    let prev_end = paragraphs
                        .iter()
                        .take(pidx)
                        .flat_map(|prev| prev.line_segs.iter())
                        .map(|s| {
                            hwpunit_to_px(s.vertical_pos.saturating_add(s.line_height), self.dpi)
                        })
                        .filter(|&e| e <= para_top + 0.5)
                        .fold(f64::NEG_INFINITY, f64::max);
                    let gap_before = para_top - prev_end;
                    // 쪽을 넘는 거대 중첩 표는 셀 사다리가 조각-국소라 표를
                    // 기술하지 못한다(49308: nested_h 2664 vs ladder_end 122 —
                    // 캡하면 쪽수 70->69 로 한글 71쪽에서 멀어짐). 표가 사다리
                    // 안에 들어갈 때만 절대배치 캡을 허용한다.
                    let anchored_not_flowing = ladder_end > 0.0
                        && nested_h <= ladder_end
                        && prev_end.is_finite()
                        && gap_before >= nested_h * 0.85;
                    if anchored_not_flowing {
                        candidate.min(ladder_end)
                    } else {
                        candidate
                    }
                }
            })
            .fold(0.0f64, f64::max)
    }

    /// 셀의 content_offset 이후 실제 남은 콘텐츠 높이를 계산한다.
    /// MeasuredCell과 동일한 높이 로직을 사용한다 (pagination 엔진이 MeasuredCell 기준으로
    /// content_offset을 산출하므로 동일 기준이어야 함).
    pub(crate) fn calc_cell_remaining_content_height(
        &self,
        cell: &crate::model::table::Cell,
        styles: &ResolvedStyleSet,
        content_offset: f64,
    ) -> f64 {
        // MeasuredCell과 동일한 높이 계산:
        // 각 줄 h+ls, 단 셀의 마지막 줄(마지막 문단의 마지막 줄)은 ls 제외
        let mut total = 0.0;
        let cell_para_count = cell.paragraphs.len();
        for (pidx, p) in cell.paragraphs.iter().enumerate() {
            let comp = compose_paragraph(p);
            let para_style = styles.para_styles.get(p.para_shape_id as usize);
            let is_last_para = pidx + 1 == cell_para_count;
            let spacing_before = if pidx > 0 {
                para_style.map(|s| s.spacing_before).unwrap_or(0.0)
            } else {
                0.0
            };
            let spacing_after = if !is_last_para {
                para_style.map(|s| s.spacing_after).unwrap_or(0.0)
            } else {
                0.0
            };
            if comp.lines.is_empty() {
                // 중첩 표 컨트롤 문단: 실제 중첩 표 높이로 계산
                let nested_h: f64 = p
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                let h = if nested_h > 0.0 {
                    nested_h
                } else {
                    hwpunit_to_px(400, self.dpi)
                };
                total += spacing_before + h + spacing_after;
            } else {
                // 중첩 표가 있는 문단: LINE_SEG 높이와 실제 중첩 표 높이 중 큰 값 사용
                let has_table_in_para = p.controls.iter().any(|c| matches!(c, Control::Table(_)));
                let line_count = comp.lines.len();
                let line_based_h: f64 = comp
                    .lines
                    .iter()
                    .enumerate()
                    .map(|(li, line)| {
                        let h = hwpunit_to_px(line.line_height, self.dpi);
                        let is_cell_last_line = is_last_para && li + 1 == line_count;
                        let ls = if !is_cell_last_line {
                            hwpunit_to_px(line.line_spacing, self.dpi)
                        } else {
                            0.0
                        };
                        spacing_before * (if li == 0 { 1.0 } else { 0.0 })
                            + h
                            + ls
                            + spacing_after * (if li + 1 == line_count { 1.0 } else { 0.0 })
                    })
                    .sum();
                if has_table_in_para {
                    let nested_h: f64 = p
                        .controls
                        .iter()
                        .map(|ctrl| {
                            if let Control::Table(t) = ctrl {
                                self.calc_nested_table_height(t, styles)
                            } else {
                                0.0
                            }
                        })
                        .sum();
                    total += nested_h.max(line_based_h);
                } else {
                    total += line_based_h;
                }
            }
        }
        (total - content_offset).max(0.0)
    }

    /// 셀 내 문단 줄 높이로부터 content_offset/content_limit 기준 줄 범위를 계산한다.
    pub(crate) fn compute_cell_line_ranges(
        &self,
        cell: &crate::model::table::Cell,
        composed_paras: &[ComposedParagraph],
        content_offset: f64,
        content_limit: f64,
        styles: &ResolvedStyleSet,
    ) -> Vec<(usize, usize)> {
        // 셀 콘텐츠의 cumulative position(누적 px) 기반 가시성 결정.
        // - LINE_SEG.vpos 는 컬럼 리셋이 발생하므로 셀 시작부터의 누적 위치로 사용 불가 → line_height + line_spacing 누적 사용.
        // - content_offset > 0: [0, content_offset) 영역의 콘텐츠는 이전 페이지 → 스킵.
        // - content_limit > 0: [0, content_limit] 영역의 콘텐츠만 표시.
        // - 중첩 표(atomic) 문단은 분할 불가 — 경계를 걸치면 한쪽 페이지에만 렌더링.
        let has_offset = content_offset > 0.0;
        let has_limit = content_limit > 0.0;

        // [Task #991] 분할 시작/중간 페이지(has_offset)의 줄 컷을 독립 재계산하지
        // 않고, 끝 페이지 패스(prefix 패스)에서 유도한다.
        //
        // 끝 페이지(`!has_offset`)와 시작 페이지가 분할 경계를 각자 계산하면,
        // `limit_reached` 전파(Task #485)·vpos 리셋 컷(Task #697)·vpos 동기화
        // (Task #700)가 두 경로에서 다르게 작동해 줄이 중복되거나 누락된다.
        // 모든 컷을 동일한 prefix 패스(`cell_line_prefix_counts`)로 통일하면,
        // - 시작 줄 = budget `content_offset` 안에 들어가는 prefix 줄 수
        // - 끝 줄   = budget `content_offset + content_limit` 안의 prefix 줄 수
        //   (limit 없으면 문단 전체)
        // 가 되어, 끝 페이지 포함분과 정확히 상보가 된다(중복·누락 불가).
        if has_offset {
            let skip = self.cell_line_prefix_counts(cell, composed_paras, content_offset, styles);
            let keep: Vec<usize> = if has_limit {
                self.cell_line_prefix_counts(
                    cell,
                    composed_paras,
                    content_offset + content_limit,
                    styles,
                )
            } else {
                composed_paras.iter().map(|c| c.lines.len()).collect()
            };
            return skip
                .iter()
                .zip(keep.iter())
                .map(|(&s, &e)| (s, e.max(s)))
                .collect();
        }

        let mut result = Vec::with_capacity(composed_paras.len());
        let mut cum: f64 = 0.0;
        // [Task #431] content_limit 은 현재 페이지에서 표시할 상대 길이(px) 의미이므로
        // 절대 좌표(cum 기반)와 비교하려면 content_offset 을 더해 절대 끝 좌표로 변환한다.
        // (Task #362 의 도입 시점에 단위 mismatch 가 있었음 — content_offset >= content_limit
        // 케이스에서 셀 내 문단이 즉시 break 되어 빈 페이지로 출력되던 결함 정정.)
        // [Task #656] abs_limit 그대로 사용 (epsilon 제거).
        // - Task #485 의 SPLIT_LIMIT_EPSILON = 2.0px 휴리스틱 마진은 typeset/layout 의
        //   trail_ls 비교 모델 어긋남을 흡수하던 임시방편이었음.
        // - 본질 정정: break 비교 시 마지막 visible 줄의 trail_ls 제외 (line_break_pos = cum + h).
        //   typeset 의 split_end_limit = avail_content 추정과 layout 의 셀 마지막 줄 trail_ls
        //   미렌더 모델 (is_cell_last_line) 과 일관 → epsilon 마진 없이 폰트 무관하게 정합.
        let abs_limit = if has_limit {
            content_offset + content_limit
        } else {
            0.0
        };

        // [Task #485 Bug-1] abs_limit 도달 후 렌더 차단 플래그.
        // 이전엔 inner break 만 빠져나와 다음 단락에서 같은 cum 으로 재평가 → 셀 마지막 단락(line_spacing 제외로 line_h 작아짐)이
        // abs_limit 안에 fit 하여 통과하는 out-of-order 결함 발생. 한 번 도달하면 이후 단락 모두 미렌더로 처리.
        let mut limit_reached = false;

        let total_paras = composed_paras.len();
        // [Task #700] 셀별 가드용 — 셀 첫 paragraph 의 LINE_SEG[0].vpos 가 0 이어야 한컴 정상 인코딩.
        let cell_first_vpos = cell
            .paragraphs
            .first()
            .and_then(|p| p.line_segs.first().map(|s| s.vertical_pos))
            .unwrap_or(-1);

        for (pi, (comp, para)) in composed_paras
            .iter()
            .zip(cell.paragraphs.iter())
            .enumerate()
        {
            // [Task #700] paragraph 진입 시 cum 을 LINE_SEG.vpos 절대값으로 동기화.
            // 한컴은 셀 콘텐츠 위치를 LINE_SEG.vpos 단위로 인코딩 (paragraph 사이 spacing 도 vpos
            // 차분에 흡수). rhwp 의 line_height + line_spacing + spacing_before/after 누적은
            // 한컴 vpos 단위와 ~수십 px 어긋나, split_end content_limit (한컴 vpos 단위) 와 비교 시
            // cut 위치가 어긋나는 회귀 (예: inner-table-01 cell[11] p[17] 까지 cut 해야 하는데
            // p[19] 까지 visible 처리). cum 을 vpos 절대값으로 동기화하여 한컴 정합화.
            //
            // [Task #697] 또한 한컴은 셀 내부 페이지 분할 위치에서 LINE_SEG.vpos 를 0 으로 리셋한
            // 인코딩을 사용 (예: cell[11] p[20] vpos=0). vpos 리셋 검출 시 cum 을 abs_limit 까지
            // 강제 진행시켜 후속 paragraph 들이 limit 초과로 cut.
            //
            // 가드:
            // - cell_first_vpos == 0 — 한컴 정상 인코딩 케이스만 (다른 케이스 회피, 회귀 방지)
            // - target_cum > cum — cum 만 전진 허용 (감소 금지, line metric 가 vpos 보다 큰 paragraph
            //   영향 차단)
            // - 차분 누적 (delta) 대신 절대 동기화 — paragraph 사이 spacing mismatch 누적으로 인한
            //   회귀 (form-002 등) 회피.
            if pi > 0 && cell_first_vpos == 0 {
                let prev_para = &cell.paragraphs[pi - 1];
                let prev_end_vpos = prev_para
                    .line_segs
                    .last()
                    .map(|s| s.vertical_pos.saturating_add(s.line_height))
                    .unwrap_or(-1);
                let cur_first_vpos = para.line_segs.first().map(|s| s.vertical_pos).unwrap_or(-1);
                if cur_first_vpos >= 0 && prev_end_vpos > 0 {
                    if cur_first_vpos < prev_end_vpos {
                        // vpos 리셋 — page-break 신호
                        if has_limit && cum < abs_limit {
                            cum = abs_limit;
                        }
                    } else {
                        // 정상 누적 — cum 을 vpos 절대값으로 동기화 (전진만)
                        let target_cum = hwpunit_to_px(cur_first_vpos, self.dpi);
                        if target_cum > cum {
                            cum = target_cum;
                        }
                    }
                }
            }

            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let is_last_para = pi + 1 == total_paras;
            // MeasuredCell 규칙: 첫 문단은 spacing_before 없음, 마지막 문단은 spacing_after 없음
            let raw_spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
            let spacing_before = if pi > 0 {
                raw_spacing_before
            } else if raw_spacing_before > 0.0 {
                let first_vpos = para
                    .line_segs
                    .first()
                    .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi))
                    .unwrap_or(0.0)
                    .max(0.0);
                raw_spacing_before.min(first_vpos)
            } else {
                0.0
            };
            let spacing_after = if !is_last_para {
                para_style.map(|s| s.spacing_after).unwrap_or(0.0)
            } else {
                0.0
            };
            let line_count = comp.lines.len();

            // [Task #485 Bug-1] 한도 초과 후 후속 단락은 강제 미렌더 (시각 순서 보존).
            if limit_reached {
                let visible_count = if line_count == 0 { 0 } else { line_count };
                result.push((visible_count, visible_count));
                continue;
            }

            // 중첩 표 포함 문단(atomic) — line_count==0 또는 has_table_in_para
            let has_table_in_para = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
            if line_count == 0 || has_table_in_para {
                let nested_h: f64 = para
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                let para_h = if line_count == 0 {
                    let h = if nested_h > 0.0 {
                        nested_h
                    } else {
                        hwpunit_to_px(400, self.dpi)
                    };
                    spacing_before + h + spacing_after
                } else {
                    let line_based_h: f64 = comp
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(li, line)| {
                            let h = hwpunit_to_px(line.line_height, self.dpi);
                            let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                            let is_cell_last_line = is_last_para && li + 1 == line_count;
                            let mut lh = if !is_cell_last_line { h + ls } else { h };
                            if li == 0 {
                                lh += spacing_before;
                            }
                            if li == line_count - 1 {
                                lh += spacing_after;
                            }
                            lh
                        })
                        .sum();
                    nested_h.max(line_based_h)
                };

                let para_start_pos = cum;
                let para_end_pos = cum + para_h;
                cum = para_end_pos;

                // 가시성 결정: atomic — 한쪽 페이지에만 렌더링.
                // - content_offset 영역 안에 끝나면(이전 페이지 전체 포함됨) → 스킵
                // - content_limit 영역을 끝점이 초과하면 → 다음 페이지로 미룸
                // - offset 경계를 걸치면 현재 페이지(continuation)에서 렌더링
                //
                // [Task #362] 한 페이지보다 큰 nested table 예외:
                // para_h 가 content_limit 자체를 초과하는 경우 (한 페이지에 어떻게 해도 못 들어감)
                // atomic 미루기 대신 visible 로 표시 (다음 페이지 PartialTable continuation 으로 분할).
                // v0.7.3 의 처리 시멘틱과 동일.
                let was_on_prev = has_offset && para_end_pos <= content_offset;
                let bigger_than_page = has_limit && para_h > content_limit;
                // [Task #431] abs_limit (= content_offset + content_limit) 와 비교 (단위 정합)
                // [Task #656] epsilon 제거 — atomic 단락은 단일 단위로 visible/skip 결정
                let exceeds_limit = has_limit && para_end_pos > abs_limit && !bigger_than_page;
                let visible_count = if line_count == 0 { 0 } else { line_count };
                if was_on_prev || exceeds_limit {
                    // (n,n): 렌더 스킵 마커. line_count==0 이면 (0,0) 동일.
                    result.push((visible_count, visible_count));
                    // [Task #485 Bug-1] limit 초과 단락 발생 시 후속 단락 차단.
                    if exceeds_limit {
                        limit_reached = true;
                    }
                } else {
                    result.push((0, visible_count));
                }
                let _ = para_start_pos; // 추적 변수 (미사용 경고 회피)
                continue;
            }

            // 일반 문단: line 단위 누적 + 위치 기반 가시성
            let mut para_start = 0;
            let mut para_end = 0;
            let mut started = false;

            for (li, line) in comp.lines.iter().enumerate() {
                let h = hwpunit_to_px(line.line_height, self.dpi);
                let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                let is_cell_last_line = is_last_para && li + 1 == line_count;
                let mut line_h = if !is_cell_last_line { h + ls } else { h };
                if li == 0 {
                    line_h += spacing_before;
                }
                if li == line_count - 1 {
                    line_h += spacing_after;
                }

                let line_end_pos = cum + line_h;

                if has_offset && line_end_pos <= content_offset {
                    // 이전 페이지에서 완전히 렌더링됨 → 스킵
                    cum = line_end_pos;
                    para_start = li + 1;
                    para_end = li + 1;
                    continue;
                }

                // [Task #656] break 비교 시 마지막 visible 줄의 trail_ls 제외.
                // - cum 누적은 line_h (h+ls) 그대로 (이전 줄들의 ls 는 다음 줄 직전 spacing 이므로 렌더)
                // - break 비교는 line_break_pos = cum + h (이 줄의 ls 제외) 로 비교
                //   → 이 줄이 visible 시 마지막 줄이면 trail_ls 미렌더 영역, abs_limit 안에 들어감
                // typeset 의 split_end_limit = avail_content 추정과 정합. 셀
                // is_cell_last_line 분기의 trail_ls 미렌더 모델과 동일 본질.
                // (Task #485 의 epsilon 휴리스틱 본질 정정 — 휴리스틱 마진 없이 일관된 모델, 폰트 무관.)
                let line_break_pos = cum + h;
                if has_limit && line_break_pos > abs_limit {
                    // [Task #485 Bug-1] outer 루프도 차단 — 후속 단락의 작은 line_h slip 방지.
                    limit_reached = true;
                    break;
                }

                cum = line_end_pos;
                if !started {
                    started = true;
                    // para_start 는 첫 가시 줄의 인덱스에 고정됨 (위 루프에서 갱신됨)
                }
                para_end = li + 1;
            }

            if !started {
                // 한 줄도 렌더링 안 됨: 모두 offset 영역에 있거나 limit 초과
                // → 누적은 이미 라인별로 처리됨
            }

            result.push((para_start, para_end));
        }

        result
    }

    /// [Task #991] 셀 콘텐츠를 누적하며 예산 `budget_px` 안에 들어가는 문단별 prefix
    /// 줄 수를 반환한다.
    ///
    /// 끝 페이지 패스(`compute_cell_line_ranges` 를 `offset=0, limit=budget` 로 호출)의
    /// 결과에서 추출한다. `offset=0` 이므로 재귀 호출은 `has_offset=false` 경로(끝 페이지
    /// 로직)를 타며 더 이상 재귀하지 않는다.
    ///
    /// 끝 페이지 결과 `(s, e)`:
    /// - `s == 0`: `e` 가 budget 안에 들어간 prefix 가시 줄 수.
    /// - `s != 0`: 한도 초과 스킵 마커 → prefix 0줄.
    fn cell_line_prefix_counts(
        &self,
        cell: &crate::model::table::Cell,
        composed_paras: &[ComposedParagraph],
        budget_px: f64,
        styles: &ResolvedStyleSet,
    ) -> Vec<usize> {
        let ranges = self.compute_cell_line_ranges(cell, composed_paras, 0.0, budget_px, styles);
        ranges
            .iter()
            .map(|&(s, e)| if s == 0 { e } else { 0 })
            .collect()
    }

    /// [Task #993] 한 셀의 콘텐츠를 "유닛" 시퀀스로 평탄화한다.
    ///
    /// 유닛 1개 = 합성 줄 1개 또는 중첩 표 atom 1개(중첩 표 문단 = 유닛 1개,
    /// 분할 불가). 유닛 높이는 `compute_cell_line_ranges`/`calc_visible_content_*`
    /// 의 줄 높이 계산과 동일 규칙(줄 h+ls, 셀 마지막 줄 ls 제외, 문단 첫·마지막
    /// 줄에 spacing_before/after). `hard_break_before` = 이 유닛 앞에 HWP vpos
    /// 리셋(셀 내부 페이지 분할, `[Task #697]`)이 있는가.
    fn nested_table_mixed_fragment_heights(
        &self,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> Vec<NestedFlowFragment> {
        if table.row_count != 1 {
            return Vec::new();
        }

        // [#4069] 단일 셀 중첩 표는 별도 높이 추정식을 다시 만들지 않고 그 셀의
        // canonical CellUnit 원장을 재사용한다. 저장 페이지 프레임 경계를 가진
        // 장문 흐름에 한정해 깊이와 무관하게 재귀하며, placeholder line_height와
        // nested_h를 동시에 더하던 기존 이중 회계를 제거한다. 단일 경계 문서는
        // #2279의 검증된 legacy 측정 원장을 유지한다.
        let mut row_cells = table
            .cells
            .iter()
            .filter(|cell| cell.row == 0 && cell.row_span == 1);
        if let (Some(cell), None) = (row_cells.next(), row_cells.next()) {
            let units = self.cell_units(cell, table, styles);
            let stored_page_frame_boundaries =
                units.iter().filter(|unit| unit.hard_break_before).count();
            let has_authoritative_frame_boundary =
                units.iter().any(|unit| unit.stored_frame_break_before);
            // [#3820 Stage 50] 페이지 하단에서 시작한 1×1 자식 표는 첫 저장
            // fragment가 짧아 문단 사이 vpos reset 직전 좌표가 body 절반에 못
            // 미칠 수 있다(59043 p35: 7540HU→0). 하지만 표 자체의 물리 높이가
            // 한 페이지를 넘고 저장 reset이 정확히 하나라면 이 경계는 로컬
            // 재시작이 아니라 다음 쪽 source cursor다. legacy mixed fallback은
            // 모든 hard break를 지우므로 이 경우에만 canonical CellUnit을 투영한다.
            // 한 페이지 이하 단일 reset과 다중 reset의 기존 판정은 유지한다.
            let body_height = self.current_body_area.get().3;
            let page_height = if body_height > 0.0 {
                body_height
            } else {
                900.0
            };
            let nested_table_height = self.calc_nested_table_height(table, styles);
            let preserve_single_multi_page_boundary = stored_page_frame_boundaries == 1
                // HWP5-origin HWPX는 변환 과정에서 단일 로컬 reset을 남길 수
                // 있으므로 #3637의 31쪽 계약처럼 기존 HWPX 원장을 유지한다.
                // 이 예외는 원본 HWP5 바이너리의 저장 좌표 계약에만 적용한다.
                && self.profile.get().native_hwp5_layout()
                && nested_table_height > page_height + 0.5;
            // direct HWPX도 물리 한 쪽을 넘는 1×1 표에 저장 reset이 하나 있으면
            // canonical CellUnit의 높이/세분성이 필요하다(#3637: 1740.6px,
            // body 971.3px). 다만 reset은 HWP5 source cursor로 승격하지 않고 아래
            // scalar projection에서 제거한다. reset이 전혀 없는 issue1891의 깊은
            // wrapper까지 이 조건에 포함하면 마지막 쪽 overflow가 34→56으로 는다.
            let direct_hwpx_single_multi_page_projection = stored_page_frame_boundaries == 1
                && self.profile.get().hwpx_stored_layout()
                && !self.profile.get().hwp5_origin_hwpx()
                && nested_table_height > page_height + 0.5;
            // canonical CellUnit의 hard-break 원장은 HWP5 저장 좌표 계약이다.
            // direct HWPX의 셀 lineSeg reset은 중첩 셀 로컬 viewport 재시작일 수
            // 있으므로, reset 수가 둘 이상이어도 이 경로로 승격하지 않는다.
            // 그렇지 않으면 #3637 pi=197의 마지막 RowBreak 조각이 한 조각 늘어
            // 후속 표 전체가 불필요한 32쪽으로 밀린다. HWP5-origin HWPX는 원본
            // HWP5의 pagination marker를 보존하므로 canonical 경로를 유지한다.
            let canonical_stored_frame_profile =
                self.profile.get().native_hwp5_layout() || self.profile.get().hwp5_origin_hwpx();
            if let Ok(pattern) = std::env::var("RHWP_DIAG_MIXFRAG") {
                if cell
                    .paragraphs
                    .iter()
                    .any(|paragraph| paragraph.text.contains(&pattern))
                {
                    let nested_controls = cell
                        .paragraphs
                        .iter()
                        .flat_map(|paragraph| paragraph.controls.iter())
                        .filter(|control| matches!(control, Control::Table(_)))
                        .count();
                    eprintln!(
                        "DIAG_MIXFRAG_PROFILE paras={} units={} resets={} authoritative={} nested_ctrls={} table_h={:.1} body_h={:.1}",
                        cell.paragraphs.len(),
                        units.len(),
                        stored_page_frame_boundaries,
                        has_authoritative_frame_boundary,
                        nested_controls,
                        nested_table_height,
                        page_height,
                    );
                    for (unit_idx, unit) in units.iter().enumerate() {
                        eprintln!(
                            "  unit[{unit_idx}] h={:.2} para={} lines={}..{} mixed={} trailing={} content_h={:.2} hard={} stored={} spacer={}",
                            unit.height,
                            unit.para_idx,
                            unit.vis_start,
                            unit.vis_end,
                            unit.mixed_nested_fragment,
                            unit.mixed_nested_trailing,
                            unit.mixed_nested_content_height,
                            unit.hard_break_before,
                            unit.stored_frame_break_before,
                            unit.empty_spacer,
                        );
                    }
                }
            }
            // [#4069 Stage 2/Task #3820 Stage 48] 저장 프레임 경계는 문단 내부인지
            // 문단 사이인지와 무관하게 하나라도 부모 원장에 보존한다. 작은 로컬 reset은
            // `is_stored_frame_rewind`의 저장 frame 판정에서 이미 제외된다.
            // 42065 p10은 같은 문단 58620→0, p14는 item7→item8의 문단간
            // 32932→0 경계이며 둘 다 한컴 정본의 실제 쪽 경계다.
            if canonical_stored_frame_profile
                && (stored_page_frame_boundaries >= 2
                    || has_authoritative_frame_boundary
                    || preserve_single_multi_page_boundary)
            {
                return units
                    .iter()
                    .map(|unit| {
                        let visible = Self::cell_unit_has_visible_content(cell, unit);
                        NestedFlowFragment {
                            height: unit.height,
                            hard_break_before: unit.hard_break_before,
                            // 물리적으로 여러 쪽을 차지하는 1×1 자식 표의 유일한
                            // 저장 reset은 부모 viewport에서 실제 쪽 경계다. 자식의
                            // 일반 CellUnit 의미는 바꾸지 않고, 부모로 투영되는
                            // fragment에만 authoritative 표식을 부여해 RowBreak의
                            // 중간-reset 완화가 첫 source 줄을 앞쪽에 흡수하지 않게 한다.
                            stored_frame_break_before: unit.stored_frame_break_before
                                || (preserve_single_multi_page_boundary && unit.hard_break_before),
                            trailing: unit.mixed_nested_trailing || !visible,
                            content_height: if unit.mixed_nested_content_height > 0.0 {
                                unit.mixed_nested_content_height
                            } else if visible {
                                unit.height
                            } else {
                                0.0
                            },
                            recursive: true,
                            starts_after_table: unit.mixed_nested_starts_after_table,
                            source_para_idx: Some(unit.para_idx),
                            recursive_block_prelude_role: unit.recursive_block_prelude_role,
                        }
                    })
                    .collect();
            }
            if self.profile.get().hwpx_stored_layout()
                && !self.profile.get().hwp5_origin_hwpx()
                && (stored_page_frame_boundaries >= 2
                    || has_authoritative_frame_boundary
                    || direct_hwpx_single_multi_page_projection)
            {
                // PR #4122 이전 direct-HWPX fallback은 빈 host의 자식 표를
                // 재귀적으로 평탄화하고, 같은 문단의 placeholder line과 표 높이를
                // 한 번만 회계했다. 단순 legacy 재측정은 그 세 동작을 잃어
                // #3637이 30쪽으로 과소 조판되거나 p26 source owner를 잃는다.
                // 현재 canonical CellUnit은 그 재귀 원장을 이미 보유하므로, 높이와
                // 가시 단위만 재사용하되 HWP5 전용 hard/stored cursor 의미를 제거해
                // 검증된 HWPX scalar viewport 계약으로 투영한다. reset이 없는 일반
                // 중첩 표까지 이 경로로 바꾸면 issue1891의 깊은 표가 마지막 쪽에서
                // 22줄 더 밀리므로, canonical 승격 대상이었던 반복/authoritative
                // reset 표에만 이 변환을 적용한다. reset 0개이거나 물리 한 쪽에
                // 못 미치는 단일 reset 표는 legacy fallback을 유지한다.
                return units
                    .iter()
                    .map(|unit| {
                        let visible = Self::cell_unit_has_visible_content(cell, unit);
                        NestedFlowFragment {
                            height: unit.height,
                            hard_break_before: false,
                            stored_frame_break_before: false,
                            trailing: unit.mixed_nested_trailing || !visible,
                            content_height: if unit.mixed_nested_content_height > 0.0 {
                                unit.mixed_nested_content_height
                            } else if visible {
                                unit.height
                            } else {
                                0.0
                            },
                            recursive: false,
                            starts_after_table: unit.mixed_nested_starts_after_table,
                            source_para_idx: Some(unit.para_idx),
                            recursive_block_prelude_role: RecursiveBlockPreludeRole::None,
                        }
                    })
                    .collect();
            }
        }

        let mut row_units: Vec<(f64, bool, f64, bool, Option<usize>)> = Vec::new();
        for cell in table.cells.iter().filter(|cell| cell.row == 0) {
            // 이 helper는 부모 셀의 Control::Table, 즉 중첩 표의 조각 유닛을
            // 산출한다. 비글자 중첩 표는 실제 배치에서 보존된 작은 cellMargin을
            // 사용하므로, 여기에서도 같은 폭으로 조판해야 한다. 기본 padding
            // 규칙(aim=false → table inMargin)으로 재조판하면 더 넓은 폭에서 줄이
            // 하나 덜 생겨, 유닛 컷은 마지막 줄을 포함했다고 판단해도 실제 SVG
            // glyph가 다음 페이지 clip에 걸린다 (76076 p33→34).
            let preserve_saved_small_margin = !table.common.treat_as_char
                && !self
                    .render_normalization_overlay()
                    .uses_owner_content_box(table);
            let (pad_left, pad_right, pad_top, pad_bottom) =
                self.resolve_cell_padding_for_context(cell, table, preserve_saved_small_margin);
            let cell_w = if cell.width < 0x8000_0000 {
                hwpunit_to_px(cell.width as i32, self.dpi) * self.render_table_width_scale(table)
            } else {
                0.0
            };
            let inner_width = (cell_w - pad_left - pad_right).max(0.0);
            let mut cell_units = Vec::new();
            let mut after_completed_multiline_table = false;
            for (pi, para) in cell.paragraphs.iter().enumerate() {
                let para_is_empty_spacer = para.text.trim().is_empty() && para.controls.is_empty();
                let starts_after_completed_multiline_table =
                    after_completed_multiline_table && !para_is_empty_spacer;
                let mut comp = compose_paragraph(para);
                crate::renderer::composer::recompose_for_cell_width(
                    &mut comp,
                    para,
                    inner_width,
                    styles,
                );
                // [#2279 axis A] 종전에는 comp.lines 빈 문단을 통째 skip 해 (a) 2단계
                // 중첩 표(빈 문단 소속)와 (b) 빈 문단 줄박스가 유닛에서 누락됐다 —
                // 86712 pi=172 r27 근거설명(25문단 + 3×12 + 5×4 내부표) 프래그먼트 합
                // 933px vs mt·한글 ~1402px 의 -448 주성분. 중첩 표는
                // calc_nested_table_height(행합+cs+outer margin, 측정 단일 출처),
                // 빈 문단은 #2169 em 줄박스 규칙으로 유닛화한다.
                let nested_h: f64 = para
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                let empty_line_box = if comp.lines.is_empty()
                    && nested_h <= 0.0
                    && para.line_segs.is_empty()
                    && para.controls.is_empty()
                    && para.text.trim().is_empty()
                {
                    let fs = para
                        .char_shapes
                        .first()
                        .and_then(|cs| styles.char_styles.get(cs.char_shape_id as usize))
                        .map(|cs| cs.font_size)
                        .unwrap_or(0.0);
                    if fs > 0.0 {
                        fs
                    } else {
                        hwpunit_to_px(400, self.dpi)
                    }
                } else {
                    0.0
                };
                if comp.lines.is_empty() && nested_h <= 0.5 && empty_line_box <= 0.5 {
                    continue;
                }

                let para_style = styles.para_styles.get(para.para_shape_id as usize);
                if pi == 0 && pad_top > 0.5 {
                    cell_units.push((pad_top, false, 0.0, false, None));
                }
                if pi > 0 {
                    let spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
                    if spacing_before > 0.5 {
                        cell_units.push((spacing_before, false, 0.0, false, None));
                    }
                }
                for (li, line) in comp.lines.iter().enumerate() {
                    let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                    let corrected_h = match para_style {
                        Some(ps) => {
                            let max_fs = line
                                .runs
                                .iter()
                                .map(|r| {
                                    let ts = super::text_measurement::resolved_to_text_style(
                                        styles,
                                        r.char_style_id,
                                        r.lang_index,
                                    );
                                    if ts.font_size > 0.0 {
                                        ts.font_size
                                    } else {
                                        12.0
                                    }
                                })
                                .fold(0.0f64, f64::max);
                            crate::renderer::corrected_line_height_for_variant_synthetic(
                                raw_lh,
                                max_fs,
                                ps.line_spacing_type,
                                ps.line_spacing,
                                self.profile.get().hwp3_layout()
                                    && para.line_segs.is_empty()
                                    && !para.text.is_empty(),
                            )
                        }
                        None => raw_lh,
                    };
                    // [#2279 axis A] 문단 말미 줄간격은 셀의 마지막 문단에서만 탈락 —
                    // mt(calc_para_lines_height / #2211 include_trailing_ls)와 정합.
                    // 종전 per-문단 탈락은 25문단 셀에서 -83px 과소(86712 r27).
                    let is_cell_last_para = pi + 1 == cell.paragraphs.len();
                    let line_spacing = if li + 1 == comp.lines.len() && is_cell_last_para {
                        0.0
                    } else {
                        hwpunit_to_px(line.line_spacing, self.dpi)
                    };
                    cell_units.push((
                        corrected_h + line_spacing,
                        false,
                        corrected_h,
                        starts_after_completed_multiline_table && li == 0,
                        Some(pi),
                    ));
                }
                if nested_h > 0.5 {
                    cell_units.push((
                        nested_h,
                        false,
                        nested_h,
                        starts_after_completed_multiline_table,
                        Some(pi),
                    ));
                }
                if empty_line_box > 0.5 {
                    cell_units.push((
                        empty_line_box,
                        false,
                        empty_line_box,
                        starts_after_completed_multiline_table,
                        Some(pi),
                    ));
                }
                if pi + 1 < cell.paragraphs.len() {
                    let spacing_after = para_style.map(|s| s.spacing_after).unwrap_or(0.0);
                    if spacing_after > 0.5 {
                        cell_units.push((spacing_after, true, 0.0, false, None));
                    }
                }
                let completed_multiline_table = para
                    .controls
                    .iter()
                    .any(|control| matches!(control, Control::Table(table) if table.row_count > 1));
                if completed_multiline_table {
                    after_completed_multiline_table = true;
                } else if !para_is_empty_spacer {
                    after_completed_multiline_table = false;
                }
            }
            if pad_bottom > 0.5 {
                cell_units.push((pad_bottom, true, 0.0, false, None));
            }
            // [#2279 진단] 1×1 중첩 셀 프래그먼트 분해 — 동작 불변.
            if let Ok(pat) = std::env::var("RHWP_DIAG_MIXFRAG") {
                if cell.paragraphs.iter().any(|p| p.text.contains(&pat)) {
                    let total: f64 = cell_units.iter().map(|(h, _, _, _, _)| *h).sum();
                    eprintln!(
                        "DIAG_MIXFRAG cell paras={} units={} total={:.1} inner_w={:.2}",
                        cell.paragraphs.len(),
                        cell_units.len(),
                        total,
                        inner_width,
                    );
                    for (pi, para) in cell.paragraphs.iter().enumerate() {
                        let mut comp = compose_paragraph(para);
                        crate::renderer::composer::recompose_for_cell_width(
                            &mut comp,
                            para,
                            inner_width,
                            styles,
                        );
                        let nctl = para.controls.len();
                        eprintln!(
                            "  p[{pi}] lines={} text_len={} ctrls={} ls_stored={} text={:?}",
                            comp.lines.len(),
                            para.text.chars().count(),
                            nctl,
                            para.line_segs.len(),
                            para.text.chars().take(16).collect::<String>(),
                        );
                    }
                }
            }
            if cell_units.len() > row_units.len() {
                row_units.resize(cell_units.len(), (0.0, true, 0.0, false, None));
            }
            for (idx, (h, trailing, content_h, starts_after_table, source_para_idx)) in
                cell_units.into_iter().enumerate()
            {
                if h > row_units[idx].0 {
                    row_units[idx] = (h, trailing, content_h, starts_after_table, source_para_idx);
                } else if (h - row_units[idx].0).abs() <= 0.5 {
                    row_units[idx].1 = row_units[idx].1 && trailing;
                    row_units[idx].2 = row_units[idx].2.max(content_h);
                    row_units[idx].3 = row_units[idx].3 || starts_after_table;
                    if row_units[idx].4 != source_para_idx {
                        row_units[idx].4 = None;
                    }
                }
            }
        }
        row_units
            .into_iter()
            .map(
                |(height, trailing, content_height, starts_after_table, source_para_idx)| {
                    NestedFlowFragment {
                        height,
                        hard_break_before: false,
                        stored_frame_break_before: false,
                        trailing,
                        content_height,
                        recursive: false,
                        starts_after_table,
                        source_para_idx,
                        recursive_block_prelude_role: RecursiveBlockPreludeRole::None,
                    }
                },
            )
            .collect()
    }

    /// [Issue #2214] 표 단위 nested-text flag에 대한 문단 로컬 기여 여부.
    /// 편집 경로와 table-wide 계산이 같은 predicate를 사용하도록 단일화한다.
    pub(crate) fn paragraph_contributes_to_table_nested_text_flag(paragraph: &Paragraph) -> bool {
        !paragraph.text.trim().is_empty()
            && paragraph
                .controls
                .iter()
                .any(|control| matches!(control, Control::Table(_)))
    }

    /// 문단이 정확히 하나의 1×1 자식 표를 host하는가.
    ///
    /// 저장 프레임 끝에서 이 형태의 표를 fragment로 푼 뒤 다음 문단 reset을
    /// 엄격히 보존할 때만 사용한다. 일반 문단 사이 reset의 orphan 완화와 분리한다.
    fn paragraph_hosts_single_cell_nested_table(paragraph: &Paragraph) -> bool {
        let mut tables = paragraph
            .controls
            .iter()
            .filter_map(|control| match control {
                Control::Table(table) => Some(table.as_ref()),
                _ => None,
            });
        matches!(
            (tables.next(), tables.next()),
            (Some(table), None) if table.row_count == 1 && table.col_count == 1
        )
    }

    /// [Issue #2063] 표에 "가시 텍스트 + 중첩 표"를 가진 셀이 하나라도 있는지 직접 계산한다.
    /// predicate table scan과 test counter는 이 helper에만 둔다.
    fn compute_table_nested_text_flag(&self, table: &crate::model::table::Table) -> bool {
        #[cfg(test)]
        self.table_nested_text_flag_scan_count
            .set(self.table_nested_text_flag_scan_count.get() + 1);
        table.cells.iter().any(|cell| {
            cell.paragraphs
                .iter()
                .any(Self::paragraph_contributes_to_table_nested_text_flag)
        })
    }

    /// [Issue #2063] 표에 "가시 텍스트 + 중첩 표"를 가진 셀이 하나라도 있는지(표 단위 불변량).
    /// `cell_units_uncached` 안에서 셀마다 계산되면 O(셀²)(52,694² ≈ 28억)로 폭증하므로
    /// 표 포인터를 키로 1회만 계산해 캐시한다(`cell_units_cache` 와 동일 조판 경계에서 clear).
    fn table_has_visible_text_with_nested_table(&self, table: &crate::model::table::Table) -> bool {
        let key = table as *const crate::model::table::Table as usize;
        if let Some(&cached) = self.table_nested_text_flag_cache.borrow().get(&key) {
            return cached;
        }
        let flag = self.compute_table_nested_text_flag(table);
        self.table_nested_text_flag_cache
            .borrow_mut()
            .insert(key, flag);
        flag
    }

    /// [#4167] 문단의 cell-units 기여 지문 — 편집이 units 를 실제로 바꿨는지 판별용.
    ///
    /// units 산출이 읽는 문단 입력만 포함한다: line_segs 의 (vertical_pos, line_height,
    /// tag) 수열(개수 포함), controls 수, 공백/빈 문단 클래스. `text_start` 와
    /// `segment_width` 는 제자리 타이핑에도 매 키 변하지만 units 산출이 읽지 않으므로
    /// 제외한다(위 `cell_units_uncached` 전 구간 grep 근거). 인접 문단 결합(직전 끝
    /// seg·직후 첫 seg 참조)도 이 지문에 담긴 경계 seg 로 판별된다 — 지문 불변이면
    /// 이웃 기여도 불변이다.
    pub(crate) fn cell_paragraph_units_fingerprint(
        para: &crate::model::paragraph::Paragraph,
    ) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        para.line_segs.len().hash(&mut h);
        for seg in &para.line_segs {
            seg.vertical_pos.hash(&mut h);
            seg.line_height.hash(&mut h);
            // units 산출이 tag 에서 읽는 비트는 synthetic 판별(bit 31)뿐이다. 원본
            // 로드 tag 의 여타 비트(예: 0x100000)는 reflow 가 재방출하지 않아 첫
            // 편집에서 무의미하게 지문을 바꾸므로 마스킹한다.
            (seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY).hash(&mut h);
        }
        para.controls.len().hash(&mut h);
        para.text.is_empty().hash(&mut h);
        para.text.trim().is_empty().hash(&mut h);
        h.finish()
    }

    /// [Issue #2214/#2424] 텍스트 편집 뒤 edited cell의 memoized units를 국소 무효화한다.
    ///
    /// cached owner flag가 false인데 edited paragraph가 false→true가 된 경우에만
    /// owner의 직접 cell units를 모두 제거하고, local witness로 flag를 true로 갱신한다.
    /// 삭제로 true→false가 되면 다른 cell의 contribution 여부를 알 수 없으므로 owner의
    /// 직접 cell units와 flag를 제거해 다음 접근에서 한 번만 다시 계산한다.
    /// 이 direct-key 제거는 predicate 재스캔이 아니며 nested/unrelated table cache는 보존한다.
    pub(crate) fn invalidate_cell_units_after_text_edit(
        &self,
        edited_cell: &crate::model::table::Cell,
        owner_table: &crate::model::table::Table,
        local_before: bool,
        local_after: bool,
        unit_fingerprint_unchanged: bool,
    ) {
        let edited_cell_key = edited_cell as *const crate::model::table::Cell as usize;
        let owner_table_key = owner_table as *const crate::model::table::Table as usize;
        let cached_owner_flag = self
            .table_nested_text_flag_cache
            .borrow()
            .get(&owner_table_key)
            .copied();
        let local_became_true = !local_before && local_after;
        let local_became_false = local_before && !local_after;

        if local_became_false {
            let mut cell_cache = self.cell_units_cache.borrow_mut();
            for cell in &owner_table.cells {
                let key = cell as *const crate::model::table::Cell as usize;
                cell_cache.remove(&key);
            }
            drop(cell_cache);
            self.table_nested_text_flag_cache
                .borrow_mut()
                .remove(&owner_table_key);
            return;
        }

        if local_became_true && cached_owner_flag == Some(false) {
            let mut cell_cache = self.cell_units_cache.borrow_mut();
            for cell in &owner_table.cells {
                let key = cell as *const crate::model::table::Cell as usize;
                cell_cache.remove(&key);
            }
            drop(cell_cache);
            self.table_nested_text_flag_cache
                .borrow_mut()
                .insert(owner_table_key, true);
            return;
        }

        // [#4167] 편집 문단의 units 기여 지문이 불변이면(제자리 타이핑 — 줄 수·높이
        // 불변) 캐시된 units 벡터 전체가 그대로 유효하다 — 거대 셀(수천 문단)의
        // 전량 recompose(11ms/키)를 생략한다. 지문이 다르면 종전대로 셀 단위 제거.
        if unit_fingerprint_unchanged {
            return;
        }
        self.cell_units_cache.borrow_mut().remove(&edited_cell_key);
        if local_became_true && cached_owner_flag.is_none() {
            // cell_units entry가 있으면 owner flag도 먼저 warm된다는 현재 cache invariant에
            // 따라 owner-wide eviction은 불필요하다. local witness로 future scan도 피한다.
            self.table_nested_text_flag_cache
                .borrow_mut()
                .insert(owner_table_key, true);
        }
    }

    /// [Task #1949] `cell_units_uncached` 의 메모이즈 래퍼. 거대 셀이 RowBreak 로
    /// 여러 페이지에 걸칠 때 각 페이지 컷 판정이 같은 셀 units 를 재계산하는 O(pages×cell)
    /// 폭증을 제거한다. 셀 포인터를 키로 표 단위 캐시(문서 재조판 경계에서 clear).
    pub(super) fn cell_units(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> std::sync::Arc<Vec<CellUnit>> {
        let key = cell as *const crate::model::table::Cell as usize;
        if let Some(cached) = self.cell_units_cache.borrow().get(&key) {
            if issue2424_profile_enabled() {
                ISSUE2424_CELL_UNITS_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            return std::sync::Arc::clone(cached);
        }
        let issue2424_started = issue2424_profile_enabled().then(std::time::Instant::now);
        let units = std::sync::Arc::new(self.cell_units_uncached(cell, table, styles));
        if let Some(started) = issue2424_started {
            use std::sync::atomic::Ordering::Relaxed;
            ISSUE2424_CELL_UNITS_MISSES.fetch_add(1, Relaxed);
            ISSUE2424_CELL_UNITS_MISS_NANOS.fetch_add(started.elapsed().as_nanos() as u64, Relaxed);
        }
        self.cell_units_cache
            .borrow_mut()
            .insert(key, std::sync::Arc::clone(&units));
        units
    }

    /// [#4128] `(cell_para_idx, target_line)` 이 속한 `cell_units` 서수.
    /// 텍스트 줄 유닛 `(li, li+1)` / atom 유닛 `(0, line_count.max(1))` 의
    /// `vis_start..vis_end` 계약을 그대로 조회한다. 콘텐츠(비 spacer) 유닛을
    /// 우선하되, 빈 문단처럼 spacer 유닛만 있는 문단은 spacer 서수로 폴백한다.
    /// 없으면 None.
    pub(super) fn cell_unit_ordinal_for(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        cell_para_idx: usize,
        target_line: usize,
    ) -> Option<usize> {
        let units = self.cell_units(cell, table, styles);
        let hit = |u: &CellUnit| {
            u.para_idx == cell_para_idx
                && u.vis_start <= target_line
                && target_line < u.vis_end.max(u.vis_start + 1)
        };
        units
            .iter()
            .position(|u| !u.empty_spacer && hit(u))
            .or_else(|| units.iter().position(|u| hit(u)))
    }

    /// 빈 native HWP5 RowBreak parent의 마지막 1×1 block child가, parent의 선언
    /// 높이보다 content flow가 커서 페이지 tail에서 child unit을 나눠야 하는 구조인지 판별한다.
    ///
    /// 이 조건은 `cell_units()`가 단일행 child를 mixed fragment로 전개하는 경로와
    /// paginator의 split-eligibility가 반드시 공유해야 한다. 한 쪽만 열면 unit은
    /// 생성돼도 `MeasuredTable`이 row를 atomic으로 보고 `advance_row_cut()`까지
    /// 도달하지 않는다 (76076 p81→82).
    fn native_short_parent_child_fragment_eligible(
        &self,
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
        child: &crate::model::table::Table,
        child_flow_height: f64,
    ) -> bool {
        let parent_declared_height = hwpunit_to_px(table.common.height as i32, self.dpi);
        let eligible = self.profile.get().hwp5_stored_pagination_layout()
            && !table.common.treat_as_char
            && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
            && matches!(table.common.vert_rel_to, VertRelTo::Para)
            && matches!(table.page_break, TablePageBreak::RowBreak)
            && table.row_count > 1
            && cell.row_span == 1
            && cell.row as usize + 1 == table.row_count as usize
            // HWP5 저장기는 block child 뒤에 vpos=0의 빈 reset 문단을 남길 수
            // 있다. 그것은 별 content host가 아니므로 허용하되, field/표/텍스트를
            // 가진 후속 문단이 있으면 일반 원자 경로를 유지한다.
            && cell.paragraphs.first().is_some_and(|host| {
                host.text.trim().is_empty()
                    && host
                        .controls
                        .iter()
                        .filter(|control| matches!(control, Control::Table(_)))
                        .count()
                        == 1
            })
            && cell.paragraphs.iter().skip(1).all(|paragraph| {
                paragraph.text.trim().is_empty()
                    && paragraph.controls.is_empty()
                    && paragraph.line_segs.len() <= 1
            })
            && child.col_count == 1
            && !child.common.treat_as_char
            // p831의 7문단 `산식 설명` child처럼 일반적인 큰 tail은 여기서
            // 분해하지 않는다. 이 경로는 page-tail에 한두 줄만 배치되는 short
            // child의 source owner를 보존하기 위한 것이다.
            && child.cells.len() == 1
            && child.cells[0].paragraphs.len() <= 3
            && parent_declared_height > 0.0
            && child_flow_height > parent_declared_height + 0.5;
        if std::env::var("RHWP_DIAG_SHORT_CHILD").is_ok()
            && child.row_count == 1
            && child.col_count == 1
            && child.cells.len() == 1
        {
            eprintln!(
                "DIAG_SHORT_CHILD eligible={} native={} parent=(rows={},h={:.1},wrap={:?},vert={:?},break={:?}) cell=(row={},span={},paras={}) child=(cols={},tac={},cells={},paras={},flow={:.1})",
                eligible,
                self.profile.get().hwp5_stored_pagination_layout(),
                table.row_count,
                parent_declared_height,
                table.common.text_wrap,
                table.common.vert_rel_to,
                table.page_break,
                cell.row,
                cell.row_span,
                cell.paragraphs.len(),
                child.col_count,
                child.common.treat_as_char,
                child.cells.len(),
                child.cells.first().map(|c| c.paragraphs.len()).unwrap_or(0),
                child_flow_height,
            );
        }
        eligible
    }

    /// Native HWP5 `RowBreak`의 마지막 행에서 이미 outer `CellUnit` cut으로
    /// 분할된 1×1 block child인지 판별한다.
    ///
    /// 짧은 child는 `native_short_parent_child_fragment_eligible`가 paginator의
    /// 분할 가능 여부까지 함께 결정한다. 이 helper는 그보다 좁은 후속 단계다:
    /// mixed split이 이미 child의 source height를 소비한 **terminal** tail에서
    /// 시작 cursor만 전달한다. 따라서 큰 child를 새로 fragment 단위로 승격하거나
    /// HWPCTRL/WASM API 계약을 바꾸지 않는다.
    fn native_terminal_rowbreak_child_source_cursor_eligible(
        &self,
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
        child: &crate::model::table::Table,
    ) -> bool {
        self.profile.get().hwp5_stored_pagination_layout()
            && !table.common.treat_as_char
            && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
            && matches!(table.common.vert_rel_to, VertRelTo::Para)
            && matches!(table.page_break, TablePageBreak::RowBreak)
            && table.row_count > 1
            && cell.row_span == 1
            && cell.row as usize + 1 == table.row_count as usize
            && cell.paragraphs.first().is_some_and(|host| {
                host.text.trim().is_empty()
                    && host
                        .controls
                        .iter()
                        .filter(|control| matches!(control, Control::Table(_)))
                        .count()
                        == 1
            })
            && cell.paragraphs.iter().skip(1).all(|paragraph| {
                paragraph.text.trim().is_empty()
                    && paragraph.controls.is_empty()
                    && paragraph.line_segs.len() <= 1
            })
            && child.row_count == 1
            && child.col_count == 1
            && !child.common.treat_as_char
            && child.cells.len() == 1
    }

    /// `RowBreak` scan에서 short parent의 마지막 child row를 분할할 수 있는지
    /// 반환한다. 구조만 맞아도 child가 실제로 한 unit이면 분할할 것이 없으므로,
    /// non-spacer unit이 둘 이상인 것을 함께 확인한다.
    pub(crate) fn native_short_parent_child_row_is_fragmentable(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .any(|cell| {
                let Some(host) = cell.paragraphs.first() else {
                    return false;
                };
                let children: Vec<&crate::model::table::Table> = host
                    .controls
                    .iter()
                    .filter_map(|control| match control {
                        Control::Table(child) => Some(child.as_ref()),
                        _ => None,
                    })
                    .collect();
                let Some(child) = children.as_slice().first().copied() else {
                    return false;
                };
                let eligible = self.native_short_parent_child_fragment_eligible(
                    table,
                    cell,
                    child,
                    self.nested_table_mixed_fragment_heights(child, styles)
                        .iter()
                        .map(|fragment| fragment.height)
                        .sum::<f64>(),
                );
                if children.len() != 1 || !eligible {
                    return false;
                }
                let units = self.cell_units(cell, table, styles);
                units
                    .iter()
                    .filter(|unit| !unit.empty_spacer)
                    .take(2)
                    .count()
                    == 2
            })
    }

    /// [#4128 테스트 전용] cell_units 요약: (para_idx, vis_start, vis_end,
    /// empty_spacer, nested_row).
    #[cfg(test)]
    pub(crate) fn debug_cell_units(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> Vec<(usize, usize, usize, bool, Option<usize>)> {
        self.cell_units(cell, table, styles)
            .iter()
            .map(|u| {
                (
                    u.para_idx,
                    u.vis_start,
                    u.vis_end,
                    u.empty_spacer,
                    u.nested_row,
                )
            })
            .collect()
    }

    fn cell_units_uncached(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> Vec<CellUnit> {
        let (pad_left, pad_right, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
        let cell_w = if cell.width < 0x8000_0000 {
            hwpunit_to_px(cell.width as i32, self.dpi) * self.render_table_width_scale(table)
        } else {
            0.0
        };
        // [#2279 axis B 보류] 측정에 렌더의 오버플로 패딩 축소 폭을 적용하는 안은
        // 86712 산식 셀(측정 5줄 vs 렌더·한글 4줄)을 정합시키지만, 어드밴스가 사다리
        // 교정된 문서(80168 pi=1056 r7: 한글 PDF 8줄 실측)에서는 한글이 지키는 패딩을
        // 깨 7줄로 과소(157→156 회귀) — shrink 는 폰트 폭 오차의 문서별 보상재로,
        // 일반화 불가(#2279 코멘트). 측정 폭은 원 패딩 유지.
        let inner_width = (cell_w - pad_left - pad_right).max(0.0);
        let line_seg_is_synthetic = |seg: &crate::model::paragraph::LineSeg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        };
        let is_block_rowbreak_table = matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) && !table.common.treat_as_char;
        let direct_hwpx_stored_frame_cell =
            self.direct_hwpx_cell_has_declared_stored_frame(cell, table);
        let is_stored_frame_rewind = |prev: &crate::model::paragraph::LineSeg,
                                      cur: &crate::model::paragraph::LineSeg|
         -> bool {
            let profile = self.profile.get();
            if (!profile.native_hwp5_layout()
                && !profile.hwp5_origin_hwpx()
                && !direct_hwpx_stored_frame_cell)
                || line_seg_is_synthetic(prev)
                || line_seg_is_synthetic(cur)
            {
                return false;
            }
            let prev_end = prev.vertical_pos.saturating_add(prev.line_height);
            if cur.vertical_pos < 0 || prev_end <= 0 || cur.vertical_pos >= prev_end {
                return false;
            }
            if direct_hwpx_stored_frame_cell {
                // 선언 object frame이 없는 synthetic 표의 vpos reset은 물리 쪽
                // 경계를 증명하지 못한다. 실제 저장 표는 선언 높이가 있으므로
                // 기존 direct HWPX frame 계약을 그대로 따른다.
                return table.common.height > 0;
            }
            // HWPX 저장 lineseg의 reset은 중첩 셀 로컬 좌표계일 수 있다
            // (#3637). HWP5 저장 계약 안에서도 작은 내부 표의 로컬 reset
            // (rowbreak-problem-pages.hwp: 9600→0HU)을 페이지 경계로 올리지
            // 않도록 직전 줄이 body 하단 절반에 도달한 경우만 인정한다.
            let body_height = self.current_body_area.get().3;
            let frame_floor = if body_height > 0.0 {
                body_height * 0.5
            } else {
                450.0
            };
            hwpunit_to_px(prev_end, self.dpi) >= frame_floor
        };
        let has_visible_text_with_nested_table =
            self.table_has_visible_text_with_nested_table(table);
        // [Task #700] vpos 동기화 가드와 동일 — 한컴 정상 인코딩(첫 문단 vpos=0) 한정.
        let cell_first_vpos = cell
            .paragraphs
            .first()
            .and_then(|p| p.line_segs.first().map(|s| s.vertical_pos))
            .unwrap_or(-1);
        let cell_has_local_vpos_origin = cell_first_vpos == 0
            || (is_block_rowbreak_table && (0..=500).contains(&cell_first_vpos));
        let preserve_linear_single_cell_vpos = is_block_rowbreak_table
            && table.row_count == 1
            && table.col_count == 1
            && (table.common.vertical_offset as i32) == 0
            && cell_first_vpos >= 0;
        let use_vpos_unit_positions = is_block_rowbreak_table
            && ((table.row_count > 1 && has_visible_text_with_nested_table)
                || preserve_linear_single_cell_vpos);
        let vpos_origin = if preserve_linear_single_cell_vpos {
            cell_first_vpos.max(0)
        } else {
            0
        };
        let normalized_vpos_px = |vertical_pos: i32| -> f64 {
            hwpunit_to_px((vertical_pos - vpos_origin).max(0), self.dpi)
        };
        let para_count = cell.paragraphs.len();
        let cell_has_visible_content = cell
            .paragraphs
            .iter()
            .any(|p| !p.text.trim().is_empty() || !p.controls.is_empty());
        // Native HWP5 RowBreak 표에는 문단 기준 Square/Tight/Through 개체 사이에,
        // 실제 줄간격이 아니라 개체 anchor를 저장한 연속 빈 문단 사다리가 남을 수 있다.
        // 이를 일반 em line으로 누적하면 row cut의 physical footprint가 Hancom보다 커져
        // 다음 개체 owner가 한 page 늦어진다 (59043 p11/p12). 단일 빈 줄은 저자가
        // 의도한 여백일 수 있으므로, 양쪽이 non-inline flow 문단인 2개 이상 run만 대상이다.
        // HWPX/CellBreak/TAC에는 stored-layout 의미가 달라 이 predicate를 적용하지 않는다.
        let native_hwp5_rowbreak_float_ladder =
            self.profile.get().native_hwp5_layout() && is_block_rowbreak_table;
        let plain_empty_paragraph: Vec<bool> = cell
            .paragraphs
            .iter()
            .map(|p| p.text.trim().is_empty() && p.controls.is_empty())
            .collect();
        let other_non_inline_flow_paragraph: Vec<bool> = cell
            .paragraphs
            .iter()
            .map(|p| {
                self.paragraph_cell_other_non_inline_control_heights(&p.controls)
                    .iter()
                    .any(|(_, height)| *height > 0.5)
            })
            .collect();
        let mut units: Vec<CellUnit> = Vec::new();
        let split_non_inline_extra =
            |extra_h: f64, top_and_bottom_h: f64, other_h: f64| -> (f64, f64) {
                if extra_h <= 0.5 {
                    return (0.0, 0.0);
                }
                if top_and_bottom_h <= 0.5 {
                    return (0.0, extra_h);
                }
                if other_h <= 0.5 {
                    return (extra_h, 0.0);
                }
                let total_h = top_and_bottom_h + other_h;
                let top_extra = extra_h * (top_and_bottom_h / total_h);
                (top_extra, extra_h - top_extra)
            };
        let append_fragment_units =
            |units: &mut Vec<CellUnit>, para_idx: usize, mut non_inline_h: f64| {
                const FILLER_UNIT_PX: f64 = 16.0;
                while non_inline_h > 0.5 {
                    let h = non_inline_h.min(FILLER_UNIT_PX);
                    units.push(CellUnit {
                        height: h,
                        hard_break_before: false,
                        stored_frame_break_before: false,
                        vpos_gap_before: false,
                        para_idx,
                        vis_start: 0,
                        vis_end: 0,
                        nested_row: None,
                        nested_table_fragment: None,
                        mixed_nested_fragment: false,
                        mixed_nested_trailing: false,
                        mixed_nested_content_height: 0.0,
                        mixed_nested_recursive: false,
                        mixed_nested_starts_after_table: false,
                        mixed_nested_source_para_idx: None,
                        recursive_block_prelude_role: RecursiveBlockPreludeRole::None,
                        top_and_bottom_flow: false,
                        empty_spacer: false,
                        non_inline_control_range: None,
                    });
                    non_inline_h -= h;
                }
            };
        let append_atomic_unit = |units: &mut Vec<CellUnit>, para_idx: usize, non_inline_h: f64| {
            if non_inline_h <= 0.5 {
                return;
            }
            units.push(CellUnit {
                height: non_inline_h,
                hard_break_before: false,
                stored_frame_break_before: false,
                vpos_gap_before: false,
                para_idx,
                vis_start: 0,
                vis_end: 0,
                nested_row: None,
                nested_table_fragment: None,
                mixed_nested_fragment: false,
                mixed_nested_trailing: false,
                mixed_nested_content_height: 0.0,
                mixed_nested_recursive: false,
                mixed_nested_starts_after_table: false,
                mixed_nested_source_para_idx: None,
                recursive_block_prelude_role: RecursiveBlockPreludeRole::None,
                top_and_bottom_flow: true,
                empty_spacer: false,
                non_inline_control_range: None,
            });
        };
        let append_non_inline_units = |units: &mut Vec<CellUnit>,
                                       para_idx: usize,
                                       extra_h: f64,
                                       top_and_bottom_h: f64,
                                       other_h: f64|
         -> std::ops::Range<usize> {
            let (top_extra_h, other_extra_h) =
                split_non_inline_extra(extra_h, top_and_bottom_h, other_h);
            // TopAndBottom flow 는 그림/도형이 통째로 다음 조각에 넘어가야 해서 atomic 으로
            // 유지한다. Square/Tight/Through flow 는 텍스트 박스 꼬리가 페이지를 걸쳐
            // 이어질 수 있으므로 기존 fragment 모델을 유지한다.
            let other_start = units.len();
            append_fragment_units(units, para_idx, other_extra_h);
            let other_end = units.len();
            append_atomic_unit(units, para_idx, top_extra_h);
            other_start..other_end
        };
        // 기존 16px generic fragment의 높이·개수·순서는 그대로 두고, 각 fragment가
        // 겹치는 Square/Tight/Through source control range만 복원한다. TopAndBottom
        // atomic unit은 이 metadata의 대상이 아니다.
        let tag_other_non_inline_control_units =
            |units: &mut [CellUnit], range: std::ops::Range<usize>, controls: &[(usize, f64)]| {
                if range.is_empty() || controls.is_empty() {
                    return;
                }
                let source_h: f64 = controls.iter().map(|(_, height)| *height).sum();
                let represented_h: f64 = units[range.clone()].iter().map(|unit| unit.height).sum();
                if source_h <= 0.5 || represented_h <= 0.5 {
                    return;
                }
                // TopAndBottom과 섞인 문단에서는 기존 비례 분할로 other flow가
                // 축소되어 있으므로, current unit 좌표를 source other-flow 좌표로
                // 환산한 뒤 겹치는 control 범위를 기록한다.
                let scale = represented_h / source_h;
                let mut rendered_offset = 0.0;
                for unit in &mut units[range] {
                    let source_start = rendered_offset / scale;
                    let source_end = (rendered_offset + unit.height) / scale;
                    let mut control_start = 0.0;
                    let mut first = None;
                    let mut last = None;
                    for (control_idx, control_h) in controls {
                        let control_end = control_start + control_h;
                        if control_end > source_start + 0.001 && control_start < source_end - 0.001
                        {
                            first.get_or_insert(*control_idx);
                            last = Some(*control_idx);
                        }
                        control_start = control_end;
                    }
                    unit.non_inline_control_range = first.zip(last);
                    rendered_offset += unit.height;
                }
            };
        for (pi, p) in cell.paragraphs.iter().enumerate() {
            let is_block_rowbreak = matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            ) && !table.common.treat_as_char;
            let (para_top_and_bottom_h, summed_para_other_non_inline_h) =
                self.paragraph_cell_non_inline_control_flow_parts(&p.controls);
            let para_other_non_inline_h =
                if native_hwp5_rowbreak_float_ladder && p.text.trim().is_empty() {
                    self.paragraph_parallel_other_non_inline_flow_band_height(&p.controls)
                        .unwrap_or(summed_para_other_non_inline_h)
                } else {
                    summed_para_other_non_inline_h
                };
            let para_other_non_inline_controls =
                self.paragraph_cell_other_non_inline_control_heights(&p.controls);
            let para_non_inline_h = para_top_and_bottom_h + para_other_non_inline_h;
            let mut comp = compose_paragraph(p);
            crate::renderer::composer::recompose_for_cell_width(&mut comp, p, inner_width, styles);
            // [#2291] 부실 저장(ls==1 인데 실폭 초과) 문단 재분할 — 가로쓰기 셀 한정.
            if cell.text_direction == 0 {
                crate::renderer::composer::recompose_stored_single_line_if_overflowing(
                    &mut comp,
                    p,
                    inner_width,
                    styles,
                );
            }
            let para_style = styles.para_styles.get(p.para_shape_id as usize);
            let is_empty_spacer_para = p.text.trim().is_empty() && p.controls.is_empty();
            let preserve_forward_stored_empty_spacer = {
                let profile = self.profile.get();
                (profile.native_hwp5_layout() || profile.hwp5_origin_hwpx())
                    && is_empty_spacer_para
                    && matches!(p.line_segs.as_slice(), [seg] if !line_seg_is_synthetic(seg))
                    && match (p.line_segs.first(), cell.paragraphs.get(pi + 1)) {
                        (Some(seg), Some(next_para)) if next_para.controls.is_empty() => {
                            match next_para.line_segs.first() {
                                Some(next) if !line_seg_is_synthetic(next) => {
                                    let full_line_box = seg.line_height > 0
                                        && next.line_height > 0
                                        && i64::from(seg.line_height) * 4
                                            >= i64::from(next.line_height) * 3;
                                    full_line_box
                                        && next.vertical_pos
                                            >= seg.vertical_pos.saturating_add(seg.line_height)
                                }
                                _ => false,
                            }
                        }
                        _ => false,
                    }
            };
            let preserve_vpos_empty_spacer = is_empty_spacer_para
                && ((preserve_linear_single_cell_vpos
                    && p.line_segs.len() == 1
                    && p.line_segs
                        .first()
                        .is_some_and(|seg| seg.vertical_pos >= cell_first_vpos))
                    // #2430 물리 16쪽의 1×1 비인라인 표는 vertical_offset이
                    // 1801HU라 선형 vpos 모드가 아니지만, p[0] 빈 Enter 뒤의
                    // p[1]이 0→1800HU로 전진하고 빈 줄 높이도 다음 본문 줄의
                    // 82%다. 이 저장 순방향 full-line box는 overlay가 아니므로
                    // 0높이로 접지 않는다. 반면 작은 장식 간격용 빈 문단은 기존
                    // collapse를 유지한다(hwpx_sample2.hwp: 500~600/1000HU).
                    // 다음 문단이 중첩 control을 host하면 그 vpos는 control 배치
                    // 좌표이므로 빈 Enter의 독립 줄박스 증거로 사용하지 않는다.
                    || preserve_forward_stored_empty_spacer);
            let legacy_single_cell_empty_spacer = is_block_rowbreak
                && table.row_count == 1
                && table.col_count == 1
                && is_empty_spacer_para
                && cell_has_visible_content
                && !preserve_vpos_empty_spacer;
            let collapse_native_float_ladder_spacer = if native_hwp5_rowbreak_float_ladder
                && is_empty_spacer_para
                && cell_has_visible_content
            {
                let run_start = (0..pi)
                    .rev()
                    .find(|&idx| !plain_empty_paragraph[idx])
                    .map_or(0, |idx| idx + 1);
                let run_end = ((pi + 1)..para_count)
                    .find(|&idx| !plain_empty_paragraph[idx])
                    .unwrap_or(para_count);
                run_end - run_start >= 2
                    && run_start > 0
                    && run_end < para_count
                    && other_non_inline_flow_paragraph[run_start - 1]
                    && other_non_inline_flow_paragraph[run_end]
            } else {
                false
            };
            let collapse_empty_rowbreak_spacer =
                legacy_single_cell_empty_spacer || collapse_native_float_ladder_spacer;
            let is_last_para = pi + 1 == para_count;
            // [Task #1488] 가시 텍스트 문단 여부 — 비가시(빈) 오버레이 스페이서 문단이 만든
            // vpos 리셋을 하드 브레이크(강제 페이지 분할)에서 제외하기 위한 게이트.
            // 가시 텍스트 문단 사이 리셋(Task #993 의도)은 그대로 하드 브레이크로 보존한다.
            let para_has_visible_text = p.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
            let para_uses_synthetic_line_segs =
                !p.line_segs.is_empty() && p.line_segs.iter().all(|seg| line_seg_is_synthetic(seg));
            let raw_spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);
            let spacing_before = if pi > 0 {
                raw_spacing_before
            } else if self.profile.get().hwpx_stored_layout()
                && is_block_rowbreak
                && para_uses_synthetic_line_segs
            {
                // HWPX 에서 lineSegArray 가 누락된 표 셀 문단은 reflow 로 합성되지만,
                // ParaShape 의 spacing_before 는 여전히 문서 속성이다. 저장 HWP 는
                // 첫 줄 vpos 에 이 값을 반영하므로 row cut 측정도 같은 값을 사용한다.
                raw_spacing_before
            } else if raw_spacing_before > 0.0 {
                let first_vpos = p
                    .line_segs
                    .first()
                    .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi))
                    .unwrap_or(0.0)
                    .max(0.0);
                raw_spacing_before.min(first_vpos)
            } else {
                0.0
            };
            let spacing_after = if !is_last_para {
                para_style.map(|s| s.spacing_after).unwrap_or(0.0)
            } else {
                0.0
            };
            // vpos 리셋 검출: 직전 문단 끝보다 현재 문단 시작 vpos 가 작으면 리셋.
            let reset_before = if pi > 0 && cell_has_local_vpos_origin {
                let prev = &cell.paragraphs[pi - 1];
                match (prev.line_segs.last(), p.line_segs.first()) {
                    (Some(prev_seg), Some(cur_seg))
                        if !line_seg_is_synthetic(prev_seg) && !line_seg_is_synthetic(cur_seg) =>
                    {
                        let prev_end = prev_seg.vertical_pos.saturating_add(prev_seg.line_height);
                        cur_seg.vertical_pos >= 0 && prev_end > 0 && cur_seg.vertical_pos < prev_end
                    }
                    _ => false,
                }
            } else {
                false
            };
            // #2430 p14의 비선형 부모 셀에는 한컴이 무시하는 빈 Enter가 있고,
            // 그 단일 lineseg도 다음 저장 좌표 0으로 rewind한다. 이를 프레임
            // 경계로 올리면 39쪽 정본이 38쪽으로 줄어든다. 실제 빈 문단은
            // #4069처럼 1×1 선형 부모의 저장 vpos를 보존하는 경우에만 증거로 쓴다.
            let stored_frame_tail_before_next_para = if cell_has_local_vpos_origin {
                match (p.line_segs.last(), cell.paragraphs.get(pi + 1)) {
                    (Some(prev), Some(next))
                        if !next.text.is_empty()
                            || !next.controls.is_empty()
                            || (preserve_linear_single_cell_vpos
                                && next.text.is_empty()
                                && next.controls.is_empty()
                                && next.line_segs.len() == 1
                                && next.line_segs.first().is_some_and(|seg| {
                                    seg.vertical_pos >= cell_first_vpos
                                        && !line_seg_is_synthetic(seg)
                                })) =>
                    {
                        next.line_segs
                            .first()
                            .is_some_and(|cur| is_stored_frame_rewind(prev, cur))
                    }
                    _ => false,
                }
            } else {
                false
            };
            let stored_frame_break_before_para = if pi > 0 && cell_has_local_vpos_origin {
                let prev_para = &cell.paragraphs[pi - 1];
                match (prev_para.line_segs.last(), p.line_segs.first()) {
                    (Some(prev), Some(cur)) => is_stored_frame_rewind(prev, cur),
                    _ => false,
                }
            } else {
                false
            };
            let prev_para_has_mixed_nested_table = if pi > 0 {
                let prev = &cell.paragraphs[pi - 1];
                !prev.text.trim().is_empty()
                    && prev.controls.iter().any(|c| matches!(c, Control::Table(_)))
            } else {
                false
            };
            let vpos_gap_threshold_hu = (12.0 / self.dpi * 7200.0).round() as i32;
            let vpos_gap_before_para = if use_vpos_unit_positions && pi > 0 && cell_first_vpos == 0
            {
                let prev = &cell.paragraphs[pi - 1];
                match (prev.line_segs.last(), p.line_segs.first()) {
                    (Some(prev_seg), Some(cur_seg))
                        if !line_seg_is_synthetic(prev_seg) && !line_seg_is_synthetic(cur_seg) =>
                    {
                        let prev_end = prev_seg
                            .vertical_pos
                            .saturating_add(prev_seg.line_height)
                            .saturating_add(prev_seg.line_spacing);
                        cur_seg.vertical_pos >= 0
                            && prev_end > 0
                            && cur_seg.vertical_pos > prev_end + vpos_gap_threshold_hu
                    }
                    _ => false,
                }
            } else {
                false
            };
            let line_reset_before = |li: usize| -> bool {
                if li == 0 {
                    return reset_before;
                }
                if !cell_has_local_vpos_origin {
                    return false;
                }
                let Some(prev) = p.line_segs.get(li - 1) else {
                    return false;
                };
                let Some(cur) = p.line_segs.get(li) else {
                    return false;
                };
                if line_seg_is_synthetic(prev) || line_seg_is_synthetic(cur) {
                    return false;
                }
                let prev_end = prev.vertical_pos.saturating_add(prev.line_height);
                cur.vertical_pos >= 0 && prev_end > 0 && cur.vertical_pos < prev_end
            };
            let stored_frame_break_before = |li: usize| -> bool {
                if li == 0 {
                    return stored_frame_break_before_para;
                }
                if !line_reset_before(li) {
                    return false;
                }
                let Some(prev) = p.line_segs.get(li - 1) else {
                    return false;
                };
                let Some(cur) = p.line_segs.get(li) else {
                    return false;
                };
                // 42065 p10의 같은 문단 58620→0HU도 위의 HWP5 저장 프레임
                // 판정과 같은 계약을 사용한다.
                is_stored_frame_rewind(prev, cur)
            };
            // [Task #993] 줄 높이는 렌더러(layout_composed_paragraph)와 동일하게
            // corrected_line_height 를 적용한다 — raw line_height 가 폰트보다
            // 작은 폴백 케이스에서 렌더러가 키운 높이를 컷 측정이 따라가지
            // 못하면 분할 표가 페이지를 넘는다(측정 공간 불일치).
            // [#2070 실험] 셀 마지막 줄 인덱스 - em 공식 게이트.
            let cell_last_line_idx = if is_last_para && !comp.lines.is_empty() {
                Some(comp.lines.len() - 1)
            } else {
                None
            };
            let corrected_h = |line: &ComposedLine, li: usize| -> f64 {
                let raw_lh = hwpunit_to_px(line.line_height, self.dpi);
                // [Task #1811] HWPX RowBreak 셀의 synthetic lineSeg 는 저장 근거가 아니라
                // reflow 산물이다. row cut 측정에서 다시 corrected_line_height 를 적용하면
                // HWP 기준보다 줄 유닛이 커져 p4→p5 split 이 한 유닛 빨라진다.
                if self.profile.get().hwpx_stored_layout()
                    && is_block_rowbreak
                    && para_uses_synthetic_line_segs
                {
                    return raw_lh;
                }
                // [#2112] 실제 저장 LINE_SEG 를 보유한 셀 문단은 저장 줄높이를 신뢰한다.
                // 한글은 압축 줄높이(lh < 글자크기)를 저장값대로 렌더하는데 corrected
                // 보정이 fs×줄간격% 로 대체해 행높이가 부풀었다(39607: 행별 +3.8~
                // +76.8px, 표 합계 +335px → 다쪽 표 쪽수 밀림). 보정은 lineseg 부재
                // 폴백(#674/#993 원 목적)에만 유지.
                if p.line_segs.iter().any(|ls| !line_seg_is_synthetic(ls)) {
                    return raw_lh;
                }
                match para_style {
                    Some(ps) => {
                        let max_fs = line
                            .runs
                            .iter()
                            .map(|r| {
                                let ts = super::text_measurement::resolved_to_text_style(
                                    styles,
                                    r.char_style_id,
                                    r.lang_index,
                                );
                                if ts.font_size > 0.0 {
                                    ts.font_size
                                } else {
                                    12.0
                                }
                            })
                            .fold(0.0f64, f64::max);
                        // [#2169] NO_LS 순수 빈 문단(runs 없음 → max_fs=0)은 한글이
                        // 완전한 em 줄박스로 취급(80168 r4: 한글 = 10줄×em + 9gap 정확).
                        // 문단 char shape fs 로 폴백 — 컨트롤 앵커 문단은 제외(r6 중첩).
                        let max_fs = if max_fs <= 0.0
                            && crate::renderer::para_has_no_stored_line_segs(p)
                            && p.controls.is_empty()
                        {
                            p.char_shapes
                                .first()
                                .and_then(|cs| styles.char_styles.get(cs.char_shape_id as usize))
                                .map(|cs| cs.font_size)
                                .unwrap_or(0.0)
                        } else {
                            max_fs
                        };
                        // [Issue #1842] 부재 LINE_SEG 셀의 placeholder(400)→corrected
                        // max_fs*ls% 팽창을 em 으로 교정 — CellBreak 표.
                        // [#2150/#2169] 일반화: 한글 NO_LS fresh 공식 — 비마지막 줄
                        // fs×ls% 동치 + 셀 마지막 줄만 em (ls 사다리 + 80168 per-row 확정).
                        crate::renderer::corrected_line_height_for_variant_synthetic(
                            raw_lh,
                            max_fs,
                            ps.line_spacing_type,
                            ps.line_spacing,
                            crate::renderer::para_has_no_stored_line_segs(p)
                                && (!p.text.is_empty() || p.controls.is_empty())
                                && (matches!(table.page_break, TablePageBreak::CellBreak)
                                    // [#2070 실험] 셀 마지막 줄 = em (5축 전면).
                                    || cell_last_line_idx == Some(li)),
                        )
                    }
                    None => raw_lh,
                }
            };
            let has_table_in_para = p.controls.iter().any(|c| matches!(c, Control::Table(_)));
            let para_has_top_and_bottom_non_inline_control =
                p.controls.iter().any(|control| match control {
                    Control::Picture(pic) => matches!(pic.common.text_wrap, TextWrap::TopAndBottom),
                    Control::Shape(shape) => {
                        let common = shape.common();
                        matches!(common.text_wrap, TextWrap::TopAndBottom)
                    }
                    _ => false,
                });
            let line_count = comp.lines.len();
            let line_core_height: f64 = comp
                .lines
                .iter()
                .enumerate()
                .map(|(li, line)| corrected_h(line, li))
                .sum();
            let para_non_inline_extra_h = if p.text.trim().is_empty() && line_count > 0 {
                (para_non_inline_h - line_core_height).max(0.0)
            } else {
                para_non_inline_h
            };
            let para_top_and_bottom_flow_unit =
                para_has_top_and_bottom_non_inline_control && !para_has_visible_text;
            let previous_single_empty_unit_idx = if pi > 0
                && plain_empty_paragraph[pi - 1]
                && units
                    .last()
                    .is_some_and(|unit| unit.para_idx == pi - 1 && unit.empty_spacer)
                && (units.len() == 1 || units[units.len() - 2].para_idx != pi - 1)
            {
                Some(units.len() - 1)
            } else {
                None
            };
            let is_exact_recursive_prelude = line_count == 1
                && para_has_visible_text
                && !has_table_in_para
                && previous_single_empty_unit_idx.is_some()
                && cell
                    .paragraphs
                    .get(pi + 1)
                    .is_some_and(Self::paragraph_hosts_single_cell_nested_table);
            if is_exact_recursive_prelude {
                let separator_idx = previous_single_empty_unit_idx.expect("checked above");
                let separator_is_explicit_page_break = matches!(
                    cell.paragraphs[pi - 1].column_type,
                    crate::model::paragraph::ColumnBreakType::Page
                        | crate::model::paragraph::ColumnBreakType::Section
                );
                units[separator_idx].recursive_block_prelude_role =
                    if separator_is_explicit_page_break {
                        RecursiveBlockPreludeRole::ExplicitPageBreakSeparator
                    } else {
                        RecursiveBlockPreludeRole::EmptySeparator
                    };
            }
            let mut unit_cum = units.iter().map(|u| u.height).sum::<f64>();
            // [Task #1073] 텍스트 없는 문단(가시 텍스트 없음 — 합성 줄은 placeholder)에 단일
            // 중첩 표가 있고 그 표가 2행 이상이면 per-중첩행 유닛으로 분해 — advance_row_cut 가
            // 중첩 표 행 경계에서 페이지 분할할 수 있게 한다. whole-row 높이 합은
            // calc_nested_table_height 와 정확히 일치(드리프트 0):
            // Σ row_h + cs*(n-1) + om_top + om_bottom + spacing.
            // 2단계+ 중첩/텍스트 동거 문단은 아래 atom 폴백 유지(범위 외).
            if has_table_in_para && p.text.trim().is_empty() {
                let nested_tables: Vec<&crate::model::table::Table> = p
                    .controls
                    .iter()
                    .filter_map(|c| match c {
                        Control::Table(t) => Some(t.as_ref()),
                        _ => None,
                    })
                    .collect();
                if nested_tables.len() == 1
                    && nested_tables[0].row_count >= 2
                    && !matches!(
                        nested_tables[0].page_break,
                        crate::model::table::TablePageBreak::None
                    )
                {
                    let nt = nested_tables[0];
                    let ncol = nt.col_count as usize;
                    let nrow = nt.row_count as usize;
                    // 분할 컷은 저장된 표 높이보다 실제 콘텐츠 높이를 기준으로 잡아야
                    // page-larger 중첩 표가 한컴처럼 행 단위로 이어진다.
                    // [#2148/#2169] NO_LS 중첩 표(왕복 synthetic 포함)만 선언-fit
                    // (fit_row_heights_to_common_height, 성장 전용) — 저장 lineseg
                    // 문서는 #1073 콘텐츠 기준 유지 (자기-export HWPX 왕복 정합).
                    let nt_all_no_ls = nt
                        .cells
                        .iter()
                        .all(|c| c.paragraphs.iter().all(|p| p.line_segs.is_empty()));
                    let rhs = if nt_all_no_ls {
                        self.resolve_row_heights(nt, ncol, nrow, None, styles, true)
                    } else {
                        self.resolve_row_heights_for_content(nt, ncol, nrow, None, styles, true)
                    };
                    let ncs = hwpunit_to_px(nt.cell_spacing as i32, self.dpi);
                    let om_top = hwpunit_to_px(nt.outer_margin_top as i32, self.dpi);
                    let om_bot = hwpunit_to_px(nt.outer_margin_bottom as i32, self.dpi);
                    for (ri, rh) in rhs.iter().enumerate() {
                        // [#4069] CELL 분할 중첩 표는 큰 행을 단일 atom으로 바깥
                        // 원장에 올리지 않는다. 행에서 콘텐츠가 가장 높은 셀의 unit
                        // 경계를 공통 높이 축으로 삼고, 각 경계에서 모든 셀의 누적
                        // cursor를 기록한다. 따라서 첫 조각과 continuation 모두 같은
                        // 자식 RowCut을 렌더러에 전달할 수 있다.
                        if matches!(nt.page_break, TablePageBreak::RowBreak) {
                            let mut row_cells: Vec<&crate::model::table::Cell> = nt
                                .cells
                                .iter()
                                .filter(|cell| cell.row as usize == ri && cell.row_span == 1)
                                .collect();
                            row_cells.sort_by_key(|cell| cell.col);
                            let row_is_auto_height = !row_cells.is_empty()
                                && row_cells.iter().all(|cell| cell.height == 0);
                            let row_has_crossing_span = nt.cells.iter().any(|cell| {
                                let start = cell.row as usize;
                                let end = start + (cell.row_span as usize).max(1);
                                cell.row_span > 1 && start <= ri && ri < end
                            });
                            let row_units: Vec<std::sync::Arc<Vec<CellUnit>>> = row_cells
                                .iter()
                                .map(|cell| self.cell_units(cell, nt, styles))
                                .collect();
                            let driver = row_units
                                .iter()
                                .enumerate()
                                .filter(|(_, cell_units)| !cell_units.is_empty())
                                .max_by(|(_, a), (_, b)| {
                                    let ah: f64 = a.iter().map(|unit| unit.height).sum();
                                    let bh: f64 = b.iter().map(|unit| unit.height).sum();
                                    ah.total_cmp(&bh)
                                })
                                .map(|(index, _)| index);

                            if let Some(driver_index) = driver.filter(|driver_index| {
                                row_is_auto_height
                                    && !row_has_crossing_span
                                    && row_units[*driver_index].len() > 1
                            }) {
                                let driver_units = &row_units[driver_index];
                                let driver_total: f64 =
                                    driver_units.iter().map(|unit| unit.height).sum();
                                let row_extra = (*rh - driver_total).max(0.0);
                                let mut driver_before = 0.0;

                                for (fragment_index, driver_unit) in driver_units.iter().enumerate()
                                {
                                    let driver_after = driver_before + driver_unit.height;
                                    let cuts_at = |height: f64| -> RowCut {
                                        row_units
                                            .iter()
                                            .map(|cell_units| {
                                                let mut consumed = 0.0;
                                                let mut count = 0usize;
                                                while count < cell_units.len()
                                                    && consumed + cell_units[count].height
                                                        <= height + 0.1
                                                {
                                                    consumed += cell_units[count].height;
                                                    count += 1;
                                                }
                                                count
                                            })
                                            .collect()
                                    };
                                    let start_cut = cuts_at(driver_before);
                                    let end_cut = cuts_at(driver_after);
                                    let terminal = row_units
                                        .iter()
                                        .zip(end_cut.iter())
                                        .all(|(cell_units, end)| *end >= cell_units.len());
                                    let mut uh = driver_unit.height;
                                    if fragment_index == 0 {
                                        uh += row_extra * 0.5;
                                    }
                                    if fragment_index + 1 == driver_units.len() {
                                        uh += row_extra - row_extra * 0.5;
                                        if ri + 1 < nrow {
                                            uh += ncs;
                                        }
                                        if ri + 1 == nrow {
                                            uh += om_bot + spacing_after;
                                        }
                                    }
                                    if ri == 0 && fragment_index == 0 {
                                        uh += om_top + spacing_before;
                                    }

                                    let mut hard_break_before = driver_unit.hard_break_before
                                        || (reset_before && ri == 0 && fragment_index == 0);
                                    let mut stored_frame_break_before =
                                        driver_unit.stored_frame_break_before;
                                    let mut vpos_gap_before =
                                        vpos_gap_before_para && ri == 0 && fragment_index == 0;
                                    for ((cell_units, start), end) in
                                        row_units.iter().zip(start_cut.iter()).zip(end_cut.iter())
                                    {
                                        if end > start {
                                            if cell_units
                                                .get(*start)
                                                .is_some_and(|unit| unit.hard_break_before)
                                            {
                                                hard_break_before = true;
                                            }
                                            if cell_units
                                                .get(*start)
                                                .is_some_and(|unit| unit.stored_frame_break_before)
                                            {
                                                stored_frame_break_before = true;
                                            }
                                            if cell_units
                                                .get(*start)
                                                .is_some_and(|unit| unit.vpos_gap_before)
                                            {
                                                vpos_gap_before = true;
                                            }
                                        }
                                    }
                                    if use_vpos_unit_positions
                                        && ri == 0
                                        && fragment_index == 0
                                        && !hard_break_before
                                    {
                                        if let Some(seg) = p.line_segs.first() {
                                            let target_top = normalized_vpos_px(seg.vertical_pos);
                                            if target_top > unit_cum {
                                                uh += target_top - unit_cum;
                                                vpos_gap_before = true;
                                            }
                                        }
                                    }

                                    units.push(CellUnit {
                                        height: uh,
                                        hard_break_before,
                                        stored_frame_break_before,
                                        vpos_gap_before,
                                        para_idx: pi,
                                        vis_start: 0,
                                        vis_end: line_count.max(1),
                                        nested_row: Some(ri),
                                        nested_table_fragment: Some(NestedTableUnitCut {
                                            start_cut,
                                            end_cut,
                                            terminal,
                                        }),
                                        mixed_nested_fragment: false,
                                        mixed_nested_trailing: false,
                                        mixed_nested_content_height: 0.0,
                                        mixed_nested_recursive: false,
                                        mixed_nested_starts_after_table: false,
                                        mixed_nested_source_para_idx: None,
                                        recursive_block_prelude_role:
                                            RecursiveBlockPreludeRole::None,
                                        top_and_bottom_flow: false,
                                        empty_spacer: false,
                                        non_inline_control_range: None,
                                    });
                                    unit_cum += uh;
                                    driver_before = driver_after;
                                }
                                continue;
                            }
                        }

                        let mut uh = *rh;
                        let hard_break_before = reset_before && ri == 0;
                        let mut vpos_gap_before = vpos_gap_before_para && ri == 0;
                        if use_vpos_unit_positions && ri == 0 && !hard_break_before {
                            if let Some(seg) = p.line_segs.first() {
                                let target_top = normalized_vpos_px(seg.vertical_pos);
                                if target_top > unit_cum {
                                    uh += target_top - unit_cum;
                                    vpos_gap_before = true;
                                }
                            }
                        }
                        if ri + 1 < nrow {
                            uh += ncs;
                        }
                        if ri == 0 {
                            uh += om_top + spacing_before;
                        }
                        if ri + 1 == nrow {
                            uh += om_bot + spacing_after;
                        }
                        units.push(CellUnit {
                            height: uh,
                            hard_break_before,
                            stored_frame_break_before: false,
                            vpos_gap_before,
                            para_idx: pi,
                            vis_start: 0,
                            vis_end: line_count.max(1),
                            nested_row: Some(ri),
                            nested_table_fragment: None,
                            mixed_nested_fragment: false,
                            mixed_nested_trailing: false,
                            mixed_nested_content_height: 0.0,
                            mixed_nested_recursive: false,
                            mixed_nested_starts_after_table: false,
                            mixed_nested_source_para_idx: None,
                            recursive_block_prelude_role: RecursiveBlockPreludeRole::None,
                            top_and_bottom_flow: false,
                            empty_spacer: false,
                            non_inline_control_range: None,
                        });
                        unit_cum += uh;
                    }
                    let non_inline_range = append_non_inline_units(
                        &mut units,
                        pi,
                        para_non_inline_extra_h,
                        para_top_and_bottom_h,
                        para_other_non_inline_h,
                    );
                    tag_other_non_inline_control_units(
                        &mut units,
                        non_inline_range,
                        &para_other_non_inline_controls,
                    );
                    continue;
                } else if nested_tables.len() == 1 && nested_tables[0].row_count == 1 {
                    // [#2007] 1×1(단일 행) 중첩 표: per-중첩행 분해(row_count>=2)가 불가하나,
                    // 그 단일 셀 콘텐츠가 페이지보다 크면(42065 pi=7: 135문단 8164px) atomic 으로
                    // 두면 못 쪼개져 under-pagination. 텍스트+중첩표 문단에 쓰이는
                    // nested_table_mixed_fragment_heights(단일 행 셀 문단을 페이지 분할 가능한
                    // fragment 로 분해)를 빈-텍스트 문단에도 적용해 splittable 유닛으로 산출.
                    let nt = nested_tables[0];
                    let frags = self.nested_table_mixed_fragment_heights(nt, styles);
                    if std::env::var("RHWP_DIAG_NESTED_OWNER").is_ok()
                        && nt.cells.len() == 1
                        && nt.cells[0].paragraphs.iter().any(|paragraph| {
                            paragraph
                                .controls
                                .iter()
                                .any(|control| matches!(control, Control::Table(_)))
                        })
                    {
                        eprintln!(
                            "DIAG_NESTED_OWNER parent_pi={pi} child_paras={} fragments={}",
                            nt.cells[0].paragraphs.len(),
                            frags.len(),
                        );
                        for (fragment_index, fragment) in frags.iter().enumerate() {
                            let is_child_table = fragment
                                .source_para_idx
                                .and_then(|source_pi| nt.cells[0].paragraphs.get(source_pi))
                                .is_some_and(|paragraph| {
                                    paragraph
                                        .controls
                                        .iter()
                                        .any(|control| matches!(control, Control::Table(_)))
                                });
                            eprintln!(
                                "  unit={fragment_index} source={:?} table={} h={:.1} trailing={} after_table={}",
                                fragment.source_para_idx,
                                is_child_table,
                                fragment.height,
                                fragment.trailing,
                                fragment.starts_after_table,
                            );
                        }
                    }
                    // 게이트: 콘텐츠가 **명백히 여러 페이지가 필요**(≥ MULTI_PAGE_PX)할 때만
                    // fragment 분해한다. 임계를 넉넉히(≈2 페이지) 두는 이유:
                    // - 한 페이지에 맞는 1×1 중첩 표(서식): fragment 렌더 미세차로 회귀(form-002).
                    // - 1~2 페이지 경계선 표(76076 규제영향분석서의 여러 ~1000px 중첩셀): fragment
                    //   경계가 기존 배치와 ±1 어긋나 공식 PDF 쪽수(issue_1891) 회귀.
                    // 42065 pi=7(8164px, 8쪽분)·2781515 별표(수쪽분)처럼 ≫ 2페이지인 거대 셀만 대상.
                    let page_avail = self.current_body_area.get().3;
                    let multi_page_px = if page_avail > 0.0 {
                        page_avail * 1.0
                    } else {
                        900.0
                    };
                    let total_frag_h: f64 = frags.iter().map(|fragment| fragment.height).sum();
                    // 저장된 fragment 높이가 표의 물리 행 높이보다 작을 수 있다. 특히
                    // 59043 p35/p36의 1×1 child는 fragment 합이 body보다 3.6px 작지만
                    // 실제 행 기하는 1336px라 한 쪽에 들어가지 않는다. fragment 합만
                    // 보면 이 표를 atom으로 소비해 p36 source owner가 사라진다.
                    // page의 남은 공간이 아니라 문서 고유 물리 높이만 사용해
                    // `cell_units_cache`가 page context에 의존하지 않게 한다.
                    let physical_nested_h = self.calc_nested_table_height(nt, styles);
                    let exceeds_physical_page = physical_nested_h > multi_page_px + 0.5;
                    // child content flow가 parent 선언 높이보다 큰 native short-parent
                    // 구조에서만 1×1 child를 fragment unit으로 전개한다. `common.height`
                    // 자체는 stale viewport일 수 있으므로, 같은 mixed fragment 원장의
                    // 합으로 판단한다. paginator도 같은 helper로 이 row만 split 가능으로
                    // 올린다 (76076 p81→82).
                    let native_short_parent_child_fragment = self
                        .native_short_parent_child_fragment_eligible(
                            table,
                            cell,
                            nt,
                            total_frag_h.max(physical_nested_h),
                        );
                    // [#4069 Stage 3] 한 페이지 이하 1×1 자식 표라도 host line이
                    // 저장 프레임 하단까지 차지하고 다음 문단이 새 프레임으로
                    // rewind하면 현재 쪽의 남은 공간에서 시작해야 한다. 원자 처리하면
                    // 42065 p15의 `조달청` 다음 표가 통째로 p16으로 밀린다.
                    // HWP5 저장 경계로 확인된 경우만 열어 form-002/#1891의 일반
                    // 단일 페이지 중첩 표 배치는 유지한다.
                    if frags.len() > 1
                        && (total_frag_h > multi_page_px
                            || exceeds_physical_page
                            || native_short_parent_child_fragment
                            || stored_frame_tail_before_next_para)
                    {
                        let om_top = hwpunit_to_px(nt.outer_margin_top as i32, self.dpi);
                        let om_bot = hwpunit_to_px(nt.outer_margin_bottom as i32, self.dpi);
                        let n = frags.len();
                        for (fi, fragment) in frags.into_iter().enumerate() {
                            let mut uh = fragment.height;
                            let hard_break_before =
                                fragment.hard_break_before || (reset_before && fi == 0);
                            let mut vpos_gap_before = vpos_gap_before_para && fi == 0;
                            if use_vpos_unit_positions && fi == 0 && !hard_break_before {
                                if let Some(seg) = p.line_segs.first() {
                                    let target_top = normalized_vpos_px(seg.vertical_pos);
                                    if target_top > unit_cum {
                                        uh += target_top - unit_cum;
                                        vpos_gap_before = true;
                                    }
                                }
                            }
                            if fi == 0 {
                                uh += om_top + spacing_before;
                            }
                            if fi + 1 == n {
                                uh += om_bot + spacing_after;
                            }
                            units.push(CellUnit {
                                height: uh,
                                hard_break_before,
                                stored_frame_break_before: fragment.stored_frame_break_before,
                                vpos_gap_before,
                                para_idx: pi,
                                vis_start: line_count,
                                vis_end: line_count,
                                nested_row: None,
                                nested_table_fragment: None,
                                mixed_nested_fragment: true,
                                mixed_nested_trailing: fragment.trailing,
                                mixed_nested_content_height: fragment.content_height,
                                mixed_nested_recursive: fragment.recursive,
                                mixed_nested_starts_after_table: fragment.starts_after_table,
                                mixed_nested_source_para_idx: fragment.source_para_idx,
                                recursive_block_prelude_role: fragment.recursive_block_prelude_role,
                                top_and_bottom_flow: false,
                                empty_spacer: false,
                                non_inline_control_range: None,
                            });
                            unit_cum += uh;
                        }
                        let non_inline_range = append_non_inline_units(
                            &mut units,
                            pi,
                            para_non_inline_extra_h,
                            para_top_and_bottom_h,
                            para_other_non_inline_h,
                        );
                        tag_other_non_inline_control_units(
                            &mut units,
                            non_inline_range,
                            &para_other_non_inline_controls,
                        );
                        continue;
                    }
                }
            }
            if has_table_in_para && !p.text.trim().is_empty() && line_count > 0 {
                let nested_h: f64 = p
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                if nested_h > 0.0 {
                    for (li, line) in comp.lines.iter().enumerate() {
                        let h = corrected_h(line, li);
                        let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                        let is_cell_last_line = is_last_para && li + 1 == line_count;
                        let is_block_rowbreak = matches!(
                            table.page_break,
                            crate::model::table::TablePageBreak::RowBreak
                        ) && !table.common.treat_as_char;
                        let include_trailing_ls = !is_cell_last_line || para_count > 1;
                        let include_trailing_ls =
                            include_trailing_ls && (!is_cell_last_line || !is_block_rowbreak);
                        let mut lh = if include_trailing_ls { h + ls } else { h };
                        if li == 0 {
                            lh += spacing_before;
                        }
                        if li == line_count - 1 {
                            lh += spacing_after;
                        }
                        let hard_break_before = line_reset_before(li);
                        let mut vpos_gap_before = if li == 0 {
                            vpos_gap_before_para
                        } else if use_vpos_unit_positions && cell_first_vpos == 0 {
                            match (p.line_segs.get(li - 1), p.line_segs.get(li)) {
                                (Some(prev), Some(cur))
                                    if !line_seg_is_synthetic(prev)
                                        && !line_seg_is_synthetic(cur) =>
                                {
                                    cur.vertical_pos
                                        > prev.vertical_pos
                                            + prev.line_height
                                            + prev.line_spacing
                                            + vpos_gap_threshold_hu
                                }
                                _ => false,
                            }
                        } else {
                            false
                        };
                        if use_vpos_unit_positions {
                            if let Some(seg) = p.line_segs.get(li) {
                                if !line_seg_is_synthetic(seg) {
                                    let target_top = normalized_vpos_px(seg.vertical_pos);
                                    if target_top > unit_cum {
                                        lh += target_top - unit_cum;
                                        vpos_gap_before = true;
                                    }
                                }
                            }
                        }
                        units.push(CellUnit {
                            height: lh,
                            hard_break_before,
                            stored_frame_break_before: stored_frame_break_before(li),
                            vpos_gap_before,
                            para_idx: pi,
                            vis_start: li,
                            vis_end: li + 1,
                            nested_row: None,
                            nested_table_fragment: None,
                            mixed_nested_fragment: false,
                            mixed_nested_trailing: false,
                            mixed_nested_content_height: 0.0,
                            mixed_nested_recursive: false,
                            mixed_nested_starts_after_table: false,
                            mixed_nested_source_para_idx: None,
                            recursive_block_prelude_role: RecursiveBlockPreludeRole::None,
                            top_and_bottom_flow: false,
                            empty_spacer: false,
                            non_inline_control_range: None,
                        });
                        unit_cum += lh;
                    }

                    let has_internal_line_reset = p
                        .line_segs
                        .windows(2)
                        .any(|pair| pair[1].vertical_pos < pair[0].vertical_pos);
                    let target_h = if has_internal_line_reset {
                        (nested_h + 4.0 - line_core_height).max(0.0)
                    } else {
                        nested_h + 4.0
                    };
                    if target_h > 0.5 {
                        let mut fragment_heights: Vec<NestedFlowFragment> = p
                            .controls
                            .iter()
                            .filter_map(|ctrl| {
                                if let Control::Table(t) = ctrl {
                                    Some(self.nested_table_mixed_fragment_heights(t, styles))
                                } else {
                                    None
                                }
                            })
                            .flatten()
                            .collect();
                        if fragment_heights.is_empty() {
                            const NESTED_FRAGMENT_UNIT_PX: f64 = 16.0;
                            let mut remaining = target_h;
                            while remaining > 0.5 {
                                let h = remaining.min(NESTED_FRAGMENT_UNIT_PX);
                                fragment_heights.push(NestedFlowFragment {
                                    height: h,
                                    hard_break_before: false,
                                    stored_frame_break_before: false,
                                    trailing: false,
                                    content_height: h,
                                    recursive: false,
                                    starts_after_table: false,
                                    source_para_idx: None,
                                    recursive_block_prelude_role: RecursiveBlockPreludeRole::None,
                                });
                                remaining -= h;
                            }
                        } else {
                            let current_h: f64 = fragment_heights
                                .iter()
                                .map(|fragment| fragment.height)
                                .sum();
                            // [Task #1809] top pad 차감(c7dbe8a2, 종전 HWPX 한정)을 소스
                            // 무관화 — 한글 편집기 대조에서 pad 적용 컷 위치가 정답
                            // (admrul_0556 p1 조각 하단: 한글 808.8 = pad 적용 808.7,
                            // 미적용 810.1). HWP5 재파스에도 동일 적용해야 정합.
                            let hwpx_rowbreak_top_pad =
                                if is_block_rowbreak && !has_internal_line_reset {
                                    p.controls
                                        .iter()
                                        .filter_map(|ctrl| {
                                            if let Control::Table(t) = ctrl {
                                                let top_pad = t
                                                    .cells
                                                    .iter()
                                                    .filter(|cell| cell.row == 0)
                                                    .map(|cell| {
                                                        let (_, _, pad_top, _) =
                                                            self.resolve_cell_padding(cell, t);
                                                        pad_top
                                                    })
                                                    .fold(0.0f64, f64::max);
                                                Some(top_pad)
                                            } else {
                                                None
                                            }
                                        })
                                        .sum::<f64>()
                                } else {
                                    0.0
                                };
                            let top_up = (target_h - current_h).max(0.0);
                            let target_h = target_h - hwpx_rowbreak_top_pad.min(top_up);
                            if target_h > current_h + 0.5 {
                                if let Some(first) = fragment_heights.first_mut() {
                                    first.height += target_h - current_h;
                                    first.content_height = first.content_height.max(first.height);
                                }
                            }
                        }
                        for fragment in fragment_heights {
                            units.push(CellUnit {
                                height: fragment.height,
                                hard_break_before: fragment.hard_break_before,
                                stored_frame_break_before: fragment.stored_frame_break_before,
                                vpos_gap_before: false,
                                para_idx: pi,
                                vis_start: line_count,
                                vis_end: line_count,
                                nested_row: None,
                                nested_table_fragment: None,
                                mixed_nested_fragment: true,
                                mixed_nested_trailing: fragment.trailing,
                                mixed_nested_content_height: fragment.content_height,
                                mixed_nested_recursive: fragment.recursive,
                                mixed_nested_starts_after_table: fragment.starts_after_table,
                                mixed_nested_source_para_idx: fragment.source_para_idx,
                                recursive_block_prelude_role: fragment.recursive_block_prelude_role,
                                top_and_bottom_flow: false,
                                empty_spacer: false,
                                non_inline_control_range: None,
                            });
                            unit_cum += fragment.height;
                        }
                    }
                    let non_inline_range = append_non_inline_units(
                        &mut units,
                        pi,
                        para_non_inline_extra_h,
                        para_top_and_bottom_h,
                        para_other_non_inline_h,
                    );
                    tag_other_non_inline_control_units(
                        &mut units,
                        non_inline_range,
                        &para_other_non_inline_controls,
                    );
                    continue;
                }
            }
            if line_count == 0 || has_table_in_para {
                // 중첩 표/빈 문단 — atomic 유닛 1개.
                let nested_h: f64 = p
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                let para_h = if collapse_empty_rowbreak_spacer {
                    0.0
                } else if line_count == 0 {
                    let h = if nested_h > 0.0 {
                        nested_h
                    } else if crate::renderer::para_has_no_stored_line_segs(p)
                        && p.controls.is_empty()
                    {
                        // [#2169] NO_LS 순수 빈 문단 = 완전한 em 줄박스 (한글 공식:
                        // 80168 r4 c2 = 10줄×em + 9gap 정확). 비마지막 문단은
                        // fs×ls%(gap 포함 동치), 셀 마지막 문단은 em.
                        let fs = p
                            .char_shapes
                            .first()
                            .and_then(|cs| styles.char_styles.get(cs.char_shape_id as usize))
                            .map(|cs| cs.font_size)
                            .unwrap_or(0.0);
                        if fs <= 0.0 {
                            hwpunit_to_px(400, self.dpi)
                        } else if is_last_para {
                            fs
                        } else {
                            match para_style {
                                Some(ps) => crate::renderer::corrected_line_height(
                                    hwpunit_to_px(400, self.dpi),
                                    fs,
                                    ps.line_spacing_type,
                                    ps.line_spacing,
                                ),
                                None => fs,
                            }
                        }
                    } else {
                        hwpunit_to_px(400, self.dpi)
                    };
                    spacing_before + h + spacing_after
                } else {
                    let line_based_h: f64 = comp
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(li, line)| {
                            let h = corrected_h(line, li);
                            let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                            let is_cell_last_line = is_last_para && li + 1 == line_count;
                            // [Task #1022/#1086] trailing ls 규칙 — HeightMeasurer 와
                            // 정합. CellBreak/TAC 표는 기존 trailing geometry 를 보존하고,
                            // block RowBreak 표는 렌더 가시 높이처럼 셀 마지막 줄
                            // trailing 을 제외해 행 fit 을 맞춘다.
                            let is_block_rowbreak = matches!(
                                table.page_break,
                                crate::model::table::TablePageBreak::RowBreak
                            ) && !table.common.treat_as_char;
                            let include_trailing_ls = !is_cell_last_line || para_count > 1;
                            let include_trailing_ls =
                                include_trailing_ls && (!is_cell_last_line || !is_block_rowbreak);
                            let mut lh = if include_trailing_ls { h + ls } else { h };
                            if li == 0 {
                                lh += spacing_before;
                            }
                            if li == line_count - 1 {
                                lh += spacing_after;
                            }
                            lh
                        })
                        .sum();
                    let has_visible_text_with_nested = use_vpos_unit_positions
                        && comp
                            .lines
                            .iter()
                            .any(|line| line.runs.iter().any(|run| !run.text.trim().is_empty()));
                    if has_visible_text_with_nested && nested_h > 0.0 {
                        line_based_h + nested_h + 4.0
                    } else {
                        nested_h.max(line_based_h)
                    }
                };
                let hard_break_before = reset_before;
                let mut para_h = para_h;
                let mut vpos_gap_before = vpos_gap_before_para;
                if use_vpos_unit_positions {
                    if let Some(seg) = p.line_segs.first() {
                        if !line_seg_is_synthetic(seg) {
                            let target_top = normalized_vpos_px(seg.vertical_pos);
                            if target_top > unit_cum {
                                let delta = target_top - unit_cum;
                                let suppress_hwpx_mixed_nested_gap =
                                    self.profile.get().hwpx_stored_layout()
                                        && prev_para_has_mixed_nested_table
                                        && delta <= 24.0;
                                if !suppress_hwpx_mixed_nested_gap {
                                    para_h += delta;
                                    vpos_gap_before = true;
                                }
                            }
                        }
                    }
                }
                units.push(CellUnit {
                    height: para_h,
                    // [Task #1488] 비가시 빈 문단(중첩표 없음)의 오버레이 리셋은 페이지를
                    // 강제 분할하지 않는다 — 여분 빈 연속 페이지 방지. 중첩표가 있으면
                    // 가시 콘텐츠를 가지므로 리셋 보존.
                    hard_break_before: hard_break_before
                        && (has_table_in_para || para_has_visible_text),
                    stored_frame_break_before: false,
                    vpos_gap_before: vpos_gap_before && !collapse_empty_rowbreak_spacer,
                    para_idx: pi,
                    vis_start: 0,
                    vis_end: if collapse_empty_rowbreak_spacer {
                        0
                    } else {
                        line_count.max(1)
                    },
                    nested_row: None,
                    nested_table_fragment: None,
                    mixed_nested_fragment: false,
                    mixed_nested_trailing: false,
                    mixed_nested_content_height: 0.0,
                    mixed_nested_recursive: false,
                    mixed_nested_starts_after_table: false,
                    mixed_nested_source_para_idx: None,
                    recursive_block_prelude_role: RecursiveBlockPreludeRole::None,
                    top_and_bottom_flow: para_top_and_bottom_flow_unit,
                    empty_spacer: is_empty_spacer_para,
                    non_inline_control_range: None,
                });
                unit_cum += para_h;
            } else {
                // 일반 텍스트 문단 — 합성 줄마다 유닛 1개.
                for (li, line) in comp.lines.iter().enumerate() {
                    let h = corrected_h(line, li);
                    let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                    let is_cell_last_line = is_last_para && li + 1 == line_count;
                    let include_trailing_ls = !is_cell_last_line || para_count > 1;
                    let include_trailing_ls =
                        include_trailing_ls && (!is_cell_last_line || !is_block_rowbreak);
                    let mut lh = if include_trailing_ls { h + ls } else { h };
                    if collapse_empty_rowbreak_spacer {
                        lh = 0.0;
                    } else {
                        if li == 0 {
                            lh += spacing_before;
                        }
                        if li == line_count - 1 {
                            lh += spacing_after;
                        }
                    }
                    let hard_break_before = line_reset_before(li);
                    let mut vpos_gap_before = if li == 0 {
                        vpos_gap_before_para
                    } else if use_vpos_unit_positions && cell_first_vpos == 0 {
                        match (p.line_segs.get(li - 1), p.line_segs.get(li)) {
                            (Some(prev), Some(cur))
                                if !line_seg_is_synthetic(prev) && !line_seg_is_synthetic(cur) =>
                            {
                                cur.vertical_pos
                                    > prev.vertical_pos
                                        + prev.line_height
                                        + prev.line_spacing
                                        + vpos_gap_threshold_hu
                            }
                            _ => false,
                        }
                    } else {
                        false
                    };
                    if use_vpos_unit_positions {
                        if let Some(seg) = p.line_segs.get(li) {
                            if !line_seg_is_synthetic(seg) {
                                let target_top = normalized_vpos_px(seg.vertical_pos);
                                if target_top > unit_cum {
                                    let delta = target_top - unit_cum;
                                    let suppress_hwpx_mixed_nested_gap =
                                        self.profile.get().hwpx_stored_layout()
                                            && li == 0
                                            && prev_para_has_mixed_nested_table
                                            && delta <= 24.0;
                                    if !suppress_hwpx_mixed_nested_gap {
                                        lh += delta;
                                        vpos_gap_before = true;
                                    }
                                }
                            }
                        }
                    }
                    units.push(CellUnit {
                        height: lh,
                        // [Task #1488] 비가시(빈 텍스트) 오버레이 스페이서 문단이 만든 vpos
                        // 리셋은 페이지를 강제 분할하지 않는다. 셀 안에서 본문 텍스트 위에
                        // 겹쳐 놓인 빈 문단(동일/역방향 vpos)들이 리셋마다 페이지를 1장씩
                        // 양산하던 여분 빈 연속 페이지 회귀를 제거한다. 가시 텍스트 문단 사이
                        // 리셋(Task #993 의도)은 그대로 하드 브레이크로 보존한다.
                        hard_break_before: hard_break_before && para_has_visible_text,
                        stored_frame_break_before: para_has_visible_text
                            && stored_frame_break_before(li),
                        vpos_gap_before: vpos_gap_before && !collapse_empty_rowbreak_spacer,
                        para_idx: pi,
                        vis_start: if collapse_empty_rowbreak_spacer {
                            0
                        } else {
                            li
                        },
                        vis_end: if collapse_empty_rowbreak_spacer {
                            0
                        } else {
                            li + 1
                        },
                        nested_row: None,
                        nested_table_fragment: None,
                        mixed_nested_fragment: false,
                        mixed_nested_trailing: false,
                        mixed_nested_content_height: 0.0,
                        mixed_nested_recursive: false,
                        mixed_nested_starts_after_table: false,
                        mixed_nested_source_para_idx: None,
                        recursive_block_prelude_role: if is_exact_recursive_prelude {
                            RecursiveBlockPreludeRole::OneLineHeadingBeforeSingleCellTable
                        } else {
                            RecursiveBlockPreludeRole::None
                        },
                        top_and_bottom_flow: para_top_and_bottom_flow_unit,
                        empty_spacer: is_empty_spacer_para,
                        non_inline_control_range: None,
                    });
                    unit_cum += lh;
                }
            }
            let non_inline_range = append_non_inline_units(
                &mut units,
                pi,
                para_non_inline_extra_h,
                para_top_and_bottom_h,
                para_other_non_inline_h,
            );
            tag_other_non_inline_control_units(
                &mut units,
                non_inline_range,
                &para_other_non_inline_controls,
            );
        }

        let units =
            Self::delay_empty_anchor_topandbottom_flow_units_before_hard_break(units, cell, table);

        if let Ok(pattern) = std::env::var("RHWP_DIAG_CELL_UNITS") {
            if cell
                .paragraphs
                .iter()
                .any(|paragraph| paragraph.text.contains(&pattern))
            {
                eprintln!(
                    "DIAG_CELL_UNITS profile={:?} rows={} cols={} cells={} break={:?} tac={} units={}",
                    self.profile.get(),
                    table.row_count,
                    table.col_count,
                    table.cells.len(),
                    table.page_break,
                    table.common.treat_as_char,
                    units.len(),
                );
                for (unit_idx, unit) in units.iter().enumerate() {
                    eprintln!(
                        "  unit[{unit_idx}] h={:.2} para={} lines={}..{} hard={} stored={} gap={} empty={} topbottom={}",
                        unit.height,
                        unit.para_idx,
                        unit.vis_start,
                        unit.vis_end,
                        unit.hard_break_before,
                        unit.stored_frame_break_before,
                        unit.vpos_gap_before,
                        unit.empty_spacer,
                        unit.top_and_bottom_flow,
                    );
                }
            }
        }

        let _ = (pad_top, pad_bottom); // [Task #1022] cell.height 필러 제거 — row_cut_content_height 가 셀별 max(cell.height, content+pad) 로 행 단계에서 정합.
        units
    }

    fn delay_empty_anchor_topandbottom_flow_units_before_hard_break(
        units: Vec<CellUnit>,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
    ) -> Vec<CellUnit> {
        if !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) || table.common.treat_as_char
        {
            return units;
        }
        let mut has_future_visible_hard_break = vec![false; units.len()];
        let mut seen_visible_hard_break = false;
        for idx in (0..units.len()).rev() {
            has_future_visible_hard_break[idx] = seen_visible_hard_break;
            let unit = &units[idx];
            if unit.hard_break_before && unit.vis_start < unit.vis_end {
                seen_visible_hard_break = true;
            }
        }

        let mut reordered = Vec::with_capacity(units.len());
        let mut pending = Vec::new();
        for (idx, unit) in units.into_iter().enumerate() {
            if has_future_visible_hard_break[idx]
                && Self::is_delayable_empty_anchor_topandbottom_flow_unit(cell, &unit)
            {
                pending.push(unit);
                continue;
            }
            if unit.hard_break_before && unit.vis_start < unit.vis_end && !pending.is_empty() {
                reordered.append(&mut pending);
            }
            reordered.push(unit);
        }
        reordered.append(&mut pending);
        reordered
    }

    fn is_delayable_empty_anchor_topandbottom_flow_unit(
        cell: &crate::model::table::Cell,
        unit: &CellUnit,
    ) -> bool {
        if !Self::is_non_inline_control_flow_unit(unit) {
            return false;
        }
        let Some(para) = cell.paragraphs.get(unit.para_idx) else {
            return false;
        };
        para.text.trim().is_empty()
            && para.controls.iter().any(|control| match control {
                Control::Picture(pic) => {
                    !pic.common.treat_as_char
                        && pic.common.flow_with_text
                        && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
                        && matches!(pic.common.vert_rel_to, VertRelTo::Para)
                }
                Control::Shape(shape) => {
                    let common = shape.common();
                    !common.treat_as_char
                        && common.flow_with_text
                        && matches!(common.text_wrap, TextWrap::TopAndBottom)
                        && matches!(common.vert_rel_to, VertRelTo::Para)
                }
                _ => false,
            })
    }

    /// [#2097] 셀 문단 cp_idx 의 첫 유닛 앞까지의 누적 콘텐츠 높이(셀-로컬).
    /// 각주 앵커 문단이 컷 조각에 포함되는 경계(인서트-인지 컷 예산 상한) 산정용.
    /// 해당 문단 유닛이 없으면 None.
    pub(crate) fn cell_para_unit_offset(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        cp_idx: usize,
    ) -> Option<f64> {
        let units = self.cell_units(cell, table, styles);
        let mut h = 0.0f64;
        for u in units.iter() {
            if u.para_idx >= cp_idx {
                return Some(h);
            }
            h += u.height;
        }
        None
    }

    pub(crate) fn cell_units_content_height(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        self.cell_units(cell, table, styles)
            .iter()
            .map(|unit| unit.height)
            .sum()
    }

    /// [Task #1718] RowBreak 셀에서 용량을 살짝 넘긴 "가시 꼬리줄"에 over-fill grace 를
    /// 줄지 판정한다.
    ///
    /// 원래 grace 조건은 `units[j+1..].any(spacer)` — 뒤 어딘가에 빈 문단 spacer 가
    /// 하나라도 있으면 grace 였다. 이 때문에 654문단 거대 셀(spacer 가 문서 전체에
    /// 흩어져 있음)에서는 연속 본문 한복판에서도 항상 grace 가 걸려 페이지당 +1~5줄
    /// over-fill → under-pagination(승강기 별표27: rhwp 40 vs 한글 48).
    ///
    /// 반대로 `all(spacer)` 로 좁히면 caption 줄 + 개체(그림상자) 앞의 spacer 처럼
    /// 뒤에 가시/개체 유닛이 남아 있는 진짜 구조적 꼬리줄까지 무너뜨린다
    /// (rowbreak-problem-pages 13쪽 회귀).
    ///
    /// 정답 판별: 오버플로 꼬리줄 다음 "첫 spacer 전까지"의 유닛과, spacer 뒤에
    /// 본문이 계속되는지를 함께 본다.
    /// - spacer 가 없다 → 순수 본문 꼬리 → grace 거부.
    /// - 그 사이가 전부 가시 텍스트 줄의 끊김 없는 연속(run) → 본문 한복판 → grace 거부.
    /// - spacer 가 바로 뒤여도 spacer run 뒤에 다시 가시 본문이 이어지면 문단 사이
    ///   빈 줄일 뿐이므로 grace 거부.
    /// - spacer 뒤가 문서/셀 끝이거나, 첫 spacer 전후에 비가시 유닛(개체/중첩/오브젝트
    ///   높이 등)이 끼어 있으면 → 구조적 꼬리줄 → grace 유지.
    fn grace_visible_tail_before_spacer(units: &[CellUnit], j: usize) -> bool {
        let Some(first_spacer) = units[j + 1..].iter().position(|u| u.empty_spacer) else {
            return false;
        };
        if first_spacer > 0 {
            // spacer 전에 비가시 유닛이 끼면 구조적 꼬리로 본다.
            return !units[j + 1..j + 1 + first_spacer]
                .iter()
                .all(|u| u.vis_start < u.vis_end);
        }

        // 오버플로 줄 바로 뒤가 spacer 인 경우에도, spacer run 뒤에 다시 일반 가시 본문이
        // 이어지면 문단 사이 빈 줄이므로 페이지 예산을 넘겨 끌어올리지 않는다.
        let after_spacers = units[j + 1..]
            .iter()
            .position(|u| !u.empty_spacer)
            .map(|idx| j + 1 + idx);
        match after_spacers {
            None => true,
            Some(idx) => {
                let next = &units[idx];
                !(next.vis_start < next.vis_end && !next.mixed_nested_fragment)
            }
        }
    }

    /// RowBreak fragment의 마지막 가시 unit을, 뒤의 구조적 spacer와 함께 현재 조각에
    /// 남길 수 있는지 판정한다. 고정 px 상한 대신 해당 unit의 실제 높이만큼만 overfill을
    /// 허용하므로, 이미 넘친 fragment나 다중 본문 line을 추가로 끌어올리지 않는다.
    fn visible_tail_fits_before_spacer(
        units: &[CellUnit],
        j: usize,
        consumed_height: f64,
        avail_height: f64,
    ) -> bool {
        let Some(tail) = units.get(j) else {
            return false;
        };
        if tail.empty_spacer || tail.vis_start >= tail.vis_end || consumed_height > avail_height {
            return false;
        }
        let overflow = (consumed_height + tail.height - avail_height).max(0.0);
        overflow <= tail.height && Self::grace_visible_tail_before_spacer(units, j)
    }

    /// [#1921] 예산 정지 유닛 `j` 부터 다음 저장 hard-break 유닛까지의 잔여 높이가
    /// 소량(오버플로 한도 48px)이면 `(흡수 후 높이, hard-break 유닛 인덱스)` 를 반환한다.
    ///
    /// 저장 hard-break 는 한글이 실제로 페이지를 넘긴 지점이므로, 그 직전의 극소 잔여
    /// 유닛은 한글 기준으로 현재 페이지에 담겨 있었다. 흡수하지 않으면 다음 fragment 가
    /// 그 잔여만 담은 sliver 페이지(59043 pi=160: 22px/쪽)가 되어 과분할된다.
    fn absorb_tail_before_stored_hard_break(
        units: &[CellUnit],
        j: usize,
        h: f64,
        avail_height: f64,
    ) -> Option<(f64, usize)> {
        const SLIVER_ABSORB_OVERFLOW_TOLERANCE_PX: f64 = 48.0;
        let mut extra = 0.0f64;
        for (k, u) in units.iter().enumerate().skip(j) {
            if k > j && u.hard_break_before {
                return Some((h + extra, k));
            }
            extra += u.height;
            if h + extra > avail_height + SLIVER_ABSORB_OVERFLOW_TOLERANCE_PX {
                return None;
            }
        }
        None
    }

    /// `absorb_tail_before_stored_hard_break`의 더 좁은 변형이다. 일반
    /// hard-break가 아니라 원본 LINE_SEG가 기록한 frame 경계에만 도달한다.
    /// RowBreak의 일반 capacity cut에서 이 tail을 남기면 그 tail만 든 물리
    /// 페이지가 생기므로, 동일한 sliver 정책을 direct row-cut에도 쓴다.
    fn absorb_tail_before_stored_frame_break(
        units: &[CellUnit],
        j: usize,
        h: f64,
        avail_height: f64,
    ) -> Option<(f64, usize)> {
        const SLIVER_ABSORB_OVERFLOW_TOLERANCE_PX: f64 = 48.0;
        let mut extra = 0.0f64;
        for (k, unit) in units.iter().enumerate().skip(j) {
            if k > j && unit.stored_frame_break_before {
                return Some((h + extra, k));
            }
            extra += unit.height;
            if h + extra > avail_height + SLIVER_ABSORB_OVERFLOW_TOLERANCE_PX {
                return None;
            }
        }
        None
    }

    /// [#3931] native HWP5 다행 RowBreak 셀의 저장 page reset 직전에서
    /// paint되지 않는 마지막 줄의 trailing line/paragraph spacing.
    ///
    /// `CellUnit` 전체 높이는 표를 통째로 측정할 때 필요하므로 변경하지 않는다.
    /// 실제 컷이 control-free 문단 경계의 `양수 vpos -> 0 이하`에서 끝날 때만
    /// 이 값을 빼서, 마지막 가시 줄은 현 쪽에 남기고 그 뒤의 공백은 물리 쪽
    /// 경계에서 버린다. control 문단의 로컬 좌표 reset은 물리 경계가 아니다.
    fn native_multirow_saved_reset_trailing_trim(
        &self,
        table: &crate::model::table::Table,
        cell: &crate::model::table::Cell,
        units: &[CellUnit],
        end_cut: usize,
        styles: &ResolvedStyleSet,
    ) -> f64 {
        if !self.profile.get().native_hwp5_layout()
            || table.row_count <= 1
            || table.common.treat_as_char
            || !matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            || !matches!(
                table.common.text_wrap,
                crate::model::shape::TextWrap::TopAndBottom
            )
            || end_cut == 0
            || end_cut >= units.len()
        {
            return 0.0;
        }

        let previous_unit = &units[end_cut - 1];
        let next_unit = &units[end_cut];
        if !next_unit.hard_break_before
            || next_unit.para_idx <= previous_unit.para_idx
            || previous_unit.vis_start >= previous_unit.vis_end
        {
            return 0.0;
        }
        let Some(previous_para) = cell.paragraphs.get(previous_unit.para_idx) else {
            return 0.0;
        };
        let Some(next_para) = cell.paragraphs.get(next_unit.para_idx) else {
            return 0.0;
        };
        if !previous_para.controls.is_empty()
            || !next_para.controls.is_empty()
            || previous_unit.vis_end != previous_para.line_segs.len()
        {
            return 0.0;
        }
        let Some(previous_seg) = previous_para.line_segs.last() else {
            return 0.0;
        };
        let Some(next_seg) = next_para.line_segs.iter().find(|seg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        }) else {
            return 0.0;
        };
        if previous_seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || previous_seg.vertical_pos <= 0
            || next_seg.vertical_pos > 0
        {
            return 0.0;
        }

        let line_spacing = hwpunit_to_px(previous_seg.line_spacing.max(0), self.dpi);
        let paragraph_spacing = styles
            .para_styles
            .get(previous_para.para_shape_id as usize)
            .map(|style| style.spacing_after)
            .unwrap_or(0.0)
            .max(0.0);
        (line_spacing + paragraph_spacing).min(previous_unit.height.max(0.0))
    }

    fn is_non_inline_control_flow_unit(unit: &CellUnit) -> bool {
        unit.vis_start == unit.vis_end
            && !unit.empty_spacer
            && unit.nested_row.is_none()
            && !unit.mixed_nested_fragment
            && !unit.mixed_nested_trailing
            && unit.mixed_nested_content_height <= 0.0
    }

    /// `unit_idx`가 tagged Square/Tight/Through control의 source entry라면, 그
    /// control이 차지하는 마지막 generic unit의 exclusive index를 돌려준다. 같은
    /// 16px fragment가 경계에서 두 control range에 걸칠 수 있으므로, entry가 되는
    /// control 모두의 마지막 unit 중 가장 뒤를 사용한다.
    fn entering_non_inline_control_range_end(units: &[CellUnit], unit_idx: usize) -> Option<usize> {
        let unit = units.get(unit_idx)?;
        let (first_control, last_control) = unit.non_inline_control_range?;
        let mut range_end = None;
        for control_idx in first_control..=last_control {
            let control_start = units.iter().position(|candidate| {
                candidate.para_idx == unit.para_idx
                    && candidate
                        .non_inline_control_range
                        .is_some_and(|(first, last)| first <= control_idx && control_idx <= last)
            });
            if control_start != Some(unit_idx) {
                continue;
            }
            let control_end = units
                .iter()
                .rposition(|candidate| {
                    candidate.para_idx == unit.para_idx
                        && candidate
                            .non_inline_control_range
                            .is_some_and(|(first, last)| {
                                first <= control_idx && control_idx <= last
                            })
                })
                .map(|idx| idx + 1)?;
            range_end = Some(range_end.unwrap_or(0).max(control_end));
        }
        range_end
    }

    fn would_orphan_non_inline_flow_before_spacer(
        units: &[CellUnit],
        j: usize,
        consumed_height: f64,
        avail_height: f64,
    ) -> bool {
        let Some(next) = units.get(j + 1) else {
            return false;
        };
        Self::is_non_inline_control_flow_unit(&units[j])
            && next.empty_spacer
            && !next.hard_break_before
            && consumed_height + units[j].height <= avail_height
            && consumed_height + units[j].height + next.height > avail_height
    }

    fn rewind_rowbreak_fragment_tail_before_topandbottom_flow(
        table: &crate::model::table::Table,
        units: &[CellUnit],
        start: usize,
        avail_height: f64,
        j: &mut usize,
        h: &mut f64,
    ) -> bool {
        if !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) || table.common.treat_as_char
            || *j >= units.len()
            || *j <= start + 1
            || !units[*j].top_and_bottom_flow
        {
            return false;
        }

        let Some(prev_idx) = units[start..*j]
            .iter()
            .rposition(|unit| !unit.empty_spacer)
            .map(|idx| start + idx)
        else {
            return false;
        };
        if prev_idx + 1 < *j
            && !units[prev_idx + 1..*j]
                .iter()
                .all(|unit| unit.empty_spacer && !unit.hard_break_before)
        {
            return false;
        }

        let prev = &units[prev_idx];
        if prev.top_and_bottom_flow || !Self::is_non_inline_control_flow_unit(prev) {
            return false;
        }
        let fragment_run = prev.height <= 16.5
            || (prev_idx > start
                && units[prev_idx - 1].para_idx == prev.para_idx
                && Self::is_non_inline_control_flow_unit(&units[prev_idx - 1])
                && !units[prev_idx - 1].top_and_bottom_flow);
        if !fragment_run {
            return false;
        }

        let rewind_h: f64 = units[prev_idx..*j].iter().map(|unit| unit.height).sum();
        let rewound_h = *h - rewind_h;
        const MAX_REWIND_BLANK_PX: f64 = 96.0;
        let max_rewind_blank = MAX_REWIND_BLANK_PX.max(units[*j].height * 0.4);
        if avail_height - rewound_h > max_rewind_blank {
            return false;
        }
        *h = rewound_h;
        *j = prev_idx;
        true
    }

    /// Mixed 1×1 nested-cell projection에서 완결 중첩 표와 뒤 tail을 fresh page로
    /// 함께 이월한다.
    ///
    /// 상위 `CellUnit`에는 깊은 자식 문단 index가 남지 않는다. 대신 자식 표 뒤의
    /// 첫 실제 unit만 `mixed_nested_starts_after_table` ownership marker를 보존한다.
    /// 표 자체가 현재 쪽에 들어가더라도 출처·설명·뒤 표를 더는 담지 못하면, 표를
    /// 현재 쪽 하단에 소비해 다음 쪽에서 순서가 뒤집힌다.
    fn rewind_rowbreak_mixed_nested_table_tail_for_fresh_page(
        table: &crate::model::table::Table,
        units: &[CellUnit],
        start: usize,
        fresh_page_height: f64,
        j: &mut usize,
        h: &mut f64,
    ) -> bool {
        if !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) || table.common.treat_as_char
            // 이 rewind는 outer RowBreak 표가 아니라, 한 셀짜리 mixed nested
            // projection 자체의 table atom/tail 경계만 다룬다. 다열 본문 표에도
            // marker가 우연히 전파될 수 있는데(21217935: 17×4), 거기서 한 cell의
            // tail을 이월하면 COM 기준 8쪽을 9쪽으로 늘린다.
            || table.row_count != 1
            || table.col_count != 1
            || *j <= start + 1
            || *j >= units.len()
            || !fresh_page_height.is_finite()
            || fresh_page_height <= 0.0
        {
            return false;
        }

        let Some(after_table) = units[start..*j]
            .iter()
            .rposition(|unit| unit.mixed_nested_fragment && unit.mixed_nested_starts_after_table)
            .map(|index| start + index)
        else {
            return false;
        };
        let Some(after_source_para) = units[after_table].mixed_nested_source_para_idx else {
            return false;
        };
        let Some(last_table_atom) = (start..after_table).rev().find(|&index| {
            let unit = &units[index];
            unit.mixed_nested_fragment
                && !unit.mixed_nested_trailing
                && unit.mixed_nested_content_height > 0.5
                && unit
                    .mixed_nested_source_para_idx
                    .is_some_and(|source_para| source_para != after_source_para)
        }) else {
            return false;
        };
        let Some(table_source_para) = units[last_table_atom].mixed_nested_source_para_idx else {
            return false;
        };
        let mut table_atom = last_table_atom;
        while table_atom > start
            && units[table_atom - 1].mixed_nested_fragment
            && units[table_atom - 1].mixed_nested_source_para_idx == Some(table_source_para)
        {
            table_atom -= 1;
        }
        let Some(end_before_table) = table_atom.checked_sub(1).filter(|end| *end > start) else {
            return false;
        };
        let tail = &units[table_atom..];
        let tail_height: f64 = tail.iter().map(|unit| unit.height).sum();
        if tail.iter().any(|unit| unit.hard_break_before) || tail_height > fresh_page_height + 0.5 {
            return false;
        }
        *h = units[start..=end_before_table]
            .iter()
            .map(|unit| unit.height)
            .sum();
        *j = end_before_table;
        true
    }

    fn should_absorb_midpage_saved_vpos_reset(
        &self,
        table: &crate::model::table::Table,
        unit: &CellUnit,
        consumed_height: f64,
        avail_height: f64,
        allow_midpage_reset_absorb: bool,
    ) -> bool {
        // RowBreak 셀에는 한컴 저장 LINE_SEG vertical_pos 리셋이 남아 있다.
        // 대부분은 쪽 경계 근처의 저장 페이지 경계로 보존해야 하지만, 현재 조각이
        // 페이지 절반도 채우지 못한 중간 리셋은 같은 쪽 안의 로컬 좌표 재시작으로
        // 보는 편이 기준 PDF와 맞다. 파일명/쪽번호가 아니라 저장 위치와 현재 예산에
        // 근거해 구분한다.
        allow_midpage_reset_absorb
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && !unit.empty_spacer
            && unit.vis_start < unit.vis_end
            && avail_height.is_finite()
            && avail_height > 0.0
            && (avail_height - consumed_height) > avail_height * 0.5
    }

    /// [Task #993] 분할 표 행 컷을 전진시킨다 — 분할 표 페이지네이션의 단일 권위 함수.
    ///
    /// `start_cut`(이전 페이지까지 셀별 소비 유닛 수)에서 시작해, 각 셀을 공통
    /// 높이 예산 `avail_height` 안에서 동시 전진시킨다. 어느 유닛도 `avail_height`
    /// 안에 안 들어가면 진행 보장을 위해 셀당 유닛 1개는 강제 소비한다. vpos
    /// 리셋(hard break)을 만나면 그 셀은 거기서 정지한다.
    ///
    /// 페이지네이터(분할 판정)와 렌더러(가시 범위)가 모두 이 함수를 호출하므로
    /// 두 경로의 컷이 정의상 일치한다.
    pub(crate) fn advance_row_cut(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        avail_height: f64,
        styles: &ResolvedStyleSet,
    ) -> RowCutResult {
        let issue2424_started = issue2424_profile_enabled().then(std::time::Instant::now);
        let result = self.advance_row_cut_inner(table, row, start_cut, avail_height, styles);
        if let Some(started) = issue2424_started {
            use std::sync::atomic::Ordering::Relaxed;
            ISSUE2424_ADVANCE_ROW_CUT_CALLS.fetch_add(1, Relaxed);
            ISSUE2424_ADVANCE_ROW_CUT_NANOS.fetch_add(started.elapsed().as_nanos() as u64, Relaxed);
        }
        result
    }

    /// Return the complete source-owned frame beginning at `start_cut` when it
    /// ends at an explicit stored vpos-frame reset.  Unlike a numeric overflow
    /// allowance, this exposes the exact CellUnit boundary that the source
    /// recorded for the current physical fragment.
    pub(crate) fn stored_frame_cut_for_row(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<RowCutResult> {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .collect();
        row_cells.sort_by_key(|cell| cell.col);
        if row_cells.is_empty() {
            return None;
        }

        // Do not use `advance_row_cut(f64::MAX)` here.  Its normal orphan
        // protection may intentionally rewind a unit *before* the stored
        // boundary, which is correct for a capacity cut but not when asking
        // for the source frame itself.
        let frame_height = row_cells
            .iter()
            .enumerate()
            .filter_map(|(cell_idx, cell)| {
                let units = self.cell_units(cell, table, styles);
                let start = start_cut
                    .get(cell_idx)
                    .copied()
                    .unwrap_or(0)
                    .min(units.len());
                units
                    .iter()
                    .enumerate()
                    .skip(start + 1)
                    .find(|(_, unit)| unit.stored_frame_break_before)
                    .map(|(end, _)| {
                        units[start..end]
                            .iter()
                            .map(|unit| unit.height)
                            .sum::<f64>()
                    })
            })
            .filter(|height| *height > 0.5)
            .reduce(f64::min)?;

        let mut end_cut = Vec::with_capacity(row_cells.len());
        let mut consumed_height = 0.0f64;
        let mut fully_consumed = true;
        for (cell_idx, cell) in row_cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut
                .get(cell_idx)
                .copied()
                .unwrap_or(0)
                .min(units.len());
            let mut end = start;
            let mut height = 0.0f64;
            while end < units.len()
                && !units[end].stored_frame_break_before
                && (height <= 0.5 || height + units[end].height <= frame_height + 0.5)
            {
                height += units[end].height;
                end += 1;
            }
            fully_consumed &= end == units.len();
            consumed_height = consumed_height.max(height);
            end_cut.push(end);
        }

        (!fully_consumed && consumed_height > 0.5).then_some(RowCutResult {
            end_cut,
            hit_hard_break: true,
            fully_consumed,
            consumed_height,
        })
    }

    /// Extend an existing row cut to the end of the omitted source paragraph.
    ///
    /// This is deliberately narrower than a row-height allowance: it only
    /// consumes the visible paragraph already selected by the normal cut and
    /// never crosses a stored frame reset.  Callers use it for a terminal
    /// response followed by a source-empty spacer, where moving a short
    /// paragraph suffix alone would otherwise create a tail-only fragment.
    pub(crate) fn paragraph_tail_cut_for_row(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        base_end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<RowCutResult> {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .collect();
        row_cells.sort_by_key(|cell| cell.col);
        if row_cells.is_empty() {
            return None;
        }

        let mut end_cut = Vec::with_capacity(row_cells.len());
        let mut consumed_height = 0.0f64;
        let mut fully_consumed = true;
        let mut extended = false;
        for (cell_idx, cell) in row_cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut
                .get(cell_idx)
                .copied()
                .unwrap_or(0)
                .min(units.len());
            let mut end = base_end_cut
                .get(cell_idx)
                .copied()
                .unwrap_or(start)
                .clamp(start, units.len());
            if let Some(first_omitted) = units.get(end) {
                if !first_omitted.empty_spacer
                    && first_omitted.vis_start < first_omitted.vis_end
                    && !first_omitted.stored_frame_break_before
                {
                    let para_idx = first_omitted.para_idx;
                    while let Some(unit) = units.get(end) {
                        if unit.para_idx != para_idx || unit.stored_frame_break_before {
                            break;
                        }
                        end += 1;
                    }
                    extended |= end > base_end_cut.get(cell_idx).copied().unwrap_or(start);
                }
            }
            fully_consumed &= end == units.len();
            consumed_height =
                consumed_height.max(units[start..end].iter().map(|unit| unit.height).sum());
            end_cut.push(end);
        }

        extended.then_some(RowCutResult {
            end_cut,
            hit_hard_break: false,
            fully_consumed,
            consumed_height,
        })
    }

    /// Extend a row cut by exactly the next visible source unit in the cell
    /// that owns a stored frame reset, or in a single-cell continuation.
    ///
    /// This is narrower than `paragraph_tail_cut_for_row`: a stored frame can
    /// own one response line without owning the remainder of that paragraph.
    /// The returned height is therefore the measured line unit, not a
    /// fixture-specific pixel allowance.
    pub(crate) fn next_visible_unit_cut_for_row(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        base_end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<RowCutResult> {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .collect();
        row_cells.sort_by_key(|cell| cell.col);
        if row_cells.is_empty() {
            return None;
        }
        let single_cell_row = row_cells.len() == 1;

        let mut end_cut = Vec::with_capacity(row_cells.len());
        let mut consumed_height = 0.0f64;
        let mut fully_consumed = true;
        let mut extended = false;
        for (cell_idx, cell) in row_cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut
                .get(cell_idx)
                .copied()
                .unwrap_or(0)
                .min(units.len());
            let mut end = base_end_cut
                .get(cell_idx)
                .copied()
                .unwrap_or(start)
                .clamp(start, units.len());
            let owns_stored_frame_reset = cell.paragraphs.iter().any(|paragraph| {
                paragraph
                    .line_segs
                    .iter()
                    .skip(1)
                    .any(|segment| segment.vertical_pos == 0)
            });
            if let Some(first_omitted) = units.get(end) {
                if (owns_stored_frame_reset || single_cell_row)
                    && !first_omitted.empty_spacer
                    && first_omitted.vis_start < first_omitted.vis_end
                    && !first_omitted.stored_frame_break_before
                {
                    end += 1;
                    extended = true;
                }
            }
            fully_consumed &= end == units.len();
            consumed_height =
                consumed_height.max(units[start..end].iter().map(|unit| unit.height).sum());
            end_cut.push(end);
        }

        extended.then_some(RowCutResult {
            end_cut,
            hit_hard_break: false,
            fully_consumed,
            consumed_height,
        })
    }

    /// Return whether a physical table row contains only source-empty spacer
    /// units.  Text/control inspection alone is insufficient here: imported
    /// HWPX may retain structural controls in a row that has no line or atom
    /// to paint.
    pub(crate) fn row_has_only_empty_spacer_units(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        let mut row_cells = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .peekable();
        row_cells.peek().is_some()
            && row_cells.all(|cell| {
                self.cell_units(cell, table, styles)
                    .iter()
                    .all(|unit| unit.empty_spacer)
            })
    }

    /// Return whether exactly one physical-row cell owns visible source
    /// content.  Direct HWPX RowBreak tables use the opposite empty cell as a
    /// structural band; a stored frame rewind in the sole visible cell must
    /// therefore not be erased by the whole-row fast path.
    pub(crate) fn row_has_single_visible_source_cell(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .filter(|cell| {
                self.cell_units(cell, table, styles)
                    .iter()
                    .any(|unit| !unit.empty_spacer && unit.vis_start < unit.vis_end)
            })
            .count()
            == 1
    }

    /// Direct HWPX RowBreak cell의 reset이 선언된 cell box 안에서 source frame을
    /// 완결하는지 판별한다. reset 뒤의 저장 lineSeg가 선언 높이를 계속 넘으면
    /// writer-local cursor일 뿐, 물리 fragment owner가 아니다.
    fn direct_hwpx_cell_has_declared_stored_frame(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
    ) -> bool {
        let profile = self.profile.get();
        if !profile.hwpx_stored_layout()
            || profile.hwp5_origin_hwpx()
            || table.common.treat_as_char
            || !matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            || cell.height >= 0x8000_0000
            || cell.paragraphs.iter().any(|paragraph| {
                paragraph
                    .controls
                    .iter()
                    .any(|control| matches!(control, Control::Table(_)))
            })
        {
            return false;
        }

        let mut reset_count = 0usize;
        let mut previous_frame_end: Option<i32> = None;
        let mut preceding_frame_end = 0i32;
        let mut trailing_frame_end = 0i32;
        for paragraph in &cell.paragraphs {
            for seg in paragraph.line_segs.iter().filter(|seg| {
                seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            }) {
                let frame_end = seg.vertical_pos.saturating_add(seg.line_height);
                if previous_frame_end
                    .is_some_and(|previous_end| previous_end > 0 && seg.vertical_pos <= 0)
                {
                    reset_count += 1;
                    preceding_frame_end =
                        preceding_frame_end.max(previous_frame_end.unwrap_or_default());
                    trailing_frame_end = 0;
                }
                if reset_count > 0 {
                    trailing_frame_end = trailing_frame_end.max(frame_end);
                }
                previous_frame_end = Some(frame_end);
            }
        }

        reset_count >= 2
            || (reset_count == 1 && {
                let source_frame_span = preceding_frame_end.saturating_add(trailing_frame_end);
                let declared_height = cell.height as i32;
                source_frame_span >= declared_height.saturating_mul(4) / 5
                    && source_frame_span <= declared_height
            })
    }

    /// Return whether a row records an in-paragraph return from a positive
    /// stored vertical position to the top of a new physical frame.  This is
    /// source pagination data, not a measured-height heuristic.
    pub(crate) fn row_has_stored_vpos_frame_rewind(
        &self,
        table: &crate::model::table::Table,
        row: usize,
    ) -> bool {
        let profile = self.profile.get();
        // Direct HWPX의 physical frame reset은 문단 내부뿐 아니라 문단 사이에도
        // 저장된다. 이 profile에서는 raw in-paragraph window를 다시 검사하지 않고,
        // 선언 cell 안에서 source frame이 완결되는 동일 predicate를 사용한다.
        // 따라서 writer-local single reset은 Stage 227 수용성 검사에서 계속 제외된다.
        if profile.hwpx_stored_layout() && !profile.hwp5_origin_hwpx() {
            return table
                .cells
                .iter()
                .filter(|cell| cell.row as usize == row && cell.row_span == 1)
                .any(|cell| self.direct_hwpx_cell_has_declared_stored_frame(cell, table));
        }

        let has_raw_rewind = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .flat_map(|cell| &cell.paragraphs)
            .any(|paragraph| {
                paragraph
                    .line_segs
                    .windows(2)
                    .any(|lines| lines[0].vertical_pos > 0 && lines[1].vertical_pos == 0)
            });
        if !has_raw_rewind {
            return false;
        }

        has_raw_rewind
    }

    /// Return the painted height of a response cell whose stored source frame
    /// consists of exactly two lines.  This includes the cell's resolved
    /// vertical padding, so callers can compare it directly with a row-cut
    /// content budget without a document-specific pixel allowance.
    pub(crate) fn row_two_line_source_frame_height(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<f64> {
        table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .filter(|cell| {
                cell.paragraphs.iter().any(|paragraph| {
                    paragraph.line_segs.len() == 2
                        && paragraph.line_segs[0].vertical_pos == 0
                        && paragraph.line_segs[1].vertical_pos > 0
                })
            })
            .map(|cell| {
                let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
                let visible_units_height: f64 = self
                    .cell_units(cell, table, styles)
                    .iter()
                    .filter(|unit| !unit.empty_spacer && unit.vis_start < unit.vis_end)
                    .map(|unit| unit.height)
                    .sum();
                visible_units_height + pad_top + pad_bottom
            })
            .filter(|height| *height > 0.5)
            .reduce(f64::max)
    }

    fn advance_row_cut_inner(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        avail_height: f64,
        styles: &ResolvedStyleSet,
    ) -> RowCutResult {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|c| c.row as usize == row && c.row_span == 1)
            .collect();
        row_cells.sort_by_key(|c| c.col);

        let mut end_cut: RowCut = Vec::with_capacity(row_cells.len());
        let mut hit_hard_break = false;
        let mut fully_consumed = true;
        let mut consumed_height = 0.0f64;
        const HARD_BREAK_REMAINING_TOLERANCE_PX: f64 = 32.0;
        let row_has_top_and_bottom_flow = row_cells
            .iter()
            .any(|cell| self.cell_has_top_and_bottom_non_inline_flow(cell));
        // [#3820 Stage 11] native HWP5의 1×1 TopAndBottom RowBreak float는 셀 문단
        // 경계의 `양수 vpos -> 0`을 실제 물리 쪽 전환으로 저장하는 경우가 있다.
        // 일반 1~2열 RowBreak 표에 적용하는 relaxed rule은 이 reset을 "공간이 남은
        // 로컬 재시작"으로 흡수하지만, p172의 `<OPTN>` 뒤 `간 특수 검사`처럼 다음
        // fragment의 내용을 기존 FootnoteArea 위에 과적재한다. 표 내부의 같은 문단
        // 줄 reset과 달리 **문단 경계** reset이고, native HWP5·empty-host float가
        // 갖는 1×1 저장 형상일 때만 relaxed rule에서 제외한다.
        let native_hwp5_single_cell_topbottom_cross_para_reset = self
            .profile
            .get()
            .native_hwp5_layout()
            && !table.common.treat_as_char
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && matches!(
                table.common.text_wrap,
                crate::model::shape::TextWrap::TopAndBottom
            )
            && table.row_count == 1
            && table.col_count == 1
            && row_cells.len() == 1
            && row_cells[0].paragraphs.windows(2).any(|pair| {
                match (
                    pair[0].line_segs.iter().rev().find(|seg| {
                        seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    }),
                    pair[1].line_segs.iter().find(|seg| {
                        seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    }),
                ) {
                    (Some(previous), Some(current)) => {
                        previous.vertical_pos > 0 && current.vertical_pos <= 0
                    }
                    _ => false,
                }
            });
        let relaxed_hard_break = matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) && (table.col_count <= 2 || table.row_count > 5)
            && !row_has_top_and_bottom_flow
            && !native_hwp5_single_cell_topbottom_cross_para_reset;
        let allow_midpage_reset_absorb =
            self.profile.get().hwpx_stored_layout() || row_has_top_and_bottom_flow;
        let rewind_internal_hard_break_orphan = Self::row_has_prior_rowspan_cover(table, row);
        let native_hwp5_atomic_non_inline_entry = self.profile.get().native_hwp5_layout()
            && matches!(
                table.page_break,
                crate::model::table::TablePageBreak::RowBreak
            )
            && !table.common.treat_as_char;
        for (i, cell) in row_cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let mut j = start;
            let mut h = 0.0f64;
            while j < units.len() {
                let u = &units[j];
                // 시작 유닛(j==start)은 항상 소비 — 진행 보장.
                if start > 0
                    && u.empty_spacer
                    && !u.hard_break_before
                    && units[start..=j].iter().all(|unit| unit.empty_spacer)
                {
                    j += 1;
                    continue;
                }
                if start > 0
                    && u.empty_spacer
                    && !u.hard_break_before
                    && units[j..]
                        .iter()
                        .all(|unit| unit.empty_spacer && !unit.hard_break_before)
                {
                    j = units.len();
                    break;
                }
                // CellUnit fragment는 Square/Tight/Through control의 source range를
                // 나눌 수 있지만 renderer는 entry fragment에서 picture 전체를 한 번
                // emit한다. native HWP5 RowBreak에서 entry만 넣으면 cell clip이 picture
                // 를 자르고 continuation은 owner가 없어지는 반쪽 control이 된다. 이미
                // content를 소비한 page에서는 range 전체가 fit할 때만 시작한다. page보다
                // 큰 control의 fresh fragment(start/h==0)는 기존 progress 경로를 보존한다.
                if native_hwp5_atomic_non_inline_entry && h > 0.5 {
                    if let Some(control_end) =
                        Self::entering_non_inline_control_range_end(&units, j)
                    {
                        let control_height: f64 =
                            units[j..control_end].iter().map(|unit| unit.height).sum();
                        if h + control_height > avail_height + 0.5 {
                            break;
                        }
                    }
                }
                // [Task #1658] 미세 fragment 낭비 페이지 방지: 거대 셀이 페이지를 가로질러 분할될
                // 때 셀 내용 vpos reset(hard_break_before)이 촘촘하면, 잔여공간이 충분한데도 reset 마다
                // 페이지를 끊어 2줄 이하만 담은 낭비 페이지가 양산된다(법령 별표 거대 셀:
                // 별표1 5→4쪽, 산업통상부 별표4 33→27쪽). 흡수 임계: continuation(start>0, 셀 중간
                // 조각)은 ≤3 유닛, fresh(start==0)는 ≤2 유닛. continuation 의 reset 은 셀 내부
                // page-wrap 인데 rhwp 가 한글 break 보다 1~3줄 일찍 capacity-break 하여 reset 직전
                // 1~3줄 orphan 을 만든다(한글 COM 대조: 한글 break @line 5/40/75 vs rhwp 3·6/74·76).
                // fresh 의 ≤2 는 #1488(가시 문단 사이 reset 3유닛 후 보존)을 깨지 않도록 유지한다.
                let waste_thresh = if start > 0 { 3 } else { 2 };
                let tiny_fragment_waste = j <= start + waste_thresh
                    && !u.empty_spacer
                    && h + u.height <= avail_height
                    && avail_height - h > HARD_BREAK_REMAINING_TOLERANCE_PX;
                // HWPX 저장 reset은 문단 내부에서 양수 vpos가 0으로 되감기고,
                // 그 행의 유일한 가시 source cell이 그 reset을 소유할 때만 물리
                // 조각 경계 권한을 갖는다. 여러 가시 셀 중 하나의 reset은 셀 로컬
                // cursor이므로 행 전체를 앞당겨 자르지 않고 일반 capacity cut에
                // 맡긴다. 한 셀에 저장 reset이 여러 개인 continuation stream은
                // 기존처럼 mid-page absorb 판정이 행과 잔여 공간을 함께 검증한다.
                let follows_single_cell_nested_host = u
                    .para_idx
                    .checked_sub(1)
                    .and_then(|para_idx| cell.paragraphs.get(para_idx))
                    .is_some_and(Self::paragraph_hosts_single_cell_nested_table);
                let hwpx_local_reset_stream = self.profile.get().hwpx_stored_layout()
                    && !table.common.treat_as_char
                    && matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    )
                    && table.row_count == 1
                    && table.col_count == 1
                    && row_cells.len() == 1
                    && start > 0
                    && units
                        .iter()
                        .filter(|unit| unit.stored_frame_break_before)
                        .nth(1)
                        .is_some();
                let strict_saved_frame_break = u.stored_frame_break_before
                    && (u.mixed_nested_recursive
                        || follows_single_cell_nested_host
                        || (self.profile.get().hwpx_stored_layout()
                            && self.row_has_stored_vpos_frame_rewind(table, row)
                            && self.row_has_single_visible_source_cell(table, row, styles)
                            && !hwpx_local_reset_stream));
                if j > start
                    && u.hard_break_before
                    && (strict_saved_frame_break
                        || ((rewind_internal_hard_break_orphan
                            || !relaxed_hard_break
                            || (!u.empty_spacer
                                && (h + u.height > avail_height
                                    || avail_height - h <= HARD_BREAK_REMAINING_TOLERANCE_PX)))
                            && !units[start..j].iter().all(|unit| unit.empty_spacer)
                            && !tiny_fragment_waste))
                {
                    if !strict_saved_frame_break
                        && self.should_absorb_midpage_saved_vpos_reset(
                            table,
                            u,
                            h,
                            avail_height,
                            allow_midpage_reset_absorb,
                        )
                    {
                        h += u.height;
                        j += 1;
                        continue;
                    }
                    if rewind_internal_hard_break_orphan {
                        Self::rewind_rowbreak_orphan_before_hard_break(
                            table,
                            &units,
                            start,
                            avail_height,
                            rewind_internal_hard_break_orphan,
                            &mut j,
                            &mut h,
                        );
                    }
                    hit_hard_break = true;
                    break;
                }
                if j > start && h + u.height > avail_height {
                    // [#3931] 저장 reset 직전 마지막 가시 줄의 trailing 공백을
                    // 물리 쪽 경계에서 제외하면 예산에 들어가는 경우, 그 줄까지
                    // 현 조각에 넣고 다음 문단 hard break 직전에서 멈춘다. source
                    // frame tail 흡수보다 먼저 적용해 본문 하단 침범을 피한다.
                    let trailing_trim = self.native_multirow_saved_reset_trailing_trim(
                        table,
                        cell,
                        &units,
                        j + 1,
                        styles,
                    );
                    if trailing_trim > 0.0 && h + u.height - trailing_trim <= avail_height + 0.5 {
                        h += (u.height - trailing_trim).max(0.0);
                        j += 1;
                        hit_hard_break = true;
                        break;
                    }

                    // source frame tail은 원본 LINE_SEG가 직접 소유한다. 셀 텍스트
                    // 편집 뒤에는 reflow suffix가 같은 tag/metrics를 계승할 수 있어
                    // line segment만으로는 원본과 구별되지 않는다. 편집 관문이 남긴
                    // provenance가 있을 때는 일반 capacity cut으로 새 줄을 분할한다.
                    if self.profile.get().native_hwp5_layout() && !table.text_reflowed_after_edit {
                        if let Some((absorbed_h, absorbed_j)) =
                            Self::absorb_tail_before_stored_frame_break(&units, j, h, avail_height)
                        {
                            h = absorbed_h;
                            j = absorbed_j;
                            hit_hard_break = true;
                            break;
                        }
                    }
                    let visible_tail_before_spacer = relaxed_hard_break
                        && Self::visible_tail_fits_before_spacer(&units, j, h, avail_height);
                    if visible_tail_before_spacer {
                        h += u.height;
                        j += 1;
                        continue;
                    }
                    if Self::rewind_rowbreak_mixed_nested_table_tail_for_fresh_page(
                        table,
                        &units,
                        start,
                        self.current_body_area.get().3.max(avail_height),
                        &mut j,
                        &mut h,
                    ) {
                        break;
                    }
                    // [#1921] sliver 흡수는 with_row_offsets 경로에만 적용한다. 이 walk 는
                    // relaxed_hard_break(hard-break 조건부 무시) 의미론이라 다음 break 로의
                    // 흡수가 비정상 경계를 강제한다(86712 공식PDF 65→66 회귀 실증).
                    break;
                }
                if j > start
                    && Self::would_orphan_non_inline_flow_before_spacer(&units, j, h, avail_height)
                {
                    // TopAndBottom 개체만 쪽 하단에 남기고 뒤 spacer 를 다음 쪽으로 보내면
                    // 기준 렌더러와 달리 그림이 한 쪽 앞당겨진다. 개체+spacer 묶음이 함께
                    // 들어가지 못할 때는 개체 유닛부터 다음 조각에서 시작하게 한다.
                    break;
                }
                h += u.height;
                j += 1;
            }
            if j < units.len()
                && Self::rewind_rowbreak_orphan_heading_before_recursive_block(
                    table,
                    &units,
                    start,
                    avail_height,
                    &mut j,
                    &mut h,
                )
            {
                // 짧은 제목만 현재 쪽에 남고 뒤의 page-scale 재귀 block이 다음 쪽으로
                // 넘어가는 경우, 제목도 같은 source block의 첫 unit으로 넘긴다.
            }
            if j < units.len()
                && Self::rewind_rowbreak_fragment_tail_before_topandbottom_flow(
                    table,
                    &units,
                    start,
                    avail_height,
                    &mut j,
                    &mut h,
                )
            {
                // 뒤 TopAndBottom 개체 앞의 텍스트박스 꼬리 fragment 를 다음 조각에
                // 남겨 continuation 에서 선행 설명 박스가 사라지지 않게 한다.
            }
            if j < units.len()
                && units[j..].iter().any(|unit| unit.hard_break_before)
                && Self::rewind_rowbreak_tail_before_pending_hard_break(
                    table,
                    &units,
                    start,
                    avail_height,
                    &mut j,
                    &mut h,
                )
            {
                hit_hard_break = true;
            }
            if j < units.len() {
                fully_consumed = false;
            }
            if h > consumed_height {
                consumed_height = h;
            }
            end_cut.push(j);
        }
        RowCutResult {
            end_cut,
            hit_hard_break,
            fully_consumed,
            consumed_height,
        }
    }

    /// [Task #1025] 행블록 컷 — rowspan(rs>1) 셀로 묶인 연속 행 블록 `[b_start, b_end)`
    /// 의 셀을 `(row, col)` 안정 순서로 순회하며 CellUnit(줄/중첩 atom) 단위로 진행한다.
    /// `advance_row_cut` 의 블록 일반화: 블록을 걸친 rs>1 셀 + 블록 내 각 행의 셀을 모두
    /// 포함한다. rs>1 라벨 셀은 첫 조각(start_cut 비었을 때)에서 전량 소비되고, 연속
    /// 조각에선 시작 인덱스가 이미 끝이라 0 유닛 진행 → 렌더 공란(한컴 정답).
    /// 거대 `row_span==1` 셀은 줄 단위로 페이지 경계까지 채우고 잔여를 다음 조각으로 넘긴다.
    ///
    /// 셀 순서·인덱스는 `row_block_content_height` / 렌더러와 공유하는 단일 정의다.
    /// 단일 비-rowspan 행(`b_end==b_start+1`, 블록 내 rs>1 셀 없음)에서는
    /// `advance_row_cut` 과 동일 결과를 낸다(회귀 0).
    pub(crate) fn advance_row_block_cut(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        start_cut: &[usize],
        avail_height: f64,
        styles: &ResolvedStyleSet,
    ) -> RowCutResult {
        let mut cells = Self::row_block_cells(table, b_start, b_end);
        // 안정 순서: (row, col) 오름차순.
        cells.sort_by_key(|c| (c.row, c.col));

        let mut end_cut: RowCut = Vec::with_capacity(cells.len());
        let mut hit_hard_break = false;
        let mut fully_consumed = true;
        let mut consumed_height = 0.0f64;
        const HARD_BREAK_REMAINING_TOLERANCE_PX: f64 = 32.0;
        let block_has_top_and_bottom_flow = cells
            .iter()
            .any(|cell| self.cell_has_top_and_bottom_non_inline_flow(cell));
        let relaxed_hard_break = matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) && (table.col_count <= 2 || table.row_count > 5)
            && !block_has_top_and_bottom_flow;
        let allow_midpage_reset_absorb =
            self.profile.get().hwpx_stored_layout() || block_has_top_and_bottom_flow;
        for (i, cell) in cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let mut j = start;
            let mut h = 0.0f64;
            while j < units.len() {
                let u = &units[j];
                // 시작 유닛(j==start)은 항상 소비 — 진행 보장.
                if start > 0
                    && u.empty_spacer
                    && !u.hard_break_before
                    && units[start..=j].iter().all(|unit| unit.empty_spacer)
                {
                    j += 1;
                    continue;
                }
                if start > 0
                    && u.empty_spacer
                    && !u.hard_break_before
                    && units[j..]
                        .iter()
                        .all(|unit| unit.empty_spacer && !unit.hard_break_before)
                {
                    j = units.len();
                    break;
                }
                let strict_saved_frame_break = u.stored_frame_break_before;
                if j > start
                    && u.hard_break_before
                    && (strict_saved_frame_break
                        || ((!relaxed_hard_break
                            || (!u.empty_spacer
                                && (h + u.height > avail_height
                                    || avail_height - h <= HARD_BREAK_REMAINING_TOLERANCE_PX)))
                            && !units[start..j].iter().all(|unit| unit.empty_spacer)))
                {
                    if !strict_saved_frame_break
                        && self.should_absorb_midpage_saved_vpos_reset(
                            table,
                            u,
                            h,
                            avail_height,
                            allow_midpage_reset_absorb,
                        )
                    {
                        h += u.height;
                        j += 1;
                        continue;
                    }
                    Self::rewind_rowbreak_orphan_before_hard_break(
                        table,
                        &units,
                        start,
                        avail_height,
                        false,
                        &mut j,
                        &mut h,
                    );
                    hit_hard_break = true;
                    break;
                }
                if j > start && h + u.height > avail_height {
                    let visible_tail_before_spacer = relaxed_hard_break
                        && Self::visible_tail_fits_before_spacer(&units, j, h, avail_height);
                    if visible_tail_before_spacer {
                        h += u.height;
                        j += 1;
                        continue;
                    }
                    if Self::rewind_rowbreak_mixed_nested_table_tail_for_fresh_page(
                        table,
                        &units,
                        start,
                        self.current_body_area.get().3.max(avail_height),
                        &mut j,
                        &mut h,
                    ) {
                        break;
                    }
                    // [#1921] sliver 흡수는 with_row_offsets 경로에만 적용한다. 이 walk 는
                    // relaxed_hard_break(hard-break 조건부 무시) 의미론이라 다음 break 로의
                    // 흡수가 비정상 경계를 강제한다(86712 공식PDF 65→66 회귀 실증).
                    break;
                }
                if j > start
                    && Self::would_orphan_non_inline_flow_before_spacer(&units, j, h, avail_height)
                {
                    // `advance_row_cut` 과 같은 CellUnit 구조 판정이다. 행블록 컷에서도
                    // TopAndBottom 개체 유닛이 뒤 spacer 와 분리되어 고립되지 않게 한다.
                    break;
                }
                h += u.height;
                j += 1;
            }
            if j < units.len()
                && Self::rewind_rowbreak_fragment_tail_before_topandbottom_flow(
                    table,
                    &units,
                    start,
                    avail_height,
                    &mut j,
                    &mut h,
                )
            {
                // `advance_row_cut` 과 같은 후처리다.
            }
            if j < units.len()
                && units[j..].iter().any(|unit| unit.hard_break_before)
                && Self::rewind_rowbreak_tail_before_pending_hard_break(
                    table,
                    &units,
                    start,
                    avail_height,
                    &mut j,
                    &mut h,
                )
            {
                hit_hard_break = true;
            }
            if j < units.len() {
                fully_consumed = false;
            }
            if h > consumed_height {
                consumed_height = h;
            }
            // [#2097 진단] 셀별 walk 결과 — 동작 불변.
            if std::env::var("RHWP_DIAG_BLKCUT").is_ok() {
                let stop = if j >= units.len() {
                    "end"
                } else if units[j].hard_break_before {
                    "hard"
                } else {
                    "budget"
                };
                eprintln!(
                    "DIAG_BLKCUT cell[{}] r={} c={} units={} start={} j={} h={:.1} stop={} next_h={:.1}",
                    i,
                    cell.row,
                    cell.col,
                    units.len(),
                    start,
                    j,
                    h,
                    stop,
                    units.get(j).map(|u| u.height).unwrap_or(0.0)
                );
            }
            end_cut.push(j);
        }
        RowCutResult {
            end_cut,
            hit_hard_break,
            fully_consumed,
            consumed_height,
        }
    }

    /// RowBreak rowspan 블록에서 셀의 행 시작 y를 반영해 컷을 전진시킨다.
    ///
    /// 일반 `advance_row_block_cut`은 블록 안의 모든 셀에 같은 예산을 주기 때문에,
    /// 위쪽 큰 셀이 페이지 경계에서 잘릴 때 아래 행의 짧은 셀까지 먼저 소비할 수 있다.
    /// 이 함수는 행별 top offset을 빼고 남은 예산으로 셀을 전진시켜 같은 블록 안의
    /// 아래 행 내용이 한컴처럼 다음 조각에 남도록 한다.
    pub(crate) fn advance_row_block_cut_with_row_offsets(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        start_cut: &[usize],
        avail_height: f64,
        row_offsets: &[f64],
        styles: &ResolvedStyleSet,
    ) -> RowCutResult {
        let mut cells = Self::row_block_cells(table, b_start, b_end);
        cells.sort_by_key(|c| (c.row, c.col));

        let mut end_cut: RowCut = Vec::with_capacity(cells.len());
        let mut hit_hard_break = false;
        let mut fully_consumed = true;
        let mut consumed_height = 0.0f64;
        // [#2291] plain 블록 walk(advance_row_block_cut)와 동일한 relaxed_hard_break
        // 의미론 — 한글 2022 는 재개방 시 저장 vpos-reset(원저작 쪽나눔 흔적)을
        // 무시하고 fresh 재배치로 쪽을 만충한다(연결맵 p26 = 81줄 실측, #2291).
        // 종전 이 walk 는 hard-break 에서 무조건 정지해 예산 잔여(≤52px)를 버리고
        // 조각 경계가 한글과 어긋났다.
        const HARD_BREAK_REMAINING_TOLERANCE_PX: f64 = 32.0;
        let block_has_top_and_bottom_flow = cells
            .iter()
            .any(|cell| self.cell_has_top_and_bottom_non_inline_flow(cell));
        let relaxed_hard_break = matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) && (table.col_count <= 2 || table.row_count > 5)
            && !block_has_top_and_bottom_flow;
        for (i, cell) in cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let cell_row = cell.row as usize;
            let row_offset = cell_row
                .checked_sub(b_start)
                .and_then(|idx| row_offsets.get(idx))
                .copied()
                .unwrap_or(0.0);
            let cell_budget = (avail_height - row_offset).max(0.0);
            let allow_force_progress = row_offset <= 0.5;
            let mut j = start;
            let mut h = 0.0f64;
            // [#2287/PR #2290 P1] 연속 조각(start>0)이 시작 직후(start+1) 저장
            // hard-break 를 만나면, start 유닛은 직전 조각의 orphan-rewind 가
            // 이월시킨 고아다 — 여기서 hard 를 쪽 경계로 존중하면 고아 혼자
            // 한 쪽(교육부 47×9 p26: 유닛 1개 17.3px sliver)이 되어 rewind 의
            // 의도(고아 방지)와 정반대가 된다. 극소 소비(h ≤ 한 줄급) 한정으로
            // 그 hard 는 이미 소비된 경계로 보고 통과한다.
            const REWIND_ORPHAN_CONT_PX: f64 = 48.0;
            // [#2291] relaxed 통과는 거대 셀(원저작 쪽나눔 흔적이 촘촘한 다문단
            // 셀)에 한정한다 — 소형 셀의 저장 hard-break 는 실제 행/조각 경계라
            // 통과시키면 조각이 비대해져 밴드 컷이 어긋난다 (21217935 8→9쪽
            // 회귀 실측). 연결맵 결함 셀은 유닛 37~123개.
            const GIANT_CELL_RELAXED_MIN_UNITS: usize = 24;
            let cell_relaxed = relaxed_hard_break && units.len() >= GIANT_CELL_RELAXED_MIN_UNITS;
            while j < units.len() {
                let u = &units[j];
                if j > start
                    && u.hard_break_before
                    && (!cell_relaxed
                        || (!u.empty_spacer
                            && (h + u.height > cell_budget
                                || cell_budget - h <= HARD_BREAK_REMAINING_TOLERANCE_PX)))
                {
                    if start > 0 && j == start + 1 && h <= REWIND_ORPHAN_CONT_PX {
                        h += u.height;
                        j += 1;
                        continue;
                    }
                    Self::rewind_rowbreak_orphan_before_hard_break(
                        table,
                        &units,
                        start,
                        cell_budget,
                        true,
                        &mut j,
                        &mut h,
                    );
                    hit_hard_break = true;
                    break;
                }
                if j > start && h + u.height > cell_budget {
                    if Self::rewind_rowbreak_mixed_nested_table_tail_for_fresh_page(
                        table,
                        &units,
                        start,
                        self.current_body_area.get().3.max(cell_budget),
                        &mut j,
                        &mut h,
                    ) {
                        break;
                    }
                    // [#1921] sliver 흡수 — advance_row_block_cut 의 예산 정지와 동일.
                    // 직후 tolerance 안의 저장 hard-break(한글 실제 페이지 경계)까지
                    // 흡수해, 다음 fragment 가 극소 잔여 sliver 페이지가 되는 것을 막는다.
                    if let Some((absorbed_h, absorbed_j)) =
                        Self::absorb_tail_before_stored_hard_break(&units, j, h, cell_budget)
                    {
                        h = absorbed_h;
                        j = absorbed_j;
                        hit_hard_break = true;
                        break;
                    }
                    break;
                }
                if j == start && !allow_force_progress && h + u.height > cell_budget {
                    break;
                }
                h += u.height;
                j += 1;
            }
            if j < units.len() {
                fully_consumed = false;
            }
            if h > 0.0 {
                consumed_height = consumed_height.max(row_offset + h);
            }
            // [#2097 진단] 오프셋 walk 셀별 결과 — 동작 불변.
            if std::env::var("RHWP_DIAG_BLKCUT").is_ok() {
                let stop = if j >= units.len() {
                    "end"
                } else if units[j].hard_break_before {
                    "hard"
                } else {
                    "budget"
                };
                eprintln!(
                    "DIAG_BLKCUT(ofs) cell[{}] r={} c={} units={} start={} j={} h={:.1} row_off={:.1} cell_budget={:.1} stop={} next_h={:.1}",
                    i,
                    cell.row,
                    cell.col,
                    units.len(),
                    start,
                    j,
                    h,
                    row_offset,
                    cell_budget,
                    stop,
                    units.get(j).map(|u| u.height).unwrap_or(0.0)
                );
            }
            end_cut.push(j);
        }
        RowCutResult {
            end_cut,
            hit_hard_break,
            fully_consumed,
            consumed_height,
        }
    }

    fn rewind_rowbreak_orphan_before_hard_break(
        table: &crate::model::table::Table,
        units: &[CellUnit],
        start: usize,
        avail_height: f64,
        force_rewind: bool,
        j: &mut usize,
        h: &mut f64,
    ) {
        if !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) || *j <= start + 1
        {
            return;
        }

        let hard_break_unit = &units[*j];
        let prev = &units[*j - 1];
        if prev.para_idx == hard_break_unit.para_idx {
            *h -= prev.height;
            *j -= 1;
            return;
        }

        if table.common.treat_as_char {
            return;
        }

        if let Some(rewind_to) = units[start..*j]
            .iter()
            .rposition(|unit| unit.vpos_gap_before)
            .map(|idx| start + idx)
        {
            if rewind_to > start {
                let rewind_h: f64 = units[rewind_to..*j].iter().map(|unit| unit.height).sum();
                let rewound_h = *h - rewind_h;
                const MAX_REWIND_BLANK_PX: f64 = 80.0;
                if !force_rewind && avail_height - rewound_h > MAX_REWIND_BLANK_PX {
                    return;
                }
                *h -= rewind_h;
                *j = rewind_to;
            }
        }
    }

    /// 재귀 1×1 표를 부모 `RowCut` 원장으로 투영할 때, source에서 한 묶음으로 표시한
    /// 빈 separator와 한 줄 제목만 현재 쪽에 들어가고 바로 다음 block이 넘치면 한컴은
    /// prelude 전체를 그 block과 함께 다음 쪽에 둔다. 제목 뒤 재귀 block의 작은
    /// 선행 fragment가 이미 들어간 경우에도, 그 연속 prefix까지 같은 묶음으로 되감는다.
    fn rewind_rowbreak_orphan_heading_before_recursive_block(
        table: &crate::model::table::Table,
        units: &[CellUnit],
        start: usize,
        avail_height: f64,
        j: &mut usize,
        h: &mut f64,
    ) -> bool {
        if !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) || table.common.treat_as_char
            || *j >= units.len()
            || *j <= start + 1
            || !avail_height.is_finite()
            || avail_height <= 0.0
        {
            return false;
        }

        let next = &units[*j];
        if !next.mixed_nested_fragment
            || !next.mixed_nested_recursive
            || next.mixed_nested_trailing
            || next.hard_break_before
            || *h + next.height <= avail_height + 0.5
        {
            return false;
        }

        // 직전 block은 모두 들어갔지만 다음 prelude의 빈 separator만 현재 조각에
        // 들어가고 제목이 예산을 넘는 형상도 같은 orphan이다. separator를 그대로
        // 소비하면 다음 조각은 제목부터 시작해 아래의 separator+heading 탐색을
        // 다시 수행할 수 없고, 제목과 작은 recursive prefix만 가진 sliver 쪽이
        // 생긴다. role은 source에서 정확히 `빈 문단 + 한 줄 제목 + 1×1 표`일 때만
        // 부여되므로 pending 제목 앞의 separator 하나만 되감는다.
        if next.recursive_block_prelude_role
            == RecursiveBlockPreludeRole::OneLineHeadingBeforeSingleCellTable
        {
            let separator_idx = *j - 1;
            let separator = &units[separator_idx];
            if !separator.hard_break_before
                && separator.mixed_nested_fragment
                && separator.mixed_nested_recursive
                && matches!(
                    separator.recursive_block_prelude_role,
                    RecursiveBlockPreludeRole::EmptySeparator
                        | RecursiveBlockPreludeRole::ExplicitPageBreakSeparator
                )
            {
                *h = (*h - separator.height).max(0.0);
                *j = separator_idx;
                return true;
            }
        }

        // `j-1`이 바로 제목인 기존 형상은 loop를 한 번도 돌지 않는다. 제목 뒤
        // recursive block의 작은 조각이 먼저 fit한 형상에서는 role=None인 연속
        // nontrailing prefix만 거슬러 올라가 가장 가까운 prelude 제목을 찾는다.
        let mut block_prefix_start = *j;
        while block_prefix_start > start {
            let unit = &units[block_prefix_start - 1];
            if unit.mixed_nested_fragment
                && unit.mixed_nested_recursive
                && !unit.mixed_nested_trailing
                && !unit.hard_break_before
                && !unit.stored_frame_break_before
                && !unit.vpos_gap_before
                && unit.recursive_block_prelude_role == RecursiveBlockPreludeRole::None
            {
                block_prefix_start -= 1;
            } else {
                break;
            }
        }
        if block_prefix_start <= start + 1 {
            return false;
        }

        let heading_idx = block_prefix_start - 1;
        let separator_idx = heading_idx - 1;
        // 현재 fragment가 separator부터 시작했다면 되감기는 `j == start`가 되어
        // pagination progress를 잃는다. 새 viewport는 separator를 예약값으로
        // 소비한 뒤 반드시 제목/자식 block 쪽으로 전진시킨다.
        if separator_idx <= start {
            return false;
        }
        let heading = &units[heading_idx];
        let separator = &units[separator_idx];
        let already_fit_recursive_prefix = block_prefix_start < *j;
        if !heading.mixed_nested_fragment
            || !heading.mixed_nested_recursive
            || heading.recursive_block_prelude_role
                != RecursiveBlockPreludeRole::OneLineHeadingBeforeSingleCellTable
            || separator.hard_break_before
            || !separator.mixed_nested_fragment
            || !separator.mixed_nested_recursive
            || !matches!(
                separator.recursive_block_prelude_role,
                RecursiveBlockPreludeRole::EmptySeparator
                    | RecursiveBlockPreludeRole::ExplicitPageBreakSeparator
            )
            // 일반 prelude는 제목 바로 뒤 block이 아직 시작되지 않았을 때만 기존
            // direct-next orphan 보정을 적용한다. 이미 recursive prefix가 들어간
            // source block까지 되감는 것은 명시적 Page/Section separator에 한정해,
            // 저장 프레임 경계에서 끝나야 할 정상 continuation을 앞당기지 않는다.
            || (already_fit_recursive_prefix
                && separator.recursive_block_prelude_role
                    != RecursiveBlockPreludeRole::ExplicitPageBreakSeparator)
        {
            return false;
        }

        let rewind_height: f64 = units[separator_idx..*j]
            .iter()
            .map(|unit| unit.height)
            .sum();
        *h = (*h - rewind_height).max(0.0);
        *j = separator_idx;
        true
    }

    fn rewind_rowbreak_tail_before_pending_hard_break(
        table: &crate::model::table::Table,
        units: &[CellUnit],
        start: usize,
        avail_height: f64,
        j: &mut usize,
        h: &mut f64,
    ) -> bool {
        if !matches!(
            table.page_break,
            crate::model::table::TablePageBreak::RowBreak
        ) || table.common.treat_as_char
            || *j <= start + 1
            || units[start..*j].iter().all(|unit| unit.empty_spacer)
        {
            return false;
        }

        let Some(rewind_to) = units[start..*j]
            .iter()
            .rposition(|unit| unit.vpos_gap_before)
            .map(|idx| start + idx)
        else {
            return false;
        };
        if units.get(*j).is_some_and(|unit| unit.hard_break_before) || rewind_to <= start {
            return false;
        }

        let rewind_h: f64 = units[rewind_to..*j].iter().map(|unit| unit.height).sum();
        let rewound_h = *h - rewind_h;
        const MAX_REWIND_BLANK_PX: f64 = 80.0;
        if avail_height - rewound_h > MAX_REWIND_BLANK_PX {
            return false;
        }
        *h -= rewind_h;
        *j = rewind_to;
        true
    }

    fn row_has_prior_rowspan_cover(table: &crate::model::table::Table, row: usize) -> bool {
        table.cells.iter().any(|cell| {
            let start = cell.row as usize;
            let end = start + (cell.row_span as usize).max(1);
            cell.row_span > 1 && start < row && row < end
        })
    }

    /// RowBreak 표의 rowspan 블록 중 셀 내부 HWP page reset 이 처음 나타나는 셀의
    /// 시작 행을 찾는다. 단순 rowspan 라벨 표는 기존 행 경계 분할을 유지한다.
    pub(crate) fn row_block_first_internal_hard_break_row(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        styles: &ResolvedStyleSet,
    ) -> Option<usize> {
        Self::row_block_cells(table, b_start, b_end)
            .iter()
            .filter_map(|cell| {
                let has_hard_break = self
                    .cell_units(cell, table, styles)
                    .iter()
                    .enumerate()
                    .any(|(i, unit)| i > 0 && unit.hard_break_before);
                has_hard_break.then_some(cell.row as usize)
            })
            .min()
    }

    /// RowBreak 표의 rowspan 블록 중 셀 내부 HWP page reset 이 있는 블록만
    /// 블록 컷 대상으로 삼기 위한 가드.
    pub(crate) fn row_block_has_internal_hard_break(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        styles: &ResolvedStyleSet,
    ) -> bool {
        self.row_block_first_internal_hard_break_row(table, b_start, b_end, styles)
            .is_some()
    }

    /// Return whether an ordinary row cut stops immediately before a saved
    /// cross-paragraph reset whose two owners are plain text paragraphs.
    ///
    /// A paragraph-local `vpos=0` also appears when a control-only paragraph
    /// starts (for example, an inline diagram table).  That transition is not
    /// by itself a physical page boundary, so callers must not treat the much
    /// broader `row_block_has_internal_hard_break` predicate as equivalent to
    /// this cut-local source contract.
    pub(crate) fn row_cut_ends_at_plain_text_saved_reset(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .collect();
        row_cells.sort_by_key(|cell| cell.col);

        row_cells.iter().enumerate().any(|(cell_idx, cell)| {
            let units = self.cell_units(cell, table, styles);
            let start = start_cut
                .get(cell_idx)
                .copied()
                .unwrap_or(0)
                .min(units.len());
            let end = end_cut
                .get(cell_idx)
                .copied()
                .unwrap_or(start)
                .clamp(start, units.len());
            if end <= start || end >= units.len() {
                return false;
            }

            let previous = &units[end - 1];
            let next = &units[end];
            if !next.hard_break_before
                || previous.vis_start >= previous.vis_end
                || next.vis_start >= next.vis_end
                || next.para_idx <= previous.para_idx
            {
                return false;
            }

            cell.paragraphs
                .get(previous.para_idx)
                .zip(cell.paragraphs.get(next.para_idx))
                .is_some_and(|(previous_para, next_para)| {
                    previous_para.controls.is_empty() && next_para.controls.is_empty()
                })
        })
    }

    /// [Task #1025] 행블록 `[b_start, b_end)` 와 교차하는 셀(rs>1 포함)을 모은다.
    /// `advance_row_block_cut` / `row_block_content_height` / 렌더러 공유 — 순서는
    /// 호출부에서 `(row, col)` 로 정렬한다.
    pub(crate) fn row_block_cells<'a>(
        table: &'a crate::model::table::Table,
        b_start: usize,
        b_end: usize,
    ) -> Vec<&'a crate::model::table::Cell> {
        table
            .cells
            .iter()
            .filter(|c| {
                let cr = c.row as usize;
                let ce = cr + (c.row_span as usize).max(1);
                cr < b_end && ce > b_start
            })
            .collect()
    }

    /// [Task #1025] 행블록 컷 범위 `[start_cut, end_cut)` 의 블록 표시 높이(패딩 포함).
    /// 블록 셀별 `content_in_cut + pad`, 블록 max. `advance_row_block_cut` 과 동일한
    /// `(row, col)` 셀 순서를 사용한다.
    pub(crate) fn row_block_content_height(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let mut cells = Self::row_block_cells(table, b_start, b_end);
        cells.sort_by_key(|c| (c.row, c.col));
        let mut max_h = 0.0f64;
        for (i, cell) in cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let su = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let eu = end_cut
                .get(i)
                .copied()
                .unwrap_or(units.len())
                .clamp(su, units.len());
            let trailing_trim = if end_cut.is_empty() {
                0.0
            } else {
                self.native_multirow_saved_reset_trailing_trim(table, cell, &units, eu, styles)
            };
            let content: f64 =
                (units[su..eu].iter().map(|u| u.height).sum::<f64>() - trailing_trim).max(0.0);
            let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
            let h = content + pad_top + pad_bottom;
            // [#2287 진단] start_cut 적용 잔여 평가 분해 — 동작 불변.
            if std::env::var("RHWP_DIAG_BLKH").is_ok() && !start_cut.is_empty() {
                eprintln!(
                    "DIAG_BLKH cell[{}] r={} c={} units={} su={} eu={} content={:.1} h={:.1}",
                    i,
                    cell.row,
                    cell.col,
                    units.len(),
                    su,
                    eu,
                    content,
                    h
                );
            }
            if h > max_h {
                max_h = h;
            }
        }
        max_h
    }

    /// [#2287] start_cut 이후 블록 잔여 콘텐츠 높이 — `advance_row_block_cut` 의
    /// spacer 소비 의미론(컷 재개 지점의 선두/후미 empty-spacer run 은 무높이
    /// 소비)을 미러한 잔여 평가. `row_block_content_height` 는 spacer 꼬리를
    /// 전량 합산해 잔여를 과대평가한다 (59043 규제영향분석서 41→44쪽 회귀 실측).
    /// [#2287/PR #2290 P1] 셀의 컷 범위(su..eu) 유닛 가시 높이 + 상하 패딩.
    /// 블록-합 보정(table_partial)에서 rowspan 셀 bbox 를 컷과 정합시키는 데 쓴다.
    pub(crate) fn cell_cut_visible_height(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        start_unit: usize,
        end_unit: usize,
    ) -> f64 {
        let units = self.cell_units(cell, table, styles);
        let su = start_unit.min(units.len());
        let eu = end_unit.clamp(su, units.len());
        let trailing_trim =
            self.native_multirow_saved_reset_trailing_trim(table, cell, &units, eu, styles);
        let content: f64 =
            (units[su..eu].iter().map(|u| u.height).sum::<f64>() - trailing_trim).max(0.0);
        if content <= 0.0 {
            return 0.0;
        }
        let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
        content + pad_top + pad_bottom
    }

    pub(crate) fn row_block_cut_remaining_height(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        start_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let mut cells = Self::row_block_cells(table, b_start, b_end);
        cells.sort_by_key(|c| (c.row, c.col));
        let mut max_h = 0.0f64;
        for (i, cell) in cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let su = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            if su >= units.len() {
                continue;
            }
            let (mut lo, mut hi) = (su, units.len());
            if su > 0 {
                while lo < hi && units[lo].empty_spacer && !units[lo].hard_break_before {
                    lo += 1;
                }
                while hi > lo && units[hi - 1].empty_spacer && !units[hi - 1].hard_break_before {
                    hi -= 1;
                }
            }
            let content: f64 = units[lo..hi].iter().map(|u| u.height).sum();
            if content <= 0.0 {
                continue;
            }
            let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
            let h = content + pad_top + pad_bottom;
            if h > max_h {
                max_h = h;
            }
        }
        max_h
    }

    /// 블록 컷 벡터를 특정 행의 per-row 컷으로 변환해 해당 행 표시 높이를 계산한다.
    pub(crate) fn row_block_cut_row_content_height(
        &self,
        table: &crate::model::table::Table,
        b_start: usize,
        b_end: usize,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let mut block_cells = Self::row_block_cells(table, b_start, b_end);
        block_cells.sort_by_key(|c| (c.row, c.col));

        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|c| c.row as usize == row && c.row_span == 1)
            .collect();
        row_cells.sort_by_key(|c| c.col);

        if row_cells.is_empty() {
            return 0.0;
        }

        let mut per_start = Vec::with_capacity(row_cells.len());
        let mut per_end = Vec::with_capacity(row_cells.len());
        let mut has_visible_range = false;
        let mut has_row_cut = false;
        for cell in row_cells {
            let block_idx = block_cells
                .iter()
                .position(|c| c.row == cell.row && c.col == cell.col);
            let units = self.cell_units(cell, table, styles);
            let su = block_idx
                .and_then(|idx| start_cut.get(idx).copied())
                .unwrap_or(0)
                .min(units.len());
            let eu = block_idx
                .and_then(|idx| end_cut.get(idx).copied())
                .unwrap_or(units.len())
                .clamp(su, units.len());
            if eu > su {
                has_visible_range = true;
            }
            if su > 0 || eu < units.len() {
                has_row_cut = true;
            }
            per_start.push(su);
            per_end.push(eu);
        }

        if !has_visible_range {
            return 0.0;
        }

        if has_row_cut {
            self.row_cut_content_height(table, row, &per_start, &per_end, styles)
        } else {
            self.row_cut_content_height(table, row, &[], &[], styles)
        }
    }

    /// [Task #1748] 셀 유닛 누적높이가 `budget`(패딩 제외 콘텐츠 예산) 안에
    /// 들어가는 선두 유닛 수를 반환한다. 컷 행에 걸친(straddling) rowspan 셀의
    /// 높이 기반 가시 유닛 컷 산출용 — 컷 페이지의 eu 와 연속 페이지의 su 가
    /// 같은 예산 식으로 계산되어 경계 줄 인덱스가 산술적으로 일치한다.
    pub(crate) fn cell_units_fitting_height(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        budget: f64,
    ) -> usize {
        const EPS: f64 = 0.1;
        let units = self.cell_units(cell, table, styles);
        let mut n = 0usize;
        let mut h = 0.0f64;
        while n < units.len() && h + units[n].height <= budget + EPS {
            h += units[n].height;
            n += 1;
        }
        n
    }

    /// HWP5 저장 pagination 계약의 `RowBreak` 표에서 정확히 두 행을 덮는 병합 셀의
    /// 두 저장 문단이 각각 한 줄 유닛이면, 행 경계의 문단 owner를 그대로 유지할 수
    /// 있는지 판정한다.
    ///
    /// 일반 rowspan 분할은 물리 높이로 잘라야 한다. 다만 이 좁은 형상은 저장된 두
    /// 문단이 두 물리 행에 정확히 대응한다. 첫 문단의 trailing line/문단 간격까지
    /// 첫 행 예산에 포함하면, ink는 들어가는데 unit만 다음 fragment로 밀려 두 문단이
    /// 재방출된다(76076 p18→p19 `11.영향평가` / `여부`).
    pub(crate) fn native_two_row_rowspan_paragraph_owner_boundary(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
    ) -> bool {
        if !self.profile.get().hwp5_stored_pagination_layout()
            || table.common.treat_as_char
            || !matches!(table.page_break, TablePageBreak::RowBreak)
            || cell.row_span != 2
            || cell.paragraphs.len() != 2
            || cell
                .paragraphs
                .iter()
                .any(|paragraph| paragraph.text.trim().is_empty() || !paragraph.controls.is_empty())
        {
            return false;
        }

        let units = self.cell_units(cell, table, styles);
        units.len() == 2
            && units.iter().enumerate().all(|(para_idx, unit)| {
                unit.para_idx == para_idx
                    && unit.vis_start == 0
                    && unit.vis_end == 1
                    && !unit.empty_spacer
                    && unit.nested_row.is_none()
                    && unit.nested_table_fragment.is_none()
                    && !unit.mixed_nested_fragment
                    && unit.non_inline_control_range.is_none()
            })
    }

    /// [Task #993] 한 셀의 유닛 범위 `[start_unit, end_unit)`를 문단별 줄 범위로
    /// 변환한다. `layout_partial_table`이 `RowCut`으로 가시 범위를 렌더할 때
    /// 사용 — 결과는 종전 `compute_cell_line_ranges`와 같은
    /// `Vec<(start_line, end_line)>` 형식(문단마다 1개, 미가시 문단은 `(0,0)`).
    pub(crate) fn cell_line_ranges_from_cut(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        start_unit: usize,
        end_unit: usize,
    ) -> Vec<(usize, usize)> {
        let units = self.cell_units(cell, table, styles);
        let mut ranges = vec![(0usize, 0usize); cell.paragraphs.len()];
        let mut seen = vec![false; cell.paragraphs.len()];
        let lo = start_unit.min(units.len());
        let hi = end_unit.min(units.len());
        for u in units.iter().take(hi).skip(lo) {
            if u.para_idx >= ranges.len() {
                continue;
            }
            if !seen[u.para_idx] {
                ranges[u.para_idx] = (u.vis_start, u.vis_end);
                seen[u.para_idx] = true;
            } else {
                let r = &mut ranges[u.para_idx];
                r.0 = r.0.min(u.vis_start);
                r.1 = r.1.max(u.vis_end);
            }
        }
        ranges
    }

    pub(crate) fn cell_cut_contains_non_inline_control_units(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        start_unit: usize,
        end_unit: usize,
        para_idx: usize,
    ) -> bool {
        let units = self.cell_units(cell, table, styles);
        let lo = start_unit.min(units.len());
        let hi = end_unit.min(units.len()).max(lo);
        let has_non_inline_control = cell.paragraphs.get(para_idx).is_some_and(|para| {
            para.controls.iter().any(|control| match control {
                Control::Picture(picture) => !picture.common.treat_as_char,
                Control::Shape(shape) => !shape.common().treat_as_char,
                _ => false,
            })
        });
        if !has_non_inline_control {
            return false;
        }

        // 현재 컷 안에 non-inline 개체가 차지하는 명시 유닛이 실제로 포함될 때만
        // 셀 안 non-inline 개체를 그린다. 같은 문단의 텍스트 줄만 continuation 에
        // 남아 있는 경우까지 허용하면 이전 쪽 그림이 모든 페이지에 반복된다.
        units.iter().take(hi).skip(lo).any(|unit| {
            unit.para_idx == para_idx
                && unit.vis_start == unit.vis_end
                && !unit.empty_spacer
                && unit.nested_row.is_none()
                && !unit.mixed_nested_fragment
        })
    }

    /// `cell_cut_contains_non_inline_control_units`의 control-identity 버전.
    ///
    /// Square/Tight/Through flow fragment는 control range의 **첫** unit을 포함한 cut만
    /// picture/shape를 emit한다. 같은 control의 뒷 unit은 다음 physical fragment에서
    /// 다시 image를 paint하지 않는다. legacy/TopAndBottom unit처럼 range가 없는 경우에는
    /// 기존 paragraph-level 판정을 유지해 저장 형식별 기존 contract를 바꾸지 않는다.
    pub(crate) fn cell_cut_starts_non_inline_control(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        start_unit: usize,
        end_unit: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> bool {
        let Some(para) = cell.paragraphs.get(para_idx) else {
            return false;
        };
        let has_non_inline_control =
            para.controls
                .get(control_idx)
                .is_some_and(|control| match control {
                    Control::Picture(picture) => !picture.common.treat_as_char,
                    Control::Shape(shape) => !shape.common().treat_as_char,
                    _ => false,
                });
        if !has_non_inline_control {
            return false;
        }

        let units = self.cell_units(cell, table, styles);
        let lo = start_unit.min(units.len());
        let hi = end_unit.min(units.len()).max(lo);
        let control_start = units.iter().position(|unit| {
            unit.para_idx == para_idx
                && unit
                    .non_inline_control_range
                    .is_some_and(|(first, last)| first <= control_idx && control_idx <= last)
        });
        if let Some(start) = control_start {
            return lo <= start && start < hi;
        }

        let mut saw_legacy_candidate = false;
        for unit in units.iter().take(hi).skip(lo) {
            let candidate = unit.para_idx == para_idx
                && unit.vis_start == unit.vis_end
                && !unit.empty_spacer
                && unit.nested_row.is_none()
                && !unit.mixed_nested_fragment;
            if !candidate {
                continue;
            }
            if unit.non_inline_control_range.is_none() {
                saw_legacy_candidate = true;
            }
        }
        saw_legacy_candidate
    }

    pub(crate) fn mixed_nested_split_from_cut(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        start_unit: usize,
        end_unit: usize,
        para_idx: usize,
    ) -> Option<NestedTableSplit> {
        let units = self.cell_units(cell, table, styles);
        let lo = start_unit.min(units.len());
        let hi = end_unit.min(units.len()).max(lo);
        let mut total = 0.0;
        let mut offset = 0.0;
        let mut visible_units: Vec<(f64, bool, bool, f64)> = Vec::new();
        let mut recursive_total = 0usize;
        let mut recursive_start = 0usize;
        let mut has_non_recursive_fragment = false;
        for (idx, unit) in units.iter().enumerate() {
            if unit.para_idx != para_idx || !unit.mixed_nested_fragment {
                continue;
            }
            if unit.mixed_nested_recursive {
                recursive_total += 1;
                if idx < lo {
                    recursive_start += 1;
                }
            } else {
                has_non_recursive_fragment = true;
            }
            total += unit.height;
            if idx < lo {
                offset += unit.height;
            }
            if idx >= lo && idx < hi {
                visible_units.push((
                    unit.height,
                    unit.mixed_nested_trailing,
                    unit.mixed_nested_recursive,
                    unit.mixed_nested_content_height,
                ));
            }
        }
        // `terminal` is scoped to this mixed nested stream, not the host
        // cell's entire unit list.  A completed inner table can be followed
        // by another paragraph/table in the same outer cell; treating it as
        // non-terminal drops its final reservation and shortens the current
        // frame (42065 p13→p15).
        let terminal = !units
            .iter()
            .skip(hi)
            .any(|unit| unit.para_idx == para_idx && unit.mixed_nested_fragment);
        let successor_trailing_reservation = trailing_reservation_after_final_source_owner(
            para_idx,
            units.get(hi).map(MixedNestedOwnerMarker::from),
            units
                .iter()
                .skip(hi.saturating_add(1))
                .map(MixedNestedOwnerMarker::from),
        );
        // A non-terminal fragment must not paint the synthetic trailing unit:
        // its successor owns that source window.  The terminal fragment is
        // different — that trailing unit can contain the final ordinary
        // paragraphs after the nested table, so discarding it clips the
        // document's last content (42065 p17's section 4).
        if offset > 0.5 && !terminal {
            while visible_units
                .last()
                .is_some_and(|(_, trailing, _, _)| *trailing)
            {
                visible_units.pop();
            }
        }
        let flow_visible: f64 = visible_units.iter().map(|(h, _, _, _)| *h).sum();
        let recursive_visible = visible_units
            .iter()
            .filter(|(_, _, recursive, _)| *recursive)
            .count();
        let recursive_cut = if recursive_total > 0 && !has_non_recursive_fragment {
            // The terminal page can begin with a synthetic trailing unit that
            // only reserved physical flow on the preceding viewport.  It has
            // no visible child content and must not become the recursive
            // child's first source owner (42065 p17: 32px blank before the
            // heading).  Keep the unit in outer flow accounting and advance
            // only the authoritative child cursor past it.
            let leading_trailing = if terminal && offset > 0.5 {
                visible_units
                    .iter()
                    .take_while(|(_, trailing, recursive, content_height)| {
                        *trailing && *recursive && *content_height <= 0.5
                    })
                    .count()
            } else {
                0
            };
            let recursive_start = recursive_start + leading_trailing;
            let recursive_end =
                recursive_start + recursive_visible.saturating_sub(leading_trailing);
            Some(NestedTableCut {
                start_row: 0,
                end_row: 1,
                start_cut: if recursive_start == 0 {
                    Vec::new()
                } else {
                    vec![recursive_start]
                },
                end_cut: if recursive_end >= recursive_total {
                    Vec::new()
                } else {
                    vec![recursive_end]
                },
                is_block_split: false,
            })
        } else {
            None
        };
        // Continuation pages still need the whole visible slice clipped in, even
        // when the same host cell has following paragraphs in the current cut.
        // Shrinking the clip to the first non-trailing unit keeps the flow
        // advance but clips the nested table content above the cell on page 8.
        let visible: f64 = flow_visible;
        let first_visible_content_height = visible_units
            .iter()
            .find_map(|(height, trailing, _, _)| (!*trailing).then_some(*height))
            .unwrap_or(0.0);
        let first_visible_paint_height = visible_units
            .iter()
            .find_map(|(_, trailing, _, content_height)| (!*trailing).then_some(*content_height))
            .unwrap_or(0.0);
        let last_visible_content_height = visible_units
            .iter()
            .rev()
            .find_map(|(height, trailing, _, _)| (!*trailing).then_some(*height))
            .unwrap_or(0.0);
        let first_visible_starts_after_table = units
            .iter()
            .skip(lo)
            .take(hi.saturating_sub(lo))
            .find(|unit| {
                unit.para_idx == para_idx
                    && unit.mixed_nested_fragment
                    && !unit.mixed_nested_trailing
            })
            .is_some_and(|unit| unit.mixed_nested_starts_after_table);
        // 1×1 host 안의 1×1 표 continuation은 이전 조각의 첫 unit을 물리
        // reservation으로 이미 전진시킨다. 다음 조각의 content origin까지 원래
        // `offset`만 쓰면 그 unit이 다시 페이지 상단에 그려져 이후 제목/표가 한 줄씩
        // 아래로 drift한다. 이 중복은 종료 조각에도 남는다. terminal tail의 물리
        // clip 높이는 아래 `terminal_single_cell_tail` 분기가 별도로 보존하므로,
        // 여기서는 종료 여부와 관계없이 content origin을 같은 기준으로 전진시킨다
        // (42065 p12–p17).
        let single_cell_nested_continuation = table.row_count == 1
                && table.col_count == 1
                && cell.paragraphs.get(para_idx).is_some_and(|paragraph| {
                    paragraph.controls.iter().any(|control| {
                        matches!(control, Control::Table(nested) if nested.row_count == 1 && nested.col_count == 1)
                    })
                });
        // PR #4122가 만든 재귀 child cursor가 있으면 그 cursor가 소유권의
        // 권위다. scalar offset 보정은 재귀 투영이 없는 기존 fallback에만 쓴다.
        let compensate_first_visible = recursive_cut.is_none()
            && offset > 0.5
            && single_cell_nested_continuation
            && !terminal
            && !first_visible_starts_after_table;
        let offset_within_start = if recursive_cut.is_some() {
            (offset - first_visible_content_height).max(0.0)
        } else if compensate_first_visible {
            offset + first_visible_content_height
        } else if self.profile.get().hwpx_stored_layout()
            && terminal
            && offset > 0.5
            && !first_visible_starts_after_table
            && cell.paragraphs.get(para_idx).is_some_and(|paragraph| {
                paragraph.controls.iter().any(|control| {
                    matches!(control, Control::Table(nested) if nested.row_count == 1 && nested.col_count == 1)
                })
            })
        {
            // The HWPX outer RowBreak cell's last source unit is already
            // painted by the preceding fragment. Its terminal 1×1 child uses
            // a fresh viewport, so the raw source offset would paint that same
            // unit at the new page top. Advance by precisely one visible child
            // line; this preserves the terminal tail while starting from the
            // next physical source owner (#3637 HWP 2020 p26→p27).
            offset + first_visible_content_height
        } else {
            offset
        };
        let is_offset_continuation = offset_within_start > 0.5;
        let has_later_host_source_owner = units
            .iter()
            .skip(hi)
            .any(|unit| Self::cell_unit_has_visible_content(cell, unit));
        let terminal_table_before_host_successor = recursive_cut.is_none()
            && terminal
            && is_offset_continuation
            && self.profile.get().native_hwp5_layout()
            && single_cell_nested_continuation
            && has_later_host_source_owner
            && cell
                .paragraphs
                .get(para_idx)
                .is_some_and(|paragraph| paragraph.text.trim().is_empty());
        let terminal_host_line_spacing = if terminal_table_before_host_successor {
            cell.paragraphs
                .get(para_idx)
                .and_then(|paragraph| paragraph.line_segs.first())
                .map(|segment| hwpunit_to_px(segment.line_spacing, self.dpi))
                .unwrap_or(0.0)
        } else {
            0.0
        };
        let terminal_continuation_inset = if terminal_table_before_host_successor {
            let nested_top_padding = cell
                .paragraphs
                .get(para_idx)
                .and_then(|paragraph| {
                    paragraph.controls.iter().find_map(|control| match control {
                        Control::Table(table) => Some(table),
                        _ => None,
                    })
                })
                .and_then(|table| {
                    table
                        .cells
                        .first()
                        .map(|nested_cell| self.resolve_cell_padding(nested_cell, table).2)
                })
                .unwrap_or(0.0);
            (first_visible_content_height - first_visible_paint_height).max(0.0)
                + nested_top_padding
        } else {
            0.0
        };
        let terminal_single_cell_tail = recursive_cut.is_none()
            && terminal
            && is_offset_continuation
            && single_cell_nested_continuation
            && !has_later_host_source_owner;
        let visible_height = if terminal_table_before_host_successor {
            // 이 mixed stream은 끝났지만 같은 host cell에는 다음 source 문단이 있다.
            // 자식 표의 실제 마지막 unit까지만 frame을 닫고, host 문단의 후행
            // line-spacing은 아래 flow에만 더한다. terminal tail 보정까지 frame에
            // 넣으면 다음 separator 앞에 빈 표 영역이 생긴다(issue2007 p14).
            flow_visible + terminal_continuation_inset
        } else if recursive_cut.is_some() && is_offset_continuation && !terminal {
            // 재귀 child cursor는 이전 viewport가 예약한 첫 가시 unit을 source offset에서
            // 되감아 정확한 시작 owner를 복원한다. paint viewport도 같은 unit만큼
            // 늘려야 end_cut 안의 마지막 줄이 셀 clip 밖으로 잘리지 않는다. child cut이
            // source 끝을 제한하므로 다음 owner를 다시 그리지는 않는다(42065 p14/p15).
            flow_visible + first_visible_content_height
        } else if recursive_cut.is_none()
            && !terminal
            && offset <= 0.5
            && single_cell_nested_continuation
            && successor_trailing_reservation > 0.5
        {
            // 현재 cut이 자식 1×1 표의 모든 실제 source unit을 포함하고, 바로 다음
            // unit이 content 없는 trailing reservation이며 그 뒤에 실제 source owner가
            // 없을 때 그 reservation은 다음 쪽의 text owner가 아니다. scalar child
            // renderer는 물리 cell 높이로 같은 cut을 다시 계산하므로, 이 작은 예약
            // 높이를 clip에 보존하지 않으면 셀 padding 때문에 마지막 실제 줄 하나가
            // fitting budget 밖으로 밀린다(42065 p15).
            // flow 높이는 아래에서 `flow_visible`을 유지해 다음 sibling 위치는 바꾸지 않는다.
            flow_visible + successor_trailing_reservation
        } else if terminal_single_cell_tail {
            // The terminal 1×1 fragment has no successor to reserve space
            // for. Its final ordinary paragraphs are still laid out one
            // first-unit below the fragment origin, however, so both the
            // nested cell clip and its host RowBreak cell must retain that
            // physical tail. Otherwise the source remains in export-text but
            // SVG/Canvas clips it (42065 p17 section 4).
            flow_visible + first_visible_content_height * 2.0 + 4.0
        } else if self.profile.get().native_hwp5_layout() && compensate_first_visible && !terminal {
            // `compensate_first_visible` advances the child content origin by
            // one unit because the preceding viewport already reserved it.
            // Native HWP5 must shorten the child paint viewport by the same
            // unit; otherwise its end advances one line past the RowCut and
            // both adjacent pages paint that line (42065 p10/p11).  Keep the
            // parent flow height unchanged so pagination/sibling placement
            // continues to use the authoritative RowCut geometry.
            (flow_visible - first_visible_content_height).max(0.0)
        } else if self.profile.get().native_hwp5_layout()
            && is_offset_continuation
            && first_visible_starts_after_table
            && !terminal
        {
            // A new physical block can begin inside a continuation cut.  Its
            // first unit is not a preceding-page reservation, but the cut's
            // final unit is the successor viewport reservation.  Do not let
            // that final line extend the nested paint window past the RowCut
            // owner boundary (42065 p10/p11).
            (flow_visible + first_visible_content_height - 4.0 - last_visible_content_height)
                .max(visible)
        } else if is_offset_continuation && !compensate_first_visible {
            // Mixed text+nested-table units include a small layout allowance
            // (`nested_h + 4.0`) so pagination has enough flow room. That
            // allowance must not expand the visible nested border, otherwise
            // the continuation box encloses the following host paragraph.
            (flow_visible + first_visible_content_height - 4.0).max(visible)
        } else {
            visible
        };
        if total <= 0.5 || visible <= 0.5 {
            return None;
        }
        let remaining = (total - offset).max(0.0);
        let flow_height = if terminal_table_before_host_successor {
            flow_visible + terminal_host_line_spacing
        } else if recursive_cut.is_some() {
            flow_visible
        } else if terminal_single_cell_tail {
            visible_height
        } else if is_offset_continuation && !compensate_first_visible {
            flow_visible + first_visible_content_height
        } else {
            flow_visible.min(remaining)
        };
        // 행 범위는 픽셀 오프셋에서 유도한다. 종전에는 `0..1` 로 고정해, 2행 이상
        // 중첩 표가 텍스트와 문단을 공유하면 연속 페이지가 **행 0 만** 다시 그리고
        // 뒤 행의 내용이 어느 페이지에도 나오지 않았다 (75544 pi=527: 2행 표,
        // off 672 + vis 747 = 전체 1419 로 높이 회계는 완전한데 end_row=1 이라
        // 행 1 의 25문단이 통째로 탈락). #1073 이 per-중첩행 컷 경로에서 고친
        // "row0 재렌더" 와 같은 결함이 혼재 문단 경로에 남아 있던 것이다.
        //
        // 높이 필드는 유닛 회계에서 온 값을 그대로 쓴다 — 조각 경계는 이미 컷이
        // 정했고, 행 변환은 "그 조각이 어느 행들을 담는가" 만 정한다.
        let nested = cell.paragraphs.get(para_idx).and_then(|p| {
            p.controls.iter().find_map(|ctrl| match ctrl {
                Control::Table(t) => Some(&**t),
                _ => None,
            })
        });
        // `terminal` 자체는 마지막 tail의 source cut을 끄는 기존 안전장치다.
        // 그러나 마지막 RowBreak 행의 1×1 block child는 앞 fragment가 child 첫
        // source unit을 이미 소비했어도 terminal이 될 수 있다. 이 경우 물리 clip만
        // 쓰면 p33의 마지막 줄을 p34 top에 다시 paint한다(76076 p33→p34).
        // `native_short_parent_child_fragment_eligible`는 짧은 child를 paginator
        // unit으로 승격하는 별도 계약이고, 여기서는 이미 mixed source cut으로
        // 도달한 terminal child의 시작 cursor만 보존한다.
        let native_short_terminal_child = nested.is_some_and(|child| {
            self.native_short_parent_child_fragment_eligible(
                table,
                cell,
                child,
                self.nested_table_mixed_fragment_heights(child, styles)
                    .iter()
                    .map(|fragment| fragment.height)
                    .sum(),
            )
        });
        let terminal_rowbreak_source_cursor = nested.is_some_and(|child| {
            self.native_terminal_rowbreak_child_source_cursor_eligible(table, cell, child)
        });
        let force_source_start_cut = offset > 0.5
            && terminal
            && (native_short_terminal_child || terminal_rowbreak_source_cursor);
        let (start_row, end_row, mut row_offset_within_start, visible_height) = match nested {
            Some(_) if recursive_cut.is_some() => (0, 1, 0.0, visible_height),
            Some(nt) if nt.row_count > 1 => {
                let ncol = nt.col_count as usize;
                let nrow = nt.row_count as usize;
                let row_heights = self.resolve_row_heights(nt, ncol, nrow, None, styles, true);
                let cs = hwpunit_to_px(nt.cell_spacing as i32, self.dpi);
                let rows = calc_nested_split_rows(&row_heights, cs, offset, visible);
                // [#3658] 종료 조각: start_row 상단 중 이전 쪽에 이미 보인 밴드만큼
                // 내부 오프셋을 부여한다. 종전(0.0 고정)에는 종료 조각이 start_row 를
                // 처음부터 재적층해 행 그리드보다 커지고, 초과한 꼬리 문단이 셀 하단
                // 드롭에 걸려 어느 쪽에도 렌더되지 않았다 (75544 pi=527: 재적층 766px
                // vs 행높이 747px → 마지막 2문단 유실). 이미 보인 밴드를 건너뛰면
                // 잔여 콘텐츠가 행 그리드 안에 들어가고 중복 렌더도 없다.
                let shown_band = if terminal && rows.start_row > 0 && rows.end_row >= nrow {
                    let mut prefix = 0.0f64;
                    for (r, rh) in row_heights.iter().enumerate().take(rows.start_row) {
                        prefix += rh;
                        if r + 1 < nrow {
                            prefix += cs;
                        }
                    }
                    let band = offset - prefix;
                    if band > 0.5 {
                        band
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let vis_h = if terminal && shown_band > 0.0 {
                    // 종료 조각의 표시 상자는 포함 행 전체(그리드) 높이를 유지한다 —
                    // 유닛 회계 기반 상자가 행 그리드보다 작으면 셀 클립이 꼬리를 자른다.
                    visible_height.max(rows.visible_height)
                } else {
                    visible_height
                };
                (rows.start_row, rows.end_row, shown_band, vis_h)
            }
            // 1행 표는 종전 규약 유지 — 행 경계가 없어 오프셋만으로 이어진다.
            _ => (0, 1, offset_within_start, visible_height),
        };
        if terminal_table_before_host_successor {
            // continuation 첫 line box의 leading과 셀 top padding을 source offset에서
            // 되돌려 현재 fragment 안에 다시 보인다. paint만 아래로 옮기며 위에서
            // 계산한 flow_height에는 더하지 않아 다음 host 문단 위치는 유지한다.
            row_offset_within_start =
                (row_offset_within_start - terminal_continuation_inset).max(0.0);
        }
        Some(NestedTableSplit {
            start_row,
            end_row,
            visible_height,
            flow_height,
            // Keep one visible content unit reserved in bbox/flow so the
            // border wraps only that tail line and the following paragraph in
            // the host cell starts below it. This reservation is physical
            // space only; `offset_within_start` above remains the full
            // consumed content origin.
            // 긴 terminal child는 source cursor가 p33까지의 단위를 이미 버린다.
            // 같은 offset으로 물리 원점까지 올리면 p34의 새 첫 줄도 clip 위로
            // 이중 소비된다. short-parent 계약은 기존 물리 inset을 유지한다.
            offset_within_start: if terminal_rowbreak_source_cursor && !native_short_terminal_child
            {
                0.0
            } else {
                row_offset_within_start
            },
            content_offset: offset,
            force_source_start_cut,
            // p33→34류(terminal_rowbreak_source_cursor만 true)는 이미 소유가 끝난 마지막
            // unit을 재생하면 안 되므로, native short-parent 형상에서만 켠다.
            replay_terminal_boundary_unit: native_short_terminal_child,
            terminal,
            recursive_cut,
        })
    }

    /// [#4069] 바깥 셀 컷에 선택된 `CELL` 분할 중첩 표 조각을 자식 표의
    /// `(row, RowCut)` 범위로 되돌린다. 측정 원장에 기록한 시작/끝 cursor를
    /// 그대로 사용하므로 페이지마다 전체 행을 다시 그리는 scalar clip이 없다.
    pub(crate) fn nested_table_split_from_cut_units(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        start_unit: usize,
        end_unit: usize,
        para_idx: usize,
    ) -> Option<NestedTableSplit> {
        let units = self.cell_units(cell, table, styles);
        let lo = start_unit.min(units.len());
        let hi = end_unit.min(units.len()).max(lo);
        let mut first_unit: Option<&CellUnit> = None;
        let mut last_unit: Option<&CellUnit> = None;
        let mut has_fragment = false;
        let mut visible_height = 0.0;
        for unit in units.iter().take(hi).skip(lo) {
            if unit.para_idx != para_idx || unit.nested_row.is_none() {
                continue;
            }
            first_unit.get_or_insert(unit);
            last_unit = Some(unit);
            has_fragment |= unit.nested_table_fragment.is_some();
            visible_height += unit.height;
        }
        if !has_fragment {
            return None;
        }
        let (first_unit, last_unit) = (first_unit?, last_unit?);
        let first_row = first_unit.nested_row?;
        let last_row = last_unit.nested_row?;
        let start_cut = first_unit
            .nested_table_fragment
            .as_ref()
            .map(|fragment| {
                if fragment.start_cut.iter().all(|cut| *cut == 0) {
                    Vec::new()
                } else {
                    fragment.start_cut.clone()
                }
            })
            .unwrap_or_default();
        let (end_cut, terminal) = last_unit
            .nested_table_fragment
            .as_ref()
            .map(|fragment| {
                if fragment.terminal {
                    (Vec::new(), true)
                } else {
                    (fragment.end_cut.clone(), false)
                }
            })
            .unwrap_or_else(|| (Vec::new(), true));
        Some(NestedTableSplit {
            start_row: first_row,
            end_row: last_row + 1,
            visible_height,
            flow_height: visible_height,
            offset_within_start: 0.0,
            content_offset: 0.0,
            force_source_start_cut: false,
            replay_terminal_boundary_unit: false,
            terminal,
            recursive_cut: Some(NestedTableCut {
                start_row: first_row,
                end_row: last_row + 1,
                start_cut,
                end_cut,
                is_block_split: false,
            }),
        })
    }

    /// 컷 유닛 범위를 **중첩 표 행 범위**로 옮긴다 (per-중첩행 유닛 경로).
    ///
    /// per-중첩행 분해가 붙은 문단은 유닛이 `nested_row` 를 들고 있으므로, 컷에 들어온
    /// 유닛들의 행 번호에서 곧바로 범위를 얻는다.
    ///
    /// 종전에는 호출부가 "컷 유닛 인덱스 == 중첩행 번호" 라고 가정해 셀이 **문단 1개**
    /// 일 때만 이 경로를 썼다. 문단이 여럿인 셀에서는 유닛에 텍스트 줄이 섞여 인덱스가
    /// 행 번호가 아니게 되고, 그러면 렌더가 `available_h` 휴리스틱으로 폴백해 연속
    /// 페이지가 행 0 부터 다시 그린다(뒤 행 유실). 유닛이 이미 행 번호를 들고 있으니
    /// 인덱스 가정을 버리고 그 필드를 읽는다.
    pub(crate) fn nested_row_range_from_cut_units(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        start_unit: usize,
        end_unit: usize,
        para_idx: usize,
    ) -> Option<(usize, usize)> {
        let units = self.cell_units(cell, table, styles);
        let lo = start_unit.min(units.len());
        let hi = end_unit.min(units.len()).max(lo);
        let mut first: Option<usize> = None;
        let mut last: Option<usize> = None;
        for unit in units.iter().take(hi).skip(lo) {
            if unit.para_idx != para_idx {
                continue;
            }
            let Some(row) = unit.nested_row else {
                continue;
            };
            first = Some(first.map_or(row, |f: usize| f.min(row)));
            last = Some(last.map_or(row, |l: usize| l.max(row)));
        }
        match (first, last) {
            (Some(f), Some(l)) => Some((f, l + 1)),
            _ => None,
        }
    }

    /// RowBreak 분할의 컷 범위에 실제 보이는 내용이 남아 있는지 확인한다.
    ///
    /// 마지막 continuation 에 빈 문단/패딩만 남은 조각은 한컴 PDF에서 별도 페이지를
    /// 만들지 않는 경우가 있어, 페이지네이터가 terminal sliver 를 걸러낼 때 사용한다.
    pub(crate) fn row_cut_range_has_visible_content(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> bool {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|c| c.row as usize == row && c.row_span == 1)
            .collect();
        row_cells.sort_by_key(|c| c.col);

        for (i, cell) in row_cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let su = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let eu = end_cut
                .get(i)
                .copied()
                .unwrap_or(units.len())
                .clamp(su, units.len());
            if units[su..eu]
                .iter()
                .any(|unit| Self::cell_unit_has_visible_content(cell, unit))
            {
                return true;
            }
        }

        false
    }

    fn cell_unit_has_visible_content(cell: &crate::model::table::Cell, unit: &CellUnit) -> bool {
        if unit.nested_row.is_some() {
            return true;
        }

        let Some(para) = cell.paragraphs.get(unit.para_idx) else {
            return false;
        };
        !para.text.trim().is_empty() || !para.controls.is_empty()
    }

    /// [Task #1809] 종전 is_hwpx_source 조기 0 반환 제거 — 컷 이월 조각의 flow
    /// extra 는 소스 무관 기하다. 한글 편집기 대조(admrul_0072 서명 셀: 텍스트→
    /// 하단 경계 한글 25.5pt = extra 적용 25.9pt, 미적용 13.9pt)로 적용이 정답.
    ///
    /// [#4129] per-para O(P×U) 재스캔을 units 1-pass run-walk 로 재작성 (O(U)).
    /// mixed 유닛은 `cell_units_uncached` 의 단일 문단 루프(ascending `pi`)에서만
    /// 생성되므로 `para_idx` 가 유닛 순서상 단조 비감소 — 문단별 mixed run 이
    /// 연속 구간이다 (단조성은 아래 debug_assert 가 지킨다). 종전 구현과의 비트
    /// 동일성은 corpus 355개 전수 RHWP_2424_SHADOW A/B 대조로 검증했고, 게이트와
    /// reference 구현은 검증 완료 후 같은 PR 체인의 후속 레이어에서 제거했다.
    fn mixed_nested_flow_extra_from_cut(
        &self,
        cell: &crate::model::table::Cell,
        table: &crate::model::table::Table,
        styles: &ResolvedStyleSet,
        start_unit: usize,
        end_unit: usize,
    ) -> f64 {
        let units = self.cell_units(cell, table, styles);
        let lo = start_unit.min(units.len());
        let hi = end_unit.min(units.len()).max(lo);
        let mut extra = 0.0;
        // [#4129 회귀 가드] 실제 유닛 방문 횟수 집계 — run-walk 는 호출당 ≤2×U.
        // per-para 재스캔(O(P×U))류가 되살아나면 통합 테스트의 스캔 총량 상한이
        // 폭발한다. 반환 직전 한 번에 프로세스 카운터로 누적한다.
        let mut issue4129_visited: u64 = 0;

        let mut u = 0;
        while u < units.len() {
            // 종전 per-para 루프는 0..paragraphs.len() 이라 범위 밖 para_idx
            // 유닛은 방문 자체가 없었다 — 동일하게 무시한다.
            if !units[u].mixed_nested_fragment || units[u].para_idx >= cell.paragraphs.len() {
                issue4129_visited += 1;
                u += 1;
                continue;
            }
            let para_idx = units[u].para_idx;
            let mut offset = 0.0;
            let mut total = 0.0;
            let mut visible_units: Vec<(f64, bool)> = Vec::new();
            let mut has_recursive_fragment = false;
            let mut has_non_recursive_fragment = false;
            let mut idx = u;
            while idx < units.len() {
                issue4129_visited += 1;
                let unit = &units[idx];
                if unit.mixed_nested_fragment {
                    if unit.para_idx != para_idx {
                        break;
                    }
                    if unit.mixed_nested_recursive {
                        has_recursive_fragment = true;
                    } else {
                        has_non_recursive_fragment = true;
                    }
                    total += unit.height;
                    if idx < lo {
                        offset += unit.height;
                    }
                    if idx >= lo && idx < hi {
                        visible_units.push((unit.height, unit.mixed_nested_trailing));
                    }
                }
                idx += 1;
            }
            debug_assert!(
                idx >= units.len() || units[idx].para_idx > para_idx,
                "cell_units mixed para_idx 단조성 위반: {} 뒤에 {}",
                para_idx,
                units[idx].para_idx,
            );
            u = idx;

            if total <= 0.5 || offset <= 0.5 {
                continue;
            }
            while visible_units.last().is_some_and(|(_, trailing)| *trailing) {
                visible_units.pop();
            }
            let flow_visible: f64 = visible_units.iter().map(|(height, _)| *height).sum();
            if flow_visible <= 0.5 {
                continue;
            }
            let first_visible_content_height = visible_units
                .iter()
                .find_map(|(height, trailing)| (!*trailing).then_some(*height))
                .unwrap_or(0.0);
            let offset_within_start = (offset - first_visible_content_height).max(0.0);
            let terminal = end_unit >= units.len();
            let authoritative_recursive_run = self.profile.get().native_hwp5_layout()
                && has_recursive_fragment
                && !has_non_recursive_fragment;
            let single_cell_nested_continuation = table.row_count == 1
                && table.col_count == 1
                && cell.paragraphs.get(para_idx).is_some_and(|paragraph| {
                    paragraph.controls.iter().any(|control| {
                        matches!(control, Control::Table(nested) if nested.row_count == 1 && nested.col_count == 1)
                    })
                });
            let native_short_parent_child = cell
                .paragraphs
                .get(para_idx)
                .and_then(|paragraph| {
                    paragraph.controls.iter().find_map(|control| match control {
                        Control::Table(child) => Some(child.as_ref()),
                        _ => None,
                    })
                })
                .is_some_and(|child| {
                    self.native_short_parent_child_fragment_eligible(
                        table,
                        cell,
                        child,
                        self.nested_table_mixed_fragment_heights(child, styles)
                            .iter()
                            .map(|fragment| fragment.height)
                            .sum(),
                    )
                });
            // 재귀 투영 run은 `mixed_nested_split_from_cut`의 child RowCut이
            // source cursor와 viewport를 이미 함께 소유한다. 여기에 scalar
            // continuation 보정을 다시 더하면 부모 행만 첫 가시 유닛만큼 커져
            // 뒤 sibling을 다음 쪽으로 민다(59043 p36의 27.7px 중복). legacy
            // mixed fallback의 42065 p17 terminal 보정과 HWPX 저장 viewport
            // 보정은 유지한다.
            if offset_within_start > 0.5 && !authoritative_recursive_run {
                if terminal && native_short_parent_child {
                    // The native short-parent continuation replays its boundary
                    // source unit in the current fragment.  The generic parent
                    // extra would reserve that whole unit a second time and leave
                    // an empty row tail (76076 p82); retain only the mixed-flow
                    // clip guard.
                    extra += 4.0;
                } else if terminal && single_cell_nested_continuation {
                    // Keep the parent RowBreak cell in lockstep with the
                    // terminal nested-cell viewport.  Reserving only one
                    // unit leaves the parent clip above the nested tail, so
                    // the last ordinary paragraphs exist in the tree but
                    // disappear in SVG/Canvas (42065 p17 section 4).
                    extra += first_visible_content_height * 2.0 + 4.0;
                } else if !single_cell_nested_continuation {
                    extra += first_visible_content_height;
                }
            }
        }

        crate::diagnostics::perf_counters::MIXED_NESTED_UNITS_SCANNED
            .fetch_add(issue4129_visited, std::sync::atomic::Ordering::Relaxed);
        extra
    }

    /// [Task #993 / #1022] 분할 행에서 컷 범위 `[start_cut, end_cut)` 사이의
    /// **행 총 높이**(패딩 포함)를 반환한다. HeightMeasurer 와 정합 — 셀별로
    /// `max(cell.height, content + pad_cell)` 를 산출해 행 max.
    ///
    /// - 분할 아닌 행(start_cut/end_cut 모두 빈 Vec): `max(cell.height,
    ///   content+pad_cell)` per cell, row max.
    /// - 분할 행(컷 범위 일부): `content_in_range + pad_cell` per cell, row max.
    ///   분할 시 cell.height 강제는 적용하지 않는다(콘텐츠가 부분이므로).
    ///
    /// 셀 인덱스는 `advance_row_cut` 과 동일하게 `row_span==1` 셀을 col
    /// 오름차순 정렬한 순서다.
    pub(crate) fn row_cut_content_height(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|c| c.row as usize == row && c.row_span == 1)
            .collect();
        row_cells.sort_by_key(|c| c.col);
        let is_whole_row = start_cut.is_empty() && end_cut.is_empty();
        let mut max_h = 0.0f64;
        for (i, cell) in row_cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let su = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            let eu = end_cut
                .get(i)
                .copied()
                .unwrap_or(units.len())
                .clamp(su, units.len());
            let mixed_nested_extra = if is_whole_row {
                0.0
            } else {
                self.mixed_nested_flow_extra_from_cut(cell, table, styles, su, eu)
            };
            let trailing_trim = if is_whole_row {
                0.0
            } else {
                self.native_multirow_saved_reset_trailing_trim(table, cell, &units, eu, styles)
            };
            let content: f64 =
                (units[su..eu].iter().map(|u| u.height).sum::<f64>() - trailing_trim).max(0.0)
                    + mixed_nested_extra;
            let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
            let has_visible_cut = units[su..eu]
                .iter()
                .any(|unit| Self::cell_unit_has_visible_content(cell, unit));
            let pad_cell = if is_whole_row || has_visible_cut {
                pad_top + pad_bottom
            } else {
                0.0
            };
            let cell_h_px = if cell.height < 0x8000_0000 {
                hwpunit_to_px(cell.height as i32, self.dpi)
            } else {
                0.0
            };
            // [#2146] 저장 LINE_SEG 이 전혀 없고 모든 문단이 1줄(폭 여유 포함)인
            // 라벨 셀(사선 헤더 등)은 재합성 초과가 순수 줄높이 인플레이션 —
            // 선언 셀높이 신뢰. (21761835 r0: 선언 3928HU=52.4px = 한글 실측,
            // 재합성 79.3px) 판정 기준은 composer::no_ls_short_label_cell 주석 참조.
            let no_ls_label_cell = cell_h_px > 0.0 && {
                let (pad_left, pad_right, _, _) = self.resolve_cell_padding(cell, table);
                let cell_w_px = if cell.width < 0x8000_0000 {
                    hwpunit_to_px(cell.width as i32, self.dpi)
                        * self.render_table_width_scale(table)
                } else {
                    0.0
                };
                crate::renderer::composer::no_ls_short_label_cell(
                    cell,
                    table,
                    (cell_w_px - pad_left - pad_right).max(0.0),
                    cell_h_px - pad_top - pad_bottom,
                    styles,
                )
            };
            let h = if is_whole_row {
                if no_ls_label_cell {
                    cell_h_px
                } else {
                    // HeightMeasurer required_height + row 단계 1 cell.height max 정합.
                    (content + pad_cell).max(cell_h_px)
                }
            } else {
                // 분할 행 — cell.height 강제 없음.
                content + pad_cell
            };
            if h > max_h {
                max_h = h;
            }
        }
        max_h
    }

    /// 새 physical page에 온전히 들어갈 1×1 중첩 표 래퍼의 선행 문단 묶음을
    /// 현재 쪽 끝에 고립시키지 않아야 하는지 판정한다.
    ///
    /// Native HWP5 `RowBreak` 표에는 비어 있는 outer host → 1×1 child table →
    /// child 본문/내부 표라는 저장 형상이 있다. outer 행의 잔여 공간에 child 본문의
    /// 앞 몇 줄만 소비하면, scalar child renderer는 그 줄을 현재 쪽에 칠하지만 한컴은
    /// 그 다음 내부 표 앞의 묶음을 새 쪽에서 함께 시작한다. 86712의 r27이 그 사례다.
    ///
    /// 이 규칙은 일반 문단/중첩 표에 적용하지 않는다. 다음 내부 표 source paragraph
    /// 직전의 mixed-unit 경계를 provenance로 식별하고, 그 전 묶음이 새 본문의 대부분을
    /// 채울 때만 true다. 단순히 새 쪽에 들어간다는 이유로 중간 길이의 도입부까지
    /// 이월하면 59043 p35처럼 PDF가 허용한 분할을 망가뜨린다. 따라서 실제로 한
    /// 페이지보다 긴 child나 이미 continuation인 행, 새 본문의 일부만 쓰는 prefix는
    /// 종전 CellUnit 분할을 유지한다.
    pub(crate) fn should_defer_fresh_rowbreak_wrapper_prefix(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        fresh_body_height: f64,
        styles: &ResolvedStyleSet,
    ) -> bool {
        if !start_cut.is_empty()
            || !self.profile.get().native_hwp5_layout()
            || table.common.treat_as_char
            || !matches!(table.page_break, TablePageBreak::RowBreak)
        {
            return false;
        }

        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .collect();
        row_cells.sort_by_key(|cell| cell.col);

        for (wrapper_index, wrapper_cell) in row_cells.iter().enumerate() {
            let Some(host) = wrapper_cell
                .paragraphs
                .iter()
                .find(|paragraph| Self::paragraph_hosts_single_cell_nested_table(paragraph))
            else {
                continue;
            };
            let Some(child) = host.controls.iter().find_map(|control| match control {
                Control::Table(table) if table.row_count == 1 && table.col_count == 1 => {
                    Some(table.as_ref())
                }
                _ => None,
            }) else {
                continue;
            };
            let Some(child_cell) = child.cells.first() else {
                continue;
            };
            let Some(first_inner_table_para) = child_cell.paragraphs.iter().position(|paragraph| {
                paragraph
                    .controls
                    .iter()
                    .any(|control| matches!(control, Control::Table(_)))
            }) else {
                if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                    eprintln!(
                        "DIAG_SCAN DEFER_WRAPPER_PREFIX? r={} c={} child_paras={} inner_table_para=none",
                        row,
                        wrapper_cell.col,
                        child_cell.paragraphs.len(),
                    );
                }
                continue;
            };

            let wrapper_units = self.cell_units(wrapper_cell, table, styles);
            let Some(inner_table_start) = wrapper_units.iter().position(|unit| {
                unit.mixed_nested_fragment
                    && !unit.mixed_nested_trailing
                    && unit.mixed_nested_source_para_idx == Some(first_inner_table_para)
            }) else {
                if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                    eprintln!(
                        "DIAG_SCAN DEFER_WRAPPER_PREFIX? r={} c={} child_inner_para={} units={} mixed_sources={:?}",
                        row,
                        wrapper_cell.col,
                        first_inner_table_para,
                        wrapper_units.len(),
                        wrapper_units
                            .iter()
                            .filter(|unit| unit.mixed_nested_fragment && !unit.mixed_nested_trailing)
                            .filter_map(|unit| unit.mixed_nested_source_para_idx)
                            .collect::<Vec<_>>(),
                    );
                }
                continue;
            };
            if inner_table_start == 0 {
                continue;
            }

            let partial_end = end_cut
                .get(wrapper_index)
                .copied()
                .unwrap_or(wrapper_units.len())
                .min(wrapper_units.len());
            if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                eprintln!(
                    "DIAG_SCAN DEFER_WRAPPER_PREFIX? r={} c={} inner_start={} partial_end={} start_cut={:?} end_cut={:?}",
                    row,
                    wrapper_cell.col,
                    inner_table_start,
                    partial_end,
                    start_cut,
                    end_cut,
                );
            }
            if partial_end == 0 || partial_end >= inner_table_start {
                continue;
            }

            // 형제 라벨 셀도 선행 묶음에 포함돼야 한다. 현재 조각에서 아직 남은
            // 형제 content가 있으면 이월이 행 구조를 바꾸므로 적용하지 않는다.
            if row_cells.iter().enumerate().any(|(index, cell)| {
                if index == wrapper_index {
                    return false;
                }
                let units = self.cell_units(cell, table, styles);
                end_cut.get(index).copied().unwrap_or(units.len()) < units.len()
            }) {
                continue;
            }

            let mut prefix_end = Vec::with_capacity(row_cells.len());
            for (index, cell) in row_cells.iter().enumerate() {
                if index == wrapper_index {
                    prefix_end.push(inner_table_start);
                } else {
                    prefix_end.push(self.cell_units(cell, table, styles).len());
                }
            }
            let prefix_height = self.row_cut_content_height(table, row, &[], &prefix_end, styles);
            // 이 helper는 scan 중 호출돼 `LayoutEngine::current_body_area`가 아직
            // 갱신되지 않은 경로가 있다. 현재 `TypesetState`가 보유한 fresh 본문
            // 높이를 호출자가 넘겨야 실제 다음 페이지 수용성을 판정할 수 있다.
            let body_height = fresh_body_height;
            if std::env::var("RHWP_DIAG_SCAN").is_ok() {
                eprintln!(
                    "DIAG_SCAN DEFER_WRAPPER_PREFIX_FIT r={} prefix={:.1} body={:.1} prefix_end={:?}",
                    row, prefix_height, body_height, prefix_end,
                );
            }
            // 86712 r27의 926.2 / 971.3px처럼 사실상 한 page를 이루는 prefix만
            // atomic start로 다룬다. 59043 r5의 569.1 / 971.3px 도입부는 PDF에서
            // 앞 page에 남아야 하므로 이 임계값 아래에서는 기존 분할을 보존한다.
            const NEAR_FULL_FRESH_BODY_RATIO: f64 = 0.80;
            if body_height > 0.0
                && prefix_height <= body_height + 0.5
                && prefix_height >= body_height * NEAR_FULL_FRESH_BODY_RATIO
            {
                return true;
            }
        }

        false
    }

    /// Fresh 1×1 wrapper fragment가 첫 내부 표를 paint하지 않는 마지막 RowCut을
    /// 반환한다.
    ///
    /// mixed projection에서 `end_cut`은 scalar child renderer에 inclusive owner로
    /// 전달된다. 따라서 첫 inner-table atom(예: unit 59) 바로 전 spacer(unit 58)를
    /// end로 넘겨도 표가 현재 fragment에 그려질 수 있다. 표 source와 그 선행 spacer
    /// 둘 다 다음 page로 넘기는 `table_atom - 2`가 안전한 경계다. 이 값은 source
    /// paragraph provenance로만 구하며 native HWP5 non-TAC RowBreak wrapper에만 적용한다.
    pub(crate) fn fresh_rowbreak_wrapper_safe_prefix_end_cut(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        end_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> Option<RowCut> {
        if !start_cut.is_empty()
            || !self.profile.get().native_hwp5_layout()
            || table.common.treat_as_char
            || !matches!(table.page_break, TablePageBreak::RowBreak)
        {
            return None;
        }
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|cell| cell.row as usize == row && cell.row_span == 1)
            .collect();
        row_cells.sort_by_key(|cell| cell.col);

        for (wrapper_index, wrapper_cell) in row_cells.iter().enumerate() {
            let Some(child) = wrapper_cell.paragraphs.iter().find_map(|paragraph| {
                if !Self::paragraph_hosts_single_cell_nested_table(paragraph) {
                    return None;
                }
                paragraph.controls.iter().find_map(|control| match control {
                    Control::Table(table) if table.row_count == 1 && table.col_count == 1 => {
                        Some(table.as_ref())
                    }
                    _ => None,
                })
            }) else {
                continue;
            };
            let Some(first_inner_table_para) = child.cells.first().and_then(|cell| {
                cell.paragraphs.iter().position(|paragraph| {
                    paragraph
                        .controls
                        .iter()
                        .any(|control| matches!(control, Control::Table(_)))
                })
            }) else {
                continue;
            };
            let units = self.cell_units(wrapper_cell, table, styles);
            let Some(first_table_atom) = units.iter().position(|unit| {
                unit.mixed_nested_fragment
                    && !unit.mixed_nested_trailing
                    && unit.mixed_nested_source_para_idx == Some(first_inner_table_para)
            }) else {
                continue;
            };
            let current_end = end_cut
                .get(wrapper_index)
                .copied()
                .unwrap_or(units.len())
                .min(units.len());
            if current_end != first_table_atom || first_table_atom < 2 {
                continue;
            }
            if row_cells.iter().enumerate().any(|(index, cell)| {
                index != wrapper_index
                    && end_cut
                        .get(index)
                        .copied()
                        .unwrap_or_else(|| self.cell_units(cell, table, styles).len())
                        < self.cell_units(cell, table, styles).len()
            }) {
                continue;
            }

            let mut safe_end_cut = end_cut.to_vec();
            safe_end_cut[wrapper_index] = first_table_atom - 2;
            return Some(safe_end_cut);
        }
        None
    }

    /// RowBreak 분할 예산에서 실제 남은 가시 내용이 있는 셀의 패딩만 예약한다.
    ///
    /// Q&A 표처럼 왼쪽 gutter 빈 셀에 큰 padding 이 들어간 행은 그 padding 때문에
    /// 오른쪽 답변 셀의 첫 줄까지 다음 쪽으로 밀릴 수 있다. 분할 행에서는 보이는
    /// cut 이 남은 셀의 padding 만 행 예산에 반영해 렌더러의 split 높이와 맞춘다.
    pub(crate) fn row_remaining_visible_padding_height(
        &self,
        table: &crate::model::table::Table,
        row: usize,
        start_cut: &[usize],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        let mut row_cells: Vec<&crate::model::table::Cell> = table
            .cells
            .iter()
            .filter(|c| c.row as usize == row && c.row_span == 1)
            .collect();
        row_cells.sort_by_key(|c| c.col);

        let mut max_padding = 0.0f64;
        for (i, cell) in row_cells.iter().enumerate() {
            let units = self.cell_units(cell, table, styles);
            let su = start_cut.get(i).copied().unwrap_or(0).min(units.len());
            if !units[su..]
                .iter()
                .any(|unit| Self::cell_unit_has_visible_content(cell, unit))
            {
                continue;
            }
            let (_, _, pad_top, pad_bottom) = self.resolve_cell_padding(cell, table);
            max_padding = max_padding.max(pad_top + pad_bottom);
        }
        max_padding
    }

    /// 줄 범위(line_ranges)에 해당하는 셀 콘텐츠의 실제 렌더링 높이를 계산한다.
    /// compute_cell_line_ranges()의 결과를 받아서, 렌더링될 줄들의 높이를 합산한다.
    /// MeasuredCell 규칙: 첫 문단 spacing_before 없음, 마지막 문단 spacing_after 없음,
    /// 셀 마지막 줄 line_spacing 제외.
    pub(crate) fn calc_visible_content_height_from_ranges(
        &self,
        composed_paras: &[ComposedParagraph],
        paragraphs: &[crate::model::paragraph::Paragraph],
        line_ranges: &[(usize, usize)],
        styles: &ResolvedStyleSet,
    ) -> f64 {
        self.calc_visible_content_height_from_ranges_with_offset(
            composed_paras,
            paragraphs,
            line_ranges,
            styles,
            0.0,
        )
    }

    /// calc_visible_content_height_from_ranges 의 확장판 — split_start 의 content_offset 을 받아서
    /// 한 페이지보다 큰 nested table 의 잔여 높이를 정확히 계산한다.
    /// [Task #362] split_start 시 nested table 잔여 높이 누락으로 row 높이가 잘못 계산되는 결함 정정.
    pub(crate) fn calc_visible_content_height_from_ranges_with_offset(
        &self,
        composed_paras: &[ComposedParagraph],
        paragraphs: &[crate::model::paragraph::Paragraph],
        line_ranges: &[(usize, usize)],
        styles: &ResolvedStyleSet,
        content_offset: f64,
    ) -> f64 {
        let para_count = paragraphs.len();
        let mut total = 0.0;
        let mut cum_pos = 0.0f64; // 누적 콘텐츠 위치 (compute_cell_line_ranges 와 동일)
        let first_visible_pi = line_ranges.iter().position(|&(s, e)| s < e);
        let _last_visible_pi = line_ranges.iter().rposition(|&(s, e)| s < e);
        for (pi, ((comp, para), &(start, end))) in composed_paras
            .iter()
            .zip(paragraphs.iter())
            .zip(line_ranges.iter())
            .enumerate()
        {
            let para_style = styles.para_styles.get(para.para_shape_id as usize);
            let is_last_para = pi + 1 == para_count;
            let line_count = comp.lines.len();
            let spacing_before = if pi > 0 {
                para_style.map(|s| s.spacing_before).unwrap_or(0.0)
            } else {
                0.0
            };
            let spacing_after = if !is_last_para {
                para_style.map(|s| s.spacing_after).unwrap_or(0.0)
            } else {
                0.0
            };
            let has_table_in_para = para.controls.iter().any(|c| matches!(c, Control::Table(_)));

            // [Task #362] nested table paragraph 의 실제 콘텐츠 높이
            // (compute_cell_line_ranges 와 동일한 시멘틱)
            let para_h = if line_count == 0 || has_table_in_para {
                let nested_h: f64 = para
                    .controls
                    .iter()
                    .map(|ctrl| {
                        if let Control::Table(t) = ctrl {
                            self.calc_nested_table_height(t, styles)
                        } else {
                            0.0
                        }
                    })
                    .sum();
                if line_count == 0 {
                    let h = if nested_h > 0.0 {
                        nested_h
                    } else {
                        hwpunit_to_px(400, self.dpi)
                    };
                    spacing_before + h + spacing_after
                } else {
                    let line_based_h: f64 = comp
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(li, line)| {
                            let h = hwpunit_to_px(line.line_height, self.dpi);
                            let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                            let is_cell_last_line = is_last_para && li + 1 == line_count;
                            let mut lh = if !is_cell_last_line { h + ls } else { h };
                            if li == 0 {
                                lh += spacing_before;
                            }
                            if li == line_count - 1 {
                                lh += spacing_after;
                            }
                            lh
                        })
                        .sum();
                    nested_h.max(line_based_h)
                }
            } else {
                0.0 // 일반 line 단위 처리는 아래 분기에서
            };

            // nested table paragraph 처리
            if (line_count == 0 || has_table_in_para) && start < end {
                // [Task #362] 한 페이지보다 큰 nested table 분할: 시작 위치가 offset 이전이면
                // 잔여 = para_end_pos - max(content_offset, para_start_pos)
                let para_start_pos = cum_pos;
                let para_end_pos = cum_pos + para_h;
                if content_offset > 0.0
                    && para_start_pos < content_offset
                    && para_end_pos > content_offset
                {
                    // 분할 케이스: offset 이후의 잔여만 누적
                    total += para_end_pos - content_offset;
                } else if content_offset > 0.0 && para_end_pos <= content_offset {
                    // 이전 페이지에서 다 표시됨
                } else {
                    // 전체 표시
                    total += para_h;
                }
                cum_pos = para_end_pos;
                continue;
            }

            if start >= end {
                // 보이지 않는 일반 paragraph: cum_pos 만 진행
                if has_table_in_para || line_count == 0 {
                    cum_pos += para_h;
                } else {
                    let line_based_h: f64 = comp
                        .lines
                        .iter()
                        .enumerate()
                        .map(|(li, line)| {
                            let h = hwpunit_to_px(line.line_height, self.dpi);
                            let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                            let is_cell_last_line = is_last_para && li + 1 == line_count;
                            let mut lh = if !is_cell_last_line { h + ls } else { h };
                            if li == 0 {
                                lh += spacing_before;
                            }
                            if li == line_count - 1 {
                                lh += spacing_after;
                            }
                            lh
                        })
                        .sum();
                    cum_pos += line_based_h;
                }
                continue;
            }

            let is_visible_first = Some(pi) == first_visible_pi;
            // spacing_before: 렌더링되는 첫 문단에서는 적용하지 않음
            if start == 0 && !is_visible_first {
                total += spacing_before;
            }
            for li in start..end {
                if li < line_count {
                    let line = &comp.lines[li];
                    let h = hwpunit_to_px(line.line_height, self.dpi);
                    let is_cell_last_line = is_last_para && li + 1 == line_count;
                    if !is_cell_last_line {
                        total += h + hwpunit_to_px(line.line_spacing, self.dpi);
                    } else {
                        total += h;
                    }
                }
            }
            // spacing_after: 마지막 문단에서는 적용하지 않음
            if end == comp.lines.len() && end > start && !is_last_para {
                total += spacing_after;
            }
            // cum_pos 갱신 (전체 paragraph 가 차지하는 위치)
            let line_based_h: f64 = comp
                .lines
                .iter()
                .enumerate()
                .map(|(li, line)| {
                    let h = hwpunit_to_px(line.line_height, self.dpi);
                    let ls = hwpunit_to_px(line.line_spacing, self.dpi);
                    let is_cell_last_line = is_last_para && li + 1 == line_count;
                    let mut lh = if !is_cell_last_line { h + ls } else { h };
                    if li == 0 {
                        lh += spacing_before;
                    }
                    if li == line_count - 1 {
                        lh += spacing_after;
                    }
                    lh
                })
                .sum();
            cum_pos += line_based_h;
        }
        total
    }
}

#[cfg(test)]
mod row_cut_tests {
    use super::{
        stored_layout_relocated_empty_rowbreak_picture_resets_offset,
        trailing_reservation_after_final_source_owner, CellUnit, LayoutEngine,
        MixedNestedOwnerMarker, RecursiveBlockPreludeRole,
    };
    use crate::model::control::Control;
    use crate::model::image::Picture;
    use crate::model::paragraph::{LineSeg, Paragraph};
    use crate::model::shape::{CommonObjAttr, TextWrap, VertRelTo};
    use crate::model::table::{Cell, Table};
    use crate::renderer::composer::{ComposedLine, ComposedParagraph, ComposedTextRun};
    use crate::renderer::style_resolver::ResolvedStyleSet;

    /// line_height=1200 HU (=16 px @96dpi), line_spacing=0 인 N줄 텍스트 문단.
    /// vpos 는 vpos_start 부터 1200 HU 간격. `.text` 가 비어 있어 [Task #1488]
    /// 가시성 게이트 기준으로 **비가시(빈)** 문단으로 취급된다.
    fn text_para(n_lines: usize, vpos_start: i32) -> Paragraph {
        Paragraph {
            text: "x".repeat(n_lines.max(1)),
            char_count: n_lines.max(1) as u32,
            line_segs: (0..n_lines)
                .map(|i| LineSeg {
                    vertical_pos: vpos_start + i as i32 * 1200,
                    line_height: 1200,
                    line_spacing: 0,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    /// `text_para` 와 동일한 line_seg 구조에 가시 텍스트를 더한 문단. [Task #1488]
    /// 가시성 게이트가 가시 문단으로 인식하므로 vpos 리셋이 하드 브레이크로 보존된다.
    /// line_seg 가 있으면 compose 가 line_seg 수만큼 줄을 만들므로 유닛 수는 보존된다.
    fn visible_text_para(n_lines: usize, vpos_start: i32) -> Paragraph {
        Paragraph {
            text: "가나다".to_string(),
            ..text_para(n_lines, vpos_start)
        }
    }

    /// [Task #1488] 비가시(빈 텍스트) 오버레이 스페이서 문단 — line_seg 만 갖고 가시
    /// 텍스트는 없다. `text_para` 가 (#stabilize-rowbreak 이후) 가시 "x" 를 갖게 되어,
    /// 빈-오버레이 게이트 검증용으로 빈 텍스트 문단을 별도 헬퍼로 분리한다.
    fn empty_overlay_para(n_lines: usize, vpos_start: i32) -> Paragraph {
        Paragraph {
            text: String::new(),
            char_count: 0,
            ..text_para(n_lines, vpos_start)
        }
    }

    fn cell(row: u16, col: u16, paragraphs: Vec<Paragraph>) -> Cell {
        Cell {
            row,
            col,
            row_span: 1,
            col_span: 1,
            width: 10000,
            paragraphs,
            ..Default::default()
        }
    }

    fn table(cells: Vec<Cell>) -> Table {
        let row_count = cells.iter().map(|c| c.row + 1).max().unwrap_or(1);
        let col_count = cells.iter().map(|c| c.col + 1).max().unwrap_or(1);
        Table {
            row_count,
            col_count,
            cells,
            ..Default::default()
        }
    }

    fn rowbreak_table(cells: Vec<Cell>) -> Table {
        Table {
            page_break: crate::model::table::TablePageBreak::RowBreak,
            ..table(cells)
        }
    }

    fn saved_reset_unit(
        height: f64,
        para_idx: usize,
        vis_end: usize,
        hard_break_before: bool,
    ) -> CellUnit {
        CellUnit {
            height,
            hard_break_before,
            stored_frame_break_before: hard_break_before,
            vpos_gap_before: false,
            para_idx,
            vis_start: 0,
            vis_end,
            nested_row: None,
            nested_table_fragment: None,
            mixed_nested_fragment: false,
            mixed_nested_trailing: false,
            mixed_nested_content_height: 0.0,
            mixed_nested_recursive: false,
            mixed_nested_starts_after_table: false,
            mixed_nested_source_para_idx: None,
            recursive_block_prelude_role: RecursiveBlockPreludeRole::None,
            top_and_bottom_flow: false,
            empty_spacer: false,
            non_inline_control_range: None,
        }
    }

    fn mixed_owner_marker(
        para_idx: usize,
        trailing: bool,
        content_height: f64,
        height: f64,
    ) -> MixedNestedOwnerMarker {
        MixedNestedOwnerMarker {
            para_idx,
            fragment: true,
            trailing,
            content_height,
            height,
        }
    }

    fn recursive_block_unit(height: f64, role: RecursiveBlockPreludeRole) -> CellUnit {
        CellUnit {
            height,
            hard_break_before: false,
            stored_frame_break_before: false,
            vpos_gap_before: false,
            para_idx: 0,
            vis_start: 0,
            vis_end: 1,
            nested_row: None,
            nested_table_fragment: None,
            mixed_nested_fragment: true,
            mixed_nested_trailing: false,
            mixed_nested_content_height: height,
            mixed_nested_recursive: true,
            mixed_nested_starts_after_table: false,
            mixed_nested_source_para_idx: None,
            recursive_block_prelude_role: role,
            top_and_bottom_flow: false,
            empty_spacer: false,
            non_inline_control_range: None,
        }
    }

    #[test]
    fn recursive_block_prelude_rewinds_already_fit_prefix_before_overflow() {
        let table = rowbreak_table(vec![]);
        let mut prior = recursive_block_unit(20.0, RecursiveBlockPreludeRole::None);
        prior.mixed_nested_fragment = false;
        prior.mixed_nested_recursive = false;
        let separator =
            recursive_block_unit(8.0, RecursiveBlockPreludeRole::ExplicitPageBreakSeparator);
        let heading = recursive_block_unit(
            12.0,
            RecursiveBlockPreludeRole::OneLineHeadingBeforeSingleCellTable,
        );
        let prefix = recursive_block_unit(55.0, RecursiveBlockPreludeRole::None);
        let pending = recursive_block_unit(10.0, RecursiveBlockPreludeRole::None);
        let units = vec![prior, separator, heading, prefix, pending];

        // prior + separator + heading + 첫 recursive 조각은 95px로 fit했지만,
        // 다음 10px 조각은 100px 예산을 넘는다. separator부터 fit한 prefix까지
        // 함께 다음 fragment로 되감아야 제목만 이전 쪽에 고립되지 않는다.
        let mut j = 4;
        let mut h = 95.0;
        assert!(
            LayoutEngine::rewind_rowbreak_orphan_heading_before_recursive_block(
                &table, &units, 0, 100.0, &mut j, &mut h,
            )
        );
        assert_eq!(j, 1);
        assert!((h - 20.0).abs() < 0.001);

        // 명시적 쪽 나누기가 없는 일반 prelude는 이미 recursive prefix가 들어간
        // 뒤까지 되감지 않는다. 저장 프레임 경계에 맞춘 정상 continuation을
        // 앞당기면 뒤쪽 모든 physical page owner가 한 쪽씩 밀린다.
        let units = vec![
            recursive_block_unit(20.0, RecursiveBlockPreludeRole::None),
            recursive_block_unit(8.0, RecursiveBlockPreludeRole::EmptySeparator),
            recursive_block_unit(
                12.0,
                RecursiveBlockPreludeRole::OneLineHeadingBeforeSingleCellTable,
            ),
            recursive_block_unit(55.0, RecursiveBlockPreludeRole::None),
            recursive_block_unit(10.0, RecursiveBlockPreludeRole::None),
        ];
        let mut j = 4;
        let mut h = 95.0;
        assert!(
            !LayoutEngine::rewind_rowbreak_orphan_heading_before_recursive_block(
                &table, &units, 0, 100.0, &mut j, &mut h,
            )
        );
        assert_eq!(j, 4);
        assert!((h - 95.0).abs() < 0.001);

        // 새 viewport가 separator부터 시작했다면 prefix가 일부 fit했더라도
        // separator까지 되감아 무진행 cut을 만들면 안 된다.
        let units = vec![
            recursive_block_unit(8.0, RecursiveBlockPreludeRole::ExplicitPageBreakSeparator),
            recursive_block_unit(
                12.0,
                RecursiveBlockPreludeRole::OneLineHeadingBeforeSingleCellTable,
            ),
            recursive_block_unit(55.0, RecursiveBlockPreludeRole::None),
            recursive_block_unit(40.0, RecursiveBlockPreludeRole::None),
        ];
        let mut j = 3;
        let mut h = 75.0;
        assert!(
            !LayoutEngine::rewind_rowbreak_orphan_heading_before_recursive_block(
                &table, &units, 0, 100.0, &mut j, &mut h,
            )
        );
        assert_eq!(j, 3);
        assert!((h - 75.0).abs() < 0.001);

        // prefix가 없는 기존 direct-next 형상도 같은 계약을 유지한다.
        let units = vec![
            recursive_block_unit(20.0, RecursiveBlockPreludeRole::None),
            recursive_block_unit(8.0, RecursiveBlockPreludeRole::EmptySeparator),
            recursive_block_unit(
                12.0,
                RecursiveBlockPreludeRole::OneLineHeadingBeforeSingleCellTable,
            ),
            recursive_block_unit(70.0, RecursiveBlockPreludeRole::None),
        ];
        let mut j = 3;
        let mut h = 40.0;
        assert!(
            LayoutEngine::rewind_rowbreak_orphan_heading_before_recursive_block(
                &table, &units, 0, 100.0, &mut j, &mut h,
            )
        );
        assert_eq!(j, 1);
        assert!((h - 20.0).abs() < 0.001);

        // 직전 recursive block 뒤에서 다음 prelude의 separator만 fit하고
        // pending 제목이 예산을 넘는 경우에도 separator를 다음 조각으로
        // 넘겨야 제목+재귀 block이 새 viewport에서 함께 시작한다.
        let units = vec![
            recursive_block_unit(20.0, RecursiveBlockPreludeRole::None),
            recursive_block_unit(8.0, RecursiveBlockPreludeRole::EmptySeparator),
            recursive_block_unit(
                12.0,
                RecursiveBlockPreludeRole::OneLineHeadingBeforeSingleCellTable,
            ),
            recursive_block_unit(70.0, RecursiveBlockPreludeRole::None),
        ];
        let mut j = 2;
        let mut h = 28.0;
        assert!(
            LayoutEngine::rewind_rowbreak_orphan_heading_before_recursive_block(
                &table, &units, 0, 30.0, &mut j, &mut h,
            )
        );
        assert_eq!(j, 1);
        assert!((h - 20.0).abs() < 0.001);
    }

    #[test]
    fn trailing_reservation_does_not_extend_before_later_source_owner() {
        let empty_reservation = mixed_owner_marker(7, true, 0.0, 3.75);
        let later_source_owner = mixed_owner_marker(7, false, 18.0, 18.0);
        let later_contentful_trailing_owner = mixed_owner_marker(7, true, 12.0, 12.0);

        assert_eq!(
            trailing_reservation_after_final_source_owner(
                7,
                Some(empty_reservation),
                [later_source_owner],
            ),
            0.0,
            "an empty reservation before a later source owner must not enlarge the scalar viewport"
        );
        assert_eq!(
            trailing_reservation_after_final_source_owner(
                7,
                Some(empty_reservation),
                [later_contentful_trailing_owner],
            ),
            0.0,
            "a contentful trailing unit is still a later source owner"
        );
        assert_eq!(
            trailing_reservation_after_final_source_owner(
                7,
                Some(empty_reservation),
                std::iter::empty(),
            ),
            3.75,
            "the final contentless reservation must preserve the last painted line"
        );
    }

    fn non_inline_picture_para(vpos_start: i32) -> Paragraph {
        let common = CommonObjAttr {
            width: 10_000,
            height: 8_000,
            treat_as_char: false,
            text_wrap: TextWrap::TopAndBottom,
            vert_rel_to: VertRelTo::Para,
            vertical_offset: 1_000,
            flow_with_text: true,
            ..Default::default()
        };
        Paragraph {
            text: "그림".to_string(),
            char_count: 2,
            line_segs: vec![LineSeg {
                vertical_pos: vpos_start,
                line_height: 1200,
                line_spacing: 0,
                ..Default::default()
            }],
            controls: vec![Control::Picture(Box::new(Picture {
                common,
                ..Default::default()
            }))],
            ..Default::default()
        }
    }

    fn empty_anchor_non_inline_picture_para(vpos_start: i32) -> Paragraph {
        let mut para = non_inline_picture_para(vpos_start);
        para.text.clear();
        para.char_count = 0;
        para
    }

    #[test]
    fn stored_layout_relocated_empty_rowbreak_picture_uses_outer_host_vpos() {
        let mut para = empty_anchor_non_inline_picture_para(0);
        let Control::Picture(picture) = &mut para.controls[0] else {
            panic!("그림 컨트롤 아님");
        };
        picture.common.vertical_offset = (-52_790i32) as u32;

        let cell = cell(0, 0, vec![para.clone()]);
        let mut host = rowbreak_table(vec![cell.clone()]);
        host.common = CommonObjAttr {
            treat_as_char: false,
            text_wrap: TextWrap::TopAndBottom,
            vert_rel_to: VertRelTo::Para,
            vertical_offset: 560,
            ..Default::default()
        };
        let Control::Picture(picture) = &para.controls[0] else {
            panic!("그림 컨트롤 아님");
        };

        assert!(
            stored_layout_relocated_empty_rowbreak_picture_resets_offset(
                true,
                true,
                Some(52_230),
                &host,
                &cell,
                &para,
                picture,
            )
        );
        assert!(
            !stored_layout_relocated_empty_rowbreak_picture_resets_offset(
                true,
                true,
                Some(52_220),
                &host,
                &cell,
                &para,
                picture,
            )
        );
        assert!(
            !stored_layout_relocated_empty_rowbreak_picture_resets_offset(
                false,
                true,
                Some(52_230),
                &host,
                &cell,
                &para,
                picture,
            )
        );
    }

    #[test]
    fn native_hwp5_same_page_stale_empty_rowbreak_picture_resets_offset() {
        let mut para = empty_anchor_non_inline_picture_para(0);
        let Control::Picture(picture) = &mut para.controls[0] else {
            panic!("그림 컨트롤 아님");
        };
        picture.common.vertical_offset = (-50_000i32) as u32;

        let cell = cell(0, 0, vec![para.clone()]);
        let mut host = rowbreak_table(vec![cell.clone()]);
        host.common = CommonObjAttr {
            treat_as_char: false,
            text_wrap: TextWrap::TopAndBottom,
            vert_rel_to: VertRelTo::Para,
            vertical_offset: 0,
            ..Default::default()
        };
        let Control::Picture(picture) = &para.controls[0] else {
            panic!("그림 컨트롤 아님");
        };

        assert!(
            stored_layout_relocated_empty_rowbreak_picture_resets_offset(
                true,
                true,
                Some(12_000),
                &host,
                &cell,
                &para,
                picture,
            ),
            "native HWP5의 page-scale stale picture offset은 current cell top으로 reset해야 한다"
        );
        assert!(
            !stored_layout_relocated_empty_rowbreak_picture_resets_offset(
                true,
                false,
                Some(12_000),
                &host,
                &cell,
                &para,
                picture,
            ),
            "HWPX stored-layout에 native HWP5 stale-offset 규칙이 번지면 안 된다"
        );

        let Control::Picture(picture) = &mut para.controls[0] else {
            panic!("그림 컨트롤 아님");
        };
        picture.common.vertical_offset = (-39_999i32) as u32;
        let Control::Picture(picture) = &para.controls[0] else {
            panic!("그림 컨트롤 아님");
        };
        assert!(
            !stored_layout_relocated_empty_rowbreak_picture_resets_offset(
                true,
                true,
                Some(12_000),
                &host,
                &cell,
                &para,
                picture,
            ),
            "page-scale 기준보다 작은 일반 음수 offset은 보정하면 안 된다"
        );
    }

    #[test]
    fn test_topandbottom_flow_height_includes_margins() {
        // TopAndBottom + Para + flow_with_text 그림은 실제 렌더 y가
        // vertical_offset + margin.top부터 시작하므로, 예약 높이도
        // vertical_offset + margin.top + height + margin.bottom이어야 한다.
        let eng = LayoutEngine::new(96.0);
        let mut para = non_inline_picture_para(0);
        let Control::Picture(pic) = &mut para.controls[0] else {
            panic!("그림 컨트롤 아님");
        };
        pic.common.vertical_offset = 720;
        pic.common.height = 7200;
        pic.common.margin.top = 720;
        pic.common.margin.bottom = 1440;

        let h = eng.paragraph_cell_non_inline_controls_flow_height(&para.controls);
        assert!(
            (h - 134.4).abs() < 0.01,
            "TopAndBottom flow height에 margin이 포함되어야 함: {h}"
        );
    }

    fn composed_text(text: &str) -> ComposedParagraph {
        ComposedParagraph {
            lines: vec![ComposedLine {
                runs: vec![ComposedTextRun {
                    text: text.to_string(),
                    ..Default::default()
                }],
                line_height: 1000,
                baseline_distance: 850,
                segment_width: 1000,
                column_start: 0,
                line_spacing: 0,
                has_line_break: false,
                char_start: 0,
            }],
            para_style_id: 0,
            inline_controls: Vec::new(),
            numbering_text: None,
            tac_controls: Vec::new(),
            footnote_positions: Vec::new(),
            tab_extended: Vec::new(),
        }
    }

    #[test]
    fn test_shrink_cell_padding_preserves_explicit_cell_margin() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let composed = vec![composed_text("12345678901234567890")];
        let paragraphs = vec![Paragraph::default()];

        let shrunk = eng.shrink_cell_padding_for_overflow(
            20.0,
            20.0,
            30.0,
            &composed,
            &paragraphs,
            &styles,
            false,
        );
        assert!(
            shrunk.0 < 20.0 || shrunk.1 < 20.0,
            "일반 셀의 기존 오버플로우 방어는 유지되어야 함: {shrunk:?}"
        );

        let preserved = eng.shrink_cell_padding_for_overflow(
            20.0,
            20.0,
            30.0,
            &composed,
            &paragraphs,
            &styles,
            true,
        );
        assert_eq!(
            preserved,
            (20.0, 20.0),
            "안 여백 지정 셀은 한컴처럼 입력한 좌우 여백을 렌더링에서도 보존해야 함"
        );
    }

    #[test]
    fn test_advance_row_cut_basic_split() {
        // 1행 1셀, 6줄(각 16px). avail=50 → 3줄(48px) 소비, 4번째(64px)는 초과.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![cell(0, 0, vec![text_para(6, 0)])]);
        let r = eng.advance_row_cut(&t, 0, &[], 50.0, &styles);
        assert_eq!(r.end_cut, vec![3]);
        assert!(!r.fully_consumed);
        assert!(!r.hit_hard_break);
        assert!((r.consumed_height - 48.0).abs() < 0.5);
    }

    #[test]
    fn test_advance_row_cut_fully_consumed() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![cell(0, 0, vec![text_para(6, 0)])]);
        let r = eng.advance_row_cut(&t, 0, &[], 500.0, &styles);
        assert_eq!(r.end_cut, vec![6]);
        assert!(r.fully_consumed);
    }

    #[test]
    fn test_advance_row_cut_force_progress() {
        // avail 이 한 줄(16px)보다 작아도 시작 유닛 1개는 강제 소비 — 무한 루프 방지.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![cell(0, 0, vec![text_para(6, 0)])]);
        let r = eng.advance_row_cut(&t, 0, &[], 5.0, &styles);
        assert_eq!(r.end_cut, vec![1]);
        assert!(!r.fully_consumed);
    }

    #[test]
    fn test_advance_row_cut_rowbreak_grace_denied_in_continuous_visible_run() {
        // [Task #1718 v2] over-fill grace 는 오버플로 꼬리줄과 첫 spacer 사이가
        // "끊김 없는 가시 텍스트 줄의 연속(run)" 이면 거부한다 — 거대 RowBreak 셀 본문
        // 한복판(spacer 는 저 멀리)에서 grace 가 걸려 페이지당 +1~5줄 과충전 →
        // under-pagination(승강기 별표27: 40 vs 한글 48) 을 막는다.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![
                visible_text_para(6, 0),     // 가시 6유닛 (vpos 0,1200,..6000)
                empty_overlay_para(1, 7200), // spacer 는 가시 run 뒤에 위치
            ],
        )]);
        // avail=52px: 3줄(48px) 소비, 4번째(64px)는 +12px 초과(<120 tolerance).
        // 첫 spacer 전까지 units[4..6]=[가시,가시] 연속 run → grace 거부 → end_cut=[3].
        let r = eng.advance_row_cut(&t, 0, &[], 52.0, &styles);
        assert_eq!(
            r.end_cut,
            vec![3],
            "연속 가시 run 한복판에서는 over-fill grace 미적용"
        );
        assert!(
            r.consumed_height <= 52.5,
            "본문 초과 채움 금지: {}",
            r.consumed_height
        );
    }

    #[test]
    fn test_advance_row_cut_rowbreak_grace_kept_for_true_tail_before_spacers() {
        // [Task #1718] 오버플로 가시라인 바로 뒤가 spacer 면(진짜 꼬리줄) grace 유지 —
        // caption/꼬리줄 보존(byeolpyo1/4 over-pagination 방지 케이스 무회귀).
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![
                visible_text_para(4, 0),
                empty_overlay_para(1, 4800), // 바로 뒤 spacer → 진짜 꼬리줄
                empty_overlay_para(1, 6000),
            ],
        )]);
        let r = eng.advance_row_cut(&t, 0, &[], 52.0, &styles);
        assert!(
            r.end_cut[0] >= 4,
            "진짜 tail-before-spacer 는 grace 로 수용: {:?}",
            r.end_cut
        );
    }

    #[test]
    fn test_advance_row_cut_rowbreak_grace_denied_before_spacer_then_visible_text() {
        // 빈 줄 spacer 뒤에 다시 일반 가시 본문이 이어지면 구조적 꼬리줄이 아니라
        // 문단 사이 여백이므로 페이지 예산을 넘겨 끌어올리지 않는다.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![
                visible_text_para(4, 0),
                empty_overlay_para(1, 4800),
                visible_text_para(2, 6000),
            ],
        )]);
        let r = eng.advance_row_cut(&t, 0, &[], 52.0, &styles);
        assert_eq!(
            r.end_cut,
            vec![3],
            "spacer 뒤 본문이 계속되면 tail-before-spacer grace 미적용"
        );
    }

    #[test]
    fn test_cell_cut_non_inline_controls_do_not_repeat_after_para_cut() {
        // 셀 안 non-inline 그림은 해당 문단의 유닛이 현재 컷에 들어올 때만 렌더
        // 후보다. 문단을 지난 뒤의 continuation 에서 되살리면 이전 쪽 그림이
        // 모든 페이지에 반복된다.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![non_inline_picture_para(0), visible_text_para(1, 1200)],
        )]);
        let cell_ref = &t.cells[0];
        let units = eng.cell_units(cell_ref, &t, &styles);
        let picture_unit = units
            .iter()
            .position(|unit| {
                unit.para_idx == 0
                    && unit.vis_start == unit.vis_end
                    && !unit.empty_spacer
                    && unit.nested_row.is_none()
                    && !unit.mixed_nested_fragment
            })
            .expect("그림 전용 유닛 존재");
        let after_picture_units = units
            .iter()
            .position(|unit| unit.para_idx == 1)
            .expect("두 번째 문단 유닛 존재");

        assert!(
            !eng.cell_cut_contains_non_inline_control_units(cell_ref, &t, &styles, 0, 1, 0),
            "그림 문단의 일반 텍스트 줄만 포함된 컷에서는 렌더하지 않음"
        );
        assert!(
            eng.cell_cut_contains_non_inline_control_units(
                cell_ref,
                &t,
                &styles,
                picture_unit,
                picture_unit + 1,
                0
            ),
            "그림 전용 유닛이 포함된 컷에서만 렌더 후보"
        );
        assert!(
            !eng.cell_cut_contains_non_inline_control_units(
                cell_ref,
                &t,
                &styles,
                after_picture_units,
                after_picture_units + 1,
                0
            ),
            "그림 문단을 지난 컷에서는 후속 페이지에 반복 렌더하지 않음"
        );
    }

    #[test]
    fn test_advance_row_cut_non_inline_flow_unit_is_atomic() {
        // TopAndBottom non-inline 그림의 흐름 높이를 줄 높이 조각으로 쪼개면
        // 한 그림이 여러 continuation 컷에 반복 렌더된다. 객체 흐름 유닛은
        // 현재 쪽에 온전히 들어가지 않으면 다음 쪽에서 통째로 시작해야 한다.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![non_inline_picture_para(0), visible_text_para(1, 1200)],
        )]);

        let r = eng.advance_row_cut(&t, 0, &[], 40.0, &styles);
        assert_eq!(r.end_cut, vec![1], "그림 앞 텍스트 줄까지만 들어감");
        assert!(!r.fully_consumed);

        let r2 = eng.advance_row_cut(&t, 0, &r.end_cut, 1_000.0, &styles);
        assert!(
            r2.end_cut[0] > r.end_cut[0],
            "다음 컷에서 그림 흐름 유닛이 전진함"
        );
    }

    #[test]
    fn test_advance_row_cut_non_inline_flow_unit_not_orphaned_before_spacer() {
        // RowBreak 거대 셀에서 TopAndBottom 그림 flow 유닛만 쪽 하단에 들어가고,
        // 바로 뒤 spacer 가 다음 쪽으로 밀리면 기준 렌더러보다 그림이 한 쪽 앞선다.
        // 그림 유닛+뒤 spacer 묶음이 함께 들어가지 못하면 그림 유닛부터 다음 조각으로 넘긴다.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![
                visible_text_para(1, 0),
                non_inline_picture_para(1200),
                empty_overlay_para(1, 2400),
                visible_text_para(1, 3600),
            ],
        )]);
        let units = eng.cell_units(&t.cells[0], &t, &styles);
        let picture_unit = units
            .iter()
            .position(|unit| {
                unit.vis_start == unit.vis_end
                    && !unit.empty_spacer
                    && unit.nested_row.is_none()
                    && !unit.mixed_nested_fragment
            })
            .expect("그림 flow 유닛 존재");
        let spacer_unit = picture_unit + 1;
        assert!(units[spacer_unit].empty_spacer, "그림 뒤 spacer 존재");

        let before_picture: f64 = units[..picture_unit].iter().map(|unit| unit.height).sum();
        let picture_height = units[picture_unit].height;
        let spacer_height = units[spacer_unit].height;
        let avail = before_picture + picture_height + spacer_height * 0.5;

        let r = eng.advance_row_cut(&t, 0, &[], avail, &styles);
        assert_eq!(
            r.end_cut,
            vec![picture_unit],
            "그림만 들어가고 뒤 spacer 가 빠지는 컷은 만들지 않음"
        );

        let b = eng.advance_row_block_cut(&t, 0, 1, &[], avail, &styles);
        assert_eq!(
            b.end_cut, r.end_cut,
            "행블록 컷도 같은 orphan 방지 조건을 적용"
        );

        let r2 = eng.advance_row_cut(&t, 0, &r.end_cut, 1_000.0, &styles);
        assert!(
            r2.end_cut[0] > spacer_unit,
            "다음 조각에서는 그림과 spacer 를 함께 전진"
        );
    }

    #[test]
    fn test_empty_anchor_topandbottom_flow_delayed_before_hard_break() {
        // 빈 anchor 문단의 TopAndBottom 그림은 저장 vpos hard break 직전까지 지연될 수 있다.
        // 이렇게 해야 그림은 다음 쪽 상단으로 넘기면서도 anchor 뒤 일반 텍스트는 이전 쪽에
        // 계속 채울 수 있다.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![
                visible_text_para(1, 0),
                empty_anchor_non_inline_picture_para(1200),
                empty_overlay_para(1, 2400),
                visible_text_para(2, 3600),
                visible_text_para(1, 1000),
            ],
        )]);
        let units = eng.cell_units(&t.cells[0], &t, &styles);
        let picture_unit = units
            .iter()
            .position(|unit| {
                unit.vis_start == unit.vis_end
                    && !unit.empty_spacer
                    && unit.nested_row.is_none()
                    && !unit.mixed_nested_fragment
            })
            .expect("지연된 그림 flow 유닛 존재");
        let hard_break_unit = units
            .iter()
            .position(|unit| unit.hard_break_before && unit.vis_start < unit.vis_end)
            .expect("저장 vpos hard break 유닛 존재");

        assert_eq!(
            picture_unit + 1,
            hard_break_unit,
            "빈 anchor 그림 flow 유닛은 다음 가시 hard break 직전에 배치"
        );
        assert!(
            units[..picture_unit]
                .iter()
                .any(|unit| unit.para_idx == 3 && unit.vis_start < unit.vis_end),
            "그림 anchor 뒤 일반 텍스트는 그림보다 앞서 흐를 수 있어야 함"
        );
    }

    #[test]
    fn test_advance_row_cut_vpos_reset_hard_break() {
        // 가시 텍스트 문단0(3줄 vpos 0..2400) + 가시 문단1(2줄 vpos 1000..) — 문단1
        // 시작 vpos 가 문단0 끝(3600)보다 작아 vpos 리셋 → 문단1 앞에서 강제 분할.
        // [Task #1488] 가시 문단 사이 리셋은 하드 브레이크로 보존(Task #993 의도).
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![cell(
            0,
            0,
            vec![visible_text_para(3, 0), visible_text_para(2, 1000)],
        )]);
        // avail 충분해도 리셋에서 정지.
        let r = eng.advance_row_cut(&t, 0, &[], 1000.0, &styles);
        assert_eq!(r.end_cut, vec![3]);
        assert!(r.hit_hard_break);
        assert!(!r.fully_consumed);
        // 다음 프래그먼트: 리셋 지점부터 재개 — 시작 유닛은 리셋이어도 소비.
        let r2 = eng.advance_row_cut(&t, 0, &r.end_cut, 1000.0, &styles);
        assert_eq!(r2.end_cut, vec![5]);
        assert!(r2.fully_consumed);
    }

    #[test]
    fn test_plain_text_saved_reset_is_distinct_from_control_local_reset() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let plain = table(vec![cell(
            0,
            0,
            vec![visible_text_para(3, 0), visible_text_para(2, 0)],
        )]);
        let plain_cut = eng.advance_row_cut(&plain, 0, &[], 1_000.0, &styles);
        assert!(eng.row_cut_ends_at_plain_text_saved_reset(
            &plain,
            0,
            &[],
            &plain_cut.end_cut,
            &styles,
        ));

        let mut control_owner = visible_text_para(2, 0);
        control_owner.controls = non_inline_picture_para(0).controls;
        let with_control = table(vec![cell(
            0,
            0,
            vec![visible_text_para(3, 0), control_owner],
        )]);
        let control_cut = eng.advance_row_cut(&with_control, 0, &[], 1_000.0, &styles);
        assert!(
            !eng.row_cut_ends_at_plain_text_saved_reset(
                &with_control,
                0,
                &[],
                &control_cut.end_cut,
                &styles,
            ),
            "control 문단의 로컬 vpos=0은 plain-text 물리 frame reset으로 승격하지 않음"
        );
    }

    #[test]
    fn test_saved_reset_trailing_trim_requires_plain_text_owners() {
        let eng = LayoutEngine::new(96.0);
        eng.set_layout_profile(crate::model::provenance::LayoutCompatibilityProfile::new(
            false, false, false, false, false, true,
        ));
        let styles = ResolvedStyleSet::default();

        let mut previous = visible_text_para(1, 1_200);
        previous.line_segs[0].line_spacing = 600;
        let next = visible_text_para(1, 0);
        let mut host = rowbreak_table(vec![
            cell(0, 0, vec![previous.clone(), next.clone()]),
            cell(1, 0, vec![visible_text_para(1, 0)]),
        ]);
        host.common = CommonObjAttr {
            treat_as_char: false,
            text_wrap: TextWrap::TopAndBottom,
            ..Default::default()
        };

        let units = vec![
            saved_reset_unit(24.0, 0, 1, false),
            saved_reset_unit(16.0, 1, 1, true),
        ];

        let plain_cell = &host.cells[0];
        assert!(
            eng.native_multirow_saved_reset_trailing_trim(&host, plain_cell, &units, 1, &styles,)
                > 0.0,
            "plain-text 문단 사이 저장 reset은 마지막 줄의 trailing spacing을 trim"
        );

        let mut control_only = non_inline_picture_para(0);
        control_only.text.clear();
        control_only.char_count = 0;
        host.cells[0].paragraphs[1] = control_only;
        assert_eq!(
            eng.native_multirow_saved_reset_trailing_trim(
                &host,
                &host.cells[0],
                &units,
                1,
                &styles,
            ),
            0.0,
            "text -> control-only 로컬 vpos reset은 trailing spacing trim 대상이 아님"
        );

        let mut previous_with_control = previous;
        previous_with_control.controls = non_inline_picture_para(1_200).controls;
        host.cells[0].paragraphs = vec![previous_with_control, next];
        assert_eq!(
            eng.native_multirow_saved_reset_trailing_trim(
                &host,
                &host.cells[0],
                &units,
                1,
                &styles,
            ),
            0.0,
            "control을 소유한 이전 문단도 plain-text 저장 reset으로 승격하지 않음"
        );
    }

    #[test]
    fn test_block_cut_row_offsets_absorbs_sliver_before_stored_hard_break() {
        // [#1921] 예산 정지 지점 직후 48px 이내에 저장 hard-break(vpos 리셋)가 있으면
        // 그 지점까지 흡수한다. 흡수하지 않으면 다음 fragment 가 극소 잔여(여기서는
        // 16px 유닛 1개)만 담은 sliver 페이지가 된다 (59043 pi=160: 946px→22px 교대).
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        // 문단0: 3줄(vpos 0..2400) = 유닛 3개(각 16px). 문단1: vpos 1000 리셋
        // → 유닛 3 앞 hard break.
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![visible_text_para(3, 0), visible_text_para(2, 1000)],
        )]);
        // 예산 40px: 유닛 0..2(32px)까지 들어가고 유닛 2(16px)에서 예산 정지 —
        // 잔여(유닛 2, 16px) 직후가 hard break 이므로 48px 한도 내 흡수.
        let r = eng.advance_row_block_cut_with_row_offsets(&t, 0, 1, &[], 40.0, &[0.0], &styles);
        assert_eq!(
            r.end_cut,
            vec![3],
            "예산 정지 직후 hard-break 까지 흡수 (sliver 방지)"
        );
        assert!(r.hit_hard_break);
        assert!(!r.fully_consumed);
        assert!(
            r.consumed_height <= 40.0 + 48.0,
            "흡수 오버플로는 48px 한도 내: {}",
            r.consumed_height
        );
        // 다음 fragment: hard-break 유닛부터 잔여 전부 — sliver 없음.
        let r2 = eng.advance_row_block_cut_with_row_offsets(
            &t,
            0,
            1,
            &r.end_cut,
            1000.0,
            &[0.0],
            &styles,
        );
        assert!(r2.fully_consumed);
    }

    #[test]
    fn test_block_cut_row_offsets_no_absorb_beyond_tolerance() {
        // [#1921] hard-break 까지 잔여가 48px 를 넘으면 흡수하지 않는다 — 정상 예산
        // 분할 유지 (86712 공식PDF 핀 계열의 비정상 경계 강제 방지).
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        // 문단0: 8줄(128px). 예산 40px → 유닛 2에서 정지. hard break 는 유닛 8 앞
        // → 잔여 6유닛(96px) > 48px 한도 → 흡수 없음.
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![visible_text_para(8, 0), visible_text_para(2, 1000)],
        )]);
        let r = eng.advance_row_block_cut_with_row_offsets(&t, 0, 1, &[], 40.0, &[0.0], &styles);
        assert_eq!(r.end_cut, vec![2], "한도 초과 시 예산 경계 유지");
        assert!(!r.hit_hard_break);
    }

    #[test]
    fn test_advance_row_cut_hwpx_midpage_vpos_reset_is_absorbed() {
        // HWPX 저장 LINE_SEG vpos 리셋이어도 페이지 절반 이상이 남은 중간 리셋이면
        // 로컬 좌표 재시작으로 보고 같은 쪽에 이어 담는다.
        let eng = LayoutEngine::new(96.0);
        eng.set_layout_profile(crate::model::provenance::LayoutCompatibilityProfile::new(
            false, false, true, true, false, false,
        ));
        let styles = ResolvedStyleSet::default();
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![visible_text_para(4, 0), visible_text_para(2, 0)],
        )]);
        let r = eng.advance_row_cut(&t, 0, &[], 200.0, &styles);
        assert_eq!(
            r.end_cut,
            vec![6],
            "중간 vpos 리셋은 페이지 경계로 보존하지 않음"
        );
        assert!(r.fully_consumed);
    }

    #[test]
    fn test_advance_row_cut_hwpx_bottom_vpos_reset_is_preserved() {
        // 같은 HWPX 저장 리셋이라도 이미 페이지 하단 근처까지 채운 경우에는
        // 한컴 저장 쪽 경계로 보존한다.
        let eng = LayoutEngine::new(96.0);
        eng.set_layout_profile(crate::model::provenance::LayoutCompatibilityProfile::new(
            false, false, true, true, false, false,
        ));
        let styles = ResolvedStyleSet::default();
        let t = rowbreak_table(vec![cell(
            0,
            0,
            vec![visible_text_para(4, 0), visible_text_para(2, 0)],
        )]);
        let r = eng.advance_row_cut(&t, 0, &[], 80.0, &styles);
        assert_eq!(r.end_cut, vec![4], "하단 vpos 리셋은 저장 쪽 경계로 보존");
        assert!(!r.fully_consumed);
    }

    #[test]
    fn test_multi_page_single_cell_nested_reset_is_authoritative_in_parent_projection() {
        // [#3820 Stage 50] 페이지 하단에서 시작한 1×1 자식 표의 첫 fragment는
        // body 절반보다 짧을 수 있다. 단일 3600HU→0 reset이더라도 표 전체가
        // 물리 body보다 크면 부모 RowCut에 저장 쪽 경계로 투영해야 한다.
        let eng = LayoutEngine::new(96.0);
        eng.current_body_area.set((0.0, 0.0, 600.0, 120.0));
        let styles = ResolvedStyleSet::default();
        let nested = rowbreak_table(vec![cell(
            0,
            0,
            vec![visible_text_para(3, 0), visible_text_para(6, 0)],
        )]);

        let child_units = eng.cell_units(&nested.cells[0], &nested, &styles);
        let reset = child_units
            .iter()
            .position(|unit| unit.hard_break_before)
            .expect("단일 저장 vpos reset");
        assert!(
            !child_units[reset].stored_frame_break_before,
            "로컬 48px reset 자체는 body 절반(60px) 기준 authoritative가 아님"
        );

        let fragments = eng.nested_table_mixed_fragment_heights(&nested, &styles);
        assert_eq!(fragments.len(), child_units.len());
        assert!(fragments[reset].hard_break_before);
        assert!(
            fragments[reset].stored_frame_break_before,
            "물리 multi-page 1×1의 유일 reset은 부모 투영에서 authoritative"
        );
        assert!(fragments.iter().all(|fragment| fragment.recursive));
    }

    #[test]
    fn test_direct_hwpx_nested_resets_keep_legacy_parent_projection() {
        // [#3820 Stage 50/#3637] direct HWPX의 반복 vpos reset은 자식 셀의
        // 로컬 viewport 좌표다. reset 개수만으로 HWP5 canonical cursor를
        // 적용하면 마지막 RowBreak fragment와 후속 표가 새 쪽으로 밀린다.
        let eng = LayoutEngine::new(96.0);
        eng.set_layout_profile(crate::model::provenance::LayoutCompatibilityProfile::new(
            false, false, true, true, false, false,
        ));
        eng.current_body_area.set((0.0, 0.0, 600.0, 120.0));
        let styles = ResolvedStyleSet::default();
        let nested = rowbreak_table(vec![cell(
            0,
            0,
            vec![
                visible_text_para(3, 0),
                visible_text_para(3, 0),
                visible_text_para(3, 0),
            ],
        )]);

        let child_units = eng.cell_units(&nested.cells[0], &nested, &styles);
        assert_eq!(
            child_units
                .iter()
                .filter(|unit| unit.hard_break_before)
                .count(),
            2,
            "fixture는 direct HWPX 로컬 reset 두 개를 가져야 함"
        );

        let fragments = eng.nested_table_mixed_fragment_heights(&nested, &styles);
        assert!(
            fragments.iter().all(|fragment| !fragment.recursive),
            "direct HWPX reset은 HWP5 canonical child cursor로 승격하지 않음"
        );
    }

    #[test]
    fn test_advance_row_cut_empty_overlay_reset_no_hard_break() {
        // [Task #1488] 비가시(빈 텍스트) 오버레이 스페이서 문단이 만든 vpos 리셋은
        // 하드 브레이크가 아니다 — 셀 본문 위에 겹친 빈 문단들이 리셋마다 여분 빈
        // 페이지를 양산하던 회귀(rowbreak-problem-pages.hwpx sec1 pi=28)를 방지한다.
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![cell(
            0,
            0,
            vec![empty_overlay_para(3, 0), empty_overlay_para(2, 1000)],
        )]);
        let r = eng.advance_row_cut(&t, 0, &[], 1000.0, &styles);
        assert!(
            !r.hit_hard_break,
            "빈 오버레이 문단 리셋은 강제 분할하지 않음"
        );
        assert_eq!(r.end_cut, vec![5]);
        assert!(r.fully_consumed);
    }

    #[test]
    fn test_advance_row_cut_rowbreak_rewinds_internal_hard_break_orphan() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        // [Task #1488] 가시 텍스트 문단으로 구성 — 가시 문단 사이 리셋은 하드 브레이크
        // 보존(Task #993 의도)이라 rewind-orphan 로직이 그대로 검증된다.
        let internal_reset = Paragraph {
            text: "가나다".to_string(),
            line_segs: vec![
                LineSeg {
                    vertical_pos: 0,
                    line_height: 1200,
                    line_spacing: 0,
                    ..Default::default()
                },
                LineSeg {
                    vertical_pos: 0,
                    line_height: 1200,
                    line_spacing: 0,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let t = rowbreak_table(vec![
            rscell(0, 0, 2, vec![visible_text_para(1, 0)]),
            cell(
                1,
                1,
                vec![
                    visible_text_para(1, 0),
                    visible_text_para(1, 1200),
                    internal_reset,
                ],
            ),
        ]);

        let r = eng.advance_row_cut(&t, 1, &[], 1000.0, &styles);

        assert_eq!(r.end_cut, vec![2]);
        assert!(r.hit_hard_break);
        assert!(!r.fully_consumed);
    }

    #[test]
    fn test_advance_row_cut_multi_cell() {
        // 1행 2셀: 셀0=3줄, 셀1=6줄. avail 충분 → 각 셀 전부 소비,
        // consumed_height = 두 셀 표시 높이의 최댓값(셀1, 96px).
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![
            cell(0, 0, vec![text_para(3, 0)]),
            cell(0, 1, vec![text_para(6, 0)]),
        ]);
        let r = eng.advance_row_cut(&t, 0, &[], 500.0, &styles);
        assert_eq!(r.end_cut, vec![3, 6]);
        assert!(r.fully_consumed);
        assert!((r.consumed_height - 96.0).abs() < 0.5);
    }

    fn rscell(row: u16, col: u16, row_span: u16, paragraphs: Vec<Paragraph>) -> Cell {
        Cell {
            row,
            col,
            row_span,
            col_span: 1,
            width: 10000,
            paragraphs,
            ..Default::default()
        }
    }

    /// [Task #1025] 단일 비-rowspan 행에서 advance_row_block_cut == advance_row_cut (회귀 0).
    #[test]
    fn test_block_cut_single_row_parity() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![
            cell(0, 0, vec![text_para(3, 0)]),
            cell(0, 1, vec![text_para(6, 0)]),
        ]);
        for avail in [50.0, 96.0, 500.0, 5.0] {
            let a = eng.advance_row_cut(&t, 0, &[], avail, &styles);
            let b = eng.advance_row_block_cut(&t, 0, 1, &[], avail, &styles);
            assert_eq!(a.end_cut, b.end_cut, "avail={avail}");
            assert_eq!(a.fully_consumed, b.fully_consumed, "avail={avail}");
            assert_eq!(a.hit_hard_break, b.hit_hard_break, "avail={avail}");
            assert!(
                (a.consumed_height - b.consumed_height).abs() < 0.5,
                "avail={avail}"
            );
        }
    }

    /// [Task #1025] rowspan 블록(rows 0-1)에서 거대 row_span==1 셀이 줄 단위로 분할.
    /// cell[label] r=0 rs=2(2줄), cell[a] r=0(2줄), cell[big] r=1(10줄).
    /// avail=80px(=5줄): 첫 조각은 라벨2 + a2 + big5 까지, big 잔여 5줄은 다음 조각.
    #[test]
    fn test_block_cut_rowspan_giant_split() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let t = table(vec![
            rscell(0, 0, 2, vec![text_para(2, 0)]), // 라벨 (rows 0-1 걸침)
            cell(0, 1, vec![text_para(2, 0)]),      // row 0 일반 셀
            cell(1, 1, vec![text_para(10, 0)]),     // row 1 거대 셀 (10줄=160px)
        ]);
        // 셀 순서 (row,col): [ (0,0)라벨, (0,1)a, (1,1)big ]
        let first = eng.advance_row_block_cut(&t, 0, 2, &[], 80.0, &styles);
        // 라벨 2줄 전량, a 2줄 전량, big 5줄(80px) 까지.
        assert_eq!(first.end_cut, vec![2, 2, 5], "first: {:?}", first.end_cut);
        assert!(!first.fully_consumed);
        // 연속 조각: 라벨/a 는 이미 전량(공란), big 잔여 5줄.
        let cont = eng.advance_row_block_cut(&t, 0, 2, &first.end_cut, 500.0, &styles);
        assert_eq!(cont.end_cut, vec![2, 2, 10], "cont: {:?}", cont.end_cut);
        assert!(cont.fully_consumed);
    }

    /// [Issue #2214 Stage 3] 실제 deferred insert 호출부가 edited cell만 제거하는지
    /// 고정한다. #2214 fixture의 owner table-wide nested-text flag는 입력 전후 불변이므로
    /// flag와 same-table sibling identity를 함께 보존해야 한다.
    #[test]
    fn issue2214_deferred_insert_uses_scoped_cache_eviction() {
        use crate::document_core::DocumentCore;

        fn owner_table(core: &DocumentCore) -> &Table {
            match &core.document.sections[0].paragraphs[0].controls[2] {
                Control::Table(table) => table.as_ref(),
                other => panic!("#2214 owner control is not a table: {other:?}"),
            }
        }

        fn uncached_table_flag(table: &Table) -> bool {
            table.cells.iter().any(|cell| {
                cell.paragraphs.iter().any(|para| {
                    !para.text.trim().is_empty()
                        && para
                            .controls
                            .iter()
                            .any(|control| matches!(control, Control::Table(_)))
                })
            })
        }

        let mut failures = Vec::new();
        for (format_label, relative) in [
            ("hwp", "samples/issue1949_giant_cell_nested_tables_perf.hwp"),
            (
                "hwpx",
                "samples/issue1949_giant_cell_nested_tables_perf.hwpx",
            ),
        ] {
            for (phase, preinsert_count) in [("stable", 0), ("flow-boundary", 43)] {
                let label = format!("{format_label}-{phase}");
                let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
                let bytes = std::fs::read(path).expect("read #2214 fixture");
                let mut core = DocumentCore::from_bytes(&bytes).expect("load #2214 fixture");
                assert_eq!(core.page_count(), 115, "{label}: initial page count");
                for inserted in 0..preinsert_count {
                    core.insert_text_in_cell_native_deferred_pagination(
                        0,
                        0,
                        2,
                        2,
                        5,
                        130 + inserted,
                        "1",
                    )
                    .expect("prepare flow boundary");
                }

                let (
                    table_key,
                    target_key,
                    sibling_key,
                    target_before,
                    sibling_before,
                    target_shape_before,
                    owner_flag_before,
                    target_units_fp_before,
                ) = {
                    let table = owner_table(&core);
                    let target = &table.cells[2];
                    let sibling = &table.cells[1];
                    let target_before = core.layout_engine.cell_units(target, table, &core.styles);
                    let sibling_before =
                        core.layout_engine.cell_units(sibling, table, &core.styles);
                    let target_para = &target.paragraphs[5];
                    (
                        table as *const Table as usize,
                        target as *const Cell as usize,
                        sibling as *const Cell as usize,
                        target_before,
                        sibling_before,
                        (
                            !target_para.text.trim().is_empty(),
                            target_para
                                .controls
                                .iter()
                                .any(|control| matches!(control, Control::Table(_))),
                        ),
                        uncached_table_flag(table),
                        LayoutEngine::cell_paragraph_units_fingerprint(target_para),
                    )
                };
                assert!(
                    core.layout_engine
                        .table_nested_text_flag_cache
                        .borrow()
                        .contains_key(&table_key),
                    "{label}: owner flag must be warmed by cell units"
                );
                core.layout_engine.table_nested_text_flag_scan_count.set(0);

                core.insert_text_in_cell_native_deferred_pagination(
                    0,
                    0,
                    2,
                    2,
                    5,
                    130 + preinsert_count,
                    "1",
                )
                .expect("deferred one-char insert");
                assert_eq!(core.page_count(), 115, "{label}: deferred page count");

                let table = owner_table(&core);
                let target = &table.cells[2];
                let sibling = &table.cells[1];
                assert_eq!(
                    table as *const Table as usize, table_key,
                    "{label}: owner table pointer stability"
                );
                assert_eq!(
                    target as *const Cell as usize, target_key,
                    "{label}: target cell pointer stability"
                );
                assert_eq!(
                    sibling as *const Cell as usize, sibling_key,
                    "{label}: sibling cell pointer stability"
                );
                let target_para = &target.paragraphs[5];
                let target_shape_after = (
                    !target_para.text.trim().is_empty(),
                    target_para
                        .controls
                        .iter()
                        .any(|control| matches!(control, Control::Table(_))),
                );
                let owner_flag_after_uncached = uncached_table_flag(table);
                assert_eq!(
                    target_shape_after, target_shape_before,
                    "{label}: target visible-text/nested-table shape must be invariant"
                );
                assert_eq!(
                    owner_flag_after_uncached, owner_flag_before,
                    "{label}: owner table-wide flag must be invariant"
                );

                let membership = {
                    let cell_cache = core.layout_engine.cell_units_cache.borrow();
                    let flag_cache = core.layout_engine.table_nested_text_flag_cache.borrow();
                    (
                        cell_cache.contains_key(&target_key),
                        cell_cache.contains_key(&sibling_key),
                        flag_cache.contains_key(&table_key),
                    )
                };
                let target_after = core.layout_engine.cell_units(target, table, &core.styles);
                let sibling_after = core.layout_engine.cell_units(sibling, table, &core.styles);
                let owner_flag_after = core
                    .layout_engine
                    .table_has_visible_text_with_nested_table(table);
                let table_scan_count = core.layout_engine.table_nested_text_flag_scan_count.get();
                let target_recomputed = !std::sync::Arc::ptr_eq(&target_before, &target_after);
                let sibling_reused = std::sync::Arc::ptr_eq(&sibling_before, &sibling_after);
                // [#4167 갱신 이력] 기대값을 "무조건 evict"에서 "units 지문 변화 시에만
                // evict"로 변경. 제자리 1자 삽입은 units 지문(줄 수·높이·vpos·synthetic·
                // 공백 클래스) 불변이라 캐시 항등 보존이 정답이 됐다 — 재계산해도 동일
                // 벡터가 나옴은 issue4167_units_fingerprint_doc_contract 가 고정한다.
                // 실측상 두 phase 의 최종 삽입 모두 지문 불변(재래핑 없음)이며, 지문이
                // 변하는 편집의 evict 계약은 issue4167_fingerprint_unchanged_edit_
                // retains_cell_units 의 false 분기가 고정한다.
                let target_units_fp_after =
                    LayoutEngine::cell_paragraph_units_fingerprint(&target.paragraphs[5]);
                let fp_stable = target_units_fp_after == target_units_fp_before;
                let desired = if fp_stable {
                    membership == (true, true, true)
                        && !target_recomputed
                        && sibling_reused
                        && owner_flag_after == owner_flag_before
                        && table_scan_count == 0
                } else {
                    membership == (false, true, true)
                        && target_recomputed
                        && sibling_reused
                        && owner_flag_after == owner_flag_before
                        && table_scan_count == 0
                };
                eprintln!(
                    "#2214 {label}: membership={membership:?} target_recomputed={target_recomputed} sibling_reused={sibling_reused} owner_flag={owner_flag_before}->{owner_flag_after} table_scans={table_scan_count}"
                );
                if !desired {
                    failures.push(format!(
                        "{label}: membership={membership:?} target_recomputed={target_recomputed} sibling_reused={sibling_reused} owner_flag_stable={} table_scans={table_scan_count}",
                        owner_flag_after == owner_flag_before,
                    ));
                }
            }
        }

        assert!(
            failures.is_empty(),
            "deferred insert must use scoped cache eviction:\n{}",
            failures.join("\n")
        );
    }

    /// [Issue #2214 Stage 3] 실제 deferred insert가 빈 nested-table host를 non-empty로
    /// 바꿔 owner flag가 false→true가 되는 경우, owner table의 모든 cell units를 evict하고
    /// flag를 true로 갱신하되 nested table 자체의 cache는 보존해야 한다.
    #[test]
    fn issue2214_deferred_insert_flag_change_evicts_owner_cells() {
        use crate::document_core::DocumentCore;

        fn owner_table(core: &DocumentCore) -> &Table {
            match &core.document.sections[0].paragraphs[0].controls[2] {
                Control::Table(table) => table.as_ref(),
                other => panic!("#2214 owner control is not a table: {other:?}"),
            }
        }

        fn uncached_table_flag(table: &Table) -> bool {
            table.cells.iter().any(|cell| {
                cell.paragraphs.iter().any(|para| {
                    !para.text.trim().is_empty()
                        && para
                            .controls
                            .iter()
                            .any(|control| matches!(control, Control::Table(_)))
                })
            })
        }

        let mut failures = Vec::new();
        for (label, relative) in [
            ("hwp", "samples/issue1949_giant_cell_nested_tables_perf.hwp"),
            (
                "hwpx",
                "samples/issue1949_giant_cell_nested_tables_perf.hwpx",
            ),
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
            let bytes = std::fs::read(path).expect("read #2214 fixture");
            let mut core = DocumentCore::from_bytes(&bytes).expect("load #2214 fixture");
            let (host_cell, host_para, nested_control) = owner_table(&core)
                .cells
                .iter()
                .enumerate()
                .find_map(|(cell_index, cell)| {
                    cell.paragraphs
                        .iter()
                        .enumerate()
                        .find_map(|(para_index, para)| {
                            if !para.text.trim().is_empty() {
                                return None;
                            }
                            para.controls
                                .iter()
                                .enumerate()
                                .find_map(|(control_index, control)| match control {
                                    Control::Table(table) if !table.cells.is_empty() => {
                                        Some((cell_index, para_index, control_index))
                                    }
                                    _ => None,
                                })
                        })
                })
                .expect("#2214 fixture must contain an empty nested-table host");

            let (
                owner_table_key,
                owner_cell_keys,
                owner_before,
                nested_table_key,
                nested_cell_key,
                nested_before,
            ) = {
                let table = owner_table(&core);
                assert!(
                    !uncached_table_flag(table),
                    "{label}: owner flag must start false"
                );
                let nested =
                    match &table.cells[host_cell].paragraphs[host_para].controls[nested_control] {
                        Control::Table(table) => table.as_ref(),
                        other => panic!("nested control changed: {other:?}"),
                    };
                let owner_before = table
                    .cells
                    .iter()
                    .map(|cell| core.layout_engine.cell_units(cell, table, &core.styles))
                    .collect::<Vec<_>>();
                let nested_before =
                    core.layout_engine
                        .cell_units(&nested.cells[0], nested, &core.styles);
                (
                    table as *const Table as usize,
                    table
                        .cells
                        .iter()
                        .map(|cell| cell as *const Cell as usize)
                        .collect::<Vec<_>>(),
                    owner_before,
                    nested as *const Table as usize,
                    &nested.cells[0] as *const Cell as usize,
                    nested_before,
                )
            };
            assert_eq!(
                core.layout_engine
                    .table_nested_text_flag_cache
                    .borrow()
                    .get(&owner_table_key)
                    .copied(),
                Some(false),
                "{label}: cached owner flag before edit"
            );
            core.layout_engine.table_nested_text_flag_scan_count.set(0);

            core.insert_text_in_cell_native_deferred_pagination(
                0, 0, 2, host_cell, host_para, 0, "x",
            )
            .expect("deferred nested-host insert");
            assert_eq!(core.page_count(), 115, "{label}: deferred page count");

            let table = owner_table(&core);
            assert_eq!(
                table as *const Table as usize, owner_table_key,
                "{label}: owner table pointer stability"
            );
            assert!(
                uncached_table_flag(table),
                "{label}: nested-host insert must flip the uncached owner flag"
            );
            assert!(
                !table.cells[host_cell].paragraphs[host_para]
                    .text
                    .trim()
                    .is_empty(),
                "{label}: nested host text"
            );
            let nested =
                match &table.cells[host_cell].paragraphs[host_para].controls[nested_control] {
                    Control::Table(table) => table.as_ref(),
                    other => panic!("nested control changed: {other:?}"),
                };
            assert_eq!(
                nested as *const Table as usize, nested_table_key,
                "{label}: nested table pointer stability"
            );
            assert_eq!(
                &nested.cells[0] as *const Cell as usize, nested_cell_key,
                "{label}: nested cell pointer stability"
            );
            assert_eq!(
                table
                    .cells
                    .iter()
                    .map(|cell| cell as *const Cell as usize)
                    .collect::<Vec<_>>(),
                owner_cell_keys,
                "{label}: owner cell pointer stability"
            );

            let membership = {
                let cell_cache = core.layout_engine.cell_units_cache.borrow();
                let flag_cache = core.layout_engine.table_nested_text_flag_cache.borrow();
                (
                    owner_cell_keys
                        .iter()
                        .any(|key| cell_cache.contains_key(key)),
                    cell_cache.contains_key(&nested_cell_key),
                    flag_cache.get(&owner_table_key).copied(),
                    flag_cache.contains_key(&nested_table_key),
                )
            };
            let owner_after = table
                .cells
                .iter()
                .map(|cell| core.layout_engine.cell_units(cell, table, &core.styles))
                .collect::<Vec<_>>();
            let nested_after =
                core.layout_engine
                    .cell_units(&nested.cells[0], nested, &core.styles);
            let table_scan_count = core.layout_engine.table_nested_text_flag_scan_count.get();
            let owner_recomputed = owner_before
                .iter()
                .zip(&owner_after)
                .all(|(before, after)| !std::sync::Arc::ptr_eq(before, after));
            let nested_reused = std::sync::Arc::ptr_eq(&nested_before, &nested_after);
            let desired = membership == (false, true, Some(true), true)
                && owner_recomputed
                && nested_reused
                && table_scan_count == 0;
            eprintln!(
                "#2214 {label}-flag-change: membership={membership:?} owner_recomputed={owner_recomputed} nested_reused={nested_reused} table_scans={table_scan_count}"
            );
            if !desired {
                failures.push(format!(
                    "{label}: membership={membership:?} owner_recomputed={owner_recomputed} nested_reused={nested_reused} table_scans={table_scan_count}"
                ));
            }
        }

        assert!(
            failures.is_empty(),
            "deferred flag change must use owner-wide scoped eviction:\n{}",
            failures.join("\n")
        );
    }

    /// [Issue #2214 Stage 3] owner table-wide flag가 불변이면 edited cell만 evict하고
    /// cached owner flag와 sibling/unrelated cache를 보존한다.
    #[test]
    fn issue2214_scoped_eviction_retains_unrelated_cache() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let edited_table = table(vec![
            cell(0, 0, vec![text_para(2, 0)]),
            cell(0, 1, vec![text_para(4, 0)]),
        ]);
        let unrelated_table = table(vec![cell(0, 0, vec![text_para(3, 0)])]);

        let edited_before = eng.cell_units(&edited_table.cells[0], &edited_table, &styles);
        let sibling_before = eng.cell_units(&edited_table.cells[1], &edited_table, &styles);
        let unrelated_before = eng.cell_units(&unrelated_table.cells[0], &unrelated_table, &styles);
        let _ = eng.table_has_visible_text_with_nested_table(&edited_table);
        let _ = eng.table_has_visible_text_with_nested_table(&unrelated_table);

        assert_eq!(
            eng.cell_units_cache.borrow().len(),
            3,
            "three warmed cell entries"
        );
        assert_eq!(
            eng.table_nested_text_flag_cache.borrow().len(),
            2,
            "two warmed table-flag entries"
        );

        let edited_cell_key = &edited_table.cells[0] as *const crate::model::table::Cell as usize;
        let sibling_cell_key = &edited_table.cells[1] as *const crate::model::table::Cell as usize;
        let unrelated_cell_key =
            &unrelated_table.cells[0] as *const crate::model::table::Cell as usize;
        let owner_table_key = &edited_table as *const crate::model::table::Table as usize;
        let unrelated_table_key = &unrelated_table as *const crate::model::table::Table as usize;
        eng.invalidate_cell_units_after_text_edit(
            &edited_table.cells[0],
            &edited_table,
            false,
            false,
            false,
        );

        let cell_cache = eng.cell_units_cache.borrow();
        let flag_cache = eng.table_nested_text_flag_cache.borrow();
        let membership = (
            cell_cache.contains_key(&edited_cell_key),
            cell_cache.contains_key(&sibling_cell_key),
            cell_cache.contains_key(&unrelated_cell_key),
            flag_cache.contains_key(&owner_table_key),
            flag_cache.contains_key(&unrelated_table_key),
        );
        drop(cell_cache);
        drop(flag_cache);
        assert_eq!(
            membership,
            (false, true, true, true, true),
            "desired scoped membership: edited cell evicted; owner flag, sibling and unrelated caches retained"
        );

        let edited_after = eng.cell_units(&edited_table.cells[0], &edited_table, &styles);
        let sibling_after = eng.cell_units(&edited_table.cells[1], &edited_table, &styles);
        let unrelated_after = eng.cell_units(&unrelated_table.cells[0], &unrelated_table, &styles);
        assert!(
            !std::sync::Arc::ptr_eq(&edited_before, &edited_after),
            "edited cell units must be recomputed"
        );
        assert!(
            std::sync::Arc::ptr_eq(&sibling_before, &sibling_after),
            "same-table sibling units must be reused"
        );
        assert!(
            std::sync::Arc::ptr_eq(&unrelated_before, &unrelated_after),
            "unrelated-table units must be reused"
        );
    }

    /// [Issue #2214 Stage 3] cold false→true는 기존 owner cell cache가 없으므로
    /// owner-wide key 순회 없이 local witness로 flag=true를 기록한다.
    #[test]
    fn issue2214_cold_local_change_records_true_without_table_scan() {
        let eng = LayoutEngine::new(96.0);
        let nested_table = table(vec![cell(0, 0, vec![visible_text_para(1, 0)])]);
        let mut nested_host = text_para(1, 0);
        nested_host.text.clear();
        nested_host.char_count = 0;
        nested_host
            .controls
            .push(Control::Table(Box::new(nested_table)));
        let mut owner_table = rowbreak_table(vec![
            cell(0, 0, vec![nested_host]),
            cell(0, 1, vec![visible_text_para(2, 0)]),
        ]);
        let owner_table_key = &owner_table as *const Table as usize;

        assert!(eng.cell_units_cache.borrow().is_empty());
        assert!(eng.table_nested_text_flag_cache.borrow().is_empty());
        eng.table_nested_text_flag_scan_count.set(0);

        owner_table.cells[0].paragraphs[0].insert_text_at(0, "x");
        eng.invalidate_cell_units_after_text_edit(
            &owner_table.cells[0],
            &owner_table,
            false,
            true,
            false,
        );

        assert!(eng.cell_units_cache.borrow().is_empty());
        assert_eq!(
            eng.table_nested_text_flag_cache
                .borrow()
                .get(&owner_table_key)
                .copied(),
            Some(true)
        );
        assert!(eng.table_has_visible_text_with_nested_table(&owner_table));
        assert_eq!(eng.table_nested_text_flag_scan_count.get(), 0);
    }

    /// [Issue #2214 Stage 3] 다른 host가 이미 owner flag=true를 만든 상태에서 두 번째
    /// empty nested host가 non-empty가 되어도 table-wide 값은 불변이다. 이 branch는 edited
    /// cell만 evict하고 owner flag·다른 owner cells·unrelated cache를 보존해야 한다.
    #[test]
    fn issue2214_cached_true_local_change_evicts_edited_cell_only() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();

        let mut visible_host = visible_text_para(1, 0);
        visible_host
            .controls
            .push(Control::Table(Box::new(table(vec![cell(
                0,
                0,
                vec![visible_text_para(1, 0)],
            )]))));
        let mut empty_host = text_para(1, 0);
        empty_host.text.clear();
        empty_host.char_count = 0;
        empty_host
            .controls
            .push(Control::Table(Box::new(table(vec![cell(
                0,
                0,
                vec![visible_text_para(1, 0)],
            )]))));
        let mut edited_table = rowbreak_table(vec![
            cell(0, 0, vec![visible_host]),
            cell(0, 1, vec![empty_host]),
            cell(1, 0, vec![visible_text_para(2, 0)]),
            cell(1, 1, vec![visible_text_para(2, 0)]),
        ]);
        let unrelated_table = table(vec![cell(0, 0, vec![text_para(3, 0)])]);

        let owner_before = edited_table
            .cells
            .iter()
            .map(|cell| eng.cell_units(cell, &edited_table, &styles))
            .collect::<Vec<_>>();
        let unrelated_before = eng.cell_units(&unrelated_table.cells[0], &unrelated_table, &styles);
        assert!(
            eng.table_has_visible_text_with_nested_table(&edited_table),
            "first visible nested host must set owner flag=true"
        );
        let _ = eng.table_has_visible_text_with_nested_table(&unrelated_table);
        let owner_cell_keys = edited_table
            .cells
            .iter()
            .map(|cell| cell as *const crate::model::table::Cell as usize)
            .collect::<Vec<_>>();
        let unrelated_cell_key =
            &unrelated_table.cells[0] as *const crate::model::table::Cell as usize;
        let owner_table_key = &edited_table as *const crate::model::table::Table as usize;
        let unrelated_table_key = &unrelated_table as *const crate::model::table::Table as usize;
        eng.table_nested_text_flag_scan_count.set(0);

        edited_table.cells[1].paragraphs[0].insert_text_at(0, "x");
        eng.invalidate_cell_units_after_text_edit(
            &edited_table.cells[1],
            &edited_table,
            false,
            true,
            false,
        );

        let membership = {
            let cell_cache = eng.cell_units_cache.borrow();
            let flag_cache = eng.table_nested_text_flag_cache.borrow();
            (
                cell_cache.contains_key(&owner_cell_keys[1]),
                owner_cell_keys
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != 1)
                    .all(|(_, key)| cell_cache.contains_key(key)),
                cell_cache.contains_key(&unrelated_cell_key),
                flag_cache.get(&owner_table_key).copied(),
                flag_cache.contains_key(&unrelated_table_key),
            )
        };
        let owner_after = edited_table
            .cells
            .iter()
            .map(|cell| eng.cell_units(cell, &edited_table, &styles))
            .collect::<Vec<_>>();
        let unrelated_after = eng.cell_units(&unrelated_table.cells[0], &unrelated_table, &styles);
        let edited_recomputed = !std::sync::Arc::ptr_eq(&owner_before[1], &owner_after[1]);
        let siblings_reused = owner_before
            .iter()
            .zip(&owner_after)
            .enumerate()
            .filter(|(index, _)| *index != 1)
            .all(|(_, (before, after))| std::sync::Arc::ptr_eq(before, after));
        let unrelated_reused = std::sync::Arc::ptr_eq(&unrelated_before, &unrelated_after);
        let table_scan_count = eng.table_nested_text_flag_scan_count.get();
        assert!(
            membership == (false, true, true, Some(true), true)
                && edited_recomputed
                && siblings_reused
                && unrelated_reused
                && table_scan_count == 0,
            "cached-true local change scope: membership={membership:?} edited_recomputed={edited_recomputed} siblings_reused={siblings_reused} unrelated_reused={unrelated_reused} table_scans={table_scan_count}"
        );
    }

    /// [Issue #2214 Stage 3] owner table-wide nested-text flag가 바뀌면 같은 표의 모든
    /// cell units가 stale할 수 있다. 이때 owner-table-wide eviction은 허용하되 unrelated
    /// table cache는 보존해야 한다.
    #[test]
    fn issue2214_table_flag_change_evicts_owner_cells_only() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let nested_table = table(vec![cell(0, 0, vec![visible_text_para(1, 0)])]);
        let mut nested_host = text_para(1, 0);
        nested_host.text.clear();
        nested_host.char_count = 0;
        nested_host
            .controls
            .push(Control::Table(Box::new(nested_table)));
        let mut edited_table = rowbreak_table(vec![
            cell(0, 0, vec![nested_host]),
            cell(0, 1, vec![visible_text_para(2, 0)]),
            cell(1, 0, vec![visible_text_para(2, 0)]),
            cell(1, 1, vec![visible_text_para(2, 0)]),
        ]);
        let unrelated_table = table(vec![cell(0, 0, vec![text_para(3, 0)])]);

        let owner_before = edited_table
            .cells
            .iter()
            .map(|cell| eng.cell_units(cell, &edited_table, &styles))
            .collect::<Vec<_>>();
        let unrelated_before = eng.cell_units(&unrelated_table.cells[0], &unrelated_table, &styles);
        assert!(
            !eng.table_has_visible_text_with_nested_table(&edited_table),
            "empty nested host must start with a false owner flag"
        );
        let _ = eng.table_has_visible_text_with_nested_table(&unrelated_table);
        eng.table_nested_text_flag_scan_count.set(0);

        edited_table.cells[0].paragraphs[0].insert_text_at(0, "x");
        assert!(
            edited_table.cells.iter().any(|cell| {
                cell.paragraphs.iter().any(|para| {
                    !para.text.trim().is_empty()
                        && para
                            .controls
                            .iter()
                            .any(|control| matches!(control, Control::Table(_)))
                })
            }),
            "edit must flip the uncached owner flag to true"
        );

        let owner_cell_keys = edited_table
            .cells
            .iter()
            .map(|cell| cell as *const crate::model::table::Cell as usize)
            .collect::<Vec<_>>();
        let unrelated_cell_key =
            &unrelated_table.cells[0] as *const crate::model::table::Cell as usize;
        let owner_table_key = &edited_table as *const crate::model::table::Table as usize;
        let unrelated_table_key = &unrelated_table as *const crate::model::table::Table as usize;
        eng.invalidate_cell_units_after_text_edit(
            &edited_table.cells[0],
            &edited_table,
            false,
            true,
            false,
        );

        let membership = {
            let cell_cache = eng.cell_units_cache.borrow();
            let flag_cache = eng.table_nested_text_flag_cache.borrow();
            (
                owner_cell_keys
                    .iter()
                    .any(|key| cell_cache.contains_key(key)),
                cell_cache.contains_key(&unrelated_cell_key),
                flag_cache.get(&owner_table_key).copied(),
                flag_cache.contains_key(&unrelated_table_key),
            )
        };
        assert_eq!(
            membership,
            (false, true, Some(true), true),
            "flag change must evict all owner cells, update owner flag, and retain unrelated caches"
        );

        let owner_after = edited_table
            .cells
            .iter()
            .map(|cell| eng.cell_units(cell, &edited_table, &styles))
            .collect::<Vec<_>>();
        let unrelated_after = eng.cell_units(&unrelated_table.cells[0], &unrelated_table, &styles);
        let table_scan_count = eng.table_nested_text_flag_scan_count.get();
        assert!(
            owner_before
                .iter()
                .zip(&owner_after)
                .all(|(before, after)| !std::sync::Arc::ptr_eq(before, after)),
            "all owner-table cell units must be recomputed after owner flag change"
        );
        assert!(
            std::sync::Arc::ptr_eq(&unrelated_before, &unrelated_after),
            "unrelated-table units must be reused"
        );
        assert!(
            eng.table_has_visible_text_with_nested_table(&edited_table),
            "owner flag must recompute to true"
        );
        assert_eq!(
            table_scan_count, 0,
            "flag update and cache rewarm must not rescan the owner table"
        );
    }

    /// [Issue #2424] 삭제로 visible nested host가 비게 되면 true owner flag를
    /// 보수적으로 버리고 owner cell units만 다시 계산한다. unrelated cache는 유지한다.
    #[test]
    fn issue2424_delete_local_contribution_recomputes_owner_flag_only() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let nested_table = table(vec![cell(0, 0, vec![visible_text_para(1, 0)])]);
        let mut nested_host = visible_text_para(1, 0);
        nested_host
            .controls
            .push(Control::Table(Box::new(nested_table)));
        let mut owner_table = rowbreak_table(vec![
            cell(0, 0, vec![nested_host]),
            cell(0, 1, vec![visible_text_para(2, 0)]),
        ]);
        let unrelated_table = table(vec![cell(0, 0, vec![visible_text_para(3, 0)])]);

        let owner_before = owner_table
            .cells
            .iter()
            .map(|cell| eng.cell_units(cell, &owner_table, &styles))
            .collect::<Vec<_>>();
        let unrelated_before = eng.cell_units(&unrelated_table.cells[0], &unrelated_table, &styles);
        assert!(eng.table_has_visible_text_with_nested_table(&owner_table));
        let owner_table_key = &owner_table as *const Table as usize;
        let unrelated_table_key = &unrelated_table as *const Table as usize;
        eng.table_nested_text_flag_scan_count.set(0);

        owner_table.cells[0].paragraphs[0].text.clear();
        owner_table.cells[0].paragraphs[0].char_count = 0;
        eng.invalidate_cell_units_after_text_edit(
            &owner_table.cells[0],
            &owner_table,
            true,
            false,
            false,
        );

        assert!(
            owner_table.cells.iter().all(|cell| {
                let key = cell as *const crate::model::table::Cell as usize;
                !eng.cell_units_cache.borrow().contains_key(&key)
            }),
            "all owner cell units must be evicted"
        );
        assert!(
            !eng.table_nested_text_flag_cache
                .borrow()
                .contains_key(&owner_table_key),
            "owner flag must be recomputed after a true→false local change"
        );
        assert!(
            eng.table_nested_text_flag_cache
                .borrow()
                .contains_key(&unrelated_table_key),
            "unrelated flag must be retained"
        );

        let owner_after = owner_table
            .cells
            .iter()
            .map(|cell| eng.cell_units(cell, &owner_table, &styles))
            .collect::<Vec<_>>();
        let unrelated_after = eng.cell_units(&unrelated_table.cells[0], &unrelated_table, &styles);
        assert!(owner_before
            .iter()
            .zip(&owner_after)
            .all(|(before, after)| !std::sync::Arc::ptr_eq(before, after)));
        assert!(std::sync::Arc::ptr_eq(&unrelated_before, &unrelated_after));
        assert!(!eng.table_has_visible_text_with_nested_table(&owner_table));
        assert_eq!(
            eng.table_nested_text_flag_scan_count.get(),
            1,
            "owner table must be rescanned once after deletion"
        );
    }

    /// [#4167] 지문 불변 편집(제자리 타이핑)은 memoized cell units 를 보존한다.
    #[test]
    fn issue4167_fingerprint_unchanged_edit_retains_cell_units() {
        let eng = LayoutEngine::new(96.0);
        let styles = ResolvedStyleSet::default();
        let owner_table = table(vec![cell(0, 0, vec![text_para(3, 0)])]);
        let _ = eng.cell_units(&owner_table.cells[0], &owner_table, &styles);
        let key = &owner_table.cells[0] as *const crate::model::table::Cell as usize;
        assert!(eng.cell_units_cache.borrow().contains_key(&key), "warmed");

        eng.invalidate_cell_units_after_text_edit(
            &owner_table.cells[0],
            &owner_table,
            true,
            true,
            true,
        );
        assert!(
            eng.cell_units_cache.borrow().contains_key(&key),
            "지문 불변이면 entry 를 보존해야 한다 — 제거되면 거대 셀 타이핑마다 전량 recompose (#4167)"
        );

        eng.invalidate_cell_units_after_text_edit(
            &owner_table.cells[0],
            &owner_table,
            true,
            true,
            false,
        );
        assert!(
            !eng.cell_units_cache.borrow().contains_key(&key),
            "지문 변경이면 종전대로 제거해야 한다"
        );
    }

    /// [#4167] units 지문은 units 산출이 읽는 입력에만 반응한다.
    #[test]
    fn issue4167_units_fingerprint_sensitivity() {
        let base = text_para(3, 0);
        let fp = LayoutEngine::cell_paragraph_units_fingerprint;

        // 불변이어야 하는 변이: text_start·segment_width 시프트, synthetic 외 tag 비트
        let mut typed = base.clone();
        for seg in &mut typed.line_segs {
            seg.text_start += 1;
            seg.segment_width += 120;
            seg.tag |= 0x0010_0000; // 원본 로드 tag 잔여 비트 — units 미소비
        }
        assert_eq!(
            fp(&base),
            fp(&typed),
            "제자리 타이핑 급 변이는 지문 불변이어야 한다"
        );

        // 변해야 하는 변이: 줄 수, 줄 높이, synthetic 비트, 공백 클래스
        let mut grown = base.clone();
        grown.line_segs.push(grown.line_segs[0].clone());
        assert_ne!(fp(&base), fp(&grown), "줄 수 변화는 지문이 변해야 한다");

        let mut taller = base.clone();
        taller.line_segs[1].line_height += 240;
        assert_ne!(fp(&base), fp(&taller), "줄 높이 변화는 지문이 변해야 한다");

        let mut synthetic = base.clone();
        synthetic.line_segs[1].tag |= crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY;
        assert_ne!(
            fp(&base),
            fp(&synthetic),
            "synthetic 비트는 지문이 변해야 한다"
        );

        let mut spacer = base.clone();
        spacer.text = "   ".to_string();
        assert_ne!(
            fp(&base),
            fp(&spacer),
            "공백 스페이서 클래스 전이는 지문이 변해야 한다"
        );
    }
}
