//! 각주 구분선을 공통 렌더 트리의 물리 선·경로로 만든다.
//!
//! HWP5 표 25·133과 한컴 저장본의 선 코드(없음 0 포함)를 사용한다.
//! 독립 Print `pdf/number_shapes/note-formats-hancom-mac-12.30.pdf`의 2~12쪽은
//! 600dpi pen 격자의 반복 선분·원·물결이며, 14~17쪽의 입체선은 잉크가 없다.
//! 색·선 종류가 잉크를 생략해도 선언된 각주 여백과 흐름 높이는 유지한다.

use super::super::render_tree::*;
use super::super::{LineStyle, PathCommand, ShapeStyle, StrokeDash};
use crate::model::footnote::FootnoteShape;
use crate::model::style::{BorderLine, BorderLineType};

#[allow(clippy::too_many_arguments)]
pub(super) fn append(
    tree: &mut PageLayoutContext,
    parent: &mut RenderNode,
    shape: &FootnoteShape,
    x: f64,
    y: f64,
    length: f64,
    width: f64,
) {
    let raw = shape.separator_line_type;
    if !crate::renderer::note_separator::has_ink(shape.separator_color)
        || raw == 0
        || (14..=17).contains(&raw)
        || length <= 0.0
    {
        return;
    }
    if raw == 7 {
        let grid = 96.0 / 600.0;
        let pen = (width / grid).round().max(1.0) * grid;
        let radius = pen * 0.75;
        let step = pen * 3.5 + grid;
        let style = ShapeStyle {
            fill_color: Some(shape.separator_color),
            stroke_color: Some(shape.separator_color),
            stroke_width: grid,
            ..Default::default()
        };
        let mut center = x;
        while center <= x + length {
            parent.children.push(RenderNode::new(
                tree.next_id(),
                RenderNodeType::Ellipse(EllipseNode::new(style.clone(), None)),
                BoundingBox::new(center - radius, y - radius, radius * 2.0, radius * 2.0),
            ));
            center += step;
        }
        return;
    }
    if raw == 12 {
        // Print에서 물결은 굵기의 1/4 pen으로 그린 사선과 1 pen 격자의 평탄부다.
        // 좌표는 다른 선 스타일과 같은 96dpi 공간이며 backend에서 다시 추측하지 않는다.
        let grid = 96.0 / 600.0;
        let height = (width / grid).round().max(1.0) * grid;
        let pen = (height / (4.0 * grid)).round().max(1.0) * grid;
        let top = y - height + grid;
        let bottom = y + grid;
        let mut commands = vec![PathCommand::MoveTo(x, top)];
        let mut current = x;
        let mut down = true;
        while current < x + length {
            let next = (current + height).min(x + length);
            let (start_y, end_y) = if down { (top, bottom) } else { (bottom, top) };
            let end_y = start_y + (end_y - start_y) * (next - current) / height;
            commands.push(PathCommand::LineTo(next, end_y));
            current = (next + grid).min(x + length);
            commands.push(PathCommand::LineTo(current, end_y));
            down = !down;
        }
        let style = ShapeStyle {
            stroke_color: Some(shape.separator_color),
            stroke_width: pen,
            ..Default::default()
        };
        let node = RenderNode::new(
            tree.next_id(),
            RenderNodeType::Path(PathNode::new(commands, style, None)),
            BoundingBox::new(x - pen / 2.0, top - pen / 2.0, length + pen, height + pen),
        );
        parent.children.push(node);
        return;
    }
    if !matches!(raw, 6 | 8..=11) {
        // 일반 선 종류는 새 base와 같은 LineType2 및 공통 Print 패턴을 소비한다.
        let border = BorderLine {
            line_type: super::border_rendering::border_line_type_from_code(raw),
            width: shape.separator_line_width,
            color: shape.separator_color,
        };
        parent
            .children
            .extend(super::border_rendering::create_border_line_nodes(
                tree,
                &border,
                x,
                y,
                x + length,
                y,
            ));
        return;
    }
    // 복선·삼중선은 각주가 선언한 전체 굵기를 공통 LineRenderType에 전달한다.
    // BorderFill 프레임용 3/4px 최소 밴드를 각주에 확대 적용하지 않는다.
    // 독립 Print(폭 index 7): 10쪽 획 0.96/0.48px, 11쪽 획 0.32/0.64/0.32px.
    let (mut dash, line_type) = crate::renderer::line_style::hwp_line_type(raw);
    if raw == 6 {
        // 긴 선은 쇄선의 긴 구간과 공백만 반복한다.
        if let Some(StrokeDash::Explicit(mut values)) = crate::renderer::stroke_pattern::border_dash(
            BorderLineType::DashDot,
            shape.separator_line_width,
        ) {
            values[2..].fill(0.0);
            dash = StrokeDash::Explicit(values);
        }
    }
    let line = LineNode::new(
        x,
        y,
        x + length,
        y,
        LineStyle {
            color: shape.separator_color,
            width,
            dash,
            line_type,
            ..Default::default()
        },
    );
    let bbox = line.ink_bbox();
    parent.children.push(RenderNode::new(
        tree.next_id(),
        RenderNodeType::Line(line),
        bbox,
    ));
}
