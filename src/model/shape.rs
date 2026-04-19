//! 그리기 개체 (Shape, Line, Rect, Ellipse, Arc, Polygon, Curve, Group, TextBox)

use super::*;
use super::paragraph::Paragraph;
use super::style::{Fill, ShapeBorderLine};

/// 개체 공통 속성 (모든 개체에 공통)
#[derive(Debug, Clone, Default)]
pub struct CommonObjAttr {
    /// 컨트롤 ID
    pub ctrl_id: u32,
    /// 속성 비트 플래그
    pub attr: u32,
    /// 세로 오프셋
    pub vertical_offset: HwpUnit,
    /// 가로 오프셋
    pub horizontal_offset: HwpUnit,
    /// 폭
    pub width: HwpUnit,
    /// 높이
    pub height: HwpUnit,
    /// Z-order
    pub z_order: i32,
    /// 바깥 여백 (좌, 우, 상, 하)
    pub margin: Padding,
    /// 인스턴스 ID
    pub instance_id: u32,
    /// 쪽나눔 방지 (0=off, 1=on)
    pub prevent_page_break: i32,
    /// 글자처럼 취급
    pub treat_as_char: bool,
    /// 세로 위치 기준
    pub vert_rel_to: VertRelTo,
    /// 세로 정렬 방식
    pub vert_align: VertAlign,
    /// 가로 위치 기준
    pub horz_rel_to: HorzRelTo,
    /// 가로 정렬 방식
    pub horz_align: HorzAlign,
    /// 텍스트 흐름 방식
    pub text_wrap: TextWrap,
    /// 너비 기준 (bit 15-17): 0=Paper, 1=Page, 2=Column, 3=Para, 4=Absolute
    pub width_criterion: SizeCriterion,
    /// 높이 기준 (bit 18-19): 0=Paper, 1=Page, 2=Absolute
    pub height_criterion: SizeCriterion,
    /// 개체 설명문
    pub description: String,
    /// 파싱된 필드 이후 추가 바이트 (라운드트립 보존용)
    pub raw_extra: Vec<u8>,
}

/// 세로 위치 기준
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum VertRelTo {
    #[default]
    Paper,
    Page,
    Para,
}

/// 세로 정렬 방식
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum VertAlign {
    #[default]
    Top,
    Center,
    Bottom,
    Inside,
    Outside,
}

/// 가로 위치 기준
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum HorzRelTo {
    #[default]
    Paper,
    Page,
    Column,
    Para,
}

/// 가로 정렬 방식
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum HorzAlign {
    #[default]
    Left,
    Center,
    Right,
    Inside,
    Outside,
}

/// 크기 기준 (너비/높이)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SizeCriterion {
    /// 종이 기준 (퍼센트)
    Paper,
    /// 쪽 기준 (퍼센트)
    Page,
    /// 단 기준 (퍼센트, 너비 전용)
    Column,
    /// 문단 기준 (퍼센트, 너비 전용)
    Para,
    /// 절대값 (HWPUNIT)
    #[default]
    Absolute,
}

/// 텍스트 흐름 방식
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TextWrap {
    #[default]
    Square,
    Tight,
    Through,
    TopAndBottom,
    BehindText,
    InFrontOfText,
}

/// 개체 요소 속성 (그리기 개체 공통)
#[derive(Debug, Clone)]
pub struct ShapeComponentAttr {
    /// SHAPE_COMPONENT 내 ctrl_id (라운드트립 보존용, 0이면 기본값 사용)
    pub ctrl_id: u32,
    /// ctrl_id가 2번 기록되는지 여부
    pub is_two_ctrl_id: bool,
    /// 그룹 내 X 오프셋
    pub offset_x: i32,
    /// 그룹 내 Y 오프셋
    pub offset_y: i32,
    /// 그룹 횟수
    pub group_level: u16,
    /// 로컬 파일 버전 (라운드트립 보존용)
    pub local_file_version: u16,
    /// 초기 폭
    pub original_width: u32,
    /// 초기 높이
    pub original_height: u32,
    /// 현재 폭
    pub current_width: u32,
    /// 현재 높이
    pub current_height: u32,
    /// 뒤집기 속성 원본 값 (bit 0: 수평, bit 1: 수직, 상위 비트 보존)
    pub flip: u32,
    /// 수평 뒤집기
    pub horz_flip: bool,
    /// 수직 뒤집기
    pub vert_flip: bool,
    /// 회전각
    pub rotation_angle: HwpUnit16,
    /// 회전 중심 좌표
    pub rotation_center: Point,
    /// 렌더링 정보 원본 바이트 (변환 행렬 등, 라운드트립 보존용)
    pub raw_rendering: Vec<u8>,
    /// 렌더링 변환 합성 결과: 아핀 행렬 [a, b, tx, c, d, ty]
    /// (x', y') = (a*x + b*y + tx, c*x + d*y + ty)
    /// render_sx = a, render_b = b, render_tx = tx
    /// render_c = c, render_sy = d, render_ty = ty
    pub render_tx: f64,
    pub render_ty: f64,
    pub render_sx: f64,
    pub render_sy: f64,
    /// 아핀 행렬 비대각 요소 (회전/전단 성분)
    pub render_b: f64,
    pub render_c: f64,
}

impl Default for ShapeComponentAttr {
    fn default() -> Self {
        Self {
            ctrl_id: 0,
            is_two_ctrl_id: false,
            offset_x: 0,
            offset_y: 0,
            group_level: 0,
            local_file_version: 0,
            original_width: 0,
            original_height: 0,
            current_width: 0,
            current_height: 0,
            flip: 0,
            horz_flip: false,
            vert_flip: false,
            rotation_angle: 0,
            rotation_center: Point::default(),
            raw_rendering: Vec::new(),
            render_tx: 0.0,
            render_ty: 0.0,
            render_sx: 1.0,
            render_sy: 1.0,
            render_b: 0.0,
            render_c: 0.0,
        }
    }
}

/// 그리기 개체 공통 속성
#[derive(Debug, Default, Clone)]
pub struct DrawingObjAttr {
    /// 개체 요소 속성
    pub shape_attr: ShapeComponentAttr,
    /// 테두리 선 정보
    pub border_line: ShapeBorderLine,
    /// 채우기 정보
    pub fill: Fill,
    /// 그림자 종류 (0=없음, 1=왼쪽위, 2=오른쪽위, 3=왼쪽아래, 4=오른쪽아래, 5=뒤쪽등)
    pub shadow_type: u32,
    /// 그림자 색상
    pub shadow_color: u32,
    /// 그림자 가로 오프셋
    pub shadow_offset_x: i32,
    /// 그림자 세로 오프셋
    pub shadow_offset_y: i32,
    /// 인스턴스 ID (instid)
    pub inst_id: u32,
    /// 그림자 투명도
    pub shadow_alpha: u8,
    /// 글상자 (텍스트가 있는 경우)
    pub text_box: Option<TextBox>,
    /// 캡션
    pub caption: Option<Caption>,
}

/// 글상자 (그리기 개체 내 텍스트)
#[derive(Debug, Default, Clone)]
pub struct TextBox {
    /// LIST_HEADER list_attr (라운드트립 보존용)
    pub list_attr: u32,
    /// 세로 정렬 (list_attr bit 5~6: 0=top, 1=center, 2=bottom)
    pub vertical_align: crate::model::table::VerticalAlign,
    /// 왼쪽 여백
    pub margin_left: HwpUnit16,
    /// 오른쪽 여백
    pub margin_right: HwpUnit16,
    /// 위쪽 여백
    pub margin_top: HwpUnit16,
    /// 아래쪽 여백
    pub margin_bottom: HwpUnit16,
    /// 텍스트 최대 폭
    pub max_width: HwpUnit,
    /// LIST_HEADER 레코드의 파싱된 필드 이후 추가 바이트 (라운드트립 보존용)
    pub raw_list_header_extra: Vec<u8>,
    /// 문단 리스트
    pub paragraphs: Vec<Paragraph>,
}

/// 그리기 개체 종류
#[derive(Debug, Clone)]
pub enum ShapeObject {
    /// 직선
    Line(LineShape),
    /// 사각형
    Rectangle(RectangleShape),
    /// 타원
    Ellipse(EllipseShape),
    /// 호
    Arc(ArcShape),
    /// 다각형
    Polygon(PolygonShape),
    /// 곡선
    Curve(CurveShape),
    /// 묶음 개체
    Group(GroupShape),
    /// 그림 개체 (묶음 내 자식으로 포함될 때)
    Picture(Box<crate::model::image::Picture>),
    /// 차트 개체 (GSO + HWPTAG_CHART_DATA)
    Chart(Box<ChartShape>),
    /// OLE 개체 (HWPTAG_SHAPE_COMPONENT_OLE)
    Ole(Box<OleShape>),
}

impl ShapeObject {
    /// 공통 속성 참조 반환
    pub fn common(&self) -> &CommonObjAttr {
        match self {
            ShapeObject::Line(s) => &s.common,
            ShapeObject::Rectangle(s) => &s.common,
            ShapeObject::Ellipse(s) => &s.common,
            ShapeObject::Arc(s) => &s.common,
            ShapeObject::Polygon(s) => &s.common,
            ShapeObject::Curve(s) => &s.common,
            ShapeObject::Group(g) => &g.common,
            ShapeObject::Picture(p) => &p.common,
            ShapeObject::Chart(c) => &c.common,
            ShapeObject::Ole(o) => &o.common,
        }
    }

    /// 공통 속성 가변 참조 반환
    pub fn common_mut(&mut self) -> &mut CommonObjAttr {
        match self {
            ShapeObject::Line(s) => &mut s.common,
            ShapeObject::Rectangle(s) => &mut s.common,
            ShapeObject::Ellipse(s) => &mut s.common,
            ShapeObject::Arc(s) => &mut s.common,
            ShapeObject::Polygon(s) => &mut s.common,
            ShapeObject::Curve(s) => &mut s.common,
            ShapeObject::Group(g) => &mut g.common,
            ShapeObject::Picture(p) => &mut p.common,
            ShapeObject::Chart(c) => &mut c.common,
            ShapeObject::Ole(o) => &mut o.common,
        }
    }

    /// 그리기 공통 속성 참조 반환 (Group/Picture/Ole 내용 없는 종류는 None)
    pub fn drawing(&self) -> Option<&DrawingObjAttr> {
        match self {
            ShapeObject::Line(s) => Some(&s.drawing),
            ShapeObject::Rectangle(s) => Some(&s.drawing),
            ShapeObject::Ellipse(s) => Some(&s.drawing),
            ShapeObject::Arc(s) => Some(&s.drawing),
            ShapeObject::Polygon(s) => Some(&s.drawing),
            ShapeObject::Curve(s) => Some(&s.drawing),
            ShapeObject::Chart(c) => Some(&c.drawing),
            ShapeObject::Ole(o) => Some(&o.drawing),
            ShapeObject::Group(_) | ShapeObject::Picture(_) => None,
        }
    }

    /// 그리기 공통 속성 가변 참조 반환 (Group/Picture는 None)
    pub fn drawing_mut(&mut self) -> Option<&mut DrawingObjAttr> {
        match self {
            ShapeObject::Line(s) => Some(&mut s.drawing),
            ShapeObject::Rectangle(s) => Some(&mut s.drawing),
            ShapeObject::Ellipse(s) => Some(&mut s.drawing),
            ShapeObject::Arc(s) => Some(&mut s.drawing),
            ShapeObject::Polygon(s) => Some(&mut s.drawing),
            ShapeObject::Curve(s) => Some(&mut s.drawing),
            ShapeObject::Chart(c) => Some(&mut c.drawing),
            ShapeObject::Ole(o) => Some(&mut o.drawing),
            ShapeObject::Group(_) | ShapeObject::Picture(_) => None,
        }
    }

    /// Z-order 값 반환
    pub fn z_order(&self) -> i32 {
        self.common().z_order
    }

    /// 개체 요소 속성 참조 반환
    pub fn shape_attr(&self) -> &ShapeComponentAttr {
        match self {
            ShapeObject::Line(s) => &s.drawing.shape_attr,
            ShapeObject::Rectangle(s) => &s.drawing.shape_attr,
            ShapeObject::Ellipse(s) => &s.drawing.shape_attr,
            ShapeObject::Arc(s) => &s.drawing.shape_attr,
            ShapeObject::Polygon(s) => &s.drawing.shape_attr,
            ShapeObject::Curve(s) => &s.drawing.shape_attr,
            ShapeObject::Group(g) => &g.shape_attr,
            ShapeObject::Picture(p) => &p.shape_attr,
            ShapeObject::Chart(c) => &c.drawing.shape_attr,
            ShapeObject::Ole(o) => &o.drawing.shape_attr,
        }
    }

    /// 개체 타입명 반환
    pub fn shape_name(&self) -> &'static str {
        match self {
            ShapeObject::Line(_) => "직선",
            ShapeObject::Rectangle(_) => "사각형",
            ShapeObject::Ellipse(_) => "타원",
            ShapeObject::Arc(_) => "호",
            ShapeObject::Polygon(_) => "다각형",
            ShapeObject::Curve(_) => "곡선",
            ShapeObject::Group(_) => "묶음",
            ShapeObject::Picture(_) => "그림(묶음내)",
            ShapeObject::Chart(_) => "차트",
            ShapeObject::Ole(_) => "OLE",
        }
    }
}

/// 직선 개체 (HWPTAG_SHAPE_COMPONENT_LINE)
#[derive(Debug, Default, Clone)]
pub struct LineShape {
    /// 공통 속성
    pub common: CommonObjAttr,
    /// 그리기 공통 속성
    pub drawing: DrawingObjAttr,
    /// 시작점
    pub start: Point,
    /// 끝점
    pub end: Point,
    /// 오른쪽/아래에서 시작했는지 여부 (hwplib: startedRightOrBottom)
    pub started_right_or_bottom: bool,
    /// 연결선 데이터 (ctrl_id='$col'일 때만 Some)
    pub connector: Option<ConnectorData>,
}

/// 연결선 타입 (9종)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(u32)]
pub enum LinkLineType {
    #[default]
    StraightNoArrow = 0,
    StraightOneWay = 1,
    StraightBoth = 2,
    StrokeNoArrow = 3,
    StrokeOneWay = 4,
    StrokeBoth = 5,
    ArcNoArrow = 6,
    ArcOneWay = 7,
    ArcBoth = 8,
}

impl LinkLineType {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::StraightNoArrow,
            1 => Self::StraightOneWay,
            2 => Self::StraightBoth,
            3 => Self::StrokeNoArrow,
            4 => Self::StrokeOneWay,
            5 => Self::StrokeBoth,
            6 => Self::ArcNoArrow,
            7 => Self::ArcOneWay,
            8 => Self::ArcBoth,
            _ => Self::StraightNoArrow,
        }
    }

    /// 꺽인 연결선인지
    pub fn is_stroke(&self) -> bool {
        matches!(self, Self::StrokeNoArrow | Self::StrokeOneWay | Self::StrokeBoth)
    }

    /// 곡선 연결선인지
    pub fn is_arc(&self) -> bool {
        matches!(self, Self::ArcNoArrow | Self::ArcOneWay | Self::ArcBoth)
    }
}

/// 연결선 제어점
#[derive(Debug, Clone, Default)]
pub struct ConnectorControlPoint {
    pub x: i32,
    pub y: i32,
    pub point_type: u16,
}

/// 연결선 추가 데이터 (SC_LINE 확장)
#[derive(Debug, Clone, Default)]
pub struct ConnectorData {
    /// 연결선 타입
    pub link_type: LinkLineType,
    /// 시작 연결 개체 instance_id
    pub start_subject_id: u32,
    /// 시작 연결점 인덱스
    pub start_subject_index: u32,
    /// 끝 연결 개체 instance_id
    pub end_subject_id: u32,
    /// 끝 연결점 인덱스
    pub end_subject_index: u32,
    /// 제어점 목록 (꺽인/곡선용)
    pub control_points: Vec<ConnectorControlPoint>,
    /// SC_LINE 끝 패딩/추가 바이트 (라운드트립 보존)
    pub raw_trailing: Vec<u8>,
}

/// 사각형 개체 (HWPTAG_SHAPE_COMPONENT_RECTANGLE)
#[derive(Debug, Default, Clone)]
pub struct RectangleShape {
    /// 공통 속성
    pub common: CommonObjAttr,
    /// 그리기 공통 속성
    pub drawing: DrawingObjAttr,
    /// 모서리 곡률 (%)
    pub round_rate: u8,
    /// 꼭짓점 X 좌표 (4개)
    pub x_coords: [i32; 4],
    /// 꼭짓점 Y 좌표 (4개)
    pub y_coords: [i32; 4],
}

/// 타원 개체 (HWPTAG_SHAPE_COMPONENT_ELLIPSE)
#[derive(Debug, Default, Clone)]
pub struct EllipseShape {
    /// 공통 속성
    pub common: CommonObjAttr,
    /// 그리기 공통 속성
    pub drawing: DrawingObjAttr,
    /// 속성 플래그
    pub attr: u32,
    /// 중심 좌표
    pub center: Point,
    /// 제1축 좌표
    pub axis1: Point,
    /// 제2축 좌표
    pub axis2: Point,
    /// 시작점1
    pub start1: Point,
    /// 끝점1
    pub end1: Point,
    /// 시작점2 (호일 때)
    pub start2: Point,
    /// 끝점2 (호일 때)
    pub end2: Point,
}

/// 호 개체 (HWPTAG_SHAPE_COMPONENT_ARC)
#[derive(Debug, Default, Clone)]
pub struct ArcShape {
    /// 공통 속성
    pub common: CommonObjAttr,
    /// 그리기 공통 속성
    pub drawing: DrawingObjAttr,
    /// 호 타입 (0: Arc, 1: CircularSector, 2: Bow)
    pub arc_type: u8,
    /// 타원 중심
    pub center: Point,
    /// 제1축 좌표
    pub axis1: Point,
    /// 제2축 좌표
    pub axis2: Point,
}

/// 다각형 개체 (HWPTAG_SHAPE_COMPONENT_POLYGON)
#[derive(Debug, Default, Clone)]
pub struct PolygonShape {
    /// 공통 속성
    pub common: CommonObjAttr,
    /// 그리기 공통 속성
    pub drawing: DrawingObjAttr,
    /// 꼭짓점 좌표 목록
    pub points: Vec<Point>,
}

/// 곡선 개체 (HWPTAG_SHAPE_COMPONENT_CURVE)
#[derive(Debug, Default, Clone)]
pub struct CurveShape {
    /// 공통 속성
    pub common: CommonObjAttr,
    /// 그리기 공통 속성
    pub drawing: DrawingObjAttr,
    /// 제어점 좌표 목록
    pub points: Vec<Point>,
    /// 세그먼트 타입 목록 (0: line, 1: curve)
    pub segment_types: Vec<u8>,
}

/// 묶음 개체 (HWPTAG_SHAPE_COMPONENT_CONTAINER)
#[derive(Debug, Default, Clone)]
pub struct GroupShape {
    /// 공통 속성
    pub common: CommonObjAttr,
    /// 개체 요소 속성
    pub shape_attr: ShapeComponentAttr,
    /// 하위 개체 목록
    pub children: Vec<ShapeObject>,
    /// 캡션
    pub caption: Option<Caption>,
}

/// 캡션 정보
#[derive(Debug, Default, Clone)]
pub struct Caption {
    /// 방향 (0: left, 1: right, 2: top, 3: bottom)
    pub direction: CaptionDirection,
    /// Left/Right 캡션의 세로 정렬 (위/가운데/아래)
    pub vert_align: CaptionVertAlign,
    /// 캡션 폭 (세로 방향일 때)
    pub width: HwpUnit,
    /// 캡션-틀 간격
    pub spacing: HwpUnit16,
    /// 텍스트 최대 길이
    pub max_width: HwpUnit,
    /// 캡션 폭에 마진 포함 여부 (가로 방향일 때만 사용)
    pub include_margin: bool,
    /// 문단 리스트
    pub paragraphs: Vec<Paragraph>,
}

/// 캡션 방향
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum CaptionDirection {
    Left,
    Right,
    Top,
    #[default]
    Bottom,
}

/// Left/Right 캡션의 세로 정렬
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum CaptionVertAlign {
    #[default]
    Top,
    Center,
    Bottom,
}

// ============================================================
// 차트 개체 (Task #195)
// ============================================================

/// 차트 종류 (1차 범위: Bar/Column/Line/Pie/Area/Scatter)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ChartType {
    Bar,
    Column,
    Line,
    Pie,
    Area,
    Scatter,
    #[default]
    Unknown,
}

/// 범례 위치
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum LegendPosition {
    #[default]
    Right,
    Left,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Hidden,
}

/// 범례
#[derive(Debug, Clone, Default)]
pub struct Legend {
    pub position: LegendPosition,
    pub visible: bool,
}

/// 축
#[derive(Debug, Clone, Default)]
pub struct Axis {
    pub label: Option<String>,
    pub labels: Vec<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

/// 데이터 시리즈 (한 줄기 막대/선 등)
#[derive(Debug, Clone, Default)]
pub struct DataSeries {
    /// 시리즈 이름 (범례 표시용)
    pub name: String,
    /// Y값 (또는 파이의 각 조각 값)
    pub values: Vec<f64>,
    /// X축 레이블 (시리즈 간 공유되지만 편의상 각자 보관)
    pub categories: Vec<String>,
    /// RGB 색상 (`0xRRGGBB`)
    pub color: Option<u32>,
}

/// 차트 개체 (GSO + HWPTAG_CHART_DATA)
#[derive(Debug, Clone, Default)]
pub struct ChartShape {
    /// 공통 속성
    pub common: CommonObjAttr,
    /// 그리기 공통 속성
    pub drawing: DrawingObjAttr,
    /// 차트 종류
    pub chart_type: ChartType,
    /// 타이틀
    pub title: Option<String>,
    /// 범례
    pub legend: Option<Legend>,
    /// X축
    pub x_axis: Option<Axis>,
    /// Y축
    pub y_axis: Option<Axis>,
    /// 데이터 시리즈 목록
    pub series: Vec<DataSeries>,
    /// CHART_DATA 레코드 원본 바이트(라운드트립 보존용, 하위 태그 전체 병합)
    pub raw_chart_data: Vec<u8>,
    /// 캡션
    pub caption: Option<Caption>,
}

// ============================================================
// OLE 개체 (Task #195)
// ============================================================

/// OLE 프리뷰 이미지 포맷
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OlePreviewFormat {
    Wmf,
    Emf,
    Png,
    Bmp,
}

/// OLE 프리뷰 이미지
#[derive(Debug, Clone)]
pub struct OlePreview {
    pub format: OlePreviewFormat,
    pub bytes: Vec<u8>,
}

/// OLE 표시 방식 (DrawingAspect)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum OleDrawingAspect {
    #[default]
    Content,
    Icon,
    Thumbnail,
    DocPrint,
}

/// OLE 개체 (HWPTAG_SHAPE_COMPONENT_OLE)
#[derive(Debug, Clone, Default)]
pub struct OleShape {
    /// 공통 속성
    pub common: CommonObjAttr,
    /// 그리기 공통 속성
    pub drawing: DrawingObjAttr,
    /// 개체 영역 가로 (HWPUNIT)
    pub extent_x: i32,
    /// 개체 영역 세로 (HWPUNIT)
    pub extent_y: i32,
    /// OLE 속성 플래그
    pub flags: u8,
    /// 표시 방식
    pub drawing_aspect: OleDrawingAspect,
    /// BinData 참조 ID (`BinData/BIN000N.OLE`)
    pub bin_data_id: u32,
    /// 프리뷰 이미지 (단계 4 이후 선택적 채움)
    pub preview: Option<OlePreview>,
    /// OLE 레코드 원본 바이트 (라운드트립 보존)
    pub raw_tag_data: Vec<u8>,
    /// 캡션
    pub caption: Option<Caption>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_obj_attr_default() {
        let attr = CommonObjAttr::default();
        assert!(!attr.treat_as_char);
        assert_eq!(attr.text_wrap, TextWrap::Square);
    }

    #[test]
    fn test_shape_object_line() {
        let line = ShapeObject::Line(LineShape::default());
        match line {
            ShapeObject::Line(_) => assert!(true),
            _ => panic!("Expected Line variant"),
        }
    }

    #[test]
    fn test_text_wrap_variants() {
        assert_eq!(TextWrap::default(), TextWrap::Square);
    }
}
