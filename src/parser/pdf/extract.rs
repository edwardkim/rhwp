use super::{recognition, table_detection, Budget, ExtractedPage, PdfImportError};
use pdf_oxide::PdfDocument;
use serde_json::Value;

fn checked<T>(
    pdf: &PdfDocument,
    result: Result<T, pdf_oxide::error::Error>,
    stage: &'static str,
    page: usize,
) -> Result<T, PdfImportError> {
    let error = |error| {
        PdfImportError::backend(error).at(stage, Some(page), pdf.import_budget().map(AsRef::as_ref))
    };
    pdf.check_import_budget().map_err(error)?;
    result.map_err(error)
}

pub(super) fn extract(
    pdf: &PdfDocument,
    page: usize,
    budget: &mut Budget,
    include_table_inference: bool,
) -> Result<ExtractedPage, PdfImportError> {
    let request = pdf
        .import_budget()
        .cloned()
        .ok_or_else(|| PdfImportError::budget("missing backend request budget"))?;
    let allocation = super::allocation::Allocation(&request);
    let media_box = checked(pdf, pdf.get_page_media_box(page), "media_box", page)?;
    let rotation = checked(pdf, pdf.get_page_rotation(page), "rotation", page)?;
    let visible_box = checked(pdf, pdf.get_page_visible_box(page), "visible_box", page)?;
    let (left, bottom, right, top) = media_box;
    if rotation != 0 || visible_box != media_box {
        return Err(PdfImportError::new(
            "PDF_GEOMETRY_UNSUPPORTED",
            "rotated/cropped pages are not supported",
        ));
    }
    if ![left, bottom, right, top].iter().all(|n| n.is_finite())
        || right <= left
        || top <= bottom
        || right - left > 100_000.
        || top - bottom > 100_000.
    {
        return Err(PdfImportError::new(
            "PDF_GEOMETRY_UNSUPPORTED",
            "invalid page dimensions",
        ));
    }
    let painted = checked(
        pdf,
        pdf.extract_ordered_painted_spans_with_fonts(page),
        "text",
        page,
    )?;
    let mut spans = allocation.vec(painted.len())?;
    let mut span_orders = allocation.vec(painted.len())?;
    let mut span_fonts = allocation.vec(painted.len())?;
    for painted in painted {
        let pdf_oxide::layout::text_block::PaintedTextSpan {
            span,
            order,
            basis,
            font,
        } = painted;
        super::rotated_text::validate(&span, basis).map_err(|message| {
            PdfImportError::new("PDF_TEXT_UNSUPPORTED", message).at(
                "text",
                Some(page),
                pdf.import_budget().map(AsRef::as_ref),
            )
        })?;
        spans.push(span);
        span_orders.push(order);
        span_fonts.push(font);
    }
    let (paths, clips) = checked(pdf, pdf.extract_paths_with_clips(page), "paths", page)?;
    let mut uses = std::collections::BTreeMap::new();
    for (order, clip) in &clips.text_uses {
        if uses.contains_key(order) {
            return Err(PdfImportError::new(
                "PDF_TEXT_UNSUPPORTED",
                "ambiguous text-paint clip witness",
            ));
        }
        allocation
            .tree_entry::<&Vec<usize>, &Option<std::sync::Arc<pdf_oxide::elements::PathClip>>>()?;
        uses.insert(order, clip);
    }
    let mut span_clips = allocation.vec(span_orders.len())?;
    for order in &span_orders {
        span_clips.push(uses.get(order).map(|clip| (*clip).clone()).ok_or_else(|| {
            PdfImportError::new(
                "PDF_TEXT_UNSUPPORTED",
                "text-paint clip witness missing or budget exceeded",
            )
        })?);
    }
    let mut paints = std::collections::BTreeMap::new();
    for (order, paint) in &clips.text_paints {
        allocation.tree_entry::<&Vec<usize>, &pdf_oxide::elements::TextStrokePaint>()?;
        paints.insert(order, paint);
    }
    let mut span_paints = allocation.vec(span_orders.len())?;
    for order in &span_orders {
        let paint = paints.get(order).map(|p| {
            let color = p.color;
            crate::model::text_paint::TextPaint {
                fill: p.fill,
                width_mpt: (f64::from(p.width) * 1000.).round() as u32,
                stroke_color: super::lowering::rgb(pdf_oxide::layout::Color::new(
                    color.0, color.1, color.2,
                )),
                fill_alpha: (p.fill_alpha.clamp(0., 1.) * 255.).round() as u8,
                stroke_alpha: (p.stroke_alpha.clamp(0., 1.) * 255.).round() as u8,
                line_join: p.line_join,
                miter_limit_milli: (p.miter_limit * 1000.).round() as u32,
            }
        });
        if paint.is_some_and(|p| !p.valid()) {
            return Err(PdfImportError::new(
                "PDF_TEXT_UNSUPPORTED",
                "invalid text stroke paint",
            ));
        }
        span_paints.push(paint);
    }
    for path in &paths {
        let next = budget
            .path_operations
            .checked_add(path.operations.len())
            .ok_or_else(|| PdfImportError::budget("path operation count overflow"))?;
        if path.operations.len() > budget.policy.max_path_operations {
            return Err(
                PdfImportError::budget("document path operation budget exceeded").limit(
                    "path operations",
                    0,
                    path.operations.len(),
                    budget.policy.max_path_operations,
                ),
            );
        }
        if next > budget.policy.max_document_path_operations {
            return Err(
                PdfImportError::budget("document path operation budget exceeded").limit(
                    "document path operations",
                    budget.path_operations,
                    path.operations.len(),
                    budget.policy.max_document_path_operations,
                ),
            );
        }
        budget.path_operations = next;
    }
    budget.spans = budget
        .spans
        .checked_add(spans.len())
        .ok_or_else(|| PdfImportError::budget("span count overflow"))?;
    // Paths already consumed the request-wide operation budget above. The
    // bounded extractor reserves their storage; lowering checks format ID
    // capacity before materialization. A second fixed path-count cap would
    // reject dense pages independently of their selected request policy.
    if spans.len() > 20_000 || budget.spans > budget.policy.backend.spans {
        return Err(PdfImportError::budget("extracted element budget exceeded"));
    }
    for span in &spans {
        budget.text_bytes = budget
            .text_bytes
            .checked_add(span.text.len())
            .ok_or_else(|| PdfImportError::budget("text byte count overflow"))?;
        if budget.text_bytes > budget.policy.max_text_bytes || span.font_name.len() > 512 {
            return Err(PdfImportError::budget(
                "document extracted text exceeds request limit",
            ));
        }
        if ![
            span.font_size,
            span.bbox.x,
            span.bbox.y,
            span.bbox.width,
            span.bbox.height,
        ]
        .iter()
        .all(|n| n.is_finite())
            || span.font_size <= 0.
        {
            return Err(PdfImportError::new(
                "PDF_GEOMETRY_UNSUPPORTED",
                "invalid span metrics",
            ));
        }
    }
    // Inspect declared image extents before pixel decode. Enumerating handles
    // still decompresses content upstream and is not a parser allocation bound.
    let handles = checked(pdf, pdf.page_image_handles(page), "image_handles", page)?;
    // Count actual paint placements, not unique resources. Reusing one image
    // with another CTM/clip/opacity is real output and must not be deduplicated.
    // Reserve before decode and before allocating the lowered image vectors.
    let next = budget
        .image_placements
        .checked_add(handles.len())
        .ok_or_else(|| PdfImportError::budget("image placement count overflow"))?;
    if next > budget.policy.max_document_image_placements {
        return Err(
            PdfImportError::budget("document image placement budget exceeded").limit(
                "document image placements",
                budget.image_placements,
                handles.len(),
                budget.policy.max_document_image_placements,
            ),
        );
    }
    budget.image_placements = next;
    // A stencil texture may be emitted as thousands of tiny Do invocations.
    // Only an uninterrupted image-paint interval may become one raster picture;
    // text and paths remain separate editable IR and keep their original order.
    let mut barriers = allocation.vec(
        span_orders
            .len()
            .checked_add(paths.len())
            .ok_or_else(|| PdfImportError::budget("paint barrier count overflow"))?,
    )?;
    for order in span_orders
        .iter()
        .chain(paths.iter().map(|path| &path.paint.source_order))
    {
        barriers.push(allocation.copy_slice(order)?);
    }
    barriers.sort_unstable();
    let mut images = Vec::new();
    let mut image_orders = Vec::new();
    let mut index = 0;
    // A rejected run (ordinary 1-bit image, sparse frame, interpolation, etc.)
    // must fall back linearly, not rescan every remaining suffix.
    let mut ordinary_until = 0;
    while index < handles.len() {
        let handle = &handles[index];
        let next_barrier = barriers.partition_point(|order| order <= &handle.source_order);
        let mut end = index + 1;
        if index >= ordinary_until && handle.bits_per_component == 1 && handle.opacity != 0.0 {
            while end < handles.len()
                && handle.same_stencil_tile_source(&handles[end])
                && barriers
                    .get(next_barrier)
                    .is_none_or(|order| order > &handles[end].source_order)
            {
                end += 1;
            }
        }
        if end > index + 1 {
            let source_pixels = handles[index..end]
                .iter()
                .try_fold(0u64, |total, h| {
                    total.checked_add(u64::from(h.width) * u64::from(h.height))
                })
                .ok_or_else(|| PdfImportError::budget("image pixel count overflow"))?;
            let remaining = (budget.policy.max_document_image_pixels as u64)
                .saturating_sub(budget.image_pixels);
            if source_pixels > remaining {
                return Err(PdfImportError::budget(
                    "declared image pixel budget exceeded",
                ));
            }
            if let Some(image) = checked(
                pdf,
                pdf_oxide::extractors::images::PdfImageHandle::decode_stencil_tiles(
                    &handles[index..end],
                    remaining.min(budget.policy.max_image_pixels as u64),
                ),
                "image_decode",
                page,
            )? {
                let output_pixels = u64::from(image.width()) * u64::from(image.height());
                budget.image_pixels += source_pixels.max(output_pixels);
                allocation.reserve(&mut images, 1)?;
                allocation.reserve(&mut image_orders, 1)?;
                let order = allocation.copy_slice(&handle.source_order)?;
                images.push(image);
                image_orders.push(order);
                index = end;
                continue;
            }
            ordinary_until = end;
        }
        index += 1;
        if handle.opacity == 0.0 {
            continue;
        }
        let Some(visible) = checked(
            pdf,
            handle.clip.visible_bounds(handle.bbox),
            "image_clip",
            page,
        )
        .map_err(|mut error| {
            if error.diagnostics.as_ref().and_then(|d| d.backend_kind) == Some("unsupported") {
                error.code = "PDF_IMAGE_UNSUPPORTED";
                error.message = "image clipping path is not supported";
            }
            error
        })?
        else {
            // This paint operation has an empty intersection with the active clip.
            continue;
        };
        let pixels = u64::from(handle.width) * u64::from(handle.height);
        budget.image_pixels = budget
            .image_pixels
            .checked_add(pixels)
            .ok_or_else(|| PdfImportError::budget("image pixel count overflow"))?;
        if pixels == 0
            || pixels > budget.policy.max_image_pixels as u64
            || budget.image_pixels > budget.policy.max_document_image_pixels as u64
        {
            return Err(PdfImportError::budget(
                "declared image pixel budget exceeded",
            ));
        }
        let decoded = handle.decode();
        let mut image = checked(pdf, decoded, "image_decode", page).map_err(|mut error| {
            if error.diagnostics.as_ref().and_then(|d| d.backend_kind) == Some("unsupported") {
                error.code = "PDF_IMAGE_UNSUPPORTED";
                error.message = "image codec or mask is not supported by the bounded import path";
            }
            error
        })?;
        // Handle decode validates the codec's declared dimensions before mask
        // composition. A higher-resolution mask may then increase the bitmap
        // resolution without changing CTM/visible bounds. Account those pixels
        // as well; a 2x2 colour image must not hide a large alpha plane.
        let decoded_pixels = u64::from(image.width()) * u64::from(image.height());
        if decoded_pixels > pixels {
            budget.image_pixels = budget
                .image_pixels
                .checked_add(decoded_pixels - pixels)
                .ok_or_else(|| PdfImportError::budget("composited image pixel count overflow"))?;
            if decoded_pixels > budget.policy.max_image_pixels as u64
                || budget.image_pixels > budget.policy.max_document_image_pixels as u64
            {
                return Err(PdfImportError::budget(
                    "composited image pixel budget exceeded",
                ));
            }
        }
        // Ownership inference and lowering consume the same visible bounds.
        // Keep the full bitmap and CTM so lowering can preserve an editable crop.
        image.set_bbox(visible);
        allocation.reserve(&mut images, 1)?;
        allocation.reserve(&mut image_orders, 1)?;
        let order = allocation.copy_slice(&handle.source_order)?;
        image_orders.push(order);
        images.push(image);
    }
    let mut table_inference = super::table_inference::Ledger::observe(
        &spans,
        &span_orders,
        &images,
        &image_orders,
        &clips,
        media_box,
        std::sync::Arc::clone(&request),
        include_table_inference,
    )?;
    let detection = table_detection::detect_ruled_tables(
        &spans,
        &paths,
        &images,
        media_box,
        &table_inference.text,
        &span_orders,
        &clips,
    );
    request.check().map_err(PdfImportError::backend)?;
    let detection = detection.map_err(|_| {
        PdfImportError::budget("table geometry inference budget or geometry rejected")
    })?;
    if let Some(tables) = detection["tables"].as_array() {
        for table in tables {
            budget.cells = budget
                .cells
                .checked_add(table["cells"].as_array().map_or(0, Vec::len))
                .ok_or_else(|| PdfImportError::budget("table cell count overflow"))?;
            // Every emitted cell requires at least one paragraph resource.
            // Use the request's finite entry policy and the IR's u16 resource
            // domain, rather than an unrelated smaller document-wide cap.
            // Per-page tile/work limits and lowering's pool preflight remain.
            if budget.cells > budget.policy.backend.entries.min(super::MAX_POOL_ENTRIES) {
                return Err(PdfImportError::budget(
                    "document inferred cell budget exceeded",
                ));
            }
        }
    }
    let recognition = recognition::recognize_page(
        &table_inference.text,
        &spans,
        &span_clips,
        &images,
        &detection,
        media_box,
        recognition::Policy::default(),
    );
    // Optional inference can defer a semantic hypothesis, but a failed request
    // reservation must stay fatal with its resource diagnostics intact.
    request.check().map_err(PdfImportError::backend)?;
    let recognition =
        recognition.map_err(|reason| {
            let mut error =
                PdfImportError::new("PDF_RECOGNITION_FAILED", "table ownership inference failed")
                    .at("recognition", Some(page), None);
            // Retain the failed rule in opt-in diagnostics, not the public message.
            error.diagnostics.as_mut().unwrap().backend_message = Some(reason);
            error
        })?;
    table_inference.reconcile(&detection)?;
    // Move detection into the internal snapshot; recognition already owns its
    // report and lowering reads it directly. Avoid copying both evidence trees.
    let mut snapshot = Value::Object(allocation.object()?);
    allocation.set(&mut snapshot, "tableDetection", detection)?;
    Ok(ExtractedPage {
        import_budget: request,
        media_box,
        spans,
        span_orders,
        span_fonts,
        span_clips,
        span_paints,
        images,
        image_orders,
        paths,
        snapshot,
        recognition,
        table_inference,
    })
}
