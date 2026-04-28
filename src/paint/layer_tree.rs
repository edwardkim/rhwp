use crate::paint::paint_op::PaintOp;
use crate::paint::profile::RenderProfile;
use crate::paint::resources::ResourceArena;
use crate::renderer::render_tree::{
    BoundingBox, GroupNode, NodeId, TableCellNode, TableNode, TextLineNode,
};

/// JSON schema version for `PageLayerTree` exports.
///
/// Increment this when the exported node/op shape changes incompatibly.
pub const PAGE_LAYER_TREE_SCHEMA_VERSION: u32 = 1;
pub const PAGE_LAYER_TREE_RESOURCE_TABLE_VERSION: u32 = 1;
pub const PAGE_LAYER_TREE_UNIT: &str = "px";
pub const PAGE_LAYER_TREE_COORDINATE_SYSTEM: &str = "page-top-left";

/// 한 페이지의 visual layer tree.
#[derive(Debug, Clone)]
pub struct PageLayerTree {
    pub page_width: f64,
    pub page_height: f64,
    pub profile: RenderProfile,
    pub root: LayerNode,
    pub resources: ResourceArena,
}

impl PageLayerTree {
    pub fn new(page_width: f64, page_height: f64, root: LayerNode) -> Self {
        Self {
            page_width,
            page_height,
            profile: RenderProfile::Screen,
            root,
            resources: ResourceArena,
        }
    }

    pub fn with_profile(
        page_width: f64,
        page_height: f64,
        root: LayerNode,
        profile: RenderProfile,
    ) -> Self {
        Self {
            page_width,
            page_height,
            profile,
            root,
            resources: ResourceArena,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CacheHint {
    #[default]
    None,
    StaticSubtree,
    PreferRaster,
    PreferVectorRecording,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipKind {
    Body,
    TableCell,
    Generic,
}

#[derive(Debug, Clone)]
pub struct LayerNode {
    pub bounds: BoundingBox,
    pub source_node_id: Option<NodeId>,
    pub kind: LayerNodeKind,
}

impl LayerNode {
    pub fn group(
        bounds: BoundingBox,
        source_node_id: Option<NodeId>,
        children: Vec<LayerNode>,
        cache_hint: CacheHint,
        group_kind: GroupKind,
    ) -> Self {
        Self {
            bounds,
            source_node_id,
            kind: LayerNodeKind::Group {
                children,
                cache_hint,
                group_kind,
            },
        }
    }

    pub fn clip_rect(
        bounds: BoundingBox,
        source_node_id: Option<NodeId>,
        clip: BoundingBox,
        child: LayerNode,
        clip_kind: ClipKind,
    ) -> Self {
        Self {
            bounds,
            source_node_id,
            kind: LayerNodeKind::ClipRect {
                clip,
                child: Box::new(child),
                clip_kind,
            },
        }
    }

    pub fn leaf(bounds: BoundingBox, source_node_id: Option<NodeId>, ops: Vec<PaintOp>) -> Self {
        Self {
            bounds,
            source_node_id,
            kind: LayerNodeKind::Leaf { ops },
        }
    }
}

#[derive(Debug, Clone)]
pub enum LayerNodeKind {
    Group {
        children: Vec<LayerNode>,
        cache_hint: CacheHint,
        group_kind: GroupKind,
    },
    ClipRect {
        clip: BoundingBox,
        child: Box<LayerNode>,
        clip_kind: ClipKind,
    },
    Leaf {
        ops: Vec<PaintOp>,
    },
}

#[derive(Debug, Clone)]
pub enum GroupKind {
    Generic,
    MasterPage,
    Header,
    Footer,
    Body,
    Column(u16),
    FootnoteArea,
    TextLine(TextLineNode),
    Table(TableNode),
    TableCell(TableCellNode),
    TextBox,
    Group(GroupNode),
}
