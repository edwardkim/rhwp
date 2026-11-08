//! CSS Selectors Level 3 + Level 4 subset — parser + specificity.
//!
//! Consumes the prelude `Vec<ComponentValue>` produced by [`super::parser`]
//! and turns it into a structured [`SelectorList`] that the matcher
//! (CSS-4) walks. Specificity is computed at parse time per CSS
//! Selectors L3 §16 — three counts (id, class/attr/pseudo-class,
//! type/pseudo-element) packed into a single `u32` for fast cascade
//! sorting.
//!
//! v0.3.35 supported surface:
//!
//! - Simple selectors: type, universal `*`, class `.x`, id `#x`,
//!   attribute `[a]` `[a=v]` `[a~=v]` `[a|=v]` `[a^=v]` `[a$=v]`
//!   `[a*=v]` with optional `i`/`s` case flag
//! - Combinators: descendant (whitespace), child `>`,
//!   next-sibling `+`, subsequent-sibling `~`
//! - Structural pseudo-classes: `:root`, `:first-child`, `:last-child`,
//!   `:only-child`, `:first-of-type`, `:last-of-type`, `:only-of-type`,
//!   `:nth-child(An+B)`, `:nth-last-child`, `:nth-of-type`,
//!   `:nth-last-of-type`, `:empty`
//! - Logical pseudo-classes: `:is()`, `:where()`, `:not()`, `:has()`
//! - Pseudo-elements: `::before`, `::after`, `::first-line`,
//!   `::first-letter`
//! - UA state pseudo-classes (`:hover`, `:focus`, `:visited`, …) parse
//!   correctly so author CSS doesn't error, but the matcher in CSS-4
//!   never matches them (no UA state in a paged-PDF pipeline)
//!
//! Out of scope (parses-and-ignores or rejects):
//! - `::part()`, `::slotted()` (no shadow DOM in HTML→PDF)
//! - `:dir()`, `:lang()` — accept-and-no-match
//! - Namespace selectors (`ns|*`) — rare in 2026 web content

use std::borrow::Cow;
use thiserror::Error;

use super::parser::ComponentValue;
use super::tokenizer::Token;

// ─────────────────────────────────────────────────────────────────────
// AST
// ─────────────────────────────────────────────────────────────────────

/// Comma-separated list of complex selectors.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectorList {
    /// Each comma-separated alternative.
    pub selectors: Vec<ComplexSelector>,
}

/// A complex selector: a sequence of compound selectors joined by
/// combinators. Stored in matcher-friendly **right-to-left** order:
/// `compounds[0]` is the rightmost (the "subject" the engine matches
/// first), with `combinators[i]` describing the relation from
/// `compounds[i+1]` (further left) to `compounds[i]`.
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexSelector {
    /// Compound selectors in right-to-left order.
    pub compounds: Vec<CompoundSelector>,
    /// Combinators in right-to-left order. `combinators.len() ==
    /// compounds.len() - 1`.
    pub combinators: Vec<Combinator>,
    /// Cached specificity for the whole complex selector.
    pub specificity: Specificity,
}

/// One compound selector: an optional type/universal selector plus
/// any number of subclass / pseudo-class / pseudo-element selectors.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompoundSelector {
    /// Type or universal element selector (the "subject" of the
    /// compound), if any.
    pub element: Option<ElementSelector>,
    /// Class, id, attribute, pseudo-class, pseudo-element selectors —
    /// any combination, in source order.
    pub subclasses: Vec<SubclassSelector>,
}

/// Type or universal selector.
#[derive(Debug, Clone, PartialEq)]
pub enum ElementSelector {
    /// `*`.
    Universal,
    /// `tagname` (lowercase).
    Type(String),
}

/// One non-element simple selector inside a compound.
#[derive(Debug, Clone, PartialEq)]
pub enum SubclassSelector {
    /// `.class`.
    Class(String),
    /// `#id`.
    Id(String),
    /// `[attr]`, `[attr=val]`, `[attr|=val]`, …
    Attribute(AttributeSelector),
    /// `:hover`, `:first-child`, `:nth-child(2n+1)`, `:not(...)`, …
    PseudoClass(PseudoClass),
    /// `::before`, `::first-line`, …
    PseudoElement(PseudoElement),
}

/// `[name op? value? flag?]`.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSelector {
    /// Attribute name (lowercased).
    pub name: String,
    /// Match operator, or `None` for `[name]` presence-only.
    pub op: Option<AttributeOp>,
    /// Comparison value.
    pub value: Option<String>,
    /// Case-sensitivity flag from `[name=val i]` / `s`.
    pub case: AttributeCase,
}

/// Attribute matching operators per CSS Selectors §6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOp {
    /// `[a=v]` exact match.
    Equals,
    /// `[a~=v]` whitespace-list contains.
    Includes,
    /// `[a|=v]` equals v or starts with `v-`.
    DashMatch,
    /// `[a^=v]` starts with.
    Prefix,
    /// `[a$=v]` ends with.
    Suffix,
    /// `[a*=v]` contains.
    Substring,
}

/// Case-sensitivity flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttributeCase {
    /// Default: ASCII case-insensitive for HTML attributes per spec.
    /// The matcher decides; here we just say "no flag was supplied".
    #[default]
    Default,
    /// `i` flag — explicit ASCII case-insensitive.
    Insensitive,
    /// `s` flag — case-sensitive.
    Sensitive,
}

/// Pseudo-class.
#[derive(Debug, Clone, PartialEq)]
pub enum PseudoClass {
    /// `:root`.
    Root,
    /// `:empty`.
    Empty,
    /// `:first-child`.
    FirstChild,
    /// `:last-child`.
    LastChild,
    /// `:only-child`.
    OnlyChild,
    /// `:first-of-type`.
    FirstOfType,
    /// `:last-of-type`.
    LastOfType,
    /// `:only-of-type`.
    OnlyOfType,
    /// `:nth-child(An+B)`.
    NthChild(AnPlusB),
    /// `:nth-last-child(An+B)`.
    NthLastChild(AnPlusB),
    /// `:nth-of-type(An+B)`.
    NthOfType(AnPlusB),
    /// `:nth-last-of-type(An+B)`.
    NthLastOfType(AnPlusB),
    /// `:is(selector-list)`.
    Is(SelectorList),
    /// `:where(selector-list)` — same as :is but contributes 0 to
    /// specificity.
    Where(SelectorList),
    /// `:not(selector-list)`.
    Not(SelectorList),
    /// `:has(selector-list)`.
    Has(SelectorList),
    /// UA state / accept-and-noop family. Parsed but the matcher
    /// always returns false. Stored as the lowercased name.
    UaState(String),
    /// `:lang(...)`, `:dir(...)`, … — parsed but accept-and-noop.
    Functional {
        /// Name (lowercased).
        name: String,
    },
}

/// `:nth-child(An+B)` coefficient pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnPlusB {
    /// Step coefficient (the `A`).
    pub a: i32,
    /// Offset (the `B`).
    pub b: i32,
}

impl AnPlusB {
    /// `2n+1` aka `odd`.
    pub fn odd() -> Self {
        Self { a: 2, b: 1 }
    }
    /// `2n` aka `even`.
    pub fn even() -> Self {
        Self { a: 2, b: 0 }
    }
}

/// Pseudo-element.
#[derive(Debug, Clone, PartialEq)]
pub enum PseudoElement {
    /// `::before`.
    Before,
    /// `::after`.
    After,
    /// `::first-line`.
    FirstLine,
    /// `::first-letter`.
    FirstLetter,
    /// Anything else parses to this with the lowercased name. Matcher
    /// never matches; preserved so cascade walk doesn't crash.
    Other(String),
}

/// Combinator between two compound selectors. `Descendant` covers the
/// whitespace combinator (`a b`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// `a b`.
    Descendant,
    /// `a > b`.
    Child,
    /// `a + b`.
    NextSibling,
    /// `a ~ b`.
    SubsequentSibling,
}

/// CSS specificity packed into a u32 as `(id << 16) | (cls << 8) | typ`.
/// Sorts naturally — higher u32 wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Specificity(pub u32);

impl Specificity {
    /// Construct from the three components (saturating each at 255 to
    /// avoid overflow on pathological selectors).
    pub fn new(id: u32, cls: u32, typ: u32) -> Self {
        Self((id.min(255) << 16) | (cls.min(255) << 8) | typ.min(255))
    }
    /// `(id, class/attr/pseudo-class, type/pseudo-element)`.
    pub fn parts(self) -> (u32, u32, u32) {
        ((self.0 >> 16) & 0xFF, (self.0 >> 8) & 0xFF, self.0 & 0xFF)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────

/// Selector parse errors. The cascade treats these as "skip this rule".
#[derive(Debug, Clone, Error, PartialEq)]
pub enum SelectorParseError {
    /// Empty prelude.
    #[error("empty selector")]
    Empty,
    /// Unexpected token at the start of a selector.
    #[error("unexpected token at selector start")]
    UnexpectedStart,
    /// Attribute selector wasn't well-formed.
    #[error("malformed attribute selector")]
    BadAttribute,
    /// `:not()` / `:is()` / etc. with no inner selector.
    #[error("functional pseudo-class missing argument")]
    MissingFunctionalArg,
    /// Other malformed syntax — kept as a catch-all so callers don't
    /// have to match every variant.
    #[error("malformed selector: {0}")]
    Other(&'static str),
}

// ─────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────

/// Parse a comma-separated selector list out of a qualified rule's
/// prelude.
pub fn parse_selector_list(
    prelude: &[ComponentValue<'_>],
) -> Result<SelectorList, SelectorParseError> {
    let trimmed = trim_ws(prelude);
    if trimmed.is_empty() {
        return Err(SelectorParseError::Empty);
    }
    let mut selectors = Vec::new();
    for chunk in split_top_level_commas(trimmed) {
        let trimmed_chunk = trim_ws(chunk);
        if trimmed_chunk.is_empty() {
            return Err(SelectorParseError::Empty);
        }
        selectors.push(parse_complex_selector(trimmed_chunk)?);
    }
    Ok(SelectorList { selectors })
}

// ─────────────────────────────────────────────────────────────────────
// Internal parser
// ─────────────────────────────────────────────────────────────────────

/// Parse a complex selector (compound combinator compound …) from a
/// pre-comma-split slice of component values.
fn parse_complex_selector(
    cvs: &[ComponentValue<'_>],
) -> Result<ComplexSelector, SelectorParseError> {
    // Walk left-to-right and record (compound, combinator-to-next).
    let mut compounds_ltr: Vec<CompoundSelector> = Vec::new();
    let mut combinators_ltr: Vec<Combinator> = Vec::new();

    let mut i = 0;
    while i < cvs.len() {
        // Skip leading whitespace; significant whitespace becomes the
        // descendant combinator only when between compounds.
        while i < cvs.len() && is_ws(&cvs[i]) {
            i += 1;
        }
        if i >= cvs.len() {
            break;
        }
        let (compound, end) = parse_compound_selector(&cvs[i..])?;
        compounds_ltr.push(compound);
        i += end;
        // Look ahead for combinator.
        let mut had_ws = false;
        while i < cvs.len() && is_ws(&cvs[i]) {
            had_ws = true;
            i += 1;
        }
        if i >= cvs.len() {
            break;
        }
        let comb = match &cvs[i] {
            ComponentValue::Token(Token::Delim('>')) => {
                i += 1;
                Combinator::Child
            },
            ComponentValue::Token(Token::Delim('+')) => {
                i += 1;
                Combinator::NextSibling
            },
            ComponentValue::Token(Token::Delim('~')) => {
                i += 1;
                Combinator::SubsequentSibling
            },
            _ if had_ws => Combinator::Descendant,
            _ => return Err(SelectorParseError::Other("missing combinator")),
        };
        combinators_ltr.push(comb);
    }

    if compounds_ltr.is_empty() {
        return Err(SelectorParseError::Empty);
    }

    let specificity = compute_specificity_ltr(&compounds_ltr);

    // Reverse to right-to-left storage.
    compounds_ltr.reverse();
    combinators_ltr.reverse();

    Ok(ComplexSelector {
        compounds: compounds_ltr,
        combinators: combinators_ltr,
        specificity,
    })
}

/// Parse one compound selector (an element selector + zero or more
/// subclass selectors). Returns the compound and the number of
/// component values consumed.
fn parse_compound_selector(
    cvs: &[ComponentValue<'_>],
) -> Result<(CompoundSelector, usize), SelectorParseError> {
    let mut compound = CompoundSelector::default();
    let mut i = 0;

    // Element selector (type or universal). Optional.
    if i < cvs.len() {
        match &cvs[i] {
            ComponentValue::Token(Token::Delim('*')) => {
                compound.element = Some(ElementSelector::Universal);
                i += 1;
            },
            ComponentValue::Token(Token::Ident(name)) => {
                compound.element = Some(ElementSelector::Type(name.to_ascii_lowercase()));
                i += 1;
            },
            _ => {},
        }
    }

    // Subclass selectors.
    while i < cvs.len() {
        match &cvs[i] {
            ComponentValue::Token(Token::Delim('.')) => {
                i += 1;
                if let Some(ComponentValue::Token(Token::Ident(name))) = cvs.get(i) {
                    compound
                        .subclasses
                        .push(SubclassSelector::Class(name.to_string()));
                    i += 1;
                } else {
                    return Err(SelectorParseError::Other(". without class name"));
                }
            },
            ComponentValue::Token(Token::Hash { value, is_id: true }) => {
                compound
                    .subclasses
                    .push(SubclassSelector::Id(value.to_string()));
                i += 1;
            },
            ComponentValue::Square(body) => {
                compound
                    .subclasses
                    .push(SubclassSelector::Attribute(parse_attribute(body)?));
                i += 1;
            },
            ComponentValue::Token(Token::Colon) => {
                i += 1;
                // ::pseudo-element vs :pseudo-class
                if matches!(cvs.get(i), Some(ComponentValue::Token(Token::Colon))) {
                    i += 1;
                    let (pe, used) = parse_pseudo_element(&cvs[i..])?;
                    compound
                        .subclasses
                        .push(SubclassSelector::PseudoElement(pe));
                    i += used;
                } else {
                    let (pc, used) = parse_pseudo_class(&cvs[i..])?;
                    compound.subclasses.push(SubclassSelector::PseudoClass(pc));
                    i += used;
                }
            },
            // Whitespace or combinator end the compound.
            ComponentValue::Token(Token::Whitespace)
            | ComponentValue::Token(Token::Delim('>'))
            | ComponentValue::Token(Token::Delim('+'))
            | ComponentValue::Token(Token::Delim('~'))
            | ComponentValue::Token(Token::Comma) => break,
            _ => break,
        }
    }

    if compound.element.is_none() && compound.subclasses.is_empty() {
        return Err(SelectorParseError::UnexpectedStart);
    }

    Ok((compound, i))
}

fn parse_attribute(body: &[ComponentValue<'_>]) -> Result<AttributeSelector, SelectorParseError> {
    let trimmed = trim_ws(body);
    if trimmed.is_empty() {
        return Err(SelectorParseError::BadAttribute);
    }
    // Expect: ident [ op string-or-ident [ flag ]? ]?
    let name = match &trimmed[0] {
        ComponentValue::Token(Token::Ident(s)) => s.to_ascii_lowercase(),
        _ => return Err(SelectorParseError::BadAttribute),
    };
    let mut i = 1;
    while i < trimmed.len() && is_ws(&trimmed[i]) {
        i += 1;
    }
    if i >= trimmed.len() {
        return Ok(AttributeSelector {
            name,
            op: None,
            value: None,
            case: AttributeCase::Default,
        });
    }
    // Operator
    let op = match (&trimmed[i], trimmed.get(i + 1)) {
        (ComponentValue::Token(Token::Delim('=')), _) => {
            i += 1;
            Some(AttributeOp::Equals)
        },
        (
            ComponentValue::Token(Token::Delim('~')),
            Some(ComponentValue::Token(Token::Delim('='))),
        ) => {
            i += 2;
            Some(AttributeOp::Includes)
        },
        (
            ComponentValue::Token(Token::Delim('|')),
            Some(ComponentValue::Token(Token::Delim('='))),
        ) => {
            i += 2;
            Some(AttributeOp::DashMatch)
        },
        (
            ComponentValue::Token(Token::Delim('^')),
            Some(ComponentValue::Token(Token::Delim('='))),
        ) => {
            i += 2;
            Some(AttributeOp::Prefix)
        },
        (
            ComponentValue::Token(Token::Delim('$')),
            Some(ComponentValue::Token(Token::Delim('='))),
        ) => {
            i += 2;
            Some(AttributeOp::Suffix)
        },
        (
            ComponentValue::Token(Token::Delim('*')),
            Some(ComponentValue::Token(Token::Delim('='))),
        ) => {
            i += 2;
            Some(AttributeOp::Substring)
        },
        _ => return Err(SelectorParseError::BadAttribute),
    };
    while i < trimmed.len() && is_ws(&trimmed[i]) {
        i += 1;
    }
    let value = match trimmed.get(i) {
        Some(ComponentValue::Token(Token::String(s))) => Some(s.to_string()),
        Some(ComponentValue::Token(Token::Ident(s))) => Some(s.to_string()),
        _ => return Err(SelectorParseError::BadAttribute),
    };
    i += 1;
    while i < trimmed.len() && is_ws(&trimmed[i]) {
        i += 1;
    }
    let case = match trimmed.get(i) {
        Some(ComponentValue::Token(Token::Ident(s))) if s.eq_ignore_ascii_case("i") => {
            AttributeCase::Insensitive
        },
        Some(ComponentValue::Token(Token::Ident(s))) if s.eq_ignore_ascii_case("s") => {
            AttributeCase::Sensitive
        },
        _ => AttributeCase::Default,
    };
    Ok(AttributeSelector {
        name,
        op,
        value,
        case,
    })
}

fn parse_pseudo_class(
    cvs: &[ComponentValue<'_>],
) -> Result<(PseudoClass, usize), SelectorParseError> {
    if cvs.is_empty() {
        return Err(SelectorParseError::MissingFunctionalArg);
    }
    match &cvs[0] {
        ComponentValue::Token(Token::Ident(name)) => {
            let lower = name.to_ascii_lowercase();
            let pc = match lower.as_str() {
                "root" => PseudoClass::Root,
                "empty" => PseudoClass::Empty,
                "first-child" => PseudoClass::FirstChild,
                "last-child" => PseudoClass::LastChild,
                "only-child" => PseudoClass::OnlyChild,
                "first-of-type" => PseudoClass::FirstOfType,
                "last-of-type" => PseudoClass::LastOfType,
                "only-of-type" => PseudoClass::OnlyOfType,
                "hover" | "focus" | "focus-within" | "focus-visible" | "active" | "visited"
                | "link" | "target" | "checked" | "disabled" | "enabled" | "required"
                | "optional" | "valid" | "invalid" | "in-range" | "out-of-range" | "read-only"
                | "read-write" | "placeholder-shown" | "default" | "indeterminate" => {
                    PseudoClass::UaState(lower)
                },
                _ => PseudoClass::UaState(lower),
            };
            Ok((pc, 1))
        },
        ComponentValue::Function { name, body } => {
            let lower = name.to_ascii_lowercase();
            let pc = match lower.as_str() {
                "nth-child" => PseudoClass::NthChild(parse_an_plus_b(body)?),
                "nth-last-child" => PseudoClass::NthLastChild(parse_an_plus_b(body)?),
                "nth-of-type" => PseudoClass::NthOfType(parse_an_plus_b(body)?),
                "nth-last-of-type" => PseudoClass::NthLastOfType(parse_an_plus_b(body)?),
                "is" => PseudoClass::Is(parse_selector_list(body)?),
                "where" => PseudoClass::Where(parse_selector_list(body)?),
                "not" => PseudoClass::Not(parse_selector_list(body)?),
                "has" => PseudoClass::Has(parse_selector_list(body)?),
                _ => PseudoClass::Functional { name: lower },
            };
            Ok((pc, 1))
        },
        _ => Err(SelectorParseError::Other("expected pseudo-class name")),
    }
}

fn parse_pseudo_element(
    cvs: &[ComponentValue<'_>],
) -> Result<(PseudoElement, usize), SelectorParseError> {
    if cvs.is_empty() {
        return Err(SelectorParseError::MissingFunctionalArg);
    }
    match &cvs[0] {
        ComponentValue::Token(Token::Ident(name)) => {
            let lower = name.to_ascii_lowercase();
            let pe = match lower.as_str() {
                "before" => PseudoElement::Before,
                "after" => PseudoElement::After,
                "first-line" => PseudoElement::FirstLine,
                "first-letter" => PseudoElement::FirstLetter,
                _ => PseudoElement::Other(lower),
            };
            Ok((pe, 1))
        },
        ComponentValue::Function { name, .. } => {
            // ::part(), ::slotted() etc. — accept-and-noop.
            Ok((PseudoElement::Other(name.to_ascii_lowercase()), 1))
        },
        _ => Err(SelectorParseError::Other("expected pseudo-element name")),
    }
}

/// Parse the body of `:nth-*(...)` per CSS Syntax §6.6.3 — handles
/// `An+B`, `even`, `odd`, plain integers, signed offsets.
fn parse_an_plus_b(body: &[ComponentValue<'_>]) -> Result<AnPlusB, SelectorParseError> {
    let trimmed = trim_ws(body);
    if trimmed.is_empty() {
        return Err(SelectorParseError::MissingFunctionalArg);
    }
    // Easy keyword cases first.
    if trimmed.len() == 1 {
        if let ComponentValue::Token(Token::Ident(s)) = &trimmed[0] {
            if s.eq_ignore_ascii_case("odd") {
                return Ok(AnPlusB::odd());
            }
            if s.eq_ignore_ascii_case("even") {
                return Ok(AnPlusB::even());
            }
            // Bare 'n' → 1n+0
            if s.eq_ignore_ascii_case("n") {
                return Ok(AnPlusB { a: 1, b: 0 });
            }
            if s.eq_ignore_ascii_case("-n") {
                return Ok(AnPlusB { a: -1, b: 0 });
            }
        }
    }

    // Reconstruct a string from the tokens and parse as An+B. Cheap,
    // robust, and we avoid chasing every form.
    let s = compose_an_plus_b_string(trimmed);
    parse_an_plus_b_string(&s).ok_or(SelectorParseError::Other("malformed An+B"))
}

fn compose_an_plus_b_string(cvs: &[ComponentValue<'_>]) -> String {
    let mut s = String::new();
    for cv in cvs {
        match cv {
            ComponentValue::Token(Token::Whitespace) => s.push(' '),
            ComponentValue::Token(Token::Number(n)) => {
                if n.is_integer {
                    s.push_str(&format!("{}", n.value as i64));
                } else {
                    s.push_str(&n.value.to_string());
                }
            },
            ComponentValue::Token(Token::Dimension { value, unit }) => {
                if value.is_integer {
                    s.push_str(&format!("{}{}", value.value as i64, unit));
                } else {
                    s.push_str(&format!("{}{}", value.value, unit));
                }
            },
            ComponentValue::Token(Token::Ident(id)) => s.push_str(id),
            ComponentValue::Token(Token::Delim(c)) => s.push(*c),
            _ => {},
        }
    }
    s
}

fn parse_an_plus_b_string(s: &str) -> Option<AnPlusB> {
    let s = s.replace(' ', "");
    // Match patterns:
    //   "even"/"odd"/"n"/"-n"/"+n"
    //   "(+/-)?<digits>n((+/-)<digits>)?"
    //   "(+/-)?<digits>" — pure offset
    let s_lower = s.to_lowercase();
    if s_lower == "even" {
        return Some(AnPlusB::even());
    }
    if s_lower == "odd" {
        return Some(AnPlusB::odd());
    }
    if let Some(idx) = s_lower.find('n') {
        let (a_part, b_part) = s_lower.split_at(idx);
        let a = match a_part {
            "" | "+" => 1,
            "-" => -1,
            other => other.parse::<i32>().ok()?,
        };
        let b_part = &b_part[1..]; // skip the 'n'
        let b = if b_part.is_empty() {
            0
        } else {
            b_part.parse::<i32>().ok()?
        };
        return Some(AnPlusB { a, b });
    }
    s_lower.parse::<i32>().ok().map(|b| AnPlusB { a: 0, b })
}

// ─────────────────────────────────────────────────────────────────────
// Specificity
// ─────────────────────────────────────────────────────────────────────

fn compute_specificity_ltr(compounds: &[CompoundSelector]) -> Specificity {
    let mut id = 0u32;
    let mut cls = 0u32;
    let mut typ = 0u32;
    for c in compounds {
        match &c.element {
            Some(ElementSelector::Type(_)) => typ += 1,
            Some(ElementSelector::Universal) | None => {},
        }
        for sub in &c.subclasses {
            match sub {
                SubclassSelector::Id(_) => id += 1,
                SubclassSelector::Class(_) | SubclassSelector::Attribute(_) => cls += 1,
                SubclassSelector::PseudoClass(pc) => match pc {
                    // :where contributes 0
                    PseudoClass::Where(_) => {},
                    // :is takes the highest specificity inside
                    PseudoClass::Is(list) | PseudoClass::Not(list) | PseudoClass::Has(list) => {
                        let max = list
                            .selectors
                            .iter()
                            .map(|s| s.specificity)
                            .max()
                            .unwrap_or(Specificity(0));
                        let (i, c, t) = max.parts();
                        id += i;
                        cls += c;
                        typ += t;
                    },
                    _ => cls += 1,
                },
                SubclassSelector::PseudoElement(_) => typ += 1,
            }
        }
    }
    Specificity::new(id, cls, typ)
}

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

fn is_ws(cv: &ComponentValue<'_>) -> bool {
    matches!(cv, ComponentValue::Token(Token::Whitespace))
}

fn trim_ws<'a, 'i>(cvs: &'a [ComponentValue<'i>]) -> &'a [ComponentValue<'i>] {
    let mut start = 0;
    while start < cvs.len() && is_ws(&cvs[start]) {
        start += 1;
    }
    let mut end = cvs.len();
    while end > start && is_ws(&cvs[end - 1]) {
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
    // Strip the implicit empty trailing slice when input ends with a
    // comma (forgiving — accept trailing commas).
    if out.last().map(|s| trim_ws(s).is_empty()).unwrap_or(false) {
        out.pop();
    }
    out
}

// Re-export Cow for callers that work with the AST. Currently unused
// directly but kept to make the API surface uniform with parser.rs.
#[allow(dead_code)]
type _Unused<'i> = Cow<'i, str>;

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::parser::{parse_stylesheet, Rule};

    fn parse_first(rule_src: &str) -> SelectorList {
        let ss = parse_stylesheet(rule_src).unwrap();
        let r = match &ss.rules[0] {
            Rule::Qualified(q) => q,
            _ => panic!("expected qualified rule"),
        };
        parse_selector_list(&r.prelude).expect("selector parse")
    }

    fn first_complex(rule_src: &str) -> ComplexSelector {
        parse_first(rule_src).selectors.into_iter().next().unwrap()
    }

    #[test]
    fn type_selector() {
        let cs = first_complex("body {}");
        assert_eq!(cs.compounds.len(), 1);
        assert_eq!(cs.compounds[0].element, Some(ElementSelector::Type("body".into())));
        assert_eq!(cs.specificity, Specificity::new(0, 0, 1));
    }

    #[test]
    fn universal_selector() {
        let cs = first_complex("* {}");
        assert_eq!(cs.compounds[0].element, Some(ElementSelector::Universal));
        assert_eq!(cs.specificity, Specificity::new(0, 0, 0));
    }

    #[test]
    fn class_selector_specificity() {
        let cs = first_complex(".foo {}");
        assert!(matches!(
            cs.compounds[0].subclasses[0],
            SubclassSelector::Class(ref s) if s == "foo"
        ));
        assert_eq!(cs.specificity, Specificity::new(0, 1, 0));
    }

    #[test]
    fn id_selector_specificity() {
        let cs = first_complex("#bar {}");
        assert!(matches!(
            cs.compounds[0].subclasses[0],
            SubclassSelector::Id(ref s) if s == "bar"
        ));
        assert_eq!(cs.specificity, Specificity::new(1, 0, 0));
    }

    #[test]
    fn compound_selector_div_dot_foo() {
        let cs = first_complex("div.foo {}");
        assert_eq!(cs.compounds.len(), 1);
        assert_eq!(cs.compounds[0].element, Some(ElementSelector::Type("div".into())));
        assert_eq!(cs.compounds[0].subclasses.len(), 1);
        assert_eq!(cs.specificity, Specificity::new(0, 1, 1));
    }

    #[test]
    fn descendant_combinator() {
        let cs = first_complex("ul li {}");
        assert_eq!(cs.compounds.len(), 2);
        assert_eq!(cs.combinators, vec![Combinator::Descendant]);
        // Stored right-to-left: compounds[0] is li (rightmost).
        assert_eq!(cs.compounds[0].element, Some(ElementSelector::Type("li".into())));
        assert_eq!(cs.compounds[1].element, Some(ElementSelector::Type("ul".into())));
    }

    #[test]
    fn child_combinator() {
        let cs = first_complex("nav > a {}");
        assert_eq!(cs.combinators, vec![Combinator::Child]);
    }

    #[test]
    fn next_sibling_combinator() {
        let cs = first_complex("h1 + p {}");
        assert_eq!(cs.combinators, vec![Combinator::NextSibling]);
    }

    #[test]
    fn subsequent_sibling_combinator() {
        let cs = first_complex("h1 ~ p {}");
        assert_eq!(cs.combinators, vec![Combinator::SubsequentSibling]);
    }

    #[test]
    fn comma_separated_list() {
        let list = parse_first("h1, h2, h3 {}");
        assert_eq!(list.selectors.len(), 3);
    }

    #[test]
    fn attribute_presence() {
        let cs = first_complex("[disabled] {}");
        let attr = match &cs.compounds[0].subclasses[0] {
            SubclassSelector::Attribute(a) => a,
            _ => panic!(),
        };
        assert_eq!(attr.name, "disabled");
        assert!(attr.op.is_none());
    }

    #[test]
    fn attribute_equals() {
        let cs = first_complex(r#"[href="https://x"] {}"#);
        let attr = match &cs.compounds[0].subclasses[0] {
            SubclassSelector::Attribute(a) => a,
            _ => panic!(),
        };
        assert_eq!(attr.op, Some(AttributeOp::Equals));
        assert_eq!(attr.value.as_deref(), Some("https://x"));
    }

    #[test]
    fn attribute_dash_match() {
        let cs = first_complex(r#"[lang|="en"] {}"#);
        let attr = match &cs.compounds[0].subclasses[0] {
            SubclassSelector::Attribute(a) => a,
            _ => panic!(),
        };
        assert_eq!(attr.op, Some(AttributeOp::DashMatch));
    }

    #[test]
    fn attribute_with_case_flag() {
        let cs = first_complex(r#"[type="email" i] {}"#);
        let attr = match &cs.compounds[0].subclasses[0] {
            SubclassSelector::Attribute(a) => a,
            _ => panic!(),
        };
        assert_eq!(attr.case, AttributeCase::Insensitive);
    }

    #[test]
    fn pseudo_class_first_child() {
        let cs = first_complex(":first-child {}");
        match &cs.compounds[0].subclasses[0] {
            SubclassSelector::PseudoClass(PseudoClass::FirstChild) => {},
            other => panic!("expected first-child, got {other:?}"),
        }
        // :first-child is class-tier specificity (0,1,0).
        assert_eq!(cs.specificity, Specificity::new(0, 1, 0));
    }

    #[test]
    fn pseudo_class_nth_child() {
        let cs = first_complex(":nth-child(2n+1) {}");
        match &cs.compounds[0].subclasses[0] {
            SubclassSelector::PseudoClass(PseudoClass::NthChild(AnPlusB { a: 2, b: 1 })) => {},
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn pseudo_class_nth_keywords() {
        let odd = first_complex(":nth-child(odd) {}");
        match &odd.compounds[0].subclasses[0] {
            SubclassSelector::PseudoClass(PseudoClass::NthChild(AnPlusB { a: 2, b: 1 })) => {},
            other => panic!("got {other:?}"),
        }
        let even = first_complex(":nth-child(even) {}");
        match &even.compounds[0].subclasses[0] {
            SubclassSelector::PseudoClass(PseudoClass::NthChild(AnPlusB { a: 2, b: 0 })) => {},
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn pseudo_element_before() {
        let cs = first_complex("p::before {}");
        match &cs.compounds[0].subclasses[0] {
            SubclassSelector::PseudoElement(PseudoElement::Before) => {},
            other => panic!("got {other:?}"),
        }
        // p (1 type) + ::before (1 pseudo-element, type-tier) = (0,0,2)
        assert_eq!(cs.specificity, Specificity::new(0, 0, 2));
    }

    #[test]
    fn pseudo_class_is_takes_max_inner_specificity() {
        let cs = first_complex(":is(#a, .b, span) {}");
        // Highest is #a → (1, 0, 0).
        assert_eq!(cs.specificity, Specificity::new(1, 0, 0));
    }

    #[test]
    fn pseudo_class_where_contributes_zero() {
        let cs = first_complex(":where(#a, .b) {}");
        assert_eq!(cs.specificity, Specificity::new(0, 0, 0));
    }

    #[test]
    fn pseudo_class_not() {
        let cs = first_complex("p:not(.lead) {}");
        // p type (1) + :not(.lead) → max(class) = 1.
        assert_eq!(cs.specificity, Specificity::new(0, 1, 1));
    }

    #[test]
    fn pseudo_class_has_basic() {
        let cs = first_complex("article:has(h1) {}");
        // article + has(h1) → (0, 0, 2)
        assert_eq!(cs.specificity, Specificity::new(0, 0, 2));
    }

    #[test]
    fn ua_state_pseudo_classes_parse() {
        // These resolve to UaState; matcher will never match.
        for s in [":hover {}", ":focus {}", ":visited {}", ":checked {}"] {
            let cs = first_complex(s);
            match &cs.compounds[0].subclasses[0] {
                SubclassSelector::PseudoClass(PseudoClass::UaState(_)) => {},
                other => panic!("got {other:?}"),
            }
        }
    }

    #[test]
    fn long_chain_specificity() {
        // body div#x.a.b > p:first-of-type
        // ids: 1, classes: 2 + 1 = 3, types: 1 + 1 + 1 = 3
        let cs = first_complex("body div#x.a.b > p:first-of-type {}");
        assert_eq!(cs.specificity, Specificity::new(1, 3, 3));
    }

    #[test]
    fn empty_selector_errors() {
        let res = parse_selector_list(&[]);
        assert!(matches!(res, Err(SelectorParseError::Empty)));
    }
}
