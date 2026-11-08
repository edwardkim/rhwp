//! Surface structured info about `<img>`, `<a>`, `<picture>`/`<source>`
//! from a parsed [`Dom`] (HTML-4).
//!
//! The API layer (Phase API) walks these to:
//!
//! - Resolve relative `src`/`href` against a base URL, fetch, embed
//!   bytes via [`crate::writer::PdfWriter`].
//! - Generate PDF link annotations for `<a href>` (URI link) and
//!   internal anchors (`href="#section"`).
//! - Pick the right `<source>` from a `<picture>` based on density
//!   descriptors (1x default for v0.3.35; mediaqueries-based selection
//!   is a v0.3.36 follow-up).

use super::dom::{Dom, NodeId, NodeKind};

/// One `<img>` (or the resolved `<picture>` source) reference.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageRef {
    /// Element id of the host `<img>` (or the `<picture>` if a source
    /// won the selection).
    pub element: NodeId,
    /// Selected `src` after srcset DPR resolution.
    pub src: String,
    /// Optional `alt` text (for accessibility + as a fallback when
    /// the image fails to load).
    pub alt: Option<String>,
    /// Explicit `width` attribute, if a numeric value.
    pub intrinsic_width: Option<f32>,
    /// Explicit `height` attribute, if a numeric value.
    pub intrinsic_height: Option<f32>,
}

/// One `<a href>` reference.
#[derive(Debug, Clone, PartialEq)]
pub struct Hyperlink {
    /// Element id of the `<a>`.
    pub element: NodeId,
    /// `href` value, exactly as given.
    pub href: String,
    /// True when href starts with `#` — internal anchor.
    pub is_internal_anchor: bool,
}

/// Aggregate output of [`extract_resources`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Resources {
    /// Image references in document order.
    pub images: Vec<ImageRef>,
    /// Hyperlinks in document order.
    pub links: Vec<Hyperlink>,
}

/// Walk `dom` collecting resource references.
pub fn extract_resources(dom: &Dom) -> Resources {
    let mut out = Resources::default();
    for id in dom.iter_elements() {
        let NodeKind::Element { tag, attrs } = &dom.node(id).kind else {
            continue;
        };
        match tag.as_str() {
            "img" => {
                // Skip <img>s that live inside a <picture> — those
                // are handled by the parent's pick_picture_source so
                // we don't double-emit.
                if is_inside_picture(dom, id) {
                    continue;
                }
                if let Some(img) = parse_img(dom, id, attrs) {
                    out.images.push(img);
                }
            },
            "picture" => {
                if let Some(img) = pick_picture_source(dom, id) {
                    out.images.push(img);
                }
            },
            "a" => {
                if let Some(link) = parse_anchor(id, attrs) {
                    out.links.push(link);
                }
            },
            _ => {},
        }
    }
    out
}

fn is_inside_picture(dom: &Dom, id: NodeId) -> bool {
    let Some(parent_id) = dom.node(id).parent else {
        return false;
    };
    matches!(
        &dom.node(parent_id).kind,
        NodeKind::Element { tag, .. } if tag == "picture"
    )
}

fn parse_img(_dom: &Dom, id: NodeId, attrs: &[(String, String)]) -> Option<ImageRef> {
    // srcset wins over src when present.
    let srcset = attrs
        .iter()
        .find(|(k, _)| k == "srcset")
        .map(|(_, v)| v.as_str());
    let src = if let Some(set) = srcset {
        select_srcset(set).unwrap_or_else(|| {
            attrs
                .iter()
                .find(|(k, _)| k == "src")
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        })
    } else {
        attrs
            .iter()
            .find(|(k, _)| k == "src")
            .map(|(_, v)| v.clone())?
    };
    if src.is_empty() {
        return None;
    }
    Some(ImageRef {
        element: id,
        src,
        alt: attrs
            .iter()
            .find(|(k, _)| k == "alt")
            .map(|(_, v)| v.clone()),
        intrinsic_width: attrs
            .iter()
            .find(|(k, _)| k == "width")
            .and_then(|(_, v)| v.parse::<f32>().ok()),
        intrinsic_height: attrs
            .iter()
            .find(|(k, _)| k == "height")
            .and_then(|(_, v)| v.parse::<f32>().ok()),
    })
}

fn pick_picture_source(dom: &Dom, picture_id: NodeId) -> Option<ImageRef> {
    // <picture> contains zero or more <source srcset="..." [media="..."]
    // [type="..."]> followed by a fallback <img>. Per HTML5 the first
    // matching <source> wins; if none match, the <img> fallback is
    // used. v0.3.35 ignores media= and type= (mediaqueries-based
    // selection is a 0.3.36 follow-up); we just pick the first
    // <source> that has a usable srcset, falling back to the last
    // <img>.
    let mut chosen_src: Option<String> = None;
    let mut fallback_img_id: Option<NodeId> = None;
    let mut fallback_img_attrs: Option<Vec<(String, String)>> = None;
    for &kid in &dom.node(picture_id).children {
        let NodeKind::Element { tag, attrs } = &dom.node(kid).kind else {
            continue;
        };
        match tag.as_str() {
            "source" => {
                if chosen_src.is_some() {
                    continue;
                }
                if let Some(set) = attrs
                    .iter()
                    .find(|(k, _)| k == "srcset")
                    .map(|(_, v)| v.as_str())
                {
                    chosen_src = select_srcset(set);
                }
            },
            "img" => {
                fallback_img_id = Some(kid);
                fallback_img_attrs = Some(attrs.clone());
            },
            _ => {},
        }
    }
    let attrs = fallback_img_attrs.as_deref().unwrap_or(&[]);
    let src = chosen_src.or_else(|| {
        attrs
            .iter()
            .find(|(k, _)| k == "src")
            .map(|(_, v)| v.clone())
    })?;
    if src.is_empty() {
        return None;
    }
    let host = fallback_img_id.unwrap_or(picture_id);
    Some(ImageRef {
        element: host,
        src,
        alt: attrs
            .iter()
            .find(|(k, _)| k == "alt")
            .map(|(_, v)| v.clone()),
        intrinsic_width: attrs
            .iter()
            .find(|(k, _)| k == "width")
            .and_then(|(_, v)| v.parse::<f32>().ok()),
        intrinsic_height: attrs
            .iter()
            .find(|(k, _)| k == "height")
            .and_then(|(_, v)| v.parse::<f32>().ok()),
    })
}

/// Pick a single URL from a `srcset` value. v0.3.35 prefers the
/// highest-DPR `<url> Nx` entry (1x default when no descriptor) and
/// ignores width descriptors (`<url> 800w`) — width-based selection
/// requires the `<sizes>` viewport-aware logic which lands in v0.3.36.
fn select_srcset(srcset: &str) -> Option<String> {
    let mut best: Option<(f32, String)> = None;
    for entry in srcset.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let mut parts = entry.split_ascii_whitespace();
        let url = parts.next()?.to_string();
        let descriptor = parts.next();
        let dpr = match descriptor {
            None => 1.0,
            Some(d) if d.ends_with('x') => d[..d.len() - 1].parse::<f32>().unwrap_or(1.0),
            Some(d) if d.ends_with('w') => continue, // width-based, skip
            _ => 1.0,
        };
        if best.as_ref().map(|(b, _)| dpr > *b).unwrap_or(true) {
            best = Some((dpr, url));
        }
    }
    best.map(|(_, url)| url)
}

fn parse_anchor(id: NodeId, attrs: &[(String, String)]) -> Option<Hyperlink> {
    let href = attrs
        .iter()
        .find(|(k, _)| k == "href")
        .map(|(_, v)| v.clone())?;
    if href.is_empty() {
        return None;
    }
    let is_internal_anchor = href.starts_with('#');
    Some(Hyperlink {
        element: id,
        href,
        is_internal_anchor,
    })
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::html::dom::parse_document;

    #[test]
    fn img_basic() {
        let d = parse_document(r#"<img src="cat.jpg" alt="A cat" width="320" height="240">"#);
        let r = extract_resources(&d);
        assert_eq!(r.images.len(), 1);
        let img = &r.images[0];
        assert_eq!(img.src, "cat.jpg");
        assert_eq!(img.alt.as_deref(), Some("A cat"));
        assert_eq!(img.intrinsic_width, Some(320.0));
        assert_eq!(img.intrinsic_height, Some(240.0));
    }

    #[test]
    fn img_no_src_skipped() {
        let d = parse_document(r#"<img alt="missing">"#);
        let r = extract_resources(&d);
        assert!(r.images.is_empty());
    }

    #[test]
    fn srcset_picks_highest_dpr() {
        let d = parse_document(
            r#"<img src="fallback.png" srcset="cat-1x.png 1x, cat-2x.png 2x, cat-3x.png 3x">"#,
        );
        let r = extract_resources(&d);
        assert_eq!(r.images[0].src, "cat-3x.png");
    }

    #[test]
    fn srcset_unitless_is_1x() {
        let d = parse_document(r#"<img srcset="a.png, b.png 2x">"#);
        let r = extract_resources(&d);
        assert_eq!(r.images[0].src, "b.png");
    }

    #[test]
    fn srcset_w_descriptors_skipped_for_v035() {
        let d = parse_document(r#"<img src="fallback.png" srcset="a.png 800w, b.png 1600w">"#);
        let r = extract_resources(&d);
        // No DPR entries → fall back to src.
        assert_eq!(r.images[0].src, "fallback.png");
    }

    #[test]
    fn picture_first_source_wins() {
        let d = parse_document(
            r#"<picture>
                 <source srcset="big.png 2x">
                 <img src="fallback.png">
               </picture>"#,
        );
        let r = extract_resources(&d);
        assert_eq!(r.images.len(), 1);
        assert_eq!(r.images[0].src, "big.png");
    }

    #[test]
    fn picture_falls_back_to_img_when_no_source_matches() {
        let d = parse_document(
            r#"<picture>
                 <source srcset="">
                 <img src="fallback.png">
               </picture>"#,
        );
        let r = extract_resources(&d);
        assert_eq!(r.images[0].src, "fallback.png");
    }

    #[test]
    fn anchor_href_external() {
        let d = parse_document(r#"<a href="https://example.com">x</a>"#);
        let r = extract_resources(&d);
        assert_eq!(r.links.len(), 1);
        assert_eq!(r.links[0].href, "https://example.com");
        assert!(!r.links[0].is_internal_anchor);
    }

    #[test]
    fn anchor_internal_anchor() {
        let d = parse_document(r##"<a href="#section-2">x</a>"##);
        let r = extract_resources(&d);
        assert!(r.links[0].is_internal_anchor);
    }

    #[test]
    fn anchor_no_href_skipped() {
        let d = parse_document(r#"<a>just text</a>"#);
        let r = extract_resources(&d);
        assert!(r.links.is_empty());
    }

    #[test]
    fn document_order_preserved() {
        let d = parse_document(r#"<img src="a.png"><a href="x"></a><img src="b.png">"#);
        let r = extract_resources(&d);
        assert_eq!(r.images[0].src, "a.png");
        assert_eq!(r.images[1].src, "b.png");
        assert_eq!(r.links[0].href, "x");
    }
}
