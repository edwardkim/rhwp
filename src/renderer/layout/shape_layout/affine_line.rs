//! 묶음의 아핀 변환 끝점과 일반 선의 획·화살표 스타일을 함께 소비한다.
use super::*;

pub(super) fn rendered_line(
    line: &crate::model::shape::LineShape,
    start: (f64, f64),
    end: (f64, f64),
    dpi: f64,
) -> (RenderNodeType, BoundingBox) {
    let (start, end) = line_direction::endpoints(line.started_right_or_bottom, start, end);
    let (x1, y1) = start;
    let (x2, y2) = end;
    if line.drawing.border_line.attr & 0x3F != 0 {
        let rendered = LineNode::new(x1, y1, x2, y2, drawing_to_line_style(&line.drawing));
        let bbox = rendered.ink_bbox();
        (RenderNodeType::Line(rendered), bbox)
    } else {
        // 원본의 선 없음 속성은 기존 비가시 path로 보존한다.
        let (style, gradient) = drawing_to_shape_style(&line.drawing, dpi);
        (
            RenderNodeType::Path(PathNode::new(
                vec![PathCommand::MoveTo(x1, y1), PathCommand::LineTo(x2, y2)],
                style,
                gradient,
            )),
            BoundingBox::new(x1.min(x2), y1.min(y2), (x2 - x1).abs(), (y2 - y1).abs()),
        )
    }
}
