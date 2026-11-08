//! Common vector projection of the authenticated source primitive in a PNG.
//! All backends consume the same cropped rectangles, color and opacity.
use super::{
    render_tree::{BoundingBox, ImageNode, PathNode},
    PathCommand, ShapeStyle,
};
pub(crate) fn project(image: &ImageNode, bbox: BoundingBox) -> Option<PathNode> {
    if image.header_footer_ref.is_some()
        || image.transform.has_transform()
        || image.brightness != 0
        || image.contrast != 0
        || image.opacity != 1.
        || !matches!(image.effect, crate::model::image::ImageEffect::RealPic)
        || image.content_inset.is_some()
    {
        return None;
    }
    let bytes = image.data.as_deref()?;
    let source = crate::model::pdf_stroke::PdfStroke::read(bytes)?;
    let dims = (
        u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?) as f64 * 75.,
        u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?) as f64 * 75.,
    );
    let reference = image
        .original_size_hu
        .map(|(w, h)| (f64::from(w), f64::from(h)))
        .unwrap_or(dims);
    if reference.0 <= 0. || reference.1 <= 0. || bbox.width <= 0. || bbox.height <= 0. {
        return None;
    }
    let (l, t, r, b) = image
        .crop
        .map(|(l, t, r, b)| (f64::from(l), f64::from(t), f64::from(r), f64::from(b)))
        .unwrap_or((0., 0., reference.0, reference.1));
    let [w, h] = source.frame;
    let crop = [
        l / reference.0 * w,
        t / reference.1 * h,
        r / reference.0 * w,
        b / reference.1 * h,
    ];
    if !crop.iter().all(|v| v.is_finite()) || crop[2] <= crop[0] || crop[3] <= crop[1] {
        return None;
    }
    let map = |x: f64, y: f64| {
        (
            bbox.x + (x - crop[0]) / (crop[2] - crop[0]) * bbox.width,
            bbox.y + (y - crop[1]) / (crop[3] - crop[1]) * bbox.height,
        )
    };
    let mut commands = Vec::new();
    for rect in source.rectangles()? {
        let l = rect[0].max(crop[0]).max(0.);
        let t = rect[1].max(crop[1]).max(0.);
        let r = rect[2].min(crop[2]).min(w);
        let b = rect[3].min(crop[3]).min(h);
        if r <= l || b <= t {
            continue;
        }
        let (x, y) = map(l, t);
        let (a, b) = map(r, b);
        commands.extend([
            PathCommand::MoveTo(x, y),
            PathCommand::LineTo(a, y),
            PathCommand::LineTo(a, b),
            PathCommand::LineTo(x, b),
            PathCommand::ClosePath,
        ]);
    }
    let mut path = PathNode::new(
        commands,
        ShapeStyle {
            fill_color: Some(source.color),
            opacity: source.alpha,
            ..Default::default()
        },
        None,
    );
    path.section_index = image.section_index;
    path.para_index = image.para_index;
    path.control_index = image.control_index;
    path.cell_index = image.cell_index;
    path.cell_para_index = image.cell_para_index;
    path.outer_table_control_index = image.outer_table_control_index;
    Some(path)
}
