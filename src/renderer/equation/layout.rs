//! 수식 레이아웃 엔진
//!
//! AST(EqNode)를 레이아웃 박스(LayoutBox)로 변환하여
//! 각 요소의 크기와 위치를 계산한다.

use super::ast::*;

/// 수식 레이아웃 박스
#[derive(Debug, Clone, serde::Serialize)]
pub struct LayoutBox {
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
    /// 함수 이름 (로만체)
    Function(String),
    /// 분수
    Fraction {
        numer: Box<LayoutBox>,
        denom: Box<LayoutBox>,
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
        sub: Option<Box<LayoutBox>>,
        sup: Option<Box<LayoutBox>>,
    },
    /// 극한
    Limit {
        is_upper: bool,
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
        body: Box<LayoutBox>,
    },
    /// 장식
    Decoration {
        kind: super::symbols::DecoKind,
        body: Box<LayoutBox>,
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

/// 수식 레이아웃 계산기
pub struct EqLayout {
    /// 기본 글꼴 크기 (px)
    pub font_size: f64,
}

/// 비율 상수
pub(crate) const SCRIPT_SCALE: f64 = 0.7; // 첨자 크기 비율
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
    /// 줄기 stroke 두께
    pub stroke_w: f64,
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
    let h = fs * INTEGRAL_SCALE;
    IntegralGeom {
        width: fs * 0.52,
        stroke_w: fs * 0.06,
        top_y: h * 0.04,
        bottom_y: h * 0.96,
        top_hook_x: fs * 0.50,
        bottom_hook_x: fs * 0.04,
    }
}

/// 큰 연산자(Σ/∏/∫) 뒤 피연산자와의 trailing 간격 (font_size 비율) — Task #1233.
/// layout_row 는 형제를 간격 0으로 붙이므로, 큰 연산자 box width 에 이 trailing 공백을
/// 더해 피연산자가 연산자에 붙지 않게 한다(TeX thin/med space 관례, 한컴 PDF 정합).
///
/// 인라인 수식은 svg.rs 에서 컨트롤 advance(tac_w)로 가로 스케일(scale_x = tac_w/자연폭)
/// 되므로, 자연폭에 더한 pad 는 scale_x 만큼 줄어 렌더된다. 분수·괄호를 포함한 큰 수식은
/// scale_x 가 작아(0.6~0.9) pad 가 약화되므로, 압축 후에도 충분한 간격이 남도록 0.45 로
/// 둔다(작업지시자 시각 판정 — 0.25 는 부족, 0.45 가 적정).
///
/// 렌더러(svg_render/canvas_render)는 limits 연산자를 `max_w = lb.width - fs*PAD` 에
/// 중앙정렬하므로(이 const 참조), pad 전체가 **순수 trailing** 이 되고 연산자는 첨자와 정렬된다.
pub(crate) const BIG_OP_TRAIL_PAD: f64 = 0.45;
const MATRIX_COL_GAP: f64 = 0.8; // 행렬 열 간격 (font_size 비율)
const MATRIX_ROW_GAP: f64 = 0.3; // 행렬 행 간격 (font_size 비율)
/// 수식 축 높이 (TeX axis_height = 0.25em) — 분수선이 배치되는 기준 위치
pub(crate) const AXIS_HEIGHT: f64 = 0.25;
/// 텍스트 기본 baseline 비율 (상단에서 baseline까지)
const TEXT_BASELINE: f64 = 0.8;

impl EqLayout {
    pub fn new(font_size: f64) -> Self {
        Self { font_size }
    }

    /// AST를 레이아웃 박스로 변환
    pub fn layout(&self, node: &EqNode) -> LayoutBox {
        self.layout_node(node, self.font_size)
    }

    fn layout_node(&self, node: &EqNode, fs: f64) -> LayoutBox {
        match node {
            EqNode::Row(children) => self.layout_row(children, fs),
            EqNode::Text(s) => self.layout_text(s, fs),
            EqNode::Number(s) => self.layout_number(s, fs),
            EqNode::Symbol(s) => self.layout_symbol(s, fs),
            EqNode::MathSymbol(s) => self.layout_math_symbol(s, fs),
            EqNode::Function(s) => self.layout_function(s, fs),
            EqNode::Quoted(s) => self.layout_number(s, fs),
            EqNode::Fraction { numer, denom } => self.layout_fraction(numer, denom, fs),
            EqNode::Atop { top, bottom } => self.layout_atop(top, bottom, fs),
            EqNode::Sqrt { index, body } => self.layout_sqrt(index, body, fs),
            EqNode::Superscript { base, sup } => self.layout_superscript(base, sup, fs),
            EqNode::Subscript { base, sub } => self.layout_subscript(base, sub, fs),
            EqNode::SubSup { base, sub, sup } => self.layout_subsup(base, sub, sup, fs),
            EqNode::BigOp { symbol, sub, sup } => self.layout_big_op(symbol, sub, sup, fs),
            EqNode::Limit { is_upper, sub } => self.layout_limit(*is_upper, sub, fs),
            EqNode::Matrix { rows, style } => self.layout_matrix(rows, *style, fs),
            EqNode::Cases { rows } => self.layout_cases(rows, fs),
            EqNode::EqAlign { rows } => self.layout_eqalign(rows, fs),
            EqNode::Rel { arrow, over, under } => self.layout_rel(arrow, over, under, fs),
            EqNode::Pile { rows, align } => self.layout_pile(rows, *align, fs),
            EqNode::Paren { left, right, body } => self.layout_paren(left, right, body, fs),
            EqNode::Decoration { kind, body } => self.layout_decoration(*kind, body, fs),
            EqNode::FontStyle { style, body } => self.layout_font_style(*style, body, fs),
            EqNode::Color { body, .. } => self.layout_node(body, fs),
            EqNode::Space(kind) => self.layout_space(*kind, fs),
            EqNode::Newline => LayoutBox {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                baseline: 0.0,
                kind: LayoutKind::Newline,
            },
            EqNode::Empty => LayoutBox {
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
        if children.is_empty() {
            return LayoutBox {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: fs,
                baseline: fs * 0.8,
                kind: LayoutKind::Row(Vec::new()),
            };
        }

        let mut boxes: Vec<LayoutBox> = children
            .iter()
            .map(|c| self.layout_node(c, fs))
            .filter(|b| b.width > 0.0 || matches!(b.kind, LayoutKind::Newline))
            .collect();

        if boxes.is_empty() {
            return LayoutBox {
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
        for b in &mut boxes {
            b.x = x;
            b.y = max_ascent - b.baseline;
            x += b.width;
        }

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: x,
            height: total_height,
            baseline: max_ascent,
            kind: LayoutKind::Row(boxes),
        }
    }

    fn layout_text(&self, text: &str, fs: f64) -> LayoutBox {
        // CJK/한글 텍스트는 이탤릭이 아니므로 italic 보정 제외
        let has_cjk = text.chars().any(|c| {
            matches!(c,
                '\u{3000}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' | '\u{AC00}'..='\u{D7AF}'
            )
        });
        let w = estimate_text_width(text, fs, !has_cjk);
        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: w,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::Text(text.to_string()),
        }
    }

    fn layout_number(&self, text: &str, fs: f64) -> LayoutBox {
        let w = estimate_text_width(text, fs, false);
        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: w,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::Number(text.to_string()),
        }
    }

    fn layout_symbol(&self, text: &str, fs: f64) -> LayoutBox {
        let w = estimate_text_width(text, fs, false);
        // 연산자 좌우 여백
        let pad = if matches!(text, "+" | "-" | "=" | "<" | ">" | "×" | "÷") {
            fs * 0.15
        } else {
            fs * 0.05
        };
        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: w + pad * 2.0,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::Symbol(text.to_string()),
        }
    }

    fn layout_math_symbol(&self, text: &str, fs: f64) -> LayoutBox {
        // 적분 기호: 큰 크기로 렌더링 (INTEGRAL_SCALE 적용 — Task #1313)
        // Task #1317: 글리프를 path 로 그리므로 advance 는 path 폭(geom.width)을 쓴다.
        if is_integral_symbol(text) {
            let op_fs = fs * INTEGRAL_SCALE;
            let geom = integral_geom(fs);
            return LayoutBox {
                x: 0.0,
                y: 0.0,
                // Task #1233: 첨자 없는 bare 적분도 뒤 피연산자와 trailing 간격 유지.
                width: geom.width + fs * BIG_OP_TRAIL_PAD,
                height: op_fs,
                baseline: op_fs * 0.7, // 적분 기호 baseline: 기호 높이의 70%
                kind: LayoutKind::MathSymbol(text.to_string()),
            };
        }
        let w = estimate_text_width(text, fs, false);
        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: w,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::MathSymbol(text.to_string()),
        }
    }

    fn layout_function(&self, name: &str, fs: f64) -> LayoutBox {
        let w = estimate_text_width(name, fs, false);
        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: w + fs * 0.02,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::Function(name.to_string()),
        }
    }

    fn layout_fraction(&self, numer: &EqNode, denom: &EqNode, fs: f64) -> LayoutBox {
        let n = self.layout_node(numer, fs);
        let d = self.layout_node(denom, fs);

        let pad = fs * FRAC_LINE_PAD;
        let line_thick = fs * FRAC_LINE_THICK;
        let axis = fs * AXIS_HEIGHT;
        let w = n.width.max(d.width) + pad * 2.0;

        let numer_h = n.height + pad;
        let denom_h = d.height + pad;

        // TeX 방식: 분수선은 baseline에서 axis_height 위에 배치
        // baseline(상단에서) = 분자높이 + 분수선두께/2 + axis_height
        // 즉, 분수선 y = baseline - axis_height (상단 기준)
        let frac_line_from_top = numer_h + line_thick / 2.0;
        let baseline = frac_line_from_top + axis;
        let total_h = numer_h + line_thick + denom_h;

        let mut n_box = n;
        n_box.x = (w - n_box.width) / 2.0;
        n_box.y = pad;

        let mut d_box = d;
        d_box.x = (w - d_box.width) / 2.0;
        d_box.y = numer_h + line_thick;

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: w,
            height: total_h,
            baseline,
            kind: LayoutKind::Fraction {
                numer: Box::new(n_box),
                denom: Box::new(d_box),
            },
        }
    }

    fn layout_atop(&self, top: &EqNode, bottom: &EqNode, fs: f64) -> LayoutBox {
        let t = self.layout_node(top, fs);
        let b = self.layout_node(bottom, fs);

        let pad = fs * FRAC_LINE_PAD;
        let axis = fs * AXIS_HEIGHT;
        let w = t.width.max(b.width) + pad * 2.0;

        let top_h = t.height + pad;
        let bottom_h = b.height + pad;
        let baseline = top_h + axis;
        let total_h = top_h + bottom_h;

        let mut top_box = t;
        top_box.x = (w - top_box.width) / 2.0;
        top_box.y = pad;

        let mut bottom_box = b;
        bottom_box.x = (w - bottom_box.width) / 2.0;
        bottom_box.y = top_h;

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: w,
            height: total_h,
            baseline,
            kind: LayoutKind::Atop {
                top: Box::new(top_box),
                bottom: Box::new(bottom_box),
            },
        }
    }

    fn layout_sqrt(&self, index: &Option<Box<EqNode>>, body: &EqNode, fs: f64) -> LayoutBox {
        let b = self.layout_node(body, fs);
        let pad = fs * SQRT_PAD;
        let sign_w = fs * 0.6; // √ 기호 폭
        let body_w = b.width + pad;
        let body_h = b.height + pad * 2.0;

        let idx = index.as_ref().map(|i| {
            let mut ib = self.layout_node(i, fs * SCRIPT_SCALE);
            ib.x = 0.0;
            ib.y = 0.0;
            ib
        });
        let idx_w = idx.as_ref().map(|i| i.width).unwrap_or(0.0);
        let total_w = idx_w.max(sign_w * 0.5) + sign_w * 0.5 + body_w;

        let mut body_box = b;
        body_box.x = total_w - body_w + pad * 0.5;
        body_box.y = pad;

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: total_w,
            height: body_h,
            baseline: body_box.y + body_box.baseline,
            kind: LayoutKind::Sqrt {
                index: idx.map(Box::new),
                body: Box::new(body_box),
            },
        }
    }

    fn layout_superscript(&self, base: &EqNode, sup: &EqNode, fs: f64) -> LayoutBox {
        let b = self.layout_node(base, fs);
        let s = self.layout_node(sup, fs * SCRIPT_SCALE);

        // sup_shift: 기준선으로부터 위첨자 상단까지의 거리 (양수 = base 상단 아래)
        let sup_shift = b.baseline - s.height * 0.7;

        let (base_y, sup_y, total_h);
        if sup_shift >= 0.0 {
            // [Task #1300] 위첨자 상단을 base 상단에 맞춘다 (한컴 정합).
            // 기존엔 base 를 sup_shift(=b.baseline 비례) 만큼 아래로 밀어 위첨자를
            // 박스 최상단에 두었는데, 키 큰 base(괄호 분수 등)에서 위첨자가 base 상단
            // 위로 과하게 치솟아 윗줄을 침범했다(#1300). base 를 밀지 않고(상단 정렬)
            // 위첨자 상단이 base 상단과 같은 높이에 오도록 한다.
            // base 가 sup 보다 낮은 경우만 sup 를 담도록 base 를 내린다.
            sup_y = 0.0;
            base_y = (s.height - b.height).max(0.0);
            total_h = (base_y + b.height).max(s.height);
        } else {
            // 위첨자가 base 상단 위로 확장 — sup를 상단에, base를 |sup_shift|만큼 내림
            sup_y = 0.0;
            base_y = -sup_shift;
            total_h = (base_y + b.height).max(s.height);
        }

        let mut base_box = b;
        base_box.x = 0.0;
        base_box.y = base_y;

        let mut sup_box = s;
        sup_box.x = base_box.width;
        sup_box.y = sup_y;

        let total_w = base_box.width + sup_box.width;

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: total_w,
            height: total_h,
            baseline: base_box.y + base_box.baseline,
            kind: LayoutKind::Superscript {
                base: Box::new(base_box),
                sup: Box::new(sup_box),
            },
        }
    }

    fn layout_subscript(&self, base: &EqNode, sub: &EqNode, fs: f64) -> LayoutBox {
        let b = self.layout_node(base, fs);
        let s = self.layout_node(sub, fs * SCRIPT_SCALE);

        let sub_shift = b.baseline * 0.4;
        let total_h = (b.height).max(sub_shift + s.height);

        let mut base_box = b;
        base_box.x = 0.0;
        base_box.y = 0.0;

        let mut sub_box = s;
        sub_box.x = base_box.width;
        sub_box.y = sub_shift;

        let total_w = base_box.width + sub_box.width;

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: total_w,
            height: total_h,
            baseline: base_box.baseline,
            kind: LayoutKind::Subscript {
                base: Box::new(base_box),
                sub: Box::new(sub_box),
            },
        }
    }

    fn layout_subsup(&self, base: &EqNode, sub: &EqNode, sup: &EqNode, fs: f64) -> LayoutBox {
        // 적분 기호: 상한은 기호 상단, 하한은 기호 하단에 맞춤
        let is_integral = matches!(base, EqNode::MathSymbol(s) if is_integral_symbol(s));

        let b = self.layout_node(base, fs);
        let sb = self.layout_node(sub, fs * SCRIPT_SCALE);
        let sp = self.layout_node(sup, fs * SCRIPT_SCALE);

        if is_integral {
            // 적분 전용 배치 (Task #1317): 글리프를 stroke path 로 그리므로 상·하한 attach
            // point 를 path 기하(`integral_geom`)에 맞춰 산출한다. SVG/Canvas/Skia 가 동일
            // geom 을 공유하여 폰트 대체에 무관하게 정합한다(정답 한글 2022 비례).
            //   - 상한(sup): 상단 갈고리 우측, 글리프 최상단 근처
            //   - 하한(sub): 하단 갈고리 우측, 글리프 최하단(=baseline 근처)
            let geom = integral_geom(fs);
            // 상·하한이 적분 줄기에 붙지 않도록 **가로 간격(gap_x)만** 키워 띄운다.
            // 세로로 벌리면 적분 박스 높이가 커져 줄 간격이 위/아래로 넓어지므로
            // 세로 위치는 컴팩트하게 유지한다 (작업지시자 피드백, Task #1317 v4).
            let gap_x = fs * 0.22;

            let mut base_box = b;
            base_box.x = 0.0;
            base_box.width = geom.width; // 글리프 advance = path 폭

            // 글리프 기준 첨자 세로 위치(박스 상단 원점) — 컴팩트(글리프 상·하단 근처).
            let sup_dy = geom.top_y - sp.height * 0.30; // 상한: 최상단 근처
            let sub_dy = geom.bottom_y - sb.height * 0.72; // 하한: 최하단 근처

            // 상한이 박스 위로 넘치지 않도록 글리프를 그만큼 아래로 내린다.
            let head = (-sup_dy).max(0.0);
            base_box.y = head;

            let mut sup_box = sp;
            sup_box.x = base_box.x + geom.top_hook_x + gap_x;
            sup_box.y = base_box.y + sup_dy;

            let mut sub_box = sb;
            sub_box.x = base_box.x + geom.bottom_hook_x + gap_x;
            sub_box.y = base_box.y + sub_dy;

            let right = (sup_box.x + sup_box.width)
                .max(sub_box.x + sub_box.width)
                .max(base_box.x + geom.width);
            let total_w = right + fs * BIG_OP_TRAIL_PAD;
            let total_h = (sub_box.y + sub_box.height)
                .max(base_box.y + base_box.height)
                .max(sup_box.y + sup_box.height);

            return LayoutBox {
                x: 0.0,
                y: 0.0,
                width: total_w,
                height: total_h,
                baseline: base_box.y + base_box.baseline,
                kind: LayoutKind::SubSup {
                    base: Box::new(base_box),
                    sub: Box::new(sub_box),
                    sup: Box::new(sup_box),
                },
            };
        }

        let sup_shift = b.baseline - sp.height * 0.7;
        let sub_shift = b.baseline * 0.4;

        let ascent = if sup_shift < 0.0 {
            sp.height - sup_shift.abs()
        } else {
            sp.height.max(0.0)
        };
        let top = sup_shift.min(0.0).abs();
        let total_h = (top + b.height)
            .max(top + sub_shift + sb.height)
            .max(ascent + b.height);

        let base_y = top.max(
            if sup_shift > 0.0 {
                0.0
            } else {
                sp.height - sup_shift.abs() - b.baseline
            }
            .max(0.0),
        );

        let mut base_box = b;
        base_box.x = 0.0;
        base_box.y = base_y;

        let mut sup_box = sp;
        sup_box.x = base_box.width;
        sup_box.y = 0.0;

        let mut sub_box = sb;
        sub_box.x = base_box.width;
        sub_box.y = base_y + sub_shift;

        let script_w = sup_box.width.max(sub_box.width);
        let total_w = base_box.width + script_w;

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: total_w,
            height: total_h
                .max(base_box.y + base_box.height)
                .max(sub_box.y + sub_box.height),
            baseline: base_box.y + base_box.baseline,
            kind: LayoutKind::SubSup {
                base: Box::new(base_box),
                sub: Box::new(sub_box),
                sup: Box::new(sup_box),
            },
        }
    }

    fn layout_big_op(
        &self,
        symbol: &str,
        sub: &Option<Box<EqNode>>,
        sup: &Option<Box<EqNode>>,
        fs: f64,
    ) -> LayoutBox {
        // 적분 기호: nolimits 스타일 (큰 기호 + 오른쪽 위/아래 첨자)
        if is_integral_symbol(symbol) {
            return self.layout_integral(symbol, sub, sup, fs);
        }
        // ∑, ∏ 등: limits 스타일 (위/아래 중앙)
        let op_fs = fs * BIG_OP_SCALE;
        let op_w = estimate_text_width(symbol, op_fs, false);
        let op_h = op_fs;

        let sub_box = sub.as_ref().map(|s| self.layout_node(s, fs * SCRIPT_SCALE));
        let sup_box = sup.as_ref().map(|s| self.layout_node(s, fs * SCRIPT_SCALE));

        let sup_h = sup_box
            .as_ref()
            .map(|b| b.height + fs * 0.05)
            .unwrap_or(0.0);
        let sub_h = sub_box
            .as_ref()
            .map(|b| b.height + fs * 0.05)
            .unwrap_or(0.0);

        let total_h = sup_h + op_h + sub_h;
        let max_w = [
            op_w,
            sub_box.as_ref().map(|b| b.width).unwrap_or(0.0),
            sup_box.as_ref().map(|b| b.width).unwrap_or(0.0),
        ]
        .iter()
        .copied()
        .fold(0.0f64, f64::max);

        let baseline = sup_h + op_h * 0.6;

        let sup_laid = sup_box.map(|mut b| {
            b.x = (max_w - b.width) / 2.0;
            b.y = 0.0;
            b
        });
        let sub_laid = sub_box.map(|mut b| {
            b.x = (max_w - b.width) / 2.0;
            b.y = sup_h + op_h;
            b
        });

        LayoutBox {
            x: 0.0,
            y: 0.0,
            // Task #1233: 피연산자가 연산자에 붙지 않도록 trailing 간격 추가.
            // sup/sub 중앙정렬은 max_w 기준 유지 → 연산자는 좌측, 우측에 순수 공백.
            width: max_w + fs * BIG_OP_TRAIL_PAD,
            height: total_h,
            baseline,
            kind: LayoutKind::BigOp {
                symbol: symbol.to_string(),
                sub: sub_laid.map(Box::new),
                sup: sup_laid.map(Box::new),
            },
        }
    }

    /// 적분 기호 레이아웃: 큰 기호 + 오른쪽 위/아래 첨자 (nolimits 스타일)
    fn layout_integral(
        &self,
        symbol: &str,
        sub: &Option<Box<EqNode>>,
        sup: &Option<Box<EqNode>>,
        fs: f64,
    ) -> LayoutBox {
        let op_fs = fs * BIG_OP_SCALE;
        let op_w = estimate_text_width(symbol, op_fs, false);
        let op_h = op_fs;

        let sub_box = sub.as_ref().map(|s| self.layout_node(s, fs * SCRIPT_SCALE));
        let sup_box = sup.as_ref().map(|s| self.layout_node(s, fs * SCRIPT_SCALE));

        // 기호 기준선: 기호 높이의 60% (중앙보다 약간 위)
        let op_baseline = op_h * 0.6;

        // 위첨자: 기호 오른쪽 위
        let sup_shift = op_h * 0.1; // 기호 상단에서 약간 아래
                                    // 아래첨자: 기호 오른쪽 아래
        let sub_shift = op_h * 0.55; // 기호 중앙 아래

        let script_x = op_w; // 첨자는 기호 오른쪽에 배치

        let mut total_w = op_w;
        let mut total_h = op_h;

        let sup_laid = sup_box.map(|mut b| {
            b.x = script_x;
            b.y = sup_shift;
            total_w = total_w.max(script_x + b.width);
            b
        });

        let sub_laid = sub_box.map(|mut b| {
            b.x = script_x;
            b.y = sub_shift;
            total_w = total_w.max(script_x + b.width);
            total_h = total_h.max(sub_shift + b.height);
            b
        });

        LayoutBox {
            x: 0.0,
            y: 0.0,
            // Task #1233: 적분 뒤 피연산자(예: f(x)dx)가 첨자에 붙지 않도록 trailing 간격.
            width: total_w + fs * BIG_OP_TRAIL_PAD,
            height: total_h,
            baseline: op_baseline,
            kind: LayoutKind::BigOp {
                symbol: symbol.to_string(),
                sub: sub_laid.map(Box::new),
                sup: sup_laid.map(Box::new),
            },
        }
    }

    fn layout_limit(&self, is_upper: bool, sub: &Option<Box<EqNode>>, fs: f64) -> LayoutBox {
        let name = if is_upper { "Lim" } else { "lim" };
        let name_w = estimate_text_width(name, fs, false);
        let name_h = fs;

        let sub_box = sub.as_ref().map(|s| self.layout_node(s, fs * SCRIPT_SCALE));
        let sub_h = sub_box
            .as_ref()
            .map(|b| b.height + fs * 0.05)
            .unwrap_or(0.0);
        let sub_w = sub_box.as_ref().map(|b| b.width).unwrap_or(0.0);

        let w = name_w.max(sub_w);
        let total_h = name_h + sub_h;

        let sub_laid = sub_box.map(|mut b| {
            b.x = (w - b.width) / 2.0;
            b.y = name_h;
            b
        });

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: w,
            height: total_h,
            baseline: fs * 0.8,
            kind: LayoutKind::Limit {
                is_upper,
                sub: sub_laid.map(Box::new),
            },
        }
    }

    fn layout_matrix(&self, rows: &[Vec<EqNode>], style: MatrixStyle, fs: f64) -> LayoutBox {
        if rows.is_empty() {
            return LayoutBox {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: fs,
                baseline: fs * 0.8,
                kind: LayoutKind::Empty,
            };
        }

        let col_gap = fs * MATRIX_COL_GAP;
        let row_gap = fs * MATRIX_ROW_GAP;

        // 모든 셀 레이아웃
        let mut cell_boxes: Vec<Vec<LayoutBox>> = rows
            .iter()
            .map(|row| row.iter().map(|c| self.layout_node(c, fs)).collect())
            .collect();

        let num_cols = cell_boxes.iter().map(|r| r.len()).max().unwrap_or(0);

        // 열 폭 계산
        let mut col_widths = vec![0.0f64; num_cols];
        for row in &cell_boxes {
            for (ci, cell) in row.iter().enumerate() {
                if ci < num_cols {
                    col_widths[ci] = col_widths[ci].max(cell.width);
                }
            }
        }

        // 행 높이 계산
        let mut row_heights: Vec<f64> = cell_boxes
            .iter()
            .map(|row| row.iter().map(|c| c.height).fold(fs, f64::max))
            .collect();

        // 셀 위치 배정
        let mut y = 0.0;
        for (ri, row) in cell_boxes.iter_mut().enumerate() {
            let rh = row_heights[ri];
            let mut x = 0.0;
            for (ci, cell) in row.iter_mut().enumerate() {
                let cw = if ci < num_cols {
                    col_widths[ci]
                } else {
                    cell.width
                };
                cell.x = x + (cw - cell.width) / 2.0;
                cell.y = y + (rh - cell.height) / 2.0;
                x += cw + if ci + 1 < num_cols { col_gap } else { 0.0 };
            }
            y += rh + row_gap;
        }

        let total_w: f64 =
            col_widths.iter().sum::<f64>() + col_gap * (num_cols.saturating_sub(1)) as f64;
        let total_h = y - row_gap;
        let bracket_pad = fs * 0.2;

        // 괄호 포함 폭
        let paren_w = match style {
            MatrixStyle::Plain => 0.0,
            _ => fs * 0.3,
        };
        let full_w = total_w + paren_w * 2.0 + bracket_pad * 2.0;

        // 셀 x 오프셋 (괄호 포함)
        let x_offset = paren_w + bracket_pad;
        for row in &mut cell_boxes {
            for cell in row.iter_mut() {
                cell.x += x_offset;
            }
        }

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: full_w,
            height: total_h,
            baseline: total_h / 2.0,
            kind: LayoutKind::Matrix {
                cells: cell_boxes,
                style,
            },
        }
    }

    fn layout_cases(&self, rows: &[EqNode], fs: f64) -> LayoutBox {
        let row_gap = fs * MATRIX_ROW_GAP;
        let mut row_boxes: Vec<LayoutBox> = rows.iter().map(|r| self.layout_node(r, fs)).collect();

        let max_w = row_boxes.iter().map(|b| b.width).fold(0.0f64, f64::max);
        let mut y = 0.0;
        for b in &mut row_boxes {
            b.x = fs * 0.3; // 왼쪽 중괄호 여백
            b.y = y;
            y += b.height + row_gap;
        }
        let total_h = y - row_gap;
        let full_w = max_w + fs * 0.6;

        // 중괄호 포함 레이아웃 → Paren으로 래핑
        let inner = LayoutBox {
            x: 0.0,
            y: 0.0,
            width: full_w,
            height: total_h,
            baseline: total_h / 2.0,
            kind: LayoutKind::Row(row_boxes),
        };

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: full_w + fs * 0.3,
            height: total_h,
            baseline: total_h / 2.0,
            kind: LayoutKind::Paren {
                left: "{".to_string(),
                right: String::new(),
                body: Box::new(inner),
            },
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

    fn layout_eqalign(&self, rows: &[(EqNode, EqNode)], fs: f64) -> LayoutBox {
        let row_gap = fs * MATRIX_ROW_GAP;
        let align_gap = fs * 0.15; // & 기준 좌우 사이 간격

        // 각 행의 왼쪽/오른쪽 레이아웃 계산
        let mut laid_rows: Vec<(LayoutBox, LayoutBox)> = rows
            .iter()
            .map(|(l, r)| (self.layout_node(l, fs), self.layout_node(r, fs)))
            .collect();

        // 왼쪽 최대 폭 (& 정렬 기준)
        let max_left_w = laid_rows
            .iter()
            .map(|(l, _)| l.width)
            .fold(0.0f64, f64::max);

        let mut y = 0.0;
        let mut total_w = 0.0f64;
        for (left, right) in &mut laid_rows {
            // 왼쪽: 오른쪽 정렬 (& 기준으로 맞춤)
            left.x = max_left_w - left.width;
            // 오른쪽: & 기준 바로 뒤
            right.x = max_left_w + align_gap;

            let row_h = left.height.max(right.height);
            let row_bl = left.baseline.max(right.baseline);
            // 베이스라인 정렬
            left.y = y + (row_bl - left.baseline);
            right.y = y + (row_bl - right.baseline);

            total_w = total_w.max(right.x + right.width);
            y += row_h + row_gap;
        }
        let total_h = (y - row_gap).max(0.0);

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: total_w,
            height: total_h,
            baseline: total_h / 2.0,
            kind: LayoutKind::EqAlign { rows: laid_rows },
        }
    }

    fn layout_pile(&self, rows: &[EqNode], align: PileAlign, fs: f64) -> LayoutBox {
        let row_gap = fs * MATRIX_ROW_GAP;
        let mut row_boxes: Vec<LayoutBox> = rows.iter().map(|r| self.layout_node(r, fs)).collect();

        let max_w = row_boxes.iter().map(|b| b.width).fold(0.0f64, f64::max);
        let mut y = 0.0;
        for b in &mut row_boxes {
            b.x = match align {
                PileAlign::Left => 0.0,
                PileAlign::Center => (max_w - b.width) / 2.0,
                PileAlign::Right => max_w - b.width,
            };
            b.y = y;
            y += b.height + row_gap;
        }
        let total_h = y - row_gap;

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: max_w,
            height: total_h,
            baseline: total_h / 2.0,
            kind: LayoutKind::Row(row_boxes),
        }
    }

    fn layout_paren(&self, left: &str, right: &str, body: &EqNode, fs: f64) -> LayoutBox {
        let b = self.layout_node(body, fs);
        let use_stretch_round = b.height > fs * 1.2 && matches!((left, right), ("(", ")"));
        let pad = if use_stretch_round {
            fs * 0.03
        } else {
            fs * PAREN_PAD
        };
        // Times New Roman '(' advance (em 기준) = 0.333. 텍스트 높이 glyph는 이 폭을 유지하고,
        // 큰 둥근 괄호 path는 한컴 HyhwpEQ 출력에 가깝게 더 좁게 잡는다. (Task #283, #1139)
        let paren_w = if use_stretch_round {
            fs * 0.27
        } else {
            fs * 0.333
        };

        let left_w = if left.is_empty() { 0.0 } else { paren_w };
        let right_w = if right.is_empty() { 0.0 } else { paren_w };

        let mut body_box = b;
        body_box.x = left_w + pad;
        body_box.y = 0.0;

        let total_w = left_w + pad + body_box.width + pad + right_w;

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: total_w,
            height: body_box.height,
            baseline: body_box.baseline,
            kind: LayoutKind::Paren {
                left: left.to_string(),
                right: right.to_string(),
                body: Box::new(body_box),
            },
        }
    }

    fn layout_decoration(
        &self,
        kind: super::symbols::DecoKind,
        body: &EqNode,
        fs: f64,
    ) -> LayoutBox {
        let b = self.layout_node(body, fs);
        let deco_h = fs * 0.25;

        let mut body_box = b;
        body_box.y = deco_h;

        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: body_box.width,
            height: body_box.height + deco_h,
            baseline: body_box.y + body_box.baseline,
            kind: LayoutKind::Decoration {
                kind,
                body: Box::new(body_box),
            },
        }
    }

    fn layout_font_style(
        &self,
        style: super::symbols::FontStyleKind,
        body: &EqNode,
        fs: f64,
    ) -> LayoutBox {
        let b = self.layout_node(body, fs);
        LayoutBox {
            x: 0.0,
            y: 0.0,
            width: b.width,
            height: b.height,
            baseline: b.baseline,
            kind: LayoutKind::FontStyle {
                style,
                body: Box::new(b),
            },
        }
    }

    fn layout_space(&self, kind: SpaceKind, fs: f64) -> LayoutBox {
        let w = match kind {
            SpaceKind::Normal => fs * 0.33,
            SpaceKind::Thin => fs * 0.17,
            SpaceKind::Tab => fs * 1.0,
        };
        LayoutBox {
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

/// 텍스트 폭 추정
pub(crate) fn estimate_text_width(text: &str, font_size: f64, italic: bool) -> f64 {
    let mut w = 0.0;
    for ch in text.chars() {
        let ratio = if ch.is_ascii() {
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
        '⊥' | '⊤' | '°' | '‰' | '‱' | '♯' => 0.5,
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
        // [#1300] 키 큰 base(괄호 분수 등)의 위첨자가 baseline 위로 과하게 치솟아
        // 윗줄을 침범하던 문제. base 밀어내기(base_y)는 sup 높이를 넘지 않아야 한다.
        fn find_sup(lb: &LayoutBox) -> Option<(&LayoutBox, &LayoutBox)> {
            match &lb.kind {
                LayoutKind::Superscript { base, sup } => Some((base, sup)),
                LayoutKind::Row(ch) => ch.iter().rev().find_map(find_sup),
                _ => None,
            }
        }
        // 위첨자 상단이 base 상단보다 위로 치솟지 않아야 한다(상단 정렬). 즉 sup.y >= base.y.
        // (이전 버그: 키 큰 base 를 아래로 밀어 sup.y=0 < base.y 가 되어 위첨자가 base 상단 위로 떠올랐다.)
        const MARGIN: f64 = 0.01;

        // 키 큰 base: 괄호로 감싼 분수 — 상단 정렬(base_y≈0) 확인
        let tall = parse_and_layout("LEFT ( {1} over {6} RIGHT )^4", 12.0);
        let (b_tall, s_tall) = find_sup(&tall).expect("tall superscript");
        assert!(
            s_tall.y >= b_tall.y - MARGIN,
            "tall: sup.y ({}) must not rise above base.y ({})",
            s_tall.y,
            b_tall.y
        );
        // 합성 baseline 이 base 자연 baseline 과 일치(이중 가산 없음)
        assert!(
            (tall.baseline - (b_tall.y + b_tall.baseline)).abs() < MARGIN,
            "tall: box baseline ({}) should equal base baseline ({})",
            tall.baseline,
            b_tall.y + b_tall.baseline
        );

        // 짧은 base: x^4 도 동일 불변(상단 정렬) — 위첨자가 base 상단 위로 안 떠오름
        let short = parse_and_layout("x^4", 12.0);
        let (b_short, s_short) = find_sup(&short).expect("short superscript");
        assert!(
            s_short.y >= b_short.y - MARGIN,
            "short: sup.y ({}) must not rise above base.y ({})",
            s_short.y,
            b_short.y
        );
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
