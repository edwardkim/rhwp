//! Typed CSS property values + per-property parsers (CSS-8).
//!
//! Cascade (CSS-5) hands back per-property `Vec<ComponentValue>`.
//! Layout (Phase LAYOUT) wants typed values: a `Color` it can paint
//! with, a `Length` it can resolve to px against a [`CalcContext`],
//! a `Display` it can switch on. This module provides the parsers
//! that bridge the two.
//!
//! v0.3.35 first cut covers the high-leverage properties:
//!
//! - **Colour values** — named (CSS Color L4 list), `#rgb` / `#rrggbb`
//!   / `#rgba` / `#rrggbbaa`, `rgb()`/`rgba()`, `hsl()`/`hsla()`,
//!   `currentColor`, `transparent`.
//! - **Lengths** — every unit from CSS-6 plus `auto`, including
//!   `calc()` / `min()` / `max()` / `clamp()` integration.
//! - **Core property keywords** — `display`, `font-style`,
//!   `font-weight`, `text-align`, `position`, `overflow`,
//!   `white-space`, `box-sizing`, `visibility`.
//! - **Font** — `font-size` (length or named keywords), `font-family`
//!   (comma-separated list of strings + generics), `font-weight`
//!   (numeric + keywords).
//! - **Margin / padding shorthand** — 1-, 2-, 3-, 4-value forms.
//! - **Width / Height** — Length or `auto`.
//!
//! Anything not covered here returns `Err(ParseError::Unsupported)` —
//! the cascade keeps the raw component values, and a later release
//! adds typed parsing without breaking the API.

use std::borrow::Cow;
use thiserror::Error;

use super::calc::{evaluate_function, Context as CalcContext, Unit};
use super::parser::ComponentValue;
use super::tokenizer::Token;

// ─────────────────────────────────────────────────────────────────────
// Top-level typed value
// ─────────────────────────────────────────────────────────────────────

/// CSS value after typed parsing. Intentionally narrow — adding new
/// variants is cheap, but every Phase LAYOUT consumer reads them so
/// changes ripple.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// `color`, `background-color`, …
    Color(Color),
    /// Resolvable length (may carry a calc tree until evaluated).
    Length(Length),
    /// Bare number — `font-weight: 700`, `line-height: 1.4`, …
    Number(f32),
    /// `font-size: 50%` etc. — caller decides what 100% means.
    Percentage(f32),
    /// Keyword the property recognises (`block`, `flex`, `auto`, …).
    Keyword(String),
    /// Plain string (`font-family: "Helvetica Neue"`).
    Str(String),
    /// Comma- or whitespace-separated list (multi-family fonts,
    /// margin shorthand, …).
    List(Vec<Value>),
    /// `url(...)` reference.
    Url(String),
    /// `none` keyword. Distinct from absent so layout can react.
    None,
}

/// CSS colour in linear sRGB with straight alpha. Components are
/// 0..=1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red 0..=1.
    pub r: f32,
    /// Green 0..=1.
    pub g: f32,
    /// Blue 0..=1.
    pub b: f32,
    /// Alpha 0..=1.
    pub a: f32,
}

impl Color {
    /// Construct from 8-bit sRGB integers.
    pub const fn rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }
    /// Construct from 8-bit sRGB integers + 8-bit alpha.
    pub const fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }
    /// `transparent` keyword.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    /// `black`.
    pub const BLACK: Self = Self::rgb_u8(0, 0, 0);
    /// `white`.
    pub const WHITE: Self = Self::rgb_u8(255, 255, 255);
}

/// Resolvable CSS length. Either an absolute value with a unit, or a
/// calc() expression preserved for later evaluation when the layout
/// context is available, or the special `auto` sentinel.
#[derive(Debug, Clone, PartialEq)]
pub enum Length {
    /// Explicit value + unit.
    Dim {
        /// Numeric component.
        value: f32,
        /// Unit.
        unit: Unit,
    },
    /// `auto` — caller resolves per property semantics.
    Auto,
    /// `calc()` body preserved as raw component values; evaluate via
    /// [`Length::resolve`] when a [`CalcContext`] is in hand.
    Calc {
        /// Function name — usually "calc" but `min`/`max`/`clamp`
        /// take the same code path.
        name: String,
        /// Function body.
        body: Vec<RawComponentValue>,
    },
}

impl Length {
    /// Evaluate to px against a calc context.
    pub fn resolve(&self, ctx: &CalcContext) -> Option<f32> {
        match self {
            Length::Dim { value, unit } => Some(unit.to_px(*value, ctx)),
            Length::Auto => None,
            Length::Calc { name, body } => {
                let cvs = raw_to_component_values(body);
                evaluate_function(name, &cvs, ctx).ok()
            },
        }
    }
}

/// Owned mirror of [`ComponentValue`] for embedding inside `Value::Length`
/// without borrowing the source stylesheet. Functionally equivalent —
/// keeps the typed-value API self-contained.
#[derive(Debug, Clone, PartialEq)]
pub enum RawComponentValue {
    /// Tokens as in the parser, but with owned strings.
    Token(RawToken),
    /// Function call.
    Function {
        /// Function name.
        name: String,
        /// Body.
        body: Vec<RawComponentValue>,
    },
    /// `( ... )`.
    Parens(Vec<RawComponentValue>),
    /// `[ ... ]`.
    Square(Vec<RawComponentValue>),
    /// `{ ... }`.
    Curly(Vec<RawComponentValue>),
}

/// Owned mirror of [`Token`] for embedding inside [`RawComponentValue`].
#[derive(Debug, Clone, PartialEq)]
pub enum RawToken {
    /// As parser-side.
    Whitespace,
    /// As parser-side.
    Cdo,
    /// As parser-side.
    Cdc,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `[`
    LeftSquare,
    /// `]`
    RightSquare,
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `name`
    Ident(String),
    /// `name(`
    Function(String),
    /// `@name`
    AtKeyword(String),
    /// `#hash`
    Hash {
        /// Value after #.
        value: String,
        /// Whether the value parses as an ident.
        is_id: bool,
    },
    /// `"..."` / `'...'`
    Str(String),
    /// `url(...)`
    Url(String),
    /// Numeric literal.
    Number {
        /// Value.
        value: f64,
        /// Whether source was integer.
        is_integer: bool,
    },
    /// `12%`
    Percentage {
        /// Value.
        value: f64,
        /// Whether source was integer.
        is_integer: bool,
    },
    /// `12px`
    Dimension {
        /// Value.
        value: f64,
        /// Whether source was integer.
        is_integer: bool,
        /// Unit string.
        unit: String,
    },
    /// Single delimiter character.
    Delim(char),
}

fn token_to_raw(t: &Token<'_>) -> RawToken {
    match t {
        Token::Whitespace => RawToken::Whitespace,
        Token::Cdo => RawToken::Cdo,
        Token::Cdc => RawToken::Cdc,
        Token::Colon => RawToken::Colon,
        Token::Semicolon => RawToken::Semicolon,
        Token::Comma => RawToken::Comma,
        Token::LeftSquare => RawToken::LeftSquare,
        Token::RightSquare => RawToken::RightSquare,
        Token::LeftParen => RawToken::LeftParen,
        Token::RightParen => RawToken::RightParen,
        Token::LeftBrace => RawToken::LeftBrace,
        Token::RightBrace => RawToken::RightBrace,
        Token::Ident(s) => RawToken::Ident(s.to_string()),
        Token::Function(s) => RawToken::Function(s.to_string()),
        Token::AtKeyword(s) => RawToken::AtKeyword(s.to_string()),
        Token::Hash { value, is_id } => RawToken::Hash {
            value: value.to_string(),
            is_id: *is_id,
        },
        Token::String(s) => RawToken::Str(s.to_string()),
        Token::BadString => RawToken::Str(String::new()),
        Token::Url(s) => RawToken::Url(s.to_string()),
        Token::BadUrl => RawToken::Url(String::new()),
        Token::Number(n) => RawToken::Number {
            value: n.value,
            is_integer: n.is_integer,
        },
        Token::Percentage(n) => RawToken::Percentage {
            value: n.value,
            is_integer: n.is_integer,
        },
        Token::Dimension { value, unit } => RawToken::Dimension {
            value: value.value,
            is_integer: value.is_integer,
            unit: unit.to_string(),
        },
        Token::Delim(c) => RawToken::Delim(*c),
        Token::Eof => RawToken::Whitespace, // shouldn't appear in values
    }
}

fn cv_to_raw(cv: &ComponentValue<'_>) -> RawComponentValue {
    match cv {
        ComponentValue::Token(t) => RawComponentValue::Token(token_to_raw(t)),
        ComponentValue::Function { name, body } => RawComponentValue::Function {
            name: name.to_string(),
            body: body.iter().map(cv_to_raw).collect(),
        },
        ComponentValue::Parens(b) => RawComponentValue::Parens(b.iter().map(cv_to_raw).collect()),
        ComponentValue::Square(b) => RawComponentValue::Square(b.iter().map(cv_to_raw).collect()),
        ComponentValue::Curly(b) => RawComponentValue::Curly(b.iter().map(cv_to_raw).collect()),
    }
}

fn raw_to_component_values(raws: &[RawComponentValue]) -> Vec<ComponentValue<'static>> {
    raws.iter().map(raw_to_cv).collect()
}

fn raw_to_cv(raw: &RawComponentValue) -> ComponentValue<'static> {
    match raw {
        RawComponentValue::Token(t) => ComponentValue::Token(raw_to_token(t)),
        RawComponentValue::Function { name, body } => ComponentValue::Function {
            name: Cow::Owned(name.clone()),
            body: body.iter().map(raw_to_cv).collect(),
        },
        RawComponentValue::Parens(b) => ComponentValue::Parens(b.iter().map(raw_to_cv).collect()),
        RawComponentValue::Square(b) => ComponentValue::Square(b.iter().map(raw_to_cv).collect()),
        RawComponentValue::Curly(b) => ComponentValue::Curly(b.iter().map(raw_to_cv).collect()),
    }
}

fn raw_to_token(t: &RawToken) -> Token<'static> {
    use super::tokenizer::Number;
    match t {
        RawToken::Whitespace => Token::Whitespace,
        RawToken::Cdo => Token::Cdo,
        RawToken::Cdc => Token::Cdc,
        RawToken::Colon => Token::Colon,
        RawToken::Semicolon => Token::Semicolon,
        RawToken::Comma => Token::Comma,
        RawToken::LeftSquare => Token::LeftSquare,
        RawToken::RightSquare => Token::RightSquare,
        RawToken::LeftParen => Token::LeftParen,
        RawToken::RightParen => Token::RightParen,
        RawToken::LeftBrace => Token::LeftBrace,
        RawToken::RightBrace => Token::RightBrace,
        RawToken::Ident(s) => Token::Ident(Cow::Owned(s.clone())),
        RawToken::Function(s) => Token::Function(Cow::Owned(s.clone())),
        RawToken::AtKeyword(s) => Token::AtKeyword(Cow::Owned(s.clone())),
        RawToken::Hash { value, is_id } => Token::Hash {
            value: Cow::Owned(value.clone()),
            is_id: *is_id,
        },
        RawToken::Str(s) => Token::String(Cow::Owned(s.clone())),
        RawToken::Url(s) => Token::Url(Cow::Owned(s.clone())),
        RawToken::Number { value, is_integer } => Token::Number(Number {
            value: *value,
            is_integer: *is_integer,
        }),
        RawToken::Percentage { value, is_integer } => Token::Percentage(Number {
            value: *value,
            is_integer: *is_integer,
        }),
        RawToken::Dimension {
            value,
            is_integer,
            unit,
        } => Token::Dimension {
            value: Number {
                value: *value,
                is_integer: *is_integer,
            },
            unit: Cow::Owned(unit.clone()),
        },
        RawToken::Delim(c) => Token::Delim(*c),
    }
}

// ─────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────

/// Property-parser errors.
#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    /// Empty value.
    #[error("empty value")]
    Empty,
    /// We don't know how to parse this property yet (v0.3.35 cuts).
    #[error("unsupported property: {0}")]
    Unsupported(String),
    /// Malformed value for a property we do know.
    #[error("malformed {property}: {reason}")]
    Malformed {
        /// Property name.
        property: String,
        /// Human-readable reason.
        reason: &'static str,
    },
}

// ─────────────────────────────────────────────────────────────────────
// Top-level dispatch
// ─────────────────────────────────────────────────────────────────────

/// Parse a property value into a typed [`Value`]. Returns
/// `Err(ParseError::Unsupported)` for properties that v0.3.35 doesn't
/// type yet — caller can fall back to inspecting the raw component
/// values for those.
pub fn parse_property(property: &str, value: &[ComponentValue<'_>]) -> Result<Value, ParseError> {
    let trimmed = trim_ws(value);
    if trimmed.is_empty() {
        return Err(ParseError::Empty);
    }
    match property.to_ascii_lowercase().as_str() {
        "color"
        | "background-color"
        | "border-color"
        | "border-top-color"
        | "border-right-color"
        | "border-bottom-color"
        | "border-left-color"
        | "outline-color"
        | "text-decoration-color"
        | "caret-color" => parse_color_value(trimmed, property),
        "width"
        | "height"
        | "min-width"
        | "min-height"
        | "max-width"
        | "max-height"
        | "top"
        | "right"
        | "bottom"
        | "left"
        | "padding-top"
        | "padding-right"
        | "padding-bottom"
        | "padding-left"
        | "border-top-width"
        | "border-right-width"
        | "border-bottom-width"
        | "border-left-width"
        | "font-size"
        | "letter-spacing"
        | "word-spacing"
        | "text-indent" => parse_length_or_auto(trimmed, property),
        "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => {
            parse_length_or_auto(trimmed, property)
        },
        "margin" => parse_box_shorthand(trimmed, property),
        "padding" => parse_box_shorthand(trimmed, property),
        "display" => parse_keyword(
            trimmed,
            property,
            &[
                "block",
                "inline",
                "inline-block",
                "flex",
                "inline-flex",
                "grid",
                "inline-grid",
                "table",
                "table-row",
                "table-cell",
                "table-header-group",
                "table-footer-group",
                "table-row-group",
                "table-column",
                "table-column-group",
                "table-caption",
                "list-item",
                "none",
                "contents",
            ],
        ),
        "position" => {
            parse_keyword(trimmed, property, &["static", "relative", "absolute", "fixed", "sticky"])
        },
        "overflow" | "overflow-x" | "overflow-y" => {
            parse_keyword(trimmed, property, &["visible", "hidden", "clip", "scroll", "auto"])
        },
        "visibility" => parse_keyword(trimmed, property, &["visible", "hidden", "collapse"]),
        "white-space" => parse_keyword(
            trimmed,
            property,
            &[
                "normal",
                "nowrap",
                "pre",
                "pre-wrap",
                "pre-line",
                "break-spaces",
            ],
        ),
        "text-align" => parse_keyword(
            trimmed,
            property,
            &["left", "right", "center", "justify", "start", "end"],
        ),
        "font-style" => parse_keyword(trimmed, property, &["normal", "italic", "oblique"]),
        "box-sizing" => parse_keyword(trimmed, property, &["content-box", "border-box"]),
        "font-weight" => parse_font_weight(trimmed),
        "font-family" => parse_font_family(trimmed),
        "line-height" => parse_line_height(trimmed),
        _ => Err(ParseError::Unsupported(property.to_string())),
    }
}

// ─────────────────────────────────────────────────────────────────────
// Colour parser
// ─────────────────────────────────────────────────────────────────────

fn parse_color_value(value: &[ComponentValue<'_>], property: &str) -> Result<Value, ParseError> {
    parse_color(value, property).map(Value::Color)
}

/// Parse a CSS colour from a value list. Handles named, hex, rgb(),
/// rgba(), hsl(), hsla(), `transparent`, `currentColor`.
pub fn parse_color(value: &[ComponentValue<'_>], property: &str) -> Result<Color, ParseError> {
    // Find the first non-whitespace component value.
    let cv = value
        .iter()
        .find(|cv| !matches!(cv, ComponentValue::Token(Token::Whitespace)))
        .ok_or(ParseError::Empty)?;
    match cv {
        ComponentValue::Token(Token::Ident(s)) => named_color(s).ok_or(ParseError::Malformed {
            property: property.to_string(),
            reason: "unknown colour keyword",
        }),
        ComponentValue::Token(Token::Hash { value, .. }) => {
            parse_hex_colour(value).ok_or(ParseError::Malformed {
                property: property.to_string(),
                reason: "malformed hex colour",
            })
        },
        ComponentValue::Function { name, body } => {
            let lower = name.to_ascii_lowercase();
            match lower.as_str() {
                "rgb" | "rgba" => parse_rgb_function(body, property),
                "hsl" | "hsla" => parse_hsl_function(body, property),
                _ => Err(ParseError::Malformed {
                    property: property.to_string(),
                    reason: "unsupported colour function",
                }),
            }
        },
        _ => Err(ParseError::Malformed {
            property: property.to_string(),
            reason: "expected colour value",
        }),
    }
}

fn parse_hex_colour(s: &str) -> Option<Color> {
    let bytes = s.as_bytes();
    let parse_nibble = |b: u8| match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    };
    match bytes.len() {
        3 => {
            let r = parse_nibble(bytes[0])?;
            let g = parse_nibble(bytes[1])?;
            let b = parse_nibble(bytes[2])?;
            Some(Color::rgb_u8(r * 17, g * 17, b * 17))
        },
        4 => {
            let r = parse_nibble(bytes[0])?;
            let g = parse_nibble(bytes[1])?;
            let b = parse_nibble(bytes[2])?;
            let a = parse_nibble(bytes[3])?;
            Some(Color::rgba_u8(r * 17, g * 17, b * 17, a * 17))
        },
        6 => {
            let r = (parse_nibble(bytes[0])? << 4) | parse_nibble(bytes[1])?;
            let g = (parse_nibble(bytes[2])? << 4) | parse_nibble(bytes[3])?;
            let b = (parse_nibble(bytes[4])? << 4) | parse_nibble(bytes[5])?;
            Some(Color::rgb_u8(r, g, b))
        },
        8 => {
            let r = (parse_nibble(bytes[0])? << 4) | parse_nibble(bytes[1])?;
            let g = (parse_nibble(bytes[2])? << 4) | parse_nibble(bytes[3])?;
            let b = (parse_nibble(bytes[4])? << 4) | parse_nibble(bytes[5])?;
            let a = (parse_nibble(bytes[6])? << 4) | parse_nibble(bytes[7])?;
            Some(Color::rgba_u8(r, g, b, a))
        },
        _ => None,
    }
}

fn parse_rgb_function(body: &[ComponentValue<'_>], property: &str) -> Result<Color, ParseError> {
    // Accept comma-separated and whitespace-separated forms. rgb()
    // accepts both 0..=255 numbers and 0%..=100% percentages for
    // colour components; alpha (4th) is always 0..=1.
    let comps = colour_components(body);
    if comps.len() < 3 {
        return Err(ParseError::Malformed {
            property: property.to_string(),
            reason: "rgb() needs 3 or 4 numeric components",
        });
    }
    let to_byte_u = |c: ColourComponent| match c {
        ColourComponent::Number(n) => n.clamp(0.0, 255.0) / 255.0,
        ColourComponent::Percentage(p) => p.clamp(0.0, 100.0) / 100.0,
    };
    let alpha = if comps.len() >= 4 {
        match comps[3] {
            ColourComponent::Number(n) => n.clamp(0.0, 1.0),
            ColourComponent::Percentage(p) => p.clamp(0.0, 100.0) / 100.0,
        }
    } else {
        1.0
    };
    Ok(Color {
        r: to_byte_u(comps[0]),
        g: to_byte_u(comps[1]),
        b: to_byte_u(comps[2]),
        a: alpha,
    })
}

fn parse_hsl_function(body: &[ComponentValue<'_>], property: &str) -> Result<Color, ParseError> {
    // hsl(h s% l% [/ a]). h is degrees (0..=360); s, l are percentages.
    let comps = colour_components(body);
    if comps.len() < 3 {
        return Err(ParseError::Malformed {
            property: property.to_string(),
            reason: "hsl() needs 3 or 4 numeric components",
        });
    }
    let hue_deg = match comps[0] {
        ColourComponent::Number(n) => n,
        ColourComponent::Percentage(p) => p * 3.6, // 100% = 360deg
    };
    let h = ((hue_deg % 360.0) + 360.0) % 360.0 / 360.0;
    let s = match comps[1] {
        ColourComponent::Percentage(p) => p.clamp(0.0, 100.0) / 100.0,
        ColourComponent::Number(n) => n.clamp(0.0, 1.0),
    };
    let l = match comps[2] {
        ColourComponent::Percentage(p) => p.clamp(0.0, 100.0) / 100.0,
        ColourComponent::Number(n) => n.clamp(0.0, 1.0),
    };
    let alpha = if comps.len() >= 4 {
        match comps[3] {
            ColourComponent::Number(n) => n.clamp(0.0, 1.0),
            ColourComponent::Percentage(p) => p.clamp(0.0, 100.0) / 100.0,
        }
    } else {
        1.0
    };
    let (r, g, b) = hsl_to_rgb(h, s, l);
    Ok(Color { r, g, b, a: alpha })
}

#[derive(Clone, Copy)]
enum ColourComponent {
    Number(f32),
    Percentage(f32),
}

fn colour_components(body: &[ComponentValue<'_>]) -> Vec<ColourComponent> {
    body.iter()
        .filter_map(|cv| match cv {
            ComponentValue::Token(Token::Number(n)) => {
                Some(ColourComponent::Number(n.value as f32))
            },
            ComponentValue::Token(Token::Percentage(n)) => {
                Some(ColourComponent::Percentage(n.value as f32))
            },
            ComponentValue::Token(Token::Dimension { value, .. }) => {
                // hsl() accepts deg/turn/rad units for hue. We treat
                // any dimension as a number for v0.3.35; CSS-9 (units
                // in colour functions) refines.
                Some(ColourComponent::Number(value.value as f32))
            },
            _ => None,
        })
        .collect()
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    (r, g, b)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

/// CSS named colour table (Color L4). Returns sRGB.
fn named_color(name: &str) -> Option<Color> {
    let name = name.to_ascii_lowercase();
    let rgb = match name.as_str() {
        "transparent" => return Some(Color::TRANSPARENT),
        "currentcolor" => return None, // caller handles inheritance
        "black" => (0, 0, 0),
        "silver" => (192, 192, 192),
        "gray" | "grey" => (128, 128, 128),
        "white" => (255, 255, 255),
        "maroon" => (128, 0, 0),
        "red" => (255, 0, 0),
        "purple" => (128, 0, 128),
        "fuchsia" | "magenta" => (255, 0, 255),
        "green" => (0, 128, 0),
        "lime" => (0, 255, 0),
        "olive" => (128, 128, 0),
        "yellow" => (255, 255, 0),
        "navy" => (0, 0, 128),
        "blue" => (0, 0, 255),
        "teal" => (0, 128, 128),
        "aqua" | "cyan" => (0, 255, 255),
        "orange" => (255, 165, 0),
        "aliceblue" => (240, 248, 255),
        "antiquewhite" => (250, 235, 215),
        "aquamarine" => (127, 255, 212),
        "azure" => (240, 255, 255),
        "beige" => (245, 245, 220),
        "bisque" => (255, 228, 196),
        "blanchedalmond" => (255, 235, 205),
        "blueviolet" => (138, 43, 226),
        "brown" => (165, 42, 42),
        "burlywood" => (222, 184, 135),
        "cadetblue" => (95, 158, 160),
        "chartreuse" => (127, 255, 0),
        "chocolate" => (210, 105, 30),
        "coral" => (255, 127, 80),
        "cornflowerblue" => (100, 149, 237),
        "cornsilk" => (255, 248, 220),
        "crimson" => (220, 20, 60),
        "darkblue" => (0, 0, 139),
        "darkcyan" => (0, 139, 139),
        "darkgoldenrod" => (184, 134, 11),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "darkgreen" => (0, 100, 0),
        "darkkhaki" => (189, 183, 107),
        "darkmagenta" => (139, 0, 139),
        "darkolivegreen" => (85, 107, 47),
        "darkorange" => (255, 140, 0),
        "darkorchid" => (153, 50, 204),
        "darkred" => (139, 0, 0),
        "darksalmon" => (233, 150, 122),
        "darkseagreen" => (143, 188, 143),
        "darkslateblue" => (72, 61, 139),
        "darkslategray" | "darkslategrey" => (47, 79, 79),
        "darkturquoise" => (0, 206, 209),
        "darkviolet" => (148, 0, 211),
        "deeppink" => (255, 20, 147),
        "deepskyblue" => (0, 191, 255),
        "dimgray" | "dimgrey" => (105, 105, 105),
        "dodgerblue" => (30, 144, 255),
        "firebrick" => (178, 34, 34),
        "floralwhite" => (255, 250, 240),
        "forestgreen" => (34, 139, 34),
        "gainsboro" => (220, 220, 220),
        "ghostwhite" => (248, 248, 255),
        "gold" => (255, 215, 0),
        "goldenrod" => (218, 165, 32),
        "greenyellow" => (173, 255, 47),
        "honeydew" => (240, 255, 240),
        "hotpink" => (255, 105, 180),
        "indianred" => (205, 92, 92),
        "indigo" => (75, 0, 130),
        "ivory" => (255, 255, 240),
        "khaki" => (240, 230, 140),
        "lavender" => (230, 230, 250),
        "lavenderblush" => (255, 240, 245),
        "lawngreen" => (124, 252, 0),
        "lemonchiffon" => (255, 250, 205),
        "lightblue" => (173, 216, 230),
        "lightcoral" => (240, 128, 128),
        "lightcyan" => (224, 255, 255),
        "lightgoldenrodyellow" => (250, 250, 210),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "lightgreen" => (144, 238, 144),
        "lightpink" => (255, 182, 193),
        "lightsalmon" => (255, 160, 122),
        "lightseagreen" => (32, 178, 170),
        "lightskyblue" => (135, 206, 250),
        "lightslategray" | "lightslategrey" => (119, 136, 153),
        "lightsteelblue" => (176, 196, 222),
        "lightyellow" => (255, 255, 224),
        "limegreen" => (50, 205, 50),
        "linen" => (250, 240, 230),
        "mediumaquamarine" => (102, 205, 170),
        "mediumblue" => (0, 0, 205),
        "mediumorchid" => (186, 85, 211),
        "mediumpurple" => (147, 112, 219),
        "mediumseagreen" => (60, 179, 113),
        "mediumslateblue" => (123, 104, 238),
        "mediumspringgreen" => (0, 250, 154),
        "mediumturquoise" => (72, 209, 204),
        "mediumvioletred" => (199, 21, 133),
        "midnightblue" => (25, 25, 112),
        "mintcream" => (245, 255, 250),
        "mistyrose" => (255, 228, 225),
        "moccasin" => (255, 228, 181),
        "navajowhite" => (255, 222, 173),
        "oldlace" => (253, 245, 230),
        "olivedrab" => (107, 142, 35),
        "orangered" => (255, 69, 0),
        "orchid" => (218, 112, 214),
        "palegoldenrod" => (238, 232, 170),
        "palegreen" => (152, 251, 152),
        "paleturquoise" => (175, 238, 238),
        "palevioletred" => (219, 112, 147),
        "papayawhip" => (255, 239, 213),
        "peachpuff" => (255, 218, 185),
        "peru" => (205, 133, 63),
        "pink" => (255, 192, 203),
        "plum" => (221, 160, 221),
        "powderblue" => (176, 224, 230),
        "rosybrown" => (188, 143, 143),
        "royalblue" => (65, 105, 225),
        "saddlebrown" => (139, 69, 19),
        "salmon" => (250, 128, 114),
        "sandybrown" => (244, 164, 96),
        "seagreen" => (46, 139, 87),
        "seashell" => (255, 245, 238),
        "sienna" => (160, 82, 45),
        "skyblue" => (135, 206, 235),
        "slateblue" => (106, 90, 205),
        "slategray" | "slategrey" => (112, 128, 144),
        "snow" => (255, 250, 250),
        "springgreen" => (0, 255, 127),
        "steelblue" => (70, 130, 180),
        "tan" => (210, 180, 140),
        "thistle" => (216, 191, 216),
        "tomato" => (255, 99, 71),
        "turquoise" => (64, 224, 208),
        "violet" => (238, 130, 238),
        "wheat" => (245, 222, 179),
        "whitesmoke" => (245, 245, 245),
        "yellowgreen" => (154, 205, 50),
        "rebeccapurple" => (102, 51, 153),
        _ => return None,
    };
    Some(Color::rgb_u8(rgb.0, rgb.1, rgb.2))
}

// ─────────────────────────────────────────────────────────────────────
// Length parser
// ─────────────────────────────────────────────────────────────────────

fn parse_length_or_auto(value: &[ComponentValue<'_>], property: &str) -> Result<Value, ParseError> {
    parse_length(value, property).map(Value::Length)
}

/// Parse one CSS length (or `auto`).
pub fn parse_length(value: &[ComponentValue<'_>], property: &str) -> Result<Length, ParseError> {
    let cv = value
        .iter()
        .find(|cv| !matches!(cv, ComponentValue::Token(Token::Whitespace)))
        .ok_or(ParseError::Empty)?;
    match cv {
        ComponentValue::Token(Token::Ident(s)) if s.eq_ignore_ascii_case("auto") => {
            Ok(Length::Auto)
        },
        ComponentValue::Token(Token::Dimension { value, unit }) => {
            let u = Unit::parse(unit).ok_or(ParseError::Malformed {
                property: property.to_string(),
                reason: "unknown unit",
            })?;
            Ok(Length::Dim {
                value: value.value as f32,
                unit: u,
            })
        },
        ComponentValue::Token(Token::Percentage(n)) => Ok(Length::Dim {
            value: n.value as f32,
            unit: Unit::Percent,
        }),
        ComponentValue::Token(Token::Number(n)) if n.value == 0.0 => Ok(Length::Dim {
            value: 0.0,
            unit: Unit::Px,
        }),
        ComponentValue::Function { name, body } => {
            let n = name.to_ascii_lowercase();
            if matches!(n.as_str(), "calc" | "min" | "max" | "clamp") {
                Ok(Length::Calc {
                    name: n,
                    body: body.iter().map(cv_to_raw).collect(),
                })
            } else {
                Err(ParseError::Malformed {
                    property: property.to_string(),
                    reason: "unsupported length function",
                })
            }
        },
        _ => Err(ParseError::Malformed {
            property: property.to_string(),
            reason: "expected length, percentage, calc, or auto",
        }),
    }
}

// ─────────────────────────────────────────────────────────────────────
// Margin / padding shorthand
// ─────────────────────────────────────────────────────────────────────

fn parse_box_shorthand(value: &[ComponentValue<'_>], property: &str) -> Result<Value, ParseError> {
    let parts: Vec<&[ComponentValue<'_>]> = value
        .split(|cv| matches!(cv, ComponentValue::Token(Token::Whitespace)))
        .filter(|chunk| !chunk.is_empty())
        .collect();
    if parts.is_empty() || parts.len() > 4 {
        return Err(ParseError::Malformed {
            property: property.to_string(),
            reason: "1..=4 length values expected",
        });
    }
    let mut lengths = Vec::with_capacity(parts.len());
    for p in parts {
        lengths.push(Value::Length(parse_length(p, property)?));
    }
    Ok(Value::List(lengths))
}

// ─────────────────────────────────────────────────────────────────────
// Font helpers
// ─────────────────────────────────────────────────────────────────────

fn parse_font_weight(value: &[ComponentValue<'_>]) -> Result<Value, ParseError> {
    let cv = value
        .iter()
        .find(|cv| !matches!(cv, ComponentValue::Token(Token::Whitespace)))
        .ok_or(ParseError::Empty)?;
    match cv {
        ComponentValue::Token(Token::Number(n)) if n.is_integer => {
            Ok(Value::Number(n.value as f32))
        },
        ComponentValue::Token(Token::Ident(s)) => {
            let lower = s.to_ascii_lowercase();
            let n = match lower.as_str() {
                "normal" => 400.0,
                "bold" => 700.0,
                "lighter" | "bolder" => return Ok(Value::Keyword(lower)),
                _ => {
                    return Err(ParseError::Malformed {
                        property: "font-weight".into(),
                        reason: "unknown font-weight keyword",
                    })
                },
            };
            Ok(Value::Number(n))
        },
        _ => Err(ParseError::Malformed {
            property: "font-weight".into(),
            reason: "expected number or keyword",
        }),
    }
}

fn parse_font_family(value: &[ComponentValue<'_>]) -> Result<Value, ParseError> {
    // Comma-separated list. Each entry is either a string literal or
    // one-or-more idents (collapsed with single spaces).
    let mut families: Vec<Value> = Vec::new();
    for chunk in split_top_level_commas(value) {
        let trimmed = trim_ws(chunk);
        if trimmed.is_empty() {
            continue;
        }
        if let ComponentValue::Token(Token::String(s)) = &trimmed[0] {
            families.push(Value::Str(s.to_string()));
            continue;
        }
        let mut name = String::new();
        for cv in trimmed {
            match cv {
                ComponentValue::Token(Token::Ident(s)) => {
                    if !name.is_empty() {
                        name.push(' ');
                    }
                    name.push_str(s);
                },
                ComponentValue::Token(Token::Whitespace) => {},
                _ => {
                    return Err(ParseError::Malformed {
                        property: "font-family".into(),
                        reason: "expected ident or string in family list",
                    })
                },
            }
        }
        if !name.is_empty() {
            families.push(Value::Str(name));
        }
    }
    if families.is_empty() {
        Err(ParseError::Empty)
    } else {
        Ok(Value::List(families))
    }
}

fn parse_line_height(value: &[ComponentValue<'_>]) -> Result<Value, ParseError> {
    let cv = value
        .iter()
        .find(|cv| !matches!(cv, ComponentValue::Token(Token::Whitespace)))
        .ok_or(ParseError::Empty)?;
    match cv {
        ComponentValue::Token(Token::Ident(s)) if s.eq_ignore_ascii_case("normal") => {
            Ok(Value::Keyword("normal".into()))
        },
        ComponentValue::Token(Token::Number(n)) => Ok(Value::Number(n.value as f32)),
        _ => parse_length(value, "line-height").map(Value::Length),
    }
}

// ─────────────────────────────────────────────────────────────────────
// Keyword helper
// ─────────────────────────────────────────────────────────────────────

fn parse_keyword(
    value: &[ComponentValue<'_>],
    property: &str,
    allowed: &[&str],
) -> Result<Value, ParseError> {
    let cv = value
        .iter()
        .find(|cv| !matches!(cv, ComponentValue::Token(Token::Whitespace)))
        .ok_or(ParseError::Empty)?;
    match cv {
        ComponentValue::Token(Token::Ident(s)) => {
            let lower = s.to_ascii_lowercase();
            if allowed.iter().any(|k| *k == lower) {
                Ok(Value::Keyword(lower))
            } else {
                Err(ParseError::Malformed {
                    property: property.to_string(),
                    reason: "unknown keyword",
                })
            }
        },
        _ => Err(ParseError::Malformed {
            property: property.to_string(),
            reason: "expected keyword",
        }),
    }
}

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::parser::{parse_stylesheet, Rule};

    fn parse(css: &'static str, property: &'static str) -> Result<Value, ParseError> {
        let ss: &'static _ = Box::leak(Box::new(parse_stylesheet(css).unwrap()));
        let r = match &ss.rules[0] {
            Rule::Qualified(q) => q,
            _ => panic!(),
        };
        let d = r.declarations.iter().find(|d| d.name == property).unwrap();
        parse_property(property, &d.value)
    }

    #[test]
    fn named_color_red() {
        let v = parse("p { color: red; }", "color").unwrap();
        assert_eq!(v, Value::Color(Color::rgb_u8(255, 0, 0)));
    }

    #[test]
    fn hex_short_form() {
        let v = parse("p { color: #f00; }", "color").unwrap();
        assert_eq!(v, Value::Color(Color::rgb_u8(255, 0, 0)));
    }

    #[test]
    fn hex_long_form() {
        let v = parse("p { color: #336699; }", "color").unwrap();
        assert_eq!(v, Value::Color(Color::rgb_u8(0x33, 0x66, 0x99)));
    }

    #[test]
    fn hex_with_alpha() {
        let v = parse("p { color: #ff000080; }", "color").unwrap();
        if let Value::Color(c) = v {
            assert_eq!(c.r, 1.0);
            assert!((c.a - (0x80 as f32 / 255.0)).abs() < 1e-3);
        } else {
            panic!()
        }
    }

    #[test]
    fn rgb_function_comma() {
        let v = parse("p { color: rgb(255, 128, 0); }", "color").unwrap();
        if let Value::Color(c) = v {
            assert_eq!(c.r, 1.0);
            assert!((c.g - 128.0 / 255.0).abs() < 1e-3);
        } else {
            panic!()
        }
    }

    #[test]
    fn rgba_function_with_alpha() {
        let v = parse("p { color: rgba(255, 0, 0, 0.5); }", "color").unwrap();
        if let Value::Color(c) = v {
            assert!((c.a - 0.5).abs() < 1e-3);
        } else {
            panic!()
        }
    }

    #[test]
    fn hsl_basic() {
        // hsl(0, 100%, 50%) = pure red.
        let v = parse("p { color: hsl(0, 100%, 50%); }", "color").unwrap();
        if let Value::Color(c) = v {
            assert!((c.r - 1.0).abs() < 1e-3);
            assert!(c.g.abs() < 1e-3);
            assert!(c.b.abs() < 1e-3);
        } else {
            panic!()
        }
    }

    #[test]
    fn transparent_keyword() {
        let v = parse("p { color: transparent; }", "color").unwrap();
        assert_eq!(v, Value::Color(Color::TRANSPARENT));
    }

    #[test]
    fn length_px() {
        let v = parse("p { width: 240px; }", "width").unwrap();
        assert_eq!(
            v,
            Value::Length(Length::Dim {
                value: 240.0,
                unit: Unit::Px
            })
        );
    }

    #[test]
    fn length_percent() {
        let v = parse("p { width: 50%; }", "width").unwrap();
        assert_eq!(
            v,
            Value::Length(Length::Dim {
                value: 50.0,
                unit: Unit::Percent
            })
        );
    }

    #[test]
    fn length_em() {
        let v = parse("p { font-size: 1.5em; }", "font-size").unwrap();
        assert_eq!(
            v,
            Value::Length(Length::Dim {
                value: 1.5,
                unit: Unit::Em
            })
        );
    }

    #[test]
    fn length_zero() {
        // Bare 0 is a valid length per spec (no unit needed).
        let v = parse("p { margin-left: 0; }", "margin-left").unwrap();
        assert_eq!(
            v,
            Value::Length(Length::Dim {
                value: 0.0,
                unit: Unit::Px
            })
        );
    }

    #[test]
    fn length_auto() {
        let v = parse("p { width: auto; }", "width").unwrap();
        assert_eq!(v, Value::Length(Length::Auto));
    }

    #[test]
    fn length_calc_resolves() {
        let v = parse("p { width: calc(100% - 20px); }", "width").unwrap();
        if let Value::Length(l) = v {
            let ctx = CalcContext {
                parent_px: 600.0,
                ..Default::default()
            };
            let resolved = l.resolve(&ctx).unwrap();
            assert!((resolved - 580.0).abs() < 1e-3);
        } else {
            panic!()
        }
    }

    #[test]
    fn margin_shorthand_one_value() {
        let v = parse("p { margin: 10px; }", "margin").unwrap();
        if let Value::List(items) = v {
            assert_eq!(items.len(), 1);
        } else {
            panic!()
        }
    }

    #[test]
    fn margin_shorthand_four_values() {
        let v = parse("p { margin: 1px 2px 3px 4px; }", "margin").unwrap();
        if let Value::List(items) = v {
            assert_eq!(items.len(), 4);
        } else {
            panic!()
        }
    }

    #[test]
    fn display_keyword() {
        let v = parse("p { display: flex; }", "display").unwrap();
        assert_eq!(v, Value::Keyword("flex".into()));
    }

    #[test]
    fn display_unknown_errors() {
        let res = parse("p { display: bogus; }", "display");
        assert!(matches!(res, Err(ParseError::Malformed { .. })));
    }

    #[test]
    fn font_weight_numeric() {
        let v = parse("p { font-weight: 700; }", "font-weight").unwrap();
        assert_eq!(v, Value::Number(700.0));
    }

    #[test]
    fn font_weight_keyword() {
        let v = parse("p { font-weight: bold; }", "font-weight").unwrap();
        assert_eq!(v, Value::Number(700.0));
    }

    #[test]
    fn font_family_list() {
        let v = parse(r#"p { font-family: "Helvetica Neue", Arial, sans-serif; }"#, "font-family")
            .unwrap();
        if let Value::List(items) = v {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Str("Helvetica Neue".into()));
            assert_eq!(items[1], Value::Str("Arial".into()));
            assert_eq!(items[2], Value::Str("sans-serif".into()));
        } else {
            panic!()
        }
    }

    #[test]
    fn font_family_unquoted_multiword() {
        let v = parse("p { font-family: Times New Roman, serif; }", "font-family").unwrap();
        if let Value::List(items) = v {
            assert_eq!(items[0], Value::Str("Times New Roman".into()));
        } else {
            panic!()
        }
    }

    #[test]
    fn line_height_number() {
        let v = parse("p { line-height: 1.5; }", "line-height").unwrap();
        assert_eq!(v, Value::Number(1.5));
    }

    #[test]
    fn line_height_normal() {
        let v = parse("p { line-height: normal; }", "line-height").unwrap();
        assert_eq!(v, Value::Keyword("normal".into()));
    }

    #[test]
    fn unsupported_property_errors() {
        let res = parse("p { quark: 7; }", "quark");
        assert!(matches!(res, Err(ParseError::Unsupported(_))));
    }
}
