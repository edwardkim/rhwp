//! Native Skia layer renderer.
//!
//! This module is available only with the `native-skia` feature.

mod anisotropic_image;
mod cache;
mod diagnostics;
mod equation_conv;
mod equation_glyph;
mod fill;
mod fill_shader;
mod font_catalog;
mod font_lookup;
mod glyph_replay;
mod hft_outline;
mod image_cache;
mod image_conv;
mod image_effect;
mod line_replay;
mod metafile_print;
mod prepared_page;
mod region_plan;
mod renderer;
mod replay_policy;
mod surface;
mod svg_bitmap_mask;
mod synthetic_typeface;
mod text_replay;
mod text_shaping;

pub use cache::{NativeSkiaCacheStats, NativeSkiaCacheUsage};
pub use prepared_page::PreparedSkiaPage;
pub use region_plan::SkiaRegionPlan;
pub use renderer::{
    native_skia_glyph_run_replay_proof, NativeGlyphRunReplayProof, NativeGlyphRunReplayProofReason,
    SkiaLayerRenderer,
};

pub use surface::SkiaSurfaceRenderOptions;
