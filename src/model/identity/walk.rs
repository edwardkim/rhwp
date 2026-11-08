//! Mutable owned-tree walk for identity assignment, reference fixup and cell formatting.
use crate::error::HwpError;
use crate::model::{
    control::Control,
    paragraph::Paragraph,
    shape::{Caption, ShapeObject},
};

pub(crate) enum Node<'a> {
    Paragraphs(&'a mut [Paragraph]),
    Paragraph(&'a mut Paragraph),
    Control(&'a mut Control),
    Shape(&'a mut ShapeObject),
}

fn paragraphs<'a>(stack: &mut Vec<Node<'a>>, paras: &'a mut [Paragraph]) {
    stack.push(Node::Paragraphs(paras));
}

fn caption<'a>(stack: &mut Vec<Node<'a>>, value: Option<&'a mut Caption>) {
    if let Some(value) = value {
        paragraphs(stack, &mut value.paragraphs);
    }
}

pub(crate) fn walk(
    paras: &mut [Paragraph],
    mut visit: impl FnMut(&mut Node<'_>) -> Result<(), HwpError>,
) -> Result<(), HwpError> {
    let mut stack = Vec::new();
    paragraphs(&mut stack, paras);
    while let Some(mut node) = stack.pop() {
        visit(&mut node)?;
        match node {
            Node::Paragraphs(paras) => {
                stack.extend(paras.iter_mut().rev().map(Node::Paragraph));
            }
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

#[derive(Clone, Copy)]
pub(crate) enum RefNode<'a> {
    Paragraphs(&'a [Paragraph]),
    Paragraph(&'a Paragraph),
    Control(&'a Control),
    Shape(&'a ShapeObject),
}

pub(crate) fn walk_ref<S: crate::storage::Storage>(
    paras: &[Paragraph],
    storage: &S,
    mut visit: impl FnMut(RefNode<'_>) -> Result<(), S::Error>,
) -> Result<(), S::Error> {
    let mut stack = storage.collect([RefNode::Paragraphs(paras)])?;
    while let Some(node) = stack.pop() {
        visit(node)?;
        match node {
            RefNode::Paragraphs(paras) => {
                for para in paras.iter().rev() {
                    storage.push(&mut stack, RefNode::Paragraph(para))?;
                }
            }
            RefNode::Paragraph(para) => {
                for control in para.controls.iter().rev() {
                    storage.push(&mut stack, RefNode::Control(control))?;
                }
            }
            RefNode::Control(control) => match control {
                Control::Table(table) => {
                    caption_ref(&mut stack, table.caption.as_ref(), storage)?;
                    for cell in table.cells.iter().rev() {
                        storage.push(&mut stack, RefNode::Paragraphs(&cell.paragraphs))?;
                    }
                }
                Control::Shape(shape) => storage.push(&mut stack, RefNode::Shape(shape))?,
                Control::Picture(pic) => caption_ref(&mut stack, pic.caption.as_ref(), storage)?,
                Control::Header(h) => {
                    storage.push(&mut stack, RefNode::Paragraphs(&h.paragraphs))?
                }
                Control::Footer(h) => {
                    storage.push(&mut stack, RefNode::Paragraphs(&h.paragraphs))?
                }
                Control::Footnote(n) => {
                    storage.push(&mut stack, RefNode::Paragraphs(&n.paragraphs))?
                }
                Control::Endnote(n) => {
                    storage.push(&mut stack, RefNode::Paragraphs(&n.paragraphs))?
                }
                Control::HiddenComment(c) => {
                    storage.push(&mut stack, RefNode::Paragraphs(&c.paragraphs))?
                }
                Control::Field(f) => {
                    storage.push(&mut stack, RefNode::Paragraphs(&f.memo_paragraphs))?
                }
                Control::SectionDef(s) => {
                    for master in s.master_pages.iter().rev() {
                        storage.push(&mut stack, RefNode::Paragraphs(&master.paragraphs))?;
                    }
                }
                _ => {}
            },
            RefNode::Shape(shape) => match shape {
                ShapeObject::Group(group) => {
                    caption_ref(&mut stack, group.caption.as_ref(), storage)?;
                    for child in group.children.iter().rev() {
                        storage.push(&mut stack, RefNode::Shape(child))?;
                    }
                }
                ShapeObject::Picture(pic) => {
                    caption_ref(&mut stack, pic.caption.as_ref(), storage)?
                }
                ShapeObject::Chart(chart) => {
                    caption_ref(&mut stack, chart.caption.as_ref(), storage)?;
                    drawing_ref(&mut stack, &chart.drawing, storage)?;
                }
                ShapeObject::Ole(ole) => {
                    caption_ref(&mut stack, ole.caption.as_ref(), storage)?;
                    drawing_ref(&mut stack, &ole.drawing, storage)?;
                }
                _ => {
                    if let Some(drawing) = shape.drawing() {
                        drawing_ref(&mut stack, drawing, storage)?;
                    }
                }
            },
        }
    }
    Ok(())
}

fn caption_ref<'a, S: crate::storage::Storage>(
    stack: &mut Vec<RefNode<'a>>,
    caption: Option<&'a Caption>,
    storage: &S,
) -> Result<(), S::Error> {
    if let Some(caption) = caption {
        storage.push(stack, RefNode::Paragraphs(&caption.paragraphs))?;
    }
    Ok(())
}
fn drawing_ref<'a, S: crate::storage::Storage>(
    stack: &mut Vec<RefNode<'a>>,
    drawing: &'a crate::model::shape::DrawingObjAttr,
    storage: &S,
) -> Result<(), S::Error> {
    caption_ref(stack, drawing.caption.as_ref(), storage)?;
    if let Some(textbox) = &drawing.text_box {
        storage.push(stack, RefNode::Paragraphs(&textbox.paragraphs))?;
    }
    Ok(())
}
