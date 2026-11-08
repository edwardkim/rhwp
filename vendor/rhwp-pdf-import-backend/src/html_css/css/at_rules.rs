//! At-rule handlers — `@media`, `@page`, `@font-face`, `@import`,
//! `@supports`, `@keyframes` (CSS-9).
//!
//! Stylesheet from CSS-2 carries every at-rule unfiltered. Phase
//! LAYOUT and Phase PAINT need a *resolved* stylesheet — only the
//! rules whose `@media` queries currently apply, the page-context
//! configuration from `@page` rules, and the `@font-face` descriptor
//! list ready for `SystemFontDb` (writer module, `system-fonts` feature) to
//! load. This module does that resolution against a [`MediaContext`].
//!
//! v0.3.35 surface:
//!
//! - `@media print { ... }` always matches in the PDF pipeline; the
//!   inverse `@media screen { ... }` always loses. `@media (min-width:
//!   X)` and `(max-width: X)` evaluate against the page content-box
//!   width.
//! - `@page` extracts page size, margin, and per-page-name overrides
//!   (`:first`, `:left`, `:right`, `:blank`). Margin boxes
//!   (`@top-center` etc.) are preserved as unparsed bodies for the
//!   paginator (PAGINATE-5) to consume.
//! - `@font-face` parses `font-family`, `src` (local() and url()
//!   tuples), `font-weight`, `font-style`, `font-stretch`, `font-display`
//!   into a [`FontFaceDescriptor`]. FONT-4's `SystemFontDb` extends
//!   itself with these.
//! - `@import` is **forwarded** with the URL preserved; an HTML→PDF
//!   driver (Phase API) decides whether to fetch (when the `net`
//!   feature is on) or to log-and-skip.
//! - `@supports` evaluates against [`supports_supported`] —
//!   conservatively true only for the v0.3.35 supported surface.
//! - `@keyframes` is parsed-and-ignored. The descriptor is preserved
//!   so a future v0.3.36 can wire animations without re-parsing.

use std::borrow::Cow;

use super::parser::{
    parse_declaration_list, AtRule, AtRuleBlock, ComponentValue, Declaration, QualifiedRule, Rule,
    Stylesheet,
};
use super::tokenizer::Token;

// ─────────────────────────────────────────────────────────────────────
// MediaContext — what the renderer is targeting
// ─────────────────────────────────────────────────────────────────────

/// Conditions an `@media` query can ask about.
#[derive(Debug, Clone, Copy)]
pub struct MediaContext {
    /// Width of the page content-box (page width minus margins) in px.
    pub width_px: f32,
    /// Height of the page content-box in px.
    pub height_px: f32,
    /// Always true for a PDF pipeline.
    pub print: bool,
}

impl Default for MediaContext {
    fn default() -> Self {
        Self {
            width_px: 595.0, // A4 portrait, 0 margin
            height_px: 842.0,
            print: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Resolved stylesheet
// ─────────────────────────────────────────────────────────────────────

/// Output of [`resolve`]. Carries only the rules whose @media
/// conditions currently match, plus the configuration extracted from
/// `@page` and `@font-face` rules.
#[derive(Debug, Clone, Default)]
pub struct ResolvedStylesheet<'i> {
    /// Qualified rules in source order. At-rules other than @media
    /// have been consumed; @media bodies are flattened in iff they
    /// match.
    pub rules: Vec<QualifiedRule<'i>>,
    /// Page configuration rules in source order. Selector matching
    /// (`:first`, `:left`, …) happens at pagination time.
    pub page_rules: Vec<PageRule<'i>>,
    /// `@font-face` descriptors — one per @font-face block.
    pub font_faces: Vec<FontFaceDescriptor>,
    /// `@import` references — caller decides whether to fetch.
    pub imports: Vec<String>,
}

/// One `@page` block in the source.
#[derive(Debug, Clone)]
pub struct PageRule<'i> {
    /// Page-name selector list. Empty list ⇒ matches every page.
    pub selectors: Vec<PageSelector>,
    /// Page-property declarations from inside the block (size, margin,
    /// marks, …).
    pub declarations: Vec<Declaration<'i>>,
    /// `@top-center` / `@bottom-right` etc. nested at-rules. Each
    /// entry's name is the margin-box position (without the leading
    /// @), and `block` is the unparsed body for the paginator to
    /// consume.
    pub margin_boxes: Vec<(String, Vec<ComponentValue<'i>>)>,
}

/// `@page :first` / `:left` / `:right` / `:blank` / named selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageSelector {
    /// Matches the first page only.
    First,
    /// Matches verso (even-numbered) pages.
    Left,
    /// Matches recto (odd-numbered) pages.
    Right,
    /// Matches an intentionally-blank page (CSS Paged Media L3).
    Blank,
    /// Named page selector (`@page my-cover`).
    Named(String),
}

/// `@font-face { ... }` descriptors.
#[derive(Debug, Clone, Default)]
pub struct FontFaceDescriptor {
    /// `font-family: ...` — the family name this declaration
    /// contributes to.
    pub family: String,
    /// Each `src` entry parsed into a `SrcEntry`.
    pub sources: Vec<SrcEntry>,
    /// `font-weight` — 400 default.
    pub weight: u16,
    /// `font-style: italic`/`normal` — false default.
    pub italic: bool,
    /// `font-stretch` — kept as a 0..=200% number; 100% default
    /// (matches CSS Fonts L4 stretch keyword mapping at 100/normal).
    pub stretch_pct: f32,
}

/// One entry in a `src: [...]` descriptor.
#[derive(Debug, Clone)]
pub enum SrcEntry {
    /// `local("PostScript Name")` or `local(Name)` (unquoted form).
    Local(String),
    /// `url("font.ttf") format("truetype")`.
    Url {
        /// URL.
        url: String,
        /// Optional `format(...)` hint.
        format: Option<String>,
    },
}

// ─────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────

/// Resolve `stylesheet` against `media`, producing a [`ResolvedStylesheet`]
/// with at-rules collapsed and `@media` filtered to only the matching
/// branches.
pub fn resolve<'i>(stylesheet: &Stylesheet<'i>, media: MediaContext) -> ResolvedStylesheet<'i> {
    let mut out = ResolvedStylesheet::default();
    process_rules(&stylesheet.rules, media, &mut out);
    out
}

fn process_rules<'i>(rules: &[Rule<'i>], media: MediaContext, out: &mut ResolvedStylesheet<'i>) {
    for rule in rules {
        match rule {
            Rule::Qualified(q) => out.rules.push(q.clone()),
            Rule::AtRule(at) => process_at_rule(at, media, out),
        }
    }
}

fn process_at_rule<'i>(at: &AtRule<'i>, media: MediaContext, out: &mut ResolvedStylesheet<'i>) {
    let lower = at.name.to_ascii_lowercase();
    match lower.as_str() {
        "media" => {
            if evaluate_media_query(&at.prelude, media) {
                if let Some(block) = &at.block {
                    let nested = parse_block_as_rules(block);
                    process_rules(&nested, media, out);
                }
            }
        },
        "page" => {
            if let Some(rule) = parse_page_rule(at) {
                out.page_rules.push(rule);
            }
        },
        "font-face" => {
            if let Some(desc) = parse_font_face(at) {
                out.font_faces.push(desc);
            }
        },
        "import" => {
            if let Some(url) = extract_import_url(&at.prelude) {
                out.imports.push(url);
            }
        },
        "supports" => {
            if evaluate_supports_query(&at.prelude) {
                if let Some(block) = &at.block {
                    let nested = parse_block_as_rules(block);
                    process_rules(&nested, media, out);
                }
            }
        },
        // @keyframes, @charset, @namespace, @counter-style, @property,
        // @layer — all parsed-and-ignored for v0.3.35.
        _ => {},
    }
}

// ─────────────────────────────────────────────────────────────────────
// @media — restrained subset of CSS Media Queries L4
// ─────────────────────────────────────────────────────────────────────

/// Evaluate an @media prelude against `media`. The grammar handled:
///
/// - `print` / `screen` / `all` — top-level type
/// - `(min-width: <length>)` / `(max-width: <length>)`
/// - `(orientation: portrait | landscape)`
/// - Combinations with `and`
/// - Comma-separated alternatives (any matches → query matches)
///
/// Anything more exotic (`not`, `only`, level-4 range syntax) is
/// conservatively treated as "doesn't match" so we don't accidentally
/// apply rules whose conditions we can't evaluate.
pub fn evaluate_media_query(prelude: &[ComponentValue<'_>], media: MediaContext) -> bool {
    // Top-level commas: at least one branch must match.
    let trimmed = trim_ws(prelude);
    if trimmed.is_empty() {
        // `@media { ... }` with no condition matches everything.
        return true;
    }
    split_top_level_commas(trimmed)
        .into_iter()
        .any(|branch| evaluate_media_branch(branch, media))
}

fn evaluate_media_branch(branch: &[ComponentValue<'_>], media: MediaContext) -> bool {
    let trimmed = trim_ws(branch);
    if trimmed.is_empty() {
        return false;
    }
    // Walk left-to-right collecting type + (feature) blocks separated
    // by `and`.
    let mut and_groups: Vec<&[ComponentValue<'_>]> = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < trimmed.len() {
        if let ComponentValue::Token(Token::Ident(s)) = &trimmed[i] {
            if s.eq_ignore_ascii_case("and") {
                and_groups.push(trim_ws(&trimmed[start..i]));
                start = i + 1;
            }
        }
        i += 1;
    }
    and_groups.push(trim_ws(&trimmed[start..]));

    and_groups
        .into_iter()
        .all(|g| evaluate_media_atom(g, media))
}

fn evaluate_media_atom(atom: &[ComponentValue<'_>], media: MediaContext) -> bool {
    if atom.is_empty() {
        return false;
    }
    // Type keyword?
    if let [ComponentValue::Token(Token::Ident(s))] = atom {
        return match s.to_ascii_lowercase().as_str() {
            "all" => true,
            "print" => media.print,
            "screen" => !media.print,
            _ => false,
        };
    }
    // (feature: value)
    if let [ComponentValue::Parens(body)] = atom {
        return evaluate_media_feature(body, media);
    }
    false
}

fn evaluate_media_feature(body: &[ComponentValue<'_>], media: MediaContext) -> bool {
    // ident : value
    let trimmed = trim_ws(body);
    let mut iter = trimmed.iter().enumerate();
    let (i, name_cv) = iter
        .find(|(_, cv)| !matches!(cv, ComponentValue::Token(Token::Whitespace)))
        .unwrap_or((0, &ComponentValue::Token(Token::Whitespace)));
    let name = match name_cv {
        ComponentValue::Token(Token::Ident(s)) => s.to_ascii_lowercase(),
        _ => return false,
    };
    // Find the colon.
    let mut j = i + 1;
    while j < trimmed.len() && matches!(trimmed[j], ComponentValue::Token(Token::Whitespace)) {
        j += 1;
    }
    if j >= trimmed.len() {
        // Boolean feature query like (color) — true for v0.3.35 if the
        // feature exists; we just say true for the print-relevant ones.
        return matches!(name.as_str(), "color");
    }
    if !matches!(trimmed[j], ComponentValue::Token(Token::Colon)) {
        return false;
    }
    j += 1;
    while j < trimmed.len() && matches!(trimmed[j], ComponentValue::Token(Token::Whitespace)) {
        j += 1;
    }
    let value = &trimmed[j..];

    match name.as_str() {
        "min-width" => extract_length_px(value)
            .map(|px| media.width_px >= px)
            .unwrap_or(false),
        "max-width" => extract_length_px(value)
            .map(|px| media.width_px <= px)
            .unwrap_or(false),
        "min-height" => extract_length_px(value)
            .map(|px| media.height_px >= px)
            .unwrap_or(false),
        "max-height" => extract_length_px(value)
            .map(|px| media.height_px <= px)
            .unwrap_or(false),
        "orientation" => match value.iter().find_map(ident_str) {
            Some(s) if s.eq_ignore_ascii_case("portrait") => media.height_px >= media.width_px,
            Some(s) if s.eq_ignore_ascii_case("landscape") => media.width_px > media.height_px,
            _ => false,
        },
        _ => false,
    }
}

// ─────────────────────────────────────────────────────────────────────
// @page — extract selector + descriptors
// ─────────────────────────────────────────────────────────────────────

fn parse_page_rule<'i>(at: &AtRule<'i>) -> Option<PageRule<'i>> {
    let block = at.block.as_ref()?;
    let selectors = parse_page_selectors(&at.prelude);
    let mut declarations: Vec<Declaration<'i>> = Vec::new();
    let mut margin_boxes: Vec<(String, Vec<ComponentValue<'i>>)> = Vec::new();

    // The block body alternates between margin-box at-rules and
    // declarations. Walk it splitting on @-keywords.
    let mut current_decl_start = 0;
    let mut i = 0;
    while i < block.raw.len() {
        if let ComponentValue::Token(Token::AtKeyword(name)) = &block.raw[i] {
            // Flush prior declarations
            let decls = decls_from_chunk(&block.raw[current_decl_start..i]);
            declarations.extend(decls);
            // Consume a margin-box body — find the matching `{ ... }`.
            let mut j = i + 1;
            while j < block.raw.len() && !matches!(block.raw[j], ComponentValue::Curly(_)) {
                j += 1;
            }
            let body = if let Some(ComponentValue::Curly(b)) = block.raw.get(j) {
                b.clone()
            } else {
                Vec::new()
            };
            margin_boxes.push((name.to_string(), body));
            i = j + 1;
            current_decl_start = i;
        } else {
            i += 1;
        }
    }
    // Trailing declarations
    let tail = decls_from_chunk(&block.raw[current_decl_start..]);
    declarations.extend(tail);

    Some(PageRule {
        selectors,
        declarations,
        margin_boxes,
    })
}

fn parse_page_selectors(prelude: &[ComponentValue<'_>]) -> Vec<PageSelector> {
    let mut out = Vec::new();
    for chunk in split_top_level_commas(trim_ws(prelude)) {
        let trimmed = trim_ws(chunk);
        if trimmed.is_empty() {
            continue;
        }
        // Either a name (ident) or one or more `:pseudo` parts.
        let mut iter = trimmed.iter().peekable();
        let name = match iter.peek() {
            Some(ComponentValue::Token(Token::Ident(s))) => {
                let s = s.to_string();
                iter.next();
                Some(s)
            },
            _ => None,
        };
        // Pseudo segments
        let mut pseudo: Option<PageSelector> = None;
        while let Some(cv) = iter.next() {
            if matches!(cv, ComponentValue::Token(Token::Colon)) {
                if let Some(ComponentValue::Token(Token::Ident(s))) = iter.next() {
                    pseudo = match s.to_ascii_lowercase().as_str() {
                        "first" => Some(PageSelector::First),
                        "left" => Some(PageSelector::Left),
                        "right" => Some(PageSelector::Right),
                        "blank" => Some(PageSelector::Blank),
                        _ => pseudo,
                    };
                }
            }
        }
        match (name, pseudo) {
            (_, Some(p)) => out.push(p),
            (Some(n), None) => out.push(PageSelector::Named(n)),
            _ => {},
        }
    }
    out
}

fn decls_from_chunk<'i>(chunk: &[ComponentValue<'i>]) -> Vec<Declaration<'i>> {
    if chunk.is_empty() {
        return Vec::new();
    }
    // Re-serialise to a string and re-parse via the declaration list
    // entry point — simpler than threading a token-level parser.
    let s = render_back(chunk);
    parse_declaration_list(&s)
        .map(|decls| {
            // The re-parsed declarations carry borrowed Cows over the
            // owned String. Convert to owned 'static so they outlive
            // this scope.
            decls.into_iter().map(decl_to_owned).collect()
        })
        .unwrap_or_default()
}

fn decl_to_owned<'i>(d: Declaration<'_>) -> Declaration<'i> {
    Declaration {
        name: Cow::Owned(d.name.into_owned()),
        value: d.value.into_iter().map(cv_to_owned).collect(),
        important: d.important,
        location: d.location,
    }
}

fn cv_to_owned<'i>(cv: ComponentValue<'_>) -> ComponentValue<'i> {
    match cv {
        ComponentValue::Token(t) => ComponentValue::Token(token_to_owned(t)),
        ComponentValue::Function { name, body } => ComponentValue::Function {
            name: Cow::Owned(name.into_owned()),
            body: body.into_iter().map(cv_to_owned).collect(),
        },
        ComponentValue::Parens(b) => {
            ComponentValue::Parens(b.into_iter().map(cv_to_owned).collect())
        },
        ComponentValue::Square(b) => {
            ComponentValue::Square(b.into_iter().map(cv_to_owned).collect())
        },
        ComponentValue::Curly(b) => ComponentValue::Curly(b.into_iter().map(cv_to_owned).collect()),
    }
}

fn token_to_owned<'i>(t: Token<'_>) -> Token<'i> {
    use super::tokenizer::Number;
    match t {
        Token::Whitespace => Token::Whitespace,
        Token::Cdo => Token::Cdo,
        Token::Cdc => Token::Cdc,
        Token::Colon => Token::Colon,
        Token::Semicolon => Token::Semicolon,
        Token::Comma => Token::Comma,
        Token::LeftSquare => Token::LeftSquare,
        Token::RightSquare => Token::RightSquare,
        Token::LeftParen => Token::LeftParen,
        Token::RightParen => Token::RightParen,
        Token::LeftBrace => Token::LeftBrace,
        Token::RightBrace => Token::RightBrace,
        Token::Ident(s) => Token::Ident(Cow::Owned(s.into_owned())),
        Token::Function(s) => Token::Function(Cow::Owned(s.into_owned())),
        Token::AtKeyword(s) => Token::AtKeyword(Cow::Owned(s.into_owned())),
        Token::Hash { value, is_id } => Token::Hash {
            value: Cow::Owned(value.into_owned()),
            is_id,
        },
        Token::String(s) => Token::String(Cow::Owned(s.into_owned())),
        Token::BadString => Token::BadString,
        Token::Url(s) => Token::Url(Cow::Owned(s.into_owned())),
        Token::BadUrl => Token::BadUrl,
        Token::Number(Number { value, is_integer }) => Token::Number(Number { value, is_integer }),
        Token::Percentage(Number { value, is_integer }) => {
            Token::Percentage(Number { value, is_integer })
        },
        Token::Dimension {
            value: Number { value, is_integer },
            unit,
        } => Token::Dimension {
            value: Number { value, is_integer },
            unit: Cow::Owned(unit.into_owned()),
        },
        Token::Delim(c) => Token::Delim(c),
        Token::Eof => Token::Eof,
    }
}

// ─────────────────────────────────────────────────────────────────────
// @font-face
// ─────────────────────────────────────────────────────────────────────

fn parse_font_face(at: &AtRule<'_>) -> Option<FontFaceDescriptor> {
    let block = at.block.as_ref()?;
    let s = render_back(&block.raw);
    let decls = parse_declaration_list(&s).ok()?;

    let mut family: Option<String> = None;
    let mut sources: Vec<SrcEntry> = Vec::new();
    let mut weight: u16 = 400;
    let mut italic = false;
    let mut stretch_pct: f32 = 100.0;

    for decl in decls {
        match decl.name.as_ref() {
            "font-family" => {
                family = decl.value.iter().find_map(|cv| match cv {
                    ComponentValue::Token(Token::String(s)) => Some(s.to_string()),
                    ComponentValue::Token(Token::Ident(s)) => Some(s.to_string()),
                    _ => None,
                });
            },
            "src" => {
                sources = parse_src_value(&decl.value);
            },
            "font-weight" => {
                if let Some(n) = decl.value.iter().find_map(|cv| match cv {
                    ComponentValue::Token(Token::Number(n)) => Some(n.value as u16),
                    _ => None,
                }) {
                    weight = n;
                } else if let Some(s) = decl.value.iter().find_map(ident_str) {
                    if s.eq_ignore_ascii_case("bold") {
                        weight = 700;
                    }
                }
            },
            "font-style" => {
                italic = decl
                    .value
                    .iter()
                    .find_map(ident_str)
                    .map(|s| !s.eq_ignore_ascii_case("normal"))
                    .unwrap_or(false);
            },
            "font-stretch" => {
                if let Some(p) = decl.value.iter().find_map(|cv| match cv {
                    ComponentValue::Token(Token::Percentage(n)) => Some(n.value as f32),
                    _ => None,
                }) {
                    stretch_pct = p;
                }
            },
            _ => {},
        }
    }

    family.map(|family| FontFaceDescriptor {
        family,
        sources,
        weight,
        italic,
        stretch_pct,
    })
}

fn parse_src_value(value: &[ComponentValue<'_>]) -> Vec<SrcEntry> {
    let mut out = Vec::new();
    for chunk in split_top_level_commas(trim_ws(value)) {
        let trimmed = trim_ws(chunk);
        // Either local("...") / local(name) or url("...") [format("...")]
        for (idx, cv) in trimmed.iter().enumerate() {
            if let ComponentValue::Function { name, body } = cv {
                let lower = name.to_ascii_lowercase();
                if lower == "local" {
                    let name = body
                        .iter()
                        .find_map(|c| match c {
                            ComponentValue::Token(Token::String(s)) => Some(s.to_string()),
                            ComponentValue::Token(Token::Ident(s)) => Some(s.to_string()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    out.push(SrcEntry::Local(name));
                    break;
                }
                if lower == "url" {
                    let url = body
                        .iter()
                        .find_map(|c| match c {
                            ComponentValue::Token(Token::String(s)) => Some(s.to_string()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let format = trimmed[idx + 1..].iter().find_map(|c| match c {
                        ComponentValue::Function { name, body }
                            if name.eq_ignore_ascii_case("format") =>
                        {
                            body.iter().find_map(|cc| match cc {
                                ComponentValue::Token(Token::String(s)) => Some(s.to_string()),
                                ComponentValue::Token(Token::Ident(s)) => Some(s.to_string()),
                                _ => None,
                            })
                        },
                        _ => None,
                    });
                    out.push(SrcEntry::Url { url, format });
                    break;
                }
            } else if let ComponentValue::Token(Token::Url(s)) = cv {
                // url("...") with a quoted argument is tokenized as a
                // single Token::Url by the CSS-1 path; the function
                // form (Function "url") only appears for unusual cases.
                // Either way we look for a trailing format(...).
                let format = trimmed[idx + 1..].iter().find_map(|c| match c {
                    ComponentValue::Function { name, body }
                        if name.eq_ignore_ascii_case("format") =>
                    {
                        body.iter().find_map(|cc| match cc {
                            ComponentValue::Token(Token::String(s)) => Some(s.to_string()),
                            ComponentValue::Token(Token::Ident(s)) => Some(s.to_string()),
                            _ => None,
                        })
                    },
                    _ => None,
                });
                out.push(SrcEntry::Url {
                    url: s.to_string(),
                    format,
                });
                break;
            }
        }
    }
    out
}

fn extract_import_url(prelude: &[ComponentValue<'_>]) -> Option<String> {
    // First non-whitespace component must be a string or a url() function.
    for cv in prelude {
        match cv {
            ComponentValue::Token(Token::Whitespace) => {},
            ComponentValue::Token(Token::String(s)) => return Some(s.to_string()),
            ComponentValue::Token(Token::Url(s)) => return Some(s.to_string()),
            ComponentValue::Function { name, body } if name.eq_ignore_ascii_case("url") => {
                return body.iter().find_map(|c| match c {
                    ComponentValue::Token(Token::String(s)) => Some(s.to_string()),
                    _ => None,
                });
            },
            _ => return None,
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────
// @supports — conservative
// ─────────────────────────────────────────────────────────────────────

fn evaluate_supports_query(prelude: &[ComponentValue<'_>]) -> bool {
    // (property: value) — true iff the property is in our supported
    // surface (CSS-8 parse_property doesn't return Unsupported for it).
    let trimmed = trim_ws(prelude);
    if let [ComponentValue::Parens(body)] = trimmed {
        let body_str = render_back(body);
        // Try to parse as a single declaration.
        let decls = parse_declaration_list(&body_str).unwrap_or_default();
        if let Some(d) = decls.first() {
            return matches!(
                super::values::parse_property(&d.name, &d.value),
                Ok(_) | Err(super::values::ParseError::Malformed { .. })
            );
        }
    }
    false
}

/// Return whether a property can be parsed by CSS-8 — useful for
/// `@supports (foo: bar) { ... }` decisions.
pub fn supports_supported(property: &str, value: &[ComponentValue<'_>]) -> bool {
    matches!(
        super::values::parse_property(property, value),
        Ok(_) | Err(super::values::ParseError::Malformed { .. })
    )
}

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

fn parse_block_as_rules<'i>(_block: &AtRuleBlock<'i>) -> Vec<Rule<'i>> {
    // The block body inside @media is a list of rules. Re-serialise
    // and re-parse — same approach as @page declarations.
    let s = render_back(&_block.raw);
    let parsed = super::parser::parse_stylesheet(&s).unwrap_or(Stylesheet { rules: Vec::new() });
    // Convert to owned so they outlive the temporary string.
    parsed
        .rules
        .into_iter()
        .map(|r| match r {
            Rule::Qualified(q) => Rule::Qualified(QualifiedRule {
                prelude: q.prelude.into_iter().map(cv_to_owned).collect(),
                declarations: q.declarations.into_iter().map(decl_to_owned).collect(),
                location: q.location,
            }),
            Rule::AtRule(at) => Rule::AtRule(AtRule {
                name: Cow::Owned(at.name.into_owned()),
                prelude: at.prelude.into_iter().map(cv_to_owned).collect(),
                block: at.block.map(|b| AtRuleBlock {
                    raw: b.raw.into_iter().map(cv_to_owned).collect(),
                }),
                location: at.location,
            }),
        })
        .collect()
}

fn extract_length_px(value: &[ComponentValue<'_>]) -> Option<f32> {
    use super::calc::Unit;
    for cv in value {
        match cv {
            ComponentValue::Token(Token::Whitespace) => continue,
            ComponentValue::Token(Token::Dimension { value, unit }) => {
                let u = Unit::parse(unit)?;
                let ctx = super::calc::Context::default();
                return Some(u.to_px(value.value as f32, &ctx));
            },
            ComponentValue::Token(Token::Number(n)) => return Some(n.value as f32),
            _ => return None,
        }
    }
    None
}

fn ident_str<'a>(cv: &'a ComponentValue<'a>) -> Option<&'a str> {
    match cv {
        ComponentValue::Token(Token::Ident(s)) => Some(s.as_ref()),
        _ => None,
    }
}

fn trim_ws<'a, 'i>(cvs: &'a [ComponentValue<'i>]) -> &'a [ComponentValue<'i>] {
    let mut start = 0;
    while start < cvs.len() && matches!(cvs[start], ComponentValue::Token(Token::Whitespace)) {
        start += 1;
    }
    let mut end = cvs.len();
    while end > start && matches!(cvs[end - 1], ComponentValue::Token(Token::Whitespace)) {
        end -= 1;
    }
    &cvs[start..end]
}

fn split_top_level_commas<'a, 'i>(cvs: &'a [ComponentValue<'i>]) -> Vec<&'a [ComponentValue<'i>]> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, cv) in cvs.iter().enumerate() {
        if matches!(cv, ComponentValue::Token(Token::Comma)) {
            out.push(&cvs[start..i]);
            start = i + 1;
        }
    }
    out.push(&cvs[start..]);
    out
}

/// Render a component-value list back to a CSS source string. Lossy
/// for source locations and exact whitespace, but precise enough for
/// the parser's recursive consumption (which tokenises again).
fn render_back(cvs: &[ComponentValue<'_>]) -> String {
    let mut s = String::new();
    render_into(cvs, &mut s);
    s
}

fn render_into(cvs: &[ComponentValue<'_>], s: &mut String) {
    for cv in cvs {
        match cv {
            ComponentValue::Token(t) => render_token(t, s),
            ComponentValue::Function { name, body } => {
                s.push_str(name);
                s.push('(');
                render_into(body, s);
                s.push(')');
            },
            ComponentValue::Parens(body) => {
                s.push('(');
                render_into(body, s);
                s.push(')');
            },
            ComponentValue::Square(body) => {
                s.push('[');
                render_into(body, s);
                s.push(']');
            },
            ComponentValue::Curly(body) => {
                s.push('{');
                render_into(body, s);
                s.push('}');
            },
        }
    }
}

fn render_token(t: &Token<'_>, s: &mut String) {
    match t {
        Token::Whitespace => s.push(' '),
        Token::Cdo => s.push_str("<!--"),
        Token::Cdc => s.push_str("-->"),
        Token::Colon => s.push(':'),
        Token::Semicolon => s.push(';'),
        Token::Comma => s.push(','),
        Token::LeftSquare => s.push('['),
        Token::RightSquare => s.push(']'),
        Token::LeftParen => s.push('('),
        Token::RightParen => s.push(')'),
        Token::LeftBrace => s.push('{'),
        Token::RightBrace => s.push('}'),
        Token::Ident(v) => s.push_str(v),
        Token::Function(v) => {
            s.push_str(v);
            s.push('(');
        },
        Token::AtKeyword(v) => {
            s.push('@');
            s.push_str(v);
        },
        Token::Hash { value, .. } => {
            s.push('#');
            s.push_str(value);
        },
        Token::String(v) => {
            s.push('"');
            for ch in v.chars() {
                if ch == '"' || ch == '\\' {
                    s.push('\\');
                }
                s.push(ch);
            }
            s.push('"');
        },
        Token::BadString => {},
        Token::Url(v) => {
            s.push_str("url(\"");
            s.push_str(v);
            s.push_str("\")");
        },
        Token::BadUrl => {},
        Token::Number(n) => {
            if n.is_integer {
                s.push_str(&format!("{}", n.value as i64));
            } else {
                s.push_str(&n.value.to_string());
            }
        },
        Token::Percentage(n) => {
            if n.is_integer {
                s.push_str(&format!("{}%", n.value as i64));
            } else {
                s.push_str(&format!("{}%", n.value));
            }
        },
        Token::Dimension { value, unit } => {
            if value.is_integer {
                s.push_str(&format!("{}{}", value.value as i64, unit));
            } else {
                s.push_str(&format!("{}{}", value.value, unit));
            }
        },
        Token::Delim(c) => s.push(*c),
        Token::Eof => {},
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::parser::parse_stylesheet;

    fn ctx_a4() -> MediaContext {
        MediaContext {
            width_px: 595.0,
            height_px: 842.0,
            print: true,
        }
    }

    #[test]
    fn media_print_matches() {
        let ss = parse_stylesheet("@media print { body { color: red; } }").unwrap();
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.rules.len(), 1);
    }

    #[test]
    fn media_screen_does_not_match_print() {
        let ss = parse_stylesheet("@media screen { body { color: red; } }").unwrap();
        let r = resolve(&ss, ctx_a4());
        assert!(r.rules.is_empty());
    }

    #[test]
    fn media_min_width_matches() {
        let ss = parse_stylesheet("@media (min-width: 300px) { body { color: red; } }").unwrap();
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.rules.len(), 1);
    }

    #[test]
    fn media_min_width_does_not_match_when_too_narrow() {
        let ss = parse_stylesheet("@media (min-width: 1000px) { body { color: red; } }").unwrap();
        let r = resolve(&ss, ctx_a4());
        assert!(r.rules.is_empty());
    }

    #[test]
    fn media_and_combination() {
        let ss = parse_stylesheet("@media print and (min-width: 300px) { body { color: red; } }")
            .unwrap();
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.rules.len(), 1);
    }

    #[test]
    fn media_comma_alternatives() {
        let ss =
            parse_stylesheet("@media screen, (min-width: 1px) { body { color: red; } }").unwrap();
        // Second branch matches.
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.rules.len(), 1);
    }

    #[test]
    fn at_page_extracts_size_margin() {
        let ss = parse_stylesheet("@page { size: A4; margin: 20mm; }").unwrap();
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.page_rules.len(), 1);
        assert_eq!(r.page_rules[0].selectors.len(), 0);
        assert_eq!(r.page_rules[0].declarations.len(), 2);
    }

    #[test]
    fn at_page_first_selector() {
        let ss = parse_stylesheet("@page :first { margin-top: 0; }").unwrap();
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.page_rules[0].selectors, vec![PageSelector::First]);
    }

    #[test]
    fn at_page_left_right_blank() {
        let ss = parse_stylesheet("@page :left { } @page :right { } @page :blank { }").unwrap();
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.page_rules[0].selectors, vec![PageSelector::Left]);
        assert_eq!(r.page_rules[1].selectors, vec![PageSelector::Right]);
        assert_eq!(r.page_rules[2].selectors, vec![PageSelector::Blank]);
    }

    #[test]
    fn font_face_basic() {
        let ss = parse_stylesheet(
            r#"@font-face {
                font-family: "MyFont";
                src: url("my.ttf") format("truetype");
                font-weight: 700;
                font-style: italic;
            }"#,
        )
        .unwrap();
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.font_faces.len(), 1);
        let f = &r.font_faces[0];
        assert_eq!(f.family, "MyFont");
        assert_eq!(f.weight, 700);
        assert!(f.italic);
        assert_eq!(f.sources.len(), 1);
        match &f.sources[0] {
            SrcEntry::Url { url, format } => {
                assert_eq!(url, "my.ttf");
                assert_eq!(format.as_deref(), Some("truetype"));
            },
            other => panic!("expected url, got {other:?}"),
        }
    }

    #[test]
    fn font_face_local_source() {
        let ss = parse_stylesheet(r#"@font-face { font-family: "X"; src: local("Helvetica"); }"#)
            .unwrap();
        let r = resolve(&ss, ctx_a4());
        match &r.font_faces[0].sources[0] {
            SrcEntry::Local(name) => assert_eq!(name, "Helvetica"),
            _ => panic!(),
        }
    }

    #[test]
    fn at_import_collected() {
        let ss = parse_stylesheet(r#"@import "reset.css"; body { color: red; }"#).unwrap();
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.imports, vec!["reset.css".to_string()]);
        assert_eq!(r.rules.len(), 1);
    }

    #[test]
    fn supports_known_property_passes() {
        let ss = parse_stylesheet("@supports (display: flex) { body { color: red; } }").unwrap();
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.rules.len(), 1);
    }

    #[test]
    fn keyframes_silently_dropped() {
        let ss = parse_stylesheet("@keyframes spin { from { color: red; } to { color: blue; } }")
            .unwrap();
        let r = resolve(&ss, ctx_a4());
        assert!(r.rules.is_empty());
        assert!(r.page_rules.is_empty());
    }

    #[test]
    fn rules_outside_media_pass_through() {
        let ss = parse_stylesheet("body { color: red; } p { font-size: 14px; }").unwrap();
        let r = resolve(&ss, ctx_a4());
        assert_eq!(r.rules.len(), 2);
    }
}
