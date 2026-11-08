//! 문단 테두리가 소비하는 실제 줄 점유와 문단 할당 영역.

use super::super::render_tree::{RenderNode, RenderNodeType};
use super::super::style_resolver::ResolvedParaStyle;
use super::LayoutRect;

/// 어울림 표의 추정 위치가 아니라 같은 단에 방출한 실제 테두리 끝을 소비한다.
/// 분할 표도 현재 조각의 원본 소유자가 일치할 때만 문단 외곽 확장의 근거가 된다.
pub(super) fn rendered_table_right_edge(
    column: &RenderNode,
    section_index: usize,
    para_index: usize,
    control_index: usize,
) -> Option<f64> {
    column.children.iter().rev().find_map(|node| {
        let RenderNodeType::Table(table) = &node.node_type else {
            return None;
        };
        (table.section_index == Some(section_index)
            && table.para_index == Some(para_index)
            && table.control_index == Some(control_index))
        .then_some(node.bbox.x + node.bbox.width)
    })
}

/// 문단 테두리의 간격 확장 전 세로 범위와 확장된 가로 범위를 구한다.
/// 앞뒤 테두리 간격은 연결 그룹의 첫/마지막 조각에서만 적용한다.
/// Square 어울림의 별도 박스는 호출자가 확정한 개체 포함 범위를 보존한다.
pub(super) fn paragraph_border_box(
    style: Option<&ResolvedParaStyle>,
    column: &LayoutRect,
    allocation_top: f64,
    line_top: f64,
    line_bottom: f64,
    flow_bottom: f64,
    wrap_box: Option<(f64, f64)>,
) -> LayoutRect {
    let ignore_margin = style.is_some_and(|style| style.border_ignore_margin);
    let (x, width) = if let Some(wrap_box) = wrap_box {
        wrap_box
    } else {
        let left = style.map_or(0.0, |style| {
            (if ignore_margin {
                style.margin_left
            } else {
                0.0
            }) - style.border_spacing[0]
        });
        let right = style.map_or(0.0, |style| {
            (if ignore_margin {
                style.margin_right
            } else {
                0.0
            }) - style.border_spacing[1]
        });
        (column.x + left, column.width - left - right)
    };
    let top = if ignore_margin {
        line_top
    } else {
        allocation_top
    };
    // 음수 줄간격으로 흐름 끝이 줄 상자 안으로 돌아가도 실제 점유는 보존한다.
    let bottom = if ignore_margin {
        line_bottom
    } else {
        flow_bottom.max(line_bottom)
    };
    LayoutRect {
        x,
        y: top,
        width,
        height: bottom - top,
    }
}
