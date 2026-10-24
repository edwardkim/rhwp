//! 열린 그리기 경로의 원본 화살표를 기존 공통 소비자에게 전달한다.

use crate::model::shape::DrawingObjAttr;
use crate::renderer::layout::utils::drawing_to_line_style;
use crate::renderer::render_tree::PathNode;
use crate::renderer::{ArrowStyle, PathCommand};

pub(super) fn publish(path: &mut PathNode, drawing: &DrawingObjAttr) {
    if path.style.stroke_color.is_none() {
        return;
    }
    let style = drawing_to_line_style(drawing);
    if style.start_arrow == ArrowStyle::None && style.end_arrow == ArrowStyle::None {
        return;
    }
    let Some((PathCommand::MoveTo(x1, y1), commands)) = path.commands.split_first() else {
        return;
    };
    let mut end = None;
    for command in commands {
        match *command {
            PathCommand::LineTo(x, y) | PathCommand::CurveTo(_, _, _, _, x, y) => {
                end = Some((x, y));
            }
            _ => return,
        }
    }
    let Some((x2, y2)) = end else {
        return;
    };
    if ![*x1, *y1, x2, y2].into_iter().all(f64::is_finite) {
        return;
    }
    path.connector_endpoints = Some((*x1, *y1, x2, y2));
    path.line_style = Some(style);
}
