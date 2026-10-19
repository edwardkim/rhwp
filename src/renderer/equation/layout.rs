//! 수식 레이아웃 엔진
//!
//! AST(EqNode)를 레이아웃 박스(LayoutBox)로 변환하여
//! 각 요소의 크기와 위치를 계산한다.

use super::ast::*;
use super::symbols::FontStyleKind;

mod atop_geometry;
mod big_op_geometry;
mod binomial_geometry;
mod cases_columns;
mod decoration_geometry;
mod delimiter_advance;
mod delimiter_outline;
mod fraction_geometry;
mod glyph_advances;
mod hancom_decoration;
mod hancom_sum;
mod integral_placement;
mod ladder_geometry;
mod left_script_geometry;
mod limit_geometry;
mod literal_text;
mod long_div_geometry;
mod matrix_geometry;
mod matrix_row_frame;
mod paren_geometry;
mod plain_matrix_geometry;
mod script_geometry;
mod source_font_advance;
mod source_font_symbol;
mod source_font_text;
mod source_glyph_runs;
mod source_greek_symbol;
mod source_italic_advance;
mod source_roman_advance;
pub(super) use source_roman_advance::measured as measured_roman_advance;
mod source_variable_style;
mod symbol_advances;
mod symbol_spacing;
mod variable_style;
pub(crate) use variable_style::is_styled_variable;

pub use super::box_types::{FontGlyphPaint, LayoutBox, LayoutKind};

/// 수식 레이아웃 계산기
#[derive(Clone, Copy)]
pub struct EqLayout {
    /// 기본 글꼴 크기 (px)
    pub font_size: f64,
    dpi: f64,
    hancom_cjk_advance: bool,
    hancom_latin: bool,
    latin_italic: bool,
    hancom_math_face: bool,
    hancom_eqn: bool,
    math_digit_advance: Option<u16>,
    variable_italic: bool,
    variable_bold: bool,
    source_variable_glyphs: bool,
    /// 다른 수식 구조에 속하지 않는 정렬 묶음의 자체 leading.
    standalone_eqalign: bool,
    legacy_hft_metrics: bool,
}

/// 비율 상수
pub(crate) const SCRIPT_SCALE: f64 = 0.7; // 첨자 크기 비율
/// 근 지수는 일반 첨자보다 작은 수학 scriptscript 크기다(MATH 50%).
pub(crate) const RADICAL_DEGREE_SCALE: f64 = 0.5;
const FRAC_LINE_PAD: f64 = 0.2; // 분수선 상하 여백 (font_size 비율)
const FRAC_LINE_THICK: f64 = 0.04; // 분수선 두께 (font_size 비율)
const SQRT_PAD: f64 = 0.1; // 제곱근 내부 상단 여백
const PAREN_PAD: f64 = 0.08; // 괄호 내부 좌우 여백
pub(crate) const BIG_OP_SCALE: f64 = 1.5; // 큰 연산자(∑/∏) 크기 비율
/// 적분(∫/∮ 등) 전용 크기 비율 — Task #1313.
/// 적분 글리프는 ∑/∏ 보다 세로로 길게 그려져야 정답(한글 2022)과 정합한다. BIG_OP_SCALE
/// (1.5) 로는 글리프가 작아 상·하한이 기호와 벌어져 보이므로 적분만 별도 스케일을 쓴다.
pub(crate) const INTEGRAL_SCALE: f64 = 2.5;

/// 적분 글리프 path 기하 — Task #1317.
///
/// 적분기호(∫)를 폰트 `<text>` 가 아닌 stroke path 로 그릴 때의 글리프 형상과,
/// layout 의 상·하한 attach point 가 **공유하는 단일 기준(SSOT)**. SVG/Canvas/Skia 가
/// 동일한 path·attach point 를 써서 폰트 대체에 무관하게 정합한다.
///
/// 모든 좌표는 글리프 박스(좌상단 원점, 높이 = `fs*INTEGRAL_SCALE`) 기준 상대 px 이며,
/// y 는 아래로 증가한다. 비율은 정답(한글 2022) `pdf/3-10월_교육_통합_2022.pdf` 9p 적분
/// (`∫_0^2(-2x^2+6x)dx`) 시각 정합 기준.
#[derive(Clone, Copy, Debug)]
pub(crate) struct IntegralGeom {
    /// 글리프 가로 폭(상·하 갈고리 포함, trailing pad 제외)
    pub width: f64,
    /// path 상단(상단 갈고리 끝) y
    pub top_y: f64,
    /// path 하단(하단 갈고리 끝) y
    pub bottom_y: f64,
    /// 상단 갈고리 우측 끝 x — 상한(sup) attach 기준
    pub top_hook_x: f64,
    /// 하단 갈고리 좌측 끝 x — 하한(sub) attach 기준
    pub bottom_hook_x: f64,
}

/// 글꼴 크기 `fs` 에 대한 적분 글리프 기하를 산출한다(SSOT).
pub(crate) fn integral_geom(fs: f64) -> IntegralGeom {
    let baseline = fs * INTEGRAL_SCALE * 0.7;
    let source_font_size = fs * 2.0;
    IntegralGeom {
        width: source_font_size * 630.0 / 1024.0,
        top_y: baseline + fs * AXIS_HEIGHT - source_font_size * 820.0 / 1024.0,
        bottom_y: baseline + fs * AXIS_HEIGHT + source_font_size * 202.0 / 1024.0,
        top_hook_x: source_font_size * 579.0 / 1024.0,
        bottom_hook_x: source_font_size * 50.0 / 1024.0,
    }
}

/// 기존 일반 limits 연산자의 피연산자 앞 여백(font_size 비율).
/// 정상 한컴 합은 첨자 상자와 피연산자의 논리 전진을 그대로 잇는다.
/// 그 경로에는 가로 압축을 보상하던 이 값을 적용하지 않는다.
/// SVG/Canvas는 layout이 생산한 기호·첨자 원점을 그대로 소비한다.
pub(crate) const BIG_OP_TRAIL_PAD: f64 = 0.45;
const MATRIX_COL_GAP: f64 = 0.8; // 행렬 열 간격 (font_size 비율)
const MATRIX_ROW_GAP: f64 = 0.3; // 행렬 행 간격 (font_size 비율)
/// 수식 축 높이 (TeX axis_height = 0.25em) — 분수선이 배치되는 기준 위치
pub(crate) const AXIS_HEIGHT: f64 = 0.25;
/// 텍스트 기본 baseline 비율 (상단에서 baseline까지)
const TEXT_BASELINE: f64 = 0.8;

impl EqLayout {
    pub fn new(font_size: f64) -> Self {
        Self {
            font_size,
            dpi: super::super::DEFAULT_DPI,
            hancom_cjk_advance: false,
            hancom_latin: false,
            latin_italic: true,
            hancom_math_face: false,
            hancom_eqn: false,
            math_digit_advance: None,
            variable_italic: true,
            variable_bold: false,
            source_variable_glyphs: true,
            standalone_eqalign: false,
            legacy_hft_metrics: false,
        }
    }

    /// HYhwpEQ 대체 한글의 전진폭은 한컴 수식의 논리 격자에서 반올림한다.
    /// 출력 DPI를 바꿔도 같은 물리 전진폭을 소비한다.
    pub(crate) fn for_equation(equation: &crate::model::control::Equation, dpi: f64) -> Self {
        Self {
            font_size: super::super::hwpunit_to_px(equation.font_size.max(1) as i32, dpi),
            dpi,
            hancom_cjk_advance: equation.font_name.eq_ignore_ascii_case("HYhwpEQ"),
            // 크기 고정은 개체의 예약 크기 계약이며 표시 글꼴을 바꾸지 않는다.
            hancom_latin: equation.font_name.eq_ignore_ascii_case("HYhwpEQ")
                || equation.font_name.eq_ignore_ascii_case("HancomEQN"),
            latin_italic: true,
            hancom_math_face: equation.font_name.eq_ignore_ascii_case("HYhwpEQ"),
            hancom_eqn: equation.font_name.eq_ignore_ascii_case("HancomEQN"),
            // 두 face의 수학 PUA 숫자만 같은 512/1024 전진폭이다.
            // 영문·기호는 서로 다르므로 HancomEQN을 HYhwpEQ로 치환하지 않는다.
            math_digit_advance: (equation.font_name.eq_ignore_ascii_case("HYhwpEQ")
                || equation.font_name.eq_ignore_ascii_case("HancomEQN"))
            .then_some(512),
            variable_italic: true,
            variable_bold: false,
            source_variable_glyphs: true,
            standalone_eqalign: false,
            legacy_hft_metrics: super::uses_legacy_hft_metrics(equation),
        }
    }

    /// AST를 레이아웃 박스로 변환
    pub fn layout(&self, node: &EqNode) -> LayoutBox {
        let root = Self {
            standalone_eqalign: matrix_geometry::is_standalone_eqalign(node),
            ..*self
        };
        let result = root.layout_node(node, self.font_size);
        if self.legacy_hft_metrics
            && result.width > 0.0
            && !standalone_cjk_fraction_owns_inset(node)
        {
            let mut body = result;
            let inset = self.font_size * 0.22;
            body.x = inset;
            return LayoutBox {
                font_size: self.font_size,
                x: 0.0,
                y: 0.0,
                width: body.width + inset,
                height: body.height,
                baseline: body.baseline,
                kind: LayoutKind::Row(vec![body]),
            };
        }
        // 독립 글꼴 프레임을 측정과 표시의 공통 루트에 보존한다.
        // 자식의 기준선 상대 원점은 유지하며 빈 수식을 글줄로 바꾸지 않는다.
        if (self.hancom_math_face || self.hancom_eqn) && result.height > 0.0 {
            root.matrix_row_box(result, self.font_size)
        } else {
            result
        }
    }

    fn text_width(&self, text: &str, fs: f64, italic: bool) -> f64 {
        if !self.legacy_hft_metrics {
            return estimate_text_width(text, fs, italic);
        }
        text.chars()
            .map(|ch| match ch {
                '0'..='9' => fs * 0.5,
                '=' | '+' | '-' | '%' => fs,
                '×' => fs * 0.68,
                _ => estimate_text_width(&ch.to_string(), fs, italic),
            })
            .sum()
    }

    fn layout_fraction_operand(&self, node: &EqNode, fs: f64) -> (LayoutBox, f64) {
        let mut body = self.layout_node(node, fs);
        if !self.legacy_hft_metrics || !operand_starts_cjk(node) {
            return (body, 0.0);
        }
        // 구형 수식의 한글 대체 face가 피연산자 프레임을 시작할 때의 전진이다.
        // 숫자 피연산자나 같은 줄의 뒤 한글 run에는 반복해서 넣지 않는다.
        let inset = fs * 0.22;
        body.x = inset;
        (
            LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                width: body.width + inset,
                height: body.height,
                baseline: body.baseline,
                kind: LayoutKind::Row(vec![body]),
            },
            inset,
        )
    }

    fn layout_node(&self, node: &EqNode, fs: f64) -> LayoutBox {
        match node {
            EqNode::Row(children) => self.layout_row(children, fs),
            EqNode::Text(s) => self.layout_text(s, fs),
            EqNode::Number(s) => self.layout_number(s, fs),
            EqNode::Symbol(s) => self.layout_symbol(s, fs),
            EqNode::MathSymbol(s) => self.layout_math_symbol(s, fs),
            EqNode::Function(s) => self.layout_function(s, fs),
            // 인용은 명령 해석만 막고 현재 글꼴 전환은 보존한다.
            EqNode::Quoted(s) => self.layout_literal_text(s, fs),
            EqNode::Fraction { numer, denom } => self.layout_fraction(numer, denom, fs),
            EqNode::Atop { top, bottom } => self.layout_atop(top, bottom, fs),
            EqNode::Binomial { top, bottom } => self.layout_binomial(top, bottom, fs),
            EqNode::Sqrt { index, body } => self.layout_sqrt(index, body, fs),
            EqNode::Superscript { base, sup } => self.layout_superscript(base, sup, fs),
            EqNode::Subscript { base, sub } => self.layout_subscript(base, sub, fs),
            EqNode::SubSup { base, sub, sup } => self.layout_subsup(base, sub, sup, fs),
            EqNode::LeftScripts { base, sub, sup } => {
                self.layout_left_scripts(base, sub.as_deref(), sup.as_deref(), fs)
            }
            EqNode::BigOp { symbol, sub, sup } => self.layout_big_op(symbol, sub, sup, fs),
            EqNode::Limit { is_upper, sub } => self.layout_limit(*is_upper, sub, fs),
            EqNode::Matrix { rows, style } => self.layout_matrix(rows, *style, fs),
            EqNode::Cases { rows } => self.layout_cases(rows, fs),
            EqNode::EqAlign {
                rows,
                alignment_marks,
            } => self.layout_eqalign(rows, alignment_marks, fs),
            EqNode::Rel { arrow, over, under } => self.layout_rel(arrow, over, under, fs),
            EqNode::LongDiv {
                divisor,
                quotient,
                body,
            } => self.layout_long_div(divisor, quotient, body, fs),
            EqNode::Pile { rows, align } => self.layout_pile(rows, *align, fs),
            EqNode::Paren { left, right, body } => self.layout_paren(left, right, body, fs),
            EqNode::Decoration { kind, body } => self.layout_decoration(*kind, body, fs),
            EqNode::FontStyle { style, body } => self.layout_font_style(*style, body, fs),
            EqNode::Color { r, g, b, body } => {
                let child = self.layout_node(body, fs);
                LayoutBox {
                    font_size: fs,
                    x: 0.0,
                    y: 0.0,
                    width: child.width,
                    height: child.height,
                    baseline: child.baseline,
                    kind: LayoutKind::Color {
                        rgb: [*r, *g, *b],
                        body: Box::new(child),
                    },
                }
            }
            EqNode::Space(kind) => self.layout_space(*kind, fs),
            EqNode::Newline => LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                baseline: 0.0,
                kind: LayoutKind::Newline,
            },
            EqNode::Empty => LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                baseline: 0.0,
                kind: LayoutKind::Empty,
            },
        }
    }

    fn layout_row(&self, children: &[EqNode], fs: f64) -> LayoutBox {
        if children.iter().any(|node| matches!(node, EqNode::Newline)) {
            return self.layout_newline_rows(children, fs);
        }
        if children.is_empty() {
            return LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: fs,
                baseline: fs * 0.8,
                kind: LayoutKind::Row(Vec::new()),
            };
        }

        let (source_nodes, mut boxes): (Vec<&EqNode>, Vec<LayoutBox>) = children
            .iter()
            .zip(self.row_children(children, fs))
            .filter(|(_, b)| b.width > 0.0 || matches!(b.kind, LayoutKind::Newline))
            .unzip();

        if boxes.is_empty() {
            return LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: fs,
                baseline: fs * 0.8,
                kind: LayoutKind::Row(Vec::new()),
            };
        }

        // 기준선 정렬: 가장 높은 baseline과 가장 깊은 descent
        let max_ascent = boxes.iter().map(|b| b.baseline).fold(0.0f64, f64::max);
        let max_descent = boxes
            .iter()
            .map(|b| b.height - b.baseline)
            .fold(0.0f64, f64::max);
        let total_height = max_ascent + max_descent;

        let mut x = 0.0;
        for index in 0..boxes.len() {
            if self.legacy_hft_metrics && index > 0 {
                // Glyph projection changes paint data, not the AST operator's
                // logical slot or the source-owned contextual spacing.
                x += legacy_operator_gap(source_nodes[index - 1], source_nodes[index], fs);
            }
            let b = &mut boxes[index];
            b.x = x;
            b.y = max_ascent - b.baseline;
            x += b.width;
        }

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: x,
            height: total_height,
            baseline: max_ascent,
            kind: LayoutKind::Row(boxes),
        }
    }

    fn layout_text(&self, text: &str, fs: f64) -> LayoutBox {
        if self.legacy_hft_metrics {
            if let Some(layout) =
                self.layout_hancom_glyphs(text, super::hancom_glyphs::Role::LegacyLiteral, fs, 0.0)
            {
                return layout;
            }
        }
        if let Some(layout) = self.layout_hancom_glyphs(
            text,
            super::hancom_glyphs::Role::Variable {
                italic: self.latin_italic,
            },
            fs,
            0.0,
        ) {
            return layout;
        }
        // 퍼센트가 Text 토큰으로 들어와도 수학 기호와 같은 전진·표시를 소비한다.
        if self.hancom_math_face && text == "%" {
            if let Some(result) = source_font_symbol::layout(text, fs, self.dpi) {
                return result;
            }
        }
        if self.hancom_math_face {
            if let Some(result) = source_greek_symbol::literal(text, fs, self.dpi) {
                return result;
            }
            if let Some(result) = source_font_text::layout(
                text,
                fs,
                self.dpi,
                self.variable_italic,
                self.variable_bold,
                self.source_variable_glyphs,
            ) {
                return result;
            }
        }
        // CJK/한글 텍스트는 이탤릭이 아니므로 italic 보정 제외
        let has_cjk = text.chars().any(|c| {
            matches!(c,
                '\u{3000}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' | '\u{AC00}'..='\u{D7AF}'
            )
        });
        let display_text =
            variable_style::display_text(text, self.variable_italic, self.variable_bold);
        let w = if has_cjk && self.hancom_cjk_advance {
            let advance = (fs * 48.0 / self.dpi).round().max(1.0) * self.dpi / 48.0;
            text.chars()
                .map(|ch| {
                    if super::text_has_cjk(&ch.to_string()) {
                        advance
                    } else {
                        estimate_text_width(&ch.to_string(), fs, false)
                    }
                })
                .sum()
        } else if !has_cjk {
            glyph_advances::text_width(&display_text, fs)
                .unwrap_or_else(|| estimate_text_width(text, fs, self.variable_italic))
        } else {
            self.text_width(text, fs, !has_cjk)
        };
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: w,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::Text(display_text),
        }
    }

    fn layout_number(&self, text: &str, fs: f64) -> LayoutBox {
        if let Some(layout) =
            self.layout_hancom_glyphs(text, super::hancom_glyphs::Role::Number, fs, 0.0)
        {
            return layout;
        }
        // 기본 수학 face와 HYhwpEQ의 수학 숫자는 모두 반각 전진폭을 갖는다.
        let w = if self.legacy_hft_metrics {
            self.text_width(text, fs, false)
        } else {
            text.chars()
                .map(|ch| {
                    if ch.is_ascii_digit() {
                        if let Some(advance) = self.math_digit_advance {
                            source_font_text::logical_advance(advance, fs, self.dpi)
                        } else {
                            fs * 0.5
                        }
                    } else {
                        estimate_text_width(&ch.to_string(), fs, false)
                    }
                })
                .sum()
        };
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: w,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::Number(text.to_string()),
        }
    }

    fn layout_symbol(&self, text: &str, fs: f64) -> LayoutBox {
        if let Some(layout) =
            self.layout_hancom_glyphs(text, super::hancom_glyphs::Role::Operator, fs, 0.0)
        {
            return layout;
        }
        if self.hancom_math_face && !self.legacy_hft_metrics {
            if !self.variable_bold {
                if let Some(result) =
                    source_font_symbol::layout_operator_outline(text, fs, self.dpi)
                {
                    return result;
                }
            }
            if let Some(result) = source_font_symbol::layout(text, fs, self.dpi) {
                return result;
            }
        }
        // 수식의 빼기 기호는 문장용 하이픈과 다른 글리프다.
        let display = if self.legacy_hft_metrics {
            text
        } else {
            match text {
                "-" => "−",
                // 인용 문자열과 달리 수식 기호의 작은따옴표는 미분 prime이다.
                "'" => "′",
                _ => text,
            }
        };
        let source_operator = (self.hancom_math_face && !self.legacy_hft_metrics)
            .then(|| source_font_symbol::operator_metrics(display, fs, self.dpi))
            .flatten();
        let w = if self.legacy_hft_metrics {
            self.text_width(text, fs, false)
        } else if self.hancom_math_face && matches!(text, "(" | ")") {
            source_font_advance::paren(fs, self.dpi)
        } else {
            source_operator
                .map(|(width, _)| width)
                .unwrap_or_else(|| estimate_text_width(display, fs, false))
        };
        // 연산자 좌우 여백
        let pad = if self.legacy_hft_metrics && matches!(text, "+" | "-" | "=") {
            0.0
        } else if source_operator.is_some() {
            0.0
        } else if super::symbols::is_binary_symbol(text) || matches!(text, "=" | "<" | ">") {
            fs * 0.15
        } else if matches!(text, "(" | ")" | "[" | "]" | "'" | "′" | "″" | "‴" | "/") {
            // 보통 크기의 구분자는 관계·이항 연산자 여백을 갖지 않는다.
            0.0
        } else {
            fs * 0.05
        };
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: w + pad * 2.0,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::Symbol(display.to_string()),
        }
    }

    fn layout_math_symbol(&self, text: &str, fs: f64) -> LayoutBox {
        if let Some(layout) =
            self.layout_hancom_glyphs(text, super::hancom_glyphs::Role::Operator, fs, 0.0)
        {
            return layout;
        }
        if self.hancom_eqn {
            if let Some(value) =
                self.layout_hancom_glyphs(text, super::hancom_glyphs::Role::Greek, fs, 0.0)
            {
                return value;
            }
        }
        // 독립 Print에서 그리스 기호는 라틴 rm/it/bold와 별도의 face 계약이다.
        if self.hancom_math_face {
            if let Some(result) = source_greek_symbol::command(text, fs, self.dpi) {
                return result;
            }
        }
        // 일반 수학 face는 기존 수학 알파벳의 표시 스타일을 쓴다.
        // AST와 논리 전진은 유지하고 SVG·Canvas·Skia가 소비할 글리프만 투영한다.
        let kind = || {
            if !text.is_empty() && text.chars().all(variable_style::is_greek_variable) {
                LayoutKind::Text(variable_style::display_text(
                    text,
                    self.variable_italic,
                    self.variable_bold,
                ))
            } else {
                LayoutKind::MathSymbol(text.to_string())
            }
        };
        if self.hancom_math_face {
            if let Some(width) = source_roman_advance::measured(text, fs, self.dpi) {
                return LayoutBox {
                    font_size: fs,
                    x: 0.0,
                    y: 0.0,
                    width,
                    height: fs,
                    baseline: fs * 0.8,
                    kind: kind(),
                };
            }
            // 명령으로 입력한 연산자도 직접 기호와 같은 논리 전진·표시 상자를 소비한다.
            // 독립 메트릭이 없는 다른 수학 명령은 기존 경로를 유지한다.
            if source_font_symbol::operator_metrics(text, fs, self.dpi).is_some() {
                return self.layout_symbol(text, fs);
            }
            if let Some(result) = source_font_symbol::layout(text, fs, self.dpi) {
                return result;
            }
        }
        // 적분 기호: 큰 크기로 렌더링 (INTEGRAL_SCALE 적용 — Task #1313)
        // Task #1317: 글리프를 path 로 그리므로 advance 는 path 폭(geom.width)을 쓴다.
        if is_integral_symbol(text) {
            let op_fs = fs * INTEGRAL_SCALE;
            let geom = integral_geom(fs);
            return LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                // Task #1233: 첨자 없는 bare 적분도 뒤 피연산자와 trailing 간격 유지.
                width: geom.width
                    + if self.hancom_math_face {
                        0.0
                    } else {
                        fs * BIG_OP_TRAIL_PAD
                    },
                height: op_fs,
                baseline: op_fs * 0.7, // 적분 기호 baseline: 기호 높이의 70%
                kind: kind(),
            };
        }
        let w = glyph_advances::text_width(text, fs)
            .unwrap_or_else(|| estimate_text_width(text, fs, false));
        let symbol = LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: w,
            height: fs,
            baseline: fs * 0.8,
            kind: kind(),
        };
        if super::symbols::is_relation_symbol(text) {
            // 관계 기호의 glyph와 양쪽 여백을 공통 배치 결과로 보존한다.
            let padding = fs * 0.15;
            let space = |x| LayoutBox {
                font_size: fs,
                x,
                y: 0.0,
                width: padding,
                height: fs,
                baseline: fs * 0.8,
                kind: LayoutKind::Space(padding),
            };
            let mut glyph = symbol;
            glyph.x = padding;
            LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                width: w + padding * 2.0,
                height: fs,
                baseline: fs * 0.8,
                kind: LayoutKind::Row(vec![space(0.0), glyph, space(padding + w)]),
            }
        } else {
            symbol
        }
    }

    fn layout_rel(
        &self,
        arrow: &str,
        over: &EqNode,
        under: &Option<Box<EqNode>>,
        fs: f64,
    ) -> LayoutBox {
        let small_fs = fs * 0.7;
        let gap = fs * 0.1;

        // 화살표 레이아웃
        let mut arrow_box = self.layout_node(&EqNode::MathSymbol(arrow.to_string()), fs);
        // 위 내용
        let mut over_box = self.layout_node(over, small_fs);
        // 아래 내용
        let mut under_box = under.as_ref().map(|u| self.layout_node(u, small_fs));

        // 전체 폭: 가장 넓은 요소 기준
        let max_w = arrow_box
            .width
            .max(over_box.width)
            .max(under_box.as_ref().map(|u| u.width).unwrap_or(0.0));

        // 화살표 폭을 max_w로 확장 (시각적으로 늘림)
        arrow_box.width = max_w;

        // 세로 배치: over → arrow → under
        let mut y = 0.0;
        over_box.x = (max_w - over_box.width) / 2.0;
        over_box.y = y;
        y += over_box.height + gap;

        arrow_box.x = 0.0;
        arrow_box.y = y;
        let arrow_center_y = y + arrow_box.height / 2.0;
        y += arrow_box.height + gap;

        if let Some(ref mut ub) = under_box {
            ub.x = (max_w - ub.width) / 2.0;
            ub.y = y;
            y += ub.height;
        } else {
            y -= gap; // under가 없으면 마지막 gap 제거
        }

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: max_w,
            height: y,
            baseline: arrow_center_y,
            kind: LayoutKind::Rel {
                arrow: Box::new(arrow_box),
                over: Box::new(over_box),
                under: under_box.map(Box::new),
            },
        }
    }

    fn layout_space(&self, kind: SpaceKind, fs: f64) -> LayoutBox {
        let w = match kind {
            // 한컴의 명시적 ~는 반 em을 예약한다. 정상 저장한 A~...~M의
            // 전체 전진과 독립 Print의 글리프 원점 간격이 같은 결과를 쓴다.
            SpaceKind::Normal if self.hancom_math_face || self.hancom_eqn => fs * 0.5,
            SpaceKind::Normal => fs * 0.33,
            SpaceKind::Thin if self.hancom_math_face => {
                // 한컴의 명시적 Thin은 BaseUnit/8을 정수 HWPUNIT로 게시한다.
                let size = super::font_frame_metrics::base_unit_at_dpi(fs, self.dpi);
                super::super::hwpunit_to_px((size / 8) as i32, self.dpi)
            }
            SpaceKind::Thin => fs * 0.17,
            SpaceKind::Tab => fs * 1.0,
        };
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: w,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::Space(w),
        }
    }
}

/// 적분 기호 여부 판별
pub(crate) fn is_integral_symbol(symbol: &str) -> bool {
    matches!(symbol, "∫" | "∬" | "∭" | "∮" | "∯" | "∰")
}

fn operand_starts_cjk(node: &EqNode) -> bool {
    match node {
        EqNode::Text(text) => text
            .chars()
            .next()
            .is_some_and(|ch| super::text_has_cjk(&ch.to_string())),
        EqNode::Row(children) => children.first().is_some_and(operand_starts_cjk),
        EqNode::FontStyle { body, .. } | EqNode::Color { body, .. } => operand_starts_cjk(body),
        _ => false,
    }
}

// A standalone CJK fraction already introduces the fallback face in its
// numerator. In a larger expression the outer row has its own registration.
// Independent 9/12/18pt Print controls distinguish the two expression scopes.
fn standalone_cjk_fraction_owns_inset(node: &EqNode) -> bool {
    match node {
        EqNode::Fraction { numer, .. } => operand_starts_cjk(numer),
        EqNode::Row(children) if children.len() == 1 => {
            standalone_cjk_fraction_owns_inset(&children[0])
        }
        EqNode::FontStyle { body, .. } | EqNode::Color { body, .. } => {
            standalone_cjk_fraction_owns_inset(body)
        }
        _ => false,
    }
}

fn legacy_operator(node: &EqNode) -> Option<&str> {
    match node {
        EqNode::Symbol(symbol) | EqNode::MathSymbol(symbol)
            if matches!(symbol.as_str(), "=" | "+" | "-" | "×") =>
        {
            Some(symbol)
        }
        EqNode::FontStyle { body, .. } | EqNode::Color { body, .. } => legacy_operator(body),
        _ => None,
    }
}

fn legacy_operator_gap(previous: &EqNode, current: &EqNode, fs: f64) -> f64 {
    let before = if legacy_operator(previous).is_none() {
        match legacy_operator(current) {
            Some("=") => 0.14,
            Some("-") => 0.11,
            Some("+") => 0.10,
            _ => 0.0,
        }
    } else {
        0.0
    };
    let after = match legacy_operator(previous) {
        Some("=") => 0.17,
        Some("-") => 0.11,
        Some("+") => 0.10,
        _ => 0.0,
    };
    fs * (before + after)
}

fn legacy_delimiter_width(delimiter: &str, fs: f64) -> Option<f64> {
    match delimiter {
        "(" | ")" => Some(fs * 0.375),
        "[" | "]" => Some(fs * 0.387),
        _ => None,
    }
}

/// 텍스트 폭 추정
pub(crate) fn estimate_text_width(text: &str, font_size: f64, italic: bool) -> f64 {
    // 일반 기호·함수·극한·큰 연산자도 paint와 같은 표시 글리프의 전진폭을 쓴다.
    // font_line에서만 실제 메트릭을 쓰고 여기서 다시 문자 종류별 비율로
    // 추정하면 등호 같은 기호가 자기 상자를 넘어 뒤 내용과 겹친다.
    if !italic {
        if let Some(width) = glyph_advances::text_width(text, font_size) {
            return width;
        }
    }
    let mut w = 0.0;
    for ch in text.chars() {
        let ratio = if matches!(ch, '(' | ')') {
            // 함께 제공하는 Latin Modern Math의 보통 괄호 전진폭: UPM 1000, 389 단위.
            0.389
        } else if ch == '−' {
            0.778
        } else if ch.is_ascii() {
            if ch.is_ascii_uppercase() {
                0.65
            } else if ch.is_ascii_lowercase() {
                0.55
            } else if ch.is_ascii_digit() {
                0.55
            } else {
                0.5
            }
        } else {
            estimate_unicode_char_width(ch)
        };
        w += font_size * ratio;
    }
    if italic {
        w *= 1.05;
    }
    w
}

/// 비-ASCII 문자의 폭 비율 추정 (font_size 대비)
fn estimate_unicode_char_width(ch: char) -> f64 {
    match ch {
        // 프라임/아포스트로피 — 매우 좁음
        '′' | '″' | '‴' | '\'' | '`' => 0.3,
        // 그리스 소문자 — 일반 라틴 소문자와 유사
        'α'..='ω' | 'ϑ' | 'ϖ' => 0.55,
        // 그리스 대문자 — 일반 라틴 대문자와 유사
        'Α'..='Ω' | 'ϒ' => 0.65,
        // 수학 연산자 — 중간 너비
        '±' | '∓' | '×' | '÷' | '·' | '∘' | '†' | '‡' | '•' => 0.6,
        // 관계 기호 — 등호 너비와 유사
        '≠' | '≤' | '≥' | '≈' | '≡' | '≅' | '∼' | '≃' | '≍' | '≐' | '∝' | '≺' | '≻' => {
            0.7
        }
        // 집합/논리 기호
        '∈' | '∉' | '∋' | '⊂' | '⊃' | '⊆' | '⊇' | '∀' | '∃' | '¬' | '∧' | '∨' => {
            0.65
        }
        '⊏' | '⊐' | '⊑' | '⊒' | '⊻' | '⊢' | '⊣' | '⊨' => 0.65,
        // 큰 연산자 기호 (단독 텍스트로 사용될 때)
        '∫' | '∬' | '∭' | '∮' | '∯' | '∰' => 0.5,
        '∑' | '∏' | '∐' => 0.8,
        '∪' | '∩' | '⊔' | '⊓' | '⊎' | '⋀' | '⋁' => 0.7,
        '⊕' | '⊗' | '⊙' | '⊖' | '⊘' => 0.7,
        // 화살표
        '←' | '→' | '↑' | '↓' | '↔' | '↕' => 0.8,
        '⇐' | '⇒' | '⇑' | '⇓' | '⇔' | '⇕' => 0.8,
        '↖' | '↗' | '↙' | '↘' | '↦' | '↩' | '↪' => 0.8,
        // 점 기호
        '⋯' | '…' | '⋮' | '⋱' => 0.8,
        // 기타 수학 기호 — 좁은 것
        '∂' | '∅' | '∇' | '∞' | '∠' | '∡' | '∢' | '⊾' => 0.6,
        // 실제 수학 face의 ⊥/⊤ advance는 약0.78em이며 좁은 기호와 다르다.
        '⊥' | '⊤' => 0.8,
        '°' | '‰' | '‱' | '♯' => 0.5,
        'ℵ' | 'ℏ' | 'ı' | 'ȷ' | 'ℓ' | '℘' | 'ℑ' | 'ℜ' | 'ℒ' | 'Å' | '℧' => 0.6,
        '℃' | '℉' => 0.9,
        '△' | '▽' | '○' | '◇' | '⋄' => 0.7,
        // CJK — 전각
        '\u{3000}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' | '\u{AC00}'..='\u{D7AF}' => 1.0,
        // 기타 비-ASCII — 중간 너비 기본값
        _ => 0.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::equation::parser::EqParser;
    use crate::renderer::equation::tokenizer::tokenize;

    fn parse_and_layout(script: &str, font_size: f64) -> LayoutBox {
        let tokens = tokenize(script);
        let ast = EqParser::new(tokens).parse();
        EqLayout::new(font_size).layout(&ast)
    }

    #[test]
    fn test_simple_text() {
        let lb = parse_and_layout("abc", 20.0);
        assert!(lb.width > 0.0);
        assert!(lb.height > 0.0);
    }

    #[test]
    fn test_fraction_layout() {
        let lb = parse_and_layout("a over b", 20.0);
        assert!(lb.width > 0.0);
        assert!(lb.height > 20.0); // 분수는 기본 높이보다 높아야 함
    }

    /// Task #1233: 큰 연산자(Σ)는 box width 에 trailing 간격(fs×BIG_OP_TRAIL_PAD)을 포함해야
    /// 피연산자가 붙지 않는다.
    #[test]
    fn test_big_op_trailing_pad() {
        // 트리 어디서든 첫 BigOp LayoutBox 를 찾는다 (top-level Row/단독 모두 대응)
        fn find_big_op(b: &LayoutBox) -> Option<&LayoutBox> {
            if matches!(b.kind, LayoutKind::BigOp { .. }) {
                return Some(b);
            }
            if let LayoutKind::Row(children) = &b.kind {
                for c in children {
                    if let Some(f) = find_big_op(c) {
                        return Some(f);
                    }
                }
            }
            None
        }
        let fs = 20.0;
        let lb = parse_and_layout("sum_{n=1}^{N} b", fs);
        let big = find_big_op(&lb).expect("BigOp 노드가 있어야 함");
        // 내부(중앙정렬된 sub/sup)의 우측 끝
        let inner = match &big.kind {
            LayoutKind::BigOp { sub, sup, .. } => {
                let s = sub.as_ref().map(|b| b.x + b.width).unwrap_or(0.0);
                let p = sup.as_ref().map(|b| b.x + b.width).unwrap_or(0.0);
                s.max(p)
            }
            _ => unreachable!(),
        };
        assert!(
            big.width - inner >= fs * BIG_OP_TRAIL_PAD * 0.9,
            "BigOp trailing 간격 부재: width={} inner={}",
            big.width,
            inner
        );
    }

    #[test]
    fn test_superscript_layout() {
        let lb = parse_and_layout("x^2", 20.0);
        assert!(lb.width > 0.0);
        assert!(lb.height > 0.0);
    }

    #[test]
    fn test_superscript_tall_base_no_overshoot() {
        // 독립 한컴 Print의 짧은 본체와 분수 모두 첨자 기준선은 본체보다 높다.
        // 본체 상단보다 첨자가 높으면 안 된다는 옛 조건은 정상 위첨자까지 거절한다.
        fn find_sup(lb: &LayoutBox) -> Option<(&LayoutBox, &LayoutBox)> {
            match &lb.kind {
                LayoutKind::Superscript { base, sup } => Some((base, sup)),
                LayoutKind::Row(ch) => ch.iter().rev().find_map(find_sup),
                _ => None,
            }
        }
        for script in ["LEFT ( {1} over {6} RIGHT )^4", "x^4"] {
            let layout = parse_and_layout(script, 12.0);
            let (base, sup) = find_sup(&layout).expect("본체와 위첨자 소유");
            let base_baseline = base.y + base.baseline;
            let sup_baseline = sup.y + sup.baseline;
            assert!(
                sup_baseline < base_baseline,
                "위첨자 기준선은 본체보다 높아야 함: {script}"
            );
            assert!(
                sup.x >= base.x + base.width,
                "위첨자는 본체 뒤에 배치되어야 함"
            );
            let relative_error = (layout.baseline - base_baseline).abs() / layout.height;
            assert!(
                relative_error < 1e-9,
                "합성 기준선은 본체 기준선을 한 번만 소비해야 함"
            );
            assert!(
                base.y >= 0.0 && sup.y >= 0.0,
                "자식 상단은 합성 프레임 안에 있어야 함"
            );
            assert!(base.y + base.height <= layout.height, "본체 점유 영역 보존");
            assert!(sup.y + sup.height <= layout.height, "위첨자 점유 영역 보존");
        }
    }

    #[test]
    fn test_superscript_fraction_baseline() {
        // #532: 분수형 위첨자 (25^{1/3}) 에서 sup의 baseline이
        // base baseline 아래로 내려가면 안 됨
        let lb = parse_and_layout("25^{{1} over {3}}", 14.0);
        let (base_box, sup_box) = match &lb.kind {
            LayoutKind::Superscript { base, sup } => (base, sup),
            LayoutKind::Row(children) => {
                // Row 내 마지막 요소가 Superscript일 수 있음
                let last = children.last().unwrap();
                match &last.kind {
                    LayoutKind::Superscript { base, sup } => (base, sup),
                    _ => panic!("Expected Superscript in Row"),
                }
            }
            _ => panic!("Expected Superscript or Row, got {:?}", lb.kind),
        };
        // sup의 상단(y)이 base의 상단보다 높거나 같아야 함
        assert!(
            sup_box.y <= base_box.y,
            "sup.y ({}) should be <= base.y ({})",
            sup_box.y,
            base_box.y
        );
        // sup의 baseline이 base baseline보다 위에 있어야 함
        let sup_baseline_abs = sup_box.y + sup_box.baseline;
        let base_baseline_abs = base_box.y + base_box.baseline;
        assert!(
            sup_baseline_abs < base_baseline_abs,
            "sup baseline ({}) should be above base baseline ({})",
            sup_baseline_abs,
            base_baseline_abs
        );
    }

    #[test]
    fn test_eq01_script() {
        // 실제 eq-01.hwp 수식
        let lb = parse_and_layout(
            "평점=입찰가격평가~배점한도 TIMES LEFT ( {최저입찰가격} over {해당입찰가격} RIGHT )",
            20.0,
        );
        assert!(lb.width > 100.0);
        assert!(lb.height > 0.0);
    }

    #[test]
    fn test_cases_korean_no_overlap() {
        // exam_math.hwp p177 CASES 수식 — 한글 혼합
        let lb = parse_and_layout(
            "a _{n+1} = {cases{``a _{n} -3&&LEFT ( LEFT |` a _{n} `RIGHT | 이~홀수인~경우 RIGHT )#``{1} over {2} a _{n}&&LEFT ( a _{n} =0~또는~ LEFT |` a _{n} `RIGHT | 이~짝수인~경우 RIGHT )}}",
            14.67,
        );
        assert!(lb.width > 0.0, "CASES width should be positive");
        assert!(lb.height > 0.0, "CASES height should be positive");

        // 전체 수식 a_{n+1} = {cases{...}} 는 Row[subscript, =, Paren{cases}]
        let top_children = match &lb.kind {
            LayoutKind::Row(children) => children,
            other => panic!("Top-level should be Row, got {:?}", other),
        };
        let cases_paren = top_children
            .iter()
            .find(|c| matches!(&c.kind, LayoutKind::Paren { .. }))
            .expect("Should contain a Paren (CASES) element");
        let cases_body = match &cases_paren.kind {
            LayoutKind::Paren { body, .. } => body,
            _ => unreachable!(),
        };
        let rows = match &cases_body.kind {
            LayoutKind::Row(rows) => rows,
            other => panic!("CASES body should be Row, got {:?}", other),
        };
        assert!(rows.len() >= 2, "CASES should have at least 2 rows");
        let row1 = &rows[0];
        let row2 = &rows[1];
        let row1_bottom = row1.y + row1.height;
        let row2_top = row2.y;
        assert!(
            row2_top >= row1_bottom,
            "CASES rows should not overlap: row1 bottom={:.1}, row2 top={:.1}",
            row1_bottom,
            row2_top
        );
    }

    #[test]
    fn test_korean_text_width_not_italic() {
        // 한글 텍스트는 이탤릭 보정 없이 폭 산출
        let korean = parse_and_layout("홀수인~경우", 20.0);
        let latin = parse_and_layout("abcdef", 20.0);
        // 한글 6자(전각 1.0×) > 라틴 6자(~0.55×)
        assert!(
            korean.width > latin.width,
            "Korean text width ({:.1}) should be larger than Latin ({:.1})",
            korean.width,
            latin.width
        );
    }
}
