//! CSS Selectors L3/L4-subset matcher (CSS-4).
//!
//! Walks a [`SelectorList`] / [`ComplexSelector`] (CSS-3) against a
//! DOM element implementing the [`Element`] trait. The matcher is
//! intentionally generic — HTML-1 supplies the trait implementation
//! over its concrete arena DOM, and downstream tests can implement
//! the trait on whatever mock graph they want (this module's own
//! tests use a tiny `Vec<Node>`-backed mock).
//!
//! Algorithm: standard right-to-left walk per CSS Selectors §15.
//! `compounds[0]` (the subject) must match the candidate element.
//! Then for each `(combinator, compound)` pair leftward, traverse the
//! DOM in the combinator's direction and require a match somewhere
//! along that path. Descendant / subsequent-sibling combinators take
//! a "match anywhere" flavour; child / next-sibling take a single
//! step.
//!
//! UA-state pseudo-classes (`:hover`, `:focus`, `:visited`, `:checked`,
//! …) always return false. Document-level state doesn't exist in a
//! paged-PDF pipeline.

use super::selectors::{
    AnPlusB, AttributeCase, AttributeOp, AttributeSelector, Combinator, ComplexSelector,
    CompoundSelector, ElementSelector, PseudoClass, SelectorList, SubclassSelector,
};

// ─────────────────────────────────────────────────────────────────────
// Element trait — the contract HTML-1 will implement
// ─────────────────────────────────────────────────────────────────────

/// Read-only DOM element view consumed by the matcher.
///
/// Implementors provide:
/// - **Identity**: tag name, id, classes, arbitrary attribute lookup.
/// - **Tree navigation**: parent, prev/next sibling.
/// - **Position info**: the various sibling-position counts that
///   `:nth-*` and `:first-child` etc. need.
///
/// All "Self" return types use `Self` rather than a borrow so
/// implementors backed by an arena (DOM-1's planned shape) can return
/// cheap `Copy` element handles instead of borrowed references.
pub trait Element: Sized + Copy {
    /// Lowercase HTML tag name (`"div"`, `"p"`, …).
    fn local_name(&self) -> &str;
    /// `id="..."` attribute value, if present.
    fn id(&self) -> Option<&str>;
    /// True if the `class` attribute contains `class_name` as a
    /// whitespace-separated token.
    fn has_class(&self, class_name: &str) -> bool;
    /// Get an attribute value by name (case-insensitive in HTML; the
    /// matcher passes the selector's exact casing — implementor
    /// normalises).
    fn attribute(&self, name: &str) -> Option<&str>;
    /// True if the named attribute is present (regardless of value).
    fn has_attribute(&self, name: &str) -> bool;

    /// Parent element, if any. `None` for the root.
    fn parent(&self) -> Option<Self>;
    /// Previous element sibling (skip text/comment nodes).
    fn prev_element_sibling(&self) -> Option<Self>;
    /// Next element sibling.
    fn next_element_sibling(&self) -> Option<Self>;

    /// True iff this is the document root (no parent).
    fn is_root(&self) -> bool {
        self.parent().is_none()
    }
    /// True iff the element has no element children and no
    /// non-whitespace text content. CSS `:empty`.
    fn is_empty(&self) -> bool;

    /// 1-indexed sibling position among **element** siblings.
    /// Used by `:nth-child` and `:first-child`/`:last-child`.
    fn sibling_index(&self) -> usize {
        let mut count = 1usize;
        let mut cur = self.prev_element_sibling();
        while let Some(s) = cur {
            count += 1;
            cur = s.prev_element_sibling();
        }
        count
    }
    /// Total number of **element** siblings including this one.
    /// Used by `:nth-last-child` and `:last-child`.
    fn sibling_count(&self) -> usize {
        let forward = {
            let mut count = 0usize;
            let mut cur = self.next_element_sibling();
            while let Some(s) = cur {
                count += 1;
                cur = s.next_element_sibling();
            }
            count
        };
        self.sibling_index() + forward
    }
    /// 1-indexed position among siblings sharing the same `local_name`.
    fn sibling_index_of_type(&self) -> usize {
        let me = self.local_name();
        let mut count = 1usize;
        let mut cur = self.prev_element_sibling();
        while let Some(s) = cur {
            if s.local_name() == me {
                count += 1;
            }
            cur = s.prev_element_sibling();
        }
        count
    }
    /// Total number of siblings sharing the same `local_name` (incl. self).
    fn sibling_count_of_type(&self) -> usize {
        let me = self.local_name();
        let forward = {
            let mut count = 0usize;
            let mut cur = self.next_element_sibling();
            while let Some(s) = cur {
                if s.local_name() == me {
                    count += 1;
                }
                cur = s.next_element_sibling();
            }
            count
        };
        self.sibling_index_of_type() + forward
    }

    /// True iff some descendant matches `pred`. Default: depth-first
    /// walk via an explicit stack so generics don't blow up
    /// instantiation depth on `:has()` calls. Implementors with an
    /// arena can override for speed.
    fn any_descendant<F: FnMut(Self) -> bool>(&self, mut pred: F) -> bool {
        let mut stack: Vec<Self> = Vec::new();
        for c in self.element_children() {
            stack.push(c);
        }
        while let Some(node) = stack.pop() {
            if pred(node) {
                return true;
            }
            for c in node.element_children() {
                stack.push(c);
            }
        }
        false
    }

    /// Iterator over direct element children. Default walks via
    /// `next_element_sibling()` from `first_element_child()`.
    fn element_children(&self) -> ElementChildren<Self> {
        ElementChildren {
            current: self.first_element_child(),
        }
    }
    /// First element child, if any.
    fn first_element_child(&self) -> Option<Self>;
}

/// Iterator returned by [`Element::element_children`].
pub struct ElementChildren<E> {
    current: Option<E>,
}
impl<E: Element> Iterator for ElementChildren<E> {
    type Item = E;
    fn next(&mut self) -> Option<E> {
        let c = self.current?;
        self.current = c.next_element_sibling();
        Some(c)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Matching
// ─────────────────────────────────────────────────────────────────────

/// True if any selector in the list matches `element`.
pub fn match_selector_list<E: Element>(list: &SelectorList, element: E) -> bool {
    list.selectors
        .iter()
        .any(|sel| match_complex_selector(sel, element))
}

/// True if `selector` matches `element`.
pub fn match_complex_selector<E: Element>(selector: &ComplexSelector, element: E) -> bool {
    if !match_compound(&selector.compounds[0], element) {
        return false;
    }
    walk_combinators(selector, 0, element)
}

fn walk_combinators<E: Element>(
    selector: &ComplexSelector,
    matched_idx: usize,
    matched: E,
) -> bool {
    if matched_idx + 1 >= selector.compounds.len() {
        // Hit the leftmost compound — full match.
        return true;
    }
    let combinator = selector.combinators[matched_idx];
    let next_compound = &selector.compounds[matched_idx + 1];

    match combinator {
        Combinator::Descendant => {
            // Try every ancestor.
            let mut cur = matched.parent();
            while let Some(anc) = cur {
                if match_compound(next_compound, anc)
                    && walk_combinators(selector, matched_idx + 1, anc)
                {
                    return true;
                }
                cur = anc.parent();
            }
            false
        },
        Combinator::Child => {
            // Single step: parent.
            if let Some(parent) = matched.parent() {
                if match_compound(next_compound, parent) {
                    return walk_combinators(selector, matched_idx + 1, parent);
                }
            }
            false
        },
        Combinator::NextSibling => {
            // Single step: prev sibling.
            if let Some(sib) = matched.prev_element_sibling() {
                if match_compound(next_compound, sib) {
                    return walk_combinators(selector, matched_idx + 1, sib);
                }
            }
            false
        },
        Combinator::SubsequentSibling => {
            // Try every preceding sibling.
            let mut cur = matched.prev_element_sibling();
            while let Some(sib) = cur {
                if match_compound(next_compound, sib)
                    && walk_combinators(selector, matched_idx + 1, sib)
                {
                    return true;
                }
                cur = sib.prev_element_sibling();
            }
            false
        },
    }
}

fn match_compound<E: Element>(compound: &CompoundSelector, element: E) -> bool {
    if let Some(elt_sel) = &compound.element {
        if !match_element_selector(elt_sel, element) {
            return false;
        }
    }
    for sub in &compound.subclasses {
        if !match_subclass(sub, element) {
            return false;
        }
    }
    true
}

fn match_element_selector<E: Element>(sel: &ElementSelector, element: E) -> bool {
    match sel {
        ElementSelector::Universal => true,
        ElementSelector::Type(name) => element.local_name().eq_ignore_ascii_case(name),
    }
}

fn match_subclass<E: Element>(sub: &SubclassSelector, element: E) -> bool {
    match sub {
        SubclassSelector::Class(c) => element.has_class(c),
        SubclassSelector::Id(id) => element.id().map(|s| s == id).unwrap_or(false),
        SubclassSelector::Attribute(a) => match_attribute(a, element),
        SubclassSelector::PseudoClass(pc) => match_pseudo_class(pc, element),
        // Pseudo-elements are *generated content boxes*, not real
        // elements. The cascade asks "does this element have a ::before
        // box?" by collecting matching ::before/::after/::first-line/
        // ::first-letter rules separately. From the matcher's POV we
        // just say "yes the host element matches" — the selector with
        // ::after applies to the element, and the cascade plumbing
        // routes the resulting style to the generated box. v0.3.35 ships
        // ::before/::after content; ::first-line/::first-letter wire in
        // during inline formatting (LAYOUT-3).
        SubclassSelector::PseudoElement(_) => true,
    }
}

fn match_attribute<E: Element>(a: &AttributeSelector, element: E) -> bool {
    let Some(actual) = element.attribute(&a.name) else {
        return a.op.is_none() && element.has_attribute(&a.name);
    };
    let Some(op) = a.op else {
        // Presence-only — already true since attribute() returned Some.
        return true;
    };
    let Some(expected) = a.value.as_deref() else {
        return false;
    };
    let case_insensitive = match a.case {
        AttributeCase::Sensitive => false,
        AttributeCase::Insensitive => true,
        // CSS Selectors L4 §6.3.6: HTML attributes are case-insensitive
        // by default for a known list (type, lang, dir, …) but case-
        // sensitive otherwise. v0.3.35 takes the simple-and-correct path
        // of case-sensitive default; the `i` flag overrides to insensitive.
        AttributeCase::Default => false,
    };
    let cmp = |a: &str, b: &str| {
        if case_insensitive {
            a.eq_ignore_ascii_case(b)
        } else {
            a == b
        }
    };
    match op {
        AttributeOp::Equals => cmp(actual, expected),
        AttributeOp::Includes => actual.split_whitespace().any(|tok| cmp(tok, expected)),
        AttributeOp::DashMatch => {
            cmp(actual, expected)
                || actual
                    .strip_prefix(expected)
                    .map(|rest| rest.starts_with('-'))
                    .unwrap_or(false)
        },
        AttributeOp::Prefix => {
            !expected.is_empty()
                && (if case_insensitive {
                    actual
                        .to_ascii_lowercase()
                        .starts_with(&expected.to_ascii_lowercase())
                } else {
                    actual.starts_with(expected)
                })
        },
        AttributeOp::Suffix => {
            !expected.is_empty()
                && (if case_insensitive {
                    actual
                        .to_ascii_lowercase()
                        .ends_with(&expected.to_ascii_lowercase())
                } else {
                    actual.ends_with(expected)
                })
        },
        AttributeOp::Substring => {
            !expected.is_empty()
                && (if case_insensitive {
                    actual
                        .to_ascii_lowercase()
                        .contains(&expected.to_ascii_lowercase())
                } else {
                    actual.contains(expected)
                })
        },
    }
}

fn match_pseudo_class<E: Element>(pc: &PseudoClass, element: E) -> bool {
    match pc {
        PseudoClass::Root => element.is_root(),
        PseudoClass::Empty => element.is_empty(),
        PseudoClass::FirstChild => element.sibling_index() == 1,
        PseudoClass::LastChild => element.sibling_index() == element.sibling_count(),
        PseudoClass::OnlyChild => element.sibling_count() == 1,
        PseudoClass::FirstOfType => element.sibling_index_of_type() == 1,
        PseudoClass::LastOfType => {
            element.sibling_index_of_type() == element.sibling_count_of_type()
        },
        PseudoClass::OnlyOfType => element.sibling_count_of_type() == 1,
        PseudoClass::NthChild(an_b) => an_plus_b_matches(*an_b, element.sibling_index()),
        PseudoClass::NthLastChild(an_b) => {
            an_plus_b_matches(*an_b, element.sibling_count() - element.sibling_index() + 1)
        },
        PseudoClass::NthOfType(an_b) => an_plus_b_matches(*an_b, element.sibling_index_of_type()),
        PseudoClass::NthLastOfType(an_b) => an_plus_b_matches(
            *an_b,
            element.sibling_count_of_type() - element.sibling_index_of_type() + 1,
        ),
        PseudoClass::Is(list) | PseudoClass::Where(list) => match_selector_list(list, element),
        PseudoClass::Not(list) => !match_selector_list(list, element),
        PseudoClass::Has(list) => element.any_descendant(|d| match_selector_list(list, d)),
        // No UA state in PDF rendering.
        PseudoClass::UaState(_) | PseudoClass::Functional { .. } => false,
    }
}

/// True if `index` (1-based) satisfies `An+B` for some integer n ≥ 0.
fn an_plus_b_matches(an_b: AnPlusB, index: usize) -> bool {
    let i = index as i64;
    let a = an_b.a as i64;
    let b = an_b.b as i64;
    if a == 0 {
        return i == b;
    }
    // i = a*n + b ⇒ n = (i - b) / a, must be a non-negative integer.
    let diff = i - b;
    if diff == 0 {
        return true;
    }
    if a.signum() != diff.signum() {
        return false;
    }
    diff % a == 0
}

// ─────────────────────────────────────────────────────────────────────
// Tests — uses a small Vec<Node>-backed mock DOM
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::parser::parse_stylesheet;
    use crate::html_css::css::parser::Rule;
    use crate::html_css::css::selectors::parse_selector_list;

    /// One node in the mock DOM.
    struct MockNode {
        tag: String,
        id: Option<String>,
        classes: Vec<String>,
        attrs: Vec<(String, String)>,
        parent: Option<usize>,
        children: Vec<usize>,
        text_content_nonempty: bool,
    }

    struct MockDom {
        nodes: Vec<MockNode>,
    }

    impl MockDom {
        fn new() -> Self {
            Self { nodes: Vec::new() }
        }
        fn add(
            &mut self,
            parent: Option<usize>,
            tag: &str,
            id: Option<&str>,
            classes: &[&str],
            attrs: &[(&str, &str)],
        ) -> usize {
            let idx = self.nodes.len();
            self.nodes.push(MockNode {
                tag: tag.to_ascii_lowercase(),
                id: id.map(|s| s.to_string()),
                classes: classes.iter().map(|s| s.to_string()).collect(),
                attrs: attrs
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                parent,
                children: Vec::new(),
                text_content_nonempty: false,
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
        fn attribute(&self, name: &str) -> Option<&str> {
            self.dom.nodes[self.idx]
                .attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v.as_str())
        }
        fn has_attribute(&self, name: &str) -> bool {
            self.attribute(name).is_some()
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
            if pos == 0 {
                None
            } else {
                Some(MockEl {
                    dom: self.dom,
                    idx: kids[pos - 1],
                })
            }
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
                && !self.dom.nodes[self.idx].text_content_nonempty
        }
        fn first_element_child(&self) -> Option<Self> {
            self.dom.nodes[self.idx].children.first().map(|&k| MockEl {
                dom: self.dom,
                idx: k,
            })
        }
    }

    fn parse_one_selector(src: &str) -> super::super::selectors::SelectorList {
        let ss = parse_stylesheet(src).unwrap();
        let r = match &ss.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        parse_selector_list(&r.prelude).unwrap()
    }

    fn matches(src: &str, dom: &MockDom, idx: usize) -> bool {
        let list = parse_one_selector(&format!("{src} {{}}"));
        match_selector_list(&list, MockEl { dom, idx })
    }

    fn build_basic_dom() -> (MockDom, Vec<usize>) {
        let mut d = MockDom::new();
        // <html><body><div id="main" class="container">
        //                <p class="lead">first</p>
        //                <p>second</p>
        //                <span data-x="42" lang="en-US">third</span>
        //              </div></body></html>
        let html = d.add(None, "html", None, &[], &[]);
        let body = d.add(Some(html), "body", None, &[], &[]);
        let div = d.add(Some(body), "div", Some("main"), &["container"], &[]);
        let p1 = d.add(Some(div), "p", None, &["lead"], &[]);
        let p2 = d.add(Some(div), "p", None, &[], &[]);
        let span = d.add(Some(div), "span", None, &[], &[("data-x", "42"), ("lang", "en-US")]);
        (d, vec![html, body, div, p1, p2, span])
    }

    #[test]
    fn match_type() {
        let (d, ix) = build_basic_dom();
        assert!(matches("p", &d, ix[3]));
        assert!(matches("p", &d, ix[4]));
        assert!(!matches("p", &d, ix[5]));
    }

    #[test]
    fn match_universal() {
        let (d, ix) = build_basic_dom();
        for &i in &ix {
            assert!(matches("*", &d, i));
        }
    }

    #[test]
    fn match_class() {
        let (d, ix) = build_basic_dom();
        assert!(matches(".lead", &d, ix[3]));
        assert!(!matches(".lead", &d, ix[4]));
        assert!(matches(".container", &d, ix[2]));
    }

    #[test]
    fn match_id() {
        let (d, ix) = build_basic_dom();
        assert!(matches("#main", &d, ix[2]));
        assert!(!matches("#main", &d, ix[3]));
    }

    #[test]
    fn match_compound() {
        let (d, ix) = build_basic_dom();
        assert!(matches("div.container", &d, ix[2]));
        assert!(matches("div#main.container", &d, ix[2]));
        assert!(!matches("p.container", &d, ix[3]));
    }

    #[test]
    fn match_descendant_combinator() {
        let (d, ix) = build_basic_dom();
        // body p — both p's are descendants of body.
        assert!(matches("body p", &d, ix[3]));
        assert!(matches("body p", &d, ix[4]));
        // body span — span is a descendant of body too.
        assert!(matches("body span", &d, ix[5]));
        // div.container > p — only direct children.
        assert!(matches("div.container > p", &d, ix[3]));
        assert!(matches("div.container > p", &d, ix[4]));
    }

    #[test]
    fn match_child_combinator_negative() {
        let (d, ix) = build_basic_dom();
        // body > p — p is grandchild of body, not child.
        assert!(!matches("body > p", &d, ix[3]));
    }

    #[test]
    fn match_next_sibling_combinator() {
        let (d, ix) = build_basic_dom();
        // p.lead + p — p2 directly follows p.lead.
        assert!(matches(".lead + p", &d, ix[4]));
        // .lead + span — span is two siblings later.
        assert!(!matches(".lead + span", &d, ix[5]));
    }

    #[test]
    fn match_subsequent_sibling_combinator() {
        let (d, ix) = build_basic_dom();
        // .lead ~ span — span is a later sibling.
        assert!(matches(".lead ~ span", &d, ix[5]));
        assert!(matches(".lead ~ p", &d, ix[4]));
    }

    #[test]
    fn match_attribute_presence() {
        let (d, ix) = build_basic_dom();
        assert!(matches("[data-x]", &d, ix[5]));
        assert!(!matches("[data-x]", &d, ix[3]));
    }

    #[test]
    fn match_attribute_equals() {
        let (d, ix) = build_basic_dom();
        assert!(matches(r#"[data-x="42"]"#, &d, ix[5]));
        assert!(!matches(r#"[data-x="99"]"#, &d, ix[5]));
    }

    #[test]
    fn match_attribute_dash_match() {
        let (d, ix) = build_basic_dom();
        // lang|=en matches "en-US".
        assert!(matches(r#"[lang|="en"]"#, &d, ix[5]));
    }

    #[test]
    fn match_attribute_prefix_suffix_substring() {
        let (d, ix) = build_basic_dom();
        assert!(matches(r#"[lang^="en"]"#, &d, ix[5]));
        assert!(matches(r#"[lang$="US"]"#, &d, ix[5]));
        assert!(matches(r#"[lang*="-"]"#, &d, ix[5]));
    }

    #[test]
    fn match_first_child_last_child_only_child() {
        let (d, ix) = build_basic_dom();
        // p.lead is :first-child of div.
        assert!(matches(":first-child", &d, ix[3]));
        // span is :last-child of div.
        assert!(matches(":last-child", &d, ix[5]));
        // div is :only-child of body.
        assert!(matches(":only-child", &d, ix[2]));
    }

    #[test]
    fn match_first_of_type() {
        let (d, ix) = build_basic_dom();
        assert!(matches("p:first-of-type", &d, ix[3]));
        assert!(!matches("p:first-of-type", &d, ix[4]));
        assert!(matches("span:first-of-type", &d, ix[5])); // sole span
    }

    #[test]
    fn match_nth_child() {
        let (d, ix) = build_basic_dom();
        // div has children at positions 1=p.lead, 2=p, 3=span.
        assert!(matches(":nth-child(1)", &d, ix[3]));
        assert!(matches(":nth-child(2)", &d, ix[4]));
        assert!(matches(":nth-child(3)", &d, ix[5]));
        assert!(matches(":nth-child(odd)", &d, ix[3]));
        assert!(!matches(":nth-child(odd)", &d, ix[4]));
        assert!(matches(":nth-child(odd)", &d, ix[5]));
    }

    #[test]
    fn match_is_takes_max_specificity_but_or_semantics() {
        let (d, ix) = build_basic_dom();
        assert!(matches(":is(p, span)", &d, ix[3]));
        assert!(matches(":is(p, span)", &d, ix[5]));
        assert!(!matches(":is(p, span)", &d, ix[2]));
    }

    #[test]
    fn match_not() {
        let (d, ix) = build_basic_dom();
        assert!(matches("p:not(.lead)", &d, ix[4]));
        assert!(!matches("p:not(.lead)", &d, ix[3]));
    }

    #[test]
    fn match_has_descendant() {
        let (d, ix) = build_basic_dom();
        // div has a span descendant.
        assert!(matches("div:has(span)", &d, ix[2]));
        // body has a span descendant too.
        assert!(matches("body:has(span)", &d, ix[1]));
        // p does not have any descendants.
        assert!(!matches("p:has(span)", &d, ix[3]));
    }

    #[test]
    fn match_root() {
        let (d, ix) = build_basic_dom();
        assert!(matches(":root", &d, ix[0]));
        assert!(!matches(":root", &d, ix[1]));
    }

    #[test]
    fn ua_state_pseudos_never_match() {
        let (d, ix) = build_basic_dom();
        for s in [":hover", ":focus", ":visited", ":checked"] {
            assert!(!matches(s, &d, ix[3]), "{s} matched in PDF context");
        }
    }

    #[test]
    fn long_descendant_chain_matches() {
        let (d, ix) = build_basic_dom();
        // html body div p
        assert!(matches("html body div p", &d, ix[4]));
    }

    #[test]
    fn match_empty() {
        let (d, ix) = build_basic_dom();
        // p1, p2, span have no children → :empty.
        assert!(matches(":empty", &d, ix[3]));
        // div has children → not empty.
        assert!(!matches(":empty", &d, ix[2]));
    }
}
