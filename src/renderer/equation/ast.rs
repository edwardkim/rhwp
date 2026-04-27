//! 수식 AST (Abstract Syntax Tree) 노드 정의
//!
//! 수식 스크립트를 파싱한 결과를 트리 구조로 표현한다.

use super::symbols::{DecoKind, FontStyleKind};

/// 행렬 스타일
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum MatrixStyle {
    Plain,   // MATRIX — 괄호 없음
    Paren,   // PMATRIX — 소괄호 ( )
    Bracket, // BMATRIX — 대괄호 [ ]
    Vert,    // DMATRIX — 세로줄 | |
}

/// 공백 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceKind {
    Normal,   // ~ (보통 공백)
    Thin,     // ` (1/4 공백)
    Tab,      // & (세로 칸 맞춤)
}

/// 수식 AST 노드
#[derive(Debug, Clone, PartialEq)]
pub enum EqNode {
    /// 수평 나열 (여러 노드의 연속)
    Row(Vec<EqNode>),

    /// 일반 텍스트 (이탤릭체로 렌더링되는 변수 등)
    Text(String),

    /// 숫자
    Number(String),

    /// 연산/관계 기호 (단일 문자: +, -, =, etc.)
    Symbol(String),

    /// Unicode 수학 기호 (명령어로부터 변환: α, ∞, ×, etc.)
    MathSymbol(String),

    /// 함수 이름 (로만체로 렌더링: sin, cos, log, etc.)
    Function(String),

    /// 분수: a OVER b
    Fraction {
        numer: Box<EqNode>,
        denom: Box<EqNode>,
    },

    /// 위아래 (분수선 없음): a ATOP b
    Atop {
        top: Box<EqNode>,
        bottom: Box<EqNode>,
    },

    /// 제곱근: SQRT x, SQRT(n) of x
    Sqrt {
        index: Option<Box<EqNode>>,
        body: Box<EqNode>,
    },

    /// 위첨자: x^y
    Superscript {
        base: Box<EqNode>,
        sup: Box<EqNode>,
    },

    /// 아래첨자: x_y
    Subscript {
        base: Box<EqNode>,
        sub: Box<EqNode>,
    },

    /// 위·아래첨자: x_a^b
    SubSup {
        base: Box<EqNode>,
        sub: Box<EqNode>,
        sup: Box<EqNode>,
    },

    /// 큰 연산자 (∫, ∑, ∏ 등): 위/아래 첨자 포함
    BigOp {
        symbol: String,
        sub: Option<Box<EqNode>>,
        sup: Option<Box<EqNode>>,
    },

    /// 극한: lim_{x→0}
    Limit {
        is_upper: bool, // Lim vs lim
        sub: Option<Box<EqNode>>,
    },

    /// 행렬: matrix{...}
    Matrix {
        rows: Vec<Vec<EqNode>>,
        style: MatrixStyle,
    },

    /// 조건식: CASES{...}
    Cases {
        rows: Vec<EqNode>,
    },

    /// 세로 쌓기: PILE/LPILE/RPILE
    Pile {
        rows: Vec<EqNode>,
        align: PileAlign,
    },

    /// 칸 맞춤 정렬: EQALIGN{...}
    /// & 기준으로 왼쪽/오른쪽을 분리하여 세로 정렬
    EqAlign {
        rows: Vec<(EqNode, EqNode)>, // (& 이전, & 이후) 쌍
    },

    /// 관계식: REL 화살표 {위} {아래}, BUILDREL 화살표 {위}
    Rel {
        arrow: String,
        over: Box<EqNode>,
        under: Option<Box<EqNode>>, // BUILDREL은 None
    },

    /// 자동 크기 괄호: LEFT ( ... RIGHT )
    Paren {
        left: String,
        right: String,
        body: Box<EqNode>,
    },

    /// 글자 장식: hat, bar, vec, etc.
    Decoration {
        kind: DecoKind,
        body: Box<EqNode>,
    },

    /// 글꼴 스타일: rm, it, bold
    FontStyle {
        style: FontStyleKind,
        body: Box<EqNode>,
    },

    /// 색상: COLOR{R,G,B}{body}
    Color {
        r: u8,
        g: u8,
        b: u8,
        body: Box<EqNode>,
    },

    /// 공백
    Space(SpaceKind),

    /// 줄바꿈 (#)
    Newline,

    /// 따옴표로 묶인 문자열 (단일 항으로 처리)
    Quoted(String),

    /// 빈 노드
    Empty,
}

/// 세로 쌓기 정렬
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PileAlign {
    Center, // PILE
    Left,   // LPILE
    Right,  // RPILE
}

impl EqNode {
    /// Row에서 불필요한 중첩 제거
    pub fn simplify(self) -> Self {
        match self {
            EqNode::Row(mut children) => {
                children = children.into_iter()
                    .map(|c| c.simplify())
                    .filter(|c| !matches!(c, EqNode::Empty))
                    .collect();
                if children.len() == 1 {
                    children.into_iter().next().unwrap()
                } else if children.is_empty() {
                    EqNode::Empty
                } else {
                    EqNode::Row(children)
                }
            }
            other => other,
        }
    }
}
