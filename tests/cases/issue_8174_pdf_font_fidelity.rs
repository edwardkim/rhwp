//! Independent CFF program and Hancom PDF rows; no absolute placement pins.
//! Full Native/fresh WASM TSV and direct review: task_m100_8174_t03_font_fidelity.md.
#![cfg(feature = "pdf-import")]

use rhwp::{
    parser::pdf::PdfImportOptions,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn text(node: &RenderNode) -> String {
    let mut result = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        result.push_str(&run.text);
    }
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}

fn rows(node: &RenderNode) -> Vec<String> {
    fn collect(node: &RenderNode, result: &mut Vec<(f64, String)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            let baseline = node.bbox.y + run.baseline;
            if let Some((_, value)) = result
                .last_mut()
                .filter(|(y, _)| (*y - baseline).abs() < 0.5)
            {
                value.push_str(&run.text);
            } else {
                result.push((baseline, run.text.clone()));
            }
        }
        for child in &node.children {
            collect(child, result);
        }
    }
    let mut result = Vec::new();
    collect(node, &mut result);
    result.into_iter().map(|(_, value)| value).collect()
}

#[test]
fn actual_cff_metrics_and_scalar_rows_survive_both_save_adapters() {
    let core = DocumentCore::from_pdf_bytes(
        include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf"),
        &PdfImportOptions {
            include_table_inference: true,
            ..Default::default()
        },
    )
    .unwrap();
    // Observation reports retain the PDF span's actual rectangle. A nominal
    // font ascent must not replace it during allocation/accounting refactors.
    let page = &core.pdf_metadata().unwrap().pages[0];
    let inference = &page.table_inference;
    let mut observed_lines = 0;
    for region in inference["textRecognition"]["bodyRegions"]
        .as_array()
        .unwrap()
    {
        for proposal in region["paragraphs"].as_array().unwrap() {
            for line in proposal["lines"].as_array().unwrap() {
                observed_lines += 1;
                let indices = line["spanIndices"].as_array().unwrap();
                assert_eq!(indices.len(), 1, "this source has one painted span per row");
                let source = &inference["sourceSpans"][indices[0].as_u64().unwrap() as usize]
                    ["observation"]["bbox"];
                let source_y = source["y"].as_f64().unwrap();
                let source_height = source["height"].as_f64().unwrap();
                let observed = &line["bbox"];
                assert!(
                    (observed["y"].as_f64().unwrap()
                        - (f64::from(page.media_box[3]) - source_y - source_height))
                        .abs()
                        < 0.0001,
                    "source user-space rectangle must survive top-left conversion"
                );
                assert_eq!(observed["height"], source["height"]);
            }
        }
    }
    assert_eq!(observed_lines, 3);
    fn check(core: &DocumentCore) {
        assert_eq!(core.page_count(), 1);
        assert_eq!(
            rows(&core.build_page_render_tree(0).unwrap().root),
            [
                "CFF: ABa office fi fl",
                "0123456789 + - = (A, B).",
                "Unicode: ÁÉÖ αβγ – —",
            ],
            "original scalars, row ownership and source order remain editable"
        );
        let trace: serde_json::Value = serde_json::from_str(
            &core
                .get_font_decision_trace_native(0, r#"{"maxCharacters":256}"#)
                .unwrap(),
        )
        .unwrap();
        let records = trace["records"].as_array().unwrap();
        for character in ['Á', 'É', 'Ö', 'α', 'β', 'γ', '–', '—'] {
            let record = records
                .iter()
                .find(|r| r["source"]["character"] == character.to_string())
                .unwrap();
            assert!(record["layoutMetric"]["metricEntry"].is_u64(), "{record}");
            assert!(!record["layoutMetric"]["widthSource"]
                .as_str()
                .unwrap()
                .starts_with("heuristic"));
            assert!(record["provenance"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| { p["evidenceAnchor"] == "data/font-metrics/nimbus-roman/source.json" }));
        }
    }
    check(&core);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        check(&DocumentCore::from_bytes(&bytes).unwrap());
    }
}

#[test]
fn unrepresentable_centered_cell_group_keeps_independent_source_rows() {
    const SOURCE: &[u8] = include_bytes!("../../pdf/issue5875/nested_table_text_caption-2020.pdf");
    fn check(core: &DocumentCore, page: u32, count: u32) {
        assert_eq!(core.page_count(), count);
        let tree = core.build_page_render_tree(page).unwrap();
        fn find<'a>(node: &'a RenderNode, found: &mut Vec<&'a RenderNode>) {
            if matches!(node.node_type, RenderNodeType::TableCell(_))
                && text(node)
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
                    == "손쳐내기식방호장치의무부하동작시험"
            {
                found.push(node);
            }
            for child in &node.children {
                find(child, found);
            }
        }
        let mut found = Vec::new();
        find(&tree.root, &mut found);
        assert_eq!(
            found.len(),
            1,
            "same cell owns the source text exactly once"
        );
        let rendered = rows(found[0])
            .into_iter()
            .map(|row| {
                row.chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            rendered,
            ["손쳐내기식", "방호장치의", "무부하", "동작시험"],
            "independent PDF row boundaries, not the importer's saved cuts"
        );
    }
    for (options, page, count) in [
        (PdfImportOptions::default(), 10, 12),
        (
            PdfImportOptions {
                source_pages: Some(vec![10]),
                ..Default::default()
            },
            0,
            1,
        ),
    ] {
        let core = DocumentCore::from_pdf_bytes(SOURCE, &options).unwrap();
        check(&core, page, count);
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            check(&DocumentCore::from_bytes(&bytes).unwrap(), page, count);
        }
    }
}
