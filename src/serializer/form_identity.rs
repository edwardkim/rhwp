//! Complete missing HWP Form identities on a detached section only.
use std::borrow::Cow;

use super::SerializeError;
use crate::model::{
    control::Control,
    document::{Document, Section},
    identity::{
        self,
        walk::{walk, Node},
        Allocator,
    },
};

pub(super) fn prepare_section<'a>(
    section: &'a Section,
    document: &Document,
    allocator: &mut Option<Allocator>,
) -> Result<Cow<'a, Section>, SerializeError> {
    if !identity::has_unassigned_forms(section) {
        return Ok(Cow::Borrowed(section));
    }
    // All sections (including raw-reused and later sections) reserve their IDs.
    // A single allocator is retained across the entire output document.
    let allocator = allocator.get_or_insert_with(|| Allocator {
        used: identity::used_instance_ids(document),
        next: 1,
    });
    let mut staged = section.clone();
    let mut order = 0u32;
    let mut assign = |node: &mut Node<'_>| {
        if let Node::Control(Control::Form(form)) = node {
            if form.common.attr == 0 && form.common.instance_id == 0 {
                // Retain #852's legacy output when it does not collide. This
                // preference is not a uniqueness proof or a field-reference rule.
                let preferred = 0x7dcd_59d6u32.wrapping_add(order);
                form.common.instance_id = if preferred != 0 && allocator.used.insert(preferred) {
                    preferred
                } else {
                    allocator.id()?
                };
            }
            order = order.wrapping_add(1);
        }
        Ok(())
    };
    walk(&mut staged.paragraphs, &mut assign)
        .map_err(|e| SerializeError::UnsupportedInput(e.to_string()))?;
    for master in &mut staged.section_def.master_pages {
        walk(&mut master.paragraphs, &mut assign)
            .map_err(|e| SerializeError::UnsupportedInput(e.to_string()))?;
    }
    staged.raw_stream = None;
    Ok(Cow::Owned(staged))
}
