//! 합성 장면 빌더 — 계약 시험이 쓰는 결정적 PaintOp 시퀀스.
//!
//! 장면은 실제 HWP 파일이 아니라 **한 페이지의 leaf op 목록**이다.
//! `replay_page` 가 plane 순서로 재정렬하므로, 고의로 뒤섞어 넣은 장면도
//! 추적 로그의 순서는 카탈로그 default plane 을 따른다.

use crate::model::control::FormType;
use crate::paint::{
    CacheHint, GroupKind, LayerNode, PageLayerTree, PaintOp, RenderProfile, TextDecorationKind,
};
use crate::renderer::equation::layout::{LayoutBox, LayoutKind};
use crate::renderer::render_tree::{
    BoundingBox, EllipseNode, EquationNode, FootnoteMarkerNode, FormObjectNode, ImageNode,
    LineNode, PageBackgroundNode, PathNode, PlaceholderNode, RawSvgNode, RectangleNode,
    TextRunNode,
};
use crate::renderer::{GradientFillInfo, LineStyle, PathCommand, ShapeStyle, TextStyle};

use super::catalog::{spec_for_kind, OpBounds};
use super::util::paint_op_kind;

/// 1×1 투명 PNG. 이미지 capability 시험용.
pub const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

/// 정직성 대조에 심는 고유 문자열.
pub const HONESTY_TEXT: &str = "M06-F-CAP";

/// 픽스처·빌더가 공유하는 한 op 의 선언.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneOp {
    /// 카탈로그 kind.
    pub kind: String,
    /// bbox (px).
    pub bounds: OpBounds,
    /// 선택 텍스트(textRun 계열).
    pub text: Option<String>,
    /// 그라디언트를 심을지.
    pub gradient: bool,
    /// 이미지를 실을지.
    pub image: bool,
}

impl SceneOp {
    /// kind + bbox 만으로 만든다.
    pub fn new(kind: impl Into<String>, x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            kind: kind.into(),
            bounds: OpBounds {
                x,
                y,
                width: w,
                height: h,
            },
            text: None,
            gradient: false,
            image: false,
        }
    }

    /// 텍스트를 붙인다.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// 그라디언트 채우기를 켠다.
    pub fn with_gradient(mut self) -> Self {
        self.gradient = true;
        self
    }

    /// 이미지 바이트를 싣는다.
    pub fn with_image(mut self) -> Self {
        self.image = true;
        self
    }
}

/// 한 페이지 장면.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneSpec {
    /// 안정 식별자 (`s001-empty` 형태).
    pub id: String,
    /// 페이지 폭 (px).
    pub width: f64,
    /// 페이지 높이 (px).
    pub height: f64,
    /// leaf op 목록. 트리 순서는 이 순서이고, 재생은 plane 으로 재정렬된다.
    pub ops: Vec<SceneOp>,
    /// 장면이 닫는 계약 한 줄.
    pub contract: String,
}

impl SceneSpec {
    /// 빈 페이지.
    pub fn empty(id: impl Into<String>, width: f64, height: f64) -> Self {
        Self {
            id: id.into(),
            width,
            height,
            ops: Vec::new(),
            contract: "빈 페이지도 begin/end 경계를 남긴다".into(),
        }
    }

    /// op 하나를 더한다.
    pub fn push(mut self, op: SceneOp) -> Self {
        self.ops.push(op);
        self
    }

    /// 계약 설명을 덮어쓴다.
    pub fn with_contract(mut self, contract: impl Into<String>) -> Self {
        self.contract = contract.into();
        self
    }

    /// `PageLayerTree` 로 만든다. 그룹 한 겹 + leaf 하나.
    pub fn to_layer_tree(&self) -> PageLayerTree {
        let bounds = BoundingBox::new(0.0, 0.0, self.width, self.height);
        let ops: Vec<PaintOp> = self.ops.iter().map(materialize_scene_op).collect();
        let leaf = LayerNode::leaf(bounds, None, ops);
        let root = LayerNode::group(
            bounds,
            None,
            vec![leaf],
            CacheHint::default(),
            GroupKind::Body,
        );
        PageLayerTree::with_profile(self.width, self.height, root, RenderProfile::Screen)
    }

    /// 재생 후 기대하는 kind 순서 (plane 재정렬 반영).
    pub fn expected_replay_kinds(&self) -> Vec<&'static str> {
        let tree = self.to_layer_tree();
        expected_kinds_after_replay(&tree)
    }
}

fn expected_kinds_after_replay(tree: &PageLayerTree) -> Vec<&'static str> {
    use crate::paint::{paint_op_replay_plane_with_layer, PaintReplayPlane};
    use crate::renderer::render_tree::RenderLayerInfo;

    fn walk<'a>(
        node: &'a LayerNode,
        inherited: Option<RenderLayerInfo>,
        plane: PaintReplayPlane,
        out: &mut Vec<&'static str>,
    ) {
        let active = node.layer.or(inherited);
        match &node.kind {
            crate::paint::LayerNodeKind::Group { children, .. } => {
                for child in children {
                    walk(child, active, plane, out);
                }
            }
            crate::paint::LayerNodeKind::ClipRect { child, .. } => {
                walk(child, active, plane, out);
            }
            crate::paint::LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if paint_op_replay_plane_with_layer(op, active) == plane {
                        out.push(paint_op_kind(op));
                    }
                }
            }
        }
    }

    let mut out = Vec::new();
    for plane in PaintReplayPlane::ORDERED {
        walk(&tree.root, None, plane, &mut out);
    }
    out
}

/// `SceneOp` 를 실제 `PaintOp` 로 만든다.
pub fn materialize_scene_op(op: &SceneOp) -> PaintOp {
    let b = BoundingBox::new(op.bounds.x, op.bounds.y, op.bounds.width, op.bounds.height);
    match op.kind.as_str() {
        "pageBackground" => PaintOp::page_background(b, page_background_node(op.gradient)),
        "textRun" => PaintOp::text_run(b, text_run_node(op.text.as_deref().unwrap_or("가"))),
        "charOverlap" => {
            PaintOp::char_overlap(b, text_run_node(op.text.as_deref().unwrap_or("겹")))
        }
        "textControlMark" => {
            PaintOp::text_control_mark(b, text_run_node(op.text.as_deref().unwrap_or("¶")))
        }
        "tabLeader" => PaintOp::tab_leader(b, text_run_node("......")),
        "textDecoration" => PaintOp::text_decoration(
            b,
            text_run_node(op.text.as_deref().unwrap_or("밑줄")),
            TextDecorationKind::Underline,
        ),
        "footnoteMarker" => PaintOp::footnote_marker(b, footnote_marker_node()),
        "line" => PaintOp::line(
            b,
            LineNode::new(
                op.bounds.x,
                op.bounds.y,
                op.bounds.x + op.bounds.width,
                op.bounds.y + op.bounds.height,
                LineStyle::default(),
            ),
        ),
        "rectangle" => {
            let gradient = if op.gradient {
                Some(Box::new(sample_gradient()))
            } else {
                None
            };
            PaintOp::rectangle(b, RectangleNode::new(0.0, ShapeStyle::default(), gradient))
        }
        "ellipse" => {
            let gradient = if op.gradient {
                Some(Box::new(sample_gradient()))
            } else {
                None
            };
            PaintOp::ellipse(b, EllipseNode::new(ShapeStyle::default(), gradient))
        }
        "path" => PaintOp::path(b, sample_path_node(op)),
        "image" => {
            let data = if op.image {
                Some(TINY_PNG.to_vec())
            } else {
                None
            };
            PaintOp::image(b, ImageNode::new(1, data), None)
        }
        "equation" => PaintOp::equation(b, sample_equation_node()),
        "formObject" => PaintOp::form_object(b, sample_form_node()),
        "placeholder" => {
            PaintOp::placeholder(b, PlaceholderNode::new(0x00FFFFFF, 0x00000000, "P".into()))
        }
        "rawSvg" => PaintOp::raw_svg(b, RawSvgNode::new("<g/>".into())),
        other => {
            // glyphRun/glyphOutline 는 셰이핑 입력이 필요해 사각형으로 대체하지 않는다.
            // 호출자가 카탈로그에 없는 kind 를 넣으면 여기서 바로 드러난다.
            panic!("materialize_scene_op: 지원하지 않는 kind `{other}`")
        }
    }
}

fn page_background_node(gradient: bool) -> PageBackgroundNode {
    PageBackgroundNode {
        background_color: None,
        border_color: None,
        border_width: 0.0,
        gradient: if gradient {
            Some(Box::new(sample_gradient()))
        } else {
            None
        },
        image: None,
    }
}

fn text_run_node(text: &str) -> TextRunNode {
    TextRunNode {
        text: text.to_string(),
        style: TextStyle {
            font_family: "sans-serif".to_string(),
            font_size: 16.0,
            ..TextStyle::default()
        },
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
        layout_positions: None,
        display_text: None,
    }
}

fn footnote_marker_node() -> FootnoteMarkerNode {
    FootnoteMarkerNode {
        number: 1,
        text: "1)".into(),
        base_font_size: 10.0,
        font_family: "sans-serif".into(),
        color: 0,
        section_index: 0,
        para_index: 0,
        control_index: 0,
    }
}

fn sample_gradient() -> GradientFillInfo {
    GradientFillInfo {
        gradient_type: 1,
        angle: 0,
        center_x: 50,
        center_y: 50,
        colors: vec![0x00FF0000, 0x000000FF],
        positions: vec![0.0, 1.0],
    }
}

fn sample_path_node(op: &SceneOp) -> PathNode {
    let x0 = op.bounds.x;
    let y0 = op.bounds.y;
    let x1 = op.bounds.x + op.bounds.width;
    let y1 = op.bounds.y + op.bounds.height;
    PathNode::new(
        vec![
            PathCommand::MoveTo(x0, y0),
            PathCommand::LineTo(x1, y0),
            PathCommand::LineTo(x1, y1),
            PathCommand::LineTo(x0, y1),
            PathCommand::ClosePath,
        ],
        ShapeStyle::default(),
        None,
    )
}

fn sample_equation_node() -> EquationNode {
    EquationNode {
        svg_content: "<text>x</text>".into(),
        layout_box: LayoutBox {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
            baseline: 8.0,
            kind: LayoutKind::Text("x".into()),
        },
        color_str: "#000000".into(),
        color: 0,
        font_size: 12.0,
        script: "x".into(),
        section_index: None,
        para_index: None,
        control_index: None,
        cell_index: None,
        cell_para_index: None,
        note_ref: None,
    }
}

fn sample_form_node() -> FormObjectNode {
    FormObjectNode {
        form_type: FormType::PushButton,
        caption: "확인".into(),
        text: String::new(),
        fore_color: "#000000".into(),
        back_color: "#ffffff".into(),
        value: 0,
        enabled: true,
        section_index: 0,
        para_index: 0,
        control_index: 0,
        name: "ok".into(),
        cell_location: None,
    }
}

/// 빌더가 만들 수 있는 kind. glyphRun/glyphOutline 은 제외.
pub fn materializable_kinds() -> &'static [&'static str] {
    &[
        "pageBackground",
        "textRun",
        "charOverlap",
        "textControlMark",
        "tabLeader",
        "textDecoration",
        "footnoteMarker",
        "line",
        "rectangle",
        "ellipse",
        "path",
        "image",
        "equation",
        "formObject",
        "placeholder",
        "rawSvg",
    ]
}

/// 내장 장면 목록 — 픽스처 생성기와 같은 식별자를 쓴다.
pub fn builtin_scenes() -> Vec<SceneSpec> {
    let mut scenes = Vec::new();
    scenes.push(
        SceneSpec::empty("s000-empty", 400.0, 300.0)
            .with_contract("빈 페이지도 begin_page/end_page 를 남긴다"),
    );
    scenes.push(
        SceneSpec::empty("s001-background", 400.0, 300.0)
            .push(SceneOp::new("pageBackground", 0.0, 0.0, 400.0, 300.0))
            .with_contract("배경만 있으면 Background plane 한 줄"),
    );
    scenes.push(
        SceneSpec::empty("s002-rect", 400.0, 300.0)
            .push(SceneOp::new("rectangle", 20.0, 20.0, 10.0, 10.0))
            .with_contract("사각형 하나"),
    );
    scenes.push(
        SceneSpec::empty("s003-line", 400.0, 300.0)
            .push(SceneOp::new("line", 0.0, 0.0, 50.0, 0.0))
            .with_contract("수평선 하나"),
    );
    scenes.push(
        SceneSpec::empty("s004-reorder", 400.0, 300.0)
            .push(SceneOp::new("rectangle", 20.0, 20.0, 10.0, 10.0))
            .push(SceneOp::new("line", 0.0, 0.0, 50.0, 0.0))
            .push(SceneOp::new("pageBackground", 0.0, 0.0, 400.0, 300.0))
            .with_contract("트리 순서가 뒤바뀌어도 배경이 먼저 재생된다"),
    );
    scenes.push(
        SceneSpec::empty("s005-text", 400.0, 300.0)
            .push(SceneOp::new("textRun", 10.0, 20.0, 120.0, 16.0).with_text(HONESTY_TEXT))
            .with_contract("벡터 텍스트 정직성 문자열"),
    );
    scenes.push(
        SceneSpec::empty("s006-gradient-rect", 400.0, 300.0)
            .push(SceneOp::new("rectangle", 0.0, 0.0, 80.0, 40.0).with_gradient())
            .with_contract("그라디언트 사각형"),
    );
    scenes.push(
        SceneSpec::empty("s007-image", 400.0, 300.0)
            .push(SceneOp::new("image", 0.0, 0.0, 8.0, 8.0).with_image())
            .with_contract("1x1 PNG 이미지"),
    );

    for (i, kind) in materializable_kinds().iter().enumerate() {
        let id = format!("s1{:02}-{kind}", i);
        let mut scene = SceneSpec::empty(id, 400.0, 300.0);
        let mut op = SceneOp::new(*kind, 12.0 + i as f64, 24.0, 40.0, 18.0);
        if *kind == "pageBackground" {
            op = SceneOp::new(*kind, 0.0, 0.0, 400.0, 300.0);
        }
        if *kind == "textRun" {
            op = op.with_text(HONESTY_TEXT);
        }
        if *kind == "image" {
            op = op.with_image();
        }
        scene = scene.push(op).with_contract(format!("{kind} 단독 장면"));
        scenes.push(scene);
    }

    scenes
}

/// 내장 장면 id 로 찾는다.
pub fn builtin_scene(id: &str) -> Option<SceneSpec> {
    builtin_scenes().into_iter().find(|s| s.id == id)
}

/// 장면의 모든 kind 가 카탈로그에 있는가.
pub fn scene_kinds_are_catalogued(scene: &SceneSpec) -> Result<(), String> {
    for op in &scene.ops {
        if spec_for_kind(&op.kind).is_none() {
            return Err(format!("{}: 카탈로그에 없는 kind {}", scene.id, op.kind));
        }
        if !op.bounds.is_finite() {
            return Err(format!("{}: 무한 bbox {:?}", scene.id, op.bounds));
        }
    }
    Ok(())
}
