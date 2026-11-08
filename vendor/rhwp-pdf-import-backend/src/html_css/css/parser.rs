//! CSS parser — tokens (CSS-1) → stylesheet AST (CSS-2).
//!
//! Implements CSS Syntax Module Level 3 §5 (parsing) — specifically:
//! "consume a list of rules", "consume a qualified rule", "consume an
//! at-rule", "consume a declaration", and "consume a component value".
//!
//! The output is a `Stylesheet` of `Rule`s. Each rule preserves its
//! prelude as a `Vec<ComponentValue>` so the selector parser (CSS-3)
//! and at-rule handlers (CSS-9) can re-tokenize subtrees on demand —
//! this matches how `cssparser` and Stylo divide labour.
//!
//! Recovery model per spec: parsing is **forgiving**. Unbalanced
//! brackets balance themselves at the next rule boundary, unknown
//! tokens get skipped, and the parser never panics on user input.

use std::borrow::Cow;

use super::tokenizer::{tokenize, SourceLocation, Token, TokenizerError};

// ─────────────────────────────────────────────────────────────────────
// AST
// ─────────────────────────────────────────────────────────────────────

/// Top-level parsed stylesheet.
#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet<'i> {
    /// Rules in source order.
    pub rules: Vec<Rule<'i>>,
}

/// One rule at any nesting level.
#[derive(Debug, Clone, PartialEq)]
pub enum Rule<'i> {
    /// `selector { decl; decl }` — the bread-and-butter style rule.
    Qualified(QualifiedRule<'i>),
    /// `@name prelude;` or `@name prelude { ... }`.
    AtRule(AtRule<'i>),
}

/// Style rule.
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedRule<'i> {
    /// The component values that came before `{`. The selector parser
    /// (CSS-3) re-walks these.
    pub prelude: Vec<ComponentValue<'i>>,
    /// Declarations inside the `{ ... }` block. Forgiving — invalid
    /// declarations are dropped during parse with a warning we emit
    /// via the tracing layer (TODO: hook up tracing in CSS-9 so
    /// downstream callers can surface bad CSS).
    pub declarations: Vec<Declaration<'i>>,
    /// Where the rule started — useful for diagnostics.
    pub location: SourceLocation,
}

/// `@name prelude [ { body } | ; ]`.
#[derive(Debug, Clone, PartialEq)]
pub struct AtRule<'i> {
    /// Name without the leading `@` (e.g. `media`, `page`, `font-face`).
    pub name: Cow<'i, str>,
    /// Component values between the at-keyword and the block / semicolon.
    pub prelude: Vec<ComponentValue<'i>>,
    /// Block contents; `None` means the at-rule terminated with `;`.
    pub block: Option<AtRuleBlock<'i>>,
    /// Source location of the leading at-keyword.
    pub location: SourceLocation,
}

/// At-rule block contents. Different at-rules accept different bodies:
/// `@media`, `@supports`, `@layer` carry nested rules; `@page`,
/// `@font-face`, `@counter-style`, `@property` carry declarations.
/// At parse time we don't know which kind we're looking at (the
/// at-rule handler decides) so we store both possibilities — the
/// raw component-value soup and a best-effort split. CSS-9 picks the
/// right one per at-rule name.
#[derive(Debug, Clone, PartialEq)]
pub struct AtRuleBlock<'i> {
    /// All component values inside `{ ... }`, in source order.
    pub raw: Vec<ComponentValue<'i>>,
}

/// Declaration: `property: value !important?;`.
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration<'i> {
    /// Property name, without leading whitespace or trailing colon.
    pub name: Cow<'i, str>,
    /// Component values that comprised the value. Property-specific
    /// parsers (CSS-8) re-walk these.
    pub value: Vec<ComponentValue<'i>>,
    /// Whether the declaration carried `!important`.
    pub important: bool,
    /// Source location of the property name.
    pub location: SourceLocation,
}

/// CSS component value — the unit on which the parser splits a value.
/// Per CSS Syntax §5.4.7, this is either a preserved token, a
/// function call, or a balanced block.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue<'i> {
    /// Plain token (any non-bracket, non-function token).
    Token(Token<'i>),
    /// `name(...)`. Body is the values between `(` and the matching `)`.
    Function {
        /// Function name (without trailing `(`).
        name: Cow<'i, str>,
        /// Arguments as a flat component-value list. Comma separators
        /// are preserved as `Token::Comma` so callers can split.
        body: Vec<ComponentValue<'i>>,
    },
    /// `( ... )` simple block.
    Parens(Vec<ComponentValue<'i>>),
    /// `[ ... ]` simple block (selectors use this for attribute
    /// matchers; values can use it for grid-template-areas etc.).
    Square(Vec<ComponentValue<'i>>),
    /// `{ ... }` simple block (used inside function calls in some
    /// experimental syntaxes; v0.3.35 just preserves it for forward
    /// compatibility).
    Curly(Vec<ComponentValue<'i>>),
}

// ─────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────

/// Parse a CSS source string into a [`Stylesheet`].
///
/// Tokenization errors propagate (currently unreachable — the
/// tokenizer's only error variant is a strict-mode hook). Parse-level
/// problems (invalid declarations, unbalanced brackets) are recovered
/// silently per the CSS spec.
pub fn parse_stylesheet(input: &str) -> Result<Stylesheet<'_>, TokenizerError> {
    let tokens = tokenize(input)?;
    let mut p = Parser::new(tokens);
    let rules = p.consume_list_of_rules(/* top_level = */ true);
    Ok(Stylesheet { rules })
}

/// Parse a single declaration list (the body of a `style="..."`
/// attribute, or the body of a rule that the caller already extracted).
/// Useful for HTML inline styles in HTML-2.
pub fn parse_declaration_list(input: &str) -> Result<Vec<Declaration<'_>>, TokenizerError> {
    let tokens = tokenize(input)?;
    let mut p = Parser::new(tokens);
    Ok(p.consume_list_of_declarations())
}

// ─────────────────────────────────────────────────────────────────────
// Parser driver — closely follows CSS Syntax §5
// ─────────────────────────────────────────────────────────────────────

struct Parser<'i> {
    tokens: Vec<(Token<'i>, SourceLocation)>,
    pos: usize,
}

impl<'i> Parser<'i> {
    fn new(tokens: Vec<(Token<'i>, SourceLocation)>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token<'i> {
        self.tokens
            .get(self.pos)
            .map(|(t, _)| t)
            .unwrap_or(&Token::Eof)
    }

    fn peek_loc(&self) -> SourceLocation {
        self.tokens
            .get(self.pos)
            .map(|(_, l)| *l)
            .unwrap_or_else(SourceLocation::start)
    }

    /// Consume one token, returning it owned.
    fn next_token(&mut self) -> (Token<'i>, SourceLocation) {
        if let Some(item) = self.tokens.get(self.pos).cloned() {
            self.pos += 1;
            item
        } else {
            (Token::Eof, SourceLocation::start())
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Token::Whitespace) {
            self.pos += 1;
        }
    }

    /// CSS Syntax §5.4.1 — consume a list of rules.
    fn consume_list_of_rules(&mut self, top_level: bool) -> Vec<Rule<'i>> {
        let mut out = Vec::new();
        loop {
            match self.peek() {
                Token::Eof => return out,
                Token::Whitespace => {
                    self.pos += 1;
                },
                Token::Cdo | Token::Cdc => {
                    if top_level {
                        // Per spec — these are tolerated at top level
                        // (legacy HTML-comment markers around inline
                        // <style>).
                        self.pos += 1;
                    } else {
                        // Inside a block, treat as the start of a
                        // qualified rule.
                        if let Some(rule) = self.consume_qualified_rule() {
                            out.push(Rule::Qualified(rule));
                        }
                    }
                },
                Token::AtKeyword(_) => {
                    let at = self.consume_at_rule();
                    out.push(Rule::AtRule(at));
                },
                _ => {
                    if let Some(rule) = self.consume_qualified_rule() {
                        out.push(Rule::Qualified(rule));
                    }
                },
            }
        }
    }

    /// CSS Syntax §5.4.2 — consume an at-rule.
    fn consume_at_rule(&mut self) -> AtRule<'i> {
        let (name_tok, location) = self.next_token();
        let name = match name_tok {
            Token::AtKeyword(n) => n,
            _ => Cow::Borrowed(""), // unreachable in normal flow
        };

        let mut prelude: Vec<ComponentValue<'i>> = Vec::new();
        let block;
        loop {
            match self.peek() {
                Token::Semicolon => {
                    self.pos += 1;
                    block = None;
                    break;
                },
                Token::Eof => {
                    block = None;
                    break;
                },
                Token::LeftBrace => {
                    self.pos += 1;
                    let body = self.consume_simple_block_body(Token::RightBrace);
                    block = Some(AtRuleBlock { raw: body });
                    break;
                },
                _ => {
                    if let Some(cv) = self.consume_component_value() {
                        prelude.push(cv);
                    }
                },
            }
        }

        AtRule {
            name,
            prelude,
            block,
            location,
        }
    }

    /// CSS Syntax §5.4.3 — consume a qualified rule. Returns `None` if
    /// the rule never finds its `{`-block before EOF (then we just drop
    /// the partial prelude — forgiving recovery).
    fn consume_qualified_rule(&mut self) -> Option<QualifiedRule<'i>> {
        let location = self.peek_loc();
        let mut prelude: Vec<ComponentValue<'i>> = Vec::new();
        loop {
            match self.peek() {
                Token::Eof => return None,
                Token::LeftBrace => {
                    self.pos += 1;
                    let body_tokens = self.consume_simple_block_body(Token::RightBrace);
                    // Re-feed the block body into a sub-parser to get
                    // declarations.
                    let declarations = declarations_from_component_values(body_tokens);
                    return Some(QualifiedRule {
                        prelude,
                        declarations,
                        location,
                    });
                },
                _ => {
                    if let Some(cv) = self.consume_component_value() {
                        prelude.push(cv);
                    }
                },
            }
        }
    }

    /// CSS Syntax §5.4.4 — consume a list of declarations. The caller
    /// has already opened the block; we run until close-brace or EOF.
    fn consume_list_of_declarations(&mut self) -> Vec<Declaration<'i>> {
        let mut out = Vec::new();
        loop {
            match self.peek() {
                Token::Eof | Token::RightBrace => return out,
                Token::Whitespace | Token::Semicolon => {
                    self.pos += 1;
                },
                Token::AtKeyword(_) => {
                    // At-rule inside a declaration block (rare —
                    // @supports nested in @media etc.). We consume but
                    // discard for v0.3.35; full nesting support is a
                    // post-release item.
                    let _ = self.consume_at_rule();
                },
                Token::Ident(_) => {
                    if let Some(decl) = self.consume_declaration() {
                        out.push(decl);
                    } else {
                        // Bad declaration — skip to next semicolon /
                        // close brace.
                        self.skip_until_semicolon_or_close();
                    }
                },
                _ => {
                    // Garbage at the start of a declaration — skip.
                    self.skip_until_semicolon_or_close();
                },
            }
        }
    }

    /// CSS Syntax §5.4.5 — consume a declaration. Returns `None` if
    /// the start doesn't match `IDENT WS* ':'`.
    fn consume_declaration(&mut self) -> Option<Declaration<'i>> {
        let (name_tok, location) = self.next_token();
        let name = match name_tok {
            Token::Ident(s) => s,
            _ => return None,
        };
        self.skip_whitespace();
        if !matches!(self.peek(), Token::Colon) {
            return None;
        }
        self.pos += 1;
        self.skip_whitespace();

        let mut value: Vec<ComponentValue<'i>> = Vec::new();
        loop {
            match self.peek() {
                Token::Eof | Token::Semicolon | Token::RightBrace => break,
                _ => {
                    if let Some(cv) = self.consume_component_value() {
                        value.push(cv);
                    }
                },
            }
        }

        // Strip trailing whitespace from the value list.
        while matches!(value.last(), Some(ComponentValue::Token(Token::Whitespace))) {
            value.pop();
        }

        // !important detection — last two non-whitespace tokens are
        // delim('!') + ident("important").
        let important = is_important_suffix(&value);
        if important {
            // Strip the !important suffix from value.
            strip_important_suffix(&mut value);
        }

        Some(Declaration {
            name,
            value,
            important,
            location,
        })
    }

    /// CSS Syntax §5.4.7 — consume a component value.
    fn consume_component_value(&mut self) -> Option<ComponentValue<'i>> {
        let (tok, _) = self.next_token();
        Some(match tok {
            Token::LeftBrace => {
                ComponentValue::Curly(self.consume_simple_block_body(Token::RightBrace))
            },
            Token::LeftParen => {
                ComponentValue::Parens(self.consume_simple_block_body(Token::RightParen))
            },
            Token::LeftSquare => {
                ComponentValue::Square(self.consume_simple_block_body(Token::RightSquare))
            },
            Token::Function(name) => {
                let body = self.consume_simple_block_body(Token::RightParen);
                ComponentValue::Function { name, body }
            },
            t => ComponentValue::Token(t),
        })
    }

    /// Consume the body of a simple block until the matching close
    /// token. The opener has already been consumed by the caller.
    /// Per spec, EOF terminates without error.
    fn consume_simple_block_body(&mut self, close: Token<'static>) -> Vec<ComponentValue<'i>> {
        let mut out = Vec::new();
        loop {
            match self.peek() {
                Token::Eof => return out,
                t if mem::discriminant(t) == mem::discriminant(&close) => {
                    self.pos += 1;
                    return out;
                },
                _ => {
                    if let Some(cv) = self.consume_component_value() {
                        out.push(cv);
                    }
                },
            }
        }
    }

    fn skip_until_semicolon_or_close(&mut self) {
        let mut depth = 0_i32;
        loop {
            match self.peek() {
                Token::Eof => return,
                Token::Semicolon if depth == 0 => {
                    self.pos += 1;
                    return;
                },
                Token::RightBrace if depth == 0 => return,
                Token::LeftParen | Token::LeftSquare | Token::LeftBrace => {
                    depth += 1;
                    self.pos += 1;
                },
                Token::RightParen | Token::RightSquare | Token::RightBrace => {
                    depth -= 1;
                    self.pos += 1;
                    if depth < 0 {
                        return;
                    }
                },
                _ => self.pos += 1,
            }
        }
    }
}

use std::mem;

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

/// Re-parse a block body (already-tokenized component values) as a
/// declaration list. Used to turn a qualified-rule body into
/// declarations after the block was consumed as raw component values.
fn declarations_from_component_values(values: Vec<ComponentValue<'_>>) -> Vec<Declaration<'_>> {
    // Flatten back to tokens and run the declaration parser. This is
    // the small price we pay for keeping the declaration parser
    // language-driven on the token stream rather than on component
    // values.
    let tokens: Vec<(Token<'_>, SourceLocation)> = component_values_to_tokens(values);
    let mut p = Parser::new(tokens);
    p.consume_list_of_declarations()
}

fn component_values_to_tokens(values: Vec<ComponentValue<'_>>) -> Vec<(Token<'_>, SourceLocation)> {
    let mut out = Vec::new();
    flatten_into(values, &mut out);
    out.push((Token::Eof, SourceLocation::start()));
    out
}

fn flatten_into<'i>(values: Vec<ComponentValue<'i>>, out: &mut Vec<(Token<'i>, SourceLocation)>) {
    let loc = SourceLocation::start();
    for cv in values {
        match cv {
            ComponentValue::Token(t) => out.push((t, loc)),
            ComponentValue::Function { name, body } => {
                out.push((Token::Function(name), loc));
                flatten_into(body, out);
                out.push((Token::RightParen, loc));
            },
            ComponentValue::Parens(body) => {
                out.push((Token::LeftParen, loc));
                flatten_into(body, out);
                out.push((Token::RightParen, loc));
            },
            ComponentValue::Square(body) => {
                out.push((Token::LeftSquare, loc));
                flatten_into(body, out);
                out.push((Token::RightSquare, loc));
            },
            ComponentValue::Curly(body) => {
                out.push((Token::LeftBrace, loc));
                flatten_into(body, out);
                out.push((Token::RightBrace, loc));
            },
        }
    }
}

fn is_important_suffix(value: &[ComponentValue<'_>]) -> bool {
    // Find the last non-whitespace token, then the one before it.
    let mut iter = value
        .iter()
        .rev()
        .filter(|cv| !matches!(cv, ComponentValue::Token(Token::Whitespace)));
    let last = iter.next();
    let second_last = iter.next();
    matches!(last, Some(ComponentValue::Token(Token::Ident(s))) if s.eq_ignore_ascii_case("important"))
        && matches!(second_last, Some(ComponentValue::Token(Token::Delim('!'))))
}

fn strip_important_suffix(value: &mut Vec<ComponentValue<'_>>) {
    // Remove `!important` plus surrounding whitespace at the end.
    while matches!(value.last(), Some(ComponentValue::Token(Token::Whitespace))) {
        value.pop();
    }
    // Pop "important"
    value.pop();
    while matches!(value.last(), Some(ComponentValue::Token(Token::Whitespace))) {
        value.pop();
    }
    // Pop '!'
    value.pop();
    while matches!(value.last(), Some(ComponentValue::Token(Token::Whitespace))) {
        value.pop();
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unwrap_ident(cv: &ComponentValue<'_>) -> String {
        match cv {
            ComponentValue::Token(Token::Ident(s)) => s.to_string(),
            other => panic!("expected ident, got {other:?}"),
        }
    }

    #[test]
    fn empty_stylesheet() {
        let s = parse_stylesheet("").unwrap();
        assert!(s.rules.is_empty());
    }

    #[test]
    fn single_rule_one_declaration() {
        let s = parse_stylesheet("body { color: red; }").unwrap();
        assert_eq!(s.rules.len(), 1);
        let r = match &s.rules[0] {
            Rule::Qualified(q) => q,
            _ => panic!("expected qualified rule"),
        };
        // Prelude has [body]
        let body = r
            .prelude
            .iter()
            .find(|cv| !matches!(cv, ComponentValue::Token(Token::Whitespace)))
            .expect("prelude must have body ident");
        assert_eq!(unwrap_ident(body), "body");
        // One declaration: color: red;
        assert_eq!(r.declarations.len(), 1);
        let d = &r.declarations[0];
        assert_eq!(d.name, "color");
        assert!(!d.important);
        assert!(matches!(
            d.value.first(),
            Some(ComponentValue::Token(Token::Ident(s))) if s == "red"
        ));
    }

    #[test]
    fn multiple_declarations() {
        let s = parse_stylesheet("p { color: red; font-size: 12px; margin: 0 }").unwrap();
        let r = match &s.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        assert_eq!(r.declarations.len(), 3);
        let names: Vec<&str> = r.declarations.iter().map(|d| d.name.as_ref()).collect();
        assert_eq!(names, vec!["color", "font-size", "margin"]);
    }

    #[test]
    fn important_flag() {
        let s = parse_stylesheet("a { color: blue !important; }").unwrap();
        let r = match &s.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        let d = &r.declarations[0];
        assert!(d.important);
        // Value should NOT include the !important tokens.
        let has_important = d.value.iter().any(|cv| {
            matches!(cv, ComponentValue::Token(Token::Ident(s)) if s.eq_ignore_ascii_case("important"))
        });
        assert!(!has_important);
    }

    #[test]
    fn invalid_declaration_is_skipped() {
        let s = parse_stylesheet("p { invalid bare tokens; color: red; }").unwrap();
        let r = match &s.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        // The invalid declaration is dropped, color: red survives.
        assert_eq!(r.declarations.len(), 1);
        assert_eq!(r.declarations[0].name, "color");
    }

    #[test]
    fn function_in_value_preserved_as_component_value() {
        let s = parse_stylesheet("div { color: rgba(255, 0, 0, 0.5); }").unwrap();
        let r = match &s.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        let d = &r.declarations[0];
        let func = d.value.iter().find_map(|cv| match cv {
            ComponentValue::Function { name, body } => Some((name, body)),
            _ => None,
        });
        let (name, body) = func.expect("rgba must be a Function component value");
        assert_eq!(name, "rgba");
        // Should have at least 4 number/ident-ish tokens inside.
        let non_ws = body
            .iter()
            .filter(|cv| !matches!(cv, ComponentValue::Token(Token::Whitespace)))
            .count();
        assert!(non_ws >= 4);
    }

    #[test]
    fn at_rule_with_block() {
        let s = parse_stylesheet("@media print { body { color: black; } }").unwrap();
        assert_eq!(s.rules.len(), 1);
        let at = match &s.rules[0] {
            Rule::AtRule(a) => a,
            _ => panic!("expected at-rule"),
        };
        assert_eq!(at.name, "media");
        assert!(at.block.is_some());
    }

    #[test]
    fn at_rule_without_block_terminated_by_semicolon() {
        let s = parse_stylesheet("@charset \"UTF-8\";").unwrap();
        let at = match &s.rules[0] {
            Rule::AtRule(a) => a,
            _ => panic!("expected at-rule"),
        };
        assert_eq!(at.name, "charset");
        assert!(at.block.is_none());
    }

    #[test]
    fn nested_blocks_preserved() {
        // Square brackets in selector preludes (attribute selector).
        let s = parse_stylesheet(r#"a[href^="https://"] { color: green; }"#).unwrap();
        let r = match &s.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        let has_square = r
            .prelude
            .iter()
            .any(|cv| matches!(cv, ComponentValue::Square(_)));
        assert!(has_square, "attribute selector [href^=...] must preserve");
    }

    #[test]
    fn unbalanced_brackets_recover_at_eof() {
        let s = parse_stylesheet("body { color: red").unwrap();
        // We at least produced a rule without panicking.
        assert!(!s.rules.is_empty());
    }

    #[test]
    fn parse_inline_style_attribute() {
        let decls = parse_declaration_list("color: red; font-size: 14px").unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].name, "color");
        assert_eq!(decls[1].name, "font-size");
    }

    #[test]
    fn comments_disappear() {
        let s =
            parse_stylesheet("/* hi */ body /* mid */ { /* in */ color: red /* tail */ }").unwrap();
        assert_eq!(s.rules.len(), 1);
        let r = match &s.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        assert_eq!(r.declarations.len(), 1);
    }

    #[test]
    fn calc_in_value() {
        let s = parse_stylesheet("div { width: calc(100% - 20px); }").unwrap();
        let r = match &s.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        let calc_fn = r.declarations[0].value.iter().find_map(|cv| match cv {
            ComponentValue::Function { name, body } if name == "calc" => Some(body),
            _ => None,
        });
        assert!(calc_fn.is_some());
    }

    #[test]
    fn at_page_with_descriptors() {
        let s = parse_stylesheet("@page { size: A4; margin: 20mm; }").unwrap();
        let at = match &s.rules[0] {
            Rule::AtRule(a) => a,
            _ => unreachable!(),
        };
        assert_eq!(at.name, "page");
        assert!(at.block.is_some());
    }
}
