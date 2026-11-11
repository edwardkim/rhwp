//! Parent-module adapter for the real private GPU policy in integration tests.
pub use rhwp::native_host::{presentation, render_snapshot, NativeHostError};
#[path = "../../src/native_host/gpu_tiles.rs"]
pub(crate) mod gpu_tiles;
