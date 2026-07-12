//! 렌더 트리 노드 (RenderNode, Box Model)
//!
//! IR(Document Model)로부터 변환된 렌더링 전용 트리 구조.
//! 각 노드는 페이지 내 위치와 크기가 계산된 상태를 가진다.

use serde::Serialize;

use super::composer::CharOverlapInfo;
use super::layout::CellContext;
use super::{GradientFillInfo, LineStyle, PathCommand, ShapeStyle, TextStyle};
use crate::model::image::ImageEffect;
use crate::model::shape::TextWrap;
use crate::model::style::ImageFillMode;
use crate::model::{ColorRef, Rect};

pub const REAL_PICTURE_WATERMARK_PAGE_OPACITY: f64 = 0.26;
pub const REAL_PICTURE_WATERMARK_FILL_OPACITY: f64 = 0.15;
pub const REAL_PICTURE_WATERMARK_OPACITY: f64 = REAL_PICTURE_WATERMARK_PAGE_OPACITY;
pub const REAL_PICTURE_WATERMARK_SATURATION: f64 = 0.91646104;
pub const REAL_PICTURE_WATERMARK_CONTRAST: f64 = 0.93125103;
pub const REAL_PICTURE_WATERMARK_BRIGHTNESS: f64 = 1.80;
pub const REAL_PICTURE_WATERMARK_CORRECTION_MATRIX: [[f64; 3]; 3] = [
    [0.9897169325, 0.1297721480, -0.0666075849],
    [0.0236280401, 1.0778421442, -0.0471323620],
    [0.0002888270, -0.0075596780, 1.0728328592],
];
pub const REAL_PICTURE_WATERMARK_CORRECTION_BIAS: [f64; 3] =
    [-0.0504989415, -0.0462952328, -0.0573305296];
pub const REAL_PICTURE_WATERMARK_CHROMA_GAIN: f64 = 3.0;
pub const REAL_PICTURE_WATERMARK_WHITE_BLEND: f64 = 0.0;
pub const REAL_PICTURE_WATERMARK_FILL_CHROMA_GAIN: f64 = 0.42;
pub const REAL_PICTURE_WATERMARK_FILL_WHITE_BLEND: f64 = 0.16;
pub const LEGACY_IMAGE_WATERMARK_OPACITY: f64 = 0.17;

pub fn is_real_picture_watermark_tone_preset(
    effect: ImageEffect,
    brightness: i8,
    contrast: i8,
) -> bool {
    matches!(effect, ImageEffect::RealPic) && brightness == -50 && contrast == 70
}

/// 렌더 노드 고유 ID
pub type NodeId = u32;

/// Render-layer metadata shared by paper/page anchored Picture/Table/Shape nodes.
///
/// This is intentionally optional on `RenderNode`: ordinary flow nodes keep
/// `None`, while out-of-flow object nodes can carry the original HWPX/HWP
/// text-wrap and z-order contract without adding parallel fields to every leaf
/// node type.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLayerInfo {
    /// Text wrapping mode that decides the coarse replay plane.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_wrap: Option<TextWrap>,
    /// Original object z-order. Smaller values are painted first.
    pub z_order: i32,
    /// Stable tie-breaker within the same z-order.
    pub stable_index: u32,
}

impl RenderLayerInfo {
    pub fn new(text_wrap: Option<TextWrap>, z_order: i32, stable_index: u32) -> Self {
        Self {
            text_wrap,
            z_order,
            stable_index,
        }
    }
}

/// 렌더 노드 (페이지 내 렌더링 가능한 요소)
#[derive(Debug, Clone, Serialize)]
pub struct RenderNode {
    /// 노드 ID
    pub id: NodeId,
    /// 노드 종류
    pub node_type: RenderNodeType,
    /// 박스 모델 (위치, 크기, 여백)
    pub bbox: BoundingBox,
    /// 원본 객체 레이어 정보 (paper/page anchored Picture/Table/Shape용)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<RenderLayerInfo>,
    /// 자식 노드 목록
    pub children: Vec<RenderNode>,
    /// 변경 여부 플래그 (dirty flag for observer pattern)
    pub dirty: bool,
    /// 가시성
    pub visible: bool,
}

impl RenderNode {
    pub fn new(id: NodeId, node_type: RenderNodeType, bbox: BoundingBox) -> Self {
        Self {
            id,
            node_type,
            bbox,
            layer: None,
            children: Vec::new(),
            dirty: true,
            visible: true,
        }
    }

    /// 레이어 메타데이터를 부여한 노드를 반환한다.
    pub fn with_layer(mut self, layer: RenderLayerInfo) -> Self {
        self.layer = Some(layer);
        self
    }

    /// 기존 노드에 레이어 메타데이터를 부여한다.
    pub fn set_layer(&mut self, layer: RenderLayerInfo) {
        self.layer = Some(layer);
    }

    /// dirty 플래그 설정 (변경된 노드만 재렌더링)
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    /// 렌더링 완료 후 dirty 플래그 초기화
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// 이 노드와 모든 자식의 dirty 플래그 초기화
    pub fn mark_clean_recursive(&mut self) {
        self.dirty = false;
        for child in &mut self.children {
            child.mark_clean_recursive();
        }
    }

    /// dirty 노드가 있는지 확인
    pub fn has_dirty_nodes(&self) -> bool {
        if self.dirty {
            return true;
        }
        self.children.iter().any(|c| c.has_dirty_nodes())
    }

    /// 렌더 트리를 JSON 문자열로 직렬화한다.
    pub fn to_json(&self) -> String {
        let mut buf = String::with_capacity(4096);
        self.write_json(&mut buf);
        buf
    }

    fn write_json(&self, buf: &mut String) {
        buf.push('{');
        // type
        let (type_str, extra) = match &self.node_type {
            RenderNodeType::Page(_) => ("Page", String::new()),
            RenderNodeType::PageBackground(_) => ("PageBg", String::new()),
            RenderNodeType::MasterPage => ("MasterPage", String::new()),
            RenderNodeType::Header => ("Header", String::new()),
            RenderNodeType::Footer => ("Footer", String::new()),
            RenderNodeType::Body { .. } => ("Body", String::new()),
            RenderNodeType::Column(c) => ("Column", format!(",\"col\":{}", c)),
            RenderNodeType::FootnoteArea => ("FootnoteArea", String::new()),
            RenderNodeType::TextLine(tl) => (
                "TextLine",
                format!(",\"pi\":{}", tl.para_index.unwrap_or(0)),
            ),
            RenderNodeType::TextRun(tr) => (
                "TextRun",
                format!(
                    ",\"text\":{},\"pi\":{}",
                    json_escape(&tr.text),
                    tr.section_index
                        .map(|_| tr.para_index.unwrap_or(0))
                        .unwrap_or(0)
                ),
            ),
            RenderNodeType::Table(tn) => (
                "Table",
                format!(
                    ",\"rows\":{},\"cols\":{}{}{}",
                    tn.row_count,
                    tn.col_count,
                    tn.para_index
                        .map(|pi| format!(",\"pi\":{}", pi))
                        .unwrap_or_default(),
                    tn.control_index
                        .map(|ci| format!(",\"ci\":{}", ci))
                        .unwrap_or_default()
                ),
            ),
            RenderNodeType::TableCell(tc) => {
                ("Cell", format!(",\"row\":{},\"col\":{}", tc.row, tc.col))
            }
            RenderNodeType::Image(_) => ("Image", String::new()),
            RenderNodeType::TextBox => ("TextBox", String::new()),
            RenderNodeType::Equation(_) => ("Equation", String::new()),
            RenderNodeType::Line(_) => ("Line", String::new()),
            RenderNodeType::Rectangle(_) => ("Rect", String::new()),
            RenderNodeType::Ellipse(_) => ("Ellipse", String::new()),
            RenderNodeType::Path(_) => ("Path", String::new()),
            RenderNodeType::Group(_) => ("Group", String::new()),
            RenderNodeType::FormObject(_) => ("Form", String::new()),
            RenderNodeType::FootnoteMarker(_) => ("FnMarker", String::new()),
            RenderNodeType::Placeholder(ph) => ("Placeholder", ph.control_ref_json_extra()),
            RenderNodeType::RawSvg(raw) => ("RawSvg", raw.control_ref_json_extra()),
        };
        buf.push_str(&format!(
            "\"type\":\"{}\",\"bbox\":{{\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}}}",
            type_str, self.bbox.x, self.bbox.y, self.bbox.width, self.bbox.height
        ));
        buf.push_str(&extra);
        if !self.children.is_empty() {
            buf.push_str(",\"children\":[");
            for (i, child) in self.children.iter().enumerate() {
                if i > 0 {
                    buf.push(',');
                }
                child.write_json(buf);
            }
            buf.push(']');
        }
        buf.push('}');
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// 렌더 노드 종류
#[derive(Debug, Clone, Serialize)]
pub enum RenderNodeType {
    /// 페이지 루트 노드
    Page(PageNode),
    /// 페이지 배경/테두리
    PageBackground(PageBackgroundNode),
    /// 바탕쪽 영역
    MasterPage,
    /// 머리말 영역
    Header,
    /// 꼬리말 영역
    Footer,
    /// 본문 영역
    Body {
        /// 콘텐츠 클리핑 영역 (페이지 경계 넘침 방지)
        clip_rect: Option<BoundingBox>,
    },
    /// 단(Column) 영역
    Column(u16),
    /// 각주 영역
    FootnoteArea,
    /// 텍스트 줄
    TextLine(TextLineNode),
    /// 텍스트 런 (동일 글자 모양의 텍스트 조각)
    TextRun(TextRunNode),
    /// 표
    Table(TableNode),
    /// 표 셀
    TableCell(TableCellNode),
    /// 직선
    Line(LineNode),
    /// 사각형
    Rectangle(RectangleNode),
    /// 타원
    Ellipse(EllipseNode),
    /// 패스 (다각형, 곡선, 호)
    Path(PathNode),
    /// 이미지
    Image(ImageNode),
    /// 묶음 개체
    Group(GroupNode),
    /// 글상자 (텍스트가 포함된 그리기 개체)
    TextBox,
    /// 수식
    Equation(EquationNode),
    /// 양식 개체
    FormObject(FormObjectNode),
    /// 각주/미주 마커 (인라인 위첨자)
    FootnoteMarker(FootnoteMarkerNode),
    /// 차트/OLE placeholder (배경 rect + 중앙 텍스트 라벨) — Task #195
    Placeholder(PlaceholderNode),
    /// 이미 생성된 SVG 조각을 그대로 출력 (OOXML 차트 등) — Task #195 단계 8
    RawSvg(RawSvgNode),
}

/// RawSvg/placeholder가 원본 문서 개체와 연결될 때 쓰는 상호작용 메타.
#[derive(Debug, Clone, Serialize)]
pub struct ObjectControlRef {
    pub section_index: usize,
    pub para_index: usize,
    pub control_index: usize,
    pub kind: &'static str,
}

impl ObjectControlRef {
    pub fn ole(section_index: usize, para_index: usize, control_index: usize) -> Self {
        Self {
            section_index,
            para_index,
            control_index,
            kind: "ole",
        }
    }

    fn json_extra(&self) -> String {
        format!(
            ",\"kind\":\"{}\",\"si\":{},\"pi\":{},\"ci\":{}",
            self.kind, self.section_index, self.para_index, self.control_index
        )
    }
}

/// 미리 렌더된 SVG 조각 (Task #195 단계 8)
#[derive(Debug, Clone, Serialize)]
pub struct RawSvgNode {
    /// 삽입할 SVG 조각 (유효한 `<g>...</g>` 또는 개별 요소)
    pub svg: String,
    /// 원본 개체 참조. OLE RawSvg 선택/속성 진입에 사용한다.
    pub control_ref: Option<ObjectControlRef>,
}

impl RawSvgNode {
    pub fn new(svg: String) -> Self {
        Self {
            svg,
            control_ref: None,
        }
    }

    pub fn ole(svg: String, section_index: usize, para_index: usize, control_index: usize) -> Self {
        Self {
            svg,
            control_ref: Some(ObjectControlRef::ole(
                section_index,
                para_index,
                control_index,
            )),
        }
    }

    fn control_ref_json_extra(&self) -> String {
        self.control_ref
            .as_ref()
            .map(ObjectControlRef::json_extra)
            .unwrap_or_default()
    }
}

/// 차트/OLE placeholder 렌더 노드 (Task #195)
#[derive(Debug, Clone, Serialize)]
pub struct PlaceholderNode {
    /// 배경 색상 (ARGB)
    pub fill_color: u32,
    /// 테두리 색상 (ARGB)
    pub stroke_color: u32,
    /// 표시할 라벨(중앙 정렬)
    pub label: String,
    /// 원본 개체 참조. OLE fallback placeholder 선택/속성 진입에 사용한다.
    pub control_ref: Option<ObjectControlRef>,
}

impl PlaceholderNode {
    pub fn new(fill_color: u32, stroke_color: u32, label: String) -> Self {
        Self {
            fill_color,
            stroke_color,
            label,
            control_ref: None,
        }
    }

    pub fn ole(
        fill_color: u32,
        stroke_color: u32,
        label: String,
        section_index: usize,
        para_index: usize,
        control_index: usize,
    ) -> Self {
        Self {
            fill_color,
            stroke_color,
            label,
            control_ref: Some(ObjectControlRef::ole(
                section_index,
                para_index,
                control_index,
            )),
        }
    }

    fn control_ref_json_extra(&self) -> String {
        self.control_ref
            .as_ref()
            .map(ObjectControlRef::json_extra)
            .unwrap_or_default()
    }
}

/// 각주/미주 마커 렌더 노드
#[derive(Debug, Clone, Serialize)]
pub struct FootnoteMarkerNode {
    /// 각주 번호
    pub number: u16,
    /// 위첨자 텍스트 ("1)" 등)
    pub text: String,
    /// 기본 폰트 크기 (본문 크기, 위첨자는 이것의 55%)
    pub base_font_size: f64,
    /// 폰트 패밀리
    pub font_family: String,
    /// 글자 색
    pub color: u32,
    /// 소속 구역/문단 인덱스
    pub section_index: usize,
    pub para_index: usize,
    /// 문단 내 컨트롤 인덱스
    pub control_index: usize,
}

/// 양식 개체 렌더 노드
#[derive(Debug, Clone, Serialize)]
pub struct FormObjectNode {
    /// 양식 개체 타입
    pub form_type: crate::model::control::FormType,
    /// 캡션 (PushButton, CheckBox, RadioButton)
    pub caption: String,
    /// 텍스트 (ComboBox, Edit)
    pub text: String,
    /// 글자 색 (CSS #rrggbb)
    pub fore_color: String,
    /// 배경 색 (CSS #rrggbb)
    pub back_color: String,
    /// 선택 상태 (CheckBox/RadioButton)
    pub value: i32,
    /// 활성화 여부
    pub enabled: bool,
    /// 문서 위치: 구역 인덱스
    pub section_index: usize,
    /// 문서 위치: 문단 인덱스 (셀 내부인 경우 셀 내 문단 인덱스)
    pub para_index: usize,
    /// 문서 위치: 컨트롤 인덱스
    pub control_index: usize,
    /// 양식 개체 이름
    pub name: String,
    /// 셀 내부 위치 (표 셀 안에 있는 경우)
    /// (table_para_index, table_control_index, cell_index, cell_para_index)
    pub cell_location: Option<(usize, usize, usize, usize)>,
}

/// 바운딩 박스 (위치 + 크기, 픽셀 단위)
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct BoundingBox {
    /// X 좌표 (페이지 내 절대 위치, px)
    pub x: f64,
    /// Y 좌표 (페이지 내 절대 위치, px)
    pub y: f64,
    /// 폭 (px)
    pub width: f64,
    /// 높이 (px)
    pub height: f64,
}

impl BoundingBox {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 다른 박스와 겹치는지 확인
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// 다른 박스를 포함하는지 확인
    pub fn contains(&self, other: &BoundingBox) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.x + self.width >= other.x + other.width
            && self.y + self.height >= other.y + other.height
    }

    /// HWPUNIT Rect를 픽셀 BoundingBox로 변환
    pub fn from_hwpunit_rect(rect: &Rect, dpi: f64) -> Self {
        let scale = dpi / super::HWPUNIT_PER_INCH;
        Self {
            x: rect.left as f64 * scale,
            y: rect.top as f64 * scale,
            width: rect.width() as f64 * scale,
            height: rect.height() as f64 * scale,
        }
    }
}

/// 페이지 노드
#[derive(Debug, Clone, Serialize)]
pub struct PageNode {
    /// 페이지 번호 (0-based)
    pub page_index: u32,
    /// 페이지 폭 (px)
    pub width: f64,
    /// 페이지 높이 (px)
    pub height: f64,
    /// 소속 구역 인덱스
    pub section_index: usize,
}

/// 페이지 배경 노드
#[derive(Debug, Clone, Serialize)]
pub struct PageBackgroundNode {
    /// 배경색
    pub background_color: Option<ColorRef>,
    /// 테두리 색상
    pub border_color: Option<ColorRef>,
    /// 테두리 두께
    pub border_width: f64,
    /// 그러데이션 채우기 (fill_color보다 우선)
    pub gradient: Option<Box<GradientFillInfo>>,
    /// 이미지 채우기 (gradient/fill_color보다 우선)
    pub image: Option<PageBackgroundImage>,
}

/// 페이지 배경 이미지 정보
#[derive(Debug, Clone, Serialize)]
pub struct PageBackgroundImage {
    /// 이미지 데이터 (JSON 직렬화 시 제외)
    #[serde(skip)]
    pub data: Vec<u8>,
    /// 이미지 채우기 모드
    pub fill_mode: ImageFillMode,
    /// 밝기
    pub brightness: i8,
    /// 명암
    pub contrast: i8,
    /// 그림 효과
    pub effect: ImageEffect,
}

impl PageBackgroundImage {
    /// 워터마크 효과 적용 여부 (Issue #1156).
    ///
    /// HWP/HWPX 에는 워터마크 적용 비트가 없다. 한컴 편집기는 "워터마크 효과"
    /// 해제 시 밝기·대비를 모두 0 으로 되돌리므로, 밝기·대비가 둘 다 0 이 아닌
    /// 경우 워터마크로 판정한다 (한쪽이라도 0 이면 워터마크 아님, effect 무관).
    pub fn is_watermark(&self) -> bool {
        self.brightness != 0 && self.contrast != 0
    }

    pub fn is_real_picture_watermark_tone_preset(&self) -> bool {
        is_real_picture_watermark_tone_preset(self.effect, self.brightness, self.contrast)
    }
}

/// 텍스트 줄 노드
#[derive(Debug, Clone, Serialize)]
pub struct TextLineNode {
    /// 줄 높이 (px)
    pub line_height: f64,
    /// 베이스라인 위치 (줄 상단으로부터, px)
    pub baseline: f64,
    /// 소속 구역 인덱스 (빈 문단 커서 위치 계산용)
    pub section_index: Option<usize>,
    /// 소속 문단 인덱스 (빈 문단 커서 위치 계산용)
    pub para_index: Option<usize>,
    /// 문단 내 줄 인덱스 (디버그 오버레이용)
    pub line_index: Option<u32>,
    /// LINE_SEG vertical_pos (HWPUNIT, 디버그 오버레이/vpos-reset 검출용)
    pub vpos: Option<i32>,
}

impl TextLineNode {
    /// 기본 생성 (문단 식별 정보 없음)
    pub fn new(line_height: f64, baseline: f64) -> Self {
        Self {
            line_height,
            baseline,
            section_index: None,
            para_index: None,
            line_index: None,
            vpos: None,
        }
    }

    /// 문단 식별 정보 포함 생성 (커서 위치 계산용)
    pub fn with_para(
        line_height: f64,
        baseline: f64,
        section_index: usize,
        para_index: usize,
    ) -> Self {
        Self {
            line_height,
            baseline,
            section_index: Some(section_index),
            para_index: Some(para_index),
            line_index: None,
            vpos: None,
        }
    }

    /// 문단 식별 + LINE_SEG vpos 정보 포함 생성 (디버그 오버레이용)
    pub fn with_para_vpos(
        line_height: f64,
        baseline: f64,
        section_index: usize,
        para_index: usize,
        line_index: u32,
        vpos: i32,
    ) -> Self {
        Self {
            line_height,
            baseline,
            section_index: Some(section_index),
            para_index: Some(para_index),
            line_index: Some(line_index),
            vpos: Some(vpos),
        }
    }
}

/// 텍스트 런 노드 (동일 글자 모양의 연속 텍스트)
#[derive(Debug, Clone, Serialize)]
pub struct TextRunNode {
    /// 텍스트 내용
    pub text: String,
    /// 텍스트 스타일
    pub style: TextStyle,
    /// 글자 모양 ID (서식 툴바용)
    pub char_shape_id: Option<u32>,
    /// 문단 모양 ID (서식 툴바용)
    pub para_shape_id: Option<u16>,
    /// 소속 구역 인덱스 (편집용)
    pub section_index: Option<usize>,
    /// 소속 문단 인덱스 (편집용)
    pub para_index: Option<usize>,
    /// 문단 내 문자 시작 오프셋 (편집용)
    pub char_start: Option<usize>,
    /// 표 셀 컨텍스트 (경로 기반, 중첩 표 지원)
    pub cell_context: Option<CellContext>,
    /// 문단 마지막 TextRun 여부 (문단부호 표시용)
    pub is_para_end: bool,
    /// 강제 줄 바꿈(Shift+Enter) 줄의 마지막 TextRun 여부
    pub is_line_break_end: bool,
    /// 글자 회전 각도 (도, 시계방향). 세로쓰기 괄호 등에 사용.
    pub rotation: f64,
    /// 세로쓰기 셀 내 글자 여부 (문단부호 위치 조정용)
    pub is_vertical: bool,
    /// 글자겹침 정보 (CharOverlap 컨트롤 렌더링용)
    pub char_overlap: Option<CharOverlapInfo>,
    /// 글자 테두리/배경 ID (1-based, 0이면 없음)
    pub border_fill_id: u16,
    /// 베이스라인 위치 (bbox.y로부터의 거리, px)
    pub baseline: f64,
    /// 누름틀 필드 마커: 이 TextRun 위치에 표시할 필드 경계 마커
    pub field_marker: FieldMarkerType,
}

/// 누름틀 필드 조판부호 마커 유형
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize)]
pub enum FieldMarkerType {
    #[default]
    None,
    /// 누름틀 시작 ([누름틀 시작])
    FieldBegin,
    /// 누름틀 끝 ([누름틀 끝])
    FieldEnd,
    /// 시작+끝 동시 (빈 필드: start == end)
    FieldBeginEnd,
    /// 도형 조판부호 마커 — 인라인 컨트롤의 텍스트 위치
    ShapeMarker(usize),
}

/// 표 노드
#[derive(Debug, Clone, Serialize)]
pub struct TableNode {
    /// 행 수
    pub row_count: u16,
    /// 열 수
    pub col_count: u16,
    /// 테두리/배경 ID
    pub border_fill_id: u16,
    /// 소속 구역 인덱스
    pub section_index: Option<usize>,
    /// 표 컨트롤을 소유한 문단 인덱스
    pub para_index: Option<usize>,
    /// 문단 내 컨트롤 인덱스
    pub control_index: Option<usize>,
}

/// 표 셀 노드
#[derive(Debug, Clone, Serialize)]
pub struct TableCellNode {
    /// 열 위치
    pub col: u16,
    /// 행 위치
    pub row: u16,
    /// 열 병합 수
    pub col_span: u16,
    /// 행 병합 수
    pub row_span: u16,
    /// 테두리/배경 ID
    pub border_fill_id: u16,
    /// 텍스트 방향 (0=가로, 1=세로/영문눕힘, 2=세로/영문세움)
    pub text_direction: u8,
    /// 셀 콘텐츠를 bounding box로 클리핑 (분할 행 셀에서 사용)
    pub clip: bool,
    /// 모델 cells 배열 내 인덱스 (getTableCellBboxes에서 resize용)
    pub model_cell_index: Option<u32>,
}

/// 도형 변환 정보 (회전/대칭)
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ShapeTransform {
    /// 회전각 (도, 시계방향)
    pub rotation: f64,
    /// 좌우 대칭
    pub horz_flip: bool,
    /// 상하 대칭
    pub vert_flip: bool,
}

impl ShapeTransform {
    /// 변환이 필요한지 여부
    pub fn has_transform(&self) -> bool {
        self.rotation != 0.0 || self.horz_flip || self.vert_flip
    }

    /// 그림 노드 한정: 회전각 90°/270° (±1° 톨러런스) 일 때 bbox extent 만 swap.
    /// 그 외 각도(0/45/180 등)는 입력 bbox 그대로 반환.
    ///
    /// hwpx 의 `<hp:sz>` 는 회전 후 외접 사각형 치수라서 portrait → 90° 회전 시
    /// bbox 가 landscape 로 들어온다. 그 위에 `rotate(90, cx, cy)` transform 을 다시
    /// 적용하면 이중회전이 되어 페이지 밖으로 튀어 나간다. 회전 전 치수로 swap 한
    /// bbox 위에 동일 rotation transform 을 적용하면 한컴 정답지와 정합한다.
    /// cx, cy 는 swap 전후 동일하므로 rotate 식 자체는 그대로 둔다.
    pub fn effective_image_bbox(&self, bbox: &BoundingBox) -> BoundingBox {
        let r = self.rotation.rem_euclid(360.0);
        let is_perpendicular = (r - 90.0).abs() < 1.0 || (r - 270.0).abs() < 1.0;
        if !is_perpendicular {
            return *bbox;
        }
        let cx = bbox.x + bbox.width / 2.0;
        let cy = bbox.y + bbox.height / 2.0;
        let new_w = bbox.height;
        let new_h = bbox.width;
        BoundingBox::new(cx - new_w / 2.0, cy - new_h / 2.0, new_w, new_h)
    }
}

/// 직선 노드
#[derive(Debug, Clone, Serialize)]
pub struct LineNode {
    /// 시작점 (px)
    pub x1: f64,
    pub y1: f64,
    /// 끝점 (px)
    pub x2: f64,
    pub y2: f64,
    /// 선 스타일
    pub style: LineStyle,
    /// 소속 구역 인덱스
    pub section_index: Option<usize>,
    /// 도형 컨트롤을 소유한 문단 인덱스
    pub para_index: Option<usize>,
    /// 문단 내 컨트롤 인덱스
    pub control_index: Option<usize>,
    /// 변환 (회전/대칭)
    pub transform: ShapeTransform,
    /// [Task #1138] 표 셀 내 도형인 경우: 셀 인덱스
    #[serde(default)]
    pub cell_index: Option<usize>,
    /// [Task #1138] 표 셀 내 도형인 경우: 셀 내 문단 인덱스
    #[serde(default)]
    pub cell_para_index: Option<usize>,
    /// [Task #1138] 표 셀 내 도형인 경우: outer paragraph 의 표 control 인덱스
    #[serde(default)]
    pub outer_table_control_index: Option<usize>,
}

impl LineNode {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, style: LineStyle) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            style,
            section_index: None,
            para_index: None,
            control_index: None,
            transform: ShapeTransform::default(),
            cell_index: None,
            cell_para_index: None,
            outer_table_control_index: None,
        }
    }
}

/// 사각형 노드
#[derive(Debug, Clone, Serialize)]
pub struct RectangleNode {
    /// 모서리 곡률 (px)
    pub corner_radius: f64,
    /// 도형 스타일
    pub style: ShapeStyle,
    /// 그라데이션 채우기 (style.fill_color보다 우선)
    pub gradient: Option<Box<GradientFillInfo>>,
    /// 소속 구역 인덱스
    pub section_index: Option<usize>,
    /// 도형 컨트롤을 소유한 문단 인덱스
    pub para_index: Option<usize>,
    /// 문단 내 컨트롤 인덱스
    pub control_index: Option<usize>,
    /// 변환 (회전/대칭)
    pub transform: ShapeTransform,
    /// [Task #1138] 표 셀 내 도형인 경우: 셀 인덱스
    #[serde(default)]
    pub cell_index: Option<usize>,
    /// [Task #1138] 표 셀 내 도형인 경우: 셀 내 문단 인덱스
    #[serde(default)]
    pub cell_para_index: Option<usize>,
    /// [Task #1138] 표 셀 내 도형인 경우: outer paragraph 의 표 control 인덱스
    #[serde(default)]
    pub outer_table_control_index: Option<usize>,
}

impl RectangleNode {
    pub fn new(
        corner_radius: f64,
        style: ShapeStyle,
        gradient: Option<Box<GradientFillInfo>>,
    ) -> Self {
        Self {
            corner_radius,
            style,
            gradient,
            section_index: None,
            para_index: None,
            control_index: None,
            transform: ShapeTransform::default(),
            cell_index: None,
            cell_para_index: None,
            outer_table_control_index: None,
        }
    }
}

/// 타원 노드
#[derive(Debug, Clone, Serialize)]
pub struct EllipseNode {
    /// 도형 스타일
    pub style: ShapeStyle,
    /// 그라데이션 채우기 (style.fill_color보다 우선)
    pub gradient: Option<Box<GradientFillInfo>>,
    /// 소속 구역 인덱스
    pub section_index: Option<usize>,
    /// 도형 컨트롤을 소유한 문단 인덱스
    pub para_index: Option<usize>,
    /// 문단 내 컨트롤 인덱스
    pub control_index: Option<usize>,
    /// 변환 (회전/대칭)
    pub transform: ShapeTransform,
    /// [Task #1138] 표 셀 내 도형인 경우: 셀 인덱스
    #[serde(default)]
    pub cell_index: Option<usize>,
    /// [Task #1138] 표 셀 내 도형인 경우: 셀 내 문단 인덱스
    #[serde(default)]
    pub cell_para_index: Option<usize>,
    /// [Task #1138] 표 셀 내 도형인 경우: outer paragraph 의 표 control 인덱스
    #[serde(default)]
    pub outer_table_control_index: Option<usize>,
}

impl EllipseNode {
    pub fn new(style: ShapeStyle, gradient: Option<Box<GradientFillInfo>>) -> Self {
        Self {
            style,
            gradient,
            section_index: None,
            para_index: None,
            control_index: None,
            transform: ShapeTransform::default(),
            cell_index: None,
            cell_para_index: None,
            outer_table_control_index: None,
        }
    }
}

/// 패스 노드
#[derive(Debug, Clone, Serialize)]
pub struct PathNode {
    /// 패스 커맨드 목록
    pub commands: Vec<PathCommand>,
    /// 도형 스타일
    pub style: ShapeStyle,
    /// 그라데이션 채우기 (style.fill_color보다 우선)
    pub gradient: Option<Box<GradientFillInfo>>,
    /// 소속 구역 인덱스
    pub section_index: Option<usize>,
    /// 도형 컨트롤을 소유한 문단 인덱스
    pub para_index: Option<usize>,
    /// 문단 내 컨트롤 인덱스
    pub control_index: Option<usize>,
    /// 변환 (회전/대칭)
    pub transform: ShapeTransform,
    /// 연결선 시작/끝 좌표 (선 선택 방식용, None이면 일반 도형)
    pub connector_endpoints: Option<(f64, f64, f64, f64)>,
    /// 연결선 화살표 (LineStyle 포함, None이면 화살표 없음)
    pub line_style: Option<LineStyle>,
    /// [Task #1138] 표 셀 내 도형인 경우: 셀 인덱스
    #[serde(default)]
    pub cell_index: Option<usize>,
    /// [Task #1138] 표 셀 내 도형인 경우: 셀 내 문단 인덱스
    #[serde(default)]
    pub cell_para_index: Option<usize>,
    /// [Task #1138] 표 셀 내 도형인 경우: outer paragraph 의 표 control 인덱스
    #[serde(default)]
    pub outer_table_control_index: Option<usize>,
}

impl PathNode {
    pub fn new(
        commands: Vec<PathCommand>,
        style: ShapeStyle,
        gradient: Option<Box<GradientFillInfo>>,
    ) -> Self {
        Self {
            commands,
            style,
            gradient,
            section_index: None,
            para_index: None,
            control_index: None,
            transform: ShapeTransform::default(),
            connector_endpoints: None,
            line_style: None,
            cell_index: None,
            cell_para_index: None,
            outer_table_control_index: None,
        }
    }
}

/// 이미지 노드
#[derive(Debug, Clone, Serialize)]
pub struct ImageNode {
    /// BinData ID 참조
    pub bin_data_id: u16,
    /// 이미지 데이터 (캐시용, JSON 직렬화 시 제외)
    #[serde(skip)]
    pub data: Option<Vec<u8>>,
    /// 소속 구역 인덱스
    pub section_index: Option<usize>,
    /// 이미지 컨트롤을 소유한 문단 인덱스
    pub para_index: Option<usize>,
    /// 문단 내 컨트롤 인덱스
    pub control_index: Option<usize>,
    /// 이미지 채우기 모드 (채우기 이미지용, Picture 컨트롤은 None)
    pub fill_mode: Option<ImageFillMode>,
    /// 이미지 원본 크기 (HWPUNIT 기반, SVG 좌표 변환 후)
    /// fill_mode가 배치 모드일 때 사용 (원래 크기대로 배치)
    pub original_size: Option<(f64, f64)>,
    /// 변환 (회전/대칭)
    pub transform: ShapeTransform,
    /// 그림 자르기: "자르기 한 후 사각형" 원본 좌표 (left, top, right, bottom)
    /// 렌더러에서 이미지 원본 px 크기와 비교하여 source rect 계산
    /// None이면 전체 이미지 표시
    pub crop: Option<(i32, i32, i32, i32)>,
    /// 원본 이미지 크기 (HWPUNIT) — `pic.shape_attr.{original_width, original_height}`.
    /// crop 좌표를 픽셀로 변환할 때 정확한 HU/px 스케일 계산에 사용.
    /// None이면 폴백 동작.
    pub original_size_hu: Option<(u32, u32)>,
    /// 그림 효과 (실사/그레이스케일/흑백/패턴)
    pub effect: ImageEffect,
    /// 밝기 (-100 ~ +100)
    pub brightness: i8,
    /// 명암(대비) (-100 ~ +100)
    pub contrast: i8,
    /// 그림 개체 전체 불투명도. 1.0=불투명, 0.0=완전 투명.
    pub opacity: f64,
    /// 텍스트 흐름 wrap 모드 (Task #516, 다층 레이어 분리용).
    /// `None` 또는 `Some(Square/TopAndBottom/Tight/Through)` 는 본문 layer 에 포함되고,
    /// `Some(BehindText)` / `Some(InFrontOfText)` 는 overlay layer 로 분리 후보.
    /// 기본값 `None` 은 기존 동작 유지.
    pub text_wrap: Option<TextWrap>,
    /// [Task #741] 외부 file path 그림 (HWP3 spec offset 74 그림 종류 0=외부 파일,
    /// 1=OLE, 2=Embedded Image / offset 83~339 그림 파일 이름).
    /// `data` 가 `None` 이고 `external_path` 가 `Some` 인 경우 placeholder 표시
    /// (점선 사각형 + 깨진 image 아이콘) — 한컴 한글 2024 viewer 정합.
    #[serde(default)]
    pub external_path: Option<String>,
    /// [Task #825] 머리말/꼬리말 그림 식별 marker.
    /// `Some(ref)` 일 때 본 ImageNode 는 머리말 또는 꼬리말 안에 위치하며,
    /// `para_index` / `control_index` 는 `Header.paragraphs[]` / `Footer.paragraphs[]`
    /// 의 inner 인덱스를 가리킨다. `outer` 는 본문 paragraph 의 Header/Footer 컨트롤
    /// 위치 (body_para_idx + header_ctrl_idx) 를 보존.
    /// `None` 일 때 본문 그림 (현행 동작).
    #[serde(default)]
    pub header_footer_ref: Option<HeaderFooterImageRef>,
    /// [Task #1151 v4] 표 셀 안 inline picture (tac=true) 인 경우: 셀 인덱스.
    /// Rectangle/Ellipse/Path 의 [Task #1138] 패턴 정합. `None` 이면 셀 외부 picture.
    #[serde(default)]
    pub cell_index: Option<usize>,
    /// [Task #1151 v4] 표 셀 안 inline picture 인 경우: 셀 내 문단 인덱스.
    #[serde(default)]
    pub cell_para_index: Option<usize>,
    /// [Task #1151 v4] 표 셀 안 inline picture 인 경우: outer paragraph 의 표 control 인덱스.
    #[serde(default)]
    pub outer_table_control_index: Option<usize>,
    /// [Task #1161] 표 셀/글상자 안 picture 의 **전체 다단계 경로**(중첩 표/글상자 지원).
    /// `TextRunNode.cell_context` 와 동일 메커니즘. 위 단일 레벨 스칼라
    /// (cell_index/cell_para_index/outer_table_control_index)는 이 경로의 innermost
    /// 투영(`CellContext::last_image_indices`)으로 유지(하위호환). 본문 picture 는 `None`.
    #[serde(default)]
    pub cell_context: Option<CellContext>,
}

/// [Task #825] 머리말/꼬리말 안 그림의 outer 위치 + 종류.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct HeaderFooterImageRef {
    /// 본문 paragraph 인덱스 (Header/Footer 컨트롤 소속 paragraph)
    pub outer_para_index: usize,
    /// 본문 paragraph 안 Header/Footer 컨트롤 인덱스
    pub outer_control_index: usize,
    /// "header" or "footer"
    pub kind: HeaderFooterKind,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HeaderFooterKind {
    Header,
    Footer,
}

impl ImageNode {
    /// 워터마크 효과 적용 여부 (Issue #1156).
    /// 밝기·대비가 둘 다 0 이 아닌 경우 워터마크 (한쪽이라도 0 이면 아님, effect 무관).
    /// HWP/HWPX 에 워터마크 비트는 없으며 한컴은 해제 시 밝기·대비를 0/0 으로 되돌린다.
    pub fn is_watermark(&self) -> bool {
        self.brightness != 0 && self.contrast != 0
    }

    pub fn is_real_picture_watermark_tone_preset(&self) -> bool {
        is_real_picture_watermark_tone_preset(self.effect, self.brightness, self.contrast)
    }

    pub fn new(bin_data_id: u16, data: Option<Vec<u8>>) -> Self {
        Self {
            bin_data_id,
            data,
            section_index: None,
            para_index: None,
            control_index: None,
            fill_mode: None,
            original_size: None,
            transform: ShapeTransform::default(),
            crop: None,
            original_size_hu: None,
            effect: ImageEffect::RealPic,
            brightness: 0,
            contrast: 0,
            opacity: 1.0,
            text_wrap: None,
            external_path: None,
            header_footer_ref: None,
            cell_index: None,
            cell_para_index: None,
            outer_table_control_index: None,
            cell_context: None,
        }
    }
}

/// 묶음 개체 노드
#[derive(Debug, Clone, Serialize)]
pub struct GroupNode {
    /// 소속 구역 인덱스
    pub section_index: Option<usize>,
    /// 묶음 개체 컨트롤을 소유한 문단 인덱스
    pub para_index: Option<usize>,
    /// 문단 내 컨트롤 인덱스
    pub control_index: Option<usize>,
}

/// 각주/미주 내부에 있는 컨트롤의 원본 위치.
#[derive(Debug, Clone, Serialize)]
pub struct NoteControlRef {
    /// `footnote` 또는 `endnote`
    pub kind: String,
    /// 원본 각주/미주 컨트롤이 있는 구역 인덱스
    pub section_index: usize,
    /// 원본 각주/미주 컨트롤이 있는 본문 문단 인덱스
    pub para_index: usize,
    /// 본문 문단 내 각주/미주 컨트롤 인덱스
    pub control_index: usize,
    /// 각주/미주 내부 문단 인덱스
    pub note_para_index: usize,
    /// 각주/미주 내부 문단 내 컨트롤 인덱스
    pub inner_control_index: usize,
}

/// 수식 노드 (SVG 인라인 렌더링)
#[derive(Debug, Clone, Serialize)]
pub struct EquationNode {
    /// 수식 SVG 조각 (viewBox 기준 상대 좌표)
    pub svg_content: String,
    /// 수식 레이아웃 트리 (Canvas 렌더링용)
    pub layout_box: crate::renderer::equation::layout::LayoutBox,
    /// 수식 색상 문자열 (#rrggbb)
    pub color_str: String,
    /// 수식 글자 색상 (0x00BBGGRR → #RRGGBB)
    pub color: u32,
    /// 수식 글자 크기 (HWPUNIT → px 변환 후)
    pub font_size: f64,
    /// 소속 구역 인덱스
    pub section_index: Option<usize>,
    /// 수식 컨트롤을 소유한 문단 인덱스
    pub para_index: Option<usize>,
    /// 문단 내 컨트롤 인덱스
    pub control_index: Option<usize>,
    /// 표 셀 내 수식인 경우: 셀 인덱스
    pub cell_index: Option<usize>,
    /// 표 셀 내 수식인 경우: 셀 내 문단 인덱스
    pub cell_para_index: Option<usize>,
    /// 각주/미주 내부 수식인 경우 원본 위치
    pub note_ref: Option<NoteControlRef>,
}

/// 인라인 Shape 좌표 맵 키. 섹션 단위 + 셀 경로로 셀 내 paragraph/control
/// 인덱스 충돌(예: 서로 다른 셀이 cp_idx=0 + ctrl_idx=N 동일)을 방지한다.
///
/// `cell_path` 는 외→내 nesting 순서로 `(control_index, cell_index, cell_para_index)`
/// 튜플 목록. 섹션 단위(셀 외부)는 빈 Vec.
pub type InlineShapeKey = (usize, usize, usize, Vec<(usize, usize, usize)>);

/// 한 페이지의 렌더 트리
#[derive(Debug, Clone, Serialize)]
pub struct PageRenderTree {
    /// 루트 노드
    pub root: RenderNode,
    /// 다음 노드 ID 카운터
    #[serde(skip)]
    next_id: NodeId,
    /// 인라인 Shape 좌표 맵: (section, para, control, cell_path) → (x, y)
    #[serde(skip)]
    inline_shape_positions: std::collections::HashMap<InlineShapeKey, (f64, f64)>,
}

impl PageRenderTree {
    /// 새 페이지 렌더 트리 생성
    pub fn new(page_index: u32, width: f64, height: f64) -> Self {
        let root = RenderNode::new(
            0,
            RenderNodeType::Page(PageNode {
                page_index,
                width,
                height,
                section_index: 0,
            }),
            BoundingBox::new(0.0, 0.0, width, height),
        );
        Self {
            root,
            next_id: 1,
            inline_shape_positions: std::collections::HashMap::new(),
        }
    }

    /// `CellContext` 를 InlineShapeKey 의 cell_path 부분으로 변환.
    fn cell_path_from_ctx(
        cell_ctx: Option<&crate::renderer::layout::CellContext>,
    ) -> Vec<(usize, usize, usize)> {
        cell_ctx
            .map(|ctx| {
                ctx.path
                    .iter()
                    .map(|e| (e.control_index, e.cell_index, e.cell_para_index))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 인라인 Shape 좌표 등록 (셀 컨텍스트 포함).
    /// [Task #1151 v4] 셀 안인 경우 InlineShapeKey 의 para 는 호출자가 전달한
    /// cell paragraph idx 가 아닌 **outer paragraph idx** (`cell_ctx.parent_para_index`)
    /// 로 정규화한다. cursor_rect 의 hit-test 가 `section.paragraphs.get(pi)` 로
    /// outer paragraph 에서 table → cell → cell paragraph 경로로 resolve 하기 위해
    /// 정합 필요.
    pub fn set_inline_shape_position(
        &mut self,
        sec: usize,
        para: usize,
        ctrl: usize,
        cell_ctx: Option<&crate::renderer::layout::CellContext>,
        x: f64,
        y: f64,
    ) {
        let cell_path = Self::cell_path_from_ctx(cell_ctx);
        let para_for_key = cell_ctx.map(|c| c.parent_para_index).unwrap_or(para);
        self.inline_shape_positions
            .insert((sec, para_for_key, ctrl, cell_path), (x, y));
    }

    /// 인라인 Shape 좌표 조회 (셀 컨텍스트 포함).
    /// [Task #1151 v4] `set_inline_shape_position` 과 동일한 para 정규화.
    pub fn get_inline_shape_position(
        &self,
        sec: usize,
        para: usize,
        ctrl: usize,
        cell_ctx: Option<&crate::renderer::layout::CellContext>,
    ) -> Option<(f64, f64)> {
        let cell_path = Self::cell_path_from_ctx(cell_ctx);
        let para_for_key = cell_ctx.map(|c| c.parent_para_index).unwrap_or(para);
        self.inline_shape_positions
            .get(&(sec, para_for_key, ctrl, cell_path))
            .copied()
    }

    /// 인라인 Shape 좌표 전체 참조 (hitTest용)
    pub fn inline_shape_positions(&self) -> &std::collections::HashMap<InlineShapeKey, (f64, f64)> {
        &self.inline_shape_positions
    }

    /// 새 노드 ID 할당
    pub fn next_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// dirty 노드 존재 여부
    pub fn needs_render(&self) -> bool {
        self.root.has_dirty_nodes()
    }

    /// 전체 트리를 clean으로 마킹
    pub fn mark_all_clean(&mut self) {
        self.root.mark_clean_recursive();
    }

    /// 동일 `bin_data_id` 를 가진 ImageNode 가 세로로 인접 겹칠 때,
    /// 트리 순서상 먼저 그려지는 (z 가 작은) 쪽의 bbox/crop 을 위에 덮는
    /// (z 가 큰) 쪽의 top 까지 축소한다.
    ///
    /// Task #1154: 동일 이미지를 두 Pic 컨트롤로 겹쳐 박스 효과를 만든 경우
    /// 두 그림의 세로 스케일이 미세하게 달라 안티엘리어싱/리샘플링 시 잔상
    /// (이중 라인) 이 노출되는 문제를 해결한다.
    ///
    /// 적용 조건 (모두 만족 시 clip):
    /// 1. `A.bin_data_id == B.bin_data_id`
    /// 2. `|A.x - B.x| <= 1.0` (수평 위치 동일)
    /// 3. `|A.width - B.width| <= 1.0` (수평 폭 동일)
    /// 4. A 가 트리 순서상 먼저 + `A.y < B.y` (A 가 위에 있음)
    /// 5. `A.y + A.height > B.y` (세로 겹침)
    ///
    /// 조건 2/3 은 의도적 시각 효과 (test-image, 3-10월_교육_통합 의 대각선
    /// 오프셋 등) 를 보호하기 위한 strict 가드.
    pub fn clip_overlapping_same_bin_images(&mut self) {
        // Phase 1: 트리 순서대로 ImageNode 의 (id, bbox, bin_id, crop) 수집
        let mut images: Vec<(NodeId, BoundingBox, u16, Option<(i32, i32, i32, i32)>)> = Vec::new();
        Self::collect_image_nodes(&self.root, &mut images);

        if images.len() < 2 {
            return;
        }

        // Phase 2: 페어 검출 + 각 LOWER 노드별 최종 clip 결정
        // 한 노드가 여러 UPPER 와 겹치면 가장 위쪽 UPPER 를 기준으로 가장 작은 height 적용
        let mut clips: std::collections::HashMap<NodeId, (f64, Option<(i32, i32, i32, i32)>)> =
            std::collections::HashMap::new();

        for i in 0..images.len() {
            for j in (i + 1)..images.len() {
                let (id_a, bbox_a, bin_a, crop_a) = &images[i];
                let (_id_b, bbox_b, bin_b, _crop_b) = &images[j];
                // 조건 1: 같은 bin_id
                if bin_a != bin_b {
                    continue;
                }
                // 조건 2/3: x, width 동일 (1px tolerance)
                if (bbox_a.x - bbox_b.x).abs() > 1.0 {
                    continue;
                }
                if (bbox_a.width - bbox_b.width).abs() > 1.0 {
                    continue;
                }
                // 조건 4: A 가 위 (A.y < B.y)
                if bbox_a.y >= bbox_b.y {
                    continue;
                }
                // 조건 5: 세로 겹침 (A.y + A.height > B.y)
                let a_bottom = bbox_a.y + bbox_a.height;
                if a_bottom <= bbox_b.y {
                    continue;
                }
                // Clip 계산: A 의 new_height = B.y - A.y
                let new_height = bbox_b.y - bbox_a.y;
                if new_height <= 0.0 || new_height >= bbox_a.height {
                    continue;
                }
                let ratio = new_height / bbox_a.height;
                let new_crop = crop_a.map(|(cl, ct, cr, cb)| {
                    let span = (cb - ct) as f64;
                    let new_cb = ct + (span * ratio).round() as i32;
                    (cl, ct, cr, new_cb)
                });
                // 여러 UPPER 와 겹칠 때 가장 작은 new_height 채택
                let entry = clips.entry(*id_a).or_insert((new_height, new_crop));
                if new_height < entry.0 {
                    *entry = (new_height, new_crop);
                }
            }
        }

        if clips.is_empty() {
            return;
        }

        // Phase 3: 트리 walk 하여 mutation 적용
        Self::apply_image_clips(&mut self.root, &clips);
    }

    fn collect_image_nodes(
        node: &RenderNode,
        out: &mut Vec<(NodeId, BoundingBox, u16, Option<(i32, i32, i32, i32)>)>,
    ) {
        if let RenderNodeType::Image(img) = &node.node_type {
            out.push((node.id, node.bbox, img.bin_data_id, img.crop));
        }
        for child in &node.children {
            Self::collect_image_nodes(child, out);
        }
    }

    fn apply_image_clips(
        node: &mut RenderNode,
        clips: &std::collections::HashMap<NodeId, (f64, Option<(i32, i32, i32, i32)>)>,
    ) {
        if let Some((new_height, new_crop)) = clips.get(&node.id) {
            node.bbox.height = *new_height;
            if let RenderNodeType::Image(ref mut img) = &mut node.node_type {
                if let Some(c) = new_crop {
                    img.crop = Some(*c);
                }
            }
        }
        for child in &mut node.children {
            Self::apply_image_clips(child, clips);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_intersects() {
        let a = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
        let b = BoundingBox::new(50.0, 50.0, 100.0, 100.0);
        let c = BoundingBox::new(200.0, 200.0, 50.0, 50.0);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_bounding_box_contains() {
        let outer = BoundingBox::new(0.0, 0.0, 200.0, 200.0);
        let inner = BoundingBox::new(10.0, 10.0, 50.0, 50.0);
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_page_render_tree() {
        let mut tree = PageRenderTree::new(0, 793.7, 1122.5);
        assert!(tree.needs_render());
        assert_eq!(tree.next_id(), 1);
        assert_eq!(tree.next_id(), 2);
        tree.mark_all_clean();
        assert!(!tree.needs_render());
    }

    #[test]
    fn test_render_node_dirty_flag() {
        let mut node = RenderNode::new(
            0,
            RenderNodeType::Body { clip_rect: None },
            BoundingBox::new(0.0, 0.0, 100.0, 100.0),
        );
        assert!(node.dirty);
        node.mark_clean();
        assert!(!node.dirty);
        node.invalidate();
        assert!(node.dirty);
    }

    #[test]
    fn test_bounding_box_from_hwpunit() {
        use crate::model::Rect;
        let rect = Rect {
            left: 0,
            top: 0,
            right: 7200,
            bottom: 7200,
        };
        let bbox = BoundingBox::from_hwpunit_rect(&rect, 96.0);
        assert!((bbox.width - 96.0).abs() < 0.01);
        assert!((bbox.height - 96.0).abs() < 0.01);
    }

    // === Task #1154: clip_overlapping_same_bin_images 단위 테스트 ===

    fn make_image_node(
        id: NodeId,
        bbox: BoundingBox,
        bin_id: u16,
        crop: Option<(i32, i32, i32, i32)>,
    ) -> RenderNode {
        let img = ImageNode {
            crop,
            ..ImageNode::new(bin_id, None)
        };
        RenderNode::new(id, RenderNodeType::Image(img), bbox)
    }

    fn image_bbox(node: &RenderNode) -> BoundingBox {
        node.bbox
    }

    fn image_crop(node: &RenderNode) -> Option<(i32, i32, i32, i32)> {
        match &node.node_type {
            RenderNodeType::Image(img) => img.crop,
            _ => None,
        }
    }

    /// exam_eng.hwp 페이지 2 의 정확한 케이스 (수평 동일 + 세로 인접)
    /// → LOWER 의 bbox.height 와 crop.bottom 이 UPPER 의 top 까지 축소되어야 한다.
    #[test]
    fn test_clip_exam_eng_pattern() {
        let mut tree = PageRenderTree::new(0, 1122.5, 1587.4);
        let lower = make_image_node(
            tree.next_id(),
            BoundingBox::new(597.15, 243.59, 408.19, 256.09),
            5,
            Some((0, 0, 189900, 120958)),
        );
        let upper = make_image_node(
            tree.next_id(),
            BoundingBox::new(597.15, 463.17, 408.19, 70.0),
            5,
            Some((0, 105958, 189900, 138540)),
        );
        let lower_id = lower.id;
        let upper_id = upper.id;
        tree.root.children.push(lower);
        tree.root.children.push(upper);

        tree.clip_overlapping_same_bin_images();

        let l = &tree.root.children[0];
        assert_eq!(l.id, lower_id);
        // 새 height = 463.17 - 243.59 = 219.58
        let bbox = image_bbox(l);
        assert!(
            (bbox.height - 219.58).abs() < 0.01,
            "lower height={}",
            bbox.height
        );
        // 새 crop.bottom = 0 + 120958 * (219.58/256.09) = 103716 (round)
        let crop = image_crop(l).unwrap();
        assert_eq!(crop.0, 0);
        assert_eq!(crop.1, 0);
        assert_eq!(crop.2, 189900);
        let expected_cb = (120958_f64 * (219.58 / 256.09)).round() as i32;
        assert_eq!(crop.3, expected_cb);

        // UPPER 은 변경되지 않음
        let u = &tree.root.children[1];
        assert_eq!(u.id, upper_id);
        let u_bbox = image_bbox(u);
        assert!((u_bbox.height - 70.0).abs() < 0.01);
        let u_crop = image_crop(u).unwrap();
        assert_eq!(u_crop, (0, 105958, 189900, 138540));
    }

    /// 다른 bin_id 페어는 무시되어야 한다.
    #[test]
    fn test_clip_different_bin_id_no_clip() {
        let mut tree = PageRenderTree::new(0, 1122.5, 1587.4);
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 100.0, 200.0, 200.0),
            1,
            Some((0, 0, 100, 100)),
        ));
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 250.0, 200.0, 100.0),
            2, // 다른 bin
            Some((0, 0, 100, 100)),
        ));

        tree.clip_overlapping_same_bin_images();

        // 둘 다 원래 크기 유지
        assert_eq!(image_bbox(&tree.root.children[0]).height, 200.0);
        assert_eq!(image_bbox(&tree.root.children[1]).height, 100.0);
    }

    /// 세로 겹침이 없으면 무시되어야 한다.
    #[test]
    fn test_clip_no_vertical_overlap_no_clip() {
        let mut tree = PageRenderTree::new(0, 1122.5, 1587.4);
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 100.0, 200.0, 100.0), // 100-200
            1,
            Some((0, 0, 100, 100)),
        ));
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 250.0, 200.0, 100.0), // 250-350 (gap)
            1,
            Some((0, 0, 100, 100)),
        ));

        tree.clip_overlapping_same_bin_images();

        assert_eq!(image_bbox(&tree.root.children[0]).height, 100.0);
        assert_eq!(image_bbox(&tree.root.children[1]).height, 100.0);
    }

    /// x 가 다르면 (의도적 대각선 오프셋) 무시되어야 한다.
    /// 회귀 보호: test-image.hwp / 3-10월_교육_통합_2022 패턴
    #[test]
    fn test_clip_different_x_no_clip() {
        let mut tree = PageRenderTree::new(0, 1122.5, 1587.4);
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 100.0, 200.0, 200.0),
            1,
            Some((0, 0, 100, 100)),
        ));
        // x 가 8px 다른 (의도적 그림자 효과 가정)
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(108.0, 150.0, 200.0, 200.0),
            1,
            Some((0, 0, 100, 100)),
        ));

        tree.clip_overlapping_same_bin_images();

        assert_eq!(image_bbox(&tree.root.children[0]).height, 200.0);
        assert_eq!(image_bbox(&tree.root.children[1]).height, 200.0);
    }

    /// width 가 다르면 (서로 다른 크기 그림) 무시되어야 한다.
    /// 회귀 보호: pic2.hwp 패턴 (같은 이미지를 다른 크기로 배치)
    #[test]
    fn test_clip_different_width_no_clip() {
        let mut tree = PageRenderTree::new(0, 1122.5, 1587.4);
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 100.0, 200.0, 200.0),
            1,
            Some((0, 0, 100, 100)),
        ));
        // width 가 100px 다른 (다른 크기 그림)
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 150.0, 100.0, 200.0),
            1,
            Some((0, 0, 100, 100)),
        ));

        tree.clip_overlapping_same_bin_images();

        assert_eq!(image_bbox(&tree.root.children[0]).height, 200.0);
        assert_eq!(image_bbox(&tree.root.children[1]).height, 200.0);
    }

    /// 트리 순서가 반대 (A.y > B.y) 일 때는 무시되어야 한다.
    /// (clip 대상은 항상 위에 깔리는 = 트리 순서상 먼저 + y 작은 쪽)
    #[test]
    fn test_clip_reversed_order_no_clip() {
        let mut tree = PageRenderTree::new(0, 1122.5, 1587.4);
        // 먼저 그려지는 노드가 아래쪽 (y 큰) — clip 안 함
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 250.0, 200.0, 100.0),
            1,
            Some((0, 0, 100, 100)),
        ));
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 100.0, 200.0, 200.0),
            1,
            Some((0, 0, 100, 100)),
        ));

        tree.clip_overlapping_same_bin_images();

        // 둘 다 원래 크기 유지
        assert_eq!(image_bbox(&tree.root.children[0]).height, 100.0);
        assert_eq!(image_bbox(&tree.root.children[1]).height, 200.0);
    }

    /// crop=None 인 경우에도 bbox 만 축소 적용.
    #[test]
    fn test_clip_without_crop() {
        let mut tree = PageRenderTree::new(0, 1122.5, 1587.4);
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 100.0, 200.0, 200.0),
            1,
            None,
        ));
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 250.0, 200.0, 100.0),
            1,
            None,
        ));

        tree.clip_overlapping_same_bin_images();

        // LOWER 의 height 가 250-100 = 150 으로 축소
        assert!((image_bbox(&tree.root.children[0]).height - 150.0).abs() < 0.01);
        assert!(image_crop(&tree.root.children[0]).is_none());
        // UPPER 은 변경 없음
        assert!((image_bbox(&tree.root.children[1]).height - 100.0).abs() < 0.01);
    }

    /// 3 개 이미지 중첩: 가장 위 → 중간 → 아래 (A < B < C)
    /// A 는 B 의 top 까지, B 는 C 의 top 까지 축소.
    #[test]
    fn test_clip_three_overlapping_chain() {
        let mut tree = PageRenderTree::new(0, 1122.5, 1587.4);
        // A: y=0 height=200 (0-200)
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 0.0, 200.0, 200.0),
            1,
            Some((0, 0, 1000, 2000)),
        ));
        // B: y=100 height=150 (100-250) — overlaps A
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 100.0, 200.0, 150.0),
            1,
            Some((0, 1000, 1000, 2500)),
        ));
        // C: y=180 height=100 (180-280) — overlaps both A and B
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 180.0, 200.0, 100.0),
            1,
            Some((0, 1800, 1000, 2800)),
        ));

        tree.clip_overlapping_same_bin_images();

        // A: 가장 가까운 upper(B) 의 top=100 까지 → height=100
        assert!((image_bbox(&tree.root.children[0]).height - 100.0).abs() < 0.01);
        // B: 가장 가까운 upper(C) 의 top=180 → height=80
        assert!((image_bbox(&tree.root.children[1]).height - 80.0).abs() < 0.01);
        // C: 변경 없음
        assert!((image_bbox(&tree.root.children[2]).height - 100.0).abs() < 0.01);
    }

    /// 자식 노드 (e.g., Body > Column > Image) 깊이에서도 동작.
    #[test]
    fn test_clip_nested_children() {
        let mut tree = PageRenderTree::new(0, 1122.5, 1587.4);
        let mut body = RenderNode::new(
            tree.next_id(),
            RenderNodeType::Body { clip_rect: None },
            BoundingBox::new(0.0, 0.0, 1000.0, 1500.0),
        );
        let _id = tree.next_id();
        body.children.push(make_image_node(
            _id,
            BoundingBox::new(597.15, 243.59, 408.19, 256.09),
            5,
            Some((0, 0, 189900, 120958)),
        ));
        let _id = tree.next_id();
        body.children.push(make_image_node(
            _id,
            BoundingBox::new(597.15, 463.17, 408.19, 70.0),
            5,
            Some((0, 105958, 189900, 138540)),
        ));
        tree.root.children.push(body);

        tree.clip_overlapping_same_bin_images();

        let body = &tree.root.children[0];
        let lower = &body.children[0];
        assert!((image_bbox(lower).height - 219.58).abs() < 0.01);
    }

    /// 단일 이미지만 있는 경우 변경 없음 (no-op).
    #[test]
    fn test_clip_single_image_no_op() {
        let mut tree = PageRenderTree::new(0, 1122.5, 1587.4);
        let _id = tree.next_id();
        tree.root.children.push(make_image_node(
            _id,
            BoundingBox::new(100.0, 100.0, 200.0, 200.0),
            1,
            Some((0, 0, 100, 100)),
        ));

        tree.clip_overlapping_same_bin_images();

        assert_eq!(image_bbox(&tree.root.children[0]).height, 200.0);
        assert_eq!(image_crop(&tree.root.children[0]), Some((0, 0, 100, 100)));
    }
}
