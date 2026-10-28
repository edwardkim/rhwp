//! BorderFill의 좌표와 색상 stop을 모든 Native shape shader가 공유한다.

use skia_safe::{
    gradient::{shaders, Colors, Gradient, Interpolation},
    paint, Color4f, Data, Paint, Rect, RuntimeEffect, TileMode,
};

use crate::renderer::{render_tree::BoundingBox, GradientFillInfo};

thread_local! {
    static SQUARE_EFFECT: Option<RuntimeEffect> =
        RuntimeEffect::make_for_shader(include_str!("../square_gradient.sksl"), None).ok();
}

pub(super) fn gradient_paint(
    gradient: &GradientFillInfo,
    bounds: &BoundingBox,
    opacity: f64,
) -> Option<Paint> {
    let replay_bounds = Rect::from_xywh(
        bounds.x as f32,
        bounds.y as f32,
        bounds.width as f32,
        bounds.height as f32,
    );
    if gradient.colors.len() < 2
        || gradient.colors.len() > 4096
        || !replay_bounds.is_finite()
        || replay_bounds.is_empty()
        || !opacity.is_finite()
    {
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
        2 => {
            // 원형은 SVG·Canvas와 같은 물리 원을 소비한다. 원뿔/사각의
            // 기존 원형 근사는 이번에 확인한 원형 계약으로 바꾸지 않는다.
            let (cx, cy, radius) = crate::renderer::gradient_geometry::radial_circle(
                (bounds.x, bounds.y, bounds.width, bounds.height),
                gradient.center_x,
                gradient.center_y,
            )?;
            let radius = if gradient.gradient_type == 2 {
                radius
            } else {
                bounds.width.max(bounds.height) / 2.0
            };
            shaders::radial_gradient(
                ((cx as f32, cy as f32), radius as f32),
                &specification,
                None,
            )
        }
        4 => {
            let (cx, cy, radius, cos, sin) = crate::renderer::square_gradient_geometry(
                gradient,
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
            );
            if !radius.is_finite() || radius <= 0.0 {
                return None;
            }
            let ramp =
                shaders::linear_gradient(((0.0, 0.0), (radius as f32, 0.0)), &specification, None)?;
            let bytes: Vec<_> = [cx as f32, cy as f32, cos as f32, sin as f32]
                .into_iter()
                .flat_map(f32::to_ne_bytes)
                .collect();
            SQUARE_EFFECT.with(|effect| {
                effect
                    .as_ref()?
                    .make_shader(Data::new_copy(&bytes), &[ramp.into()], None)
            })
        }
        3 => shaders::radial_gradient(
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
