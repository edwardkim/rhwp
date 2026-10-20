//! CSS counters + pseudo-element `content:` evaluation (CSS-10).
//!
//! Phase LAYOUT (next) needs two related pieces of CSS-10's job:
//!
//! 1. **Parse `content:` declarations** into a typed [`Content`] list
//!    so that ::before/::after generated boxes know what text to show.
//!    Functions covered: `counter(name [, style])`,
//!    `counters(name, sep [, style])`, `attr(name)`,
//!    `open-quote` / `close-quote` keywords, plain string literals,
//!    `none`/`normal` keywords.
//! 2. **Track counter state** across a document-order DOM walk so
//!    `counter(page)`, `counter(chapter)`, etc. produce the right
//!    integer at each pseudo-element. The state machine handles
//!    `counter-reset`, `counter-increment`, and `counter-set`.
//!
//! Counter-reset/-increment/-set parsing also lives here because the
//! cascade gives us their `Vec<ComponentValue>` and the layout walker
//! needs to apply them in source order — coupling everything counter-
//! related into one module keeps the split clean.

use std::collections::HashMap;

use super::parser::ComponentValue;
use super::tokenizer::Token;

// ─────────────────────────────────────────────────────────────────────
// Content list — typed `content:` value
// ─────────────────────────────────────────────────────────────────────

/// One item in a parsed `content:` declaration. The list is what
/// pseudo-element layout iterates to compose the generated text.
#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    /// Literal string ("Section ", " — done").
    Str(String),
    /// `counter(name [, style])`.
    Counter {
        /// Counter name.
        name: String,
        /// List-style numbering (default "decimal").
        style: ListStyle,
    },
    /// `counters(name, sep [, style])` — joins every same-named counter
    /// in scope from outermost to innermost.
    Counters {
        /// Counter name.
        name: String,
        /// Separator string between levels.
        separator: String,
        /// List-style numbering.
        style: ListStyle,
    },
    /// `attr(name)` — pulls the named attribute from the host element.
    Attr {
        /// Attribute name.
        name: String,
    },
    /// `open-quote` / `close-quote` — uses the `quotes` property to
    /// pick the correct character. v0.3.35 ships hard-coded English
    /// double quotes; CSS-10b can extend.
    OpenQuote,
    /// See [`Content::OpenQuote`].
    CloseQuote,
}

/// Numbering system for `counter()` / `counters()`. v0.3.35 covers
/// the common set; @counter-style custom systems are a later release.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStyle {
    /// `1, 2, 3, ...`.
    Decimal,
    /// `01, 02, …, 10` (zero-padded to width 2 — spec says
    /// auto-pad based on the largest counter, but v0.3.35 uses 2).
    DecimalLeadingZero,
    /// `i, ii, iii, iv`.
    LowerRoman,
    /// `I, II, III, IV`.
    UpperRoman,
    /// `a, b, c, …, z, aa, ab, …`.
    LowerAlpha,
    /// `A, B, C, …`.
    UpperAlpha,
    /// `α, β, γ, …`.
    LowerGreek,
    /// `disc`/`circle`/`square` for unordered lists — these aren't
    /// counters per se but the parser tolerates them.
    Disc,
    /// As `Disc`.
    Circle,
    /// As `Disc`.
    Square,
    /// `none`.
    None,
}

impl ListStyle {
    /// Render an integer in this style. Negative numbers (rare for
    /// counters but possible after counter-set) render as decimal.
    pub fn render(self, n: i32) -> String {
        if n <= 0 {
            return n.to_string();
        }
        match self {
            ListStyle::Decimal => n.to_string(),
            ListStyle::DecimalLeadingZero => format!("{n:02}"),
            ListStyle::LowerRoman => to_roman(n).to_lowercase(),
            ListStyle::UpperRoman => to_roman(n),
            ListStyle::LowerAlpha => to_alpha(n, 'a'),
            ListStyle::UpperAlpha => to_alpha(n, 'A'),
            ListStyle::LowerGreek => to_greek(n),
            ListStyle::Disc => "•".to_string(),
            ListStyle::Circle => "◦".to_string(),
            ListStyle::Square => "▪".to_string(),
            ListStyle::None => String::new(),
        }
    }

    /// Look up by lowercase keyword.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "decimal" => Self::Decimal,
            "decimal-leading-zero" => Self::DecimalLeadingZero,
            "lower-roman" => Self::LowerRoman,
            "upper-roman" => Self::UpperRoman,
            "lower-alpha" | "lower-latin" => Self::LowerAlpha,
            "upper-alpha" | "upper-latin" => Self::UpperAlpha,
            "lower-greek" => Self::LowerGreek,
            "disc" => Self::Disc,
            "circle" => Self::Circle,
            "square" => Self::Square,
            "none" => Self::None,
            _ => return None,
        })
    }
}

fn to_roman(mut n: i32) -> String {
    const TABLE: &[(i32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for &(value, sym) in TABLE {
        while n >= value {
            out.push_str(sym);
            n -= value;
        }
    }
    out
}

fn to_alpha(n: i32, base: char) -> String {
    // n=1 → base, n=27 → "aa", … in spreadsheet-column style.
    let mut n = n;
    let mut chars = Vec::new();
    while n > 0 {
        n -= 1;
        chars.push(((n % 26) as u8 + base as u8) as char);
        n /= 26;
    }
    chars.reverse();
    chars.into_iter().collect()
}

fn to_greek(mut n: i32) -> String {
    // 24 lower-case Greek letters, then wrap.
    const GREEK: &[char] = &[
        'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο', 'π', 'ρ', 'σ',
        'τ', 'υ', 'φ', 'χ', 'ψ', 'ω',
    ];
    let mut out = String::new();
    while n > 0 {
        let idx = (n - 1) % GREEK.len() as i32;
        out.insert(0, GREEK[idx as usize]);
        n /= GREEK.len() as i32;
    }
    out
}

// ─────────────────────────────────────────────────────────────────────
// `content:` parser
// ─────────────────────────────────────────────────────────────────────

/// Parse a typed `content:` declaration value. Returns `None` for
/// `content: none` / `normal` (caller treats as "no pseudo-element box
/// generated" per spec). Returns `Some(Vec)` otherwise.
pub fn parse_content(value: &[ComponentValue<'_>]) -> Option<Vec<Content>> {
    let trimmed = trim_ws(value);
    if let [ComponentValue::Token(Token::Ident(s))] = trimmed {
        if s.eq_ignore_ascii_case("none") || s.eq_ignore_ascii_case("normal") {
            return None;
        }
    }
    let mut out = Vec::new();
    for cv in trimmed {
        match cv {
            ComponentValue::Token(Token::Whitespace) => {},
            ComponentValue::Token(Token::String(s)) => out.push(Content::Str(s.to_string())),
            ComponentValue::Token(Token::Ident(s)) => match s.to_ascii_lowercase().as_str() {
                "open-quote" => out.push(Content::OpenQuote),
                "close-quote" => out.push(Content::CloseQuote),
                _ => {},
            },
            ComponentValue::Function { name, body } => {
                let lower = name.to_ascii_lowercase();
                match lower.as_str() {
                    "counter" => {
                        let (n, style) = parse_counter_args(body);
                        out.push(Content::Counter {
                            name: n,
                            style: style.unwrap_or(ListStyle::Decimal),
                        });
                    },
                    "counters" => {
                        let (n, sep, style) = parse_counters_args(body);
                        out.push(Content::Counters {
                            name: n,
                            separator: sep,
                            style: style.unwrap_or(ListStyle::Decimal),
                        });
                    },
                    "attr" => {
                        if let Some(name) = body.iter().find_map(|c| match c {
                            ComponentValue::Token(Token::Ident(s)) => Some(s.to_string()),
                            _ => None,
                        }) {
                            out.push(Content::Attr { name });
                        }
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }
    Some(out)
}

fn parse_counter_args(body: &[ComponentValue<'_>]) -> (String, Option<ListStyle>) {
    let mut iter = body.iter().filter(|cv| {
        !matches!(
            cv,
            ComponentValue::Token(Token::Whitespace) | ComponentValue::Token(Token::Comma)
        )
    });
    let name = match iter.next() {
        Some(ComponentValue::Token(Token::Ident(s))) => s.to_string(),
        _ => return (String::new(), None),
    };
    let style = match iter.next() {
        Some(ComponentValue::Token(Token::Ident(s))) => ListStyle::parse(s),
        _ => None,
    };
    (name, style)
}

fn parse_counters_args(body: &[ComponentValue<'_>]) -> (String, String, Option<ListStyle>) {
    let mut iter = body.iter().filter(|cv| {
        !matches!(
            cv,
            ComponentValue::Token(Token::Whitespace) | ComponentValue::Token(Token::Comma)
        )
    });
    let name = match iter.next() {
        Some(ComponentValue::Token(Token::Ident(s))) => s.to_string(),
        _ => return (String::new(), String::new(), None),
    };
    let sep = match iter.next() {
        Some(ComponentValue::Token(Token::String(s))) => s.to_string(),
        _ => return (name, String::new(), None),
    };
    let style = match iter.next() {
        Some(ComponentValue::Token(Token::Ident(s))) => ListStyle::parse(s),
        _ => None,
    };
    (name, sep, style)
}

// ─────────────────────────────────────────────────────────────────────
// counter-reset / counter-increment / counter-set parsers
// ─────────────────────────────────────────────────────────────────────

/// One instruction extracted from a counter-{reset,increment,set}
/// declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterOp {
    /// Counter name.
    pub name: String,
    /// Numeric argument (0 default for counter-reset, 1 for counter-
    /// increment, 0 for counter-set if absent).
    pub value: i32,
}

/// Parse `counter-reset: chapter section 0; counter-increment: chapter`
/// style values. The `default` is the spec-mandated fallback when the
/// value is omitted (0 for reset/set, 1 for increment).
pub fn parse_counter_ops(value: &[ComponentValue<'_>], default: i32) -> Vec<CounterOp> {
    let mut out = Vec::new();
    let mut name: Option<String> = None;
    for cv in value {
        match cv {
            ComponentValue::Token(Token::Whitespace) => continue,
            ComponentValue::Token(Token::Ident(s)) => {
                // Flush any prior name with the default value.
                if let Some(prior) = name.take() {
                    out.push(CounterOp {
                        name: prior,
                        value: default,
                    });
                }
                if s.eq_ignore_ascii_case("none") {
                    return Vec::new();
                }
                name = Some(s.to_string());
            },
            ComponentValue::Token(Token::Number(n)) if n.is_integer => {
                if let Some(prior) = name.take() {
                    out.push(CounterOp {
                        name: prior,
                        value: n.value as i32,
                    });
                }
            },
            _ => {},
        }
    }
    if let Some(prior) = name {
        out.push(CounterOp {
            name: prior,
            value: default,
        });
    }
    out
}

// ─────────────────────────────────────────────────────────────────────
// CounterState — document-order tracking
// ─────────────────────────────────────────────────────────────────────

/// One nested scope in the counter stack. Each enter() pushes a new
/// scope (corresponds to entering a DOM element); each leave() pops
/// it. `counter-reset` creates a counter in the current scope;
/// `counter-increment` walks outward and modifies the nearest one.
#[derive(Debug, Default, Clone)]
struct Scope {
    counters: HashMap<String, i32>,
}

/// Mutable counter state that the layout walker carries while walking
/// the DOM in source order.
#[derive(Debug, Default, Clone)]
pub struct CounterState {
    stack: Vec<Scope>,
}

impl CounterState {
    /// Create a fresh state with one root scope (so counters defined
    /// before any explicit reset still have a home).
    pub fn new() -> Self {
        Self {
            stack: vec![Scope::default()],
        }
    }

    /// Push a new scope when entering an element.
    pub fn enter(&mut self) {
        self.stack.push(Scope::default());
    }

    /// Pop the current scope when leaving an element.
    pub fn leave(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Apply a `counter-reset` op in the current scope.
    pub fn apply_reset(&mut self, op: &CounterOp) {
        let top = self
            .stack
            .last_mut()
            .expect("CounterState always has at least one scope");
        top.counters.insert(op.name.clone(), op.value);
    }

    /// Apply a `counter-increment`. Walks scopes outward to find an
    /// existing counter; if none, creates one in the root scope.
    pub fn apply_increment(&mut self, op: &CounterOp) {
        for scope in self.stack.iter_mut().rev() {
            if let Some(v) = scope.counters.get_mut(&op.name) {
                *v += op.value;
                return;
            }
        }
        // Implicit reset to 0, then increment.
        let root = &mut self.stack[0];
        root.counters.insert(op.name.clone(), op.value);
    }

    /// Apply `counter-set`. Like increment but absolute — sets the
    /// nearest counter to `value`, creating one in the current scope
    /// if absent.
    pub fn apply_set(&mut self, op: &CounterOp) {
        for scope in self.stack.iter_mut().rev() {
            if let Some(v) = scope.counters.get_mut(&op.name) {
                *v = op.value;
                return;
            }
        }
        let top = self.stack.last_mut().unwrap();
        top.counters.insert(op.name.clone(), op.value);
    }

    /// Current value of the nearest counter named `name`. Returns 0
    /// if no such counter is in scope (matches spec — counter-using
    /// pseudo-elements still render `0`).
    pub fn counter(&self, name: &str) -> i32 {
        for scope in self.stack.iter().rev() {
            if let Some(v) = scope.counters.get(name) {
                return *v;
            }
        }
        0
    }

    /// All values for `name` from outermost to innermost — used by
    /// `counters(name, sep)`.
    pub fn counters(&self, name: &str) -> Vec<i32> {
        self.stack
            .iter()
            .filter_map(|s| s.counters.get(name).copied())
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Content evaluation
// ─────────────────────────────────────────────────────────────────────

/// Render a parsed `content:` list to the displayed string. The
/// `attr_lookup` closure resolves `attr(name)` references against the
/// host element.
pub fn evaluate_content(
    content: &[Content],
    state: &CounterState,
    attr_lookup: impl Fn(&str) -> Option<String>,
) -> String {
    let mut out = String::new();
    for item in content {
        match item {
            Content::Str(s) => out.push_str(s),
            Content::Counter { name, style } => {
                let n = state.counter(name);
                out.push_str(&style.render(n));
            },
            Content::Counters {
                name,
                separator,
                style,
            } => {
                let parts: Vec<String> = state
                    .counters(name)
                    .into_iter()
                    .map(|n| style.render(n))
                    .collect();
                out.push_str(&parts.join(separator));
            },
            Content::Attr { name } => {
                if let Some(v) = attr_lookup(name) {
                    out.push_str(&v);
                }
            },
            // English defaults; v0.3.36 reads the `quotes` property.
            Content::OpenQuote => out.push('“'),
            Content::CloseQuote => out.push('”'),
        }
    }
    out
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

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::parser::{parse_stylesheet, Rule};

    fn first_decl_value(css: &'static str, property: &'static str) -> Vec<ComponentValue<'static>> {
        let ss: &'static _ = Box::leak(Box::new(parse_stylesheet(css).unwrap()));
        let r = match &ss.rules[0] {
            Rule::Qualified(q) => q,
            _ => panic!(),
        };
        r.declarations
            .iter()
            .find(|d| d.name == property)
            .unwrap()
            .value
            .clone()
    }

    // ---- ListStyle::render ----------------------------------------

    #[test]
    fn decimal_render() {
        assert_eq!(ListStyle::Decimal.render(5), "5");
    }

    #[test]
    fn decimal_leading_zero_pads() {
        assert_eq!(ListStyle::DecimalLeadingZero.render(7), "07");
    }

    #[test]
    fn lower_roman_render() {
        assert_eq!(ListStyle::LowerRoman.render(4), "iv");
        assert_eq!(ListStyle::LowerRoman.render(9), "ix");
        assert_eq!(ListStyle::LowerRoman.render(40), "xl");
        assert_eq!(ListStyle::LowerRoman.render(1994), "mcmxciv");
    }

    #[test]
    fn lower_alpha_wraps() {
        assert_eq!(ListStyle::LowerAlpha.render(1), "a");
        assert_eq!(ListStyle::LowerAlpha.render(26), "z");
        assert_eq!(ListStyle::LowerAlpha.render(27), "aa");
        assert_eq!(ListStyle::LowerAlpha.render(28), "ab");
    }

    #[test]
    fn upper_alpha_wraps() {
        assert_eq!(ListStyle::UpperAlpha.render(1), "A");
        assert_eq!(ListStyle::UpperAlpha.render(28), "AB");
    }

    // ---- Content parser --------------------------------------------

    #[test]
    fn content_string_only() {
        let v = first_decl_value(r#"p::before { content: "Section "; }"#, "content");
        let c = parse_content(&v).unwrap();
        assert_eq!(c, vec![Content::Str("Section ".into())]);
    }

    #[test]
    fn content_none_returns_none() {
        let v = first_decl_value("p::before { content: none; }", "content");
        assert!(parse_content(&v).is_none());
    }

    #[test]
    fn content_counter_default_decimal() {
        let v = first_decl_value(r#"h2::before { content: counter(chapter) ". "; }"#, "content");
        let c = parse_content(&v).unwrap();
        assert_eq!(
            c,
            vec![
                Content::Counter {
                    name: "chapter".into(),
                    style: ListStyle::Decimal,
                },
                Content::Str(". ".into()),
            ]
        );
    }

    #[test]
    fn content_counter_with_style() {
        let v = first_decl_value(
            r#"h2::before { content: counter(chapter, lower-roman); }"#,
            "content",
        );
        let c = parse_content(&v).unwrap();
        assert_eq!(
            c,
            vec![Content::Counter {
                name: "chapter".into(),
                style: ListStyle::LowerRoman,
            }]
        );
    }

    #[test]
    fn content_counters_with_separator() {
        let v = first_decl_value(r#"h3::before { content: counters(section, "."); }"#, "content");
        let c = parse_content(&v).unwrap();
        assert_eq!(
            c,
            vec![Content::Counters {
                name: "section".into(),
                separator: ".".into(),
                style: ListStyle::Decimal,
            }]
        );
    }

    #[test]
    fn content_attr() {
        let v = first_decl_value(r#"a::after { content: attr(href); }"#, "content");
        let c = parse_content(&v).unwrap();
        assert_eq!(
            c,
            vec![Content::Attr {
                name: "href".into()
            }]
        );
    }

    #[test]
    fn content_quotes() {
        let v = first_decl_value(r#"q::before { content: open-quote; }"#, "content");
        let c = parse_content(&v).unwrap();
        assert_eq!(c, vec![Content::OpenQuote]);
    }

    // ---- counter-reset / -increment parsing -----------------------

    #[test]
    fn parse_counter_reset_default_zero() {
        let v = first_decl_value("body { counter-reset: chapter; }", "counter-reset");
        let ops = parse_counter_ops(&v, 0);
        assert_eq!(
            ops,
            vec![CounterOp {
                name: "chapter".into(),
                value: 0,
            }]
        );
    }

    #[test]
    fn parse_counter_reset_with_value() {
        let v = first_decl_value("body { counter-reset: chapter 5; }", "counter-reset");
        let ops = parse_counter_ops(&v, 0);
        assert_eq!(
            ops,
            vec![CounterOp {
                name: "chapter".into(),
                value: 5,
            }]
        );
    }

    #[test]
    fn parse_counter_reset_multiple() {
        let v = first_decl_value("body { counter-reset: chapter 0 section 1; }", "counter-reset");
        let ops = parse_counter_ops(&v, 0);
        assert_eq!(
            ops,
            vec![
                CounterOp {
                    name: "chapter".into(),
                    value: 0,
                },
                CounterOp {
                    name: "section".into(),
                    value: 1,
                },
            ]
        );
    }

    #[test]
    fn parse_counter_increment_default_one() {
        let v = first_decl_value("h1 { counter-increment: chapter; }", "counter-increment");
        let ops = parse_counter_ops(&v, 1);
        assert_eq!(ops[0].value, 1);
    }

    // ---- CounterState behaviour -----------------------------------

    #[test]
    fn counter_state_basic_increment() {
        let mut st = CounterState::new();
        st.apply_reset(&CounterOp {
            name: "n".into(),
            value: 0,
        });
        assert_eq!(st.counter("n"), 0);
        st.apply_increment(&CounterOp {
            name: "n".into(),
            value: 1,
        });
        assert_eq!(st.counter("n"), 1);
        st.apply_increment(&CounterOp {
            name: "n".into(),
            value: 2,
        });
        assert_eq!(st.counter("n"), 3);
    }

    #[test]
    fn counter_state_nested_scopes() {
        let mut st = CounterState::new();
        // Outer
        st.apply_reset(&CounterOp {
            name: "x".into(),
            value: 0,
        });
        st.apply_increment(&CounterOp {
            name: "x".into(),
            value: 1,
        });
        assert_eq!(st.counter("x"), 1);
        // Inner shadowed reset
        st.enter();
        st.apply_reset(&CounterOp {
            name: "x".into(),
            value: 100,
        });
        assert_eq!(st.counter("x"), 100);
        // counters() sees both levels
        assert_eq!(st.counters("x"), vec![1, 100]);
        st.leave();
        // Outer restored
        assert_eq!(st.counter("x"), 1);
    }

    #[test]
    fn counter_state_set_overwrites() {
        let mut st = CounterState::new();
        st.apply_reset(&CounterOp {
            name: "n".into(),
            value: 0,
        });
        st.apply_set(&CounterOp {
            name: "n".into(),
            value: 42,
        });
        assert_eq!(st.counter("n"), 42);
    }

    #[test]
    fn counter_state_unknown_returns_zero() {
        let st = CounterState::new();
        assert_eq!(st.counter("nope"), 0);
    }

    // ---- evaluate_content end-to-end ------------------------------

    #[test]
    fn evaluate_counter_in_content() {
        let mut st = CounterState::new();
        st.apply_reset(&CounterOp {
            name: "chapter".into(),
            value: 2,
        });
        let content = parse_content(&first_decl_value(
            r#"h2::before { content: "Chapter " counter(chapter) ". "; }"#,
            "content",
        ))
        .unwrap();
        let s = evaluate_content(&content, &st, |_| None);
        assert_eq!(s, "Chapter 2. ");
    }

    #[test]
    fn evaluate_counters_with_separator() {
        let mut st = CounterState::new();
        st.apply_reset(&CounterOp {
            name: "section".into(),
            value: 1,
        });
        st.enter();
        st.apply_reset(&CounterOp {
            name: "section".into(),
            value: 2,
        });
        let content = parse_content(&first_decl_value(
            r#"h3::before { content: counters(section, "."); }"#,
            "content",
        ))
        .unwrap();
        let s = evaluate_content(&content, &st, |_| None);
        assert_eq!(s, "1.2");
    }

    #[test]
    fn evaluate_attr() {
        let st = CounterState::new();
        let content =
            parse_content(&first_decl_value(r#"a::after { content: attr(href); }"#, "content"))
                .unwrap();
        let s = evaluate_content(&content, &st, |name| {
            if name == "href" {
                Some("https://example.com".to_string())
            } else {
                None
            }
        });
        assert_eq!(s, "https://example.com");
    }
}
