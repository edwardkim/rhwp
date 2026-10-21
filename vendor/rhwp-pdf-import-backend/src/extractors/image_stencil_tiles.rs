//! Bake uninterrupted repeated stencil paints, never PDF text or table structure.
use super::{
    sample_buffer, ColorSpace, ImageData, PdfImage, PdfImageHandle, PdfImageSource, PixelFormat,
};
use crate::{error::Result, extractors::image_clip::ImageClip, geometry::Rect};

impl PdfImageHandle<'_> {
    /// A cheap candidate check. The decoder additionally verifies ImageMask and
    /// Interpolate before composing. Different resources, paint colours, affine
    /// directions and unsupported clips keep their ordinary independent path.
    pub fn same_stencil_tile_source(&self, other: &Self) -> bool {
        matches!((&self.source, &other.source), (PdfImageSource::XObject(a), PdfImageSource::XObject(b)) if a == b)
            && std::ptr::eq(self.doc, other.doc)
            && self.bits_per_component == 1
            && self.width == other.width
            && self.height == other.height
            && self.stencil_colour == other.stencil_colour
            && self.opacity == other.opacity
            && [self, other].iter().all(|h| {
                h.ctm.b == 0.0
                    && h.ctm.c == 0.0
                    && h.ctm.a.is_finite()
                    && h.ctm.a > 0.0
                    && h.ctm.d.is_finite()
                    && h.ctm.d > 0.0
                    && h.ctm.e.is_finite()
                    && h.ctm.f.is_finite()
                    && matches!(
                        h.clip,
                        ImageClip::Rectangle(_) | ImageClip::Unbounded | ImageClip::Empty
                    )
            })
    }

    /// The caller proves there is no intervening text/path paint. All invocations
    /// still consume the original traversal/placement budget. This only reduces
    /// repeated pixel decode and the number of emitted background IR pictures.
    pub fn decode_stencil_tiles(run: &[Self], max_pixels: u64) -> Result<Option<PdfImage>> {
        let Some(first) = run.first() else {
            return Ok(None);
        };
        let Some(budget) = first.doc.import_budget() else {
            return Ok(None);
        };
        if run.len() < 2 || !run.iter().all(|h| first.same_stencil_tile_source(h)) {
            return Ok(None);
        }
        let PdfImageSource::XObject(reference) = first.source else {
            return Ok(None);
        };
        let object = first.doc.load_object(reference)?;
        let Some(dict) = object.as_dict() else {
            return Ok(None);
        };
        // Conservative: masks/indirect sampling flags use the ordinary decoder.
        if !dict
            .get("ImageMask")
            .is_some_and(|v| matches!(v, crate::object::Object::Boolean(true)))
            || dict.get("Interpolate").is_some_and(|v| {
                !matches!(v, crate::object::Object::Boolean(false) | crate::object::Object::Null)
            })
        {
            return Ok(None);
        }
        let mut bounds: Option<Rect> = None;
        let mut dx = 0.0f64;
        let mut dy = 0.0f64;
        for h in run {
            budget.check()?;
            if let Some(r) = h.clip.visible_bounds(h.bbox)? {
                bounds = Some(match bounds {
                    None => r,
                    Some(b) => Rect::new(
                        b.x.min(r.x),
                        b.y.min(r.y),
                        (b.x + b.width).max(r.x + r.width) - b.x.min(r.x),
                        (b.y + b.height).max(r.y + r.height) - b.y.min(r.y),
                    ),
                });
                dx = dx.max(f64::from(h.width) / f64::from(h.ctm.a));
                dy = dy.max(f64::from(h.height) / f64::from(h.ctm.d));
            }
        }
        let Some(frame) = bounds else { return Ok(None) };
        let width = (f64::from(frame.width) * dx).ceil();
        let height = (f64::from(frame.height) * dy).ceil();
        let pixels = width * height;
        let source_pixels = f64::from(first.width) * f64::from(first.height) * run.len() as f64;
        if !pixels.is_finite()
            || width < 1.0
            || height < 1.0
            || width > 16_384.0
            || height > 16_384.0
            || pixels > max_pixels.min(16_000_000) as f64
            || pixels > source_pixels * 4.0
        {
            return Ok(None);
        }
        let tile = first.decode()?;
        let ImageData::Raw {
            pixels: samples,
            format: PixelFormat::RGBA,
        } = tile.data()
        else {
            return Ok(None);
        };
        if tile.width() != first.width || tile.height() != first.height {
            return Ok(None);
        }
        // All source stencils share one solid colour. Their alpha comes from a
        // one-bit mask and the shared invocation opacity. Repeated overlaps use
        // source-over rather than dropping duplicate calls.
        let (width, height) = (width as usize, height as usize);
        let mut output = sample_buffer(width * height * 4, Some(budget))?;
        output.resize(width * height * 4, 0);
        let sx = width as f64 / f64::from(frame.width);
        let sy = height as f64 / f64::from(frame.height);
        for h in run {
            budget.check()?;
            let Some(r) = h.clip.visible_bounds(h.bbox)? else {
                continue;
            };
            let x0 = ((f64::from(r.x) - f64::from(frame.x)) * sx)
                .floor()
                .max(0.0) as usize;
            let y0 = ((f64::from(frame.y + frame.height) - f64::from(r.y + r.height)) * sy)
                .floor()
                .max(0.0) as usize;
            let x1 =
                (((f64::from(r.x + r.width) - f64::from(frame.x)) * sx).ceil() as usize).min(width);
            let y1 = (((f64::from(frame.y + frame.height) - f64::from(r.y)) * sy).ceil() as usize)
                .min(height);
            for y in y0..y1 {
                let py = f64::from(frame.y + frame.height) - (y as f64 + 0.5) / sy;
                if py < f64::from(r.y) || py >= f64::from(r.y + r.height) {
                    continue;
                }
                let v = (py - f64::from(h.ctm.f)) / f64::from(h.ctm.d);
                if !(0.0..1.0).contains(&v) {
                    continue;
                }
                let ty = ((1.0 - v) * f64::from(first.height)).floor() as usize;
                for x in x0..x1 {
                    let px = f64::from(frame.x) + (x as f64 + 0.5) / sx;
                    if px < f64::from(r.x) || px >= f64::from(r.x + r.width) {
                        continue;
                    }
                    let u = (px - f64::from(h.ctm.e)) / f64::from(h.ctm.a);
                    if !(0.0..1.0).contains(&u) || ty >= first.height as usize {
                        continue;
                    }
                    let tx = (u * f64::from(first.width)).floor() as usize;
                    let source = &samples[(ty * first.width as usize + tx) * 4..][..4];
                    if source[3] == 0 {
                        continue;
                    }
                    let dest = &mut output[(y * width + x) * 4..][..4];
                    dest[..3].copy_from_slice(&source[..3]);
                    dest[3] = (u32::from(source[3])
                        + (u32::from(dest[3]) * (255 - u32::from(source[3])) + 127) / 255)
                        as u8;
                }
            }
        }
        let mut image = PdfImage::new(
            width as u32,
            height as u32,
            ColorSpace::DeviceRGB,
            8,
            ImageData::Raw {
                pixels: output,
                format: PixelFormat::RGBA,
            },
        );
        image.set_bbox(frame);
        image.set_matrix([frame.width, 0.0, 0.0, frame.height, frame.x, frame.y]);
        image.interpolate = Some(false);
        Ok(Some(image))
    }
}
