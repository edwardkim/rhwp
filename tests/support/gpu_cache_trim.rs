// Execute the actual std-only cache removal policy without enabling a GPU SDK.
#[path = "../../src/native_host/gpu_tiles/cache_trim.rs"]
mod cache_trim;
pub(crate) use cache_trim::{remove_oldest_idle, remove_unique};
