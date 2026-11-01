//! Horizontal text in a rotated, orthogonally scaled PDF frame stays editable.
//! ISO 32000-1 §9.4.4 rotates both glyphs and their displacements. Merely
//! rotating an axis-aligned span bbox would move the original baseline origin.
use crate::model::{shape::RectangleShape, text_clip::TextClipPath};
use pdf_oxide::{elements::PathClip, layout::TextSpan};

pub(super) fn validate(span: &TextSpan, basis: [f32; 4]) -> Result<(), &'static str> {
    if span.wmode != 0 || span.mirrored {
        return Err("vertical/mirrored text is not supported");
    }
    if !basis.iter().all(|v| v.is_finite()) || !span.rotation_degrees.is_finite() {
        return Err("invalid text transformation");
    }
    if span.rotation_degrees == 0.0 {
        return Ok(());
    }
    let [a, b, c, d] = basis.map(f64::from);
    let scale2 = a * a + b * b;
    if scale2 <= 0.0
        || a * d - b * c <= 0.0
        || (a * c + b * d).abs() > scale2 * 1e-6
        || span.horizontal_scaling <= 0.0
    {
        return Err("rotated text with shear or reflection is not supported");
    }
    if !(50.0..=200.0).contains(&span.horizontal_scaling) {
        return Err("rotated text scaling exceeds the editable character ratio");
    }
    // The standard shape rotation property is integral degrees. Accept only
    // when that quantization displaces every frame point by < half a HWPUNIT.
    let delta = f64::from(span.rotation_degrees - span.rotation_degrees.round()).to_radians();
    let radius = f64::from(span.bbox.width).hypot(f64::from(span.font_size));
    if radius * delta.abs() > 0.005 {
        return Err("text rotation exceeds the precision of the editable shape property");
    }
    Ok(())
}

pub(super) fn apply(
    rect: &mut RectangleShape,
    span: &TextSpan,
    media_box: (f32, f32, f32, f32),
    clip: Option<&PathClip>,
    allocation: super::allocation::Allocation<'_>,
) -> Result<(), String> {
    if span.rotation_degrees == 0.0 {
        return Ok(());
    }
    let width = f64::from(rect.common.width) / 100.0;
    let height = f64::from(rect.common.height) / 100.0;
    let baseline = f64::from(
        rect.drawing.text_box.as_ref().unwrap().paragraphs[0].line_segs[0].baseline_distance,
    ) / 100.0;
    let angle = f64::from(span.rotation_degrees.round());
    let (sin, cos) = angle.to_radians().sin_cos();
    let origin = (
        f64::from(span.bbox.x - media_box.0),
        f64::from(media_box.3 - span.bbox.y),
    );
    // R = [cos sin; -sin cos] in the page's downward-y frame. Solve
    // center + R * (-width/2, baseline-height/2) = source baseline origin.
    let local = (-width / 2.0, baseline - height / 2.0);
    let center = (
        origin.0 - cos * local.0 - sin * local.1,
        origin.1 + sin * local.0 - cos * local.1,
    );
    let hull = (
        cos.abs() * width + sin.abs() * height,
        sin.abs() * width + cos.abs() * height,
    );
    rect.common = super::lowering::common(
        (center.0 - hull.0 / 2.0) as f32,
        (center.1 - hull.1 / 2.0) as f32,
        hull.0 as f32,
        hull.1 as f32,
        rect.common.instance_id,
    )?;
    let attr = &mut rect.drawing.shape_attr;
    attr.rotation_angle = (-angle).rem_euclid(360.0) as _;
    // HWP/HWPX also carry the rotation matrix about the unrotated center.
    // Keep original/current extents in local text space; common extents are AABB.
    let (cx, cy) = (width * 50.0, height * 50.0);
    let tx = cx - cos * cx - sin * cy;
    let ty = cy + sin * cx - cos * cy;
    let identity = [1.0_f64, 0.0, 0.0, 0.0, 1.0, 0.0];
    let rotation = [cos, sin, tx, -sin, cos, ty];
    attr.raw_rendering = 1_u16.to_le_bytes().to_vec();
    for matrix in [identity, identity, rotation] {
        for value in matrix {
            attr.raw_rendering.extend(value.to_le_bytes());
        }
    }
    attr.render_sx = cos;
    attr.render_b = sin;
    attr.render_c = -sin;
    attr.render_sy = cos;
    attr.render_tx = tx;
    attr.render_ty = ty;
    let paths: Option<Vec<TextClipPath>> = super::text_clip::local_paths_mapped(
        clip,
        true,
        |x, y| {
            let dx = f64::from(x - media_box.0) - center.0;
            let dy = f64::from(media_box.3 - y) - center.1;
            (
                (cos * dx - sin * dy + width / 2.0) * 100.0,
                (sin * dx + cos * dy + height / 2.0) * 100.0,
            )
        },
        allocation,
    )?;
    rect.drawing.text_box.as_mut().unwrap().paint_clip = paths;
    Ok(())
}
