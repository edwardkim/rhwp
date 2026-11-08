//! HTML + CSS → PDF rendering pipeline.
//!
//! v0.3.35 (issue #248) ships a pure-Rust, MIT/Apache-only pipeline that
//! takes an HTML document plus optional CSS and produces a paginated
//! PDF. The architecture mirrors a small browser:
//!
//! ```text
//!   HTML ─► css::tokenizer ─┐                  ─► layout::box_tree ─►
//!   CSS  ─► css::tokenizer ─┴► css::parser ─►            cascade ─►
//!                              css::selectors ─►   layout::inline ─►
//!                              css::cascade   ─►   layout::flex_grid_glue ─►
//!                                                         ▼
//!                                                     paginate ─►
//!                                                         ▼
//!                                                       paint ─►
//!                                                         ▼
//!                                              pdf_oxide::writer
//! ```
//!
//! Sub-modules land incrementally per the plan (PLAN-1, PRE-FLIGHT-AUDIT):
//!
//! - [`css`] — tokenizer (CSS-1), parser (CSS-2), selectors (CSS-3..4),
//!   cascade (CSS-5), calc/var (CSS-6..7), property parsing (CSS-8),
//!   at-rules (CSS-9), counters/pseudo-elements (CSS-10).
//! - `html` — tokenizer + DOM (HTML-1..4).
//! - `layout` — box tree, inline formatting, Taffy glue, floats, margin
//!   collapsing, multi-column, tables (LAYOUT-1..7).
//! - `paginate` — fragmentation, `@page`, page-break-* (PAGINATE-1..6).
//! - `paint` — display list → ContentStreamBuilder (PAINT-1..7).

pub mod css;
pub mod html;
pub mod layout;
pub mod paginate;
pub mod paint;
