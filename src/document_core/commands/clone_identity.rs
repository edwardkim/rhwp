//! Editing adapter to the model's shared identity reservations.
mod remap;
pub(super) use crate::model::identity::next_instance_id;
pub(super) use remap::reidentify_clipboard;
pub(super) use remap::reidentify_with_allocator;
