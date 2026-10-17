//! CSS Custom Properties (`var()`) resolver — CSS-7.
//!
//! Custom properties are CSS properties whose name begins with `--`.
//! They inherit by default (the cascade in CSS-5 already handles this
//! because every cascade operation walks `inherit_to_parent` style by
//! name, regardless of the name's prefix). What CSS-7 adds is the
//! `var()` substitution: when a regular property's value contains a
//! `var(--name)` reference, this module looks the custom property up
//! in [`ComputedStyles`] and substitutes its component values in
//! place. If the custom property isn't defined, the optional fallback
//! after the comma is used; failing that, the substitution returns
//! `Err(VarError::Undefined)` and the calling property parser
//! (CSS-8) treats the declaration as invalid (per CSS Custom
//! Properties L1 §3.2 — "computed value of `var()` reference is the
//! initial value of the property").
//!
//! Cycle detection uses depth-first colour marking: a custom property
//! currently being resolved is grey; one whose resolution is complete
//! is black. Encountering a grey property again is a cycle, and the
//! whole substitution returns [`VarError::Cycle`].
//!
//! Substitution is recursive — `--a: var(--b)`, `--b: var(--c)`,
//! `--c: 12px` resolves `var(--a)` to `12px`. Caller-supplied
//! fallbacks may themselves contain `var()` calls; those are resolved
//! by the same machinery.

use std::collections::{HashMap, HashSet};
use thiserror::Error;

use super::cascade::ComputedStyles;
use super::parser::ComponentValue;
use super::tokenizer::Token;

/// `var()` resolution errors.
#[derive(Debug, Error, PartialEq)]
pub enum VarError {
    /// `var(--name)` referenced a custom property that isn't defined
    /// and provides no fallback.
    #[error("undefined custom property: --{0}")]
    Undefined(String),

    /// Cycle detected — a custom property transitively references
    /// itself.
    #[error("cycle in custom property resolution involving --{0}")]
    Cycle(String),

    /// `var(...)` was malformed at parse time (no name, etc.).
    #[error("malformed var() invocation")]
    Malformed,
}

/// Substitute every `var(...)` reference inside `value` with its
/// resolved component values, using `styles` as the source of custom
/// property definitions.
///
/// Custom properties inside `styles` are themselves expected to live
/// under their full `--name` key. The cascade in CSS-5 stores them
/// that way already.
pub fn substitute<'i>(
    value: &[ComponentValue<'i>],
    styles: &ComputedStyles<'i>,
) -> Result<Vec<ComponentValue<'i>>, VarError> {
    let mut visited = HashSet::new();
    substitute_inner(value, styles, &mut visited)
}

fn substitute_inner<'i>(
    value: &[ComponentValue<'i>],
    styles: &ComputedStyles<'i>,
    visiting: &mut HashSet<String>,
) -> Result<Vec<ComponentValue<'i>>, VarError> {
    let mut out: Vec<ComponentValue<'i>> = Vec::with_capacity(value.len());
    for cv in value {
        match cv {
            ComponentValue::Function { name, body } if name.eq_ignore_ascii_case("var") => {
                let (var_name, fallback) = parse_var_args(body)?;
                if !visiting.insert(var_name.clone()) {
                    return Err(VarError::Cycle(var_name));
                }
                let resolved = match styles.get(&var_name) {
                    Some(rv) => substitute_inner(&rv.value, styles, visiting)?,
                    None => match fallback {
                        Some(fb) => substitute_inner(&fb, styles, visiting)?,
                        None => {
                            visiting.remove(&var_name);
                            return Err(VarError::Undefined(var_name));
                        },
                    },
                };
                visiting.remove(&var_name);
                out.extend(resolved);
            },
            ComponentValue::Function { name, body } => {
                let inner = substitute_inner(body, styles, visiting)?;
                out.push(ComponentValue::Function {
                    name: name.clone(),
                    body: inner,
                });
            },
            ComponentValue::Parens(body) => {
                out.push(ComponentValue::Parens(substitute_inner(body, styles, visiting)?));
            },
            ComponentValue::Square(body) => {
                out.push(ComponentValue::Square(substitute_inner(body, styles, visiting)?));
            },
            ComponentValue::Curly(body) => {
                out.push(ComponentValue::Curly(substitute_inner(body, styles, visiting)?));
            },
            other => out.push(other.clone()),
        }
    }
    Ok(out)
}

/// Parse the body of `var(name [, fallback])`. The name is the leading
/// `--ident`; everything after the first top-level comma is the
/// fallback (preserved as a `Vec<ComponentValue>` so it can itself
/// contain `var()` calls).
fn parse_var_args<'i>(
    body: &[ComponentValue<'i>],
) -> Result<(String, Option<Vec<ComponentValue<'i>>>), VarError> {
    // Find the first non-whitespace component-value — must be an Ident
    // beginning with `--`.
    let mut iter = body.iter().enumerate();
    let (i, name_cv) = loop {
        let (i, cv) = iter.next().ok_or(VarError::Malformed)?;
        if !matches!(cv, ComponentValue::Token(Token::Whitespace)) {
            break (i, cv);
        }
    };
    let name = match name_cv {
        ComponentValue::Token(Token::Ident(s)) if s.starts_with("--") => s.to_string(),
        _ => return Err(VarError::Malformed),
    };

    // After the name, either the body ends or there's a comma followed
    // by the fallback.
    let mut j = i + 1;
    while j < body.len() && matches!(body[j], ComponentValue::Token(Token::Whitespace)) {
        j += 1;
    }
    if j >= body.len() {
        return Ok((name, None));
    }
    if !matches!(body[j], ComponentValue::Token(Token::Comma)) {
        return Err(VarError::Malformed);
    }
    j += 1;
    while j < body.len() && matches!(body[j], ComponentValue::Token(Token::Whitespace)) {
        j += 1;
    }
    let fallback = if j < body.len() {
        Some(body[j..].to_vec())
    } else {
        // var(--x, ) — empty fallback. Spec: empty fallback is the
        // empty value, not "undefined".
        Some(Vec::new())
    };
    Ok((name, fallback))
}

/// Walk every custom property in `styles` and pre-resolve any internal
/// `var()` references, returning a flat map. Useful when the same
/// stylesheet is going to substitute many regular properties — the
/// pre-resolved map lets the substitution skip the recursive walk.
///
/// Cycles are reported but otherwise non-fatal — the cycling property
/// is dropped from the output and resolution moves on.
pub fn resolve_custom_properties<'i>(
    styles: &ComputedStyles<'i>,
) -> HashMap<String, Vec<ComponentValue<'i>>> {
    let mut out = HashMap::new();
    for (name, rv) in styles.iter() {
        if !name.starts_with("--") {
            continue;
        }
        let mut visiting = HashSet::new();
        if let Ok(resolved) = substitute_inner(&rv.value, styles, &mut visiting) {
            out.insert(name.to_string(), resolved);
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
    use crate::html_css::css::cascade::cascade;
    use crate::html_css::css::matcher::Element;
    use crate::html_css::css::parser::parse_stylesheet;

    // ---- Tiny mock DOM (one element, no parents) -------------------

    struct Node;

    #[derive(Clone, Copy)]
    struct E;
    impl Element for E {
        fn local_name(&self) -> &str {
            "div"
        }
        fn id(&self) -> Option<&str> {
            None
        }
        fn has_class(&self, _: &str) -> bool {
            false
        }
        fn attribute(&self, _: &str) -> Option<&str> {
            None
        }
        fn has_attribute(&self, _: &str) -> bool {
            false
        }
        fn parent(&self) -> Option<Self> {
            None
        }
        fn prev_element_sibling(&self) -> Option<Self> {
            None
        }
        fn next_element_sibling(&self) -> Option<Self> {
            None
        }
        fn is_empty(&self) -> bool {
            true
        }
        fn first_element_child(&self) -> Option<Self> {
            None
        }
    }

    fn cascade_for(css: &'static str) -> ComputedStyles<'static> {
        let ss: &'static _ = Box::leak(Box::new(parse_stylesheet(css).unwrap()));
        cascade(ss, E, None)
    }

    fn render_to_string(values: &[ComponentValue<'_>]) -> String {
        // Cheap printer for assertions — joins idents/numbers/dimensions
        // with single spaces.
        let mut out = String::new();
        for cv in values {
            match cv {
                ComponentValue::Token(Token::Ident(s)) => {
                    if !out.is_empty() {
                        out.push(' ');
                    }
                    out.push_str(s);
                },
                ComponentValue::Token(Token::Number(n)) => {
                    if !out.is_empty() {
                        out.push(' ');
                    }
                    out.push_str(&format!("{}", n.value));
                },
                ComponentValue::Token(Token::Dimension { value, unit }) => {
                    if !out.is_empty() {
                        out.push(' ');
                    }
                    out.push_str(&format!("{}{}", value.value, unit));
                },
                ComponentValue::Token(Token::Whitespace) => {},
                _ => {},
            }
        }
        out
    }

    #[test]
    fn simple_substitution() {
        let styles = cascade_for("div { --c: red; color: var(--c); }");
        let color = styles.get("color").unwrap();
        let resolved = substitute(&color.value, &styles).unwrap();
        assert_eq!(render_to_string(&resolved), "red");
    }

    #[test]
    fn fallback_when_undefined() {
        let styles = cascade_for("div { color: var(--missing, blue); }");
        let color = styles.get("color").unwrap();
        let resolved = substitute(&color.value, &styles).unwrap();
        assert_eq!(render_to_string(&resolved), "blue");
    }

    #[test]
    fn undefined_without_fallback_errors() {
        let styles = cascade_for("div { color: var(--missing); }");
        let color = styles.get("color").unwrap();
        let res = substitute(&color.value, &styles);
        assert!(matches!(res, Err(VarError::Undefined(s)) if s == "--missing"));
    }

    #[test]
    fn nested_var_substitution() {
        let styles =
            cascade_for("div { --base: 12px; --bigger: var(--base); width: var(--bigger); }");
        let width = styles.get("width").unwrap();
        let resolved = substitute(&width.value, &styles).unwrap();
        assert_eq!(render_to_string(&resolved), "12px");
    }

    #[test]
    fn cycle_two_step_detected() {
        let styles = cascade_for("div { --a: var(--b); --b: var(--a); color: var(--a); }");
        let color = styles.get("color").unwrap();
        let res = substitute(&color.value, &styles);
        assert!(matches!(res, Err(VarError::Cycle(_))));
    }

    #[test]
    fn cycle_self_reference_detected() {
        let styles = cascade_for("div { --x: var(--x); color: var(--x); }");
        let color = styles.get("color").unwrap();
        let res = substitute(&color.value, &styles);
        assert!(matches!(res, Err(VarError::Cycle(_))));
    }

    #[test]
    fn fallback_can_use_var() {
        let styles = cascade_for("div { --known: green; color: var(--missing, var(--known)); }");
        let color = styles.get("color").unwrap();
        let resolved = substitute(&color.value, &styles).unwrap();
        assert_eq!(render_to_string(&resolved), "green");
    }

    #[test]
    fn substitution_inside_function() {
        // The substitution must walk into nested function bodies too.
        let styles = cascade_for("div { --pad: 10px; width: calc(100% - var(--pad)); }");
        let width = styles.get("width").unwrap();
        let resolved = substitute(&width.value, &styles).unwrap();
        let s = format!("{:?}", resolved);
        // We expect the var() to be replaced by 10px somewhere
        // inside the calc() function body.
        assert!(s.contains("Dimension"));
        assert!(!s.contains("\"var\""));
    }

    #[test]
    fn empty_fallback_substitutes_to_empty() {
        let styles = cascade_for("div { color: var(--missing,); }");
        let color = styles.get("color").unwrap();
        let resolved = substitute(&color.value, &styles).unwrap();
        // After substitution the value is empty.
        assert!(render_to_string(&resolved).is_empty());
    }

    #[test]
    fn resolve_custom_properties_collects_all() {
        let styles = cascade_for("div { --a: red; --b: 12px; --c: var(--a); color: green; }");
        let resolved = resolve_custom_properties(&styles);
        assert!(resolved.contains_key("--a"));
        assert!(resolved.contains_key("--b"));
        assert!(resolved.contains_key("--c"));
        // Regular properties are not in the map.
        assert!(!resolved.contains_key("color"));
    }
}
