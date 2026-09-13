//! Assign identities on a detached clipboard tree, then reconnect owned refs.
use std::collections::BTreeMap;

use crate::error::HwpError;
use crate::model::identity::{
    paragraph_ids, subject_alias, used_instance_ids,
    walk::{walk, Node},
    Allocator,
};
use crate::model::{
    control::Control,
    document::Document,
    image::Picture,
    paragraph::Paragraph,
    raw_provenance::record_digest,
    shape::{common_obj_offsets, CommonObjAttr, ShapeObject},
};

#[derive(Default)]
struct ReferenceMap(BTreeMap<u32, Option<u32>>);

impl ReferenceMap {
    fn add(&mut self, old: u32, new: u32) {
        self.0
            .entry(old)
            .and_modify(|value| {
                if *value != Some(new) {
                    *value = None;
                }
            })
            .or_insert(Some(new));
    }

    fn rewrite(&self, id: &mut u32, kind: &str) -> Result<(), HwpError> {
        match self.0.get(id) {
            Some(Some(new)) => *id = *new,
            Some(None) => {
                return Err(HwpError::RenderError(format!(
                    "ambiguous {kind} identity {id} in copied control tree"
                )))
            }
            // An external reference keeps pointing at the original document.
            None => {}
        }
        Ok(())
    }
}

/// Only mutates the caller's staged copy. Names, styles, resources, content and
/// external refs are retained. Opaque extension payloads are not interpreted.
pub(crate) fn reidentify_clipboard(
    doc: &Document,
    paras: &mut [Paragraph],
) -> Result<(), HwpError> {
    let mut used = used_instance_ids(doc);
    used.extend(paragraph_ids(paras)); // clipboard can outlive cut/source deletion
    let mut allocator = Allocator { used, next: 1 };
    reidentify_with_allocator(paras, &mut allocator)
}

/// A request owns the allocator; each copy owns its reference maps.
pub(crate) fn reidentify_with_allocator(
    paras: &mut [Paragraph],
    allocator: &mut Allocator,
) -> Result<(), HwpError> {
    let mut subjects = ReferenceMap::default();
    let mut fields = ReferenceMap::default();

    // All destinations are assigned before any forward/backward ref is resolved.
    walk(paras, |node| {
        match node {
            Node::Paragraph(para) => {
                if let Some(raw) = para.raw_header_extra.get_mut(6..10) {
                    raw.copy_from_slice(&allocator.id()?.to_le_bytes());
                }
            }
            Node::Control(control) => match control {
                Control::Table(table) => remap_common_raw(
                    &mut table.common,
                    &mut table.raw_ctrl_data,
                    &mut table.raw_ctrl_seal,
                    allocator,
                )?,
                Control::Picture(pic) => remap_picture(pic, allocator, &mut subjects)?,
                Control::Equation(eq) => remap_common_raw(
                    &mut eq.common,
                    &mut eq.raw_ctrl_data,
                    &mut eq.raw_ctrl_seal,
                    allocator,
                )?,
                Control::Form(form) => form.common.instance_id = allocator.common()?,
                Control::Footnote(note) => note.instance_id = allocator.id()?,
                Control::Endnote(note) => note.instance_id = allocator.id()?,
                Control::Field(field) => {
                    let old = field.field_id;
                    field.field_id = allocator.id()?;
                    fields.add(old, field.field_id);
                    // field.instance_id / end_field_id are shared HWPX fieldid
                    // metadata, not beginIDRef's unique target. Preserve them.
                }
                _ => {}
            },
            Node::Shape(shape) => {
                if let ShapeObject::Picture(pic) = shape {
                    remap_picture(pic, allocator, &mut subjects)?;
                } else {
                    let old_common = shape.common().instance_id;
                    let old_drawing = shape.drawing().map(|d| d.inst_id);
                    let new_common = allocator.common()?;
                    shape.common_mut().instance_id = new_common;
                    let target = if let Some(drawing) = shape.drawing_mut() {
                        drawing.inst_id = allocator.id()?;
                        drawing.inst_id
                    } else {
                        new_common
                    };
                    if let Some(old) = old_drawing.filter(|id| *id != 0) {
                        subjects.add(old, target);
                    }
                    subject_common(&mut subjects, old_common, target);
                    if let ShapeObject::Ole(ole) = shape {
                        if ole.hwpx_ole_id.is_some() {
                            ole.hwpx_ole_id = Some(allocator.id()?);
                        }
                    }
                }
            }
        }
        Ok(())
    })?;

    walk(paras, |node| {
        match node {
            Node::Paragraph(para) => {
                // Same-paragraph ranges use control_idx, so they need no ID edit.
                for end in &mut para.orphan_field_ends {
                    fields.rewrite(&mut end.begin_id_ref, "field begin")?;
                }
            }
            Node::Shape(ShapeObject::Line(line)) => {
                if let Some(connector) = &mut line.connector {
                    if connector.start_subject_id != 0 {
                        subjects.rewrite(&mut connector.start_subject_id, "connector subject")?;
                    }
                    if connector.end_subject_id != 0 {
                        subjects.rewrite(&mut connector.end_subject_id, "connector subject")?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    })
}

fn subject_common(map: &mut ReferenceMap, old: u32, target: u32) {
    if old != 0 {
        map.add(old, target);
        map.add(subject_alias(old), target);
    }
}

fn remap_picture(
    pic: &mut Picture,
    allocator: &mut Allocator,
    subjects: &mut ReferenceMap,
) -> Result<(), HwpError> {
    let old = pic.common.instance_id;
    pic.common.instance_id = allocator.common()?;
    subject_common(subjects, old, pic.common.instance_id);
    pic.instance_id = allocator.id()?;
    if !pic.raw_picture_extra.is_empty() {
        let raw = pic.raw_picture_extra.get_mut(1..5).ok_or_else(|| {
            HwpError::RenderError(
                "truncated picture identity payload in copied control tree".into(),
            )
        })?;
        raw.copy_from_slice(&pic.instance_id.to_le_bytes());
    }
    Ok(())
}

fn remap_common_raw(
    common: &mut CommonObjAttr,
    raw: &mut [u8],
    seal: &mut Option<[u8; 32]>,
    allocator: &mut Allocator,
) -> Result<(), HwpError> {
    let sealed_and_valid = seal.is_some_and(|digest| digest == record_digest(common));
    common.instance_id = allocator.common()?;
    if !raw.is_empty() {
        let slot = raw
            .get_mut(common_obj_offsets::INSTANCE_ID)
            .ok_or_else(|| {
                HwpError::RenderError(
                    "truncated common identity payload in copied control tree".into(),
                )
            })?;
        slot.copy_from_slice(&common.instance_id.to_le_bytes());
    }
    // A previously stale seal must stay stale. Re-sealing it would revive other
    // obsolete raw properties. An unsealed source keeps its legacy raw policy.
    if sealed_and_valid {
        *seal = Some(record_digest(common));
    }
    Ok(())
}
