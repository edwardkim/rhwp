//! 실제 그림 프레임에서 폭 0의 Print 채움 띠를 만든다.

use super::PaintOp;
use crate::renderer::render_tree::{BoundingBox, PathNode, RectangleNode};
use crate::renderer::{alpha_to_opacity, PathCommand, StrokeDash};

pub(super) fn cosmetic_ring(image: BoundingBox, rect: &RectangleNode) -> Option<PaintOp> {
    if rect.picture_border_width_hu != Some(0)
        || rect.print_picture_pen.is_some()
        || rect.style.stroke_dash != StrokeDash::Explicit([0.0; 6])
    {
        return None;
    }
    let pixel = rect.style.stroke_width;
    if ![image.x, image.y].into_iter().all(f64::is_finite)
        || ![image.width, image.height, pixel]
            .into_iter()
            .all(|extent| extent.is_finite() && extent > 0.0)
    {
        return None;
    }
    let (left, top) = (image.x, image.y);
    let (right, bottom) = (left + image.width, top + image.height);
    let bounds = BoundingBox::new(left, top, image.width + pixel, image.height + pixel);
    // 한컴의 채움 다각형은 오른쪽과 아래로 장치 한 픽셀을 더 차지한다.
    // 안쪽 경계는 반대 방향으로 닫아 그림 내용과 투명도를 보존한다.
    let mut commands = vec![
        PathCommand::MoveTo(left, top),
        PathCommand::LineTo(right + pixel, top),
        PathCommand::LineTo(right + pixel, bottom),
        PathCommand::LineTo(right, bottom + pixel),
        PathCommand::LineTo(left, bottom + pixel),
        PathCommand::ClosePath,
    ];
    if image.width > pixel && image.height > pixel {
        commands.extend([
            PathCommand::MoveTo(left + pixel, top + pixel),
            PathCommand::LineTo(left + pixel, bottom),
            PathCommand::LineTo(right, bottom),
            PathCommand::LineTo(right, top + pixel),
            PathCommand::ClosePath,
        ]);
    }
    let mut style = rect.style.clone();
    style.fill_color = Some(style.stroke_color.take()?);
    style.opacity *= alpha_to_opacity(style.stroke_alpha);
    Some(PaintOp::path(bounds, PathNode::new(commands, style, None)))
}
