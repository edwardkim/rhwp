use super::ExtractedPage;
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
        Alignment, Fill, FillType, Font, ShapeBorderLine, SolidFill, PARA_ATTR2_LINE_WRAP_MASK,
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

fn text_para(text: &str, shape: u32, width: u32, size: u32) -> Paragraph {
    let mut length = 0;
    let char_offsets = text
        .chars()
        .map(|c| {
            let offset = length;
            length += c.len_utf16() as u32;
            offset
        })
        .collect();
    Paragraph {
        text: text.into(),
        char_offsets,
        char_count: length + 1,
        has_para_text: true,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: shape,
        }],
        line_segs: vec![LineSeg {
            line_height: size as i32,
            text_height: size as i32,
            baseline_distance: (size as f32 * 0.95).round() as i32,
            segment_width: width as i32,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }],
        ..Default::default()
    }
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
    let (left, bottom, right, top) = page.media_box;
    let section = doc.sections.len() - 1;
    let pd = &mut doc.sections[section].section_def.page_def;
    pd.width = hu(right - left)?;
    pd.height = hu(top - bottom)?;
    pd.margin_left = 0;
    pd.margin_right = 0;
    pd.margin_top = 0;
    pd.margin_bottom = 0;
    pd.margin_header = 0;
    pd.margin_footer = 0;
    pd.margin_gutter = 0;
    // Each PDF span belongs to one source line. Keep that constraint on editor reflow.
    let mut single_line = doc.doc_info.para_shapes[0].clone();
    single_line.raw_data = None;
    single_line.attr2 = (single_line.attr2 & !PARA_ATTR2_LINE_WRAP_MASK) | 1;
    let single_line_id = doc.doc_info.para_shapes.len() as u16;
    doc.doc_info.para_shapes.push(single_line);
    let mut owner = Paragraph::new_empty();
    let mut id = *next_id;
    let mut lines = 0;
    let mut painted = 0;
    let mut unsupported_paths = 0;
    let mut rule_sources = Vec::new();
    let mut image_sources = Vec::new();
    let mut paint_orders = Vec::new();
    let mut local_path_pictures = 0;
    let mut warnings = vec![
        "positional text boxes, not semantic paragraph/table reconstruction".to_string(),
        "Text/path interleaving, text clipping, blend modes and embedded fonts are not fully reconstructed"
            .into(),
    ];

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
                rule_sources.push(
                    json!({"path":source_path,"control":owner.controls.len(),"instanceId":id}),
                );
                paint_orders.push((owner.controls.len(), path.paint.source_order.clone()));
                owner.controls.push(control);
                id += 1;
            }
            continue;
        }
        paint_orders.push((owner.controls.len(), path.paint.source_order.clone()));
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
            rule_sources
                .push(json!({"path":source_path,"control":owner.controls.len(),"instanceId":id}));
            owner
                .controls
                .push(Control::Shape(Box::new(ShapeObject::Rectangle(rect))));
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
        rule_sources
            .push(json!({"path":source_path,"control":owner.controls.len(),"instanceId":id}));
        owner
            .controls
            .push(Control::Shape(Box::new(ShapeObject::Line(shape))));
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
        if u64::from(image.width()) * u64::from(image.height()) > 16000000 {
            return Err(
                "image pixel budget exceeded (after extraction; not a parser allocation guarantee)"
                    .into(),
            );
        }
        image_sources.push(json!({"image":source_image,"control":owner.controls.len()}));
        paint_orders.push((
            owner.controls.len(),
            page.image_orders[source_image].clone(),
        ));
        owner.controls.push(lower_picture(
            image,
            doc,
            left,
            top,
            id,
            &page.import_budget,
        )?);
        id += 1;
    }

    paint_orders.sort_by(|a, b| a.1.cmp(&b.1));
    for (z, (control, _)) in paint_orders.iter().enumerate() {
        match &mut owner.controls[*control] {
            Control::Shape(s) => s.common_mut().z_order = z as i32,
            Control::Picture(p) => p.common.z_order = z as i32,
            _ => {}
        }
    }

    let mut unknown_fonts = 0;
    for span in &page.spans {
        if span.text.is_empty() {
            continue;
        }
        if span.rotation_degrees != 0.0 || span.wmode != 0 || span.mirrored {
            return Err("rotated/vertical/mirrored text requires additional lowering".into());
        }
        let font_name = if span.font_name == "Unknown" || span.font_name.is_empty() {
            unknown_fonts += 1;
            "함초롬바탕"
        } else {
            // ISO 32000-1 9.6.4: a six-uppercase-letter subset tag is not
            // part of the installed font family/PostScript name.
            span.font_name
                .split_once('+')
                .filter(|(tag, _)| tag.len() == 6 && tag.bytes().all(|b| b.is_ascii_uppercase()))
                .map_or(span.font_name.as_str(), |(_, family)| family)
        };
        let font_id = if let Some(i) = doc.doc_info.font_faces[0]
            .iter()
            .position(|f| f.name == font_name)
        {
            i as u16
        } else {
            let i = doc.doc_info.font_faces[0].len() as u16;
            for group in &mut doc.doc_info.font_faces {
                group.push(Font {
                    name: font_name.into(),
                    ..Default::default()
                });
            }
            i
        };
        let mut cs = doc.doc_info.char_shapes[0].clone();
        cs.font_ids = [font_id; 7];
        cs.base_size = hu(span.font_size)? as i32;
        cs.bold = span.font_weight.is_bold();
        cs.italic = span.is_italic;
        cs.text_color = rgb(span.color);
        let shape = doc.doc_info.char_shapes.len() as u32;
        doc.doc_info.char_shapes.push(cs);
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
        let text = super::structured_lowering::span_text_with_spaces(span);
        let mut paragraph = text_para(&text, shape, width, height);
        paragraph.para_shape_id = single_line_id;
        paragraph.char_count_msb = true;
        let rect = RectangleShape {
            common: attrs,
            x_coords: [0, width as i32, width as i32, 0],
            y_coords: [0, 0, height as i32, height as i32],
            drawing: DrawingObjAttr {
                shape_attr: component(width, height),
                inst_id: id,
                text_box: Some(TextBox {
                    max_width: width,
                    text_margin_explicit: true,
                    paragraphs: vec![paragraph],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        owner
            .controls
            .push(Control::Shape(Box::new(ShapeObject::Rectangle(rect))));
        id += 1;
    }
    if unknown_fonts > 0 {
        warnings.push(format!(
            "{unknown_fonts} spans have Unknown font; explicit fallback used"
        ));
    }
    if unsupported_paths > 0 {
        warnings.push(format!(
            "{unsupported_paths} paths preserved in extraction only"
        ));
    }
    let detected = page.snapshot["tables"].as_array().map_or(0, Vec::len);
    if detected > 0 {
        warnings.push(format!(
            "{detected} detected tables not lowered to semantic Table IR yet"
        ));
    }
    owner.char_count = 1 + owner.controls.len() as u32 * 8;
    owner.control_mask = 0x00000800;
    doc.sections[section].paragraphs = vec![owner];
    *next_id = id;
    let report = json!({
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
        "compatibilityCorrections":["picture-point-pair-layout", "picture-frame-HWPUNIT", "full-image-crop-extent", "explicit-zero-text-margins", "single-line-span-reflow"],
        "hancomCompatibility":"unverified-awaiting-editor-retest",
        "warnings":warnings
    });
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
    let b = image.bbox().ok_or("image has no placement geometry")?;
    let matrix = image.matrix();
    let data = image
        .to_png_bytes_with_budget(budget)
        .map_err(|e| e.to_string())?;
    let storage = (doc.bin_data_content.len() + 1) as u16;
    doc.doc_info.bin_data_list.push(BinData {
        // HWP5 table 18: embedded payloads use type 1, default compression,
        // and NotAccessed. The binary writer emits this attribute verbatim.
        // Leaving it at 0 labels the storage-id/extension payload as a link.
        attr: 0x0001,
        data_type: BinDataType::Embedding,
        storage_id: storage,
        extension: Some("png".into()),
        ..Default::default()
    });
    doc.bin_data_content.push(BinDataContent {
        id: storage,
        extension: "png".into(),
        data: BinDataBytes::Loaded(data),
    });
    let attrs = common(b.x - left, top - b.y - b.height, b.width, b.height, id)?;
    // PDF clips are in page user space; HWP crop coordinates select a region
    // of the full bitmap. The visible frame and the selected pixels have the
    // same proportions, preserving scale instead of fitting all pixels into
    // a smaller cell. PDF y points up, image crop y points down.
    let img_dim = (image.width() * 75, image.height() * 75);
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
    Ok(Control::Picture(Box::new(pic)))
}
