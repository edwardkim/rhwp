//! HWPX centered cell paragraphs must honor saved LINE_SEG vertical positions.
//!
//! The fixture has a 1x1 treat-as-char table. Its single cell is vertically
//! centered and starts with a TAC blue rounded rectangle title, followed by
//! ordinary paragraphs. The old layout applied saved paragraph vpos only for
//! top-aligned cells, so the intro paragraph was laid out cumulatively and
//! overlapped the title shape.

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

fn load_fixture() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwpx/hwpx-centered-cell-vpos-after-tac-shape.hwpx");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    HwpDocument::from_bytes(&bytes).expect("parse centered-cell-vpos HWPX fixture")
}

fn attr_f64(tag: &str, name: &str) -> Option<f64> {
    let pat = format!("{}=\"", name);
    let i = tag.find(&pat)? + pat.len();
    let j = tag[i..].find('"')? + i;
    tag[i..j].parse().ok()
}

fn translate_y(tag: &str) -> Option<f64> {
    let i = tag.find("translate(")? + "translate(".len();
    let rest = &tag[i..];
    let j = rest.find(')')?;
    let mut parts = rest[..j].split([',', ' ']).filter(|s| !s.is_empty());
    let _x = parts.next()?;
    parts.next()?.parse().ok()
}

fn iter_tags<'a>(svg: &'a str, open: &'a str) -> impl Iterator<Item = &'a str> {
    let mut rest = svg;
    std::iter::from_fn(move || {
        let p = rest.find(open)?;
        rest = &rest[p..];
        let end = rest.find('>').map(|e| e + 1).unwrap_or(rest.len());
        let tag = &rest[..end];
        rest = &rest[end..];
        Some(tag)
    })
}

fn text_ys_exact(svg: &str, needle: &str) -> Vec<f64> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(p) = rest.find("<text") {
        rest = &rest[p..];
        let close = rest.find("</text>").map(|e| e + 7).unwrap_or(rest.len());
        let node = &rest[..close];
        let Some(gt) = node.find('>') else {
            break;
        };
        let attrs = &node[..gt];
        let body = &node[gt + 1..node.find("</text>").unwrap_or(node.len())];
        if body.trim() == needle {
            if let Some(y) = attr_f64(attrs, "y").or_else(|| translate_y(attrs)) {
                out.push(y);
            }
        }
        rest = &rest[close..];
    }
    out
}

fn blue_title_rect_bottom(svg: &str) -> f64 {
    iter_tags(svg, "<rect")
        .filter(|tag| tag.contains("fill=\"#7a7cc4\"") || tag.contains("fill=\"#7A7CC4\""))
        .filter_map(|tag| Some(attr_f64(tag, "y")? + attr_f64(tag, "height")?))
        .next()
        .expect("blue title rectangle")
}

#[test]
fn fixture_is_reduced_to_one_page() {
    let doc = load_fixture();
    assert_eq!(
        doc.page_count(),
        1,
        "fixture should contain only the page that reproduces the centered-cell vpos overlap"
    );
}

#[test]
fn centered_cell_intro_paragraph_uses_saved_vpos_after_tac_title_shape() {
    let doc = load_fixture();
    let svg = doc.render_page_svg_native(0).expect("render fixture page");

    let title_bottom = blue_title_rect_bottom(&svg);
    let mun_ys = text_ys_exact(&svg, "문");
    assert!(
        mun_ys.len() >= 3,
        "expected header, title, and intro paragraph '문' glyphs, got {mun_ys:?}"
    );

    let intro_y = mun_ys[2];
    assert!(
        intro_y > title_bottom + 20.0,
        "intro paragraph overlaps the TAC title shape: intro_y={intro_y:.1}, title_bottom={title_bottom:.1}, all 문 ys={mun_ys:?}"
    );
}
