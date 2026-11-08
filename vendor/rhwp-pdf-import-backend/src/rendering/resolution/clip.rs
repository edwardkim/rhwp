//! Clip-resolution stage.
//!
//! The active clip stack is owned by the operator walker; the resolver simply
//! projects "the most recent composed mask, if any" into a [`ClipPlan`]. The
//! walker is responsible for *composing* the stack (intersecting nested
//! masks) — that's a stateful operation tied to the operator dispatch order,
//! not to the resolution of a single intent. The resolver's only job is to
//! take a borrow of the composed result and hand it to the backend wrapped in
//! `Arc` so the same mask can serve both sides of a fill-then-stroke pair.

use std::sync::Arc;

use super::resolved::ClipPlan;

pub(crate) struct ClipResolver;

impl ClipResolver {
    pub(crate) const fn new() -> Self {
        Self
    }

    /// Wrap a composed clip mask reference into a [`ClipPlan`].
    ///
    /// Passing `None` means "no clip in effect"; passing `Some(mask)` produces
    /// a `Mask` plan that wraps the input in `Arc`. The wrap is intentional —
    /// the operator walker may want to call `resolve_with_mask` twice for a
    /// `B`/`b` fill-then-stroke pair, and the `Arc::clone` between calls is
    /// cheaper than a full `Mask` clone.
    pub(crate) fn resolve_with_mask(&self, mask: Option<Arc<tiny_skia::Mask>>) -> ClipPlan {
        match mask {
            None => ClipPlan::None,
            Some(m) => ClipPlan::Mask(m),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_mask_yields_none() {
        let plan = ClipResolver::new().resolve_with_mask(None);
        match plan {
            ClipPlan::None => {},
            _ => panic!("expected None"),
        }
    }

    #[test]
    fn mask_round_trip_preserves_arc_identity() {
        // The pipeline's contract is that the fill and stroke sides of a
        // single B/b operator share one composed mask. Cloning the Arc
        // (not the Mask) is how we achieve that without rasterising twice.
        let mask = Arc::new(tiny_skia::Mask::new(8, 8).expect("8x8 mask"));
        let plan_a = ClipResolver::new().resolve_with_mask(Some(mask.clone()));
        let plan_b = ClipResolver::new().resolve_with_mask(Some(mask.clone()));
        let (a, b) = match (&plan_a, &plan_b) {
            (ClipPlan::Mask(a), ClipPlan::Mask(b)) => (a, b),
            _ => panic!("both plans should be Mask"),
        };
        assert!(Arc::ptr_eq(a, b), "the resolver must wrap, not clone, the underlying Mask");
    }

    #[test]
    fn mask_dimensions_round_trip() {
        // Sanity check that the resolver doesn't accidentally re-allocate
        // or resize the mask; we hand the same data to the backend.
        let mask = Arc::new(tiny_skia::Mask::new(13, 17).expect("13x17 mask"));
        let plan = ClipResolver::new().resolve_with_mask(Some(mask));
        match plan {
            ClipPlan::Mask(m) => {
                assert_eq!(m.width(), 13);
                assert_eq!(m.height(), 17);
            },
            _ => panic!("expected Mask"),
        }
    }
}
