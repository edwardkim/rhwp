//! Stage25 source origin and selected mixed-child physical frame.
//! Native/fresh WASM 전18쪽·완료7/10쪽과 독립 Print 근거: Stage25 보고서.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
fn load(ext: &str) -> HwpDocument {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/rowbreak-problem-pages.{ext}"));
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}
fn table(n: &RenderNode, section: usize, para: usize) -> Option<&RenderNode> {
    if matches!(&n.node_type,RenderNodeType::Table(t) if t.section_index==Some(section) && t.para_index==Some(para) && t.control_index==Some(0))
    {
        return Some(n);
    }
    n.children.iter().find_map(|c| table(c, section, para))
}
fn text(n: &RenderNode) -> String {
    let mut out = String::new();
    if let RenderNodeType::TextRun(r) = &n.node_type {
        out.push_str(&r.text)
    }
    for c in &n.children {
        out.push_str(&text(c))
    }
    out
}
fn line<'a>(n: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
    if matches!(n.node_type, RenderNodeType::TextLine(_)) && text(n).contains(needle) {
        return Some(n);
    }
    n.children.iter().find_map(|c| line(c, needle))
}
fn cell(n: &RenderNode, row: u16, col: u16) -> &RenderNode {
    n.children
        .iter()
        .find(|c| matches!(&c.node_type,RenderNodeType::TableCell(t)if t.row==row &&t.col==col))
        .unwrap()
}
fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}
#[test]
fn rejected_forward_origin_cannot_move_the_public_manual_table_away_from_csap_title() {
    for ext in ["hwpx", "hwp"] {
        let d = load(ext);
        let page = d.build_page_render_tree(9).unwrap();
        let title = line(&page.root, "서비스 보안인증제도").unwrap();
        let grid = table(&page.root, 1, 1).unwrap();
        assert!(text(grid).contains("공공데이터"));
        assert!(grid.bbox.y >= title.bbox.y + title.bbox.height);
        assert!(grid.bbox.y + grid.bbox.height <= page.root.bbox.height);
        #[cfg(feature = "native-skia")]
        {
            // Independent Print p10 stroked header border y723.409pt, MediaBox top841pt.
            // 96DPI top-left y156.788CSSpx ±2; keep owner/flow relations above.
            let bytes = d
                .render_page_png_native_with_profile_and_export_options(
                    9,
                    rhwp::paint::RenderProfile::Print,
                    &rhwp::document_core::queries::rendering::PngExportOptions {
                        dpi: Some(96.),
                        ..Default::default()
                    },
                )
                .unwrap();
            let png = image::load_from_memory(&bytes).unwrap().to_rgb8();
            let y = 156.788_f64;
            let columns = 90..700;
            let ink = columns
                .clone()
                .filter(|&x| {
                    ((y - 2.).ceil() as u32..=(y + 2.).floor() as u32).any(|row| {
                        let p = png.get_pixel(x, row);
                        p[0].max(p[1]).max(p[2]) <= 220
                    })
                })
                .count();
            assert!(
                ink * 100 >= columns.count() * 95,
                "independent Print table upper border missing: {ext}, ink={ink}"
            );
        }
    }
}
#[test]
fn continuous_child_cut_shares_parent_alignment_and_preserves_all_child_text() {
    let mut gaps = Vec::new();
    for ext in ["hwpx", "hwp"] {
        let d = load(ext);
        let page = d.build_page_render_tree(6).unwrap();
        let parent = table(&page.root, 0, 21).unwrap();
        let target = cell(parent, 3, 1);
        let first = line(target, "이용자는 클라우드컴퓨팅서비스").unwrap();
        assert!(
            first.bbox.y >= target.bbox.y
                && first.bbox.y + first.bbox.height <= target.bbox.y + target.bbox.height
        );
        gaps.push(first.bbox.y - target.bbox.y);
        let source = match &d.document().sections[0].paragraphs[21].controls[0] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("source table"),
        };
        let host = source
            .cells
            .iter()
            .find(|c| c.row == 3 && c.col == 1)
            .unwrap();
        let child = match &host.paragraphs[1].controls[0] {
            rhwp::model::control::Control::Table(t) => t,
            _ => panic!("source child"),
        };
        let expected = child.cells[0]
            .paragraphs
            .iter()
            .map(|p| p.text.as_str())
            .collect::<String>();
        let mut actual = String::new();
        for pi in [6, 7] {
            let p = d.build_page_render_tree(pi).unwrap();
            let t = table(&p.root, 0, 21).unwrap();
            let c = cell(t, 3, 1);
            let inner = c
                .children
                .iter()
                .find(|n| matches!(n.node_type, RenderNodeType::Table(_)))
                .unwrap();
            actual.push_str(&text(inner));
        }
        assert_eq!(
            compact(&actual),
            compact(&expected),
            "source child appears exactly once across p7/p8: {ext}"
        );
    }
    // Exact-input independent Prints preserve the same parent center space.
    assert!(
        (gaps[0] - gaps[1]).abs() <= 0.5,
        "source frame alignment differs by format: {gaps:?}"
    );
}
