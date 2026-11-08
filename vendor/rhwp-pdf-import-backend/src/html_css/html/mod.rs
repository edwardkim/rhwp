//! HTML parser + DOM for the v0.3.35 HTML+CSS→PDF pipeline.
//!
//! Phase HTML's job is small but specific: take a UTF-8 HTML source
//! string, build a flat arena-allocated DOM, and expose it via a
//! handle type that implements [`super::css::Element`] so the cascade
//! (CSS-5) can match selectors against real elements.
//!
//! This is **not** a WHATWG-conformance HTML parser. It targets
//! well-formed documents and the common malformed patterns that show
//! up in HTML→PDF inputs (missing close tags on `<p>`/`<li>`, unquoted
//! attributes, inline `<style>` blocks). For HTML scraped from the
//! wild that needs perfect spec parsing, downstream callers can preprocess
//! with html5ever (MPL — out of our deny list) or any other parser
//! before feeding the resulting HTML string here.

pub mod dom;
pub mod resources;
pub mod stylesheets;
pub mod tokenizer;

pub use dom::{parse_document, Dom, DomElement, NodeId, NodeKind};
pub use resources::{extract_resources, Hyperlink, ImageRef, Resources};
pub use stylesheets::{extract_stylesheets, ExtractedStyles, InlineStyle, StylesheetSource};
pub use tokenizer::{tokenize, HtmlToken};
