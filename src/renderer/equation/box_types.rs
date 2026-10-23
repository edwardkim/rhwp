//! 수식 측정·배치가 공유하는 결과 타입.

use super::ast::MatrixStyle;

/// 측정 단계에서 선택한 글리프의 합성 굵기와 장평.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct FontGlyphPaint {
    pub stroke_width: f64,
    pub scale_x: f64,
}

/// 수식 레이아웃 박스
#[derive(Debug, Clone, serde::Serialize)]
pub struct LayoutBox {
    /// 측정에 사용한 글자 크기. 각 backend가 첨자 비율을 다시 추정하지 않는다.
    pub font_size: f64,
    /// X 위치 (부모 기준 상대 좌표)
    pub x: f64,
    /// Y 위치 (부모 기준 상대 좌표)
    pub y: f64,
    /// 폭
    pub width: f64,
    /// 높이
    pub height: f64,
    /// 기준선 (상단으로부터의 거리, 텍스트 정렬의 기준)
    pub baseline: f64,
    /// 렌더링 요소
    pub kind: LayoutKind,
}

/// 레이아웃 요소 종류
#[derive(Debug, Clone, serde::Serialize)]
pub enum LayoutKind {
    /// 선택된 한컴 글리프와 측정에 사용한 같은 전진폭.
    HancomGlyphs(super::hancom_glyphs::GlyphRun),
    /// 수평 나열
    Row(Vec<LayoutBox>),
    /// 일반 텍스트 (이탤릭)
    Text(String),
    /// 숫자
    Number(String),
    /// 기호
    Symbol(String),
    /// 수학 기호 (Unicode)
    MathSymbol(String),
    /// 실제 선택한 대체 글꼴과 전진폭을 함께 전달하는 표시 글리프.
    FontGlyph {
        text: String,
        font_family: &'static str,
        /// 표시 글꼴의 차이를 배치 단계에서 해소한 공통 공개 윤곽.
        outline: Option<Vec<super::GlyphOutlineCommand>>,
        /// 윤곽의 fill+stroke 또는 글꼴 글리프의 가로 변환. 없는 경우 원 배율이다.
        #[serde(skip_serializing_if = "Option::is_none")]
        paint: Option<FontGlyphPaint>,
    },
    /// 함수 이름 (로만체)
    Function(String),
    /// 색 범위는 배치와 무관하며 모든 출력 경로에 같은 본체를 공급한다.
    Color { rgb: [u8; 3], body: Box<LayoutBox> },
    /// 분수
    Fraction {
        numer: Box<LayoutBox>,
        denom: Box<LayoutBox>,
        /// 측정에서 정한 기준선 위 수학 축. 각 출력 경로가 그대로 소비한다.
        axis_height: f64,
        /// 실제 수식 face의 괘선 두께. 출력 경로가 일반 MATH 비율로 덮어쓰지 않는다.
        rule_thickness: f64,
        /// 피연산자의 등록 여백과 구분한 괘선의 수평 시작·끝. 모든 paint가 공유한다.
        rule_start: f64,
        rule_end: f64,
    },
    /// 위아래 배치 (분수선 없음)
    Atop {
        top: Box<LayoutBox>,
        bottom: Box<LayoutBox>,
    },
    /// 제곱근
    Sqrt {
        index: Option<Box<LayoutBox>>,
        body: Box<LayoutBox>,
    },
    /// 위첨자
    Superscript {
        base: Box<LayoutBox>,
        sup: Box<LayoutBox>,
    },
    /// 아래첨자
    Subscript {
        base: Box<LayoutBox>,
        sub: Box<LayoutBox>,
    },
    /// 위·아래첨자
    SubSup {
        base: Box<LayoutBox>,
        sub: Box<LayoutBox>,
        sup: Box<LayoutBox>,
    },
    /// 큰 연산자
    BigOp {
        symbol: String,
        op_font_size: f64,
        op_width: f64,
        op_x: f64,
        op_y: f64,
        op_baseline: f64,
        sub: Option<Box<LayoutBox>>,
        sup: Option<Box<LayoutBox>>,
    },
    /// 극한
    Limit {
        is_upper: bool,
        /// 한컴 이름 글리프와 측정에 사용한 같은 메트릭.
        name: Option<Box<LayoutBox>>,
        name_font_size: f64,
        sub_font_size: f64,
        sub: Option<Box<LayoutBox>>,
    },
    /// 행렬
    Matrix {
        cells: Vec<Vec<LayoutBox>>,
        style: MatrixStyle,
    },
    /// 관계식 (REL/BUILDREL) — 화살표 위/아래 내용
    Rel {
        arrow: Box<LayoutBox>,
        over: Box<LayoutBox>,
        under: Option<Box<LayoutBox>>,
    },
    /// 칸 맞춤 정렬 (EQALIGN)
    EqAlign {
        rows: Vec<(LayoutBox, LayoutBox)>, // (왼쪽, 오른쪽) 쌍
    },
    /// 괄호
    Paren {
        left: String,
        right: String,
        left_width: f64,
        right_width: f64,
        delimiter_height: f64,
        glyph_delimiters: bool,
        /// 조판이 발행한 구분자 상대 윤곽. 논리 점유·글자 크기와 ink를 분리한다.
        #[serde(skip_serializing_if = "Option::is_none")]
        left_outline: Option<Vec<super::GlyphOutlineCommand>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        right_outline: Option<Vec<super::GlyphOutlineCommand>>,
        body: Box<LayoutBox>,
    },
    /// 장식
    Decoration {
        kind: super::symbols::DecoKind,
        body: Box<LayoutBox>,
        /// 레이아웃에서 발행한 부정 기하. backend가 글자 종류나 DPI를 재추측하지 않는다.
        #[serde(skip_serializing_if = "Option::is_none")]
        negation: Option<super::NegationGeometry>,
        /// 배치에서 발행한 공개 장식 윤곽. 없는 장식은 기존 표시 경로를 쓴다.
        #[serde(skip_serializing_if = "Option::is_none")]
        outline: Option<Vec<super::GlyphOutlineCommand>>,
    },
    /// 글꼴 스타일
    FontStyle {
        style: super::symbols::FontStyleKind,
        body: Box<LayoutBox>,
    },
    /// 공백
    Space(f64),
    /// 줄바꿈 (세로 쌓기용 마커)
    Newline,
    /// 빈 박스
    Empty,
}
