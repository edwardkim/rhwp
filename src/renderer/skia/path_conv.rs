use skia_safe::PathBuilder;

use crate::renderer::{svg_arc_to_beziers, PathCommand};

pub fn to_skia_path(commands: &[PathCommand]) -> skia_safe::Path {
    let mut builder = PathBuilder::new();
    let mut current = (0.0, 0.0);
    for cmd in commands {
        match *cmd {
            PathCommand::MoveTo(x, y) => {
                builder.move_to((x as f32, y as f32));
                current = (x, y);
            }
            PathCommand::LineTo(x, y) => {
                builder.line_to((x as f32, y as f32));
                current = (x, y);
            }
            PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                builder.cubic_to(
                    (x1 as f32, y1 as f32),
                    (x2 as f32, y2 as f32),
                    (x as f32, y as f32),
                );
                current = (x, y);
            }
            PathCommand::ArcTo(rx, ry, rotation, large_arc, sweep, x, y) => {
                for bezier in svg_arc_to_beziers(
                    current.0, current.1, rx, ry, rotation, large_arc, sweep, x, y,
                ) {
                    if let PathCommand::CurveTo(x1, y1, x2, y2, ex, ey) = bezier {
                        builder.cubic_to(
                            (x1 as f32, y1 as f32),
                            (x2 as f32, y2 as f32),
                            (ex as f32, ey as f32),
                        );
                        current = (ex, ey);
                    }
                }
            }
            PathCommand::ClosePath => {
                builder.close();
            }
        }
    }
    builder.detach()
}
