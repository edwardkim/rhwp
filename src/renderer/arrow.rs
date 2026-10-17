//! One arrow outline for SVG, Canvas, Skia and serialized CanvasKit replay.
//! Independent Hancom Print measurements and limits: task_m100_7688_stage5.md.

use super::{ArrowStyle, LineStyle, PathCommand};

pub(crate) struct ArrowHead {
    /// Local coordinates: tip at (0, 0), base along positive x.
    pub commands: Vec<PathCommand>,
    pub filled: bool,
}

/// HWP size codes enumerate length first, perpendicular width second.
/// The head retains its size on short lines; line length is not a size input.
pub(crate) fn dimensions(stroke_width: f64, size: u8) -> (f64, f64) {
    let size = if size <= 8 { size } else { 4 };
    let unit = stroke_width.max(3.84);
    (
        unit * (2 + 2 * (size / 3)) as f64,
        unit * (2 + 2 * (size % 3)) as f64,
    )
}

pub(crate) fn head(style: ArrowStyle, stroke_width: f64, size: u8) -> Option<ArrowHead> {
    let (length, width) = dimensions(stroke_width, size);
    head_with_dimensions(style, length, width)
}

pub(crate) fn head_with_dimensions(
    style: ArrowStyle,
    length: f64,
    width: f64,
) -> Option<ArrowHead> {
    use PathCommand::{ClosePath, CurveTo, LineTo, MoveTo};
    let h = width / 2.0;
    let mut commands = match style {
        ArrowStyle::None => return None,
        ArrowStyle::Arrow => vec![MoveTo(0.0, 0.0), LineTo(length, -h), LineTo(length, h)],
        ArrowStyle::ConcaveArrow => vec![
            MoveTo(0.0, 0.0),
            LineTo(length, -h),
            LineTo(length * 0.7, 0.0),
            LineTo(length, h),
        ],
        ArrowStyle::Diamond | ArrowStyle::OpenDiamond => vec![
            MoveTo(0.0, 0.0),
            LineTo(length / 2.0, -h),
            LineTo(length, 0.0),
            LineTo(length / 2.0, h),
        ],
        ArrowStyle::Square | ArrowStyle::OpenSquare => vec![
            MoveTo(0.0, -h),
            LineTo(length, -h),
            LineTo(length, h),
            LineTo(0.0, h),
        ],
        ArrowStyle::Circle | ArrowStyle::OpenCircle => {
            // Same ellipse as the previous paint paths, expressed as cubic curves.
            let cx = length / 2.0;
            let rx = length * 0.4;
            let ry = h * 0.8;
            let k = 4.0 * (std::f64::consts::PI / 8.0).tan() / 3.0;
            vec![
                MoveTo(cx + rx, 0.0),
                CurveTo(cx + rx, k * ry, cx + k * rx, ry, cx, ry),
                CurveTo(cx - k * rx, ry, cx - rx, k * ry, cx - rx, 0.0),
                CurveTo(cx - rx, -k * ry, cx - k * rx, -ry, cx, -ry),
                CurveTo(cx + k * rx, -ry, cx + rx, -k * ry, cx + rx, 0.0),
            ]
        }
    };
    commands.push(ClosePath);
    Some(ArrowHead {
        commands,
        filled: !matches!(
            style,
            ArrowStyle::OpenDiamond | ArrowStyle::OpenCircle | ArrowStyle::OpenSquare
        ),
    })
}

impl ArrowHead {
    /// Direction points out of the line; the outline extends back into it.
    pub(crate) fn at(&self, x: f64, y: f64, dx: f64, dy: f64) -> Vec<PathCommand> {
        let p = |along: f64, perp: f64| (x - dx * along + dy * perp, y - dy * along - dx * perp);
        self.commands
            .iter()
            .map(|command| match *command {
                PathCommand::MoveTo(a, b) => {
                    let (x, y) = p(a, b);
                    PathCommand::MoveTo(x, y)
                }
                PathCommand::LineTo(a, b) => {
                    let (x, y) = p(a, b);
                    PathCommand::LineTo(x, y)
                }
                PathCommand::CurveTo(a, b, c, d, e, f) => {
                    let (a, b) = p(a, b);
                    let (c, d) = p(c, d);
                    let (e, f) = p(e, f);
                    PathCommand::CurveTo(a, b, c, d, e, f)
                }
                PathCommand::ClosePath => PathCommand::ClosePath,
                PathCommand::ArcTo(..) => unreachable!("arrow outlines use cubic ellipses"),
            })
            .collect()
    }
}

pub(crate) fn line_heads(endpoints: (f64, f64, f64, f64), style: &LineStyle) -> Vec<ArrowHead> {
    heads(endpoints, style, None)
}

pub(crate) fn connector_heads(path: &super::render_tree::PathNode) -> Vec<ArrowHead> {
    match (&path.line_style, path.connector_endpoints) {
        (Some(style), Some(endpoints)) => heads(endpoints, style, Some(&path.commands)),
        _ => Vec::new(),
    }
}

fn heads(
    endpoints: (f64, f64, f64, f64),
    style: &LineStyle,
    commands: Option<&[PathCommand]>,
) -> Vec<ArrowHead> {
    let (x1, y1, x2, y2) = endpoints;
    let chord = (x2 - x1, y2 - y1);
    let mut start = (-chord.0, -chord.1);
    let mut end = chord;
    if let Some(commands) = commands {
        for command in commands.iter().skip(1) {
            let control = match *command {
                PathCommand::LineTo(x, y) | PathCommand::CurveTo(x, y, ..) => (x, y),
                _ => continue,
            };
            if control != (x1, y1) {
                start = (x1 - control.0, y1 - control.1);
                break;
            }
        }
        for pair in commands.windows(2).rev() {
            let control = match pair[1] {
                PathCommand::CurveTo(_, _, x, y, _, _) => (x, y),
                PathCommand::LineTo(..) => match pair[0] {
                    PathCommand::MoveTo(x, y)
                    | PathCommand::LineTo(x, y)
                    | PathCommand::CurveTo(_, _, _, _, x, y) => (x, y),
                    _ => continue,
                },
                _ => continue,
            };
            if control != (x2, y2) {
                end = (x2 - control.0, y2 - control.1);
                break;
            }
        }
    }
    let mut result = Vec::new();
    for (x, y, direction, arrow, size) in [
        (x1, y1, start, style.start_arrow, style.start_arrow_size),
        (x2, y2, end, style.end_arrow, style.end_arrow_size),
    ] {
        let norm = direction.0.hypot(direction.1);
        if !norm.is_finite() || norm <= 0.0 {
            continue;
        }
        if let Some(head) = head(arrow, style.width.max(0.5), size) {
            result.push(ArrowHead {
                commands: head.at(x, y, direction.0 / norm, direction.1 / norm),
                filled: head.filled,
            });
        }
    }
    result
}
