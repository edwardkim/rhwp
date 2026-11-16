// Namespace adapter keeps this production path safe in a shared integration suite.
#[path = "../../src/native_host/gpu_tiles/frame_resources.rs"]
mod implementation;
pub(crate) use implementation::FrameResources;
