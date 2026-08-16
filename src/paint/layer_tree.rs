use crate::paint::paint_op::PaintOp;
use crate::paint::profile::RenderProfile;
use crate::paint::resources::ResourceArena;
use crate::renderer::render_tree::{
    BoundingBox, FieldMarkerType, GroupNode, NodeId, RenderLayerInfo, TableCellNode, TableNode,
    TextLineNode, TextRunNode,
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

    pub fn with_bin_data_epoch(mut self, bin_data_epoch: u32) -> Self {
        self.resources.set_source_image_epoch(bin_data_epoch);
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
    TextBox,
    Generic,
}

#[derive(Debug, Clone)]
pub struct LayerNode {
    pub bounds: BoundingBox,
    pub source_node_id: Option<NodeId>,
    pub layer: Option<RenderLayerInfo>,
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
            layer: None,
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
            layer: None,
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
            layer: None,
            kind: LayerNodeKind::Leaf { ops },
        }
    }

    pub fn with_layer(mut self, layer: Option<RenderLayerInfo>) -> Self {
        self.layer = layer;
        self
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::render_tree::TextRunNode;
    use crate::renderer::TextStyle;

    fn text_run(text: &str) -> TextRunNode {
        TextRunNode {
            text: text.to_string(),
            style: TextStyle::default(),
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 12.0,
            field_marker: Default::default(),
            display_text: None,
        }
    }

    fn leaf_with(texts: &[&str]) -> LayerNode {
        let ops = texts
            .iter()
            .map(|t| PaintOp::text_run(BoundingBox::default(), text_run(t)))
            .collect();
        LayerNode::leaf(BoundingBox::default(), None, ops)
    }

    /// UTF-8 바이트 길이와 UTF-16 코드유닛 수는 한글·이모지에서 **갈라진다**.
    /// 두 범위가 각자의 단위를 지키는지가 이 표의 핵심 계약이다.
    #[test]
    fn utf8_and_utf16_ranges_diverge_for_korean_and_emoji() {
        let table = TextSourceTable::from_layer_node(&leaf_with(&["한글", "ab", "🙂"]));
        assert_eq!(table.entries.len(), 3);

        // 한글 2자 = UTF-8 6바이트, UTF-16 2유닛
        assert_eq!(table.entries[0].utf8_range, TextSourceRange::new(0, 6));
        assert_eq!(table.entries[0].utf16_range, TextSourceRange::new(0, 2));

        // ASCII 는 둘이 같다
        assert_eq!(table.entries[1].utf8_range, TextSourceRange::new(0, 2));
        assert_eq!(table.entries[1].utf16_range, TextSourceRange::new(0, 2));

        // BMP 밖 이모지 = UTF-8 4바이트, UTF-16 2유닛(서로게이트 쌍)
        assert_eq!(table.entries[2].utf8_range, TextSourceRange::new(0, 4));
        assert_eq!(table.entries[2].utf16_range, TextSourceRange::new(0, 2));
    }

    /// 트리 재귀는 Group·ClipRect 를 뚫고 중첩된 TextRun 까지 닿아야 한다.
    #[test]
    fn collection_descends_through_group_and_clip_rect() {
        let deep = LayerNode::clip_rect(
            BoundingBox::default(),
            None,
            BoundingBox::default(),
            leaf_with(&["안쪽"]),
            ClipKind::TextBox,
        );
        let root = LayerNode::group(
            BoundingBox::default(),
            None,
            vec![leaf_with(&["바깥"]), deep],
            CacheHint::None,
            GroupKind::Generic,
        );

        let table = TextSourceTable::from_layer_node(&root);
        let texts: Vec<&str> = table.entries.iter().map(|e| e.text.as_str()).collect();
        assert_eq!(texts, vec!["바깥", "안쪽"], "중첩된 TextRun 을 놓쳤다");
    }

    /// id 는 수집 순서의 색인이다 — 소비자가 이 순서로 되짚는다.
    #[test]
    fn entry_ids_are_sequential_collection_indices() {
        let table = TextSourceTable::from_layer_node(&leaf_with(&["a", "b", "c"]));
        for (idx, entry) in table.entries.iter().enumerate() {
            assert_eq!(entry.id, TextSourceId(idx as u32));
        }
    }

    /// TextRun 이 없으면 표는 비어 있다 — 다른 op 를 텍스트로 착각하지 않는다.
    #[test]
    fn tree_without_text_runs_yields_empty_table() {
        let empty_leaf = LayerNode::leaf(BoundingBox::default(), None, vec![]);
        let root = LayerNode::group(
            BoundingBox::default(),
            None,
            vec![empty_leaf],
            CacheHint::None,
            GroupKind::Generic,
        );
        assert!(TextSourceTable::from_layer_node(&root).is_empty());
    }

    /// 문단 끝·줄바꿈 표식의 오프셋도 각자의 단위를 지켜야 한다.
    #[test]
    fn paragraph_end_annotation_offsets_follow_their_own_units() {
        let mut run = text_run("한글");
        run.is_para_end = true;
        let node = LayerNode::leaf(
            BoundingBox::default(),
            None,
            vec![PaintOp::text_run(BoundingBox::default(), run)],
        );

        let table = TextSourceTable::from_layer_node(&node);
        match &table.entries[0].annotations[..] {
            [TextSourceAnnotation::ParagraphEnd {
                offset_utf8,
                offset_utf16,
            }] => {
                assert_eq!(*offset_utf8, 6, "UTF-8 오프셋이 바이트 길이가 아니다");
                assert_eq!(*offset_utf16, 2, "UTF-16 오프셋이 코드유닛 수가 아니다");
            }
            other => panic!("기대한 ParagraphEnd 주석이 아니다: {other:?}"),
        }
    }

    /// 안정 키는 구역·문단·문자 시작이 다 있어야 만들어진다.
    #[test]
    fn stable_source_key_requires_section_and_para_index() {
        assert_eq!(stable_text_source_key(&text_run("가")), None);

        let mut run = text_run("가");
        run.section_index = Some(1);
        assert_eq!(
            stable_text_source_key(&run),
            None,
            "문단 색인이 없는데 키가 났다"
        );

        run.para_index = Some(2);
        assert_eq!(
            stable_text_source_key(&run).as_deref(),
            Some("section:1/para:2/char:0")
        );
    }
}
