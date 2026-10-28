//! Flat arena DOM built from the HTML tokenizer (HTML-2).
//!
//! Trades expressiveness for cheap traversal: every node is an entry
//! in `Dom::nodes`; references between nodes are `NodeId` (a `u32`
//! index). Sibling iteration walks parent's `children` vector directly.
//!
//! [`DomElement`] is a `Copy` handle pairing a `NodeId` with a
//! `&Dom`. It implements [`super::super::css::Element`] so the cascade
//! engine matches selectors against real document nodes — closing the
//! loop from CSS-4 (Element trait + matcher) through CSS-5 (cascade).
//!
//! Tree construction handles the common implicit-close cases that
//! showed up in the v0.3.35 fixtures (`<p>` auto-closes when another
//! `<p>` opens; `<li>` auto-closes when another `<li>` opens). The
//! full WHATWG insertion-mode state machine is *not* implemented —
//! v0.3.35 documents that callers needing wild-HTML tolerance should
//! preprocess.

use crate::html_css::css::matcher::{Element, ElementChildren};

use super::tokenizer::{tokenize, HtmlToken};

/// Index into [`Dom::nodes`].
pub type NodeId = u32;

/// What kind of node.
#[derive(Debug, Clone)]
pub enum NodeKind {
    /// The document root. Always at `NodeId(0)`.
    Document,
    /// `<tag attrs>...</tag>`.
    Element {
        /// Lowercase tag name.
        tag: String,
        /// Attribute map. Order is irrelevant for the matcher; we use
        /// a `Vec` because attribute counts are typically tiny and
        /// linear scan beats a HashMap for sub-10-attr nodes.
        attrs: Vec<(String, String)>,
    },
    /// Text content.
    Text(String),
    /// HTML comment — preserved for round-tripping but layout ignores.
    Comment(String),
    /// Raw text from a `<style>` or `<script>` element.
    RawText {
        /// `"style"` or `"script"`.
        host_tag: String,
        /// Body bytes.
        body: String,
    },
}

/// One node in the DOM.
#[derive(Debug, Clone)]
pub struct Node {
    /// What kind of node.
    pub kind: NodeKind,
    /// Parent node, or `None` for the document root.
    pub parent: Option<NodeId>,
    /// Direct children in source order.
    pub children: Vec<NodeId>,
}

/// The whole DOM.
#[derive(Debug, Clone, Default)]
pub struct Dom {
    /// All nodes, including the synthetic Document root at index 0.
    pub nodes: Vec<Node>,
}

impl Dom {
    /// Construct a fresh DOM with just the document root.
    pub fn new() -> Self {
        let mut d = Self {
            nodes: Vec::with_capacity(64),
        };
        d.nodes.push(Node {
            kind: NodeKind::Document,
            parent: None,
            children: Vec::new(),
        });
        d
    }

    /// Document root.
    pub const ROOT: NodeId = 0;

    /// Get a node by id.
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id as usize]
    }

    /// Iterate all element nodes in document order.
    pub fn iter_elements(&self) -> impl Iterator<Item = NodeId> + '_ {
        (0..self.nodes.len() as NodeId)
            .filter(move |&i| matches!(self.nodes[i as usize].kind, NodeKind::Element { .. }))
    }

    /// Get a [`DomElement`] handle for the given id, returning `None`
    /// if the id doesn't point to an element.
    pub fn element(&self, id: NodeId) -> Option<DomElement<'_>> {
        if matches!(self.nodes[id as usize].kind, NodeKind::Element { .. }) {
            Some(DomElement { dom: self, id })
        } else {
            None
        }
    }

    /// Find the first element matching the predicate (DFS).
    pub fn find_element(
        &self,
        mut pred: impl FnMut(&str, &[(String, String)]) -> bool,
    ) -> Option<NodeId> {
        for id in self.iter_elements() {
            if let NodeKind::Element { tag, attrs } = &self.nodes[id as usize].kind {
                if pred(tag, attrs) {
                    return Some(id);
                }
            }
        }
        None
    }

    /// Find the first element with the given lowercase tag name.
    pub fn find_by_tag(&self, name: &str) -> Option<NodeId> {
        self.find_element(|tag, _| tag == name)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tree construction
// ─────────────────────────────────────────────────────────────────────

/// Tags that close any currently-open `<p>` when they open. The HTML5
/// spec's full list is bigger; this covers what HTML→PDF inputs hit.
const CLOSES_P_ON_OPEN: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "details",
    "div",
    "dl",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "main",
    "menu",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "table",
    "ul",
];

/// Parse an HTML source string into a [`Dom`].
pub fn parse_document(input: &str) -> Dom {
    let tokens = tokenize(input);
    let mut dom = Dom::new();
    let mut stack: Vec<NodeId> = vec![Dom::ROOT];

    for tok in tokens {
        match tok {
            HtmlToken::Doctype(_) | HtmlToken::Eof => {
                // Doctype is ignored; Eof terminates outer loop.
                if matches!(tok, HtmlToken::Eof) {
                    break;
                }
            },
            HtmlToken::Comment(body) => {
                let parent = *stack.last().unwrap();
                push_child(
                    &mut dom,
                    parent,
                    Node {
                        kind: NodeKind::Comment(body),
                        parent: Some(parent),
                        children: Vec::new(),
                    },
                );
            },
            HtmlToken::Text(s) => {
                if s.is_empty() {
                    continue;
                }
                let parent = *stack.last().unwrap();
                push_child(
                    &mut dom,
                    parent,
                    Node {
                        kind: NodeKind::Text(s),
                        parent: Some(parent),
                        children: Vec::new(),
                    },
                );
            },
            HtmlToken::RawText { host_tag, body } => {
                // Synthesize a host element with one RawText child so
                // CSS-3 (selectors) can still match `style`, `script`,
                // and HTML-3 can extract the body via NodeKind::RawText
                // child.
                let parent = *stack.last().unwrap();
                let host = push_child(
                    &mut dom,
                    parent,
                    Node {
                        kind: NodeKind::Element {
                            tag: host_tag.clone(),
                            attrs: Vec::new(),
                        },
                        parent: Some(parent),
                        children: Vec::new(),
                    },
                );
                push_child(
                    &mut dom,
                    host,
                    Node {
                        kind: NodeKind::RawText { host_tag, body },
                        parent: Some(host),
                        children: Vec::new(),
                    },
                );
            },
            HtmlToken::StartTag {
                name,
                attrs,
                self_closing,
            } => {
                // Implicit-close handling.
                if CLOSES_P_ON_OPEN.contains(&name.as_str()) && stack_top_is(&dom, &stack, "p") {
                    stack.pop();
                }
                if name == "li" && stack_top_is(&dom, &stack, "li") {
                    stack.pop();
                }
                let parent = *stack.last().unwrap();
                let new_id = push_child(
                    &mut dom,
                    parent,
                    Node {
                        kind: NodeKind::Element {
                            tag: name.clone(),
                            attrs,
                        },
                        parent: Some(parent),
                        children: Vec::new(),
                    },
                );
                if !self_closing {
                    stack.push(new_id);
                }
            },
            HtmlToken::EndTag { name } => {
                // Pop until we find a matching open tag, or until we
                // can't (forgive over-eager closes by leaving the
                // stack alone).
                let mut found_at: Option<usize> = None;
                for (i, &nid) in stack.iter().enumerate().rev() {
                    if let NodeKind::Element { tag, .. } = &dom.nodes[nid as usize].kind {
                        if tag == &name {
                            found_at = Some(i);
                            break;
                        }
                    }
                }
                if let Some(i) = found_at {
                    while stack.len() > i {
                        stack.pop();
                    }
                }
            },
        }
    }

    dom
}

fn push_child(dom: &mut Dom, parent: NodeId, mut node: Node) -> NodeId {
    let id = dom.nodes.len() as NodeId;
    node.parent = Some(parent);
    dom.nodes.push(node);
    dom.nodes[parent as usize].children.push(id);
    id
}

fn stack_top_is(dom: &Dom, stack: &[NodeId], tag_name: &str) -> bool {
    let Some(&top) = stack.last() else {
        return false;
    };
    matches!(
        &dom.nodes[top as usize].kind,
        NodeKind::Element { tag, .. } if tag == tag_name
    )
}

// ─────────────────────────────────────────────────────────────────────
// DomElement — Copy handle implementing the CSS-4 Element trait
// ─────────────────────────────────────────────────────────────────────

/// Cheap (`Copy`) handle that pairs a [`NodeId`] with the [`Dom`]
/// arena. Implements [`crate::html_css::css::matcher::Element`] so the
/// CSS cascade matches selectors against real document nodes.
#[derive(Debug, Clone, Copy)]
pub struct DomElement<'a> {
    /// Backing arena.
    pub dom: &'a Dom,
    /// Node id (must point to an Element).
    pub id: NodeId,
}

impl<'a> DomElement<'a> {
    fn node(&self) -> &Node {
        &self.dom.nodes[self.id as usize]
    }

    fn elem(&self) -> (&str, &[(String, String)]) {
        match &self.node().kind {
            NodeKind::Element { tag, attrs } => (tag, attrs),
            _ => panic!("DomElement points to non-element node"),
        }
    }

    /// All attribute (name, value) pairs.
    pub fn attrs(&self) -> &[(String, String)] {
        self.elem().1
    }
}

impl<'a> Element for DomElement<'a> {
    fn local_name(&self) -> &str {
        self.elem().0
    }
    fn id(&self) -> Option<&str> {
        self.elem()
            .1
            .iter()
            .find(|(k, _)| k == "id")
            .map(|(_, v)| v.as_str())
    }
    fn has_class(&self, c: &str) -> bool {
        self.elem()
            .1
            .iter()
            .find(|(k, _)| k == "class")
            .map(|(_, v)| v.split_ascii_whitespace().any(|t| t == c))
            .unwrap_or(false)
    }
    fn attribute(&self, name: &str) -> Option<&str> {
        self.elem()
            .1
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
    fn has_attribute(&self, name: &str) -> bool {
        self.attribute(name).is_some()
    }
    fn parent(&self) -> Option<Self> {
        let p = self.node().parent?;
        // Skip past the synthetic Document root — selectors expect a
        // None parent at the html element so :root matches it.
        if matches!(self.dom.nodes[p as usize].kind, NodeKind::Document) {
            None
        } else {
            Some(DomElement {
                dom: self.dom,
                id: p,
            })
        }
    }
    fn prev_element_sibling(&self) -> Option<Self> {
        let p = self.node().parent?;
        let kids = &self.dom.nodes[p as usize].children;
        let pos = kids.iter().position(|&k| k == self.id)?;
        for &kid in kids[..pos].iter().rev() {
            if matches!(self.dom.nodes[kid as usize].kind, NodeKind::Element { .. }) {
                return Some(DomElement {
                    dom: self.dom,
                    id: kid,
                });
            }
        }
        None
    }
    fn next_element_sibling(&self) -> Option<Self> {
        let p = self.node().parent?;
        let kids = &self.dom.nodes[p as usize].children;
        let pos = kids.iter().position(|&k| k == self.id)?;
        for &kid in &kids[pos + 1..] {
            if matches!(self.dom.nodes[kid as usize].kind, NodeKind::Element { .. }) {
                return Some(DomElement {
                    dom: self.dom,
                    id: kid,
                });
            }
        }
        None
    }
    fn is_empty(&self) -> bool {
        // No element children and no non-whitespace text.
        for &kid in &self.node().children {
            match &self.dom.nodes[kid as usize].kind {
                NodeKind::Element { .. } => return false,
                NodeKind::Text(s) if !s.trim().is_empty() => return false,
                _ => {},
            }
        }
        true
    }
    fn first_element_child(&self) -> Option<Self> {
        for &kid in &self.node().children {
            if matches!(self.dom.nodes[kid as usize].kind, NodeKind::Element { .. }) {
                return Some(DomElement {
                    dom: self.dom,
                    id: kid,
                });
            }
        }
        None
    }
}

// Manual default implementation of element_children to inherit the
// trait's blanket version — the trait method does the right thing
// already.
impl<'a> DomElement<'a> {
    /// Iterator alias matching [`Element::element_children`].
    pub fn children_iter(&self) -> ElementChildren<DomElement<'a>> {
        Element::element_children(self)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::matcher::{match_selector_list, Element as CssElement};
    use crate::html_css::css::parser::{parse_stylesheet, Rule};
    use crate::html_css::css::selectors::parse_selector_list;

    fn dom(html: &str) -> Dom {
        parse_document(html)
    }

    fn first_element(d: &Dom, tag: &str) -> NodeId {
        d.find_by_tag(tag)
            .unwrap_or_else(|| panic!("must find <{tag}>"))
    }

    #[test]
    fn parse_simple_paragraph() {
        let d = dom("<p>hello</p>");
        let p = first_element(&d, "p");
        let kids = &d.node(p).children;
        assert_eq!(kids.len(), 1);
        match &d.node(kids[0]).kind {
            NodeKind::Text(s) => assert_eq!(s, "hello"),
            other => panic!("expected text, got {other:?}"),
        }
    }

    #[test]
    fn parse_attributes() {
        let d = dom(r#"<a href="x" class="foo bar" id="z">link</a>"#);
        let a = first_element(&d, "a");
        let el = d.element(a).unwrap();
        assert_eq!(el.id(), Some("z"));
        assert!(el.has_class("foo"));
        assert!(el.has_class("bar"));
        assert!(!el.has_class("baz"));
        assert_eq!(el.attribute("href"), Some("x"));
    }

    #[test]
    fn nested_structure() {
        let d = dom("<div><p>a</p><p>b</p></div>");
        let div = first_element(&d, "div");
        let kids = &d.node(div).children;
        assert_eq!(kids.len(), 2);
        for &k in kids {
            assert!(matches!(d.node(k).kind, NodeKind::Element { ref tag, .. } if tag == "p"));
        }
    }

    #[test]
    fn implicit_p_close_on_new_block() {
        let d = dom("<p>first<p>second");
        // Should produce two sibling <p>s, not nested.
        let body_or_root = Dom::ROOT;
        let kids = &d.node(body_or_root).children;
        assert_eq!(
            kids.iter()
                .filter(
                    |&&k| matches!(d.node(k).kind, NodeKind::Element { ref tag, .. } if tag == "p")
                )
                .count(),
            2
        );
    }

    #[test]
    fn implicit_li_close() {
        let d = dom("<ul><li>a<li>b<li>c</ul>");
        let ul = first_element(&d, "ul");
        let kids = &d.node(ul).children;
        let li_count = kids
            .iter()
            .filter(
                |&&k| matches!(d.node(k).kind, NodeKind::Element { ref tag, .. } if tag == "li"),
            )
            .count();
        assert_eq!(li_count, 3);
    }

    #[test]
    fn void_element_no_close_needed() {
        let d = dom("<div><br><br><br></div>");
        let div = first_element(&d, "div");
        assert_eq!(d.node(div).children.len(), 3);
    }

    #[test]
    fn style_block_preserved_as_raw_text() {
        let d = dom("<head><style>p{color:red;}</style></head>");
        let style = first_element(&d, "style");
        let kid = d.node(style).children[0];
        match &d.node(kid).kind {
            NodeKind::RawText { body, .. } => assert!(body.contains("color:red")),
            other => panic!("expected RawText, got {other:?}"),
        }
    }

    #[test]
    fn entities_decoded_in_text() {
        let d = dom("<p>Bread &amp; butter</p>");
        let p = first_element(&d, "p");
        let kid = d.node(p).children[0];
        if let NodeKind::Text(s) = &d.node(kid).kind {
            assert_eq!(s, "Bread & butter");
        } else {
            panic!();
        }
    }

    // ---- Element trait integration with the CSS matcher -----------

    #[test]
    fn cascade_matches_against_dom() {
        let d = dom(
            r#"<html><body><div id="main" class="container"><p class="lead">x</p></div></body></html>"#,
        );
        let p = first_element(&d, "p");
        let pe = d.element(p).unwrap();

        let ss = parse_stylesheet("p.lead {}").unwrap();
        let r = match &ss.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        let list = parse_selector_list(&r.prelude).unwrap();
        assert!(match_selector_list(&list, pe));
    }

    #[test]
    fn descendant_combinator_via_dom() {
        let d = dom(r#"<html><body><div id="main"><span><a>x</a></span></div></body></html>"#);
        let a = first_element(&d, "a");
        let ae = d.element(a).unwrap();

        let ss = parse_stylesheet("#main a {}").unwrap();
        let r = match &ss.rules[0] {
            Rule::Qualified(q) => q,
            _ => unreachable!(),
        };
        let list = parse_selector_list(&r.prelude).unwrap();
        assert!(match_selector_list(&list, ae));
    }

    #[test]
    fn nth_child_via_dom() {
        let d = dom("<ul><li>a</li><li>b</li><li>c</li></ul>");
        let lis: Vec<NodeId> = d
            .iter_elements()
            .filter(
                |&id| matches!(d.node(id).kind, NodeKind::Element { ref tag, .. } if tag == "li"),
            )
            .collect();
        assert_eq!(lis.len(), 3);
        let pe2 = d.element(lis[1]).unwrap();
        assert_eq!(pe2.sibling_index(), 2);
    }

    #[test]
    fn parent_skips_document_root() {
        let d = dom("<html><body></body></html>");
        let html = d.element(first_element(&d, "html")).unwrap();
        // <html>'s parent is the synthetic Document, but we mask that
        // out so :root matches.
        assert!(html.parent().is_none());
    }

    #[test]
    fn case_insensitive_attribute_lookup() {
        let d = dom(r#"<input TYPE="text">"#);
        let inp = d.element(first_element(&d, "input")).unwrap();
        // attribute() lookup is case-insensitive.
        assert_eq!(inp.attribute("type"), Some("text"));
        assert_eq!(inp.attribute("TYPE"), Some("text"));
    }
}
