//! Compatibility names for the GL presenter; all backends share one tile policy.

pub(super) use super::super::gpu_tiles::{
    CachedTile, ManagedTarget, TargetAllocation, TargetBudget, TileCache, TileConfig,
    TilePlacement, TilePlan, TileTarget,
};
pub use super::super::gpu_tiles::{
    NativeGpuTargetStats as NativeGlTargetStats, NativeGpuTileStats as NativeGlTileStats,
};
