// Use the exact private Native FFI policy without widening the public ABI.
#[allow(dead_code)]
#[path = "../../bindings/Native/src/gpu_metal/admission.rs"]
mod implementation;
pub(crate) use implementation::{Admission, Credit, Stats, MAX_PROCESS_PRESENTERS};
