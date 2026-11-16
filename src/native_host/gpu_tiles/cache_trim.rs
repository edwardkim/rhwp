//! Owner-thread cache removal policy. Strong pins always win over a trim target.
use std::rc::Rc;

pub(crate) fn remove_oldest_idle<T>(entries: &mut Vec<Rc<T>>) -> Option<Rc<T>> {
    let index = entries
        .iter()
        .position(|entry| Rc::strong_count(entry) == 1)?;
    Some(entries.remove(index))
}

// Used by the Metal/D3D scratch owners; GL retains its existing plain Vec.
#[allow(dead_code)]
pub(crate) fn remove_unique<T>(entry: &mut Option<Rc<T>>) -> bool {
    if entry
        .as_ref()
        .is_some_and(|entry| Rc::strong_count(entry) == 1)
    {
        entry.take();
        true
    } else {
        false
    }
}
