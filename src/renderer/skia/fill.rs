//! 사각형 BorderFill의 좌표와 색상 stop을 Native shader로 소비한다.

use skia_safe::{
    gradient::{shaders, Colors, Gradient, Interpolation},
    paint, Color4f, Paint, TileMode,
};

use crate::renderer::{render_tree::BoundingBox, GradientFillInfo};

pub(super) fn gradient_paint(
    gradient: &GradientFillInfo,
    bounds: &BoundingBox,
    opacity: f64,
) -> Option<Paint> {
    if gradient.colors.len() < 2 || bounds.width <= 0.0 || bounds.height <= 0.0 {
        return None;
    }
    let (source_colors, positions) = crate::renderer::linear_gradient_color_stops(
        gradient,
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
    );
    let colors: Vec<_> = source_colors
        .iter()
        .map(|&color| Color4f::from(super::renderer::colorref_to_skia(color, opacity as f32)))
        .collect();
    let stops: Vec<_> = positions.iter().map(|&p| p as f32).collect();
    if stops
        .iter()
        .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
        || stops.windows(2).any(|pair| pair[0] > pair[1])
    {
        return None;
    }
    let specification = Gradient::new(
        Colors::new(&colors, Some(&stops), TileMode::Clamp, None),
        Interpolation::default(),
    );
    let shader = match gradient.gradient_type {
        // Canvas와 같은 중심·반경을 소비한다. 원뿔/사각의 현행 근사는 유지한다.
        2..=4 => shaders::radial_gradient(
            (
                (
                    (bounds.x + bounds.width * f64::from(gradient.center_x) / 100.0) as f32,
                    (bounds.y + bounds.height * f64::from(gradient.center_y) / 100.0) as f32,
                ),
                bounds.width.max(bounds.height) as f32 / 2.0,
            ),
            &specification,
            None,
        ),
        _ => {
            let (x0, y0, x1, y1) = crate::renderer::linear_gradient_axis(
                gradient.angle,
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            );
            shaders::linear_gradient(
                ((x0 as f32, y0 as f32), (x1 as f32, y1 as f32)),
                &specification,
                None,
            )
        }
    }?;
    let mut result = Paint::default();
    result.set_anti_alias(true);
    result.set_style(paint::Style::Fill);
    result.set_shader(shader);
    Some(result)
}
