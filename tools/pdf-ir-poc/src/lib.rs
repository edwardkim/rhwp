pub mod hwpx_compatibility;
mod lowering;
pub mod recognition;
mod structured_lowering;
pub mod table_detection;
pub mod table_hierarchy;
pub mod table_topology;

use pdf_oxide::{elements::PathContent, extractors::PdfImage, layout::TextSpan, PdfDocument};
use rhwp::model::document::Document;
use serde_json::{json, Value};

pub struct Compilation {
    pub document: Document,
    pub hwpx: Vec<u8>,
    pub report: Value,
}

pub(crate) struct ExtractedPage {
    pub media_box: (f32, f32, f32, f32),
    pub spans: Vec<TextSpan>,
    pub images: Vec<PdfImage>,
    pub paths: Vec<PathContent>,
    pub snapshot: Value,
}

fn extract(bytes: &[u8], page: usize) -> Result<ExtractedPage, String> {
    if bytes.len() > 16 * 1024 * 1024 || !bytes.starts_with(b"%PDF-") {
        return Err("POC accepts PDF byte buffers up to 16 MiB".into());
    }
    let pdf = PdfDocument::from_bytes(bytes.to_vec()).map_err(|e| e.to_string())?;
    let count = pdf.page_count().map_err(|e| e.to_string())?;
    if page >= count || count > 128 {
        return Err("page out of range or POC page-count limit exceeded".into());
    }
    let rotation = pdf.get_page_rotation(page).map_err(|e| e.to_string())?;
    let media_box = pdf.get_page_media_box(page).map_err(|e| e.to_string())?;
    let visible_box = pdf.get_page_visible_box(page).map_err(|e| e.to_string())?;
    if rotation != 0 || visible_box != media_box {
        return Err("rotated/cropped pages require a coordinate adapter beyond this POC".into());
    }
    let spans = pdf.extract_spans(page).map_err(|e| e.to_string())?;
    let images = pdf.extract_images(page).map_err(|e| e.to_string())?;
    let paths = pdf.extract_paths(page).map_err(|e| e.to_string())?;
    let tables = pdf.extract_tables(page).map_err(|e| e.to_string())?;
    if spans.len() > 20000 || images.len() > 32 || paths.len() > 20000 {
        return Err("POC element budget exceeded".into());
    }
    let detection = table_detection::detect_ruled_tables(&spans, &paths, &images, media_box)?;
    let recognition = recognition::recognize_page(
        &spans,
        &images,
        &detection,
        media_box,
        recognition::Policy::default(),
    )?;
    let table_json: Vec<_> = tables
        .iter()
        .map(|table| {
            json!({
                "bbox": table.bbox, "colCount": table.col_count,
                "rows": table.rows.iter().map(|row| json!({
                    "header": row.is_header,
                    "cells": row.cells.iter().map(|cell| json!({
                        "text": cell.text, "bbox": cell.bbox,
                        "rowspan": cell.rowspan, "colspan": cell.colspan
                    })).collect::<Vec<_>>()
                })).collect::<Vec<_>>()
            })
        })
        .collect();
    let snapshot = json!({
        "pdfOxideVersion": "0.3.78", "sourcePage": page + 1,
        "sourcePageCount": count, "mediaBox": media_box,
        "visibleBox": visible_box, "rotation": rotation,
        "spans": spans, "tables": table_json,
        "tableDetection": detection,
        "recognition": recognition,
        "images": images.iter().map(|i| json!({
            "width":i.width(), "height":i.height(), "bbox":i.bbox(),
            "rotation":i.rotation_degrees(), "matrix":i.matrix()
        })).collect::<Vec<_>>(),
        "paths": paths,
        "untrustedContent": true,
        "security": "input/result budgets only; parser allocation, CPU and decompression isolation are not proven"
    });
    Ok(ExtractedPage {
        media_box,
        spans,
        images,
        paths,
        snapshot,
    })
}

pub fn inspect_page(bytes: &[u8], page: usize) -> Result<Value, String> {
    Ok(extract(bytes, page)?.snapshot)
}

pub fn compile_page(bytes: &[u8], page: usize) -> Result<Compilation, String> {
    compile_with_rules(bytes, page, false, false)
}

pub fn compile_page_painted_rules(bytes: &[u8], page: usize) -> Result<Compilation, String> {
    compile_with_rules(bytes, page, true, false)
}

pub fn compile_page_tables(bytes: &[u8], page: usize) -> Result<Compilation, String> {
    compile_with_rules(bytes, page, true, true)
}

fn compile_with_rules(
    bytes: &[u8],
    page: usize,
    painted_rules: bool,
    tables: bool,
) -> Result<Compilation, String> {
    let extracted = extract(bytes, page)?;
    let (document, mut lowering_report) = if tables {
        structured_lowering::lower(&extracted)?
    } else {
        lowering::lower(&extracted, painted_rules)?
    };
    let serialized = rhwp::serializer::serialize_hwpx(&document).map_err(|e| e.to_string())?;
    let (hwpx, margins) = hwpx_compatibility::explicit_zero_text_margins(&serialized)?;
    lowering_report["explicitZeroTextMargins"] = json!(margins);
    let report = json!({
        "extraction":extracted.snapshot, "lowering":lowering_report,
        "documentIr": {
            "sectionDef":document.sections[0].section_def,
            "paragraphs":document.sections[0].paragraphs,
            "charShapes":document.doc_info.char_shapes,
            "paraShapes":document.doc_info.para_shapes,
            "tabDefs":document.doc_info.tab_defs,
            "fonts":document.doc_info.font_faces
        }
    });
    Ok(Compilation {
        document,
        hwpx,
        report,
    })
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct PdfCompilation {
        hwpx: Vec<u8>,
        report: String,
    }

    #[wasm_bindgen]
    impl PdfCompilation {
        pub fn hwpx(&self) -> Vec<u8> {
            self.hwpx.clone()
        }
        pub fn report(&self) -> String {
            self.report.clone()
        }
    }

    #[wasm_bindgen]
    pub fn compile_pdf_page(bytes: &[u8], page: usize) -> Result<PdfCompilation, JsValue> {
        let compiled = super::compile_page(bytes, page).map_err(|e| JsValue::from_str(&e))?;
        Ok(PdfCompilation {
            hwpx: compiled.hwpx,
            report: compiled.report.to_string(),
        })
    }

    #[wasm_bindgen]
    pub fn compile_pdf_page_painted_rules(
        bytes: &[u8],
        page: usize,
    ) -> Result<PdfCompilation, JsValue> {
        let compiled =
            super::compile_page_painted_rules(bytes, page).map_err(|e| JsValue::from_str(&e))?;
        Ok(PdfCompilation {
            hwpx: compiled.hwpx,
            report: compiled.report.to_string(),
        })
    }

    #[wasm_bindgen]
    pub fn compile_pdf_page_tables(bytes: &[u8], page: usize) -> Result<PdfCompilation, JsValue> {
        let compiled =
            super::compile_page_tables(bytes, page).map_err(|e| JsValue::from_str(&e))?;
        Ok(PdfCompilation {
            hwpx: compiled.hwpx,
            report: compiled.report.to_string(),
        })
    }
}
