use super::{
    recognition, table_detection, Budget, ExtractedPage, PdfImportError, MAX_DOCUMENT_IMAGE_PIXELS,
    MAX_TEXT_BYTES,
};
use pdf_oxide::PdfDocument;
use serde_json::json;

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
) -> Result<ExtractedPage, PdfImportError> {
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
    let spans = checked(pdf, pdf.extract_painted_spans(page), "text", page)?;
    let paths = checked(pdf, pdf.extract_paths(page), "paths", page)?;
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
    if spans.len() > 20_000 || paths.len() > 20_000 || budget.spans > budget.policy.backend.spans {
        return Err(PdfImportError::budget("extracted element budget exceeded"));
    }
    for span in &spans {
        budget.text_bytes = budget
            .text_bytes
            .checked_add(span.text.len())
            .ok_or_else(|| PdfImportError::budget("text byte count overflow"))?;
        if budget.text_bytes > MAX_TEXT_BYTES || span.font_name.len() > 512 {
            return Err(PdfImportError::budget(
                "document extracted text exceeds 2 MiB",
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
        if span.rotation_degrees != 0. || span.wmode != 0 || span.mirrored {
            return Err(PdfImportError::new(
                "PDF_TEXT_UNSUPPORTED",
                "rotated/vertical/mirrored text is not supported",
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
    let mut images = Vec::with_capacity(handles.len());
    let mut image_orders = Vec::with_capacity(handles.len());
    for handle in handles {
        if handle.opacity == 0.0 {
            continue;
        }
        let Some(visible) = handle.clip.visible_bounds(handle.bbox).map_err(|_| {
            PdfImportError::new(
                "PDF_IMAGE_UNSUPPORTED",
                "non-rectangular image clipping is not supported",
            )
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
        if pixels == 0 || pixels > 16_000_000 || budget.image_pixels > MAX_DOCUMENT_IMAGE_PIXELS {
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
            if decoded_pixels > 16_000_000 || budget.image_pixels > MAX_DOCUMENT_IMAGE_PIXELS {
                return Err(PdfImportError::budget(
                    "composited image pixel budget exceeded",
                ));
            }
        }
        // Ownership inference and lowering consume the same visible bounds.
        // Keep the full bitmap and CTM so lowering can preserve an editable crop.
        image.set_bbox(visible);
        image_orders.push(handle.source_order.clone());
        images.push(image);
    }
    let detection = table_detection::detect_ruled_tables(&spans, &paths, &images, media_box)
        .map_err(|_| {
            PdfImportError::budget("table geometry inference budget or geometry rejected")
        })?;
    if let Some(tables) = detection["tables"].as_array() {
        for table in tables {
            budget.cells = budget
                .cells
                .checked_add(table["cells"].as_array().map_or(0, Vec::len))
                .ok_or_else(|| PdfImportError::budget("table cell count overflow"))?;
            if budget.cells > 16_384 {
                return Err(PdfImportError::budget(
                    "document inferred cell budget exceeded",
                ));
            }
        }
    }
    let recognition = recognition::recognize_page(
        &spans,
        &images,
        &detection,
        media_box,
        recognition::Policy::default(),
    )
    .map_err(|_| {
        PdfImportError::new("PDF_RECOGNITION_FAILED", "table ownership inference failed")
    })?;
    Ok(ExtractedPage {
        import_budget: pdf
            .import_budget()
            .cloned()
            .ok_or_else(|| PdfImportError::budget("missing backend request budget"))?,
        media_box,
        spans,
        images,
        image_orders,
        paths,
        snapshot: json!({"tableDetection": detection, "recognition": recognition}),
    })
}
