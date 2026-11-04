use super::{
    allocation::{message, Allocation},
    value_macro::request_json,
    ExtractedPage,
};
use crate::model::{
    bin_data::{BinData, BinDataBytes, BinDataContent, BinDataType},
    control::Control,
    document::Document,
    image::{CropInfo, ImageAttr, Picture},
    paragraph::{CharShapeRef, LineSeg, Paragraph},
    shape::{
        CommonObjAttr, DrawingObjAttr, LineShape, RectangleShape, ShapeComponentAttr, ShapeObject,
        TextBox, TextWrap,
    },
    style::{
        Alignment, Fill, FillType, Font, ShapeBorderLine, SolidFill, SubstFont,
        PARA_ATTR2_LINE_WRAP_MASK,
    },
    Point,
};
use pdf_oxide::{
    elements::{LineCap, PathOperation},
    layout::Color,
};
use serde_json::{json, Value};

pub(crate) fn hu(points: f32) -> Result<u32, String> {
    if !points.is_finite() || !(0.0..=100000.0).contains(&points) {
        return Err(format!("unsupported coordinate/extent: {points}"));
    }
    Ok((points * 100.0).round() as u32)
}

pub(super) fn rgb(c: Color) -> u32 {
    let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
    channel(c.r) | (channel(c.g) << 8) | (channel(c.b) << 16)
}

/// PDF BaseFont may be a PostScript face name, whereas HWP stores a family.
/// This exact pair is independently present in Arial's OpenType name records
/// (nameID 6 / nameID 1). Do not guess a family by deleting arbitrary suffixes.
fn imported_font_family(name: &str) -> &str {
    let base = name
        .split_once('+')
        .filter(|(tag, _)| tag.len() == 6 && tag.bytes().all(|b| b.is_ascii_uppercase()))
        .map_or(name, |(_, family)| family);
    match base {
        "ArialMT" => "Arial",
        _ => base,
    }
}

/// OpenType name IDs identify the family independently of a PDF BaseFont label.
/// A subset's PostScript label need not be an installed CSS/Skia family name.
fn program_family(
    font: &pdf_oxide::fonts::FontInfo,
    allocation: Allocation<'_>,
) -> Result<Option<String>, String> {
    let Some(bytes) = &font.embedded_font_data else {
        return Ok(None);
    };
    let Ok(face) = ttf_parser::Face::parse(bytes, 0) else {
        return Ok(None);
    };
    for id in [
        ttf_parser::name_id::FAMILY,
        ttf_parser::name_id::TYPOGRAPHIC_FAMILY,
    ] {
        let names = face.names();
        let selected = names
            .into_iter()
            .filter(|n| {
                n.name_id == id && n.is_unicode() && !n.name.is_empty() && n.name.len() <= 1024
            })
            .min_by_key(|n| (n.language_id != 0x0409, n.language_id));
        if let Some(name) = selected {
            // UTF-16 -> UTF-8 needs at most 3 bytes per code unit; reserve before decoding.
            if name.name.len() % 2 != 0 {
                continue;
            }
            let mut value = allocation
                .string_capacity(name.name.len() * 2)
                .map_err(message)?;
            let mut valid = true;
            for ch in char::decode_utf16(
                name.name
                    .chunks_exact(2)
                    .map(|b| u16::from_be_bytes([b[0], b[1]])),
            ) {
                match ch {
                    Ok(ch) if !ch.is_control() => value.push(ch),
                    _ => {
                        valid = false;
                        break;
                    }
                }
            }
            if valid && !value.trim().is_empty() {
                return Ok(Some(value));
            }
        }
    }
    Ok(None)
}

pub(super) fn component(width: u32, height: u32) -> ShapeComponentAttr {
    ShapeComponentAttr {
        original_width: width,
        original_height: height,
        current_width: width,
        current_height: height,
        rotation_center: Point {
            x: (width / 2) as i32,
            y: (height / 2) as i32,
        },
        ..Default::default()
    }
}

pub(super) fn common(x: f32, y: f32, w: f32, h: f32, id: u32) -> Result<CommonObjAttr, String> {
    // HWP stores signed page offsets in these u32 bit fields. Extents remain
    // nonnegative; positions may lie beyond the page origin. Quantize to the
    // format's 1/100 pt unit, without clamping content to the page boundary.
    let offset = |v: f32| {
        if !v.is_finite() || v.abs() > 100000.0 {
            return Err(format!("unsupported page offset: {v}"));
        }
        Ok(((v * 100.0).round() as i32) as u32)
    };
    Ok(CommonObjAttr {
        horizontal_offset: offset(x)?,
        vertical_offset: offset(y)?,
        width: hu(w)?,
        height: hu(h)?,
        instance_id: id,
        z_order: id as i32,
        allow_overlap: true,
        text_wrap: TextWrap::InFrontOfText,
        ..Default::default()
    })
}

fn text_para(
    text: String,
    shape: u32,
    width: u32,
    size: u32,
    allocation: Allocation<'_>,
) -> Result<Paragraph, String> {
    let mut length = 0;
    let char_offsets = allocation
        .collect(text.chars().map(|c| {
            let offset = length;
            length += Paragraph::char_stream_len(c);
            offset
        }))
        .map_err(message)?;
    Ok(Paragraph {
        text,
        char_offsets,
        char_count: length + 1,
        has_para_text: true,
        char_shapes: allocation
            .collect([CharShapeRef {
                start_pos: 0,
                char_shape_id: shape,
            }])
            .map_err(message)?,
        line_segs: allocation
            .collect([LineSeg {
                line_height: size as i32,
                text_height: size as i32,
                baseline_distance: (size as f32 * 0.95).round() as i32,
                segment_width: width as i32,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }])
            .map_err(message)?,
        ..Default::default()
    })
}

pub(super) fn new_document() -> Result<Document, String> {
    let spec =
        serde_json::from_value(json!({"version":"1", "blocks":[]})).map_err(|e| e.to_string())?;
    let mut doc = crate::scaffold::build_scaffold(&spec);
    doc.provenance.format = crate::model::provenance::SourceFormat::Pdf;
    doc.doc_info.para_shapes[0].alignment = Alignment::Left;
    doc.doc_info.para_shapes[0].line_spacing = 100;
    Ok(doc)
}

pub(super) fn lower(
    page: &ExtractedPage,
    doc: &mut Document,
    next_id: &mut u32,
) -> Result<Value, String> {
    let allocation = Allocation(&page.import_budget);
    let (left, bottom, right, top) = page.media_box;
    let section = doc.sections.len() - 1;
    let pd = &mut doc.sections[section].section_def.page_def;
    let width = hu(right - left)?;
    let height = hu(top - bottom)?;
    // HWP stores the portrait paper dimensions and applies orientation when
    // deriving the physical page. A wide MediaBox with portrait orientation
    // renders locally but Hancom normalizes it to a narrow page on printing.
    pd.landscape = width > height;
    (pd.width, pd.height) = if pd.landscape {
        (height, width)
    } else {
        (width, height)
    };
    pd.margin_left = 0;
    pd.margin_right = 0;
    pd.margin_top = 0;
    pd.margin_bottom = 0;
    pd.margin_header = 0;
    pd.margin_footer = 0;
    pd.margin_gutter = 0;
    // Each PDF span belongs to one source line. Keep that constraint on editor reflow.
    let mut single_line = allocation
        .clone_record(&doc.doc_info.para_shapes[0])
        .map_err(message)?;
    single_line.raw_data = None;
    single_line.attr2 = (single_line.attr2 & !PARA_ATTR2_LINE_WRAP_MASK) | 1;
    let single_line_id = doc.doc_info.para_shapes.len() as u16;
    allocation
        .push(&mut doc.doc_info.para_shapes, single_line)
        .map_err(message)?;
    allocation
        .bytes(std::mem::size_of::<LineSeg>())
        .map_err(message)?;
    let mut owner = Paragraph::new_empty();
    let mut id = *next_id;
    let mut lines = 0;
    let mut painted = 0;
    let mut unsupported_paths = 0;
    let mut rule_sources = Vec::new();
    let mut image_sources = Vec::new();
    let mut paint_orders = Vec::new();
    let mut local_path_pictures = 0;
    let mut warnings = allocation.collect([
        allocation.string("positional text boxes, not semantic paragraph/table reconstruction").map_err(message)?,
        allocation.string("Text/path interleaving, text clipping, blend modes and embedded fonts are not fully reconstructed").map_err(message)?,
    ]).map_err(message)?;

    // Keep straight rules as editable IR shapes even when table inference misses them.
    for (source_path, path) in page.paths.iter().enumerate() {
        let simple_rule = matches!(
            path.operations.as_slice(),
            [PathOperation::MoveTo(..), PathOperation::LineTo(..)]
        ) && path.stroke_color.is_some()
            && path.fill_color.is_none()
            && path.dash_pattern.is_none()
            && path.matrix.is_none()
            && path.paint.stroke_alpha == 1.0
            && match path.paint.clip {
                pdf_oxide::extractors::image_clip::ImageClip::Unbounded => true,
                pdf_oxide::extractors::image_clip::ImageClip::Rectangle(clip) => {
                    let bounds = path.rendered_bbox();
                    bounds.x >= clip.x
                        && bounds.y >= clip.y
                        && bounds.x + bounds.width <= clip.x + clip.width
                        && bounds.y + bounds.height <= clip.y + clip.height
                }
                _ => false,
            };
        if !simple_rule {
            if let Some(control) = super::path_paint::lower(page, path, doc, id)? {
                local_path_pictures += usize::from(matches!(control, Control::Picture(_)));
                allocation.push(&mut rule_sources,
                    request_json!(allocation, {"path":source_path,"control":owner.controls.len(),"instanceId":id})?,
                ).map_err(message)?;
                allocation
                    .push(
                        &mut paint_orders,
                        (
                            owner.controls.len(),
                            allocation
                                .copy_slice(&path.paint.source_order)
                                .map_err(message)?,
                        ),
                    )
                    .map_err(message)?;
                allocation
                    .push(&mut owner.controls, control)
                    .map_err(message)?;
                id += 1;
            }
            continue;
        }
        allocation
            .push(
                &mut paint_orders,
                (
                    owner.controls.len(),
                    allocation
                        .copy_slice(&path.paint.source_order)
                        .map_err(message)?,
                ),
            )
            .map_err(message)?;
        let [PathOperation::MoveTo(x1, y1), PathOperation::LineTo(x2, y2)] =
            path.operations.as_slice()
        else {
            unsupported_paths += 1;
            continue;
        };
        let Some(color) = path.stroke_color else {
            unsupported_paths += 1;
            continue;
        };
        if path.fill_color.is_some() || path.dash_pattern.is_some() || path.matrix.is_some() {
            unsupported_paths += 1;
            continue;
        }
        let x = x1.min(*x2);
        let y = y1.max(*y2);
        let w = (x2 - x1).abs();
        let h = (y2 - y1).abs();
        if path.line_cap == LineCap::Butt
            && path.stroke_width > 0.
            && ((w == 0. && h > 0.) || (h == 0. && w > 0.))
        {
            // A flat-cap axis stroke has this exact painted area, not a padded zero-size frame.
            let half = path.stroke_width / 2.;
            let (px, py, pw, ph) = if w == 0. {
                (x - left - half, top - y, path.stroke_width, h)
            } else {
                (x - left, top - y - half, w, path.stroke_width)
            };
            let attrs = common(px, py, pw, ph, id)?;
            let width = attrs.width;
            let height = attrs.height;
            let rect = RectangleShape {
                common: attrs,
                x_coords: [0, width as i32, width as i32, 0],
                y_coords: [0, 0, height as i32, height as i32],
                drawing: DrawingObjAttr {
                    shape_attr: component(width, height),
                    inst_id: id,
                    fill: Fill {
                        fill_type: FillType::Solid,
                        solid: Some(SolidFill {
                            background_color: rgb(color),
                            pattern_color: 0,
                            pattern_type: -1,
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            allocation.push(&mut rule_sources, request_json!(allocation, {"path":source_path,"control":owner.controls.len(),"instanceId":id})?).map_err(message)?;
            owner.controls.push(Control::Shape(
                allocation
                    .boxed(ShapeObject::Rectangle(rect))
                    .map_err(message)?,
            ));
            painted += 1;
            id += 1;
            continue;
        }
        let attrs = common(x - left, top - y, w, h, id)?;
        let start = Point {
            x: hu(*x1 - x)? as i32,
            y: hu(y - *y1)? as i32,
        };
        let end = Point {
            x: hu(*x2 - x)? as i32,
            y: hu(y - *y2)? as i32,
        };
        let shape = LineShape {
            common: attrs,
            drawing: DrawingObjAttr {
                shape_attr: component(hu(w)?, hu(h)?),
                border_line: ShapeBorderLine {
                    color: rgb(color),
                    width: hu(path.stroke_width)? as i32,
                    attr: 1,
                    ..Default::default()
                },
                inst_id: id,
                ..Default::default()
            },
            start,
            end,
            ..Default::default()
        };
        allocation.push(&mut rule_sources, request_json!(allocation, {"path":source_path,"control":owner.controls.len(),"instanceId":id})?).map_err(message)?;
        owner.controls.push(Control::Shape(
            allocation
                .boxed(ShapeObject::Line(shape))
                .map_err(message)?,
        ));
        lines += 1;
        id += 1;
    }

    for (source_image, image) in page.images.iter().enumerate() {
        let b = image.bbox().ok_or("image has no placement geometry")?;
        let matrix = image.matrix();
        if !matrix.iter().all(|value| value.is_finite())
            || matrix[0] <= 0.0
            || matrix[3] <= 0.0
            // Off-diagonal entries are physical endpoint displacements in the
            // image unit square. Only accept those indistinguishable from zero
            // in HWP's native coordinate unit. The rounded angle alone can be
            // 360 degrees for a tiny negative value and is not authoritative.
            || (matrix[1] * 100.0).round() != 0.0
            || (matrix[2] * 100.0).round() != 0.0
        {
            return Err("rotated/mirrored/sheared image is not supported by this POC".into());
        }
        // extract() validates both source and composited pixel dimensions
        // against this request's policy before constructing ExtractedPage.
        // PNG encoding independently enforces the same request's buffer/work
        // limits; a second fixed pixel cap here would override that policy.
        allocation
            .push(
                &mut image_sources,
                request_json!(allocation, {"image":source_image,"control":owner.controls.len()})?,
            )
            .map_err(message)?;
        allocation
            .push(
                &mut paint_orders,
                (
                    owner.controls.len(),
                    allocation
                        .copy_slice(&page.image_orders[source_image])
                        .map_err(message)?,
                ),
            )
            .map_err(message)?;
        allocation
            .push(
                &mut owner.controls,
                lower_picture(image, doc, left, top, id, &page.import_budget)?,
            )
            .map_err(message)?;
        id += 1;
    }

    let mut unknown_fonts = 0;
    let mut font_identity_mappings = std::collections::BTreeMap::new();
    let mut font_substitutions = std::collections::BTreeMap::new();
    // These are all fields changed from the common scaffold shape below.
    // Reuse equal source styles within a page instead of allocating one per
    // PDF text-show. Source spans and paint order remain separately recorded.
    let mut source_shapes = std::collections::BTreeMap::new();
    let mut program_families = std::collections::BTreeMap::new();
    for (span_index, span) in page.spans.iter().enumerate() {
        if span.text.is_empty() {
            continue;
        }
        let scoped_font = page.span_fonts[span_index].as_ref();
        let key = scoped_font.map(|font| std::sync::Arc::as_ptr(font) as usize);
        if let (Some(key), Some(font)) = (key, scoped_font) {
            if !program_families.contains_key(&key) {
                let family = program_family(font, allocation)?;
                allocation
                    .insert(&mut program_families, key, family)
                    .map_err(message)?;
            }
        }
        let declared_family = key
            .and_then(|key| program_families.get(&key))
            .and_then(Option::as_deref);
        let font_name = if let Some(family) = declared_family {
            family
        } else if span.font_name == "Unknown" || span.font_name.is_empty() {
            unknown_fonts += 1;
            "함초롬바탕"
        } else {
            imported_font_family(&span.font_name)
        };
        if font_name != span.font_name {
            allocation
                .insert(
                    &mut font_identity_mappings,
                    span.font_name.as_str(),
                    allocation.string(font_name).map_err(message)?,
                )
                .map_err(message)?;
        }
        // ISO 32000-1 standard Type1 Helvetica can omit its program. Preserve
        // that declaration and its AFM advances; make the paint/save fallback
        // explicit instead of depending on each host's unrelated default face.
        // FontInfo is the actual scoped resource, not a page-level name guess.
        let substitute = page.span_fonts[span_index]
            .as_ref()
            .filter(|font| {
                font.subtype == "Type1"
                    && font.embedded_program_present == Some(false)
                    && imported_font_family(&font.base_font) == font_name
                    && matches!(
                        font_name,
                        "Helvetica"
                            | "Helvetica-Bold"
                            | "Helvetica-Oblique"
                            | "Helvetica-BoldOblique"
                    )
            })
            .map(|_| {
                Ok::<_, String>(SubstFont {
                    face: allocation.string("Arial").map_err(message)?,
                    font_type: 1,
                    ..Default::default()
                })
            })
            .transpose()?;
        let font_id = if let Some(i) = doc.doc_info.font_faces[0]
            .iter()
            .position(|f| f.name == font_name && f.subst_font == substitute)
        {
            i as u16
        } else {
            let i = doc.doc_info.font_faces[0].len() as u16;
            for group in &mut doc.doc_info.font_faces {
                allocation
                    .push(
                        group,
                        Font {
                            name: allocation.string(font_name).map_err(message)?,
                            subst_font: substitute
                                .as_ref()
                                .map(|s| allocation.clone_record(s).map_err(message))
                                .transpose()?,
                            ..Default::default()
                        },
                    )
                    .map_err(message)?;
            }
            i
        };
        if let Some(substitute) = &substitute {
            allocation
                .insert(
                    &mut font_substitutions,
                    font_id,
                    request_json!(allocation, {
                        "sourceFace":font_name, "sourceProgram":"absent",
                        "substituteFace":substitute.face, "reason":"pdf-base14-without-program",
                    })?,
                )
                .map_err(message)?;
        }
        let mut cs = allocation
            .clone_record(&doc.doc_info.char_shapes[0])
            .map_err(message)?;
        cs.font_ids = [font_id; 7];
        cs.base_size = hu(span.font_size)? as i32;
        if span.rotation_degrees != 0.0 {
            cs.ratios = [span.horizontal_scaling.round() as u8; 7];
        }
        cs.bold = span.font_weight.is_bold();
        cs.italic = span.is_italic;
        cs.text_color = rgb(span.color);
        cs.text_paint = page.span_paints[span_index];
        // Type3 metrics-only glyph programs carry searchable text without ink.
        // Retain the strings/advances in editable IR, but do not invent a second
        // glyph from a host fallback font over the PDF's independent artwork.
        if scoped_font.is_some_and(|font| font.type3_metrics_only) {
            cs.text_paint = Some(crate::model::text_paint::TextPaint {
                fill: false,
                width_mpt: 0,
                stroke_color: 0,
                fill_alpha: 0,
                stroke_alpha: 0,
                line_join: 0,
                miter_limit_milli: 10_000,
            });
        }
        // PDF space advances belong to the source font/text state, not HWP's
        // default half-em space. Select the existing common IR contract only
        // when measured origin-to-origin spaces agree with the declared metric.
        // Nominal char_widths omit TJ/Tw displacement, so they cannot prove it.
        if let Some(metric) =
            crate::renderer::font_metrics_data::find_metric(font_name, cs.bold, cs.italic)
        {
            if let Some(space) = metric.metric.get_width(' ') {
                let expected = span.font_size * f32::from(space) / f32::from(metric.metric.em_size);
                let source_spaces: Vec<_> = allocation
                    .collect(
                        span.text
                            .chars()
                            .enumerate()
                            .filter(|(_, ch)| *ch == ' ')
                            .map(|(i, _)| {
                                span.char_x_offsets
                                    .get(i + 1)
                                    .zip(span.char_x_offsets.get(i))
                                    .map(|(next, here)| next - here)
                            }),
                    )
                    .map_err(message)?;
                if span.char_x_offsets.len() == span.text.chars().count()
                    && !source_spaces.is_empty()
                    && source_spaces
                        .iter()
                        .all(|width| width.is_some_and(|width| (width - expected).abs() <= 0.01))
                {
                    cs.use_font_space = true;
                }
            }
        }
        let key = (
            font_id,
            cs.base_size,
            cs.ratios,
            cs.bold,
            cs.italic,
            cs.text_color,
            cs.use_font_space,
            cs.text_paint,
        );
        let shape = if let Some(&id) = source_shapes.get(&key) {
            id
        } else {
            let id = doc.doc_info.char_shapes.len() as u32;
            allocation
                .push(&mut doc.doc_info.char_shapes, cs)
                .map_err(message)?;
            allocation
                .insert(&mut source_shapes, key, id)
                .map_err(message)?;
            id
        };
        // On the supported unrotated path, bbox.y is a baseline. Exact font metrics remain a limitation.
        let attrs = common(
            span.bbox.x - left,
            top - span.bbox.y - span.font_size * 0.95,
            span.bbox.width,
            span.font_size,
            id,
        )?;
        let width = attrs.width;
        let height = attrs.height;
        let text = super::structured_lowering::span_text_with_spaces(
            span,
            super::allocation::Allocation(&page.import_budget),
        )?;
        let mut paragraph = text_para(text, shape, width, height, allocation)?;
        paragraph.para_shape_id = single_line_id;
        paragraph.char_count_msb = true;
        let mut rect = RectangleShape {
            common: attrs,
            x_coords: [0, width as i32, width as i32, 0],
            y_coords: [0, 0, height as i32, height as i32],
            drawing: DrawingObjAttr {
                shape_attr: component(width, height),
                inst_id: id,
                text_box: Some(TextBox {
                    max_width: width,
                    text_margin_explicit: true,
                    paint_clip: super::text_clip::local_paths(
                        page.span_clips[span_index].as_deref(),
                        span.bbox.x,
                        span.bbox.y + span.font_size * 0.95,
                        allocation,
                    )?,
                    paragraphs: allocation.collect([paragraph]).map_err(message)?,
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        super::rotated_text::apply(
            &mut rect,
            span,
            page.media_box,
            page.span_clips[span_index].as_deref(),
            allocation,
        )?;
        allocation
            .push(
                &mut paint_orders,
                (
                    owner.controls.len(),
                    allocation
                        .copy_slice(&page.span_orders[span_index])
                        .map_err(message)?,
                ),
            )
            .map_err(message)?;
        owner.controls.push(Control::Shape(
            allocation
                .boxed(ShapeObject::Rectangle(rect))
                .map_err(message)?,
        ));
        id += 1;
    }
    allocation
        .sort_by(&mut paint_orders, |a, b| a.1.cmp(&b.1))
        .map_err(message)?;
    for (z, (control, _)) in paint_orders.iter().enumerate() {
        match &mut owner.controls[*control] {
            Control::Shape(s) => s.common_mut().z_order = z as i32,
            Control::Picture(p) => p.common.z_order = z as i32,
            _ => {}
        }
    }

    if unknown_fonts > 0 {
        allocation
            .push(
                &mut warnings,
                allocation
                    .display(&format_args!(
                        "{unknown_fonts} spans have Unknown font; explicit fallback used"
                    ))
                    .map_err(message)?,
            )
            .map_err(message)?;
    }
    if unsupported_paths > 0 {
        allocation
            .push(
                &mut warnings,
                allocation
                    .display(&format_args!(
                        "{unsupported_paths} paths preserved in extraction only"
                    ))
                    .map_err(message)?,
            )
            .map_err(message)?;
    }
    let detected = page.snapshot["tables"].as_array().map_or(0, Vec::len);
    if detected > 0 {
        allocation
            .push(
                &mut warnings,
                allocation
                    .display(&format_args!(
                        "{detected} detected tables not lowered to semantic Table IR yet"
                    ))
                    .map_err(message)?,
            )
            .map_err(message)?;
    }
    owner.char_count = 1 + owner.controls.len() as u32 * 8;
    owner.control_mask = 0x00000800;
    doc.sections[section].paragraphs = allocation.collect([owner]).map_err(message)?;
    *next_id = id;
    let report = request_json!(allocation, {
        "mode":"positional-page", "textBoxes":page.spans.iter().filter(|s|!s.text.is_empty()).count(),
        "pictures":doc.bin_data_content.len(), "straightRules":lines,
        "paintedRuleRectangles":painted,
        "ruleSourceControls":rule_sources,
        "imageSourceControls":image_sources,
        "compoundPathPictures":local_path_pictures,
        "ruleSourceControlScope":"positional controls before table replacement; instance IDs remain stable",
        "singleLineParagraphs":"SQUEEZE",
        "detectedTables":detected, "semanticTables":0,
        "unsupportedPaths":unsupported_paths, "unknownFontSpans":unknown_fonts,
        "fontIdentityMappings":font_identity_mappings,
        "fontSubstitutions":font_substitutions,
        "compatibilityCorrections":["picture-point-pair-layout", "picture-frame-HWPUNIT", "full-image-crop-extent", "explicit-zero-text-margins", "single-line-span-reflow"],
        "hancomCompatibility":"unverified-awaiting-editor-retest",
        "warnings":warnings
    })?;
    Ok(report)
}

pub(super) fn lower_picture(
    image: &pdf_oxide::extractors::PdfImage,
    doc: &mut Document,
    left: f32,
    top: f32,
    id: u32,
    budget: &pdf_oxide::import_limits::ImportBudget,
) -> Result<Control, String> {
    let allocation = Allocation(budget);
    let b = image.bbox().ok_or("image has no placement geometry")?;
    let matrix = image.matrix();
    // HWP image dimensions use 1/7200 inch at the 96 dpi import scale; crop
    // coordinates are signed i32. A pixel-area cap alone does not bound a
    // single long dimension. Reject before PNG encoding or resource insertion.
    let dimension = |pixels: u32| {
        pixels
            .checked_mul(75)
            .filter(|&value| value <= i32::MAX as u32)
            .ok_or("image dimension exceeds HWP crop coordinate range")
    };
    let img_dim = (dimension(image.width())?, dimension(image.height())?);
    let data = image
        .to_png_bytes_with_budget(budget)
        .map_err(|e| e.to_string())?;
    let storage = (doc.bin_data_content.len() + 1) as u16;
    allocation
        .push(
            &mut doc.doc_info.bin_data_list,
            BinData {
                // HWP5 table 18: embedded payloads use type 1, default compression,
                // and NotAccessed. The binary writer emits this attribute verbatim.
                // Leaving it at 0 labels the storage-id/extension payload as a link.
                attr: 0x0001,
                data_type: BinDataType::Embedding,
                storage_id: storage,
                extension: Some(allocation.string("png").map_err(message)?),
                ..Default::default()
            },
        )
        .map_err(message)?;
    allocation
        .push(
            &mut doc.bin_data_content,
            BinDataContent {
                id: storage,
                extension: allocation.string("png").map_err(message)?,
                data: BinDataBytes::Loaded(data),
            },
        )
        .map_err(message)?;
    let attrs = common(b.x - left, top - b.y - b.height, b.width, b.height, id)?;
    // PDF clips are in page user space; HWP crop coordinates select a region
    // of the full bitmap. The visible frame and the selected pixels have the
    // same proportions, preserving scale instead of fitting all pixels into
    // a smaller cell. PDF y points up, image crop y points down.
    let crop_x = |x: f32| {
        ((f64::from(x) - f64::from(matrix[4])) / f64::from(matrix[0]) * f64::from(img_dim.0))
            .round() as i32
    };
    let crop_y = |y: f32| {
        ((f64::from(matrix[5]) + f64::from(matrix[3]) - f64::from(y)) / f64::from(matrix[3])
            * f64::from(img_dim.1))
        .round() as i32
    };
    let pic = Picture {
        shape_attr: component(attrs.width, attrs.height),
        common: attrs,
        instance_id: id,
        img_dim,
        // Picture uses packed point pairs, not separate X/Y arrays.
        border_x: [0, 0, hu(b.width)? as i32, 0],
        border_y: [
            hu(b.width)? as i32,
            hu(b.height)? as i32,
            0,
            hu(b.height)? as i32,
        ],
        crop: CropInfo {
            left: crop_x(b.x),
            top: crop_y(b.y + b.height),
            right: crop_x(b.x + b.width),
            bottom: crop_y(b.y),
        },
        image_attr: ImageAttr {
            bin_data_id: storage,
            ..Default::default()
        },
        ..Default::default()
    };
    Ok(Control::Picture(allocation.boxed(pic).map_err(message)?))
}
