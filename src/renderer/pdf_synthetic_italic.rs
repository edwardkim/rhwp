//! Synthetic oblique for upright faces selected by the compatibility PDF backend.

use std::collections::HashMap;

const SHEAR: f32 = 0.25;
const SVG_NS: &str = "http://www.w3.org/2000/svg";

/// Diagnostic counts, without document text or font names.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PdfItalicReport {
    pub synthesized_texts: usize,
    pub unsupported_texts: usize,
}

enum Decision {
    Unchanged,
    Shear(f32),
    Unsupported,
}

fn decide(text: &usvg::Text, db: &usvg::fontdb::Database) -> Decision {
    let styles: Vec<_> = text
        .chunks()
        .iter()
        .flat_map(|chunk| chunk.spans())
        .filter(|span| span.is_visible())
        .map(|span| span.font().style())
        .collect();
    if !styles.iter().any(|style| *style != usvg::FontStyle::Normal) {
        return Decision::Unchanged;
    }

    let mut upright = false;
    let mut slanted = false;
    let mut baseline: Option<f32> = None;
    let mut uniform_baseline = true;
    for span in text.layouted().iter().filter(|span| span.visible) {
        for glyph in &span.positioned_glyphs {
            if glyph.text.chars().all(char::is_whitespace) {
                continue;
            }
            match db.face(glyph.font).map(|face| face.style) {
                Some(usvg::fontdb::Style::Normal) => upright = true,
                Some(_) => slanted = true,
                None => return Decision::Unsupported,
            }
            let ts = glyph.transform();
            uniform_baseline &= ts.kx == 0.0
                && ts.ky == 0.0
                && ts.ty.is_finite()
                && baseline.is_none_or(|y| (y - ts.ty).abs() < 0.001);
            baseline.get_or_insert(ts.ty);
        }
    }
    if !upright {
        return Decision::Unchanged;
    }
    // A whole text transform is safe only when every visible glyph needs the same
    // synthesis. Never skew genuine italic faces or upright parts of mixed text.
    if slanted
        || styles.contains(&usvg::FontStyle::Normal)
        || !uniform_baseline
        || text.writing_mode() != usvg::WritingMode::LeftToRight
        || text
            .chunks()
            .iter()
            .any(|chunk| !matches!(chunk.text_flow(), usvg::TextFlow::Linear))
    {
        return Decision::Unsupported;
    }
    baseline.map_or(Decision::Unchanged, Decision::Shear)
}

fn collect_decisions(
    group: &usvg::Group,
    db: &usvg::fontdb::Database,
    decisions: &mut HashMap<String, Decision>,
) {
    for node in group.children() {
        match node {
            usvg::Node::Group(group) => collect_decisions(group, db, decisions),
            usvg::Node::Text(text) => {
                decisions.insert(text.id().to_owned(), decide(text, db));
            }
            _ => {}
        }
    }
}

fn insertions(svg: &str, mut edits: Vec<(usize, String)>) -> String {
    edits.sort_by_key(|edit| edit.0);
    let extra: usize = edits.iter().map(|edit| edit.1.len()).sum();
    let mut result = String::with_capacity(svg.len() + extra);
    let mut cursor = 0;
    for (at, value) in edits {
        result.push_str(&svg[cursor..at]);
        result.push_str(&value);
        cursor = at;
    }
    result.push_str(&svg[cursor..]);
    result
}

fn attribute_position(svg: &str, node: roxmltree::Node<'_, '_>) -> usize {
    // roxmltree has already validated the XML; only locate the end of its QName.
    let start = node.range().start + 1;
    start
        + svg[start..]
            .find(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/')
            .expect("validated XML opening tag")
}

/// Parse a PDF page while retaining searchable text and synthesizing missing italic faces.
///
/// rhwp emits separately positioned text clusters. Uniform horizontal clusters can
/// be sheared around their actual baseline without changing layout or advances.
/// Arbitrary mixed/path/vertical SVG text is left intact and reported as unsupported.
pub fn parse_svg_with_synthetic_italic(
    svg: &str,
    options: &usvg::Options<'_>,
) -> Result<(usvg::Tree, PdfItalicReport), String> {
    let parse = |source: &str| usvg::Tree::from_str(source, options).map_err(|e| e.to_string());
    let mut report = PdfItalicReport::default();
    // Ordinary rhwp pages keep their existing single parse, including unmodified IDs.
    if ![b"italic".as_slice(), b"oblique".as_slice()]
        .iter()
        .any(|word| {
            svg.as_bytes()
                .windows(word.len())
                .any(|s| s.eq_ignore_ascii_case(word))
        })
    {
        return Ok((parse(svg)?, report));
    }
    let xml = roxmltree::Document::parse(svg).map_err(|e| e.to_string())?;
    let mut prefix = "rhwp-pdf-italic-".to_owned();
    while xml
        .descendants()
        .filter_map(|node| node.attribute("id"))
        .any(|id| id.starts_with(&prefix))
    {
        prefix.push('_');
    }
    let edits = xml
        .descendants()
        .filter(|node| node.has_tag_name((SVG_NS, "text")) && node.attribute("id").is_none())
        .enumerate()
        .map(|(index, node)| {
            (
                attribute_position(svg, node),
                format!(" id=\"{prefix}{index}\""),
            )
        })
        .collect();
    let tagged = insertions(svg, edits);
    let tree = parse(&tagged)?;
    let mut decisions = HashMap::new();
    collect_decisions(tree.root(), tree.fontdb(), &mut decisions);

    let xml = roxmltree::Document::parse(&tagged).map_err(|e| e.to_string())?;
    let mut edits = Vec::new();
    for node in xml
        .descendants()
        .filter(|node| node.has_tag_name((SVG_NS, "text")))
    {
        let Some(decision) = node.attribute("id").and_then(|id| decisions.get(id)) else {
            continue;
        };
        match decision {
            Decision::Unchanged => {}
            Decision::Unsupported => report.unsupported_texts += 1,
            Decision::Shear(baseline) => {
                // CSS transforms can override a presentation attribute. Do not silently
                // claim synthesis in that case (rhwp's own text uses SVG attributes).
                if node
                    .attribute("style")
                    .is_some_and(|style| style.to_ascii_lowercase().contains("transform"))
                {
                    report.unsupported_texts += 1;
                    continue;
                }
                let matrix = format!("matrix(1 0 -{SHEAR} 1 {} 0)", baseline * SHEAR);
                if let Some(attr) = node.attribute_node("transform") {
                    edits.push((attr.range_value().end, format!(" {matrix}")));
                } else {
                    edits.push((
                        attribute_position(&tagged, node),
                        format!(" transform=\"{matrix}\""),
                    ));
                }
                report.synthesized_texts += 1;
            }
        }
    }
    if edits.is_empty() {
        Ok((tree, report))
    } else {
        Ok((parse(&insertions(&tagged, edits))?, report))
    }
}
