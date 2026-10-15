//! Request adapter for the Adobe-aware CMYK JPEG path.
use crate::import_limits::ImportBudget;

pub(super) struct Request<'a>(pub(super) &'a ImportBudget);

impl jpeg_decoder_limited::allocation::AllocationBudget for Request<'_> {
    fn reserve(&self, bytes: usize, _kind: &'static str) -> bool {
        self.0.charge_codec_allocation(bytes).is_ok()
    }
    fn reject(&self) {
        self.0.reject("CMYK JPEG allocation");
    }
}
