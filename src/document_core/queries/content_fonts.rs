//! Faces requested by body and nested content for renderer font supply:
//! equation faces and the public faces of synthesized Hancom PUA glyphs (#8201).

use crate::model::{control::Control, paragraph::Paragraph, shape::ShapeObject};
use std::collections::BTreeSet;

pub(crate) fn collect(
    paragraphs: &[Paragraph],
    fonts: &mut BTreeSet<String>,
    legacy_symbols: bool,
) {
    for paragraph in paragraphs {
        for key in crate::model::provenance::body_chars(&paragraph.text, legacy_symbols) {
            if let Some(glyph) = crate::renderer::enclosed_pua(key) {
                fonts.extend(
                    crate::renderer::enclosed_pua_geometry::public_faces(glyph)
                        .iter()
                        .cloned(),
                );
            }
        }
        for control in &paragraph.controls {
            match control {
                Control::Equation(equation) => {
                    if !equation.font_name.trim().is_empty() {
                        fonts.insert(equation.font_name.clone());
                    }
                }
                Control::Table(table) => {
                    for cell in &table.cells {
                        collect(&cell.paragraphs, fonts, legacy_symbols);
                    }
                    if let Some(caption) = &table.caption {
                        collect(&caption.paragraphs, fonts, legacy_symbols);
                    }
                }
                Control::Footnote(note) => collect(&note.paragraphs, fonts, legacy_symbols),
                Control::Endnote(note) => collect(&note.paragraphs, fonts, legacy_symbols),
                Control::Header(header) => collect(&header.paragraphs, fonts, legacy_symbols),
                Control::Footer(footer) => collect(&footer.paragraphs, fonts, legacy_symbols),
                Control::HiddenComment(comment) => {
                    collect(&comment.paragraphs, fonts, legacy_symbols)
                }
                Control::Field(field) => collect(&field.memo_paragraphs, fonts, legacy_symbols),
                Control::SectionDef(section) => {
                    for page in &section.master_pages {
                        collect(&page.paragraphs, fonts, legacy_symbols);
                    }
                }
                Control::Shape(object) => shape(object, fonts, legacy_symbols),
                Control::Picture(picture) => {
                    if let Some(caption) = &picture.caption {
                        collect(&caption.paragraphs, fonts, legacy_symbols);
                    }
                }
                _ => {}
            }
        }
    }
}

fn shape(object: &ShapeObject, fonts: &mut BTreeSet<String>, legacy_symbols: bool) {
    if let Some(drawing) = object.drawing() {
        if let Some(text_box) = &drawing.text_box {
            collect(&text_box.paragraphs, fonts, legacy_symbols);
        }
        if let Some(caption) = &drawing.caption {
            collect(&caption.paragraphs, fonts, legacy_symbols);
        }
    }
    match object {
        ShapeObject::Group(group) => {
            for child in &group.children {
                shape(child, fonts, legacy_symbols);
            }
            if let Some(caption) = &group.caption {
                collect(&caption.paragraphs, fonts, legacy_symbols);
            }
        }
        ShapeObject::Picture(picture) => {
            if let Some(caption) = &picture.caption {
                collect(&caption.paragraphs, fonts, legacy_symbols);
            }
        }
        ShapeObject::Chart(chart) => {
            if let Some(caption) = &chart.caption {
                collect(&caption.paragraphs, fonts, legacy_symbols);
            }
        }
        ShapeObject::Ole(ole) => {
            if let Some(caption) = &ole.caption {
                collect(&caption.paragraphs, fonts, legacy_symbols);
            }
        }
        _ => {}
    }
}
