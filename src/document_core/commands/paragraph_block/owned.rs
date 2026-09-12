//! Bounded, read-only owned-tree cost inspection. This is not reference validation.
use crate::{
    error::HwpError,
    model::{
        control::Control,
        document::Document,
        header_footer::MasterPage,
        paragraph::Paragraph,
        shape::{Caption, OleShape, ShapeObject, TextBox},
        table::Cell,
    },
};

enum Node<'a> {
    Paras(&'a [Paragraph]),
    Controls(&'a [Control]),
    Shapes(&'a [ShapeObject]),
    Cells(&'a [Cell]),
    Masters(&'a [MasterPage]),
    Para(&'a Paragraph),
    Control(&'a Control),
    Shape(&'a ShapeObject),
    Cell(&'a Cell),
    Master(&'a MasterPage),
    Caption(&'a Caption),
    TextBox(&'a TextBox),
    Ole(&'a OleShape),
}

#[derive(Default)]
pub(super) struct Cost {
    pub nodes: usize,
    pub depth: usize,
    pub mapping_bytes: usize,
    pub skipped_bytes: usize,
}

struct Walk {
    cost: Cost,
    max_nodes: usize,
    max_depth: usize,
    source: bool,
}

impl Walk {
    fn visit(&mut self, depth: usize) -> Result<(), HwpError> {
        self.cost.nodes = self
            .cost
            .nodes
            .checked_add(1)
            .ok_or_else(|| super::invalid("owned node count overflow"))?;
        if self.cost.nodes > self.max_nodes || depth > self.max_depth {
            return Err(super::invalid("owned node/depth budget exceeded"));
        }
        self.cost.depth = self.cost.depth.max(depth);
        if self.source {
            // Two typed paths (source and destination), each conservatively
            // charged 16 bytes per step, plus 64 bytes of per-entry storage.
            let entry = depth
                .checked_mul(32)
                .and_then(|n| n.checked_add(64))
                .ok_or_else(|| super::invalid("mapping size overflow"))?;
            self.cost.mapping_bytes = self
                .cost
                .mapping_bytes
                .checked_add(entry)
                .ok_or_else(|| super::invalid("mapping size overflow"))?;
        }
        Ok(())
    }

    fn walk(&mut self, root: Node<'_>) -> Result<(), HwpError> {
        // Slice cursors queue one sibling at a time: no allocation proportional
        // to a wide table's cell count before the node budget has been checked.
        let mut stack = vec![(root, 1usize)];
        while let Some((node, depth)) = stack.pop() {
            macro_rules! slice {
                ($slice:expr, $cursor:ident, $item:ident) => {{
                    if let Some((first, rest)) = $slice.split_first() {
                        if !rest.is_empty() {
                            stack.push((Node::$cursor(rest), depth));
                        }
                        stack.push((Node::$item(first), depth));
                    }
                    continue;
                }};
            }
            match node {
                Node::Paras(s) => slice!(s, Paras, Para),
                Node::Controls(s) => slice!(s, Controls, Control),
                Node::Shapes(s) => slice!(s, Shapes, Shape),
                Node::Cells(s) => slice!(s, Cells, Cell),
                Node::Masters(s) => slice!(s, Masters, Master),
                _ => {}
            }
            self.visit(depth)?;
            let child_depth = depth
                .checked_add(1)
                .ok_or_else(|| super::invalid("owned depth overflow"))?;
            macro_rules! child {
                ($v:expr) => {
                    stack.push(($v, child_depth))
                };
            }
            macro_rules! caption {
                ($v:expr) => {
                    if let Some(c) = $v {
                        child!(Node::Caption(c));
                    }
                };
            }
            match node {
                Node::Para(p) => {
                    if self.source {
                        let skipped = p
                            .source_line_seg_vertical_pos
                            .as_ref()
                            .map_or(0, Vec::len)
                            .checked_mul(std::mem::size_of::<i32>())
                            .ok_or_else(|| super::invalid("skipped buffer size overflow"))?;
                        self.cost.skipped_bytes = self
                            .cost
                            .skipped_bytes
                            .checked_add(skipped)
                            .ok_or_else(|| super::invalid("skipped buffer size overflow"))?;
                    }
                    child!(Node::Controls(&p.controls));
                }
                Node::Cell(c) => child!(Node::Paras(&c.paragraphs)),
                Node::Master(m) => child!(Node::Paras(&m.paragraphs)),
                Node::Caption(c) => child!(Node::Paras(&c.paragraphs)),
                Node::TextBox(t) => child!(Node::Paras(&t.paragraphs)),
                Node::Control(c) => match c {
                    Control::Table(t) => {
                        child!(Node::Cells(&t.cells));
                        caption!(t.caption.as_ref());
                    }
                    Control::Shape(s) => child!(Node::Shape(s)),
                    Control::Picture(p) => caption!(p.caption.as_ref()),
                    Control::Header(h) => child!(Node::Paras(&h.paragraphs)),
                    Control::Footer(f) => child!(Node::Paras(&f.paragraphs)),
                    Control::Footnote(n) => child!(Node::Paras(&n.paragraphs)),
                    Control::Endnote(n) => child!(Node::Paras(&n.paragraphs)),
                    Control::HiddenComment(c) => child!(Node::Paras(&c.paragraphs)),
                    Control::Field(f) => child!(Node::Paras(&f.memo_paragraphs)),
                    Control::SectionDef(s) => child!(Node::Masters(&s.master_pages)),
                    // Form's custom serde serializer allocates a sorted map.
                    // Never invoke that serializer in the read-only cost meter.
                    Control::Form(_) if self.source => {
                        return Err(super::invalid("Form is unsupported by block budgeting"));
                    }
                    Control::Form(_)
                    | Control::ColumnDef(_)
                    | Control::AutoNumber(_)
                    | Control::NewNumber(_)
                    | Control::PageNumberPos(_)
                    | Control::Bookmark(_)
                    | Control::IndexMark(_)
                    | Control::PageNumCtrl(_)
                    | Control::Hyperlink(_)
                    | Control::Ruby(_)
                    | Control::CharOverlap(_)
                    | Control::PageHide(_)
                    | Control::Equation(_)
                    | Control::Unknown(_) => {}
                },
                Node::Shape(s) => {
                    if let Some(d) = s.drawing() {
                        if let Some(t) = &d.text_box {
                            child!(Node::TextBox(t));
                        }
                        caption!(d.caption.as_ref());
                    }
                    match s {
                        ShapeObject::Group(g) => {
                            child!(Node::Shapes(&g.children));
                            caption!(g.caption.as_ref());
                        }
                        ShapeObject::Picture(p) => caption!(p.caption.as_ref()),
                        ShapeObject::Chart(c) => caption!(c.caption.as_ref()),
                        ShapeObject::Ole(o) => {
                            caption!(o.caption.as_ref());
                            if let Some(fallback) = &o.chart_switch_fallback {
                                child!(Node::Ole(fallback));
                            }
                        }
                        ShapeObject::Line(_)
                        | ShapeObject::Rectangle(_)
                        | ShapeObject::Ellipse(_)
                        | ShapeObject::Arc(_)
                        | ShapeObject::Polygon(_)
                        | ShapeObject::Curve(_) => {}
                    }
                }
                Node::Ole(o) => {
                    if let Some(t) = &o.drawing.text_box {
                        child!(Node::TextBox(t));
                    }
                    caption!(o.drawing.caption.as_ref());
                    caption!(o.caption.as_ref());
                    if let Some(f) = &o.chart_switch_fallback {
                        child!(Node::Ole(f));
                    }
                }
                Node::Paras(_)
                | Node::Controls(_)
                | Node::Shapes(_)
                | Node::Cells(_)
                | Node::Masters(_) => unreachable!("slice cursors handled above"),
            }
        }
        Ok(())
    }
}

pub(super) fn source(paras: &[Paragraph], nodes: usize, depth: usize) -> Result<Cost, HwpError> {
    let mut walk = Walk {
        cost: Cost::default(),
        max_nodes: nodes,
        max_depth: depth,
        source: true,
    };
    walk.walk(Node::Paras(paras))?;
    Ok(walk.cost)
}

pub(super) fn document(document: &Document, max_nodes: usize) -> Result<usize, HwpError> {
    let mut walk = Walk {
        cost: Cost::default(),
        max_nodes,
        max_depth: usize::MAX,
        source: false,
    };
    for section in &document.sections {
        // Charge even an empty section so a broad section list is bounded too.
        walk.visit(1)?;
        walk.walk(Node::Paras(&section.paragraphs))?;
        walk.walk(Node::Masters(&section.section_def.master_pages))?;
    }
    Ok(walk.cost.nodes)
}
