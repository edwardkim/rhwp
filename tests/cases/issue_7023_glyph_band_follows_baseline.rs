//! #7023 baseline diagnostic and #7048 renderer regression.
//! The original chemical-labeling fixture had a body/footer collision. The
//! saved picture/successor flow fix removes that collision; keep a negative
//! diagnostic guard here. Independent inflated-line baseline detection remains
//! covered by layout_anomaly_glyph_band's deterministic three-pair fixture.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::diagnostics::layout_anomaly::{scan_document, AnomalyOptions};
use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp";

#[test]
fn issue_7023_corrected_body_footer_does_not_overlap() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let report = scan_document(&core, &AnomalyOptions::default()).expect("scan");
    fn text(node: &rhwp::renderer::render_tree::RenderNode, out: &mut String) {
        if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(run.display_or_text());
        }
        for child in &node.children {
            text(child, out);
        }
    }
    let target_page = (0..core.page_count())
        .find(|&page| {
            let tree = core.build_page_render_tree(page).expect("page tree");
            let mut content = String::new();
            text(&tree.root, &mut content);
            content.contains("규제영향분석서 작성") && content.contains("XI")
        })
        .expect("최초 실물 증거의 본문과 로마자 꼬리말이 같은 쪽에 있어야 한다");
    let overlap = report
        .pages
        .iter()
        .find(|p| p.page == target_page)
        .is_some_and(|page| {
            page.text_overlap.iter().any(|o| {
                (o.path_a.contains("/Body/") && o.path_b.contains("/Footer"))
                    || (o.path_b.contains("/Body/") && o.path_a.contains("/Footer"))
            })
        });
    assert!(!overlap, "수정된 본문은 꼬리말과 겹치면 안 된다");
}
