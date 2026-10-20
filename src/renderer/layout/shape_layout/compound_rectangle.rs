//! 사각형의 복합선을 모든 backend가 같은 닫힌 띠로 소비하게 한다.
//!
//! 정상 한컴 저장본의 독립 Print에서 60/425/1600 HU와 삼중선의
//! 3200/6400/10000 HU를 대조했다. 선폭은 600dpi 장치 격자에 반올림하고,
//! 띠의 부호 있는 경계는 위쪽 정수로 올린다. 화면 최소 px를 보강하지 않는다.
//! 근거: mydocs/working/task_m100_7642_stage1900.md.

use crate::model::shape::DrawingObjAttr;
use crate::renderer::render_tree::{
    BoundingBox, PageLayoutContext, PathNode, RenderNode, RenderNodeType,
};
use crate::renderer::{PathCommand, ShapeStyle, DEFAULT_DPI};

const DEVICE_GRID: f64 = DEFAULT_DPI / 600.0;

/// 원래 Rectangle의 채우기·흐름·소유를 유지하며 이미지 뒤에 테두리를 추가한다.
pub(super) fn append_border(
    tree: &mut PageLayoutContext,
    node: &mut RenderNode,
    drawing: &DrawingObjAttr,
) {
    let RenderNodeType::Rectangle(rect) = &mut node.node_type else {
        return;
    };
    let bands: &[(f64, f64)] = match drawing.border_line.attr & 0x3f {
        // 바깥쪽부터의 선/빈 간격 비율이다. 직선·표 괘선의 비율과 구분한다.
        8 => &[(0.0, 1.0 / 3.0), (2.0 / 3.0, 1.0)],
        9 => &[(0.0, 0.2), (0.5, 1.0)],
        10 => &[(0.0, 0.5), (0.8, 1.0)],
        11 => &[(0.0, 0.15), (0.31, 0.69), (0.85, 1.0)],
        _ => return,
    };
    let Some(color) = rect.style.stroke_color else {
        return;
    };
    // 폭이 없는 원본과 이미 stroke를 생략한 글상자는 기존 경로를 보존한다.
    if drawing.border_line.width <= 0 || rect.style.stroke_width <= 0.0 {
        return;
    }
    let bounds = node.bbox;
    if ![
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
        rect.corner_radius,
    ]
    .into_iter()
    .all(f64::is_finite)
        || bounds.width <= 0.0
        || bounds.height <= 0.0
    {
        return;
    }
    let device_width = (f64::from(drawing.border_line.width) / 12.0)
        .round()
        .max(1.0);
    let mut commands = Vec::new();
    for &(start, end) in bands {
        let outer = (start - 0.5) * device_width;
        let inner = (end - 0.5) * device_width;
        append_contour(&mut commands, bounds, rect.corner_radius, outer, false);
        append_contour(&mut commands, bounds, rect.corner_radius, inner, true);
    }
    let path = PathNode {
        section_index: rect.section_index,
        para_index: rect.para_index,
        control_index: rect.control_index,
        cell_index: rect.cell_index,
        cell_para_index: rect.cell_para_index,
        outer_table_control_index: rect.outer_table_control_index,
        // 같은 bbox로 부모와 같은 중심에서 회전·대칭을 한 번만 적용한다.
        transform: rect.transform,
        ..PathNode::new(
            commands,
            ShapeStyle {
                fill_color: Some(color),
                opacity: rect.style.opacity,
                ..ShapeStyle::default()
            },
            None,
        )
    };
    rect.style.stroke_color = None;
    rect.style.stroke_width = 0.0;
    let id = tree.next_id();
    node.children
        .push(RenderNode::new(id, RenderNodeType::Path(path), bounds));
}

/// 외곽과 반대 winding인 구멍을 같은 Path에 넣는다. 내부가 사라지면 구멍을 만들지 않는다.
fn append_contour(
    commands: &mut Vec<PathCommand>,
    bounds: BoundingBox,
    radius: f64,
    inset_units: f64,
    reverse: bool,
) {
    let inset_start = inset_units.ceil() * DEVICE_GRID;
    let inset_end = (-inset_units).ceil() * DEVICE_GRID;
    let x = bounds.x + inset_start;
    let y = bounds.y + inset_start;
    let right = bounds.x + bounds.width + inset_end;
    let bottom = bounds.y + bounds.height + inset_end;
    let w = right - x;
    let h = bottom - y;
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    // 둥글지 않은 모서리는 miter를 유지한다. 평행 곡선의 내부 반지름만 0이 될 수 있다.
    let r = if radius > 0.0 {
        (radius - inset_units * DEVICE_GRID)
            .max(0.0)
            .min(w.min(h) / 2.0)
    } else {
        0.0
    };
    let k = r * 0.552_284_749_830_793_6;
    commands.push(PathCommand::MoveTo(x + r, y));
    if r == 0.0 {
        if reverse {
            commands.push(PathCommand::LineTo(x, bottom));
            commands.push(PathCommand::LineTo(right, bottom));
            commands.push(PathCommand::LineTo(right, y));
        } else {
            commands.push(PathCommand::LineTo(right, y));
            commands.push(PathCommand::LineTo(right, bottom));
            commands.push(PathCommand::LineTo(x, bottom));
        }
        commands.push(PathCommand::ClosePath);
        return;
    }
    if reverse {
        commands.push(PathCommand::CurveTo(x + r - k, y, x, y + r - k, x, y + r));
        commands.push(PathCommand::LineTo(x, bottom - r));
        commands.push(PathCommand::CurveTo(
            x,
            bottom - r + k,
            x + r - k,
            bottom,
            x + r,
            bottom,
        ));
        commands.push(PathCommand::LineTo(right - r, bottom));
        commands.push(PathCommand::CurveTo(
            right - r + k,
            bottom,
            right,
            bottom - r + k,
            right,
            bottom - r,
        ));
        commands.push(PathCommand::LineTo(right, y + r));
        commands.push(PathCommand::CurveTo(
            right,
            y + r - k,
            right - r + k,
            y,
            right - r,
            y,
        ));
    } else {
        commands.push(PathCommand::LineTo(right - r, y));
        commands.push(PathCommand::CurveTo(
            right - r + k,
            y,
            right,
            y + r - k,
            right,
            y + r,
        ));
        commands.push(PathCommand::LineTo(right, bottom - r));
        commands.push(PathCommand::CurveTo(
            right,
            bottom - r + k,
            right - r + k,
            bottom,
            right - r,
            bottom,
        ));
        commands.push(PathCommand::LineTo(x + r, bottom));
        commands.push(PathCommand::CurveTo(
            x + r - k,
            bottom,
            x,
            bottom - r + k,
            x,
            bottom - r,
        ));
        commands.push(PathCommand::LineTo(x, y + r));
        commands.push(PathCommand::CurveTo(x, y + r - k, x + r - k, y, x + r, y));
    }
    commands.push(PathCommand::ClosePath);
}
