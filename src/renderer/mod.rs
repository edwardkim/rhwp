//! 렌더링 엔진 모듈
//!
//! IR(Document Model) → 렌더 트리 → 백엔드 렌더링 파이프라인을 구현한다.
//! Renderer Trait으로 추상화하여 Canvas/SVG/HTML 백엔드를 선택할 수 있다.

use serde::Serialize;

use crate::model::style::{LineSpacingType, UnderlineType};

pub mod canvas;
pub mod composer;
pub mod equation;
pub mod font_metrics_data;
pub mod height_measurer;
pub mod html;
pub mod layout;
pub mod page_layout;
pub mod page_number;
pub mod pagination;
pub mod render_tree;
pub mod scheduler;
pub mod style_resolver;
pub mod svg;
pub mod svg_fragment;
#[cfg(not(target_arch = "wasm32"))]
pub mod pdf;
pub mod typeset;
#[cfg(target_arch = "wasm32")]
pub mod web_canvas;

use crate::model::ColorRef;

/// 렌더링 백엔드 종류
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderBackend {
    /// Canvas 2D API (1차)
    Canvas,
    /// SVG 엘리먼트 생성 (2차)
    Svg,
    /// HTML DOM 생성 (3차)
    Html,
}

impl RenderBackend {
    /// 문자열로부터 백엔드 파싱
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "canvas" => Some(RenderBackend::Canvas),
            "svg" => Some(RenderBackend::Svg),
            "html" => Some(RenderBackend::Html),
            _ => None,
        }
    }
}

/// 탭 정지 (렌더링용)
#[derive(Debug, Clone, Serialize)]
pub struct TabStop {
    /// 절대 위치 (px, 단 시작 기준)
    pub position: f64,
    /// 탭 종류 (0=왼쪽, 1=오른쪽, 2=가운데, 3=소수점)
    pub tab_type: u8,
    /// 채움 종류 (0=없음, 1=실선, 2=파선, 3=점선)
    pub fill_type: u8,
}

/// 탭 리더(채움 기호) 렌더링 정보
#[derive(Debug, Clone, Serialize)]
pub struct TabLeaderInfo {
    /// 리더 시작 x (run 내 상대 좌표)
    pub start_x: f64,
    /// 리더 끝 x (run 내 상대 좌표)
    pub end_x: f64,
    /// 채움 종류 (1=실선, 2=파선, 3=점선)
    pub fill_type: u8,
}

/// 텍스트 렌더링 스타일
#[derive(Debug, Clone, Serialize)]
pub struct TextStyle {
    /// 글꼴 이름
    pub font_family: String,
    /// 글꼴 크기 (px)
    pub font_size: f64,
    /// 글자 색상
    pub color: ColorRef,
    /// 진하게
    pub bold: bool,
    /// 기울임
    pub italic: bool,
    /// 밑줄 위치 (None/Bottom/Top)
    pub underline: UnderlineType,
    /// 취소선
    pub strikethrough: bool,
    /// 자간 (px)
    pub letter_spacing: f64,
    /// 장평 비율 (1.0 = 100%, 0.8 = 80%)
    pub ratio: f64,
    /// 기본 탭 간격 (px, 0이면 font_size 기반 fallback)
    pub default_tab_width: f64,
    /// 커스텀 탭 정지 목록 (position 오름차순)
    pub tab_stops: Vec<TabStop>,
    /// 문단 오른쪽 끝 자동 탭 여부
    pub auto_tab_right: bool,
    /// 사용 가능 너비 (px, auto_tab_right 계산용)
    pub available_width: f64,
    /// 단 시작으로부터 run 시작 위치 (탭 절대좌표 변환용)
    pub line_x_offset: f64,
    /// 탭 리더 정보 (compute_char_positions 후 채움)
    pub tab_leaders: Vec<TabLeaderInfo>,
    /// HWPX 인라인 탭 확장 데이터 ([width, leader, type, ...])
    pub inline_tabs: Vec<[u16; 7]>,
    /// 양쪽 정렬용: 공백 문자당 추가 간격 (px)
    pub extra_word_spacing: f64,
    /// 배분/나눔 정렬용: 글자당 추가 간격 (px)
    pub extra_char_spacing: f64,
    /// Task #352: dash leader (3+ 연속 '-') 시퀀스의 글자당 추가 간격 (px).
    /// PDF 와 같이 라인 슬랙을 dash leader 가 흡수하도록 하여, 공백 분배
    /// 부담을 줄이고 자연스러운 단어 간격을 유지한다. 0 이면 미적용.
    pub extra_dash_advance: f64,
    /// 외곽선 종류 (0=없음, 1~6=종류)
    pub outline_type: u8,
    /// 그림자 종류 (0=없음, 1=비연속, 2=연속)
    pub shadow_type: u8,
    /// 그림자 색
    pub shadow_color: ColorRef,
    /// 그림자 X 오프셋 (px)
    pub shadow_offset_x: f64,
    /// 그림자 Y 오프셋 (px)
    pub shadow_offset_y: f64,
    /// 양각
    pub emboss: bool,
    /// 음각
    pub engrave: bool,
    /// 위 첨자
    pub superscript: bool,
    /// 아래 첨자
    pub subscript: bool,
    /// 강조점 종류 (0=없음, 1~6)
    pub emphasis_dot: u8,
    /// 밑줄 모양 (표 27 선 종류, 0=실선 ~ 10=삼중선)
    pub underline_shape: u8,
    /// 취소선 모양 (표 27 선 종류, 0=실선 ~ 10=삼중선)
    pub strike_shape: u8,
    /// 밑줄 색상
    pub underline_color: ColorRef,
    /// 취소선 색상
    pub strike_color: ColorRef,
    /// 음영 색 (형광펜, 0xFFFFFF = 없음)
    pub shade_color: ColorRef,
}

impl TextStyle {
    /// 시각적 bold 여부.
    ///
    /// CharShape.bold=true 외에도 HY헤드라인M 같은 heavy display face 를
    /// 사용할 때 true 를 반환. 해당 face 가 fallback 으로 대체될 때 발생하는
    /// 시각 bold 소실을 보완하기 위해 SVG 출력 시 font-weight="bold" 강제에
    /// 사용된다.
    pub fn is_visually_bold(&self) -> bool {
        self.bold || crate::renderer::style_resolver::is_heavy_display_face(&self.font_family)
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: String::new(),
            font_size: 0.0,
            color: 0,
            bold: false,
            italic: false,
            underline: UnderlineType::None,
            strikethrough: false,
            letter_spacing: 0.0,
            ratio: 1.0,
            default_tab_width: 0.0,
            tab_stops: Vec::new(),
            auto_tab_right: false,
            available_width: 0.0,
            line_x_offset: 0.0,
            tab_leaders: Vec::new(),
            inline_tabs: Vec::new(),
            extra_word_spacing: 0.0,
            extra_char_spacing: 0.0,
            extra_dash_advance: 0.0,
            outline_type: 0,
            shadow_type: 0,
            shadow_color: 0x00B2B2B2,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            emboss: false,
            engrave: false,
            superscript: false,
            subscript: false,
            emphasis_dot: 0,
            underline_shape: 0,
            strike_shape: 0,
            underline_color: 0,
            strike_color: 0,
            shade_color: 0x00FFFFFF,
        }
    }
}

/// 패턴 채우기 정보 (HWP pattern_type 1~6)
#[derive(Debug, Clone, Copy, Serialize)]
pub struct PatternFillInfo {
    /// 패턴 종류 (1=가로줄, 2=세로줄, 3=역대각선, 4=대각선, 5=십자, 6=격자)
    pub pattern_type: i32,
    /// 무늬색
    pub pattern_color: ColorRef,
    /// 배경색
    pub background_color: ColorRef,
}

/// 도형 렌더링 스타일
#[derive(Debug, Clone, Serialize)]
pub struct ShapeStyle {
    /// 채우기 색상 (None이면 채우기 없음)
    pub fill_color: Option<ColorRef>,
    /// 패턴 채우기 (pattern_type > 0일 때)
    pub pattern: Option<PatternFillInfo>,
    /// 테두리 색상
    pub stroke_color: Option<ColorRef>,
    /// 테두리 두께 (px)
    pub stroke_width: f64,
    /// 테두리 종류
    pub stroke_dash: StrokeDash,
    /// 투명도 (0.0=완전투명, 1.0=불투명)
    pub opacity: f64,
    /// 그림자 (None이면 그림자 없음)
    pub shadow: Option<ShadowStyle>,
}

/// 도형 그림자 스타일
#[derive(Debug, Clone, Serialize)]
pub struct ShadowStyle {
    /// 그림자 종류 (1~8)
    pub shadow_type: u32,
    /// 그림자 색상
    pub color: ColorRef,
    /// X 오프셋 (px)
    pub offset_x: f64,
    /// Y 오프셋 (px)
    pub offset_y: f64,
    /// 투명도 (0~255, 0=불투명)
    pub alpha: u8,
}

impl Default for ShapeStyle {
    fn default() -> Self {
        Self {
            fill_color: None,
            pattern: None,
            stroke_color: None,
            stroke_width: 0.0,
            stroke_dash: StrokeDash::default(),
            opacity: 1.0,
            shadow: None,
        }
    }
}

/// 그라데이션 채우기 렌더링 정보
#[derive(Debug, Clone, Serialize)]
pub struct GradientFillInfo {
    /// 유형 (1: 줄무늬/선형, 2: 원형, 3: 원뿔형, 4: 사각형)
    pub gradient_type: i16,
    /// 기울임 각도 (도)
    pub angle: i16,
    /// 가로 중심 (%)
    pub center_x: i16,
    /// 세로 중심 (%)
    pub center_y: i16,
    /// 색상 목록 (ColorRef)
    pub colors: Vec<ColorRef>,
    /// 색상 위치 (0.0~1.0 정규화)
    pub positions: Vec<f64>,
}

/// 선 렌더링 스타일
#[derive(Debug, Clone, Default, Serialize)]
pub struct LineStyle {
    /// 선 색상
    pub color: ColorRef,
    /// 선 두께 (px)
    pub width: f64,
    /// 선 종류
    pub dash: StrokeDash,
    /// 선 렌더링 종류 (이중선/삼중선 등)
    pub line_type: LineRenderType,
    /// 시작 화살표
    pub start_arrow: ArrowStyle,
    /// 끝 화살표
    pub end_arrow: ArrowStyle,
    /// 시작 화살표 크기 (HWP bits 22-25: 0=작은-작은 ~ 8=큰-큰)
    pub start_arrow_size: u8,
    /// 끝 화살표 크기 (HWP bits 26-29)
    pub end_arrow_size: u8,
    /// 그림자
    pub shadow: Option<ShadowStyle>,
}

/// 테두리 점선 종류
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub enum StrokeDash {
    #[default]
    Solid,
    Dash,
    Dot,
    DashDot,
    DashDotDot,
}

/// 선 렌더링 종류 (이중선/삼중선)
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub enum LineRenderType {
    #[default]
    Single,
    /// 이중선 (같은 굵기)
    Double,
    /// 가는선-굵은선 이중선
    ThinThickDouble,
    /// 굵은선-가는선 이중선
    ThickThinDouble,
    /// 가는선-굵은선-가는선 삼중선
    ThinThickThinTriple,
}

/// 화살표 스타일
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub enum ArrowStyle {
    #[default]
    None,
    /// 화살 모양 (채움)
    Arrow,
    /// 오목한 화살 모양 (채움)
    ConcaveArrow,
    /// 속이 빈 다이아몬드
    OpenDiamond,
    /// 속이 빈 원
    OpenCircle,
    /// 속이 빈 사각
    OpenSquare,
    /// 속이 채운 다이아몬드
    Diamond,
    /// 속이 채운 원
    Circle,
    /// 속이 채운 사각
    Square,
}

/// 패스 커맨드 (벡터 도형용)
#[derive(Debug, Clone, Copy, Serialize)]
pub enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64),
    /// SVG arc: (rx, ry, x_rotation, large_arc_flag, sweep_flag, x, y)
    ArcTo(f64, f64, f64, bool, bool, f64, f64),
    ClosePath,
}

/// SVG arc(endpoint parameterization)를 cubic bezier 곡선으로 변환
///
/// SVG spec: Implementation Notes - Arc Conversion
/// (x1, y1): 시작점, (x2, y2): 끝점, rx/ry: 반지름,
/// phi: x축 회전(도), large_arc/sweep: 플래그
pub fn svg_arc_to_beziers(
    x1: f64, y1: f64,
    mut rx: f64, mut ry: f64,
    phi_deg: f64,
    large_arc: bool, sweep: bool,
    x2: f64, y2: f64,
) -> Vec<PathCommand> {
    use std::f64::consts::PI;

    let mut result = Vec::new();

    // 퇴화 케이스: 시작점 == 끝점
    if (x1 - x2).abs() < 1e-6 && (y1 - y2).abs() < 1e-6 {
        return result;
    }
    // 퇴화 케이스: 반지름 0
    rx = rx.abs();
    ry = ry.abs();
    if rx < 1e-6 || ry < 1e-6 {
        result.push(PathCommand::LineTo(x2, y2));
        return result;
    }

    let phi = phi_deg.to_radians();
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();

    // Step 1: (x1', y1') 계산
    let dx = (x1 - x2) / 2.0;
    let dy = (y1 - y2) / 2.0;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;

    // Step 2: 반지름 보정 (너무 작은 경우 확대)
    let x1p2 = x1p * x1p;
    let y1p2 = y1p * y1p;
    let lambda = x1p2 / (rx * rx) + y1p2 / (ry * ry);
    if lambda > 1.0 {
        let s = lambda.sqrt();
        rx *= s;
        ry *= s;
    }
    let rx2 = rx * rx;
    let ry2 = ry * ry;

    // Step 3: 중심점' (cx', cy') 계산
    let num = (rx2 * ry2 - rx2 * y1p2 - ry2 * x1p2).max(0.0);
    let den = rx2 * y1p2 + ry2 * x1p2;
    let sq = if den > 1e-10 { (num / den).sqrt() } else { 0.0 };
    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let cxp = sign * sq * rx * y1p / ry;
    let cyp = sign * sq * (-ry * x1p) / rx;

    // Step 4: 중심점 (cx, cy) 계산
    let cx = cos_phi * cxp - sin_phi * cyp + (x1 + x2) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (y1 + y2) / 2.0;

    // Step 5: θ1 (시작 각도), dθ (호 각도) 계산
    let theta1 = ((y1p - cyp) / ry).atan2((x1p - cxp) / rx);
    let theta2 = ((-y1p - cyp) / ry).atan2((-x1p - cxp) / rx);
    let mut dtheta = theta2 - theta1;

    if !sweep && dtheta > 0.0 {
        dtheta -= 2.0 * PI;
    }
    if sweep && dtheta < 0.0 {
        dtheta += 2.0 * PI;
    }

    // 호를 최대 90° 세그먼트로 분할하여 bezier 근사
    let n_segs = (dtheta.abs() / (PI / 2.0 + 0.001)).ceil().max(1.0) as usize;
    let seg_angle = dtheta / n_segs as f64;

    for i in 0..n_segs {
        let t1 = theta1 + seg_angle * i as f64;
        let t2 = theta1 + seg_angle * (i + 1) as f64;

        // 호 세그먼트의 bezier 제어점 계산
        // alpha = 4/3 * tan(segment_angle / 4)
        let alpha = 4.0 / 3.0 * (seg_angle / 4.0).tan();

        let cos_t1 = t1.cos();
        let sin_t1 = t1.sin();
        let cos_t2 = t2.cos();
        let sin_t2 = t2.sin();

        // 단위 원 위의 제어점 (반지름 적용 전)
        let ep1x = cos_t1 - alpha * sin_t1;
        let ep1y = sin_t1 + alpha * cos_t1;
        let ep2x = cos_t2 + alpha * sin_t2;
        let ep2y = sin_t2 - alpha * cos_t2;

        // 반지름 적용
        let cp1x = rx * ep1x;
        let cp1y = ry * ep1y;
        let cp2x = rx * ep2x;
        let cp2y = ry * ep2y;
        let endx = rx * cos_t2;
        let endy = ry * sin_t2;

        // 회전(phi) + 이동(cx, cy) 적용
        result.push(PathCommand::CurveTo(
            cos_phi * cp1x - sin_phi * cp1y + cx,
            sin_phi * cp1x + cos_phi * cp1y + cy,
            cos_phi * cp2x - sin_phi * cp2y + cx,
            sin_phi * cp2x + cos_phi * cp2y + cy,
            cos_phi * endx - sin_phi * endy + cx,
            sin_phi * endx + cos_phi * endy + cy,
        ));
    }

    result
}

/// 렌더러 트레이트 (모든 백엔드가 구현)
pub trait Renderer {
    /// 페이지 렌더링 시작
    fn begin_page(&mut self, width: f64, height: f64);
    /// 페이지 렌더링 종료
    fn end_page(&mut self);

    /// 텍스트 그리기
    fn draw_text(&mut self, text: &str, x: f64, y: f64, style: &TextStyle);
    /// 사각형 그리기 (corner_radius > 0이면 둥근 모서리)
    fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64, corner_radius: f64, style: &ShapeStyle);
    /// 선 그리기
    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: &LineStyle);
    /// 타원 그리기
    fn draw_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, style: &ShapeStyle);
    /// 이미지 그리기
    fn draw_image(&mut self, data: &[u8], x: f64, y: f64, w: f64, h: f64);
    /// 패스 그리기 (벡터 도형)
    fn draw_path(&mut self, commands: &[PathCommand], style: &ShapeStyle);
}

/// HWPUNIT → 픽셀 변환 (96 DPI 기준)
pub const DEFAULT_DPI: f64 = 96.0;
pub const HWPUNIT_PER_INCH: f64 = 7200.0;

/// LINE_SEG line_height가 줄의 최대 글자 크기보다 작으면
/// ParaShape의 줄간격 설정으로 재계산한다.
/// height_measurer와 layout 양쪽에서 동일 로직을 사용해야 한다.
#[inline]
pub fn corrected_line_height(
    raw_lh: f64,
    max_fs: f64,
    ls_type: LineSpacingType,
    ls_val: f64,
) -> f64 {
    if max_fs > 0.0 && raw_lh < max_fs {
        match ls_type {
            LineSpacingType::Percent   => max_fs * ls_val / 100.0,
            LineSpacingType::Fixed     => ls_val.max(max_fs),
            LineSpacingType::SpaceOnly => max_fs + ls_val,
            LineSpacingType::Minimum   => ls_val.max(max_fs),
        }
    } else {
        raw_lh
    }
}

/// HWPUNIT을 픽셀로 변환
#[inline]
pub fn hwpunit_to_px(hwpunit: i32, dpi: f64) -> f64 {
    hwpunit as f64 * dpi / HWPUNIT_PER_INCH
}

/// 픽셀을 HWPUNIT으로 변환
#[inline]
pub fn px_to_hwpunit(px: f64, dpi: f64) -> i32 {
    (px * HWPUNIT_PER_INCH / dpi) as i32
}

/// CSS generic fallback 반환 (serif 또는 sans-serif)
///
/// 폰트 이름에 명조/바탕/궁서 등 세리프 계열 키워드가 포함되면 "serif",
/// 그 외에는 "sans-serif"를 반환한다.
pub fn generic_fallback(font_family: &str) -> &'static str {
    if font_family.is_empty() {
        // Sans-serif: Windows → macOS/iOS → Android → 오픈소스 → generic
        return "'Malgun Gothic','맑은 고딕','Apple SD Gothic Neo','Noto Sans KR','Pretendard',sans-serif";
    }
    // 고정폭 키워드
    let lower = font_family.to_ascii_lowercase();
    if font_family.contains("굴림체") || font_family.contains("바탕체")
        || lower.contains("gulimche") || lower.contains("batangche")
        || lower.contains("coding") || lower.contains("courier")
    {
        // Monospace: Windows → 오픈소스 → generic
        return "'GulimChe','굴림체','D2Coding','Noto Sans Mono',monospace";
    }
    // 세리프 키워드 (한글)
    if font_family.contains("바탕") || font_family.contains("명조")
        || font_family.contains("궁서")
    {
        // Serif: Windows → macOS(Bold 보유 우선) → macOS 기본 → Android → 오픈소스 → 리눅스 시스템 → generic
        // Nanum Myeongjo 는 macOS 10.9+ 기본 설치이며 Bold variant 보유.
        // AppleMyungjo 보다 앞에 두어야 macOS Chrome 에서 CJK 글리프 bold 매칭 성공.
        return "'Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR',serif";
    }
    // 세리프 키워드 (영문)
    if lower.contains("times") || lower.contains("hymjre")
        || lower.contains("palatino") || lower.contains("georgia")
        || lower.contains("batang") || lower.contains("gungsuh")
    {
        return "'Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR',serif";
    }
    // Sans-serif: Windows → macOS/iOS → Android → 오픈소스 → generic
    "'Malgun Gothic','맑은 고딕','Apple SD Gothic Neo','Noto Sans KR','Pretendard',sans-serif"
}

// ============================================================
// 자동 번호 매기기 (AutoNumber)
// ============================================================

use crate::model::control::AutoNumberType;

/// 자동 번호 카운터
///
/// 각 번호 종류별로 카운터를 유지하여 순차적인 번호를 생성한다.
#[derive(Debug, Clone, Default)]
pub struct AutoNumberCounter {
    /// 그림 번호
    pub picture: u16,
    /// 표 번호
    pub table: u16,
    /// 수식 번호
    pub equation: u16,
    /// 각주 번호
    pub footnote: u16,
    /// 미주 번호
    pub endnote: u16,
    /// 쪽 번호
    pub page: u16,
}

impl AutoNumberCounter {
    /// 새 카운터 생성
    pub fn new() -> Self {
        Self::default()
    }

    /// 번호 증가 후 현재 값 반환
    pub fn increment(&mut self, number_type: AutoNumberType) -> u16 {
        match number_type {
            AutoNumberType::Picture => {
                self.picture += 1;
                self.picture
            }
            AutoNumberType::Table => {
                self.table += 1;
                self.table
            }
            AutoNumberType::Equation => {
                self.equation += 1;
                self.equation
            }
            AutoNumberType::Footnote => {
                self.footnote += 1;
                self.footnote
            }
            AutoNumberType::Endnote => {
                self.endnote += 1;
                self.endnote
            }
            AutoNumberType::Page => {
                self.page += 1;
                self.page
            }
        }
    }

    /// 현재 번호 조회 (증가 없이)
    pub fn current(&self, number_type: AutoNumberType) -> u16 {
        match number_type {
            AutoNumberType::Picture => self.picture,
            AutoNumberType::Table => self.table,
            AutoNumberType::Equation => self.equation,
            AutoNumberType::Footnote => self.footnote,
            AutoNumberType::Endnote => self.endnote,
            AutoNumberType::Page => self.page,
        }
    }

    /// 모든 카운터 초기화
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// 번호 형식
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum NumberFormat {
    /// 아라비아 숫자: 1, 2, 3
    #[default]
    Digit,
    /// 원 문자: ①, ②, ③
    CircledDigit,
    /// 로마 숫자 대문자: I, II, III
    RomanUpper,
    /// 로마 숫자 소문자: i, ii, iii
    RomanLower,
    /// 영문 대문자: A, B, C
    LatinUpper,
    /// 영문 소문자: a, b, c
    LatinLower,
    /// 한글 가나다: 가, 나, 다
    HangulGaNaDa,
    /// 한글 일이삼: 일, 이, 삼
    HangulNumber,
    /// 한자 一二三: 一, 二, 三
    HanjaNumber,
}

impl NumberFormat {
    /// HWP 형식 코드에서 변환
    pub fn from_hwp_format(format: u8) -> Self {
        match format {
            0 => NumberFormat::Digit,
            1 => NumberFormat::CircledDigit,
            2 => NumberFormat::RomanUpper,
            3 => NumberFormat::RomanLower,
            4 => NumberFormat::LatinUpper,
            5 => NumberFormat::LatinLower,
            6 => NumberFormat::HangulGaNaDa,
            7 => NumberFormat::HangulNumber,
            8 => NumberFormat::HanjaNumber,
            _ => NumberFormat::Digit,
        }
    }
}

/// 번호를 문자열로 변환
pub fn format_number(number: u16, format: NumberFormat) -> String {
    match format {
        NumberFormat::Digit => number.to_string(),
        NumberFormat::CircledDigit => format_circled_digit(number),
        NumberFormat::RomanUpper => format_roman(number, true),
        NumberFormat::RomanLower => format_roman(number, false),
        NumberFormat::LatinUpper => format_latin(number, true),
        NumberFormat::LatinLower => format_latin(number, false),
        NumberFormat::HangulGaNaDa => format_hangul_ganada(number),
        NumberFormat::HangulNumber => format_hangul_number(number),
        NumberFormat::HanjaNumber => format_hanja_number(number),
    }
}

/// 원 문자 변환 (① ~ ⑳, 이후 숫자)
fn format_circled_digit(n: u16) -> String {
    const CIRCLED: [char; 20] = [
        '①', '②', '③', '④', '⑤', '⑥', '⑦', '⑧', '⑨', '⑩',
        '⑪', '⑫', '⑬', '⑭', '⑮', '⑯', '⑰', '⑱', '⑲', '⑳',
    ];
    if n >= 1 && n <= 20 {
        CIRCLED[(n - 1) as usize].to_string()
    } else {
        n.to_string()
    }
}

/// 로마 숫자 변환
fn format_roman(n: u16, upper: bool) -> String {
    if n == 0 || n > 3999 {
        return n.to_string();
    }

    let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
    let symbols_upper = ["M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I"];
    let symbols_lower = ["m", "cm", "d", "cd", "c", "xc", "l", "xl", "x", "ix", "v", "iv", "i"];

    let symbols = if upper { &symbols_upper } else { &symbols_lower };
    let mut result = String::new();
    let mut num = n as i32;

    for (i, &val) in values.iter().enumerate() {
        while num >= val {
            result.push_str(symbols[i]);
            num -= val;
        }
    }
    result
}

/// 영문자 변환 (A-Z, AA-AZ, ...)
fn format_latin(n: u16, upper: bool) -> String {
    if n == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut num = n;

    while num > 0 {
        num -= 1;
        let c = if upper {
            (b'A' + (num % 26) as u8) as char
        } else {
            (b'a' + (num % 26) as u8) as char
        };
        result.insert(0, c);
        num /= 26;
    }
    result
}

/// 한글 가나다 변환
fn format_hangul_ganada(n: u16) -> String {
    const GANADA: [char; 14] = ['가', '나', '다', '라', '마', '바', '사', '아', '자', '차', '카', '타', '파', '하'];
    if n >= 1 && n <= 14 {
        GANADA[(n - 1) as usize].to_string()
    } else {
        n.to_string()
    }
}

/// 한글 숫자 변환 (일, 이, 삼, ...)
fn format_hangul_number(n: u16) -> String {
    const HANGUL_DIGITS: [&str; 10] = ["", "일", "이", "삼", "사", "오", "육", "칠", "팔", "구"];
    const HANGUL_UNITS: [&str; 4] = ["", "십", "백", "천"];
    const HANGUL_LARGE: [&str; 4] = ["", "만", "억", "조"];

    if n == 0 {
        return "영".to_string();
    }

    let mut result = String::new();
    let mut num = n as u32;
    let mut large_unit = 0;

    while num > 0 {
        let group = (num % 10000) as usize;
        if group > 0 {
            let mut group_str = String::new();
            let mut g = group;
            let mut unit = 0;

            while g > 0 {
                let digit = g % 10;
                if digit > 0 {
                    let digit_str = if digit == 1 && unit > 0 { "" } else { HANGUL_DIGITS[digit] };
                    group_str.insert_str(0, HANGUL_UNITS[unit]);
                    group_str.insert_str(0, digit_str);
                }
                g /= 10;
                unit += 1;
            }
            group_str.push_str(HANGUL_LARGE[large_unit]);
            result.insert_str(0, &group_str);
        }
        num /= 10000;
        large_unit += 1;
    }
    result
}

/// 한자 숫자 변환 (一, 二, 三, ...)
fn format_hanja_number(n: u16) -> String {
    const HANJA_DIGITS: [&str; 10] = ["", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
    const HANJA_UNITS: [&str; 4] = ["", "十", "百", "千"];
    const HANJA_LARGE: [&str; 4] = ["", "萬", "億", "兆"];

    if n == 0 {
        return "零".to_string();
    }

    let mut result = String::new();
    let mut num = n as u32;
    let mut large_unit = 0;

    while num > 0 {
        let group = (num % 10000) as usize;
        if group > 0 {
            let mut group_str = String::new();
            let mut g = group;
            let mut unit = 0;

            while g > 0 {
                let digit = g % 10;
                if digit > 0 {
                    let digit_str = if digit == 1 && unit > 0 { "" } else { HANJA_DIGITS[digit] };
                    group_str.insert_str(0, HANJA_UNITS[unit]);
                    group_str.insert_str(0, digit_str);
                }
                g /= 10;
                unit += 1;
            }
            group_str.push_str(HANJA_LARGE[large_unit]);
            result.insert_str(0, &group_str);
        }
        num /= 10000;
        large_unit += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_backend_from_str() {
        assert_eq!(RenderBackend::from_str("canvas"), Some(RenderBackend::Canvas));
        assert_eq!(RenderBackend::from_str("svg"), Some(RenderBackend::Svg));
        assert_eq!(RenderBackend::from_str("html"), Some(RenderBackend::Html));
        assert_eq!(RenderBackend::from_str("unknown"), None);
    }

    #[test]
    fn test_hwpunit_to_px() {
        // 1인치 = 7200 HWPUNIT, 96 DPI → 96px
        let px = hwpunit_to_px(7200, 96.0);
        assert!((px - 96.0).abs() < 0.01);
    }

    #[test]
    fn test_px_to_hwpunit() {
        let hu = px_to_hwpunit(96.0, 96.0);
        assert_eq!(hu, 7200);
    }

    #[test]
    fn test_a4_page_size_px() {
        // A4: 210mm × 297mm = 59528 × 84188 HWPUNIT
        let w = hwpunit_to_px(59528, 96.0);
        let h = hwpunit_to_px(84188, 96.0);
        // A4 @ 96DPI ≈ 793.7 × 1122.5 px
        assert!((w - 793.7).abs() < 1.0);
        assert!((h - 1122.5).abs() < 1.0);
    }

    #[test]
    fn test_auto_number_counter() {
        let mut counter = AutoNumberCounter::new();
        assert_eq!(counter.increment(AutoNumberType::Picture), 1);
        assert_eq!(counter.increment(AutoNumberType::Picture), 2);
        assert_eq!(counter.increment(AutoNumberType::Table), 1);
        assert_eq!(counter.current(AutoNumberType::Picture), 2);
        assert_eq!(counter.current(AutoNumberType::Table), 1);
        counter.reset();
        assert_eq!(counter.current(AutoNumberType::Picture), 0);
    }

    #[test]
    fn test_format_number_digit() {
        assert_eq!(format_number(1, NumberFormat::Digit), "1");
        assert_eq!(format_number(123, NumberFormat::Digit), "123");
    }

    #[test]
    fn test_format_number_circled() {
        assert_eq!(format_number(1, NumberFormat::CircledDigit), "①");
        assert_eq!(format_number(10, NumberFormat::CircledDigit), "⑩");
        assert_eq!(format_number(20, NumberFormat::CircledDigit), "⑳");
        assert_eq!(format_number(21, NumberFormat::CircledDigit), "21");
    }

    #[test]
    fn test_format_number_roman() {
        assert_eq!(format_number(1, NumberFormat::RomanUpper), "I");
        assert_eq!(format_number(4, NumberFormat::RomanUpper), "IV");
        assert_eq!(format_number(9, NumberFormat::RomanUpper), "IX");
        assert_eq!(format_number(10, NumberFormat::RomanLower), "x");
        assert_eq!(format_number(14, NumberFormat::RomanLower), "xiv");
    }

    #[test]
    fn test_format_number_latin() {
        assert_eq!(format_number(1, NumberFormat::LatinUpper), "A");
        assert_eq!(format_number(26, NumberFormat::LatinUpper), "Z");
        assert_eq!(format_number(27, NumberFormat::LatinUpper), "AA");
        assert_eq!(format_number(1, NumberFormat::LatinLower), "a");
    }

    #[test]
    fn test_generic_fallback() {
        let serif = "'Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR',serif";
        let sans = "'Malgun Gothic','맑은 고딕','Apple SD Gothic Neo','Noto Sans KR','Pretendard',sans-serif";
        let mono = "'GulimChe','굴림체','D2Coding','Noto Sans Mono',monospace";
        // 세리프 계열
        assert_eq!(generic_fallback("함초롬바탕"), serif);
        assert_eq!(generic_fallback("바탕"), serif);
        assert_eq!(generic_fallback("궁서"), serif);
        assert_eq!(generic_fallback("HY견명조"), serif);
        assert_eq!(generic_fallback("Times New Roman"), serif);
        assert_eq!(generic_fallback("Palatino Linotype"), serif);
        // 산세리프 계열
        assert_eq!(generic_fallback("함초롬돋움"), sans);
        assert_eq!(generic_fallback("돋움"), sans);
        assert_eq!(generic_fallback("굴림"), sans);
        assert_eq!(generic_fallback("Arial"), sans);
        assert_eq!(generic_fallback("맑은 고딕"), sans);
        // 고정폭 계열
        assert_eq!(generic_fallback("굴림체"), mono);
        assert_eq!(generic_fallback("바탕체"), mono);
        assert_eq!(generic_fallback("Courier New"), mono);
        // 빈 문자열
        assert_eq!(generic_fallback(""), sans);
    }

    #[test]
    fn test_format_number_hangul() {
        assert_eq!(format_number(1, NumberFormat::HangulGaNaDa), "가");
        assert_eq!(format_number(2, NumberFormat::HangulGaNaDa), "나");
        assert_eq!(format_number(1, NumberFormat::HangulNumber), "일");
        assert_eq!(format_number(12, NumberFormat::HangulNumber), "십이");
    }
}
