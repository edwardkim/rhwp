//! CSS cascade — walks a [`Stylesheet`] against a DOM element and
//! produces [`ComputedStyles`]: the property values that should apply
//! to that element after origin/importance/specificity/source-order
//! sorting and inheritance from the parent.
//!
//! This is the layer that turns "we have rules" into "this element's
//! `color` is `red`". CSS-8 (property parsing) consumes the resulting
//! `Vec<ComponentValue>` per property to produce typed values; the
//! layout engine (Phase LAYOUT) consumes those.
//!
//! v0.3.35 cascade order matches CSS Cascading and Inheritance L4 §6.4
//! reduced to a single origin (Author):
//!
//!   1. !important Author declarations
//!   2. Normal Author declarations (later source order overrides earlier)
//!
//! User-Agent and User origins are not yet sourced separately —
//! UA defaults will arrive as a hard-coded baseline stylesheet in
//! CSS-9 (`@page` plus a small UA reset). Until then, properties
//! that aren't declared anywhere fall back to the static [`initial_value`]
//! lookup.
//!
//! Inheritance: hard-coded list of inherited properties matching the
//! CSS Cascade L4 spec for the v0.3.35 supported surface. When a
//! property is inherited and the element doesn't declare it, the
//! parent's computed value flows down.

use std::collections::HashMap;

use super::counters::{evaluate_content, parse_content, CounterState};
use super::matcher::{match_complex_selector, Element};
use super::parser::{ComponentValue, Declaration, Rule, Stylesheet};
use super::selectors::{parse_selector_list, PseudoElement, Specificity, SubclassSelector};
use super::tokenizer::Token;

// ─────────────────────────────────────────────────────────────────────
// Output type
// ─────────────────────────────────────────────────────────────────────

/// Per-element computed styles — the cascade output.
///
/// Indexed by lowercase property name. Values are kept as the raw
/// component-value sequence so CSS-8 property parsers can re-walk them
/// without a second tokenisation pass.
#[derive(Debug, Clone, Default)]
pub struct ComputedStyles<'i> {
    properties: HashMap<String, ResolvedValue<'i>>,
}

/// One winning declaration after the cascade picked it.
#[derive(Debug, Clone)]
pub struct ResolvedValue<'i> {
    /// Component values from the declaration's RHS.
    pub value: Vec<ComponentValue<'i>>,
    /// Whether the source declaration carried `!important`.
    pub important: bool,
    /// Specificity of the matching selector — kept so layered cascades
    /// (UA + Author) can re-sort once UA stylesheets land.
    pub specificity: Specificity,
    /// Source order across the whole stylesheet — kept for the same
    /// reason.
    pub source_order: usize,
    /// True if this value was inherited from the parent rather than
    /// declared on the element itself. Layout sometimes wants to know.
    pub inherited: bool,
}

impl<'i> ComputedStyles<'i> {
    /// Look up a property by name (lowercase).
    pub fn get(&self, name: &str) -> Option<&ResolvedValue<'i>> {
        self.properties.get(name)
    }

    /// Iterate all (name, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &ResolvedValue<'i>)> {
        self.properties.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Number of resolved properties.
    pub fn len(&self) -> usize {
        self.properties.len()
    }

    /// Empty?
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Public entry points
// ─────────────────────────────────────────────────────────────────────

/// Compute styles for `element` against `stylesheet`, taking
/// inheritance from `parent_styles` (pass `None` for the root).
///
/// Inline styles (parsed from the `style="..."` attribute via
/// [`super::parser::parse_declaration_list`]) can be merged via
/// [`apply_inline_declarations`] after this returns — they always win
/// over rule-sourced declarations of the same specificity tier (L4
/// §6.4 step 8).
pub fn cascade<'i, E: Element>(
    stylesheet: &'i Stylesheet<'i>,
    element: E,
    parent_styles: Option<&ComputedStyles<'i>>,
) -> ComputedStyles<'i> {
    // Step 1: collect every (matching declaration + provenance).
    // Provenance lives at the rule level — the same selector might
    // hit multiple declarations.
    let mut candidates: HashMap<String, Vec<Candidate<'i>>> = HashMap::new();
    let mut source_order: usize = 0;
    for rule in &stylesheet.rules {
        if let Rule::Qualified(qrule) = rule {
            let Ok(list) = parse_selector_list(&qrule.prelude) else {
                source_order += qrule.declarations.len();
                continue;
            };
            // Pick the highest specificity among the comma-separated
            // selectors that actually match (CSS L4 §6.4 step 7).
            let best_specificity = list
                .selectors
                .iter()
                .filter(|sel| match_complex_selector(sel, element))
                .map(|sel| sel.specificity)
                .max();
            if let Some(specificity) = best_specificity {
                for decl in &qrule.declarations {
                    source_order += 1;
                    candidates
                        .entry(decl.name.to_string())
                        .or_default()
                        .push(Candidate {
                            decl,
                            specificity,
                            source_order,
                        });
                }
            } else {
                source_order += qrule.declarations.len();
            }
        }
    }

    // Step 2: pick the winner per property.
    let mut properties: HashMap<String, ResolvedValue<'i>> = HashMap::new();
    for (name, mut cands) in candidates {
        cands.sort_by(|a, b| {
            // Important true > important false.
            // Then specificity ascending; we'll pick the last (max).
            // Then source_order ascending; we'll pick the last (max).
            (a.decl.important, a.specificity, a.source_order).cmp(&(
                b.decl.important,
                b.specificity,
                b.source_order,
            ))
        });
        if let Some(winner) = cands.last() {
            properties.insert(
                name,
                ResolvedValue {
                    value: winner.decl.value.clone(),
                    important: winner.decl.important,
                    specificity: winner.specificity,
                    source_order: winner.source_order,
                    inherited: false,
                },
            );
        }
    }

    // Step 3: inheritance — for each property in the inherited list
    // that's NOT declared on this element, pull from parent.
    if let Some(parent) = parent_styles {
        for &prop in INHERITED_PROPERTIES {
            if !properties.contains_key(prop) {
                if let Some(parent_val) = parent.properties.get(prop) {
                    let mut inherited = parent_val.clone();
                    inherited.inherited = true;
                    properties.insert(prop.to_string(), inherited);
                }
            }
        }
    }

    ComputedStyles { properties }
}

/// Merge a list of inline-style declarations (from an HTML
/// `style="..."` attribute) into an existing [`ComputedStyles`]. Inline
/// styles are treated as having `Specificity::new(1, 0, 0, 0)` —
/// higher than any rule-sourced selector — per CSS L4 §6.4 step 8.
pub fn apply_inline_declarations<'i>(
    styles: &mut ComputedStyles<'i>,
    declarations: &[Declaration<'i>],
) {
    let inline_specificity = Specificity::new(255, 255, 255);
    let mut order = usize::MAX / 2; // far past any rule-sourced order
    for decl in declarations {
        order += 1;
        let candidate = ResolvedValue {
            value: decl.value.clone(),
            important: decl.important,
            specificity: inline_specificity,
            source_order: order,
            inherited: false,
        };
        // Inline always wins over non-important rules; an !important
        // *rule* still beats a non-important inline style per spec L4
        // §6.4 (User and Author !important come after Author inline
        // when sorted).
        match styles.properties.get(decl.name.as_ref()) {
            Some(existing) if existing.important && !decl.important => {
                // Don't overwrite an !important rule with a normal inline.
                continue;
            },
            _ => {
                styles.properties.insert(decl.name.to_string(), candidate);
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Pseudo-element generated content (::before / ::after)
// ─────────────────────────────────────────────────────────────────────

/// Which generated-content pseudo-element to resolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PseudoKind {
    /// `::before`.
    Before,
    /// `::after`.
    After,
}

impl PseudoKind {
    fn matches(self, pe: &PseudoElement) -> bool {
        matches!(
            (self, pe),
            (PseudoKind::Before, PseudoElement::Before) | (PseudoKind::After, PseudoElement::After)
        )
    }
}

/// Resolve the generated text for `element`'s `::before` / `::after`
/// pseudo-element. Returns `None` when no `content:` declaration
/// matches, when it resolves to `none` / `normal`, or when the result
/// is empty. `counter()` / `counters()` references use an empty counter
/// state for the v0.3.37 first cut — literal strings, `attr()`,
/// `open-quote` / `close-quote` all work.
pub fn pseudo_content_for<'i, E: Element>(
    stylesheet: &'i Stylesheet<'i>,
    element: E,
    pseudo: PseudoKind,
) -> Option<String> {
    #[derive(Default)]
    struct Cand<'a> {
        value: &'a [ComponentValue<'a>],
        important: bool,
        specificity: Specificity,
        source_order: usize,
    }

    let mut best: Option<Cand<'_>> = None;
    let mut source_order: usize = 0;
    for rule in &stylesheet.rules {
        let Rule::Qualified(qrule) = rule else {
            continue;
        };
        let Ok(list) = parse_selector_list(&qrule.prelude) else {
            source_order += qrule.declarations.len();
            continue;
        };
        // Pick the highest specificity among selectors that (a) target
        // the requested pseudo-element and (b) match the host element.
        let mut sel_spec: Option<Specificity> = None;
        for sel in &list.selectors {
            if sel.compounds.is_empty() {
                continue;
            }
            let subject = &sel.compounds[0];
            let targets_pseudo = subject.subclasses.iter().any(|s| match s {
                SubclassSelector::PseudoElement(pe) => pseudo.matches(pe),
                _ => false,
            });
            if !targets_pseudo {
                continue;
            }
            if match_complex_selector(sel, element) {
                sel_spec = Some(sel_spec.map_or(sel.specificity, |s| s.max(sel.specificity)));
            }
        }
        let Some(specificity) = sel_spec else {
            source_order += qrule.declarations.len();
            continue;
        };
        for decl in &qrule.declarations {
            source_order += 1;
            if decl.name.as_ref() != "content" {
                continue;
            }
            let replace = match &best {
                None => true,
                Some(cur) => {
                    (cur.important, cur.specificity, cur.source_order)
                        < (decl.important, specificity, source_order)
                },
            };
            if replace {
                best = Some(Cand {
                    value: decl.value.as_slice(),
                    important: decl.important,
                    specificity,
                    source_order,
                });
            }
        }
    }

    let cand = best?;
    let parsed = parse_content(cand.value)?;
    let state = CounterState::new();
    let rendered =
        evaluate_content(&parsed, &state, |name| element.attribute(name).map(|s| s.to_string()));
    if rendered.is_empty() {
        None
    } else {
        Some(rendered)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Inherited property list — from CSS Cascade L4 + the property specs
// for the v0.3.35 supported surface
// ─────────────────────────────────────────────────────────────────────

/// Properties that inherit by default. Inheritance can be forced for
/// any property via the `inherit` keyword (handled in CSS-7 once
/// `var()`/`inherit`/`initial`/`unset`/`revert` keywords land).
const INHERITED_PROPERTIES: &[&str] = &[
    // Typography
    "color",
    "font",
    "font-family",
    "font-feature-settings",
    "font-kerning",
    "font-language-override",
    "font-size",
    "font-size-adjust",
    "font-stretch",
    "font-style",
    "font-synthesis",
    "font-variant",
    "font-variant-alternates",
    "font-variant-caps",
    "font-variant-east-asian",
    "font-variant-ligatures",
    "font-variant-numeric",
    "font-variant-position",
    "font-weight",
    "font-optical-sizing",
    "letter-spacing",
    "line-height",
    "text-align",
    "text-align-last",
    "text-decoration-line",
    "text-decoration-style",
    "text-decoration-color",
    "text-decoration-skip-ink",
    "text-emphasis",
    "text-emphasis-color",
    "text-emphasis-position",
    "text-emphasis-style",
    "text-indent",
    "text-justify",
    "text-orientation",
    "text-rendering",
    "text-shadow",
    "text-transform",
    "text-underline-offset",
    "text-underline-position",
    "white-space",
    "word-break",
    "word-spacing",
    "overflow-wrap",
    "hyphens",
    "tab-size",
    // Direction / writing mode
    "direction",
    "unicode-bidi",
    "writing-mode",
    // Lists
    "list-style",
    "list-style-image",
    "list-style-position",
    "list-style-type",
    // Tables
    "border-collapse",
    "border-spacing",
    "caption-side",
    "empty-cells",
    // Visibility / cursor (cursor not used in PDF but inherits per spec)
    "visibility",
    "cursor",
    // Quotes & generated content
    "quotes",
    // Counter inheritance happens specially via counter-reset/increment;
    // not in this list.
];

// ─────────────────────────────────────────────────────────────────────
// Initial-value fallback (small subset for v0.3.35; expanded in CSS-8)
// ─────────────────────────────────────────────────────────────────────

/// CSS initial value for a property, expressed as the source string.
/// Returns `None` for properties this lookup doesn't know — callers
/// (CSS-8 typed parsers) treat that as "use the property-specific
/// initial value defined inline".
pub fn initial_value(property: &str) -> Option<&'static str> {
    Some(match property {
        "color" => "black",
        "background-color" => "transparent",
        "display" => "inline",
        "position" => "static",
        "font-family" => "serif",
        "font-size" => "16px",
        "font-style" => "normal",
        "font-weight" => "normal",
        "line-height" => "normal",
        "text-align" => "start",
        "text-decoration" => "none",
        "white-space" => "normal",
        "margin" => "0",
        "padding" => "0",
        "border-width" => "0",
        "width" => "auto",
        "height" => "auto",
        "overflow" => "visible",
        "visibility" => "visible",
        _ => return None,
    })
}

// ─────────────────────────────────────────────────────────────────────
// Internal
// ─────────────────────────────────────────────────────────────────────

struct Candidate<'i> {
    decl: &'i Declaration<'i>,
    specificity: Specificity,
    source_order: usize,
}

/// Helper: extract the first ident token from a property value, useful
/// for property lookups in tests and for trivial CSS-8 stubs.
pub fn first_ident<'a>(value: &'a [ComponentValue<'a>]) -> Option<&'a str> {
    value.iter().find_map(|cv| match cv {
        ComponentValue::Token(Token::Ident(s)) => Some(s.as_ref()),
        _ => None,
    })
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::matcher::Element;
    use crate::html_css::css::parser::parse_stylesheet;

    // Minimal mock DOM — same shape as matcher tests, kept local
    // because we don't want to expose matcher::tests as a sibling
    // module.

    struct MockNode {
        tag: String,
        id: Option<String>,
        classes: Vec<String>,
        parent: Option<usize>,
        children: Vec<usize>,
    }

    struct MockDom {
        nodes: Vec<MockNode>,
    }

    impl MockDom {
        fn add(
            &mut self,
            parent: Option<usize>,
            tag: &str,
            id: Option<&str>,
            classes: &[&str],
        ) -> usize {
            let idx = self.nodes.len();
            self.nodes.push(MockNode {
                tag: tag.into(),
                id: id.map(String::from),
                classes: classes.iter().map(|s| s.to_string()).collect(),
                parent,
                children: Vec::new(),
            });
            if let Some(p) = parent {
                self.nodes[p].children.push(idx);
            }
            idx
        }
    }

    #[derive(Clone, Copy)]
    struct MockEl<'a> {
        dom: &'a MockDom,
        idx: usize,
    }
    impl<'a> Element for MockEl<'a> {
        fn local_name(&self) -> &str {
            &self.dom.nodes[self.idx].tag
        }
        fn id(&self) -> Option<&str> {
            self.dom.nodes[self.idx].id.as_deref()
        }
        fn has_class(&self, c: &str) -> bool {
            self.dom.nodes[self.idx].classes.iter().any(|x| x == c)
        }
        fn attribute(&self, _: &str) -> Option<&str> {
            None
        }
        fn has_attribute(&self, _: &str) -> bool {
            false
        }
        fn parent(&self) -> Option<Self> {
            self.dom.nodes[self.idx].parent.map(|i| MockEl {
                dom: self.dom,
                idx: i,
            })
        }
        fn prev_element_sibling(&self) -> Option<Self> {
            let p = self.dom.nodes[self.idx].parent?;
            let kids = &self.dom.nodes[p].children;
            let pos = kids.iter().position(|&k| k == self.idx)?;
            (pos > 0).then(|| MockEl {
                dom: self.dom,
                idx: kids[pos - 1],
            })
        }
        fn next_element_sibling(&self) -> Option<Self> {
            let p = self.dom.nodes[self.idx].parent?;
            let kids = &self.dom.nodes[p].children;
            let pos = kids.iter().position(|&k| k == self.idx)?;
            kids.get(pos + 1).map(|&k| MockEl {
                dom: self.dom,
                idx: k,
            })
        }
        fn is_empty(&self) -> bool {
            self.dom.nodes[self.idx].children.is_empty()
        }
        fn first_element_child(&self) -> Option<Self> {
            self.dom.nodes[self.idx].children.first().map(|&k| MockEl {
                dom: self.dom,
                idx: k,
            })
        }
    }

    fn build_dom() -> (MockDom, Vec<usize>) {
        let mut d = MockDom { nodes: Vec::new() };
        let html = d.add(None, "html", None, &[]);
        let body = d.add(Some(html), "body", None, &[]);
        let div = d.add(Some(body), "div", Some("main"), &["container"]);
        let p = d.add(Some(div), "p", None, &["lead"]);
        (d, vec![html, body, div, p])
    }

    #[test]
    fn single_rule_applies() {
        let css = "p { color: red; }";
        let ss = parse_stylesheet(css).unwrap();
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        let styles = cascade(&ss, p, None);
        let color = styles.get("color").expect("color must be set");
        assert_eq!(first_ident(&color.value), Some("red"));
    }

    #[test]
    fn higher_specificity_wins() {
        let css = "
            p { color: red; }
            .lead { color: green; }
            #main p { color: blue; }
        ";
        let ss = parse_stylesheet(css).unwrap();
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        let styles = cascade(&ss, p, None);
        // #main p has specificity (1, 0, 1); .lead is (0, 1, 0); p is (0, 0, 1).
        // The id-bearing rule wins.
        let color = styles.get("color").expect("color set");
        assert_eq!(first_ident(&color.value), Some("blue"));
    }

    #[test]
    fn later_source_order_wins_ties() {
        let css = "
            p { color: red; }
            p { color: green; }
        ";
        let ss = parse_stylesheet(css).unwrap();
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        let styles = cascade(&ss, p, None);
        let color = styles.get("color").unwrap();
        assert_eq!(first_ident(&color.value), Some("green"));
    }

    #[test]
    fn important_beats_normal() {
        let css = "
            #main p { color: blue !important; }
            .lead { color: green; }
            p { color: red; }
        ";
        let ss = parse_stylesheet(css).unwrap();
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        let styles = cascade(&ss, p, None);
        let color = styles.get("color").unwrap();
        assert!(color.important);
        assert_eq!(first_ident(&color.value), Some("blue"));
    }

    #[test]
    fn inherits_color_from_parent() {
        let css = "div { color: green; }";
        let ss = parse_stylesheet(css).unwrap();
        let (d, ix) = build_dom();
        let div = MockEl {
            dom: &d,
            idx: ix[2],
        };
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };

        let div_styles = cascade(&ss, div, None);
        assert_eq!(first_ident(&div_styles.get("color").unwrap().value), Some("green"));

        let p_styles = cascade(&ss, p, Some(&div_styles));
        let color = p_styles.get("color").expect("color must be inherited");
        assert!(color.inherited);
        assert_eq!(first_ident(&color.value), Some("green"));
    }

    #[test]
    fn does_not_inherit_non_inherited_property() {
        let css = "div { background-color: yellow; }";
        let ss = parse_stylesheet(css).unwrap();
        let (d, ix) = build_dom();
        let div = MockEl {
            dom: &d,
            idx: ix[2],
        };
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };

        let div_styles = cascade(&ss, div, None);
        assert!(div_styles.get("background-color").is_some());

        let p_styles = cascade(&ss, p, Some(&div_styles));
        // background-color is not inheritable.
        assert!(p_styles.get("background-color").is_none());
    }

    #[test]
    fn inline_declarations_override_rules() {
        let css = "p { color: red; }";
        let ss = parse_stylesheet(css).unwrap();
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        let mut styles = cascade(&ss, p, None);
        let inline = crate::html_css::css::parser::parse_declaration_list("color: blue").unwrap();
        apply_inline_declarations(&mut styles, &inline);
        assert_eq!(first_ident(&styles.get("color").unwrap().value), Some("blue"));
    }

    #[test]
    fn important_rule_beats_normal_inline() {
        let css = "p { color: red !important; }";
        let ss = parse_stylesheet(css).unwrap();
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        let mut styles = cascade(&ss, p, None);
        let inline = crate::html_css::css::parser::parse_declaration_list("color: blue").unwrap();
        apply_inline_declarations(&mut styles, &inline);
        // !important rule still wins.
        assert_eq!(first_ident(&styles.get("color").unwrap().value), Some("red"));
    }

    #[test]
    fn unmatching_selectors_skipped() {
        let css = "
            .nope { color: red; }
            p { color: green; }
        ";
        let ss = parse_stylesheet(css).unwrap();
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        let styles = cascade(&ss, p, None);
        assert_eq!(first_ident(&styles.get("color").unwrap().value), Some("green"));
    }

    #[test]
    fn pseudo_before_literal_string_resolves() {
        let css = r#"p::before { content: "§ "; }"#;
        let ss: &'static _ = Box::leak(Box::new(parse_stylesheet(css).unwrap()));
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        let got = pseudo_content_for(ss, p, PseudoKind::Before);
        assert_eq!(got.as_deref(), Some("§ "));
    }

    #[test]
    fn pseudo_after_only_matches_after_selector() {
        let css = r#"
            p::before { content: "X"; }
            p::after  { content: "Y"; }
        "#;
        let ss: &'static _ = Box::leak(Box::new(parse_stylesheet(css).unwrap()));
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        let before = pseudo_content_for(ss, p, PseudoKind::Before);
        let after = pseudo_content_for(ss, p, PseudoKind::After);
        assert_eq!(before.as_deref(), Some("X"));
        assert_eq!(after.as_deref(), Some("Y"));
    }

    #[test]
    fn pseudo_content_none_returns_none() {
        let css = "p::before { content: none; }";
        let ss: &'static _ = Box::leak(Box::new(parse_stylesheet(css).unwrap()));
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        assert!(pseudo_content_for(ss, p, PseudoKind::Before).is_none());
    }

    #[test]
    fn pseudo_content_unmatched_element_returns_none() {
        let css = r#"div::before { content: "D"; }"#;
        let ss: &'static _ = Box::leak(Box::new(parse_stylesheet(css).unwrap()));
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        assert!(pseudo_content_for(ss, p, PseudoKind::Before).is_none());
    }

    #[test]
    fn many_properties() {
        let css = "p { color: red; font-size: 14px; margin: 0; }";
        let ss = parse_stylesheet(css).unwrap();
        let (d, ix) = build_dom();
        let p = MockEl {
            dom: &d,
            idx: ix[3],
        };
        let styles = cascade(&ss, p, None);
        assert_eq!(styles.len(), 3);
        assert!(styles.get("color").is_some());
        assert!(styles.get("font-size").is_some());
        assert!(styles.get("margin").is_some());
    }
}
