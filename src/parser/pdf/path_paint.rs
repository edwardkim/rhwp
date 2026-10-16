//! Preserve PDF paint that has no equivalent editable HWP path primitive.
//! Simple rectangles remain editable. Compound Beziers retain their winding,
//! alpha and clip in an object-local 216 dpi PNG, not a flattened page image.
use super::{
    lowering::{common, component, lower_picture, rgb},
    ExtractedPage,
};
use crate::model::{
    control::Control,
    document::Document,
    shape::{DrawingObjAttr, PolygonShape, RectangleShape, ShapeObject},
    style::{Fill, FillType, SolidFill},
    Point,
};
use pdf_oxide::{
    elements::{LineCap, LineJoin, PathContent, PathOperation},
    extractors::images::{ColorSpace, ImageData, PdfImage, PixelFormat},
    geometry::Rect,
};
use tiny_skia::{Paint, PathBuilder, PixmapMut, Stroke, Transform};

pub(super) fn lower(
    page: &ExtractedPage,
    path: &PathContent,
    doc: &mut Document,
    id: u32,
) -> Result<Option<Control>, String> {
    if path.fill_color.is_none() && path.stroke_color.is_none() {
        return Ok(None);
    }
    if let [PathOperation::Rectangle(x, y, w, h)] = path.operations.as_slice() {
        let frame = Rect::new(x.min(x + w), y.min(y + h), w.abs(), h.abs());
        if path.stroke_color.is_none()
            && path.matrix.is_none()
            && !matches!(
                path.paint.clip,
                pdf_oxide::extractors::image_clip::ImageClip::Unsupported
            )
        {
            let Some(visible) = path
                .paint
                .clip
                .visible_bounds(frame)
                .map_err(|e| e.to_string())?
            else {
                return Ok(None);
            };
            let attrs = common(
                visible.x - page.media_box.0,
                page.media_box.3 - visible.y - visible.height,
                visible.width,
                visible.height,
                id,
            )?;
            let (width, height) = (attrs.width, attrs.height);
            return Ok(Some(Control::Shape(Box::new(ShapeObject::Rectangle(
                RectangleShape {
                    common: attrs,
                    x_coords: [0, width as i32, width as i32, 0],
                    y_coords: [0, 0, height as i32, height as i32],
                    drawing: DrawingObjAttr {
                        shape_attr: component(width, height),
                        inst_id: id,
                        fill: Fill {
                            fill_type: FillType::Solid,
                            solid: Some(SolidFill {
                                background_color: rgb(path.fill_color.unwrap()),
                                pattern_color: 0,
                                pattern_type: -1,
                            }),
                            alpha: ((1.0 - path.paint.fill_alpha) * 255.0).round() as u8,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )))));
        }
    }
    // A single closed polygon has an exact editable HWP counterpart. Multiple
    // subpaths (including holes) must keep one winding calculation instead.
    if let Some(fill_color) = path.fill_color.filter(|_| {
        path.stroke_color.is_none()
            && path.matrix.is_none()
            && matches!(path.operations.first(), Some(PathOperation::MoveTo(..)))
            && path.operations[1..]
                .iter()
                .all(|o| matches!(o, PathOperation::LineTo(..) | PathOperation::ClosePath))
            && !path.operations[..path.operations.len() - 1]
                .iter()
                .any(|o| matches!(o, PathOperation::ClosePath))
    }) {
        let b = path.bbox;
        if !matches!(
            path.paint.clip,
            pdf_oxide::extractors::image_clip::ImageClip::Unsupported
        ) && path
            .paint
            .clip
            .visible_bounds(b)
            .map_err(|e| e.to_string())?
            .is_some_and(|visible| visible == b)
        {
            let attrs = common(
                b.x - page.media_box.0,
                page.media_box.3 - b.y - b.height,
                b.width,
                b.height,
                id,
            )?;
            let mut points = Vec::new();
            for op in &path.operations {
                match *op {
                    PathOperation::MoveTo(x, y) | PathOperation::LineTo(x, y) => {
                        points.push(Point {
                            x: ((x - b.x) * 100.0).round() as i32,
                            y: ((b.y + b.height - y) * 100.0).round() as i32,
                        })
                    }
                    PathOperation::ClosePath => {
                        if let Some(first) = points.first().cloned() {
                            points.push(first);
                        }
                    }
                    _ => {}
                }
            }
            return Ok(Some(Control::Shape(Box::new(ShapeObject::Polygon(
                PolygonShape {
                    drawing: DrawingObjAttr {
                        shape_attr: component(attrs.width, attrs.height),
                        inst_id: id,
                        fill: Fill {
                            fill_type: FillType::Solid,
                            solid: Some(SolidFill {
                                background_color: rgb(fill_color),
                                pattern_color: 0,
                                pattern_type: -1,
                            }),
                            alpha: ((1.0 - path.paint.fill_alpha) * 255.0).round() as u8,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    common: attrs,
                    points,
                    ..Default::default()
                },
            )))));
        }
    }
    let budget = &page.import_budget;
    budget.check().map_err(|e| e.to_string())?;
    // Geometry is already in page user space. Include stroke joins/caps rather
    // than clipping them to the centreline bbox. The default miter limit is 10.
    let pad = if path.stroke_color.is_some() {
        path.stroke_width * 5.0
    } else {
        0.0
    };
    let b = path.bbox;
    let bounds = Rect::new(
        b.x - pad,
        b.y - pad,
        b.width + 2.0 * pad,
        b.height + 2.0 * pad,
    );
    let bounds = if matches!(
        path.paint.clip,
        pdf_oxide::extractors::image_clip::ImageClip::Unsupported
    ) {
        if path.paint.clip_path.is_none() {
            return Err("missing PDF clip geometry".into());
        }
        let (left, bottom, right, top) = page.media_box;
        let Some(b) = bounds.intersection(&Rect::new(left, bottom, right - left, top - bottom))
        else {
            return Ok(None);
        };
        b
    } else {
        let Some(b) = path
            .paint
            .clip
            .visible_bounds(bounds)
            .map_err(|e| e.to_string())?
        else {
            return Ok(None);
        };
        b
    };
    if ![bounds.x, bounds.y, bounds.width, bounds.height]
        .iter()
        .all(|x| x.is_finite())
    {
        return Err("nonfinite PDF path geometry".into());
    }
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return Ok(None);
    }
    const PIXELS_PER_POINT: f32 = 3.0;
    let width = (bounds.width * PIXELS_PER_POINT).ceil() as u32;
    let height = (bounds.height * PIXELS_PER_POINT).ceil() as u32;
    let pixels = u64::from(width) * u64::from(height);
    let bytes = usize::try_from(pixels.checked_mul(4).ok_or("path raster size overflow")?)
        .map_err(|_| "path raster size overflow")?;
    if pixels > 16_000_000 || bytes > budget.limits.stream_bytes {
        return Err(budget.reject("path raster buffer").to_string());
    }
    let path_bytes = path
        .operations
        .len()
        .checked_mul(64)
        .ok_or("path buffer size overflow")?;
    budget
        .charge_decoded(path_bytes)
        .map_err(|e| e.to_string())?;
    let mut pb = PathBuilder::with_capacity(path.operations.len() * 5, path.operations.len() * 3);
    for op in &path.operations {
        budget.check().map_err(|e| e.to_string())?;
        match *op {
            PathOperation::MoveTo(x, y) => pb.move_to(x, y),
            PathOperation::LineTo(x, y) => pb.line_to(x, y),
            PathOperation::CurveTo(a, b, c, d, e, f) => pb.cubic_to(a, b, c, d, e, f),
            PathOperation::Rectangle(x, y, w, h) => {
                pb.move_to(x, y);
                pb.line_to(x + w, y);
                pb.line_to(x + w, y + h);
                pb.line_to(x, y + h);
                pb.close();
            }
            PathOperation::ClosePath => pb.close(),
        }
    }
    let path_data = pb.finish().ok_or("invalid PDF path")?;
    budget.charge_decoded(bytes).map_err(|e| e.to_string())?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(bytes)
        .map_err(|_| budget.reject("path raster allocation").to_string())?;
    pixels.resize(bytes, 0);
    let mut pixmap =
        PixmapMut::from_bytes(&mut pixels, width, height).ok_or("invalid path raster")?;
    // Keep exact physical frame dimensions; ceil only controls resolution.
    let sx = width as f32 / bounds.width;
    let sy = height as f32 / bounds.height;
    let transform = Transform::from_row(
        sx,
        0.0,
        0.0,
        -sy,
        -bounds.x * sx,
        (bounds.y + bounds.height) * sy,
    );
    let mut clip_mask = None;
    if matches!(
        path.paint.clip,
        pdf_oxide::extractors::image_clip::ImageClip::Unsupported
    ) {
        let mask_bytes = width as usize * height as usize;
        budget
            .charge_decoded(mask_bytes)
            .map_err(|e| e.to_string())?;
        let mut mask = tiny_skia::Mask::from_vec(
            vec![255; mask_bytes],
            tiny_skia::IntSize::from_wh(width, height).ok_or("clip mask geometry")?,
        )
        .ok_or("clip mask allocation")?;
        let mut clip = path.paint.clip_path.as_deref();
        while let Some(c) = clip {
            budget.check().map_err(|e| e.to_string())?;
            budget
                .charge_decoded(mask_bytes + c.operations.len() * 64)
                .map_err(|e| e.to_string())?;
            let mut pb = PathBuilder::with_capacity(c.operations.len() * 5, c.operations.len() * 3);
            for op in &c.operations {
                match *op {
                    PathOperation::MoveTo(x, y) => pb.move_to(x, y),
                    PathOperation::LineTo(x, y) => pb.line_to(x, y),
                    PathOperation::CurveTo(a, b, c, d, e, f) => pb.cubic_to(a, b, c, d, e, f),
                    PathOperation::Rectangle(x, y, w, h) => {
                        pb.move_to(x, y);
                        pb.line_to(x + w, y);
                        pb.line_to(x + w, y + h);
                        pb.line_to(x, y + h);
                        pb.close();
                    }
                    PathOperation::ClosePath => pb.close(),
                }
            }
            let p = pb.finish().ok_or("invalid clip path")?;
            mask.intersect_path(
                &p,
                if c.even_odd {
                    tiny_skia::FillRule::EvenOdd
                } else {
                    tiny_skia::FillRule::Winding
                },
                true,
                transform,
            );
            clip = c.parent.as_deref();
        }
        clip_mask = Some(mask);
    }
    let paint = |color: pdf_oxide::layout::Color, alpha: f32| {
        let mut p = Paint::default();
        p.set_color_rgba8(
            (color.r * 255.0).round() as u8,
            (color.g * 255.0).round() as u8,
            (color.b * 255.0).round() as u8,
            (alpha * 255.0).round() as u8,
        );
        p
    };
    if let Some(color) = path.fill_color {
        pixmap.fill_path(
            &path_data,
            &paint(color, path.paint.fill_alpha),
            if path.paint.even_odd {
                tiny_skia::FillRule::EvenOdd
            } else {
                tiny_skia::FillRule::Winding
            },
            transform,
            clip_mask.as_ref(),
        )
    }
    if let Some(color) = path.stroke_color {
        let stroke = Stroke {
            width: path.stroke_width,
            line_cap: match path.line_cap {
                LineCap::Butt => tiny_skia::LineCap::Butt,
                LineCap::Round => tiny_skia::LineCap::Round,
                LineCap::Square => tiny_skia::LineCap::Square,
            },
            line_join: match path.line_join {
                LineJoin::Miter => tiny_skia::LineJoin::Miter,
                LineJoin::Round => tiny_skia::LineJoin::Round,
                LineJoin::Bevel => tiny_skia::LineJoin::Bevel,
            },
            miter_limit: 10.0,
            dash: path
                .dash_pattern
                .as_ref()
                .and_then(|(d, p)| tiny_skia::StrokeDash::new(d.clone(), *p)),
        };
        pixmap.stroke_path(
            &path_data,
            &paint(color, path.paint.stroke_alpha),
            &stroke,
            transform,
            clip_mask.as_ref(),
        );
    }
    // tiny-skia owns premultiplied samples; PNG stores straight RGBA.
    for pixel in pixels.chunks_exact_mut(4) {
        let a = u32::from(pixel[3]);
        if a != 0 {
            for c in &mut pixel[..3] {
                *c = ((u32::from(*c) * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    let mut image = PdfImage::new(
        width,
        height,
        ColorSpace::DeviceRGB,
        8,
        ImageData::Raw {
            pixels,
            format: PixelFormat::RGBA,
        },
    );
    image.set_bbox(bounds);
    image.set_matrix([bounds.width, 0.0, 0.0, bounds.height, bounds.x, bounds.y]);
    lower_picture(&image, doc, page.media_box.0, page.media_box.3, id, budget).map(Some)
}
