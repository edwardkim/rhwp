//! Mutable owned-tree walk shared by identity assignment and reference fixup.
use crate::error::HwpError;
use crate::model::{
    control::Control,
    paragraph::Paragraph,
    shape::{Caption, ShapeObject},
};

pub(super) enum Node<'a> {
    Paragraph(&'a mut Paragraph),
    Control(&'a mut Control),
    Shape(&'a mut ShapeObject),
}

fn paragraphs<'a>(stack: &mut Vec<Node<'a>>, paras: &'a mut [Paragraph]) {
    stack.extend(paras.iter_mut().rev().map(Node::Paragraph));
}

fn caption<'a>(stack: &mut Vec<Node<'a>>, value: Option<&'a mut Caption>) {
    if let Some(value) = value {
        paragraphs(stack, &mut value.paragraphs);
    }
}

pub(super) fn walk(
    paras: &mut [Paragraph],
    mut visit: impl FnMut(&mut Node<'_>) -> Result<(), HwpError>,
) -> Result<(), HwpError> {
    let mut stack = Vec::new();
    paragraphs(&mut stack, paras);
    while let Some(mut node) = stack.pop() {
        visit(&mut node)?;
        match node {
            Node::Paragraph(para) => {
                stack.extend(para.controls.iter_mut().rev().map(Node::Control))
            }
            Node::Control(control) => match control {
                Control::Table(table) => {
                    caption(&mut stack, table.caption.as_mut());
                    for cell in table.cells.iter_mut().rev() {
                        paragraphs(&mut stack, &mut cell.paragraphs);
                    }
                }
                Control::Shape(shape) => stack.push(Node::Shape(shape)),
                Control::Picture(pic) => caption(&mut stack, pic.caption.as_mut()),
                Control::Header(h) => paragraphs(&mut stack, &mut h.paragraphs),
                Control::Footer(h) => paragraphs(&mut stack, &mut h.paragraphs),
                Control::Footnote(n) => paragraphs(&mut stack, &mut n.paragraphs),
                Control::Endnote(n) => paragraphs(&mut stack, &mut n.paragraphs),
                Control::HiddenComment(c) => paragraphs(&mut stack, &mut c.paragraphs),
                Control::Field(f) => paragraphs(&mut stack, &mut f.memo_paragraphs),
                Control::SectionDef(s) => {
                    for master in s.master_pages.iter_mut().rev() {
                        paragraphs(&mut stack, &mut master.paragraphs);
                    }
                }
                _ => {}
            },
            Node::Shape(shape) => match shape {
                ShapeObject::Group(group) => {
                    caption(&mut stack, group.caption.as_mut());
                    stack.extend(group.children.iter_mut().rev().map(Node::Shape));
                }
                ShapeObject::Picture(pic) => caption(&mut stack, pic.caption.as_mut()),
                ShapeObject::Chart(chart) => {
                    caption(&mut stack, chart.caption.as_mut());
                    drawing(&mut stack, &mut chart.drawing);
                }
                ShapeObject::Ole(ole) => {
                    caption(&mut stack, ole.caption.as_mut());
                    drawing(&mut stack, &mut ole.drawing);
                }
                _ => {
                    if let Some(d) = shape.drawing_mut() {
                        drawing(&mut stack, d);
                    }
                }
            },
        }
    }
    Ok(())
}

fn drawing<'a>(stack: &mut Vec<Node<'a>>, d: &'a mut crate::model::shape::DrawingObjAttr) {
    caption(stack, d.caption.as_mut());
    if let Some(textbox) = &mut d.text_box {
        paragraphs(stack, &mut textbox.paragraphs);
    }
}
