//! Extract stylesheet sources from a parsed [`Dom`] (HTML-3).
//!
//! Three sources of CSS in an HTML document:
//!
//! 1. `<style>` blocks — body served as-is.
//! 2. `<link rel="stylesheet" href="...">` — URL or path the caller
//!    fetches and feeds back as a `StylesheetSource::Inline`.
//! 3. Per-element `style="..."` attributes — applied as inline
//!    declarations during cascade (CSS-5
//!    `apply_inline_declarations`).
//!
//! HTML-3 surfaces all three so the API layer (Phase API) can decide
//! how to fetch link references — local file when `href` looks like
//! a path, network fetch when it's an HTTP(S) URL and the optional
//! `net` feature is enabled.

use super::dom::{Dom, NodeId, NodeKind};

/// One CSS source extracted from the document.
#[derive(Debug, Clone, PartialEq)]
pub enum StylesheetSource {
    /// `<style>...</style>` body, ready for `parse_stylesheet`.
    Inline(String),
    /// `<link rel="stylesheet" href="...">` — caller fetches.
    External {
        /// `href` value, exactly as it appeared in the attribute.
        href: String,
        /// `media="..."` value if present (for the @media-equivalent
        /// gating). Defaults to `"all"` when absent.
        media: String,
    },
}

/// Inline `style="..."` declaration source for a single element.
#[derive(Debug, Clone, PartialEq)]
pub struct InlineStyle {
    /// Element this inline style applies to.
    pub element: NodeId,
    /// Raw CSS source from the `style` attribute (declaration list,
    /// no rule braces).
    pub source: String,
}

/// All stylesheet sources extracted from `dom`, in document order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExtractedStyles {
    /// `<style>` and `<link>` sources, in document order.
    pub sheets: Vec<StylesheetSource>,
    /// Per-element inline-style declarations, in document order.
    pub inline_styles: Vec<InlineStyle>,
}

/// Walk `dom` collecting every CSS source.
pub fn extract_stylesheets(dom: &Dom) -> ExtractedStyles {
    let mut out = ExtractedStyles::default();
    for id in dom.iter_elements() {
        let node = dom.node(id);
        let NodeKind::Element { tag, attrs } = &node.kind else {
            continue;
        };
        match tag.as_str() {
            "style" => {
                // The tokenizer stashed the body as a single RawText
                // child of the synthetic <style> element.
                if let Some(&kid) = node.children.first() {
                    if let NodeKind::RawText { body, .. } = &dom.node(kid).kind {
                        out.sheets.push(StylesheetSource::Inline(body.clone()));
                    }
                }
            },
            "link" => {
                let rel = attrs
                    .iter()
                    .find(|(k, _)| k == "rel")
                    .map(|(_, v)| v.as_str())
                    .unwrap_or("");
                if !rel
                    .split_ascii_whitespace()
                    .any(|t| t.eq_ignore_ascii_case("stylesheet"))
                {
                    continue;
                }
                let Some(href) = attrs
                    .iter()
                    .find(|(k, _)| k == "href")
                    .map(|(_, v)| v.clone())
                else {
                    continue;
                };
                let media = attrs
                    .iter()
                    .find(|(k, _)| k == "media")
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "all".into());
                out.sheets.push(StylesheetSource::External { href, media });
            },
            _ => {},
        }
        // Inline style attribute on any element.
        if let Some((_, value)) = attrs.iter().find(|(k, _)| k == "style") {
            if !value.trim().is_empty() {
                out.inline_styles.push(InlineStyle {
                    element: id,
                    source: value.clone(),
                });
            }
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::html::dom::parse_document;

    #[test]
    fn extracts_inline_style_block() {
        let d = parse_document("<head><style>p { color: red; }</style></head>");
        let s = extract_stylesheets(&d);
        assert_eq!(s.sheets.len(), 1);
        match &s.sheets[0] {
            StylesheetSource::Inline(src) => assert!(src.contains("color: red")),
            other => panic!("expected inline, got {other:?}"),
        }
    }

    #[test]
    fn extracts_link_stylesheet() {
        let d = parse_document(r#"<link rel="stylesheet" href="reset.css">"#);
        let s = extract_stylesheets(&d);
        assert_eq!(s.sheets.len(), 1);
        match &s.sheets[0] {
            StylesheetSource::External { href, media } => {
                assert_eq!(href, "reset.css");
                assert_eq!(media, "all"); // default
            },
            _ => panic!(),
        }
    }

    #[test]
    fn link_rel_alternate_skipped() {
        // Only rel=stylesheet (case-insensitive, possibly multi-token)
        // counts.
        let d = parse_document(r#"<link rel="alternate" href="feed.xml">"#);
        let s = extract_stylesheets(&d);
        assert!(s.sheets.is_empty());
    }

    #[test]
    fn link_rel_multi_token_with_stylesheet_matches() {
        let d = parse_document(r#"<link rel="alternate stylesheet" href="dark.css">"#);
        let s = extract_stylesheets(&d);
        assert_eq!(s.sheets.len(), 1);
    }

    #[test]
    fn link_picks_up_media_attribute() {
        let d = parse_document(r#"<link rel="stylesheet" href="print.css" media="print">"#);
        let s = extract_stylesheets(&d);
        match &s.sheets[0] {
            StylesheetSource::External { media, .. } => assert_eq!(media, "print"),
            _ => panic!(),
        }
    }

    #[test]
    fn extracts_inline_style_attribute() {
        let d = parse_document(r#"<p style="color: red; font-size: 14px;">x</p>"#);
        let s = extract_stylesheets(&d);
        assert_eq!(s.inline_styles.len(), 1);
        let ist = &s.inline_styles[0];
        assert!(ist.source.contains("color: red"));
        assert!(ist.source.contains("font-size: 14px"));
        assert_eq!(ist.element, d.find_by_tag("p").unwrap());
    }

    #[test]
    fn multiple_inline_style_attrs() {
        let d = parse_document(r#"<div style="color: red"><p style="margin: 0">x</p></div>"#);
        let s = extract_stylesheets(&d);
        assert_eq!(s.inline_styles.len(), 2);
    }

    #[test]
    fn empty_inline_style_skipped() {
        let d = parse_document(r#"<p style="  ">x</p>"#);
        let s = extract_stylesheets(&d);
        assert!(s.inline_styles.is_empty());
    }

    #[test]
    fn document_order_preserved() {
        let d = parse_document(
            r#"<head><style>a {}</style><link rel="stylesheet" href="b.css"><style>c {}</style></head>"#,
        );
        let s = extract_stylesheets(&d);
        assert_eq!(s.sheets.len(), 3);
        assert!(matches!(&s.sheets[0], StylesheetSource::Inline(src) if src.contains("a")));
        assert!(matches!(&s.sheets[1], StylesheetSource::External { href, .. } if href == "b.css"));
        assert!(matches!(&s.sheets[2], StylesheetSource::Inline(src) if src.contains("c")));
    }

    #[test]
    fn empty_document() {
        let d = parse_document("");
        let s = extract_stylesheets(&d);
        assert!(s.sheets.is_empty());
        assert!(s.inline_styles.is_empty());
    }
}
