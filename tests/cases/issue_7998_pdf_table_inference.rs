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
    // Independently authored PDF: four closed faces, A/B above C/D. Visual evidence:
    // task_m100_7998_stage1.md — Native 100%, fresh Docker WASM 99.53052%,
    // both 1 page at 96dpi, including direct border/content inspection.
    let input = include_bytes!("../../pdf/issue7998/solid-control.pdf");
    assert_cell_content_roundtrip(input);
}

#[test]
fn native_periodic_grid_owns_editable_cell_text_and_keeps_its_source_frame() {
    use rhwp::{renderer::render_tree::RenderNodeType, DocumentCore};
    // Independently authored 200pt page: 20..180pt outer frame, 100pt row/column
    // cuts, four letters. Every internal dash is 0.4pt with a 0.6pt empty gap.
    // Stage3/pass5 evidence: Native 99.94962%, fresh Docker WASM 98.88438%,
    // 96dpi, one page, borders/letters directly inspected. The former producer
    // returned zero actual tables; its captures and report are retained.
    let input = include_bytes!("../../pdf/issue7998/periodic-native-grid.pdf");
    assert_cell_content_roundtrip(input);
    let core = DocumentCore::from_pdf_bytes(input, &PdfImportOptions::default()).unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    fn find_table(
        n: &rhwp::renderer::render_tree::RenderNode,
    ) -> Option<&rhwp::renderer::render_tree::RenderNode> {
        if matches!(n.node_type, RenderNodeType::Table(_)) {
            Some(n)
        } else {
            n.children.iter().find_map(find_table)
        }
    }
    let table = find_table(&tree.root).expect("actual table in final output");
    let source_extent_px = 160.0 * 96.0 / 72.0;
    assert!(
        (table.bbox.height - source_extent_px).abs() <= 2.0,
        "background offsets must not grow the source's row frame: {:?}",
        table.bbox
    );
    assert!((table.bbox.width - source_extent_px).abs() <= 2.0);
    for cell in &table.children {
        if !matches!(cell.node_type, RenderNodeType::TableCell(_)) {
            continue;
        }
        for text in cell
            .children
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        {
            assert!(text.bbox.y >= cell.bbox.y);
            assert!(text.bbox.y + text.bbox.height <= cell.bbox.y + cell.bbox.height);
        }
    }
}

fn assert_cell_content_roundtrip(input: &[u8]) {
    use rhwp::{model::control::Control, DocumentCore};
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

// P1/P2 qualification: Native/fresh Docker WASM full-page minimum 99.20990%
// across the four new author-annotated PDFs; direct inspection and producer/input
// hashes: mydocs/working/task_m100_7998_stage3.md (P2 section).
// Both backends satisfy the visual prerequisite; assertions protect structure/ownership.
fn assert_pdf_cell_paragraph_case(input: &[u8], centered: bool) {
    use rhwp::{
        model::{control::Control, document::Document, style::Alignment, table::Table},
        DocumentCore,
    };
    fn table(doc: &Document) -> &Table {
        doc.sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .find_map(|c| match c {
                Control::Table(t) => Some(t.as_ref()),
                _ => None,
            })
            .expect("actual editable table")
    }
    fn texts(doc: &Document) -> Vec<Vec<String>> {
        let t = table(doc);
        [(0, 0), (0, 1), (1, 0), (1, 1)]
            .iter()
            .map(|&(row, col)| {
                t.cell_at(row, col)
                    .unwrap()
                    .paragraphs
                    .iter()
                    .map(|p| p.text.clone())
                    .collect()
            })
            .collect()
    }
    fn expected(centered: bool) -> Vec<Vec<String>> {
        vec![
            vec!["ALPHA BETA GAMMA".into(), "SECOND".into()],
            if centered {
                vec!["ALPHA BETA GAMMA".into()]
            } else {
                vec!["FIRST".into(), "SECOND".into()]
            },
            vec!["LEFT".into()],
            vec!["RIGHT".into()],
        ]
    }
    fn assert_source(doc: &Document, centered: bool) {
        let t = table(doc);
        assert_eq!((t.row_count, t.col_count), (2, 2));
        assert_eq!(
            texts(doc),
            expected(centered),
            "visual wraps must not become paragraph ends or hard breaks"
        );
        let first = &t.cell_at(0, 0).unwrap().paragraphs[0];
        assert_eq!(
            first.serializable_line_segs().len(),
            2,
            "one paragraph owns two displayed rows"
        );
        assert!(!first.text.contains('\n') && !first.text.contains('\t'));
        assert_eq!(first.char_offsets.len(), first.text.chars().count());
        if centered {
            assert!(
                doc.doc_info.para_shapes[first.para_shape_id as usize].indent < 0,
                "source hanging continuation stays editable indentation"
            );
            let second = &t.cell_at(0, 1).unwrap().paragraphs[0];
            assert!(matches!(
                doc.doc_info.para_shapes[second.para_shape_id as usize].alignment,
                Alignment::Center
            ));
        }
        let paint_owners = doc.sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .filter(|c| {
                matches!(
                    c,
                    Control::Table(_) | Control::Shape(_) | Control::Picture(_)
                )
            })
            .count();
        assert_eq!(
            paint_owners, 1,
            "cell text and rules cannot also remain in page-level overlays"
        );
    }
    let mut core = DocumentCore::from_pdf_bytes(input, &PdfImportOptions::default()).unwrap();
    assert_source(core.document(), centered);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_source(&rhwp::parser::parse_document(&bytes).unwrap(), centered);
    }
    let (pi, ci) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, p)| {
            p.controls
                .iter()
                .enumerate()
                .find_map(|(ci, c)| matches!(c, Control::Table(_)).then_some((pi, ci)))
        })
        .unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    fn find_table(
        n: &rhwp::renderer::render_tree::RenderNode,
    ) -> Option<&rhwp::renderer::render_tree::RenderNode> {
        if matches!(
            n.node_type,
            rhwp::renderer::render_tree::RenderNodeType::Table(_)
        ) {
            Some(n)
        } else {
            n.children.iter().find_map(find_table)
        }
    }
    for cell in &find_table(&tree.root).unwrap().children {
        use rhwp::renderer::render_tree::RenderNodeType;
        if !matches!(cell.node_type, RenderNodeType::TableCell(_)) {
            continue;
        }
        for line in cell
            .children
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        {
            assert!(line.bbox.x >= cell.bbox.x && line.bbox.y >= cell.bbox.y);
            assert!(
                line.bbox.x + line.bbox.width
                    <= cell.bbox.x + cell.bbox.width + f64::EPSILON * 1024.
            );
            assert!(
                line.bbox.y + line.bbox.height
                    <= cell.bbox.y + cell.bbox.height + f64::EPSILON * 1024.
            );
        }
    }
    core.insert_text_in_cell_native(0, pi, ci, 0, 0, 0, "IR:")
        .unwrap();
    let mut wanted = expected(centered);
    wanted[0][0].insert_str(0, "IR:");
    assert_eq!(texts(core.document()), wanted);
    core.delete_text_in_cell_native(0, pi, ci, 0, 0, 0, 3)
        .unwrap();
    assert_eq!(texts(core.document()), expected(centered));
    core.split_paragraph_in_cell_native(0, pi, ci, 0, 0, 6, None)
        .unwrap();
    let mut split = expected(centered);
    split[0] = vec!["ALPHA ".into(), "BETA GAMMA".into(), "SECOND".into()];
    assert_eq!(texts(core.document()), split);
    core.merge_paragraph_in_cell_native(0, pi, ci, 0, 1)
        .unwrap();
    assert_eq!(texts(core.document()), expected(centered));
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_eq!(
            texts(&rhwp::parser::parse_document(&bytes).unwrap()),
            expected(centered),
            "edited paragraph boundaries survive serialization"
        );
    }
}

#[test]
fn pdf_visual_wrap_maps_to_one_editable_paragraph_through_both_save_formats() {
    assert_pdf_cell_paragraph_case(
        include_bytes!("../../pdf/issue7998/multiline-cell.pdf"),
        false,
    );
}

#[test]
fn pdf_multiline_paragraphs_keep_hanging_indent_and_center_alignment() {
    assert_pdf_cell_paragraph_case(
        include_bytes!("../../pdf/issue7998/paragraph-origins.pdf"),
        true,
    );
}

// Exact source PDFs qualified on Native and fresh Docker WASM before activation.
#[test]
fn pdf_tj_and_split_operators_keep_the_same_editable_word_and_paragraph_boundaries() {
    for input in [
        include_bytes!("../../pdf/issue7998/tj-cell.pdf").as_slice(),
        include_bytes!("../../pdf/issue7998/split-cell.pdf").as_slice(),
    ] {
        assert_pdf_cell_paragraph_case(input, false);
    }
}

#[test]
fn published_helvetica_advances_and_trace_use_their_actual_afm_source() {
    use rhwp::renderer::font_metrics_data::find_metric;
    let inputs = [
        (
            false,
            false,
            include_str!("../../data/font-metrics/adobe-core14/Helvetica.afm"),
        ),
        (
            true,
            false,
            include_str!("../../data/font-metrics/adobe-core14/Helvetica-Bold.afm"),
        ),
        (
            false,
            true,
            include_str!("../../data/font-metrics/adobe-core14/Helvetica-Oblique.afm"),
        ),
        (
            true,
            true,
            include_str!("../../data/font-metrics/adobe-core14/Helvetica-BoldOblique.afm"),
        ),
    ];
    for (bold, italic, afm) in inputs {
        let metric = find_metric("Helvetica", bold, italic).unwrap().metric;
        assert_eq!(metric.em_size, 1000);
        for (ch, glyph) in [
            ('A', "A"),
            ('I', "I"),
            ('W', "W"),
            (' ', "space"),
            ('\'', "quotesingle"),
            ('`', "grave"),
        ] {
            let record = afm
                .lines()
                .find(|line| {
                    line.split(';')
                        .any(|part| part.trim() == format!("N {glyph}"))
                })
                .unwrap();
            let width: u16 = record
                .split(';')
                .find_map(|part| part.trim().strip_prefix("WX "))
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                metric.get_width(ch),
                Some(width),
                "{glyph} bold={bold} italic={italic}"
            );
        }
        assert_eq!(
            metric.get_width('한'),
            None,
            "ASCII evidence cannot invent CJK widths"
        );
    }
    assert!(find_metric("HelveticaCorp-Custom", false, false).is_none());
    let core = rhwp::DocumentCore::from_pdf_bytes(
        include_bytes!("../../pdf/issue7998/multiline-cell.pdf"),
        &PdfImportOptions::default(),
    )
    .unwrap();
    let trace: Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":32}"#)
            .unwrap(),
    )
    .unwrap();
    let record = trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["source"]["character"] == "A")
        .unwrap();
    assert_eq!(record["layoutMetric"]["matchKind"], "exact");
    let origin = record["provenance"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["sourceOwner"] == "adobe-core14-afm")
        .unwrap();
    assert_eq!(
        origin["evidenceAnchor"],
        "data/font-metrics/adobe-core14/Helvetica.afm"
    );
    assert_eq!(origin["reason"], "publishedAfmOutsideHistoricalLedger");
}

#[test]
fn pdf_glyph_source_references_remain_unique_when_word_spaces_are_inferred() {
    let imported = parse_pdf_with_options(
        include_bytes!("../../pdf/issue7998/split-cell.pdf"),
        &PdfImportOptions {
            include_table_inference: true,
            ..Default::default()
        },
    )
    .unwrap();
    let evidence = &imported.metadata.pages[0].table_inference;
    let mut owned = BTreeSet::new();
    let mut inferred = 0;
    for line in evidence["sourceTypography"]["lines"].as_array().unwrap() {
        assert_eq!(line["status"], "observed_origins_with_nominal_widths");
        for ch in line["characters"].as_array().unwrap() {
            let span = ch["span"].as_u64().unwrap() as usize;
            if let Some(index) = ch["sourceCharacter"].as_u64() {
                assert!(
                    owned.insert((span, index)),
                    "a source scalar cannot belong to two cells/lines"
                );
                let source = evidence["sourceSpans"][span]["observation"]["text"]
                    .as_str()
                    .unwrap();
                assert_eq!(
                    ch["character"].as_str().unwrap(),
                    source.chars().nth(index as usize).unwrap().to_string()
                );
                let prefix: String = source.chars().take(index as usize).collect();
                assert_eq!(
                    ch["sourceUtf16"].as_u64().unwrap() as usize,
                    prefix.encode_utf16().count()
                );
            } else {
                assert_eq!(ch["character"], " ");
                assert!(ch["sourceUtf16"].is_null());
                inferred += 1;
            }
        }
    }
    let original: usize = evidence["sourceSpans"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["observation"]["text"].as_str().unwrap().chars().count())
        .sum();
    assert_eq!(
        owned.len(),
        original,
        "all source scalars are preserved exactly once"
    );
    assert_eq!(
        inferred, 1,
        "the independently authored inter-word gap is not a tab"
    );
}

#[test]
fn pdf_public_page_five_cell_runs_fit_without_losing_the_final_glyphs() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    let core = rhwp::DocumentCore::from_pdf_bytes(
        include_bytes!("../../pdf/pr7704/6778-square-2020.pdf"),
        &PdfImportOptions::default(),
    )
    .unwrap();
    let tree = core.build_page_render_tree(4).unwrap();
    fn first_table(n: &RenderNode) -> Option<&RenderNode> {
        if matches!(n.node_type, RenderNodeType::Table(_)) {
            Some(n)
        } else {
            n.children.iter().find_map(first_table)
        }
    }
    fn runs<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(n.node_type, RenderNodeType::TextRun(_)) {
            out.push(n);
        }
        for child in &n.children {
            runs(child, out);
        }
    }
    let table = first_table(&tree.root).unwrap();
    let mut endings = String::new();
    for cell in &table.children {
        if !matches!(cell.node_type, RenderNodeType::TableCell(_)) {
            continue;
        }
        let mut text = Vec::new();
        runs(cell, &mut text);
        for node in text {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                endings.push_str(&run.text);
                let hu = rhwp::renderer::hwpunit_to_px(1, 96.);
                assert!(node.bbox.x >= cell.bbox.x - hu);
                assert!(
                    node.bbox.x + node.bbox.width <= cell.bbox.x + cell.bbox.width + hu,
                    "source glyphs must fit their owning cell, not be hidden by its clip"
                );
            }
        }
    }
    assert!(endings.contains("먼지목욕"));
    assert!(endings.contains("바닥"));
}
// PDF p7 row 1/column 3 has four independently visible source lines. Extended
// tabs own eight HWP stream units; saving must not move later glyphs to new lines.
#[test]
fn pdf_extended_tabs_keep_source_cell_lines_after_both_save_formats() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    use rhwp::{
        model::{control::Control, paragraph::Paragraph},
        DocumentCore,
    };
    fn paragraph_axis(p: &Paragraph) {
        // HWP extended controls and tabs each own 8 stream units. Floating
        // source paint can precede text; real content controls may remain
        // between glyphs. Account for every slot without pinning text at zero.
        let (mut pos, mut slots) = (0, 0);
        for (ch, offset) in p.text.chars().zip(&p.char_offsets) {
            let gap = offset.checked_sub(pos).expect("text stream order");
            assert_eq!(
                gap % 8,
                0,
                "only complete control slots separate source characters"
            );
            slots += gap / 8;
            pos = offset + if ch == '\t' { 8 } else { ch.len_utf16() as u32 };
        }
        let tail = p
            .char_count
            .checked_sub(pos + 1)
            .expect("paragraph terminator follows text");
        assert_eq!(tail % 8, 0);
        slots += tail / 8;
        assert_eq!(
            slots as usize,
            p.controls.len(),
            "every physical control slot is counted once"
        );
        assert_eq!(p.text.chars().count(), p.char_offsets.len());
    }
    fn cell(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::TableCell(c) if c.row == 1 && c.col == 3) {
            Some(node)
        } else {
            node.children.iter().find_map(cell)
        }
    }
    fn text(node: &RenderNode) -> String {
        if let RenderNodeType::TextRun(r) = &node.node_type {
            r.text.clone()
        } else {
            node.children.iter().map(text).collect()
        }
    }
    fn lines(node: &RenderNode, out: &mut Vec<String>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            out.push(text(node).chars().filter(|c| !c.is_whitespace()).collect());
        } else {
            for child in &node.children {
                lines(child, out);
            }
        }
    }
    fn verify(core: &mut DocumentCore) {
        let table = core.document().sections[6]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .find_map(|c| {
                if let Control::Table(t) = c {
                    Some(t)
                } else {
                    None
                }
            })
            .unwrap();
        assert!(table
            .cells
            .iter()
            .flat_map(|c| &c.paragraphs)
            .any(|p| p.text.contains('\t')));
        for p in table.cells.iter().flat_map(|c| &c.paragraphs) {
            paragraph_axis(p);
        }
        let tree = core.build_page_render_tree(6).unwrap();
        let mut actual = Vec::new();
        lines(cell(&tree.root).unwrap(), &mut actual);
        assert_eq!(
            actual,
            ["·스톨간충분한", "공간확보,공용", "공간과급수시설", "준비"]
        );
    }
    let mut core = DocumentCore::from_pdf_bytes(
        include_bytes!("../../pdf/pr7704/6778-square-2020.pdf"),
        &PdfImportOptions::default(),
    )
    .unwrap();
    verify(&mut core);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let mut saved = DocumentCore::from_bytes(&bytes).unwrap();
        verify(&mut saved);
    }
}

// Source PDFs, annotation, producer hashes and all-page Native/fresh Docker WASM
// qualification: task_m100_7998_stage3.md, P2 mixed-baseline slice.
fn assert_pdf_script_case(
    mut core: rhwp::DocumentCore,
    expected: &str,
    displacement_em: f64,
    size_ratio: f64,
) {
    use rhwp::{
        model::control::Control,
        renderer::render_tree::{RenderNode, RenderNodeType},
        DocumentCore,
    };
    fn first_cell(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::TableCell(c) if c.row == 0 && c.col == 0) {
            Some(node)
        } else {
            node.children.iter().find_map(first_cell)
        }
    }
    fn check(
        core: &DocumentCore,
        expected: &str,
        displacement_em: f64,
        size_ratio: f64,
        source_lines: bool,
    ) {
        let table = core.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .find_map(|c| {
                if let Control::Table(t) = c {
                    Some(t)
                } else {
                    None
                }
            })
            .expect("editable Table");
        let cell = table.cell_at(0, 0).unwrap();
        assert_eq!(
            cell.paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>(),
            [expected, "SECOND"]
        );
        assert_eq!(
            table
                .cell_at(0, 1)
                .unwrap()
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>(),
            ["FIRST", "SECOND"]
        );
        assert_eq!(table.cell_at(1, 0).unwrap().paragraphs[0].text, "LEFT");
        assert_eq!(table.cell_at(1, 1).unwrap().paragraphs[0].text, "RIGHT");
        let tree = core.build_page_render_tree(0).unwrap();
        let cell = first_cell(&tree.root).unwrap();
        let lines: Vec<_> = cell
            .children
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
            .collect();
        let first = lines.first().unwrap();
        let runs: Vec<_> = first
            .children
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) => Some((node, run)),
                _ => None,
            })
            .collect();
        let (ordinary_node, ordinary) = runs.iter().find(|(_, r)| r.text.contains('H')).unwrap();
        let (script_node, script) = runs
            .iter()
            .find(|(_, r)| r.text == "2" || r.text == "12")
            .expect("source ASCII digit, not a Unicode replacement");
        let (body_size, body_y) = ordinary.style.script_draw_metrics(
            ordinary.style.font_size,
            ordinary_node.bbox.y + ordinary.baseline,
        );
        let (small_size, small_y) = script
            .style
            .script_draw_metrics(script.style.font_size, script_node.bbox.y + script.baseline);
        assert!(
            (small_size / body_size - size_ratio).abs() < 0.001,
            "source glyph size ratio, no second script scaling"
        );
        assert!(
            ((small_y - body_y) / body_size - displacement_em).abs() < 0.01,
            "source PDF baseline displacement relative to 10pt, got {}",
            (small_y - body_y) / body_size
        );
        assert!(!script.style.superscript && !script.style.subscript);
        assert!(small_y > cell.bbox.y && small_y < cell.bbox.y + cell.bbox.height);
        let continuation = lines.iter().skip(1).find(|line| {
            line.children
                .iter()
                .any(|n| matches!(&n.node_type, RenderNodeType::TextRun(r) if r.text.contains('G')))
        });
        if let Some(continuation) = continuation {
            assert!(continuation.bbox.y > first.bbox.y);
        } else {
            assert!(
                !source_lines,
                "untouched source retains the authored second line"
            );
        }
    }
    check(&core, expected, displacement_em, size_ratio, true);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let saved = DocumentCore::from_bytes(&bytes).unwrap();
        check(&saved, expected, displacement_em, size_ratio, true);
    }
    let (pi, ci) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, p)| {
            p.controls
                .iter()
                .enumerate()
                .find_map(|(ci, c)| matches!(c, Control::Table(_)).then_some((pi, ci)))
        })
        .unwrap();
    core.insert_text_in_cell_native(0, pi, ci, 0, 0, 3, " IR")
        .unwrap();
    core.delete_text_in_cell_native(0, pi, ci, 0, 0, 3, 3)
        .unwrap();
    check(&core, expected, displacement_em, size_ratio, false);
    core.split_paragraph_in_cell_native(0, pi, ci, 0, 0, 3, None)
        .unwrap();
    core.merge_paragraph_in_cell_native(0, pi, ci, 0, 1)
        .unwrap();
    check(&core, expected, displacement_em, size_ratio, false);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let saved = DocumentCore::from_bytes(&bytes).unwrap();
        check(&saved, expected, displacement_em, size_ratio, false);
    }
}

#[test]
fn pdf_script_scalars_sizes_and_baselines_survive_real_cell_edits_and_both_saves() {
    for (input, expected, displacement_em, size_ratio) in [
        (
            include_bytes!("../../pdf/issue7998/solid-baseline.pdf").as_slice(),
            "H2O GAMMA",
            -0.4,
            0.6,
        ),
        (
            include_bytes!("../../pdf/issue7998/superscript.pdf").as_slice(),
            "H2O GAMMA",
            -0.4,
            0.6,
        ),
        (
            include_bytes!("../../pdf/issue7998/subscript.pdf").as_slice(),
            "H2O GAMMA",
            0.3,
            0.6,
        ),
        (
            include_bytes!("../../pdf/issue7998/leading-script.pdf").as_slice(),
            "2HO GAMMA",
            -0.4,
            0.6,
        ),
        (
            include_bytes!("../../pdf/issue7998/text-rise.pdf").as_slice(),
            "H2O GAMMA",
            -0.4,
            0.6,
        ),
        (
            include_bytes!("../../pdf/issue7998/mixed-sizes.pdf").as_slice(),
            "H2O GAMMA",
            0.0,
            0.8,
        ),
        (
            include_bytes!("../../pdf/issue7998/tm-rise.pdf").as_slice(),
            "H2O GAMMA",
            -0.2,
            1.0,
        ),
        (
            include_bytes!("../../pdf/issue7998/same-font-ts.pdf").as_slice(),
            "H2O GAMMA",
            -0.2,
            1.0,
        ),
        (
            include_bytes!("../../pdf/issue7998/multi-rise.pdf").as_slice(),
            "H12O GAMMA",
            -0.4,
            0.6,
        ),
    ] {
        assert_pdf_script_case(
            rhwp::DocumentCore::from_pdf_bytes(input, &PdfImportOptions::default()).unwrap(),
            expected,
            displacement_em,
            size_ratio,
        );
    }
}

#[test]
fn pdf_mixed_fonts_retain_editable_paragraph_boundaries() {
    assert_pdf_cell_paragraph_case(include_bytes!("../../pdf/issue7998/mixed-fonts.pdf"), false);
}

#[test]
fn pdf_small_overlapping_line_is_not_reclassified_as_an_adjacent_script() {
    let report = inference(
        include_bytes!("../../pdf/issue7998/separate-small-line.pdf"),
        true,
    );
    let cells = report["textRecognition"]["tables"][0]["cells"]
        .as_array()
        .unwrap();
    let cell = cells
        .iter()
        .find(|c| c["row"] == 0 && c["column"] == 0)
        .unwrap();
    let lines: Vec<_> = cell["recognition"]["paragraphs"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| p["lines"].as_array().unwrap())
        .map(|l| l["previewText"].as_str().unwrap())
        .collect();
    assert_eq!(
        lines,
        ["H", "small", "GAMMA", "SECOND"],
        "parallel ink is not an inline script; paragraph uncertainty may remain"
    );
}

// Same all-page qualification as the mixed-baseline fixtures; see stage3.
#[test]
fn pdf_baseline_is_preserved_when_horizontal_metrics_are_deferred() {
    use rhwp::{
        renderer::render_tree::{RenderNode, RenderNodeType},
        DocumentCore,
    };
    fn line(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::TextLine(_))
            && node.children.iter().any(
                |n| matches!(&n.node_type, RenderNodeType::TextRun(r) if r.text.contains("/m")),
            )
        {
            Some(node)
        } else {
            node.children.iter().find_map(line)
        }
    }
    fn check(core: &DocumentCore) {
        let tree = core.build_page_render_tree(5).unwrap();
        let line = line(&tree.root).expect("source unit m squared on page six");
        let runs: Vec<_> = line
            .children
            .iter()
            .filter_map(|n| {
                if let RenderNodeType::TextRun(r) = &n.node_type {
                    Some((n, r))
                } else {
                    None
                }
            })
            .collect();
        let (body_node, body) = runs.iter().find(|(_, r)| r.text.contains("/m")).unwrap();
        let (script_node, script) = runs
            .iter()
            .find(|(_, r)| r.text.trim_end_matches('\t') == "2")
            .expect("ASCII digit kept independently of font metrics");
        let (body_size, body_y) = body
            .style
            .script_draw_metrics(body.style.font_size, body_node.bbox.y + body.baseline);
        let (script_size, script_y) = script
            .style
            .script_draw_metrics(script.style.font_size, script_node.bbox.y + script.baseline);
        // Independent painted PDF glyphs: ordinary baseline 612.8892822266pt,
        // 2 at 617.68408203125pt; source sizes 11.0279483795 and 7.0722708702pt.
        assert!(
            (script_size / body_size - 7.0722708702 / 11.0279483795).abs() < 0.002,
            "source size retained"
        );
        let displacement = (script_y - body_y) / body_size;
        assert!(
            (displacement + (617.68408203125 - 612.8892822266) / 11.0279483795).abs() < 0.01,
            "horizontal uncertainty must not erase independent baseline, got {displacement}"
        );
        assert!(
            !script.style.superscript && !script.style.subscript,
            "source PDF size is already scaled"
        );
    }
    let core = DocumentCore::from_pdf_bytes(
        include_bytes!("../../pdf/pr7704/6778-square-2020.pdf"),
        &PdfImportOptions::default(),
    )
    .unwrap();
    check(&core);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        check(&DocumentCore::from_bytes(&bytes).unwrap());
    }
}

// Qualified source/save evidence and independent before failures: stage29 report.
fn whitespace_source(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/pdf/issue7998/horizontal-metrics/native"
        ))
        .join(format!("{name}.pdf")),
    )
    .unwrap()
}
fn whitespace_first_cell(core: &rhwp::DocumentCore) -> &rhwp::model::table::Cell {
    let rhwp::model::control::Control::Table(table) = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .find(|c| matches!(c, rhwp::model::control::Control::Table(_)))
        .unwrap()
    else {
        unreachable!()
    };
    assert_eq!(
        (table.row_count, table.col_count, table.cells.len()),
        (1, 2, 2)
    );
    &table.cells[0]
}
fn whitespace_paint(core: &rhwp::DocumentCore, gap: f64) {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn table(n: &RenderNode) -> Option<&RenderNode> {
        if matches!(n.node_type, RenderNodeType::Table(_)) {
            Some(n)
        } else {
            n.children.iter().find_map(table)
        }
    }
    fn run<'a>(n: &'a RenderNode, start: &str) -> Option<&'a RenderNode> {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            if r.text.starts_with(start) {
                return Some(n);
            }
        }
        n.children.iter().find_map(|n| run(n, start))
    }
    fn letters(n: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            out.extend(r.text.chars().filter(|c| !c.is_whitespace()));
        }
        for c in &n.children {
            letters(c, out);
        }
    }
    let tree = core.build_page_render_tree(0).unwrap();
    let t = table(&tree.root).unwrap();
    let cells: Vec<_> = t
        .children
        .iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::TableCell(_)))
        .collect();
    assert_eq!(cells.len(), 2);
    let a = run(cells[0], "A").unwrap();
    let b = run(cells[0], "B").unwrap();
    let c = run(cells[0], "C").unwrap();
    // ArialMT A=667/1000em, 12pt; independent TJ/Tw separator, not fitted fields.
    // Source page has 100% raw in both backends; retain inclusion/order too.
    assert!(((b.bbox.x - a.bbox.x) - (8.004 + gap) * 96. / 72.).abs() <= 2.0);
    assert!(b.bbox.x >= a.bbox.x + a.bbox.width - 0.5);
    assert!(c.bbox.y >= a.bbox.y + a.bbox.height - 0.5);
    for n in [a, b, c] {
        assert!(n.bbox.x >= cells[0].bbox.x && n.bbox.y >= cells[0].bbox.y);
        assert!(n.bbox.x + n.bbox.width <= cells[0].bbox.x + cells[0].bbox.width + 0.5);
        assert!(n.bbox.y + n.bbox.height <= cells[0].bbox.y + cells[0].bbox.height + 0.5);
    }
    let tail = run(&tree.root, "AFTER").unwrap();
    assert!(tail.bbox.y >= t.bbox.y + t.bbox.height);
    let mut painted = String::new();
    letters(&tree.root, &mut painted);
    assert_eq!(painted, "ABCDCONTROLCELLAFTERTABLE");
}
#[test]
fn joint_whitespace_metrics_preserve_source_cell_paragraph_and_both_saves() {
    use rhwp::DocumentCore;
    for (name, gap) in [
        ("wide-inferred", 15.),
        ("narrow-inferred", 2.88),
        ("wide-explicit", 15.),
    ] {
        let input = whitespace_source(name);
        let report = inference(&input, true);
        let first = report["sourceTypography"]["lines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| {
                l["context"] == serde_json::json!(["page", "table:0", "cell:0"])
                    && l["lineIndex"] == 0
            })
            .unwrap();
        assert_eq!(first["jointWhitespaceFits"], 1);
        assert_eq!(first["status"], "observed_origins_with_nominal_widths");
        let chars = first["characters"].as_array().unwrap();
        assert_eq!(
            chars
                .iter()
                .map(|c| c["character"].as_str().unwrap())
                .collect::<String>(),
            "A B"
        );
        assert!((chars[1]["advance"].as_f64().unwrap() - gap).abs() < 0.0001);
        assert_eq!(
            chars[1]["span"],
            if name == "wide-explicit" { 0 } else { 1 }
        );
        let mut core = DocumentCore::from_pdf_bytes(&input, &PdfImportOptions::default()).unwrap();
        let p = &whitespace_first_cell(&core).paragraphs;
        assert_eq!(core.document().doc_info.para_shapes[p[0].para_shape_id as usize].attr2 & 3,0,"editable PDF paragraphs use normal line wrapping; observed source lines remain in LineSeg");
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].text, "A B C D");
        assert_eq!(p[0].line_segs.len(), 2);
        assert!(p[0].controls.is_empty());
        for item in &p[0].char_shapes {
            let s = &core.document().doc_info.char_shapes[item.char_shape_id as usize];
            assert!(s.ratios.iter().all(|r| (50..=200).contains(r)));
            assert!(s.spacings.iter().all(|s| (-50..=50).contains(s)));
        }
        whitespace_paint(&core, gap);
        for bytes in [
            core.export_hwp_with_adapter_snapshot().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(
                whitespace_first_cell(&reopened).paragraphs[0].text,
                "A B C D"
            );
            whitespace_paint(&reopened, gap);
        }
        core.insert_text_in_cell_native(0, 0, 0, 0, 0, 1, "X")
            .unwrap();
        assert_eq!(whitespace_first_cell(&core).paragraphs[0].text, "AX B C D");
        core.delete_text_in_cell_native(0, 0, 0, 0, 0, 1, 1)
            .unwrap();
        assert_eq!(whitespace_first_cell(&core).paragraphs[0].text, "A B C D");
    }
}
#[test]
fn whitespace_outside_combined_fields_stays_unverified_and_ordinary_path_stays_ordinary() {
    for (name, joint, legacy) in [("ordinary", 0, false), ("too-wide", 0, true)] {
        let input = whitespace_source(name);
        let report = inference(&input, true);
        let first = report["sourceTypography"]["lines"]
            .as_array()
            .unwrap()
            .iter()
            .find(|l| {
                l["context"] == serde_json::json!(["page", "table:0", "cell:0"])
                    && l["lineIndex"] == 0
            })
            .unwrap();
        if legacy {
            assert_eq!(first["status"], "unverified_legacy_line_frame");
            assert_eq!(
                first["reason"],
                "source whitespace advance exceeds combined HWP character range"
            );
        } else {
            assert_eq!(first["jointWhitespaceFits"], joint);
            assert_eq!(first["status"], "observed_origins_with_nominal_widths");
        }
        let core =
            rhwp::DocumentCore::from_pdf_bytes(&input, &PdfImportOptions::default()).unwrap();
        assert_eq!(
            whitespace_first_cell(&core)
                .paragraphs
                .iter()
                .flat_map(|p| p.text.chars())
                .filter(|c| !c.is_whitespace())
                .collect::<String>(),
            "ABCD"
        );
    }
}
#[test]
fn plain_rectangular_strokes_use_native_cell_borders_without_overlay_objects() {
    use rhwp::DocumentCore;
    for (name, gap) in [
        ("wide-inferred", 15.),
        ("narrow-inferred", 2.88),
        ("wide-explicit", 15.),
    ] {
        let input = std::fs::read(
            std::path::Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/pdf/issue7998/horizontal-metrics/rectangle"
            ))
            .join(format!("{name}.pdf")),
        )
        .unwrap();
        let core = DocumentCore::from_pdf_bytes(&input, &PdfImportOptions::default()).unwrap();
        let cell = whitespace_first_cell(&core);
        assert_eq!(cell.paragraphs.len(), 1);
        let p = &cell.paragraphs[0];
        assert_eq!(p.text, "A B C D");
        assert_eq!(p.line_segs.len(), 2);
        assert!(
            p.controls.is_empty(),
            "a plain rectangular stroke is a native cell border, not a selectable overlay"
        );
        assert_eq!(inference(&input, true)["nativeBorderCells"], 2);
        assert_eq!(
            p.utf16_pos_to_char_idx(p.line_seg_text_start(1)),
            4,
            "the next source line still starts at C after all graphic slots"
        );
        whitespace_paint(&core, gap);
        for bytes in [
            core.export_hwp_with_adapter_snapshot().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            let p = &whitespace_first_cell(&reopened).paragraphs[0];
            assert_eq!(p.text, "A B C D");
            assert_eq!(p.line_segs.len(), 2);
            assert!(p.control_text_positions().iter().all(|at| *at == 0));
            assert_eq!(p.utf16_pos_to_char_idx(p.line_seg_text_start(1)), 4);
            whitespace_paint(&reopened, gap);
        }
    }
}

#[test]
fn regenerated_cell_bands_anchor_before_multiline_text_after_row_and_column_edits() {
    use rhwp::{model::control::Control, DocumentCore};
    let input = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/pdf/issue7998/horizontal-metrics/compound/wide-inferred.pdf"
    ))
    .unwrap();
    assert_eq!(
        inference(&input, true)["nativeBorderCells"],
        0,
        "this source exercises retained paint, not ordinary native borders"
    );
    for row in [true, false] {
        let mut core = DocumentCore::from_pdf_bytes(&input, &PdfImportOptions::default()).unwrap();
        let ci = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|c| matches!(c, Control::Table(_)))
            .unwrap();
        if row {
            core.insert_table_row_native(0, 0, ci, 0, false).unwrap();
        } else {
            core.insert_table_column_native(0, 0, ci, 0, false).unwrap();
        }
        let saved = [
            core.export_hwp_with_adapter_snapshot().unwrap(),
            core.export_hwpx_native().unwrap(),
        ];
        for current in
            std::iter::once(core).chain(saved.iter().map(|b| DocumentCore::from_bytes(b).unwrap()))
        {
            let Control::Table(table) = current.document().sections[0].paragraphs[0]
                .controls
                .iter()
                .find(|c| matches!(c, Control::Table(_)))
                .unwrap()
            else {
                panic!()
            };
            assert_eq!(
                (table.row_count, table.col_count),
                if row { (2, 2) } else { (1, 3) }
            );
            let paragraphs: Vec<_> = table.cells.iter().flat_map(|c| &c.paragraphs).collect();
            assert_eq!(
                paragraphs
                    .iter()
                    .map(|p| p.text.as_str())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>(),
                ["A B C D", "CONTROL CELL"]
            );
            for p in &paragraphs {
                assert!(!p.controls.is_empty());
                if !p.text.is_empty() {
                    assert!(
                        p.control_text_positions().iter().all(|at| *at == 0),
                        "new cell-local bands must not follow the last text line"
                    );
                } else {
                    assert_eq!(p.char_count, p.controls.len() as u32 * 8 + 1);
                }
            }
            let p = paragraphs.iter().find(|p| p.text == "A B C D").unwrap();
            assert_eq!(p.line_segs.len(), 2);
            assert_eq!(p.utf16_pos_to_char_idx(p.line_seg_text_start(1)), 4);
        }
    }
}

#[test]
fn cell_frame_anchors_survive_leading_text_after_both_reopen_formats() {
    use rhwp::{model::control::Control, DocumentCore};
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/pdf/issue7998/horizontal-metrics/alpha/wide-inferred.pdf"
    ))
    .unwrap();
    let original = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
    let files = [
        original.export_hwp_with_adapter_snapshot().unwrap(),
        original.export_hwpx_native().unwrap(),
    ];
    for mut c in
        std::iter::once(original).chain(files.iter().map(|b| DocumentCore::from_bytes(b).unwrap()))
    {
        let ci = c.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|c| matches!(c, Control::Table(_)))
            .unwrap();
        let p = &whitespace_first_cell(&c).paragraphs[0];
        assert!(p.controls.iter().all(|c| match c {
            Control::Picture(p) => p.common.cell_frame_paint,
            Control::Shape(s) => s.common().cell_frame_paint,
            _ => false,
        }));
        assert!(
            p.controls
                .iter()
                .any(|c| matches!(c,Control::Picture(p)if p.common.cell_border_paint.is_none())),
            "frame-only role must not invent border-edit authority"
        );
        c.insert_text_in_cell_native(
            0,
            0,
            ci,
            0,
            0,
            0,
            "FIRST SECOND THIRD FOURTH FIFTH SIXTH SEVENTH EIGHTH ",
        )
        .unwrap();
        let p = &whitespace_first_cell(&c).paragraphs[0];
        assert!(p.line_segs.len() > 2);
        assert!(p.control_text_positions().iter().all(|at| *at == 0));
        for data in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let reread = DocumentCore::from_bytes(&data).unwrap();
            let p = &whitespace_first_cell(&reread).paragraphs[0];
            assert!(p.control_text_positions().iter().all(|at| *at == 0));
            assert!(p.text.ends_with("A B C D"));
        }
    }
}

#[test]
fn empty_new_cell_frame_anchor_survives_typing_after_both_save_formats() {
    use rhwp::{model::control::Control, DocumentCore};
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/pdf/issue7998/horizontal-metrics/compound/wide-inferred.pdf"
    ))
    .unwrap();
    let original = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
    let files = [
        original.export_hwp_with_adapter_snapshot().unwrap(),
        original.export_hwpx_native().unwrap(),
    ];
    for mut c in
        std::iter::once(original).chain(files.iter().map(|b| DocumentCore::from_bytes(b).unwrap()))
    {
        let ci = c.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|c| matches!(c, Control::Table(_)))
            .unwrap();
        c.insert_table_row_native(0, 0, ci, 0, false).unwrap();
        let text = "FIRST SECOND THIRD FOURTH FIFTH SIXTH SEVENTH EIGHTH ";
        c.insert_text_in_cell_native(0, 0, ci, 0, 0, 0, text)
            .unwrap();
        for file in [
            c.export_hwp_with_adapter_snapshot().unwrap(),
            c.export_hwpx_native().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&file).unwrap();
            let Control::Table(t) = reopened.document().sections[0].paragraphs[0]
                .controls
                .iter()
                .find(|c| matches!(c, Control::Table(_)))
                .unwrap()
            else {
                unreachable!()
            };
            let p = &t.cells[0].paragraphs[0];
            assert_eq!(p.text, text);
            assert!(p.line_segs.len() > 1);
            assert!(p.control_text_positions().iter().all(|&at| at == 0));
            assert_eq!(
                t.cells
                    .iter()
                    .flat_map(|c| &c.paragraphs)
                    .map(|p| p.text.as_str())
                    .collect::<String>(),
                format!("{text}A B C DCONTROL CELL")
            );
        }
    }
}
#[test]
fn legacy_border_sidecars_restore_edit_roles_without_inventing_frame_anchors() {
    use rhwp::{
        model::{border_paint, control::Control, paragraph::Paragraph},
        DocumentCore,
    };
    fn clear(ps: &mut [Paragraph], borders: bool) {
        for p in ps {
            for c in &mut p.controls {
                match c {
                    Control::Table(t) => {
                        for cell in &mut t.cells {
                            clear(&mut cell.paragraphs, borders)
                        }
                    }
                    Control::Shape(s) => {
                        s.common_mut().cell_frame_paint = false;
                        if borders {
                            s.common_mut().cell_border_paint = None;
                            s.common_mut().outside_cell_border_paint = None;
                        }
                    }
                    Control::Picture(p) => {
                        p.common.cell_frame_paint = false;
                        if borders {
                            p.common.cell_border_paint = None;
                            p.common.outside_cell_border_paint = None;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    for (file, version) in [
        ("periodic-borders/thin-fill-occluder.pdf", 2),
        ("nested-dash-parent.pdf", 3),
        ("compound-borders/compound-solid.pdf", 4),
        ("retained-border-structure/nonuniform-axis.pdf", 5),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("pdf/issue7998")
            .join(file);
        let mut core = DocumentCore::from_pdf_bytes(
            &std::fs::read(path).unwrap(),
            &PdfImportOptions::default(),
        )
        .unwrap();
        if version == 5 {
            let ci = core.document().sections[0].paragraphs[0]
                .controls
                .iter()
                .position(|c| matches!(c, Control::Table(_)))
                .unwrap();
            core.insert_table_column_native(0, 0, ci, 0, false).unwrap();
        }
        let mut legacy = core.document().clone();
        for sec in &mut legacy.sections {
            clear(&mut sec.paragraphs, false)
        }
        let bytes = border_paint::encode(&legacy).unwrap().unwrap();
        assert_eq!(bytes[7], version, "{file}");
        let mut plain = legacy.clone();
        for sec in &mut plain.sections {
            clear(&mut sec.paragraphs, true)
        }
        assert!(border_paint::encode(&plain).unwrap().is_none());
        border_paint::decode(&mut plain, &bytes);
        assert_eq!(border_paint::encode(&plain).unwrap(), Some(bytes));
        // Re-encoding stays legacy: no frame role is guessed from coincident geometry.
    }
}

#[test]
fn plain_rectangle_properties_survive_real_axis_edits_and_both_native_saves() {
    use rhwp::{
        model::{control::Control, style::BorderLineType},
        DocumentCore,
    };
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/pdf/issue7998/horizontal-metrics/closed-large/wide-inferred.pdf"
    ))
    .unwrap();
    for mode in 0..3 {
        let mut core = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
        let ci = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|c| matches!(c, Control::Table(_)))
            .unwrap();
        match mode {
            1 => {
                core.insert_table_row_native(0, 0, ci, 0, false).unwrap();
            }
            2 => {
                core.insert_table_column_native(0, 0, ci, 0, false).unwrap();
            }
            _ => {}
        }
        let saved = [
            core.export_hwp_with_adapter_snapshot().unwrap(),
            core.export_hwpx_native().unwrap(),
        ];
        for c in
            std::iter::once(core).chain(saved.iter().map(|b| DocumentCore::from_bytes(b).unwrap()))
        {
            let Control::Table(t) = c.document().sections[0].paragraphs[0]
                .controls
                .iter()
                .find(|c| matches!(c, Control::Table(_)))
                .unwrap()
            else {
                unreachable!()
            };
            assert_eq!(
                (t.row_count, t.col_count),
                match mode {
                    1 => (2, 2),
                    2 => (1, 3),
                    _ => (1, 2),
                }
            );
            assert_eq!(
                t.cells
                    .iter()
                    .flat_map(|c| &c.paragraphs)
                    .map(|p| p.text.as_str())
                    .collect::<String>(),
                "A B C DCONTROL CELL"
            );
            for cell in &t.cells {
                assert!(
                    cell.paragraphs.iter().all(|p| p.controls.is_empty()),
                    "native cell borders must not create selectable overlay objects"
                );
                let fill = &c.document().doc_info.border_fills[cell.border_fill_id as usize - 1];
                for border in fill.borders.iter().take(4) {
                    assert_eq!(border.line_type, BorderLineType::Solid);
                    assert_eq!(border.color, 0);
                }
            }
        }
    }
}

#[test]
fn transparent_or_occluded_rectangles_do_not_invent_opaque_native_cell_borders() {
    use rhwp::{model::control::Control, DocumentCore};
    for name in ["alpha-frame", "occluded-frame"] {
        let path = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/pdf/issue7998/horizontal-metrics/"
        ))
        .join(if name == "alpha-frame" {
            "alpha"
        } else {
            "occluded"
        })
        .join("wide-inferred.pdf");
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(
            inference(&bytes, true)["nativeBorderCells"],
            0,
            "{name}: rectangle geometry alone does not prove full opaque ink"
        );
        let core = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions::default()).unwrap();
        let Control::Table(t) = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .find(|c| matches!(c, Control::Table(_)))
            .unwrap()
        else {
            unreachable!()
        };
        assert_eq!(
            t.cells
                .iter()
                .flat_map(|c| &c.paragraphs)
                .map(|p| p.text.as_str())
                .collect::<String>(),
            "A B C DCONTROL CELL"
        );
        assert!(
            t.cells
                .iter()
                .flat_map(|c| &c.paragraphs)
                .any(|p| !p.controls.is_empty()),
            "retain source paint rather than filling the missing/transparent edge"
        );
    }
}
