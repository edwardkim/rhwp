//! Prepared, bounded Canvas2D replay for the explicit strict-outline opt-in.
//!
//! Preparation owns every path, shader and decoded image before a caller may
//! suppress its TextRun alternative. Drawing never starts an asynchronous load.

use std::io::Cursor;

use wasm_bindgen::{Clamped, JsCast, JsValue};
use web_sys::{
    CanvasGradient, CanvasRenderingContext2d, CanvasWindingRule, HtmlCanvasElement, Path2d,
};

use crate::paint::{
    BitmapGlyphFiltering, ColorGlyphFormat, ColorGradientStop, ColorPaintGraphNodeKind,
    ColorPaintGraphPayload, GlyphOutlineFillRule, GlyphOutlinePaintOrder, GlyphOutlinePayloadKind,
    GlyphOutlineStrokeStyle, GlyphRunReplayEligibility, LayerAffineTransform,
    LayerGlyphOutlinePaint, ResolvedColor, ResourceArena, TextVariantQuality,
};
use crate::renderer::render_tree::BoundingBox;
use crate::renderer::{svg_arc_to_beziers, PathCommand};

const MAX_PREPARED_BYTES: usize = 32 * 1024 * 1024;
const MAX_ENCODED_IMAGE_BYTES: usize = 16 * 1024 * 1024;
const MAX_IMAGE_PIXELS: u64 = 4_194_304;
const MAX_IMAGE_DIMENSION: u32 = 8192;
const MAX_PATH_COMMANDS: usize = 16_384;
const MAX_GRADIENT_STOPS: usize = 4096;
const MAX_GRAPH_VISITS: usize = 4096;
const MAX_GRAPH_DEPTH: usize = 64;

pub(super) struct PreparedGlyphOutline {
    commands: Vec<PreparedCommand>,
    byte_cost: usize,
    path_commands: usize,
    limit: usize,
}

enum PreparedCommand {
    Save,
    Restore,
    Transform(LayerAffineTransform),
    Clip(Path2d, GlyphOutlineFillRule),
    Path {
        path: Path2d,
        rule: GlyphOutlineFillRule,
        fill: PreparedFill,
        stroke: Option<GlyphOutlineStrokeStyle>,
    },
    Image {
        canvas: HtmlCanvasElement,
        destination: BoundingBox,
        smoothing: bool,
    },
}

enum PreparedFill {
    Color(String),
    Gradient(CanvasGradient),
}

impl PreparedGlyphOutline {
    pub(super) fn byte_cost(&self) -> usize {
        self.byte_cost
    }

    /// Every object used here was created successfully during preparation.
    /// State is restored even when a browser operation reports failure.
    pub(super) fn draw(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        context.save();
        let mut depth = 0usize;
        let result = (|| {
            context.set_global_composite_operation("source-over")?;
            for command in &self.commands {
                match command {
                    PreparedCommand::Save => {
                        context.save();
                        depth += 1;
                    }
                    PreparedCommand::Restore => {
                        context.restore();
                        depth -= 1;
                    }
                    PreparedCommand::Transform(t) => {
                        context.transform(t.a, t.b, t.c, t.d, t.e, t.f)?;
                    }
                    PreparedCommand::Clip(path, rule) => {
                        context.clip_with_path_2d_and_winding(path, winding(*rule));
                    }
                    PreparedCommand::Path {
                        path,
                        rule,
                        fill,
                        stroke,
                    } => {
                        match fill {
                            PreparedFill::Color(color) => context.set_fill_style_str(color),
                            PreparedFill::Gradient(gradient) => {
                                context.set_fill_style_canvas_gradient(gradient);
                            }
                        }
                        if let Some(stroke) = stroke {
                            context.set_stroke_style_str(&css_color(stroke.color));
                            context.set_line_width(stroke.width);
                            context.set_line_join(stroke.join.as_str());
                            context.set_line_cap(stroke.cap.as_str());
                            context.set_miter_limit(stroke.miter_limit);
                            context.set_line_dash(&js_sys::Array::new())?;
                            context.set_line_dash_offset(0.0);
                            if stroke.paint_order == GlyphOutlinePaintOrder::StrokeThenFill {
                                context.stroke_with_path(path);
                            }
                        }
                        context.fill_with_path_2d_and_winding(path, winding(*rule));
                        if stroke.as_ref().is_some_and(|stroke| {
                            stroke.paint_order == GlyphOutlinePaintOrder::FillThenStroke
                        }) {
                            context.stroke_with_path(path);
                        }
                    }
                    PreparedCommand::Image {
                        canvas,
                        destination,
                        smoothing,
                    } => {
                        context.set_image_smoothing_enabled(*smoothing);
                        context.draw_image_with_html_canvas_element_and_dw_and_dh(
                            canvas,
                            destination.x,
                            destination.y,
                            destination.width,
                            destination.height,
                        )?;
                    }
                }
            }
            Ok(())
        })();
        for _ in 0..depth {
            context.restore();
        }
        context.restore();
        result
    }

    fn reserve(&mut self, bytes: usize) -> Option<()> {
        let total = self.byte_cost.checked_add(bytes)?;
        if total > self.limit {
            return None;
        }
        self.byte_cost = total;
        Some(())
    }

    fn push(&mut self, command: PreparedCommand) -> Option<()> {
        self.reserve(std::mem::size_of::<PreparedCommand>())?;
        self.commands.push(command);
        Some(())
    }

    fn path(&mut self, commands: &[PathCommand]) -> Option<Path2d> {
        if !crate::paint::paint_op::path_commands_are_finite(commands) {
            return None;
        }
        self.path_commands = self.path_commands.checked_add(commands.len())?;
        if self.path_commands > MAX_PATH_COMMANDS {
            return None;
        }
        self.reserve(std::mem::size_of_val(commands))?;
        let result = Path2d::new().ok()?;
        let mut current = (0.0, 0.0);
        let mut origin = current;
        let mut drawable = false;
        for command in commands {
            match *command {
                PathCommand::MoveTo(x, y) => {
                    result.move_to(x, y);
                    current = (x, y);
                    origin = current;
                }
                PathCommand::LineTo(x, y) => {
                    result.line_to(x, y);
                    current = (x, y);
                    drawable = true;
                }
                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                    result.bezier_curve_to(x1, y1, x2, y2, x, y);
                    current = (x, y);
                    drawable = true;
                }
                PathCommand::ArcTo(rx, ry, rotation, large, sweep, x, y) => {
                    let segments = svg_arc_to_beziers(
                        current.0, current.1, rx, ry, rotation, large, sweep, x, y,
                    );
                    if !segments.is_empty()
                        && !crate::paint::paint_op::path_commands_are_finite(&segments)
                    {
                        return None;
                    }
                    self.path_commands = self.path_commands.checked_add(segments.len())?;
                    if self.path_commands > MAX_PATH_COMMANDS {
                        return None;
                    }
                    self.reserve(std::mem::size_of_val(segments.as_slice()))?;
                    if segments.is_empty() && current != (x, y) {
                        result.line_to(x, y);
                    }
                    for segment in segments {
                        match segment {
                            PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                                result.bezier_curve_to(x1, y1, x2, y2, x, y);
                            }
                            PathCommand::LineTo(x, y) => result.line_to(x, y),
                            _ => return None,
                        }
                    }
                    current = (x, y);
                    drawable = true;
                }
                PathCommand::ClosePath => {
                    result.close_path();
                    current = origin;
                }
            }
        }
        drawable.then_some(result)
    }

    fn gradient(
        &mut self,
        gradient: CanvasGradient,
        stops: &[ColorGradientStop],
    ) -> Option<PreparedFill> {
        if !(2..=MAX_GRADIENT_STOPS).contains(&stops.len()) {
            return None;
        }
        self.reserve(std::mem::size_of_val(stops))?;
        let mut previous = 0.0;
        for stop in stops {
            if !stop.offset.is_finite() || !(previous..=1.0).contains(&stop.offset) {
                return None;
            }
            previous = stop.offset;
            gradient
                .add_color_stop(stop.offset as f32, &resolved_color(&stop.color, 1.0)?)
                .ok()?;
        }
        Some(PreparedFill::Gradient(gradient))
    }
}

pub(super) fn prepare(
    outline: &LayerGlyphOutlinePaint,
    bbox: BoundingBox,
    resources: &ResourceArena,
    raster_scale: f64,
    max_bytes: usize,
) -> Option<PreparedGlyphOutline> {
    let diagnostics = &outline.diagnostics;
    if !diagnostics.strict_visual_eligible
        || diagnostics.replay_eligibility != GlyphRunReplayEligibility::Portable
        || !matches!(
            diagnostics.quality,
            TextVariantQuality::Exact | TextVariantQuality::PositionAdjusted
        )
        || diagnostics.missing_glyph_count != 0
        || diagnostics.cluster_mismatch_count != 0
        || diagnostics.used_fallback_font_count != 0
        || (diagnostics.quality == TextVariantQuality::PositionAdjusted
            && (!diagnostics.max_residual_after_adjustment_px.is_finite()
                || diagnostics.max_residual_after_adjustment_px > 0.25))
        || !outline.paint_style.is_fill_only_glyph_replay()
        || !outline.has_exclusive_payload_family()
        || !outline.has_portable_geometry()
        || !raster_scale.is_finite()
        || raster_scale <= 0.0
    {
        return None;
    }
    let mut prepared = PreparedGlyphOutline {
        commands: Vec::new(),
        byte_cost: 0,
        path_commands: 0,
        limit: max_bytes.min(MAX_PREPARED_BYTES),
    };
    let identity = LayerAffineTransform {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };
    match outline.payload_kind {
        GlyphOutlinePayloadKind::MonochromeFill | GlyphOutlinePayloadKind::MonochromeFillStroke => {
            if outline.paths.is_empty() || outline.paths.len() > 4096 {
                return None;
            }
            let stroke = if outline.payload_kind == GlyphOutlinePayloadKind::MonochromeFillStroke {
                let stroke = outline.stroke.as_ref()?;
                if !stroke.is_strict_subset() {
                    return None;
                }
                Some(stroke.clone())
            } else {
                None
            };
            checked_transform(identity, outline.placement.run_to_page)?;
            prepared.push(PreparedCommand::Transform(outline.placement.run_to_page))?;
            for outline_path in &outline.paths {
                let path = prepared.path(&outline_path.commands)?;
                prepared.push(PreparedCommand::Path {
                    path,
                    rule: outline_path.fill_rule,
                    fill: PreparedFill::Color(css_color(outline.paint_style.color)),
                    stroke: stroke.clone(),
                })?;
            }
        }
        GlyphOutlinePayloadKind::ColorLayers => {
            if !outline.paths.is_empty() {
                return None;
            }
            let payload = outline.color_layers.as_ref()?;
            let placement = checked_transform(identity, outline.placement.run_to_page)?;
            prepared.push(PreparedCommand::Transform(placement))?;
            if payload.color_format == ColorGlyphFormat::ColrV0
                && payload.has_colrv0_resolved_layer_contract()
            {
                if payload.layers.is_empty() || payload.layers.len() > 4096 {
                    return None;
                }
                for layer in &payload.layers {
                    let path = prepared.path(layer.commands.as_deref()?)?;
                    let fill = PreparedFill::Color(resolved_color(
                        layer.fill.as_ref()?,
                        layer.opacity.unwrap_or(1.0),
                    )?);
                    prepared.push(PreparedCommand::Save)?;
                    if let Some(transform) = layer.transform_to_run {
                        checked_transform(placement, transform)?;
                        prepared.push(PreparedCommand::Transform(transform))?;
                    }
                    prepared.push(PreparedCommand::Path {
                        path,
                        rule: layer.fill_rule?,
                        fill,
                        stroke: None,
                    })?;
                    prepared.push(PreparedCommand::Restore)?;
                }
            } else if payload.has_colrv1_supported_graph_contract() {
                let graph = payload.paint_graph.as_ref()?;
                let (_, context) = canvas(1, 1)?;
                let mut graph_preparation = GraphPreparation {
                    graph,
                    context: &context,
                    prepared: &mut prepared,
                    visits: 0,
                };
                graph_preparation.visit(graph.root_node_id, placement, 0)?;
            } else {
                return None;
            }
        }
        GlyphOutlinePayloadKind::BitmapGlyph => {
            if !outline.paths.is_empty() {
                return None;
            }
            let payload = outline.bitmap_glyph.as_ref()?;
            if !payload.has_strict_visual_contract() {
                return None;
            }
            let bytes = resources.image_bytes(payload.image_ref)?;
            let (canvas, byte_cost) =
                decoded_canvas(bytes, prepared.limit.saturating_sub(prepared.byte_cost))?;
            prepared.reserve(byte_cost)?;
            if let Some(transform) = payload.transform_to_run {
                checked_transform(identity, transform)?;
                prepared.push(PreparedCommand::Transform(transform))?;
            }
            prepared.push(PreparedCommand::Image {
                canvas,
                destination: payload.placement,
                smoothing: payload.filtering == BitmapGlyphFiltering::Linear,
            })?;
        }
        GlyphOutlinePayloadKind::SvgGlyph => {
            if !outline.paths.is_empty() {
                return None;
            }
            let payload = outline.svg_glyph.as_ref()?;
            let destination = payload.font_em_box.unwrap_or(bbox);
            if !payload.has_static_sanitized_contract()
                || !payload.has_portable_view_box_mapping(destination)
            {
                return None;
            }
            let width = (destination.width * raster_scale).ceil();
            let height = (destination.height * raster_scale).ceil();
            if !width.is_finite()
                || !height.is_finite()
                || width < 1.0
                || height < 1.0
                || width > f64::from(MAX_IMAGE_DIMENSION)
                || height > f64::from(MAX_IMAGE_DIMENSION)
                || width * height > MAX_IMAGE_PIXELS as f64
                || width * height * 4.0 > prepared.limit.saturating_sub(prepared.byte_cost) as f64
            {
                return None;
            }
            let fragment = resources.svg_fragment(payload.svg_ref)?;
            let raster = crate::renderer::static_svg_raster::prepare_static_svg_raster(
                fragment,
                payload.view_box,
                destination,
                raster_scale,
                &css_color(outline.paint_style.color),
            )?;
            let (canvas, byte_cost) = decoded_canvas(
                &raster.png,
                prepared.limit.saturating_sub(prepared.byte_cost),
            )?;
            if (canvas.width(), canvas.height()) != (raster.width, raster.height) {
                return None;
            }
            prepared.reserve(byte_cost.max(raster.byte_cost))?;
            if let Some(transform) = payload.transform_to_run {
                checked_transform(identity, transform)?;
                prepared.push(PreparedCommand::Transform(transform))?;
            }
            prepared.push(PreparedCommand::Image {
                canvas,
                destination,
                smoothing: true,
            })?;
        }
    }
    (!prepared.commands.is_empty()).then_some(prepared)
}

struct GraphPreparation<'a> {
    graph: &'a ColorPaintGraphPayload,
    context: &'a CanvasRenderingContext2d,
    prepared: &'a mut PreparedGlyphOutline,
    visits: usize,
}

impl GraphPreparation<'_> {
    fn visit(&mut self, node_id: u32, matrix: LayerAffineTransform, depth: usize) -> Option<()> {
        self.visits = self.visits.checked_add(1)?;
        if self.visits > MAX_GRAPH_VISITS || depth > MAX_GRAPH_DEPTH {
            return None;
        }
        let node = self
            .graph
            .nodes
            .iter()
            .find(|node| node.node_id == node_id)?;
        let (commands, rule, fill) = match node.kind {
            ColorPaintGraphNodeKind::Transform => {
                let transform = node.transform.as_ref()?;
                let combined = checked_transform(matrix, transform.transform)?;
                self.prepared.push(PreparedCommand::Save)?;
                self.prepared
                    .push(PreparedCommand::Transform(transform.transform))?;
                self.visit(transform.child_node_id, combined, depth + 1)?;
                self.prepared.push(PreparedCommand::Restore)?;
                return Some(());
            }
            ColorPaintGraphNodeKind::Composite => {
                let composite = node.composite.as_ref()?;
                for child in [composite.backdrop_node_id, composite.source_node_id] {
                    self.visit(child, matrix, depth + 1)?;
                }
                return Some(());
            }
            ColorPaintGraphNodeKind::Clip => {
                let clip = node.clip.as_ref()?;
                let path = self.prepared.path(&clip.clip_commands)?;
                self.prepared.push(PreparedCommand::Save)?;
                self.prepared
                    .push(PreparedCommand::Clip(path, clip.fill_rule))?;
                self.visit(clip.child_node_id, matrix, depth + 1)?;
                self.prepared.push(PreparedCommand::Restore)?;
                return Some(());
            }
            ColorPaintGraphNodeKind::SolidPath => {
                let solid = node.solid_path.as_ref()?;
                (
                    solid.commands.as_slice(),
                    solid.fill_rule,
                    PreparedFill::Color(resolved_color(&solid.fill, 1.0)?),
                )
            }
            ColorPaintGraphNodeKind::LinearGradientPath => {
                let path = node.linear_gradient_path.as_ref()?;
                let gradient = &path.gradient;
                let fill = self.context.create_linear_gradient(
                    gradient.x0,
                    gradient.y0,
                    gradient.x1,
                    gradient.y1,
                );
                (
                    path.commands.as_slice(),
                    path.fill_rule,
                    self.prepared.gradient(fill, &gradient.stops)?,
                )
            }
            ColorPaintGraphNodeKind::RadialGradientPath => {
                let path = node.radial_gradient_path.as_ref()?;
                let gradient = &path.gradient;
                let fill = self
                    .context
                    .create_radial_gradient(
                        gradient.cx,
                        gradient.cy,
                        0.0,
                        gradient.cx,
                        gradient.cy,
                        gradient.radius,
                    )
                    .ok()?;
                (
                    path.commands.as_slice(),
                    path.fill_rule,
                    self.prepared.gradient(fill, &gradient.stops)?,
                )
            }
            ColorPaintGraphNodeKind::SweepGradientPath => {
                let path = node.sweep_gradient_path.as_ref()?;
                let gradient = &path.gradient;
                // Detect unsupported browser capabilities before painting.
                let create =
                    js_sys::Reflect::get(self.context, &JsValue::from_str("createConicGradient"))
                        .ok()?
                        .dyn_into::<js_sys::Function>()
                        .ok()?;
                let fill = create
                    .call3(
                        self.context,
                        &JsValue::from_f64(gradient.start_angle_degrees.to_radians()),
                        &JsValue::from_f64(gradient.cx),
                        &JsValue::from_f64(gradient.cy),
                    )
                    .ok()?
                    .dyn_into::<CanvasGradient>()
                    .ok()?;
                (
                    path.commands.as_slice(),
                    path.fill_rule,
                    self.prepared.gradient(fill, &gradient.stops)?,
                )
            }
        };
        let path = self.prepared.path(commands)?;
        self.prepared.push(PreparedCommand::Path {
            path,
            rule,
            fill,
            stroke: None,
        })
    }
}

fn css_color(color: u32) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        color & 255,
        (color >> 8) & 255,
        (color >> 16) & 255,
    )
}

fn resolved_color(color: &ResolvedColor, opacity: f64) -> Option<String> {
    if color
        .color_space
        .as_deref()
        .is_some_and(|space| !space.eq_ignore_ascii_case("srgb"))
        || !opacity.is_finite()
        || !(0.0..=1.0).contains(&opacity)
        || color
            .rgba
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
    {
        return None;
    }
    let [red, green, blue, alpha] = color.rgba;
    Some(format!(
        "rgba({},{},{},{})",
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
        f64::from(alpha) * opacity,
    ))
}

fn winding(rule: GlyphOutlineFillRule) -> CanvasWindingRule {
    match rule {
        GlyphOutlineFillRule::NonZero => CanvasWindingRule::Nonzero,
        GlyphOutlineFillRule::EvenOdd => CanvasWindingRule::Evenodd,
    }
}

fn checked_transform(
    left: LayerAffineTransform,
    right: LayerAffineTransform,
) -> Option<LayerAffineTransform> {
    let matrix = LayerAffineTransform {
        a: left.a * right.a + left.c * right.b,
        b: left.b * right.a + left.d * right.b,
        c: left.a * right.c + left.c * right.d,
        d: left.b * right.c + left.d * right.d,
        e: left.a * right.e + left.c * right.f + left.e,
        f: left.b * right.e + left.d * right.f + left.f,
    };
    [matrix.a, matrix.b, matrix.c, matrix.d, matrix.e, matrix.f]
        .into_iter()
        .all(crate::paint::paint_op::portable_glyph_scalar)
        .then_some(matrix)
}

fn canvas(width: u32, height: u32) -> Option<(HtmlCanvasElement, CanvasRenderingContext2d)> {
    let canvas = web_sys::window()?
        .document()?
        .create_element("canvas")
        .ok()?
        .dyn_into::<HtmlCanvasElement>()
        .ok()?;
    canvas.set_width(width);
    canvas.set_height(height);
    let context = canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    Some((canvas, context))
}

fn decoded_canvas(bytes: &[u8], max_bytes: usize) -> Option<(HtmlCanvasElement, usize)> {
    if bytes.is_empty() || bytes.len() > MAX_ENCODED_IMAGE_BYTES {
        return None;
    }
    let dimensions = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()?;
    let (width, height) = dimensions;
    let pixels = u64::from(width).checked_mul(u64::from(height))?;
    if width == 0
        || height == 0
        || width > MAX_IMAGE_DIMENSION
        || height > MAX_IMAGE_DIMENSION
        || pixels > MAX_IMAGE_PIXELS
        || pixels.checked_mul(4)? > max_bytes as u64
    {
        return None;
    }
    let mut decoder = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(MAX_PREPARED_BYTES as u64);
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    decoder.limits(limits);
    let image = decoder.decode().ok()?.into_rgba8();
    if image.dimensions() != dimensions {
        return None;
    }
    let pixels = image.into_raw();
    let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(pixels.as_slice()),
        width,
        height,
    )
    .ok()?;
    let (canvas, context) = canvas(width, height)?;
    context.put_image_data(&image_data, 0.0, 0.0).ok()?;
    Some((canvas, pixels.len()))
}
