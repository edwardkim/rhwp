//! Image-only coloured tiling patterns (ISO 32000-1:2008 8.7.2–8.7.3).
//! Page, Form and pattern cells share the same bounded image/clip walker.

use super::*;
use crate::content::{parse_content_stream, Matrix, Operator};
use crate::extractors::image_clip::{ImageClip, ImageClipTracker, ImageGraphicsState};
use crate::extractors::images::{image_handle_from_inline, PdfImageHandle};
use crate::path_color::PathColorSpace;

fn unsupported(message: &str) -> Error {
    Error::UnsupportedPattern(message.into())
}

fn fills(op: &Operator) -> bool {
    matches!(
        op,
        Operator::Fill
            | Operator::FillEvenOdd
            | Operator::FillStroke
            | Operator::FillStrokeEvenOdd
            | Operator::CloseFillStroke
            | Operator::CloseFillStrokeEvenOdd
    )
}

fn number(value: &Object) -> Option<f32> {
    value
        .as_real()
        .or_else(|| value.as_integer().map(|v| v as f64))
        .map(|v| v as f32)
        .filter(|v| v.is_finite())
}

fn finite_matrix(m: Matrix) -> bool {
    [m.a, m.b, m.c, m.d, m.e, m.f].iter().all(|v| v.is_finite())
}

fn inverse(m: Matrix) -> Result<Matrix> {
    let [a, b, c, d, e, f] = [m.a, m.b, m.c, m.d, m.e, m.f].map(f64::from);
    let det = a * d - b * c;
    if !det.is_finite() || det == 0.0 {
        return Err(unsupported("singular pattern matrix"));
    }
    let v = [
        d / det,
        -b / det,
        -c / det,
        a / det,
        (c * f - d * e) / det,
        (b * e - a * f) / det,
    ]
    .map(|v| v as f32);
    let m = Matrix {
        a: v[0],
        b: v[1],
        c: v[2],
        d: v[3],
        e: v[4],
        f: v[5],
    };
    if !finite_matrix(m) {
        return Err(unsupported("non-finite inverse pattern matrix"));
    }
    Ok(m)
}

/// Cells intersect iff bbox.max + i*step > clip.min and bbox.min + i*step < clip.max.
/// A negative step generates the same integer lattice; overlap order is unspecified
/// by 8.7.3.1. Do not iterate from zero to a remote translated visible region.
fn tile_range(min: f32, max: f32, bmin: f32, bmax: f32, step: f32) -> Result<(i64, i64)> {
    let step = f64::from(step.abs());
    let first = ((f64::from(min) - f64::from(bmax)) / step).floor() + 1.0;
    let last = ((f64::from(max) - f64::from(bmin)) / step).ceil() - 1.0;
    if !first.is_finite()
        || !last.is_finite()
        || first.abs() >= i64::MAX as f64
        || last.abs() >= i64::MAX as f64
    {
        return Err(unsupported("pattern tile index overflow"));
    }
    Ok((first as i64, last as i64))
}

impl PdfDocument {
    fn image_resource_dictionary(
        &self,
        resources: Option<&Object>,
        key: &str,
    ) -> Result<Option<Object>> {
        resources
            .and_then(Object::as_dict)
            .and_then(|d| d.get(key))
            .map(|obj| self.image_resource_object(obj))
            .transpose()
    }

    fn image_resource_object(&self, object: &Object) -> Result<Object> {
        match object.as_reference() {
            Some(reference) => self.load_object(reference),
            None => crate::object_copy::clone_object(object, self.import_budget.as_deref()),
        }
    }

    pub(super) fn walk_image_handles<'s>(
        &'s self,
        operators: &[Operator],
        resources: Option<&Object>,
        initial: ImageGraphicsState,
        paint_order: &mut usize,
        stack: &mut Vec<ObjectRef>,
        image_only: bool,
    ) -> Result<Vec<PdfImageHandle<'s>>> {
        let xobjects = self.image_resource_dictionary(resources, "XObject")?;
        let cs_map = self.build_color_space_map(resources)?;
        let mut graphics = ImageClipTracker::from_image_state(initial.clone());
        graphics.set_budget(self.import_budget.clone());
        let mut handles = Vec::new();
        for (ordinal, op) in operators.iter().enumerate() {
            self.check_import_budget()?;
            // The parsed operators are shared across tiles; replay itself also
            // consumes work, rather than only charging the one-time parse.
            if image_only {
                if let Some(budget) = &self.import_budget {
                    budget.charge_operator()?;
                }
            }
            if let Operator::SetExtGState { dict_name } = op {
                if let Some(resources) = resources {
                    if let Some(alpha) = self.paint_alpha(resources, dict_name)?.0 {
                        graphics.set_alpha(alpha);
                    }
                }
            }
            self.process_image_colour_operator(op, resources, &mut graphics)?;
            let before = graphics.current();
            let mut more = if fills(op) && matches!(before.fill_space, PathColorSpace::Pattern(_)) {
                let PathColorSpace::Pattern(Some(ref name)) = before.fill_space else {
                    return Err(unsupported("pattern fill has no selected resource"));
                };
                self.collect_pattern_images(
                    name,
                    resources,
                    &initial,
                    before.alpha,
                    graphics.fill_clip(matches!(
                        op,
                        Operator::FillEvenOdd
                            | Operator::FillStrokeEvenOdd
                            | Operator::CloseFillStrokeEvenOdd
                    ))?,
                    paint_order,
                    stack,
                )?
            } else {
                Vec::new()
            };
            if image_only
                && matches!(
                    op,
                    Operator::Tj { .. }
                        | Operator::TJ { .. }
                        | Operator::Quote { .. }
                        | Operator::DoubleQuote { .. }
                        | Operator::Stroke
                        | Operator::CloseAndStroke
                        | Operator::Fill
                        | Operator::FillEvenOdd
                        | Operator::FillStroke
                        | Operator::FillStrokeEvenOdd
                        | Operator::CloseFillStroke
                        | Operator::CloseFillStrokeEvenOdd
                        | Operator::PaintShading { .. }
                )
                && !(matches!(op, Operator::Fill | Operator::FillEvenOdd)
                    && matches!(before.fill_space, PathColorSpace::Pattern(_)))
            {
                return Err(unsupported("non-image painting in image tiling pattern"));
            }
            graphics.apply(op)?;
            let state = graphics.current();
            match op {
                Operator::Do { name } => {
                    if let Some(dict) = xobjects.as_ref().and_then(Object::as_dict) {
                        more = match self.collect_handles_from_do(
                            name,
                            dict,
                            resources,
                            state,
                            paint_order,
                            stack,
                            image_only,
                        ) {
                            Ok(more) => more,
                            Err(e @ (Error::UnsupportedPattern(_) | Error::Unsupported(_))) => {
                                return Err(e)
                            },
                            Err(e) if image_only => return Err(e),
                            Err(_) => {
                                self.check_import_budget()?;
                                Vec::new()
                            },
                        };
                    } else if image_only {
                        return Err(unsupported("missing pattern image resources"));
                    }
                },
                Operator::InlineImage { dict, data } => {
                    let data = match &self.import_budget {
                        Some(budget) => budget.copy_bytes(data)?,
                        None => data.clone(),
                    };
                    if let Some(mut h) =
                        image_handle_from_inline(self, dict, data, state.ctm, *paint_order, &cs_map)
                    {
                        h.opacity = state.alpha;
                        h.stencil_colour = state.fill_colour;
                        h.clip = state.clip;
                        more.push(h);
                        *paint_order += 1;
                    } else if image_only {
                        return Err(unsupported("invalid inline pattern image"));
                    }
                },
                _ => {},
            }
            if let Some(budget) = &self.import_budget {
                budget.charge_entries(more.len())?;
            }
            for mut h in more {
                // Cull invisible cells/images before pixel decode and preserve
                // the original operator order relative to editable text/paths.
                if image_only
                    && !matches!(h.clip, ImageClip::Unsupported)
                    && h.clip.visible_bounds(h.bbox)?.is_none()
                {
                    continue;
                }
                h.source_order.insert(0, ordinal);
                handles.push(h);
            }
        }
        Ok(handles)
    }

    fn collect_pattern_images<'s>(
        &'s self,
        name: &str,
        resources: Option<&Object>,
        initial: &ImageGraphicsState,
        opacity: f32,
        clip: ImageClip,
        paint_order: &mut usize,
        stack: &mut Vec<ObjectRef>,
    ) -> Result<Vec<PdfImageHandle<'s>>> {
        let Some(visible) = clip.bounds()? else {
            return Ok(Vec::new());
        };
        let patterns = self
            .image_resource_dictionary(resources, "Pattern")?
            .ok_or_else(|| unsupported("missing Pattern resources"))?;
        let reference = patterns
            .as_dict()
            .and_then(|d| d.get(name))
            .and_then(Object::as_reference)
            .ok_or_else(|| unsupported("missing pattern stream reference"))?;
        if let Some(budget) = &self.import_budget {
            budget.form_depth(stack.len() + 1)?;
            budget.xobject_visit()?;
        }
        if stack.contains(&reference) || stack.len() >= 100 {
            return Err(unsupported("cyclic or deeply nested tiling pattern"));
        }
        let pattern = self.load_object(reference)?;
        let dict = pattern
            .as_dict()
            .ok_or_else(|| unsupported("invalid pattern dictionary"))?;
        if dict.get("PatternType").and_then(Object::as_integer) != Some(1)
            || dict.get("PaintType").and_then(Object::as_integer) != Some(1)
        {
            return Err(unsupported("only coloured image tiling patterns are supported"));
        }
        if !matches!(dict.get("TilingType").and_then(Object::as_integer), Some(1..=3)) {
            return Err(unsupported("invalid pattern TilingType"));
        }
        let bbox = dict
            .get("BBox")
            .and_then(Object::as_array)
            .filter(|a| a.len() == 4)
            .and_then(|a| {
                Some([
                    number(&a[0])?,
                    number(&a[1])?,
                    number(&a[2])?,
                    number(&a[3])?,
                ])
            })
            .ok_or_else(|| unsupported("invalid pattern BBox"))?;
        if bbox[2] <= bbox[0] || bbox[3] <= bbox[1] {
            return Err(unsupported("empty pattern BBox"));
        }
        let dx = dict
            .get("XStep")
            .and_then(number)
            .filter(|v| *v != 0.0)
            .ok_or_else(|| unsupported("invalid pattern XStep"))?
            .abs();
        let dy = dict
            .get("YStep")
            .and_then(number)
            .filter(|v| *v != 0.0)
            .ok_or_else(|| unsupported("invalid pattern YStep"))?
            .abs();
        let matrix = match dict.get("Matrix") {
            Some(obj) => self
                .parse_matrix_from_object(obj)
                .filter(|m| finite_matrix(*m))
                .ok_or_else(|| unsupported("invalid pattern Matrix"))?,
            None => Matrix::identity(),
        }
        .multiply(&initial.ctm);
        let inv = inverse(matrix)?;
        let local = ImageClip::transformed_image_rectangle(
            inv,
            [
                visible.x,
                visible.y,
                visible.x + visible.width,
                visible.y + visible.height,
            ],
        )
        .bounds()?
        .ok_or_else(|| unsupported("invalid pattern visible bounds"))?;
        let (x0, x1) = tile_range(local.x, local.x + local.width, bbox[0], bbox[2], dx)?;
        let (y0, y1) = tile_range(local.y, local.y + local.height, bbox[1], bbox[3], dy)?;
        if x1 < x0 || y1 < y0 {
            return Ok(Vec::new());
        }
        let count = x1
            .checked_sub(x0)
            .and_then(|v| v.checked_add(1))
            .zip(y1.checked_sub(y0).and_then(|v| v.checked_add(1)))
            .and_then(|(x, y)| x.checked_mul(y))
            .and_then(|v| usize::try_from(v).ok());
        let limit = self
            .import_budget
            .as_ref()
            .map_or(10_000, |b| b.limits.xobject_visits);
        if count.is_none_or(|n| n > limit) {
            return Err(self.import_budget.as_ref().map_or_else(
                || unsupported("pattern tile count limit"),
                |b| b.reject("pattern tile count limit"),
            ));
        }
        let resources = dict
            .get("Resources")
            .map(|o| self.image_resource_object(o))
            .transpose()?
            .ok_or_else(|| unsupported("missing pattern cell Resources"))?;
        let data = self.cached_form_stream(&pattern, reference)?;
        let operators = self.parse_import_content(&data, parse_content_stream)?;
        stack.push(reference);
        let result = (|| {
            let mut handles = Vec::new();
            let mut pixel_caches = HashMap::new();
            for (tile, (x, y)) in (y0..=y1)
                .flat_map(|y| (x0..=x1).map(move |x| (x, y)))
                .enumerate()
            {
                if let Some(budget) = &self.import_budget {
                    budget.xobject_visit()?;
                }
                let ctm = Matrix::translation(
                    (x as f64 * f64::from(dx)) as f32,
                    (y as f64 * f64::from(dy)) as f32,
                )
                .multiply(&matrix);
                if !finite_matrix(ctm) {
                    return Err(unsupported("pattern tile matrix overflow"));
                }
                let tile_clip = clip
                    .clone()
                    .intersect(ImageClip::transformed_image_rectangle(ctm, bbox));
                if tile_clip.bounds()?.is_none() {
                    continue;
                }
                let mut state = initial.clone();
                state.ctm = ctm;
                state.clip = tile_clip;
                // Cell alpha is local; the outer pattern fill applies opacity
                // once to each emitted paint, never once per nesting ancestor.
                state.alpha = 1.0;
                let mut more = self.walk_image_handles(
                    &operators,
                    Some(&resources),
                    state,
                    paint_order,
                    stack,
                    true,
                )?;
                for h in &mut more {
                    if let Some(budget) = &self.import_budget {
                        budget.charge_entries(h.source_order.len() + 1)?;
                    }
                    h.pattern_pixels = Some(Arc::clone(
                        pixel_caches
                            .entry(h.source_order.clone())
                            .or_insert_with(|| Arc::new(std::sync::OnceLock::new())),
                    ));
                    h.source_order.insert(0, tile);
                    h.opacity *= opacity;
                }
                if let Some(budget) = &self.import_budget {
                    budget.charge_entries(more.len())?;
                }
                handles.append(&mut more);
            }
            Ok(handles)
        })();
        stack.pop();
        result
    }
}
