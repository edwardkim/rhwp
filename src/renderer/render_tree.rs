//! 렌더 트리 노드 (RenderNode, Box Model)
//!
//! IR(Document Model)로부터 변환된 렌더링 전용 트리 구조.
//! 각 노드는 페이지 내 위치와 크기가 계산된 상태를 가진다.

use crate::model::{ColorRef, Rect};
use crate::model::style::ImageFillMode;
use crate::model::image::ImageEffect;
use super::{TextStyle, ShapeStyle, LineStyle, PathCommand, GradientFillInfo};
use super::composer::CharOverlapInfo;
use super::layout::CellContext;

/// 렌더 노드 고유 ID
pub type NodeId = u32;

/// 렌더 노드 (페이지 내 렌더링 가능한 요소)
#[derive(Debug, Clone)]
pub struct RenderNode {
    /// 노드 ID
    pub id: NodeId,
    /// 노드 종류
    pub node_type: RenderNodeType,
    /// 박스 모델 (위치, 크기, 여백)
    pub bbox: BoundingBox,
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
            children: Vec::new(),
            dirty: true,
            visible: true,
        }
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
            RenderNodeType::TextLine(tl) => ("TextLine", format!(
                ",\"pi\":{}", tl.para_index.unwrap_or(0))),
            RenderNodeType::TextRun(tr) => ("TextRun", format!(
                ",\"text\":{},\"pi\":{}", json_escape(&tr.text),
                tr.section_index.map(|_| tr.para_index.unwrap_or(0)).unwrap_or(0))),
            RenderNodeType::Table(tn) => ("Table", format!(
                ",\"rows\":{},\"cols\":{}{}{}", tn.row_count, tn.col_count,
                tn.para_index.map(|pi| format!(",\"pi\":{}", pi)).unwrap_or_default(),
                tn.control_index.map(|ci| format!(",\"ci\":{}", ci)).unwrap_or_default())),
            RenderNodeType::TableCell(tc) => ("Cell", format!(
                ",\"row\":{},\"col\":{}", tc.row, tc.col)),
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
            RenderNodeType::Placeholder(_) => ("Placeholder", String::new()),
            RenderNodeType::RawSvg(_) => ("RawSvg", String::new()),
        };
        buf.push_str(&format!("\"type\":\"{}\",\"bbox\":{{\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}}}",
            type_str, self.bbox.x, self.bbox.y, self.bbox.width, self.bbox.height));
        buf.push_str(&extra);
        if !self.children.is_empty() {
            buf.push_str(",\"children\":[");
            for (i, child) in self.children.iter().enumerate() {
                if i > 0 { buf.push(','); }
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
#[derive(Debug, Clone)]
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

/// 미리 렌더된 SVG 조각 (Task #195 단계 8)
#[derive(Debug, Clone)]
pub struct RawSvgNode {
    /// 삽입할 SVG 조각 (유효한 `<g>...</g>` 또는 개별 요소)
    pub svg: String,
}

/// 차트/OLE placeholder 렌더 노드 (Task #195)
#[derive(Debug, Clone)]
pub struct PlaceholderNode {
    /// 배경 색상 (ARGB)
    pub fill_color: u32,
    /// 테두리 색상 (ARGB)
    pub stroke_color: u32,
    /// 표시할 라벨(중앙 정렬)
    pub label: String,
}

/// 각주/미주 마커 렌더 노드
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Copy, Default)]
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
        Self { x, y, width, height }
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct PageBackgroundImage {
    /// 이미지 데이터
    pub data: Vec<u8>,
    /// 이미지 채우기 모드
    pub fill_mode: super::super::model::style::ImageFillMode,
}

/// 텍스트 줄 노드
#[derive(Debug, Clone)]
pub struct TextLineNode {
    /// 줄 높이 (px)
    pub line_height: f64,
    /// 베이스라인 위치 (줄 상단으로부터, px)
    pub baseline: f64,
    /// 소속 구역 인덱스 (빈 문단 커서 위치 계산용)
    pub section_index: Option<usize>,
    /// 소속 문단 인덱스 (빈 문단 커서 위치 계산용)
    pub para_index: Option<usize>,
}

impl TextLineNode {
    /// 기본 생성 (문단 식별 정보 없음)
    pub fn new(line_height: f64, baseline: f64) -> Self {
        Self { line_height, baseline, section_index: None, para_index: None }
    }

    /// 문단 식별 정보 포함 생성 (커서 위치 계산용)
    pub fn with_para(line_height: f64, baseline: f64, section_index: usize, para_index: usize) -> Self {
        Self { line_height, baseline, section_index: Some(section_index), para_index: Some(para_index) }
    }
}

/// 텍스트 런 노드 (동일 글자 모양의 연속 텍스트)
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Copy, PartialEq, Default)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, Copy, Default)]
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
}

/// 직선 노드
#[derive(Debug, Clone)]
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
}

impl LineNode {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, style: LineStyle) -> Self {
        Self { x1, y1, x2, y2, style,
            section_index: None, para_index: None, control_index: None,
            transform: ShapeTransform::default() }
    }
}

/// 사각형 노드
#[derive(Debug, Clone)]
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
}

impl RectangleNode {
    pub fn new(corner_radius: f64, style: ShapeStyle, gradient: Option<Box<GradientFillInfo>>) -> Self {
        Self {
            corner_radius, style, gradient,
            section_index: None, para_index: None, control_index: None,
            transform: ShapeTransform::default(),
        }
    }
}

/// 타원 노드
#[derive(Debug, Clone)]
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
}

impl EllipseNode {
    pub fn new(style: ShapeStyle, gradient: Option<Box<GradientFillInfo>>) -> Self {
        Self { style, gradient,
            section_index: None, para_index: None, control_index: None,
            transform: ShapeTransform::default() }
    }
}

/// 패스 노드
#[derive(Debug, Clone)]
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
}

impl PathNode {
    pub fn new(commands: Vec<PathCommand>, style: ShapeStyle, gradient: Option<Box<GradientFillInfo>>) -> Self {
        Self { commands, style, gradient,
            section_index: None, para_index: None, control_index: None,
            transform: ShapeTransform::default(),
            connector_endpoints: None,
            line_style: None }
    }
}

/// 이미지 노드
#[derive(Debug, Clone)]
pub struct ImageNode {
    /// BinData ID 참조
    pub bin_data_id: u16,
    /// 이미지 데이터 (캐시용)
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
    /// 그림 효과 (실사/그레이스케일/흑백/패턴)
    pub effect: ImageEffect,
}

impl ImageNode {
    pub fn new(bin_data_id: u16, data: Option<Vec<u8>>) -> Self {
        Self {
            bin_data_id, data,
            section_index: None, para_index: None, control_index: None,
            fill_mode: None, original_size: None,
            transform: ShapeTransform::default(),
            crop: None,
            effect: ImageEffect::RealPic,
        }
    }
}

/// 묶음 개체 노드
#[derive(Debug, Clone)]
pub struct GroupNode {
    /// 소속 구역 인덱스
    pub section_index: Option<usize>,
    /// 묶음 개체 컨트롤을 소유한 문단 인덱스
    pub para_index: Option<usize>,
    /// 문단 내 컨트롤 인덱스
    pub control_index: Option<usize>,
}

/// 수식 노드 (SVG 인라인 렌더링)
#[derive(Debug, Clone)]
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
}

/// 한 페이지의 렌더 트리
#[derive(Debug, Clone)]
pub struct PageRenderTree {
    /// 루트 노드
    pub root: RenderNode,
    /// 다음 노드 ID 카운터
    next_id: NodeId,
    /// 인라인 Shape 좌표 맵: (section, para, control) → (x, y)
    inline_shape_positions: std::collections::HashMap<(usize, usize, usize), (f64, f64)>,
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
        Self { root, next_id: 1, inline_shape_positions: std::collections::HashMap::new() }
    }

    /// 인라인 Shape 좌표 등록
    pub fn set_inline_shape_position(&mut self, sec: usize, para: usize, ctrl: usize, x: f64, y: f64) {
        self.inline_shape_positions.insert((sec, para, ctrl), (x, y));
    }

    /// 인라인 Shape 좌표 조회
    pub fn get_inline_shape_position(&self, sec: usize, para: usize, ctrl: usize) -> Option<(f64, f64)> {
        self.inline_shape_positions.get(&(sec, para, ctrl)).copied()
    }

    /// 인라인 Shape 좌표 전체 참조 (hitTest용)
    pub fn inline_shape_positions(&self) -> &std::collections::HashMap<(usize, usize, usize), (f64, f64)> {
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
        let rect = Rect { left: 0, top: 0, right: 7200, bottom: 7200 };
        let bbox = BoundingBox::from_hwpunit_rect(&rect, 96.0);
        assert!((bbox.width - 96.0).abs() < 0.01);
        assert!((bbox.height - 96.0).abs() < 0.01);
    }
}
