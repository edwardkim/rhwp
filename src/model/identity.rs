//! Shared identity reservations for editing and serialization (#3587).
//! Existing objects (including duplicate identities) stay untouched. Walk the owned IR;
//! dimensions, clock time and wrapping hashes cannot establish uniqueness.
use std::collections::BTreeSet;

use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{common_obj_offsets, Caption, ShapeObject};

pub(crate) mod walk;

enum Node<'a> {
    Paragraphs(&'a [Paragraph]),
    Control(&'a Control),
    Shape(&'a ShapeObject),
}

fn push_caption<'a>(pending: &mut Vec<Node<'a>>, caption: Option<&'a Caption>) {
    if let Some(caption) = caption {
        pending.push(Node::Paragraphs(&caption.paragraphs));
    }
}

fn reserve_raw(used: &mut BTreeSet<u32>, slot: Option<&[u8]>) {
    if let Some([a, b, c, d]) = slot {
        used.insert(u32::from_le_bytes([*a, *b, *c, *d]));
    }
}

/// Allocate a nonzero common object ID from the current document snapshot.
///
/// Reserve drawing and field IDs as well, without conflating their reference
/// namespaces or rewriting them. Reserve both table IR and retained HWP raw IDs
/// when they disagree: either may be observed by the corresponding serializer.
/// Unknown opaque payloads are not decoded here; this does not certify arbitrary
/// block cloning. No persisted counter can become stale after a direct IR edit.
pub(crate) fn next_instance_id(document: &Document) -> Result<u32, HwpError> {
    Allocator {
        used: used_instance_ids(document),
        next: 1,
    }
    .id()
}

pub(crate) fn used_instance_ids(document: &Document) -> BTreeSet<u32> {
    let mut pending = Vec::new();
    for section in &document.sections {
        pending.push(Node::Paragraphs(&section.paragraphs));
        for master in &section.section_def.master_pages {
            pending.push(Node::Paragraphs(&master.paragraphs));
        }
    }
    collect_ids(pending, |_| {})
}

pub(crate) fn paragraph_ids(paragraphs: &[Paragraph]) -> BTreeSet<u32> {
    collect_ids(vec![Node::Paragraphs(paragraphs)], |_| {})
}

pub(crate) fn has_unassigned_forms(section: &crate::model::document::Section) -> bool {
    let mut pending = vec![Node::Paragraphs(&section.paragraphs)];
    for master in &section.section_def.master_pages {
        pending.push(Node::Paragraphs(&master.paragraphs));
    }
    let mut found = false;
    collect_ids(pending, |form| {
        found |= form.common.attr == 0 && form.common.instance_id == 0;
    });
    found
}

fn collect_ids(
    mut pending: Vec<Node<'_>>,
    mut on_form: impl FnMut(&crate::model::control::FormObject),
) -> BTreeSet<u32> {
    let mut used = BTreeSet::new();
    // Explicit work stack avoids recursive calls on nested tables/groups.
    while let Some(node) = pending.pop() {
        match node {
            Node::Paragraphs(paragraphs) => {
                for paragraph in paragraphs {
                    if let Some(raw) = paragraph.raw_header_extra.get(6..10) {
                        used.insert(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]));
                    }
                    used.extend(paragraph.field_ranges.iter().map(|r| r.end_field_id));
                    for end in &paragraph.orphan_field_ends {
                        used.extend([end.begin_id_ref, end.field_id]);
                    }
                    pending.extend(paragraph.controls.iter().map(Node::Control));
                }
            }
            Node::Control(control) => match control {
                Control::Table(table) => {
                    used.insert(table.common.instance_id);
                    if let Some(raw) = table.raw_ctrl_data.get(common_obj_offsets::INSTANCE_ID) {
                        used.insert(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]));
                    }
                    for cell in &table.cells {
                        pending.push(Node::Paragraphs(&cell.paragraphs));
                    }
                    push_caption(&mut pending, table.caption.as_ref());
                }
                Control::Shape(shape) => pending.push(Node::Shape(shape)),
                Control::Picture(picture) => {
                    used.insert(picture.common.instance_id);
                    used.insert(picture.instance_id);
                    reserve_raw(&mut used, picture.raw_picture_extra.get(1..5));
                    used.insert(subject_alias(picture.common.instance_id));
                    push_caption(&mut pending, picture.caption.as_ref());
                }
                Control::Equation(equation) => {
                    used.insert(equation.common.instance_id);
                    reserve_raw(
                        &mut used,
                        equation.raw_ctrl_data.get(common_obj_offsets::INSTANCE_ID),
                    );
                }
                Control::Form(form) => {
                    used.insert(form.common.instance_id);
                    on_form(form);
                }
                Control::Header(header) => pending.push(Node::Paragraphs(&header.paragraphs)),
                Control::Footer(footer) => pending.push(Node::Paragraphs(&footer.paragraphs)),
                Control::Footnote(note) => {
                    used.insert(note.instance_id);
                    pending.push(Node::Paragraphs(&note.paragraphs));
                }
                Control::Endnote(note) => {
                    used.insert(note.instance_id);
                    pending.push(Node::Paragraphs(&note.paragraphs));
                }
                Control::HiddenComment(comment) => {
                    pending.push(Node::Paragraphs(&comment.paragraphs));
                }
                Control::Field(field) => {
                    used.insert(field.field_id);
                    used.extend(field.instance_id);
                    pending.push(Node::Paragraphs(&field.memo_paragraphs));
                }
                Control::SectionDef(section) => {
                    for master in &section.master_pages {
                        pending.push(Node::Paragraphs(&master.paragraphs));
                    }
                }
                _ => {}
            },
            Node::Shape(shape) => {
                used.insert(shape.common().instance_id);
                used.insert(subject_alias(shape.common().instance_id));
                if let Some(drawing) = shape.drawing() {
                    used.insert(drawing.inst_id);
                    if let Some(textbox) = &drawing.text_box {
                        pending.push(Node::Paragraphs(&textbox.paragraphs));
                    }
                    push_caption(&mut pending, drawing.caption.as_ref());
                }
                match shape {
                    ShapeObject::Line(line) => {
                        if let Some(connector) = &line.connector {
                            used.extend([connector.start_subject_id, connector.end_subject_id]);
                        }
                    }
                    ShapeObject::Group(group) => {
                        pending.extend(group.children.iter().map(Node::Shape));
                        push_caption(&mut pending, group.caption.as_ref());
                    }
                    ShapeObject::Picture(picture) => {
                        used.insert(picture.instance_id);
                        reserve_raw(&mut used, picture.raw_picture_extra.get(1..5));
                        push_caption(&mut pending, picture.caption.as_ref());
                    }
                    ShapeObject::Chart(chart) => {
                        push_caption(&mut pending, chart.caption.as_ref());
                    }
                    ShapeObject::Ole(ole) => {
                        used.extend(ole.hwpx_ole_id);
                        push_caption(&mut pending, ole.caption.as_ref());
                    }
                    _ => {}
                }
            }
        }
    }
    used
}

// Same legacy subject alias accepted by update_connected_lines_native.
pub(crate) fn subject_alias(common_id: u32) -> u32 {
    if common_id == 0 {
        0
    } else {
        (common_id & 0x3fff_ffff) + 1
    }
}

pub(crate) struct Allocator {
    pub(crate) used: BTreeSet<u32>,
    pub(crate) next: u64,
}

impl Allocator {
    pub(crate) fn id(&mut self) -> Result<u32, HwpError> {
        // First unused positive ID. The cursor is shared for an entire clone,
        // avoiding repeated rescans from 1 for every nested object.
        while self.next <= u32::MAX as u64 {
            let candidate = self.next as u32;
            self.next += 1;
            if self.used.insert(candidate) {
                return Ok(candidate);
            }
        }
        Err(HwpError::RenderError(
            "object instance identity space exhausted".into(),
        ))
    }

    pub(crate) fn common(&mut self) -> Result<u32, HwpError> {
        loop {
            let candidate = self.id()?;
            // Do not introduce an alias that can capture another object's ref.
            if self.used.insert(subject_alias(candidate)) {
                return Ok(candidate);
            }
        }
    }
}
