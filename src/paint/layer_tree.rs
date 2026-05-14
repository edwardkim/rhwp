use crate::paint::paint_op::PaintOp;
use crate::paint::profile::RenderProfile;
use crate::paint::resources::ResourceArena;
use crate::renderer::render_tree::{
    BoundingBox, FieldMarkerType, GroupNode, NodeId, TableCellNode, TableNode, TextLineNode,
    TextRunNode,
};

/// 한 페이지의 visual layer tree.
#[derive(Debug, Clone)]
pub struct PageLayerTree {
    pub page_width: f64,
    pub page_height: f64,
    pub profile: RenderProfile,
    pub output_options: LayerOutputOptions,
    pub root: LayerNode,
    pub resources: ResourceArena,
    pub text_sources: TextSourceTable,
}

impl PageLayerTree {
    pub fn new(page_width: f64, page_height: f64, root: LayerNode) -> Self {
        let text_sources = TextSourceTable::from_layer_node(&root);
        Self {
            page_width,
            page_height,
            profile: RenderProfile::Screen,
            output_options: LayerOutputOptions::default(),
            root,
            resources: ResourceArena::default(),
            text_sources,
        }
    }

    pub fn with_profile(
        page_width: f64,
        page_height: f64,
        root: LayerNode,
        profile: RenderProfile,
    ) -> Self {
        let text_sources = TextSourceTable::from_layer_node(&root);
        Self {
            page_width,
            page_height,
            profile,
            output_options: LayerOutputOptions::default(),
            root,
            resources: ResourceArena::default(),
            text_sources,
        }
    }

    pub fn with_output_options(mut self, output_options: LayerOutputOptions) -> Self {
        self.output_options = output_options;
        self
    }

    pub fn with_text_sources(mut self, text_sources: TextSourceTable) -> Self {
        self.text_sources = text_sources;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSourceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSourceRange {
    pub start: u32,
    pub end: u32,
}

impl TextSourceRange {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextSourceSpan {
    pub id: TextSourceId,
    pub utf8_range: TextSourceRange,
    pub utf16_range: TextSourceRange,
    pub stable_source_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSourceEntry {
    pub id: TextSourceId,
    pub stable_source_key: Option<String>,
    pub text: String,
    pub utf8_range: TextSourceRange,
    pub utf16_range: TextSourceRange,
    pub annotations: Vec<TextSourceAnnotation>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextSourceAnnotation {
    FieldMarker {
        marker: FieldMarkerType,
        range_utf8: TextSourceRange,
        range_utf16: TextSourceRange,
    },
    ParagraphEnd {
        offset_utf8: u32,
        offset_utf16: u32,
    },
    LineBreakEnd {
        offset_utf8: u32,
        offset_utf16: u32,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextSourceTable {
    pub entries: Vec<TextSourceEntry>,
}

impl TextSourceTable {
    pub fn from_layer_node(root: &LayerNode) -> Self {
        let mut table = Self::default();
        table.collect_from_node(root);
        table
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn collect_from_node(&mut self, node: &LayerNode) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    self.collect_from_node(child);
                }
            }
            LayerNodeKind::ClipRect { child, .. } => {
                self.collect_from_node(child);
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::TextRun { run, .. } = op {
                        let id = TextSourceId(self.entries.len() as u32);
                        let utf8_range = TextSourceRange::new(0, run.text.len() as u32);
                        let utf16_range =
                            TextSourceRange::new(0, run.text.encode_utf16().count() as u32);
                        self.entries.push(TextSourceEntry {
                            id,
                            stable_source_key: stable_text_source_key(run),
                            text: run.text.clone(),
                            utf8_range,
                            utf16_range,
                            annotations: text_source_annotations(
                                run,
                                utf8_range.end,
                                utf16_range.end,
                            ),
                        });
                    }
                }
            }
        }
    }
}

fn stable_text_source_key(run: &TextRunNode) -> Option<String> {
    let section = run.section_index?;
    let para = run.para_index?;
    let char_start = run.char_start.unwrap_or(0);
    let mut key = format!("section:{section}/para:{para}/char:{char_start}");
    if let Some(cell) = &run.cell_context {
        let path = cell
            .path
            .iter()
            .map(|entry| {
                format!(
                    "{}:{}:{}:{}",
                    entry.control_index,
                    entry.cell_index,
                    entry.cell_para_index,
                    entry.text_direction
                )
            })
            .collect::<Vec<_>>()
            .join(".");
        key.push_str("/cell:");
        key.push_str(&cell.parent_para_index.to_string());
        key.push(':');
        key.push_str(&path);
    }
    Some(key)
}

fn text_source_annotations(
    run: &TextRunNode,
    utf8_end: u32,
    utf16_end: u32,
) -> Vec<TextSourceAnnotation> {
    let mut annotations = Vec::new();
    if run.field_marker != FieldMarkerType::None {
        annotations.push(TextSourceAnnotation::FieldMarker {
            marker: run.field_marker,
            range_utf8: TextSourceRange::new(0, utf8_end),
            range_utf16: TextSourceRange::new(0, utf16_end),
        });
    }
    if run.is_para_end {
        annotations.push(TextSourceAnnotation::ParagraphEnd {
            offset_utf8: utf8_end,
            offset_utf16: utf16_end,
        });
    }
    if run.is_line_break_end {
        annotations.push(TextSourceAnnotation::LineBreakEnd {
            offset_utf8: utf8_end,
            offset_utf16: utf16_end,
        });
    }
    annotations
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerOutputOptions {
    pub show_paragraph_marks: bool,
    pub show_control_codes: bool,
    pub show_transparent_borders: bool,
    pub clip_enabled: bool,
    pub debug_overlay: bool,
}

impl Default for LayerOutputOptions {
    fn default() -> Self {
        Self {
            show_paragraph_marks: false,
            show_control_codes: false,
            show_transparent_borders: false,
            clip_enabled: true,
            debug_overlay: false,
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
