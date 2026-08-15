//! DSEL 구문 트리와 **축·속성 사전**.
//!
//! ## 사전이 왜 AST 옆에 있나
//!
//! 축(`para`·`table`·`cell` …)과 그 축이 가진 속성은 세 곳에서 필요하다 — 파서가
//! 오타를 후보와 함께 거절할 때, 평가기가 속성값을 뽑을 때, 스키마 산출기가
//! 문법을 자기서술할 때. 셋이 각자 목록을 들면 그 셋은 반드시 어긋난다(rhwp 가
//! `capabilities`·`ir_schema`·`ontology` 를 전부 **유도**로 만든 것과 같은 이유).
//! 그래서 목록은 여기 하나뿐이고, 나머지는 전부 이 사전을 읽는다.
//!
//! ## 축 계층이 사전에 들어 있는 이유
//!
//! `table` 은 `control` 의 특수화다 — `control[kind=table]` 과 `table` 은 같은 것을
//! 고른다. 이 관계를 [`AxisKind::specializes`] 로 **데이터로** 적어 두면
//! `ontology` 가 `rdfs:subClassOf` 를 유도할 때 손으로 계층을 다시 적지 않아도
//! 된다. 억지 계층을 만들지 않는다는 `ontology` 의 규약도 그대로 지켜진다 —
//! 여기 적힌 관계는 평가기가 실제로 그렇게 구현한 것뿐이다.
//!
//! ## 문서 트리의 실제 모양
//!
//! 축은 rhwp IR 을 그대로 따른다. 지어낸 층은 없다.
//!
//! ```text
//! Document
//!  └ section          Document::sections
//!     └ para          Section::paragraphs
//!        ├ run        Paragraph::char_shapes 가 나누는 구간
//!        └ control    Paragraph::controls
//!           ├ table   Control::Table
//!           │  └ cell Table::cells
//!           │     └ para   Cell::paragraphs   ← 여기서 재귀한다
//!           ├ picture/equation/field/…
//!           └ footnote/header/footer
//!              └ para        각자의 문단 목록
//! ```

use super::error::SelectorError;

pub use super::suggest::{nearest, unknown_attr, unknown_axis};

/// 선택자 하나 — 쉼표로 이어진 경로들의 합집합.
///
/// `source` 를 들고 다니는 이유: 평가 단계 오류도 캐럿을 그려야 하는데, 그때
/// 원문이 없으면 오프셋만 있는 반쪽 진단이 된다. 선택자 문자열은 짧으므로
/// 사본 비용보다 진단 품질이 이긴다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    /// 합집합 가지들. 최소 하나.
    pub paths: Vec<Path>,
    /// 파싱한 원문.
    pub source: String,
}

impl Selector {
    /// 이 선택자가 건드릴 수 있는 축의 집합 — 정책 게이트가 읽는다.
    ///
    /// 마지막 스텝의 축만 센다. 중간 스텝은 **경유**일 뿐 결과에 들어가지 않으므로,
    /// 중간 축까지 요구 능력에 포함시키면 게이트가 실제보다 넓게 막는다
    /// (`table cell` 이 표 편집 능력을 요구하게 되는 식).
    pub fn result_axes(&self) -> Vec<Axis> {
        let mut out: Vec<Axis> = self
            .paths
            .iter()
            .filter_map(|p| p.steps.last().map(|s| s.axis))
            .collect();
        out.sort_by_key(|a| a.name());
        out.dedup();
        out
    }

    /// 중첩 선택자(`:has`·`:not`)까지 포함한 최대 중첩 깊이.
    ///
    /// 평가 상한(`Limit`)을 파싱 직후에 판정하려고 쓴다 — 깊이 폭발을 평가 중에
    /// 발견하면 이미 시간을 쓴 뒤다.
    pub fn max_nesting(&self) -> usize {
        self.paths
            .iter()
            .flat_map(|p| p.steps.iter())
            .flat_map(|s| s.preds.iter())
            .map(|p| match p {
                Pred::Pseudo(Pseudo::Has(inner)) | Pred::Pseudo(Pseudo::Not(inner)) => {
                    1 + inner.max_nesting()
                }
                _ => 0,
            })
            .max()
            .unwrap_or(0)
    }
}

/// 결합자로 이어진 스텝 열. 첫 스텝의 결합자는 항상 [`Combinator::Root`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub steps: Vec<Step>,
}

/// 경로의 한 마디 — 결합자 + 축 + 술어들.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// 앞 스텝과의 관계.
    pub combinator: Combinator,
    /// 고를 노드 종류.
    pub axis: Axis,
    /// 걸러 낼 조건들. 순서는 원문 순서를 보존한다 — 평가 비용이 다른 술어를
    /// 재배치하는 최적화는 하지 않는다. 같은 선택자가 항상 같은 순서로 평가되어야
    /// 오류 보고 위치가 재현된다.
    pub preds: Vec<Pred>,
    /// 원문에서 축 이름이 시작한 문자 오프셋.
    pub offset: usize,
}

/// 스텝 사이의 관계.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// 경로의 첫 스텝 — 문서 루트에서 시작.
    Root,
    /// 공백 — 임의 깊이 자손.
    Descendant,
    /// `>` — 직계 자식.
    Child,
    /// `+` — 바로 다음 형제.
    NextSibling,
    /// `~` — 이후 모든 형제.
    FollowingSibling,
}

impl Combinator {
    /// 원문 기호. `Root` 는 기호가 없다.
    pub const fn symbol(self) -> &'static str {
        match self {
            Combinator::Root => "",
            Combinator::Descendant => " ",
            Combinator::Child => ">",
            Combinator::NextSibling => "+",
            Combinator::FollowingSibling => "~",
        }
    }
}

/// 축 — 구체 종류 또는 전체.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// `*` — 종류를 가리지 않는다.
    Any,
    /// 이름 붙은 축.
    Kind(AxisKind),
}

impl Axis {
    /// 축 이름 — `*` 포함.
    pub const fn name(self) -> &'static str {
        match self {
            Axis::Any => "*",
            Axis::Kind(k) => k.name(),
        }
    }

    /// 이 축에서 쓸 수 있는 속성 목록.
    ///
    /// `*` 는 **모든 축의 공통 속성만** 준다. 합집합을 주면 `*[rows>2]` 가 문법
    /// 오류를 통과하고 평가에서 조용히 아무것도 안 고르는 결과가 된다 — 선택자가
    /// 빈 결과를 낸 이유가 "그런 속성이 없어서"인지 "조건에 맞는 게 없어서"인지
    /// 구별할 수 없게 되는 것이 최악이다.
    pub fn attributes(self) -> &'static [AttrDef] {
        match self {
            Axis::Any => COMMON_ATTRS,
            Axis::Kind(k) => k.attributes(),
        }
    }
}

/// 이름 붙은 축 — rhwp IR 의 실제 노드 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AxisKind {
    /// `Document::sections` 의 한 구역.
    Section,
    /// 문단 — 구역·셀·각주·머리말 어디에 있든 같은 축이다.
    Para,
    /// 글자 모양이 같은 연속 구간 (`Paragraph::char_shapes` 가 나눈다).
    Run,
    /// `Paragraph::controls` 의 컨트롤 일반.
    Control,
    /// `Control::Table`.
    Table,
    /// `Table::cells` 의 셀.
    Cell,
    /// `Control::Picture`.
    Picture,
    /// `Control::Equation`.
    Equation,
    /// `Control::Field` — 누름틀·메모·하이퍼링크 등 필드 컨트롤.
    Field,
    /// `Control::Footnote`.
    Footnote,
    /// `Control::Endnote`.
    Endnote,
    /// `Control::Header`.
    Header,
    /// `Control::Footer`.
    Footer,
    /// `Control::Bookmark`.
    Bookmark,
    /// `Control::Hyperlink`.
    Hyperlink,
    /// `Control::Shape` — 그리기 개체.
    Shape,
}

/// 축 이름 ↔ 종류 대응표. 파서·스키마 산출기가 함께 읽는다.
pub const AXIS_NAMES: &[(&str, AxisKind)] = &[
    ("section", AxisKind::Section),
    ("para", AxisKind::Para),
    ("run", AxisKind::Run),
    ("control", AxisKind::Control),
    ("table", AxisKind::Table),
    ("cell", AxisKind::Cell),
    ("picture", AxisKind::Picture),
    ("equation", AxisKind::Equation),
    ("field", AxisKind::Field),
    ("footnote", AxisKind::Footnote),
    ("endnote", AxisKind::Endnote),
    ("header", AxisKind::Header),
    ("footer", AxisKind::Footer),
    ("bookmark", AxisKind::Bookmark),
    ("hyperlink", AxisKind::Hyperlink),
    ("shape", AxisKind::Shape),
];

impl AxisKind {
    /// 이름으로 축을 찾는다.
    pub fn from_name(name: &str) -> Option<AxisKind> {
        AXIS_NAMES.iter().find(|(n, _)| *n == name).map(|(_, k)| *k)
    }

    /// 안정 이름.
    pub const fn name(self) -> &'static str {
        match self {
            AxisKind::Section => "section",
            AxisKind::Para => "para",
            AxisKind::Run => "run",
            AxisKind::Control => "control",
            AxisKind::Table => "table",
            AxisKind::Cell => "cell",
            AxisKind::Picture => "picture",
            AxisKind::Equation => "equation",
            AxisKind::Field => "field",
            AxisKind::Footnote => "footnote",
            AxisKind::Endnote => "endnote",
            AxisKind::Header => "header",
            AxisKind::Footer => "footer",
            AxisKind::Bookmark => "bookmark",
            AxisKind::Hyperlink => "hyperlink",
            AxisKind::Shape => "shape",
        }
    }

    /// 한 줄 설명 — 스키마·`capabilities` 로 그대로 나간다.
    pub const fn doc(self) -> &'static str {
        match self {
            AxisKind::Section => "구역 — 문서의 최상위 분할",
            AxisKind::Para => "문단 — 구역·셀·각주·머리말 안 어디에 있든 같은 축",
            AxisKind::Run => "글자 모양이 같은 연속 구간",
            AxisKind::Control => "컨트롤 일반 — 표·그림·필드 등의 상위 축",
            AxisKind::Table => "표",
            AxisKind::Cell => "표의 셀",
            AxisKind::Picture => "그림",
            AxisKind::Equation => "수식",
            AxisKind::Field => "필드 컨트롤 — 누름틀·메모 등",
            AxisKind::Footnote => "각주",
            AxisKind::Endnote => "미주",
            AxisKind::Header => "머리말",
            AxisKind::Footer => "꼬리말",
            AxisKind::Bookmark => "책갈피",
            AxisKind::Hyperlink => "하이퍼링크",
            AxisKind::Shape => "그리기 개체",
        }
    }

    /// 이 축이 특수화하는 상위 축.
    ///
    /// `table` → `control` 처럼, `control` 로 고를 수 있는 것을 좁혀 고르는 관계만
    /// 적는다. `cell` 은 `table` 의 **자식**이지 특수화가 아니므로 여기 없다 —
    /// 포함 관계와 특수화 관계를 섞으면 온톨로지가 거짓을 말한다.
    pub const fn specializes(self) -> Option<AxisKind> {
        match self {
            AxisKind::Table
            | AxisKind::Picture
            | AxisKind::Equation
            | AxisKind::Field
            | AxisKind::Footnote
            | AxisKind::Endnote
            | AxisKind::Header
            | AxisKind::Footer
            | AxisKind::Bookmark
            | AxisKind::Hyperlink
            | AxisKind::Shape => Some(AxisKind::Control),
            _ => None,
        }
    }

    /// 이 축이 문단을 담을 수 있나 — 재귀 하강의 근거.
    ///
    /// 셀·각주·미주·머리말·꼬리말이 문단을 담는다는 것은 IR 의 사실이다
    /// (`Cell::paragraphs`, `Footnote::paragraphs` …). 평가기의 하강 규칙과 이
    /// 함수가 어긋나면 `table para` 가 셀 안 문단을 놓친다.
    pub const fn holds_paragraphs(self) -> bool {
        matches!(
            self,
            AxisKind::Section
                | AxisKind::Cell
                | AxisKind::Footnote
                | AxisKind::Endnote
                | AxisKind::Header
                | AxisKind::Footer
        )
    }

    /// 이 축의 속성 목록.
    pub const fn attributes(self) -> &'static [AttrDef] {
        match self {
            AxisKind::Section => SECTION_ATTRS,
            AxisKind::Para => PARA_ATTRS,
            AxisKind::Run => RUN_ATTRS,
            AxisKind::Control => CONTROL_ATTRS,
            AxisKind::Table => TABLE_ATTRS,
            AxisKind::Cell => CELL_ATTRS,
            AxisKind::Field => FIELD_ATTRS,
            AxisKind::Hyperlink => HYPERLINK_ATTRS,
            AxisKind::Bookmark => BOOKMARK_ATTRS,
            // 나머지 컨트롤은 아직 고유 속성이 없다 — 공통 속성만으로 고른다.
            // "없다"를 빈 목록이 아니라 공통 목록으로 두는 이유: `picture[index=0]`
            // 이 문법 오류가 되면 안 된다.
            _ => CONTROL_ATTRS,
        }
    }
}

/// 속성 하나의 정의.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttrDef {
    /// 선택자에 적는 이름.
    pub name: &'static str,
    /// 값 타입 — 비교 연산자 적합성 판정에 쓴다.
    pub ty: AttrType,
    /// 한 줄 설명.
    pub doc: &'static str,
}

/// 속성 값 타입.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrType {
    /// 문자열 — `=`·`!=`·`^=`·`$=`·`*=`·`~=` 가능. 대소 비교는 불가.
    Str,
    /// 정수 — 여섯 비교자 전부 가능하되 부분 일치는 불가.
    Int,
    /// 불리언 — `=`·`!=` 만. 속성 이름 단독은 `= true` 와 같다.
    Bool,
}

impl AttrType {
    /// 이 타입에 이 연산자를 쓸 수 있나.
    ///
    /// 문자열에 `>=` 를 허용하지 않는 이유: 사전식 비교는 로캘에 따라 답이 달라진다.
    /// 로캘 의존 결과는 커널의 결정론 규약을 깬다.
    pub const fn accepts(self, op: CmpOp) -> bool {
        match self {
            AttrType::Str => !matches!(op, CmpOp::Gt | CmpOp::Lt | CmpOp::Ge | CmpOp::Le),
            AttrType::Int => matches!(
                op,
                CmpOp::Eq | CmpOp::Ne | CmpOp::Gt | CmpOp::Lt | CmpOp::Ge | CmpOp::Le
            ),
            AttrType::Bool => matches!(op, CmpOp::Eq | CmpOp::Ne),
        }
    }

    /// 스키마용 이름.
    pub const fn as_str(self) -> &'static str {
        match self {
            AttrType::Str => "string",
            AttrType::Int => "integer",
            AttrType::Bool => "boolean",
        }
    }
}

/// 모든 축이 갖는 속성.
pub const COMMON_ATTRS: &[AttrDef] = &[AttrDef {
    name: "index",
    ty: AttrType::Int,
    doc: "형제 중 0 기준 순번",
}];

const SECTION_ATTRS: &[AttrDef] = &[
    AttrDef {
        name: "index",
        ty: AttrType::Int,
        doc: "구역 순번 (0 기준)",
    },
    AttrDef {
        name: "paras",
        ty: AttrType::Int,
        doc: "직계 문단 수",
    },
];

const PARA_ATTRS: &[AttrDef] = &[
    AttrDef {
        name: "index",
        ty: AttrType::Int,
        doc: "형제 문단 중 순번 (0 기준)",
    },
    AttrDef {
        name: "text",
        ty: AttrType::Str,
        doc: "문단 텍스트 (제어문자 제외)",
    },
    AttrDef {
        name: "len",
        ty: AttrType::Int,
        doc: "텍스트 문자 수 (제어문자 제외)",
    },
    AttrDef {
        name: "styleId",
        ty: AttrType::Int,
        doc: "문단 스타일 ID",
    },
    AttrDef {
        name: "shapeId",
        ty: AttrType::Int,
        doc: "문단 모양 ID",
    },
    AttrDef {
        name: "empty",
        ty: AttrType::Bool,
        doc: "텍스트가 비었는가 (공백만 있어도 참)",
    },
    AttrDef {
        name: "controls",
        ty: AttrType::Int,
        doc: "직계 컨트롤 수",
    },
];

const RUN_ATTRS: &[AttrDef] = &[
    AttrDef {
        name: "index",
        ty: AttrType::Int,
        doc: "문단 안 구간 순번 (0 기준)",
    },
    AttrDef {
        name: "text",
        ty: AttrType::Str,
        doc: "구간 텍스트",
    },
    AttrDef {
        name: "len",
        ty: AttrType::Int,
        doc: "구간 문자 수",
    },
    AttrDef {
        name: "charShapeId",
        ty: AttrType::Int,
        doc: "글자 모양 ID",
    },
];

const CONTROL_ATTRS: &[AttrDef] = &[
    AttrDef {
        name: "index",
        ty: AttrType::Int,
        doc: "문단 안 컨트롤 순번 (0 기준)",
    },
    AttrDef {
        name: "kind",
        ty: AttrType::Str,
        doc: "컨트롤 종류 이름 — 축 이름과 같은 어휘",
    },
    AttrDef {
        name: "inline",
        ty: AttrType::Bool,
        doc: "글자처럼 취급되는가",
    },
];

const TABLE_ATTRS: &[AttrDef] = &[
    AttrDef {
        name: "index",
        ty: AttrType::Int,
        doc: "문단 안 컨트롤 순번 (0 기준)",
    },
    AttrDef {
        name: "kind",
        ty: AttrType::Str,
        doc: "컨트롤 종류 이름",
    },
    AttrDef {
        name: "inline",
        ty: AttrType::Bool,
        doc: "글자처럼 취급되는가",
    },
    AttrDef {
        name: "rows",
        ty: AttrType::Int,
        doc: "행 수",
    },
    AttrDef {
        name: "cols",
        ty: AttrType::Int,
        doc: "열 수",
    },
];

const CELL_ATTRS: &[AttrDef] = &[
    AttrDef {
        name: "index",
        ty: AttrType::Int,
        doc: "표 안 셀 순번 (행 우선, 0 기준)",
    },
    AttrDef {
        name: "row",
        ty: AttrType::Int,
        doc: "행 주소 (0 기준)",
    },
    AttrDef {
        name: "col",
        ty: AttrType::Int,
        doc: "열 주소 (0 기준)",
    },
    AttrDef {
        name: "rowSpan",
        ty: AttrType::Int,
        doc: "행 병합 개수",
    },
    AttrDef {
        name: "colSpan",
        ty: AttrType::Int,
        doc: "열 병합 개수",
    },
    AttrDef {
        name: "text",
        ty: AttrType::Str,
        doc: "셀 안 문단 텍스트를 개행으로 이은 값",
    },
    AttrDef {
        name: "header",
        ty: AttrType::Bool,
        doc: "제목 셀인가",
    },
    AttrDef {
        name: "name",
        ty: AttrType::Str,
        doc: "셀 필드 이름 (없으면 어떤 값과도 같지 않다)",
    },
];

const FIELD_ATTRS: &[AttrDef] = &[
    AttrDef {
        name: "index",
        ty: AttrType::Int,
        doc: "문단 안 컨트롤 순번 (0 기준)",
    },
    AttrDef {
        name: "kind",
        ty: AttrType::Str,
        doc: "컨트롤 종류 이름",
    },
    AttrDef {
        name: "inline",
        ty: AttrType::Bool,
        doc: "글자처럼 취급되는가",
    },
    AttrDef {
        name: "name",
        ty: AttrType::Str,
        doc: "필드 이름 — CTRL_DATA 이름이 있으면 그것, 없으면 command",
    },
    AttrDef {
        name: "type",
        ty: AttrType::Str,
        doc: "필드 타입 이름",
    },
];

const HYPERLINK_ATTRS: &[AttrDef] = &[
    AttrDef {
        name: "index",
        ty: AttrType::Int,
        doc: "문단 안 컨트롤 순번 (0 기준)",
    },
    AttrDef {
        name: "kind",
        ty: AttrType::Str,
        doc: "컨트롤 종류 이름",
    },
    AttrDef {
        name: "inline",
        ty: AttrType::Bool,
        doc: "글자처럼 취급되는가",
    },
];

const BOOKMARK_ATTRS: &[AttrDef] = &[
    AttrDef {
        name: "index",
        ty: AttrType::Int,
        doc: "문단 안 컨트롤 순번 (0 기준)",
    },
    AttrDef {
        name: "kind",
        ty: AttrType::Str,
        doc: "컨트롤 종류 이름",
    },
    AttrDef {
        name: "inline",
        ty: AttrType::Bool,
        doc: "글자처럼 취급되는가",
    },
    AttrDef {
        name: "name",
        ty: AttrType::Str,
        doc: "책갈피 이름",
    },
];

/// 스텝의 술어 — 속성 조건 또는 의사 선택자.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pred {
    Attr(AttrPred),
    Pseudo(Pseudo),
}

/// `[name op value]` 또는 `[name]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrPred {
    /// 속성 이름.
    pub name: String,
    /// 비교자와 값. `None` 이면 존재 검사 — 불리언은 참, 나머지는 "값이 있는가".
    pub compare: Option<(CmpOp, Literal)>,
    /// 속성 이름이 시작한 문자 오프셋.
    pub offset: usize,
}

/// 비교 연산자.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    /// `^=` 접두.
    Prefix,
    /// `$=` 접미.
    Suffix,
    /// `*=` 부분.
    Substr,
    /// `~=` 글롭.
    Glob,
}

impl CmpOp {
    pub const fn symbol(self) -> &'static str {
        match self {
            CmpOp::Eq => "=",
            CmpOp::Ne => "!=",
            CmpOp::Gt => ">",
            CmpOp::Lt => "<",
            CmpOp::Ge => ">=",
            CmpOp::Le => "<=",
            CmpOp::Prefix => "^=",
            CmpOp::Suffix => "$=",
            CmpOp::Substr => "*=",
            CmpOp::Glob => "~=",
        }
    }
}

/// 리터럴 값.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Str(String),
    Int(i64),
    Bool(bool),
}

impl Literal {
    /// 스키마·오류 메시지용 타입 이름.
    pub const fn type_name(&self) -> &'static str {
        match self {
            Literal::Str(_) => "string",
            Literal::Int(_) => "integer",
            Literal::Bool(_) => "boolean",
        }
    }
}

/// 의사 선택자.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pseudo {
    /// `:first` — 형제 중 첫째.
    First,
    /// `:last` — 형제 중 마지막.
    Last,
    /// `:nth(n)` — 0 기준 순번. 음수는 뒤에서 센다(`-1` = 마지막).
    Nth(i64),
    /// `:range(a..b)` — 0 기준 반열림 구간 `[a, b)`. 음수 인덱스 허용.
    Range { from: i64, to: i64 },
    /// `:contains("…")` — 텍스트 부분 일치.
    Contains(String),
    /// `:matches("…")` — 글롭 일치.
    Matches(String),
    /// `:empty` — 텍스트가 비었다.
    Empty,
    /// `:not(sel)` — 중첩 선택자에 걸리지 않는다.
    Not(Box<Selector>),
    /// `:has(sel)` — 자손 중에 중첩 선택자에 걸리는 것이 있다.
    Has(Box<Selector>),
}

impl Pseudo {
    /// 안정 이름.
    pub const fn name(&self) -> &'static str {
        match self {
            Pseudo::First => "first",
            Pseudo::Last => "last",
            Pseudo::Nth(_) => "nth",
            Pseudo::Range { .. } => "range",
            Pseudo::Contains(_) => "contains",
            Pseudo::Matches(_) => "matches",
            Pseudo::Empty => "empty",
            Pseudo::Not(_) => "not",
            Pseudo::Has(_) => "has",
        }
    }
}

/// 의사 선택자 하나의 정의 — 파서·스키마가 함께 읽는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PseudoDef {
    pub name: &'static str,
    /// 인자 모양.
    pub arity: PseudoArity,
    pub doc: &'static str,
}

/// 의사 선택자의 인자 모양.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PseudoArity {
    /// 인자 없음 — 괄호를 쓰면 오류.
    None,
    /// 정수 하나.
    Int,
    /// 문자열 하나.
    Str,
    /// `a..b` 범위.
    Range,
    /// 중첩 선택자 하나.
    Selector,
}

/// 의사 선택자 사전.
pub const PSEUDO_DEFS: &[PseudoDef] = &[
    PseudoDef {
        name: "first",
        arity: PseudoArity::None,
        doc: "형제 중 첫째",
    },
    PseudoDef {
        name: "last",
        arity: PseudoArity::None,
        doc: "형제 중 마지막",
    },
    PseudoDef {
        name: "empty",
        arity: PseudoArity::None,
        doc: "텍스트가 비었다 (공백만 있어도 참)",
    },
    PseudoDef {
        name: "nth",
        arity: PseudoArity::Int,
        doc: "0 기준 순번. 음수는 뒤에서 센다 (-1 = 마지막)",
    },
    PseudoDef {
        name: "range",
        arity: PseudoArity::Range,
        doc: "0 기준 반열림 구간 a..b. 음수 인덱스 허용",
    },
    PseudoDef {
        name: "contains",
        arity: PseudoArity::Str,
        doc: "텍스트 부분 일치",
    },
    PseudoDef {
        name: "matches",
        arity: PseudoArity::Str,
        doc: "글롭 일치 — * ? [abc] [a-z] [!abc]",
    },
    PseudoDef {
        name: "not",
        arity: PseudoArity::Selector,
        doc: "중첩 선택자에 걸리지 않는 것만",
    },
    PseudoDef {
        name: "has",
        arity: PseudoArity::Selector,
        doc: "자손 중 중첩 선택자에 걸리는 것이 있는 것만",
    },
];

impl PseudoDef {
    pub fn from_name(name: &str) -> Option<&'static PseudoDef> {
        PSEUDO_DEFS.iter().find(|d| d.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_axis_name_round_trips() {
        for (name, kind) in AXIS_NAMES {
            assert_eq!(AxisKind::from_name(name), Some(*kind));
            assert_eq!(kind.name(), *name);
        }
    }

    #[test]
    fn specialized_axes_expose_the_parent_attributes() {
        // `table` 은 `control` 의 특수화이므로 control 의 속성을 전부 갖는다.
        // 갖지 않으면 `control[kind=table]` 로는 되는데 `table` 로는 안 되는
        // 비대칭이 생긴다.
        for attr in AxisKind::Control.attributes() {
            assert!(
                AxisKind::Table
                    .attributes()
                    .iter()
                    .any(|a| a.name == attr.name),
                "table 축에 control 속성 `{}` 이 없다",
                attr.name
            );
        }
    }

    #[test]
    fn specialization_never_points_at_a_containment_parent() {
        // cell 은 table 안에 있지만 table 의 특수화가 아니다.
        assert_eq!(AxisKind::Cell.specializes(), None);
        assert_eq!(AxisKind::Table.specializes(), Some(AxisKind::Control));
    }

    #[test]
    fn wildcard_axis_exposes_only_common_attributes() {
        let names: Vec<&str> = Axis::Any.attributes().iter().map(|a| a.name).collect();
        assert_eq!(names, vec!["index"]);
    }

    #[test]
    fn string_attributes_reject_ordering_operators() {
        assert!(!AttrType::Str.accepts(CmpOp::Ge));
        assert!(AttrType::Str.accepts(CmpOp::Prefix));
        assert!(!AttrType::Int.accepts(CmpOp::Substr));
        assert!(AttrType::Bool.accepts(CmpOp::Ne));
        assert!(!AttrType::Bool.accepts(CmpOp::Gt));
    }

    #[test]
    fn nearest_is_deterministic_under_ties() {
        // 같은 거리 후보가 둘이면 사전순 앞선 쪽. 선언 순서를 뒤집어도 같아야 한다.
        let forward = nearest("cellx", ["cell", "cells"]);
        let backward = nearest("cellx", ["cells", "cell"]);
        assert_eq!(forward, backward);
    }

    #[test]
    fn nearest_gives_up_beyond_distance_two() {
        assert_eq!(nearest("표", AXIS_NAMES.iter().map(|(n, _)| *n)), None);
    }

    #[test]
    fn unknown_axis_suggests_the_close_one() {
        let err = unknown_axis("tabel", 0);
        assert_eq!(err.hint.as_deref(), Some("`table` 를 뜻했나"));
    }

    #[test]
    fn unknown_attr_scopes_candidates_to_the_axis() {
        let err = unknown_attr(Axis::Kind(AxisKind::Cell), "rowspan", 5);
        // 후보는 cell 의 속성뿐 — para 의 `text` 가 섞이더라도 cell 에도 있으니
        // 확인은 cell 고유 속성으로 한다.
        assert!(err.expected.iter().any(|e| e == "rowSpan"));
        assert!(!err.expected.iter().any(|e| e == "styleId"));
    }

    #[test]
    fn pseudo_defs_cover_every_pseudo_variant_name() {
        let variants = [
            Pseudo::First,
            Pseudo::Last,
            Pseudo::Nth(0),
            Pseudo::Range { from: 0, to: 1 },
            Pseudo::Contains(String::new()),
            Pseudo::Matches(String::new()),
            Pseudo::Empty,
        ];
        for v in variants {
            assert!(
                PseudoDef::from_name(v.name()).is_some(),
                "사전에 `{}` 가 없다",
                v.name()
            );
        }
        assert!(PseudoDef::from_name("not").is_some());
        assert!(PseudoDef::from_name("has").is_some());
    }

    #[test]
    fn paragraph_holders_match_the_ir_shape() {
        // 문단을 담는 축은 IR 의 사실이다. 하나라도 빠지면 `table para` 가
        // 셀 안 문단을 놓친다.
        assert!(AxisKind::Cell.holds_paragraphs());
        assert!(AxisKind::Section.holds_paragraphs());
        assert!(AxisKind::Footnote.holds_paragraphs());
        assert!(!AxisKind::Table.holds_paragraphs());
        assert!(!AxisKind::Para.holds_paragraphs());
    }
}
