//! Equation faces requested by body and nested content for renderer font supply.

use crate::model::{control::Control, paragraph::Paragraph, shape::ShapeObject};
use std::collections::BTreeSet;

pub(crate) fn collect(paragraphs: &[Paragraph], fonts: &mut BTreeSet<String>) {
    for paragraph in paragraphs {
        for control in &paragraph.controls {
            match control {
                Control::Equation(equation) => {
                    if !equation.font_name.trim().is_empty() {
                        fonts.insert(equation.font_name.clone());
                    }
                }
                Control::Table(table) => {
                    for cell in &table.cells {
                        collect(&cell.paragraphs, fonts);
                    }
                    if let Some(caption) = &table.caption {
                        collect(&caption.paragraphs, fonts);
                    }
                }
                Control::Footnote(note) => collect(&note.paragraphs, fonts),
                Control::Endnote(note) => collect(&note.paragraphs, fonts),
                Control::Header(header) => collect(&header.paragraphs, fonts),
                Control::Footer(footer) => collect(&footer.paragraphs, fonts),
                Control::HiddenComment(comment) => collect(&comment.paragraphs, fonts),
                Control::Field(field) => collect(&field.memo_paragraphs, fonts),
                Control::SectionDef(section) => {
                    for page in &section.master_pages {
                        collect(&page.paragraphs, fonts);
                    }
                }
                Control::Shape(object) => shape(object, fonts),
                Control::Picture(picture) => {
                    if let Some(caption) = &picture.caption {
                        collect(&caption.paragraphs, fonts);
                    }
                }
                _ => {}
            }
        }
    }
}

fn shape(object: &ShapeObject, fonts: &mut BTreeSet<String>) {
    if let Some(drawing) = object.drawing() {
        if let Some(text_box) = &drawing.text_box {
            collect(&text_box.paragraphs, fonts);
        }
        if let Some(caption) = &drawing.caption {
            collect(&caption.paragraphs, fonts);
        }
    }
    match object {
        ShapeObject::Group(group) => {
            for child in &group.children {
                shape(child, fonts);
            }
            if let Some(caption) = &group.caption {
                collect(&caption.paragraphs, fonts);
            }
        }
        ShapeObject::Picture(picture) => {
            if let Some(caption) = &picture.caption {
                collect(&caption.paragraphs, fonts);
            }
        }
        ShapeObject::Chart(chart) => {
            if let Some(caption) = &chart.caption {
                collect(&caption.paragraphs, fonts);
            }
        }
        ShapeObject::Ole(ole) => {
            if let Some(caption) = &ole.caption {
                collect(&caption.paragraphs, fonts);
            }
        }
        _ => {}
    }
}
