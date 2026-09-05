//! Prepare strict glyph resources before selecting a text alternative.
//!
//! Preparation is atomic: a failed font, path, shader or image never suppresses
//! the anchored TextRun. Prepared Skia objects are reused by the actual draw.

use std::collections::HashSet;

use skia_safe::{
    font_arguments::{variation_position, VariationPosition},
    gradient::{shaders, Colors, Gradient, Interpolation},
    Canvas, Color, Color4f, Data, FilterMode, Font, FontArguments, FontMgr, FourByteTag, Image,
    Matrix, MipmapMode, Paint, Path, PathBuilder, Point, Rect, SamplingOptions, TileMode,
};

use crate::paint::{
    BitmapGlyphFiltering, ColorGlyphFormat, ColorPaintGraphNodeKind, GlyphOutlineFillRule,
    GlyphOutlinePaintOrder, GlyphOutlinePayloadKind, GlyphRunReplayEligibility,
    LayerAffineTransform, LayerGlyphOutlinePaint, LayerGlyphRunPaint, ResolvedColor, ResourceArena,
    TextVariantQuality,
};
use crate::renderer::render_tree::BoundingBox;
use crate::renderer::{svg_arc_to_beziers, PathCommand};

use super::renderer::{colorref_to_skia, NativeGlyphRunReplayProofReason as FontFailure};

pub(super) const MAX_PREPARED_GLYPH_BYTES: usize = 64 * 1024 * 1024;
const MAX_PATH_COMMANDS: usize = 100_000;
const MAX_IMAGE_BYTES: usize = 4 * 1024 * 1024;
const MAX_IMAGE_PIXELS: u64 = 4096 * 4096;

pub(super) struct PreparedGlyph {
    draws: Vec<PreparedDraw>,
    pub byte_cost: usize,
}

struct PreparedDraw {
    transforms: Vec<Matrix>,
    paint: Paint,
    content: DrawContent,
}

enum DrawContent {
    Glyphs {
        font: Font,
        glyphs: Vec<u16>,
        positions: Vec<Point>,
    },
    Path(Path),
    Image {
        image: Image,
        destination: Rect,
        sampling: SamplingOptions,
    },
}

impl PreparedGlyph {
    pub fn draw(&self, canvas: &Canvas) {
        for draw in &self.draws {
            canvas.save();
            for transform in &draw.transforms {
                canvas.concat(transform);
            }
            match &draw.content {
                DrawContent::Glyphs {
                    font,
                    glyphs,
                    positions,
                } => {
                    canvas.draw_glyphs_at(
                        glyphs,
                        positions.as_slice(),
                        (0.0, 0.0),
                        font,
                        &draw.paint,
                    );
                }
                DrawContent::Path(path) => {
                    canvas.draw_path(path, &draw.paint);
                }
                DrawContent::Image {
                    image,
                    destination,
                    sampling,
                } => {
                    canvas.draw_image_rect_with_sampling_options(
                        image,
                        None,
                        destination,
                        *sampling,
                        &draw.paint,
                    );
                }
            }
            canvas.restore();
        }
    }
}

pub(super) fn finite_scalar(value: f64) -> bool {
    value.is_finite() && value.abs() <= f64::from(f32::MAX)
}

fn matrix(transform: LayerAffineTransform) -> Option<Matrix> {
    let values = [
        transform.a,
        transform.b,
        transform.c,
        transform.d,
        transform.e,
        transform.f,
    ];
    values
        .into_iter()
        .all(finite_scalar)
        .then(|| Matrix::from_affine(&values.map(|value| value as f32)))
}

fn fill_paint(color: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    paint
}

pub(super) fn construct_glyph_font(
    run: &LayerGlyphRunPaint,
    resources: &ResourceArena,
    font_mgr: &FontMgr,
) -> Result<PreparedGlyph, Vec<FontFailure>> {
    let instance = &run.shape_key.font_instance;
    let face = resources
        .font_resources()
        .faces
        .iter()
        .find(|face| face.id == instance.face_key)
        .ok_or_else(|| vec![FontFailure::FontFaceMissing])?;
    let blob = resources
        .font_resources()
        .blobs
        .iter()
        .find(|blob| blob.id == face.blob_key)
        .ok_or_else(|| vec![FontFailure::FontBlobMissing])?;
    let bytes = blob
        .data_ref
        .as_ref()
        .and_then(|data_ref| resources.font_blob_bytes_for_ref(data_ref))
        .ok_or_else(|| vec![FontFailure::FontBlobBytesMissing])?;
    let unavailable = || {
        let mut reasons = vec![FontFailure::ExactFaceUnavailable];
        if face.face_index != 0 {
            reasons.push(FontFailure::FaceIndexUnsupported);
        }
        if !instance.variations.is_empty() {
            reasons.push(FontFailure::FontVariationUnsupported);
        }
        reasons
    };
    // Parse the requested collection face first. Font backends must never turn
    // an out-of-range face into a successful face-zero family fallback.
    let parsed = ttf_parser::Face::parse(bytes, face.face_index).map_err(|_| unavailable())?;
    if run
        .glyph_ids
        .iter()
        .any(|glyph| *glyph >= u32::from(parsed.number_of_glyphs()))
    {
        return Err(vec![FontFailure::GlyphIdOutOfRange]);
    }
    let mut typeface = font_mgr
        .new_from_data(bytes, Some(face.face_index as usize))
        .ok_or_else(unavailable)?;
    if !instance.variations.is_empty() {
        if instance.variations.len() > 16 {
            return Err(vec![FontFailure::FontVariationUnsupported]);
        }
        let parameters = typeface
            .variation_design_parameters()
            .ok_or_else(|| vec![FontFailure::FontVariationUnsupported])?;
        let mut seen = HashSet::new();
        let mut coordinates = Vec::with_capacity(instance.variations.len());
        for variation in &instance.variations {
            let tag: [u8; 4] = variation
                .tag
                .as_bytes()
                .try_into()
                .map_err(|_| vec![FontFailure::FontVariationUnsupported])?;
            let axis = FourByteTag::from_chars(
                tag[0] as char,
                tag[1] as char,
                tag[2] as char,
                tag[3] as char,
            );
            let parameter = parameters
                .iter()
                .find(|parameter| parameter.tag == axis)
                .ok_or_else(|| vec![FontFailure::FontVariationUnsupported])?;
            if !seen.insert(tag)
                || !variation.value.is_finite()
                || variation.value < parameter.min
                || variation.value > parameter.max
            {
                return Err(vec![FontFailure::FontVariationUnsupported]);
            }
            coordinates.push(variation_position::Coordinate {
                axis,
                value: variation.value,
            });
        }
        let mut arguments = FontArguments::new().set_variation_design_position(VariationPosition {
            coordinates: &coordinates,
        });
        arguments.set_collection_index(face.face_index as usize);
        typeface = typeface
            .clone_with_arguments(&arguments)
            .ok_or_else(|| vec![FontFailure::FontVariationUnsupported])?;
        let actual = typeface
            .variation_design_position()
            .ok_or_else(|| vec![FontFailure::FontVariationUnsupported])?;
        if coordinates.iter().any(|expected| {
            !actual.iter().any(|value| {
                value.axis == expected.axis && (value.value - expected.value).abs() <= 0.001
            })
        }) {
            return Err(vec![FontFailure::FontVariationUnsupported]);
        }
    }
    let mut font = Font::from_typeface(typeface, instance.size_px as f32);
    font.set_embolden(instance.synthetic_bold);
    font.set_skew_x(if instance.synthetic_italic {
        -0.25
    } else {
        0.0
    });
    font.set_edging(skia_safe::font::Edging::AntiAlias);
    let transforms =
        vec![matrix(run.placement.run_to_page)
            .ok_or_else(|| vec![FontFailure::PlacementNotFinite])?];
    Ok(PreparedGlyph {
        byte_cost: bytes.len() + run.glyph_ids.len() * (std::mem::size_of::<Point>() + 2),
        draws: vec![PreparedDraw {
            transforms,
            paint: fill_paint(colorref_to_skia(run.paint_style.color, 1.0)),
            content: DrawContent::Glyphs {
                font,
                glyphs: run.glyph_ids.iter().map(|glyph| *glyph as u16).collect(),
                positions: run
                    .positions
                    .iter()
                    .map(|position| Point::new(position.x as f32, position.y as f32))
                    .collect(),
            },
        }],
    })
}

fn path(commands: &[PathCommand], fill_rule: GlyphOutlineFillRule) -> Option<Path> {
    if commands.is_empty() || commands.len() > MAX_PATH_COMMANDS {
        return None;
    }
    let mut builder = PathBuilder::new();
    let mut current = (0.0, 0.0);
    let mut origin = current;
    for command in commands {
        match *command {
            PathCommand::MoveTo(x, y) | PathCommand::LineTo(x, y) => {
                if !finite_scalar(x) || !finite_scalar(y) {
                    return None;
                }
                if matches!(command, PathCommand::MoveTo(..)) {
                    builder.move_to((x as f32, y as f32));
                    origin = (x, y);
                } else {
                    builder.line_to((x as f32, y as f32));
                }
                current = (x, y);
            }
            PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                if ![x1, y1, x2, y2, x, y].into_iter().all(finite_scalar) {
                    return None;
                }
                builder.cubic_to(
                    (x1 as f32, y1 as f32),
                    (x2 as f32, y2 as f32),
                    (x as f32, y as f32),
                );
                current = (x, y);
            }
            PathCommand::ArcTo(rx, ry, rotation, large, sweep, x, y) => {
                if ![rx, ry, rotation, x, y].into_iter().all(finite_scalar) {
                    return None;
                }
                if rx == 0.0 || ry == 0.0 {
                    builder.line_to((x as f32, y as f32));
                } else {
                    for segment in svg_arc_to_beziers(
                        current.0, current.1, rx, ry, rotation, large, sweep, x, y,
                    ) {
                        if let PathCommand::CurveTo(x1, y1, x2, y2, ex, ey) = segment {
                            if ![x1, y1, x2, y2, ex, ey].into_iter().all(finite_scalar) {
                                return None;
                            }
                            builder.cubic_to(
                                (x1 as f32, y1 as f32),
                                (x2 as f32, y2 as f32),
                                (ex as f32, ey as f32),
                            );
                        }
                    }
                }
                current = (x, y);
            }
            PathCommand::ClosePath => {
                builder.close();
                current = origin;
            }
        }
    }
    builder.set_fill_type(match fill_rule {
        GlyphOutlineFillRule::NonZero => skia_safe::PathFillType::Winding,
        GlyphOutlineFillRule::EvenOdd => skia_safe::PathFillType::EvenOdd,
    });
    Some(builder.detach())
}

fn resolved_color(color: &ResolvedColor, opacity: f64) -> Option<Color> {
    if !color
        .rgba
        .into_iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
        || !opacity.is_finite()
        || !(0.0..=1.0).contains(&opacity)
        || color
            .color_space
            .as_deref()
            .is_some_and(|space| !space.eq_ignore_ascii_case("srgb"))
    {
        return None;
    }
    let channel = |value: f32| (value * 255.0).round() as u8;
    Some(Color::from_argb(
        channel(color.rgba[3] * opacity as f32),
        channel(color.rgba[0]),
        channel(color.rgba[1]),
        channel(color.rgba[2]),
    ))
}

fn gradient_stops(stops: &[crate::paint::ColorGradientStop]) -> Option<(Vec<Color4f>, Vec<f32>)> {
    if stops.len() < 2 || stops.len() > MAX_PATH_COMMANDS {
        return None;
    }
    let mut colors = Vec::with_capacity(stops.len());
    let mut positions = Vec::with_capacity(stops.len());
    let mut last = 0.0;
    for stop in stops {
        if !stop.offset.is_finite() || stop.offset < last || stop.offset > 1.0 {
            return None;
        }
        colors.push(Color4f::from(resolved_color(&stop.color, 1.0)?));
        positions.push(stop.offset as f32);
        last = stop.offset;
    }
    Some((colors, positions))
}

pub(super) fn prepare_glyph_outline(
    outline: &LayerGlyphOutlinePaint,
    bbox: BoundingBox,
    resources: &ResourceArena,
    raster_scale: f32,
) -> Option<PreparedGlyph> {
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
        || !finite_scalar(outline.placement.baseline_y)
    {
        return None;
    }
    let placement = matrix(outline.placement.run_to_page)?;
    let mut result = PreparedGlyph {
        draws: Vec::new(),
        byte_cost: 0,
    };
    match outline.payload_kind {
        GlyphOutlinePayloadKind::MonochromeFill | GlyphOutlinePayloadKind::MonochromeFillStroke => {
            if outline.paths.is_empty() || outline.paths.len() > 4096 {
                return None;
            }
            let fill = fill_paint(colorref_to_skia(outline.paint_style.color, 1.0));
            let stroke = if outline.payload_kind == GlyphOutlinePayloadKind::MonochromeFillStroke {
                let style = outline.stroke.as_ref()?;
                if !style.is_strict_subset()
                    || !finite_scalar(style.width)
                    || !finite_scalar(style.miter_limit)
                {
                    return None;
                }
                let mut paint = fill_paint(colorref_to_skia(style.color, 1.0));
                paint.set_style(skia_safe::paint::Style::Stroke);
                paint.set_stroke_width(style.width as f32);
                paint.set_stroke_miter(style.miter_limit as f32);
                paint.set_stroke_join(skia_safe::paint::Join::Miter);
                paint.set_stroke_cap(skia_safe::paint::Cap::Butt);
                Some((paint, style.paint_order))
            } else {
                None
            };
            for outline_path in &outline.paths {
                let path = path(&outline_path.commands, outline_path.fill_rule)?;
                result.byte_cost = result.byte_cost.checked_add(
                    outline_path.commands.len() * std::mem::size_of::<PathCommand>(),
                )?;
                if result.byte_cost > MAX_PREPARED_GLYPH_BYTES {
                    return None;
                }
                let paints = match &stroke {
                    Some((stroke, GlyphOutlinePaintOrder::StrokeThenFill)) => {
                        vec![stroke.clone(), fill.clone()]
                    }
                    Some((stroke, GlyphOutlinePaintOrder::FillThenStroke)) => {
                        vec![fill.clone(), stroke.clone()]
                    }
                    None => vec![fill.clone()],
                    _ => return None,
                };
                for paint in paints {
                    result.draws.push(PreparedDraw {
                        transforms: vec![placement],
                        paint,
                        content: DrawContent::Path(path.clone()),
                    });
                }
            }
        }
        GlyphOutlinePayloadKind::ColorLayers => {
            let payload = outline.color_layers.as_ref()?;
            if payload.color_format == ColorGlyphFormat::ColrV0
                && payload.has_colrv0_resolved_layer_contract()
            {
                if payload.layers.len() > 4096 {
                    return None;
                }
                for layer in &payload.layers {
                    let commands = layer.commands.as_deref()?;
                    let path = path(commands, layer.fill_rule?)?;
                    let mut transforms = vec![placement];
                    if let Some(transform) = layer.transform_to_run {
                        transforms.push(matrix(transform)?);
                    }
                    result.byte_cost = result
                        .byte_cost
                        .checked_add(commands.len() * std::mem::size_of::<PathCommand>())?;
                    if result.byte_cost > MAX_PREPARED_GLYPH_BYTES {
                        return None;
                    }
                    result.draws.push(PreparedDraw {
                        transforms,
                        paint: fill_paint(resolved_color(
                            layer.fill.as_ref()?,
                            layer.opacity.unwrap_or(1.0),
                        )?),
                        content: DrawContent::Path(path),
                    });
                }
            } else if payload.has_colrv1_supported_graph_contract() {
                let graph = payload.paint_graph.as_ref()?;
                let mut transforms = vec![placement];
                let mut node_id = graph.root_node_id;
                for _ in 0..64 {
                    let node = graph.nodes.iter().find(|node| node.node_id == node_id)?;
                    let (commands, fill_rule, paint) = match node.kind {
                        ColorPaintGraphNodeKind::Transform => {
                            let transform = node.transform.as_ref()?;
                            transforms.push(matrix(transform.transform)?);
                            node_id = transform.child_node_id;
                            continue;
                        }
                        ColorPaintGraphNodeKind::SolidPath => {
                            let leaf = node.solid_path.as_ref()?;
                            (
                                &leaf.commands,
                                leaf.fill_rule,
                                fill_paint(resolved_color(&leaf.fill, 1.0)?),
                            )
                        }
                        ColorPaintGraphNodeKind::LinearGradientPath => {
                            let leaf = node.linear_gradient_path.as_ref()?;
                            let gradient = &leaf.gradient;
                            if ![gradient.x0, gradient.y0, gradient.x1, gradient.y1]
                                .into_iter()
                                .all(finite_scalar)
                            {
                                return None;
                            }
                            let (colors, stops) = gradient_stops(&gradient.stops)?;
                            let specification = Gradient::new(
                                Colors::new(&colors, Some(&stops), TileMode::Clamp, None),
                                Interpolation::default(),
                            );
                            let shader = shaders::linear_gradient(
                                (
                                    (gradient.x0 as f32, gradient.y0 as f32),
                                    (gradient.x1 as f32, gradient.y1 as f32),
                                ),
                                &specification,
                                None,
                            )?;
                            let mut paint = fill_paint(Color::WHITE);
                            paint.set_shader(shader);
                            (&leaf.commands, leaf.fill_rule, paint)
                        }
                        ColorPaintGraphNodeKind::RadialGradientPath => {
                            let leaf = node.radial_gradient_path.as_ref()?;
                            let gradient = &leaf.gradient;
                            if ![gradient.cx, gradient.cy, gradient.radius]
                                .into_iter()
                                .all(finite_scalar)
                                || gradient.radius <= 0.0
                            {
                                return None;
                            }
                            let (colors, stops) = gradient_stops(&gradient.stops)?;
                            let specification = Gradient::new(
                                Colors::new(&colors, Some(&stops), TileMode::Clamp, None),
                                Interpolation::default(),
                            );
                            let shader = shaders::radial_gradient(
                                (
                                    (gradient.cx as f32, gradient.cy as f32),
                                    gradient.radius as f32,
                                ),
                                &specification,
                                None,
                            )?;
                            let mut paint = fill_paint(Color::WHITE);
                            paint.set_shader(shader);
                            (&leaf.commands, leaf.fill_rule, paint)
                        }
                        ColorPaintGraphNodeKind::SweepGradientPath => {
                            let leaf = node.sweep_gradient_path.as_ref()?;
                            let gradient = &leaf.gradient;
                            if ![
                                gradient.cx,
                                gradient.cy,
                                gradient.start_angle_degrees,
                                gradient.end_angle_degrees,
                            ]
                            .into_iter()
                            .all(finite_scalar)
                            {
                                return None;
                            }
                            let (colors, stops) = gradient_stops(&gradient.stops)?;
                            let specification = Gradient::new(
                                Colors::new(&colors, Some(&stops), TileMode::Clamp, None),
                                Interpolation::default(),
                            );
                            let shader = shaders::sweep_gradient(
                                (gradient.cx as f32, gradient.cy as f32),
                                (
                                    gradient.start_angle_degrees as f32,
                                    gradient.end_angle_degrees as f32,
                                ),
                                &specification,
                                None,
                            )?;
                            let mut paint = fill_paint(Color::WHITE);
                            paint.set_shader(shader);
                            (&leaf.commands, leaf.fill_rule, paint)
                        }
                        _ => return None,
                    };
                    result.byte_cost = commands.len() * std::mem::size_of::<PathCommand>();
                    result.draws.push(PreparedDraw {
                        transforms,
                        paint,
                        content: DrawContent::Path(path(commands, fill_rule)?),
                    });
                    break;
                }
            } else {
                return None;
            }
        }
        GlyphOutlinePayloadKind::BitmapGlyph => {
            let payload = outline.bitmap_glyph.as_ref()?;
            if !payload.has_strict_visual_contract() {
                return None;
            }
            let bytes = resources.image_bytes(payload.image_ref)?;
            if bytes.len() > MAX_IMAGE_BYTES {
                return None;
            }
            let encoded = Image::from_encoded(Data::new_copy(bytes))?;
            let pixels = u64::try_from(encoded.width())
                .ok()?
                .checked_mul(u64::try_from(encoded.height()).ok()?)?;
            if pixels == 0 || pixels > MAX_IMAGE_PIXELS {
                return None;
            }
            // Image construction alone only reads a header. Eagerly decode before
            // replacing the text fallback so a truncated strike cannot drop ink.
            let image = encoded.make_raster_image(None, None)?;
            let transforms = optional_matrix(payload.transform_to_run)?;
            result.byte_cost = pixels as usize * 4;
            result.draws.push(PreparedDraw {
                transforms: transforms.into_iter().collect(),
                paint: fill_paint(Color::WHITE),
                content: DrawContent::Image {
                    image,
                    destination: drawable_rect(payload.placement)?,
                    sampling: SamplingOptions::new(
                        match payload.filtering {
                            BitmapGlyphFiltering::Nearest => FilterMode::Nearest,
                            BitmapGlyphFiltering::Linear => FilterMode::Linear,
                        },
                        MipmapMode::None,
                    ),
                },
            });
        }
        GlyphOutlinePayloadKind::SvgGlyph => {
            let payload = outline.svg_glyph.as_ref()?;
            if !payload.has_static_sanitized_contract() {
                return None;
            }
            let fragment = resources.svg_fragment(payload.svg_ref)?;
            let (image, bytes) = svg_glyph_image(fragment, payload.view_box, bbox, raster_scale)?;
            let transforms = optional_matrix(payload.transform_to_run)?;
            result.byte_cost = bytes;
            result.draws.push(PreparedDraw {
                transforms: transforms.into_iter().collect(),
                paint: fill_paint(Color::WHITE),
                content: DrawContent::Image {
                    image,
                    destination: drawable_rect(bbox)?,
                    sampling: SamplingOptions::new(FilterMode::Linear, MipmapMode::None),
                },
            });
        }
    }
    (!result.draws.is_empty()).then_some(result)
}

fn drawable_rect(bbox: BoundingBox) -> Option<Rect> {
    ([
        bbox.x,
        bbox.y,
        bbox.width,
        bbox.height,
        bbox.x + bbox.width,
        bbox.y + bbox.height,
    ]
    .into_iter()
    .all(finite_scalar)
        && bbox.width > 0.0
        && bbox.height > 0.0)
        .then(|| {
            Rect::from_xywh(
                bbox.x as f32,
                bbox.y as f32,
                bbox.width as f32,
                bbox.height as f32,
            )
        })
}

fn svg_glyph_image(
    fragment: &str,
    view_box: BoundingBox,
    bbox: BoundingBox,
    scale: f32,
) -> Option<(Image, usize)> {
    use quick_xml::{
        events::{BytesEnd, Event},
        Reader, Writer,
    };
    use resvg::{tiny_skia, usvg};
    if !crate::renderer::static_svg::static_svg_fragment_has_path_layer(fragment) {
        return None;
    }
    drawable_rect(bbox)?;
    drawable_rect(view_box)?;
    let width = (bbox.width * f64::from(scale)).ceil();
    let height = (bbox.height * f64::from(scale)).ceil();
    if !width.is_finite()
        || !height.is_finite()
        || width < 1.0
        || height < 1.0
        || width * height > MAX_IMAGE_PIXELS as f64
    {
        return None;
    }
    // The payload viewBox is authoritative. SVG containers act as groups in
    // the static path contract, preserving their paint/transform attributes.
    let mut reader = Reader::from_str(fragment);
    let mut writer = Writer::new(Vec::new());
    loop {
        let event = reader.read_event().ok()?;
        let empty = matches!(&event, Event::Empty(_));
        match event {
            Event::Eof => break,
            Event::Start(mut element) | Event::Empty(mut element)
                if element.name().as_ref() == "svg" =>
            {
                element.set_name("g");
                writer
                    .write_event(if empty {
                        Event::Empty(element)
                    } else {
                        Event::Start(element)
                    })
                    .ok()?;
            }
            Event::End(element) if element.name().as_ref() == "svg" => {
                writer.write_event(Event::End(BytesEnd::new("g"))).ok()?;
            }
            event => {
                writer.write_event(event).ok()?;
            }
        }
    }
    let inner = String::from_utf8(writer.into_inner()).ok()?;
    let svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"{} {} {} {}\" preserveAspectRatio=\"none\">{inner}</svg>", view_box.x,view_box.y,view_box.width,view_box.height);
    let mut options = usvg::Options::default();
    options.image_href_resolver = usvg::ImageHrefResolver {
        resolve_data: Box::new(|_, _, _| None),
        resolve_string: Box::new(|_, _| None),
    };
    let tree = usvg::Tree::from_str(&svg, &options).ok()?;
    if tree.root().children().is_empty() {
        return None;
    }
    let mut pixels = tiny_skia::Pixmap::new(width as u32, height as u32)?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixels.as_mut());
    let byte_cost = pixels.data().len();
    let encoded = pixels.encode_png().ok()?;
    let image = Image::from_encoded(Data::new_copy(&encoded))?.make_raster_image(None, None)?;
    Some((image, byte_cost))
}

fn optional_matrix(transform: Option<LayerAffineTransform>) -> Option<Option<Matrix>> {
    match transform {
        Some(transform) => matrix(transform).map(Some),
        None => Some(None),
    }
}
