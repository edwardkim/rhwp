//! 시각 레이어 IR 모듈
//!
//! semantic render tree를 backend-friendly layer tree로 변환한다.

pub mod builder;
mod json;
pub mod layer_tree;
pub mod paint_op;
pub mod profile;
pub mod resources;

pub use builder::LayerBuilder;
pub use layer_tree::{
    CacheHint, ClipKind, GroupKind, LayerNode, LayerNodeKind, LayerOutputOptions, PageLayerTree,
    PAGE_LAYER_TREE_COORDINATE_SYSTEM, PAGE_LAYER_TREE_RESOURCE_TABLE_VERSION,
    PAGE_LAYER_TREE_SCHEMA_VERSION, PAGE_LAYER_TREE_UNIT,
};
pub use paint_op::PaintOp;
pub use profile::RenderProfile;
pub use resources::ResourceArena;
