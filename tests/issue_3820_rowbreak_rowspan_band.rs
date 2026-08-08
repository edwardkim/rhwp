//! Issue #3820 — RowBreak rowspan band continuation geometry.
//!
//! Hancom 2024 PDF for `76076_regulatory_analysis.hwp` renders the short
//! `주요내용` row at the bottom of p35, then retains only that row's blank
//! physical tail above p36's `11.영향평가 여부`.  Page count alone cannot detect
//! this: moving the whole row to p36 keeps the document at 82 pages while
//! changing both page images.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/76076_regulatory_analysis.hwp";

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("parse 76076 authority fixture")
}

fn text_y(node: &RenderNode, needle: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return Some(node.bbox.y);
        }
    }
    node.children.iter().find_map(|child| text_y(child, needle))
}

#[test]
fn issue_3820_rowbreak_rowspan_band_keeps_pdf_page_35_36_boundary() {
    let core = core();

    // Hancom PDF p35: `주요내용` is still painted at y≈736pt (≈979px CSS).
    let p35 = core.build_page_render_tree(34).expect("render HWP PDF p35");
    let summary_y = text_y(&p35.root, "주요내용")
        .expect("p35 must retain the `주요내용` content before the page boundary");
    assert!(
        (975.0..=984.0).contains(&summary_y),
        "p35 `주요내용` y={summary_y:.1}px; PDF-aligned row band must remain at the footer"
    );

    // Hancom PDF p36 has the blank tail of that row, but not its text; the
    // next visible row (`11.영향평가 여부`) begins only after that tail at y≈108px.
    let p36 = core.build_page_render_tree(35).expect("render HWP PDF p36");
    assert!(
        text_y(&p36.root, "주요내용").is_none(),
        "p36 must not repaint p35-owned `주요내용` text"
    );
    let impact_y = text_y(&p36.root, "영향평가").expect("p36 must resume at `11.영향평가 여부`");
    assert!(
        (103.0..=113.0).contains(&impact_y),
        "p36 `11.영향평가` y={impact_y:.1}px; blank rowspan tail was lost or overgrown"
    );
}
