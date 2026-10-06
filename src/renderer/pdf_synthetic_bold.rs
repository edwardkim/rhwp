//! PDF-only faux bold for the actual regular face selected by usvg.

use super::synthetic_italic::{attribute_position, insertions};
use std::{borrow::Cow, collections::HashMap};

// Hancom 12.0.0.4605 emits stroke width / font size = 2.66 / 133 = 0.02
// for regular-only Dotum/Batang/Gulim/New Gulim (issue6936-bold-faces-2020.pdf).
const SYNTHETIC_BOLD_STROKE_EM: f32 = 0.02;

const SVG_NS: &str = "http://www.w3.org/2000/svg";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PdfBoldReport {
    pub synthesized_texts: usize,
    pub unsupported_texts: usize,
}

enum Decision {
    Unchanged,
    Stroke {
        color: usvg::Color,
        opacity: f32,
        width: f32,
    },
    Unsupported,
}

fn decide(text: &usvg::Text, db: &usvg::fontdb::Database) -> Decision {
    let weights: Vec<_> = text
        .chunks()
        .iter()
        .flat_map(|c| c.spans())
        .filter(|s| s.is_visible())
        .map(|s| s.font().weight())
        .collect();
    if !weights.iter().any(|w| *w >= 600) {
        return Decision::Unchanged;
    }
    let mut regular = false;
    let mut bold = false;
    for span in text.layouted().iter().filter(|s| s.visible) {
        for glyph in &span.positioned_glyphs {
            if glyph.text.chars().all(char::is_whitespace) {
                continue;
            }
            match db.face(glyph.font) {
                Some(face) if face.weight.0 >= 600 => bold = true,
                Some(_) => regular = true,
                None => return Decision::Unsupported,
            }
        }
    }
    if !regular {
        return Decision::Unchanged;
    }
    // rhwp positions uniform clusters separately. Arbitrary mixed SVG text must
    // never embolden a real bold face or a normal span along with a fallback.
    if bold || weights.iter().any(|w| *w < 600) {
        return Decision::Unsupported;
    }
    let mut paint = None;
    for span in text.layouted().iter().filter(|s| s.visible) {
        let Some(fill) = &span.fill else {
            return Decision::Unsupported;
        };
        let usvg::Paint::Color(color) = fill.paint() else {
            return Decision::Unsupported;
        };
        if span.stroke.is_some() {
            return Decision::Unchanged;
        }
        let current = (*color, fill.opacity().get(), span.font_size.get());
        if paint.is_some_and(|previous| previous != current) {
            return Decision::Unsupported;
        }
        paint = Some(current);
    }
    match paint {
        Some((color, opacity, size)) => Decision::Stroke {
            color,
            opacity,
            width: size * SYNTHETIC_BOLD_STROKE_EM,
        },
        None => Decision::Unchanged,
    }
}

fn collect(
    group: &usvg::Group,
    db: &usvg::fontdb::Database,
    result: &mut HashMap<String, Decision>,
) {
    for node in group.children() {
        match node {
            usvg::Node::Group(group) => collect(group, db, result),
            usvg::Node::Text(text) => {
                result.insert(text.id().to_owned(), decide(text, db));
            }
            _ => {}
        }
    }
}

/// Keep font identity, glyph advances and text unchanged; add paint only for
/// uniform clusters whose selected face has no bold weight. Browser SVG is untouched.
pub fn prepare_svg_with_synthetic_bold<'a>(
    svg: &'a str,
    options: &usvg::Options<'_>,
) -> Result<(Cow<'a, str>, PdfBoldReport), String> {
    let mut report = PdfBoldReport::default();
    if !svg.contains("font-weight") && !svg.contains("bold") {
        return Ok((Cow::Borrowed(svg), report));
    }
    let xml = roxmltree::Document::parse(svg).map_err(|e| e.to_string())?;
    let mut prefix = "rhwp-pdf-bold-".to_owned();
    while xml
        .descendants()
        .filter_map(|n| n.attribute("id"))
        .any(|id| id.starts_with(&prefix))
    {
        prefix.push('_');
    }
    let edits = xml
        .descendants()
        .filter(|n| n.has_tag_name((SVG_NS, "text")) && n.attribute("id").is_none())
        .enumerate()
        .map(|(i, n)| (attribute_position(svg, n), format!(" id=\"{prefix}{i}\"")))
        .collect();
    let tagged = insertions(svg, edits);
    let tree = usvg::Tree::from_str(&tagged, options).map_err(|e| e.to_string())?;
    let mut decisions = HashMap::new();
    collect(tree.root(), tree.fontdb(), &mut decisions);
    let xml = roxmltree::Document::parse(&tagged).map_err(|e| e.to_string())?;
    let mut edits = Vec::new();
    for node in xml
        .descendants()
        .filter(|n| n.has_tag_name((SVG_NS, "text")))
    {
        match node.attribute("id").and_then(|id| decisions.get(id)) {
            Some(Decision::Stroke {
                color,
                opacity,
                width,
            }) => {
                if node.descendants().skip(1).any(|child| {
                    child.attribute("stroke").is_some()
                        || child
                            .attribute("style")
                            .is_some_and(|style| style.to_ascii_lowercase().contains("stroke"))
                }) {
                    report.unsupported_texts += 1;
                    continue;
                }
                // Inline style overrides inherited stroke and preserves existing style.
                let style = format!(";stroke:rgb({},{},{});stroke-opacity:{opacity};stroke-width:{width};stroke-linejoin:round;paint-order:fill stroke",color.red,color.green,color.blue);
                if let Some(attr) = node.attribute_node("style") {
                    edits.push((attr.range_value().end, style));
                } else {
                    edits.push((
                        attribute_position(&tagged, node),
                        format!(" style=\"{}\"", style.trim_start_matches(';')),
                    ));
                }
                report.synthesized_texts += 1;
            }
            Some(Decision::Unsupported) => report.unsupported_texts += 1,
            _ => {}
        }
    }
    if edits.is_empty() {
        Ok((Cow::Borrowed(svg), report))
    } else {
        Ok((Cow::Owned(insertions(&tagged, edits)), report))
    }
}
