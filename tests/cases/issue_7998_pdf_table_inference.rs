//! Observation/API contracts on synthetic PDF streams; not rendering goldens.
#![cfg(feature = "pdf-import")]

use rhwp::parser::pdf::{parse_pdf_with_options, PdfImportOptions};
use serde_json::Value;
use std::collections::BTreeSet;

fn pdf(content: &str) -> Vec<u8> {
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Count 1 /Kids [3 0 R] >>".into(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>".into(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream",content.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),
    ];
    let mut bytes = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", i + 1).as_bytes());
    }
    let xref = bytes.len();
    bytes.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    bytes
}

fn inference(bytes: &[u8], full: bool) -> Value {
    let imported = parse_pdf_with_options(
        bytes,
        &PdfImportOptions {
            include_table_inference: full,
            ..PdfImportOptions::default()
        },
    )
    .unwrap();
    serde_json::to_value(&imported.metadata.pages[0].table_inference).unwrap()
}

#[test]
fn clip_without_paint_is_observed_without_inventing_border_ink() {
    // PDF user-space rectangle is [10,20]-[40,60]. In a 200pt page, top=140.
    let source = pdf("q 10 20 30 40 re W n 1 1 1 rg BT /F1 10 Tf 12 40 Td (WHITE) Tj ET Q");
    let report = inference(&source, true);
    assert_eq!(report["clipCount"], 1);
    assert_eq!(report["paintPathCount"], 0);
    assert_eq!(report["regions"][0]["kind"], "clip_not_paint");
    assert_eq!(
        report["regions"][0]["bounds"],
        serde_json::json!({"x":10.,"y":140.,"width":30.,"height":40.})
    );
    assert_eq!(
        report["regions"][0]["sourceBounds"],
        serde_json::json!({"x":10.,"y":20.,"width":30.,"height":40.})
    );
    let glyphs = report["glyphs"].as_array().unwrap();
    assert_eq!(
        glyphs
            .iter()
            .map(|g| g["text"].as_str().unwrap())
            .collect::<String>(),
        "WHITE"
    );
    assert!(glyphs
        .iter()
        .all(|g| g["visibility"] == "painted_run_occlusion_unknown"));
}

#[test]
fn source_glyphs_have_unique_t0_ownership_and_diagnostics_are_opt_in() {
    let source = pdf("BT /F1 10 Tf 20 100 Td (LEFT) Tj 80 0 Td (RIGHT) Tj ET");
    let summary = inference(&source, false);
    assert!(summary.get("glyphs").is_none());
    assert!(summary.get("paintPaths").is_none());
    let report = inference(&source, true);
    let glyphs = report["glyphs"].as_array().unwrap();
    let mut owned = BTreeSet::new();
    for block in report["initialBlocks"].as_array().unwrap() {
        for id in block["glyphIds"].as_array().unwrap() {
            assert!(
                owned.insert(id.as_u64().unwrap()),
                "a glyph cannot belong to two T0 blocks"
            );
        }
    }
    assert_eq!(owned, (0..glyphs.len() as u64).collect());
    let ids: BTreeSet<_> = glyphs.iter().map(|g| g["id"].as_str().unwrap()).collect();
    assert_eq!(ids.len(), glyphs.len());
    assert_eq!(
        report,
        inference(&source, true),
        "observation IDs cannot depend on allocator identity"
    );
    assert_eq!(summary["glyphCount"], report["glyphCount"]);
}

#[test]
fn local_clip_axes_remain_hypotheses_without_painted_separators() {
    let source =
        pdf("q 10 20 30 40 re W n Q q 40 20 30 40 re W n Q BT /F1 10 Tf 12 40 Td (A) Tj ET");
    let report = inference(&source, true);
    assert_eq!(report["paintPathCount"], 0);
    assert_eq!(report["semanticTables"], 0);
    let candidates = report["candidates"].as_array().unwrap();
    assert!(candidates
        .iter()
        .any(|c| c["generator"] == "clip_adjacency"));
    assert!(candidates
        .iter()
        .all(|c| c["status"] == "ambiguous" && c["authoredStructure"] == "unknown"));
}

#[test]
fn subpoint_strokes_remain_observations_and_periodic_coverage_is_not_paint() {
    // Four independent 0.25pt strokes, with 0.75pt gaps: the gaps contain no ink.
    let content="10 100 m 10.25 100 l S 11 100 m 11.25 100 l S 12 100 m 12.25 100 l S 13 100 m 13.25 100 l S";
    let report = inference(&pdf(content), true);
    let detection = &report["tableDetection"];
    let observed = detection["boundaryObservations"].as_array().unwrap();
    assert_eq!(
        observed.len(),
        4,
        "nonzero stroke length is independent of axis snapping tolerance"
    );
    let paint: f64 = observed
        .iter()
        .map(|s| s["to"].as_f64().unwrap() - s["from"].as_f64().unwrap())
        .sum();
    let logical = detection["logicalBoundaries"].as_array().unwrap();
    assert_eq!(logical.len(), 1);
    assert_eq!(logical[0]["paintKind"], "periodic_carrier");
    assert!(logical[0]["to"].as_f64().unwrap() - logical[0]["from"].as_f64().unwrap() > paint);
    assert_eq!(
        report["semanticTables"], 0,
        "one-dimensional carriers do not establish cells"
    );
    let duplicated = inference(&pdf(&format!("{content} {content}")), true);
    let other = &duplicated["tableDetection"]["logicalBoundaries"][0];
    for coordinate in ["at", "from", "to"] {
        assert_eq!(
            logical[0][coordinate], other[coordinate],
            "duplicate ink must not shift axes or carrier bounds"
        );
    }
}

#[test]
fn supported_solid_grid_keeps_unique_cell_content_through_both_save_formats() {
    use rhwp::{model::control::Control, DocumentCore};
    // Independently authored PDF: four closed faces, A/B above C/D. Visual evidence:
    // task_m100_7998_stage1.md — Native 100%, fresh Docker WASM 99.53052%,
    // both 1 page at 96dpi, including direct border/content inspection.
    let input = include_bytes!("../../pdf/issue7998/solid-control.pdf");
    let core = DocumentCore::from_pdf_bytes(input, &PdfImportOptions::default()).unwrap();
    assert_eq!(core.pdf_metadata().unwrap().pages[0].semantic_tables, 1);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let saved = rhwp::parser::parse_document(&bytes).unwrap();
        assert_eq!(saved.sections.len(), 1);
        let controls: Vec<_> = saved.sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            // HWP serialization also emits section/column definitions; they are not paint.
            .filter(|c| {
                matches!(
                    c,
                    Control::Table(_) | Control::Shape(_) | Control::Picture(_)
                )
            })
            .collect();
        assert!(
            saved.sections[0]
                .paragraphs
                .iter()
                .all(|p| !p.text.chars().any(|c| matches!(c, 'A' | 'B' | 'C' | 'D'))),
            "source characters must not also remain in the outer paragraph"
        );
        assert_eq!(
            controls.len(),
            1,
            "source text/rules must not also remain as duplicate floating controls"
        );
        let Control::Table(table) = controls[0] else {
            panic!("supported geometry must remain an editable table");
        };
        assert_eq!((table.row_count, table.col_count), (2, 2));
        let cells: std::collections::BTreeMap<_, _> = table
            .cells
            .iter()
            .map(|cell| {
                (
                    (cell.row, cell.col),
                    cell.paragraphs
                        .iter()
                        .map(|p| p.text.as_str())
                        .collect::<String>(),
                )
            })
            .collect();
        assert_eq!(
            cells,
            std::collections::BTreeMap::from([
                ((0, 0), "A".into()),
                ((0, 1), "B".into()),
                ((1, 0), "C".into()),
                ((1, 1), "D".into())
            ])
        );
        assert!(table
            .cells
            .iter()
            .all(|c| c.row_span == 1 && c.col_span == 1));
    }
}
