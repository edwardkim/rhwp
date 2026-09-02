//! `docdiff` 결과 타입 — 좌표(`NodePath`)·발견(`Finding`)·집계(`DiffSummary`).
//!
//! 이 파일에는 **판정 로직이 없다.** 비교는 [`super::compare`], 집계는
//! [`super::summary`] 가 맡고 여기는 두 쪽이 공유하는 **계약**만 둔다. 결과 타입을
//! 비교기와 분리해 두는 이유는 소비자(회귀 시험·CLI·MCP)가 비교 알고리즘이 아니라
//! **결과 모양**에 의존하기 때문이다.

use std::fmt;

/// 문서 안의 한 지점을 가리키는 좌표 한 칸.
///
/// 첨자는 전부 0 부터 센다. 표 셀만 행·열 주소를 함께 싣는데, 병합 때문에
/// `cells` 안의 순번보다 `(row, col)` 이 사람에게 훨씬 잘 읽히기 때문이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathStep {
    /// 구역 — `sec[i]`
    Section(usize),
    /// 문단 — `para[i]`
    Paragraph(usize),
    /// 문단 안 컨트롤 — `ctrl[i]`
    Control(usize),
    /// 표 셀 — `cell[r{row},c{col}]`
    TableCell {
        /// 셀의 행 주소
        row: u16,
        /// 셀의 열 주소
        col: u16,
    },
    /// 문서 정보(`doc_info`) 아래의 스타일 — `style[i]`
    Style(usize),
}

impl fmt::Display for PathStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathStep::Section(i) => write!(f, "sec[{}]", i),
            PathStep::Paragraph(i) => write!(f, "para[{}]", i),
            PathStep::Control(i) => write!(f, "ctrl[{}]", i),
            PathStep::TableCell { row, col } => write!(f, "cell[r{},c{}]", row, col),
            PathStep::Style(i) => write!(f, "style[{}]", i),
        }
    }
}

/// 차이가 난 지점의 좌표 — `sec[0]/para[3]/ctrl[1]/cell[r2,c0]/para[0]` 처럼 읽힌다.
///
/// 회귀 검증이 "달라졌다"가 아니라 "**어디가** 달라졌다"를 말할 수 있게 하는 값이다.
/// 문자열이 아니라 타입으로 두는 이유는 소비자가 구역 번호 같은 것을 문자열 파싱
/// 없이 꺼내 쓸 수 있어야 하기 때문이다([`NodePath::section`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodePath {
    steps: Vec<PathStep>,
}

impl NodePath {
    /// 문서 전체를 가리키는 빈 경로.
    pub fn root() -> Self {
        Self::default()
    }

    /// 이 경로 아래 한 칸 내려간 새 경로를 만든다(원본은 그대로).
    pub fn child(&self, step: PathStep) -> Self {
        let mut steps = self.steps.clone();
        steps.push(step);
        Self { steps }
    }

    /// 경로를 이루는 좌표 칸들.
    pub fn steps(&self) -> &[PathStep] {
        &self.steps
    }

    /// 문서 전체를 가리키는 빈 경로인가.
    pub fn is_root(&self) -> bool {
        self.steps.is_empty()
    }

    /// 이 경로가 속한 구역 번호 — 구역 밖(문서 정보 등)이면 `None`.
    pub fn section(&self) -> Option<usize> {
        self.steps.iter().find_map(|s| match s {
            PathStep::Section(i) => Some(*i),
            _ => None,
        })
    }

    /// 이 경로가 가리키는 **최말단** 문단 번호 — 문단을 안 가리키면 `None`.
    ///
    /// 표 셀 안 문단이면 셀 안에서의 번호가 나온다(바깥 문단 번호가 아니다).
    pub fn paragraph(&self) -> Option<usize> {
        self.steps.iter().rev().find_map(|s| match s {
            PathStep::Paragraph(i) => Some(*i),
            _ => None,
        })
    }
}

impl fmt::Display for NodePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.steps.is_empty() {
            return write!(f, "doc");
        }
        for (i, step) in self.steps.iter().enumerate() {
            if i > 0 {
                write!(f, "/")?;
            }
            write!(f, "{}", step)?;
        }
        Ok(())
    }
}

/// 차이의 종류.
///
/// 종류는 **의미 단위**다 — 직렬화기 충실도 축(글자모양 시퀀스·lineseg 같은 것)은
/// 여기 없다. 그쪽은 [`crate::serializer::hwpx::roundtrip::diff_documents`] 의 몫이다
/// (경계는 모듈 문서 참고).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingKind {
    /// 구역 수가 달라졌다.
    SectionCountChanged,
    /// B 에만 있는 문단(추가됨).
    ParagraphAdded,
    /// A 에만 있는 문단(삭제됨).
    ParagraphRemoved,
    /// 짝지어진 문단의 본문 텍스트가 달라졌다.
    TextChanged,
    /// 짝지어진 문단의 문단모양·스타일 참조가 달라졌다.
    ParagraphStyleChanged,
    /// 표의 행·열 수 또는 셀 병합 모양이 달라졌다.
    TableShapeChanged,
    /// 문단이 품은 컨트롤 개수가 달라졌다.
    ControlCountChanged,
    /// 같은 자리의 컨트롤 종류가 달라졌다(예: 표 → 그림).
    ControlKindChanged,
    /// 문서 정보의 스타일 개수가 달라졌다.
    StyleCountChanged,
    /// 같은 번호 스타일의 정의가 달라졌다.
    StyleChanged,
}

impl FindingKind {
    /// 선언 순서 그대로의 전체 목록 — 집계 순서의 단일 출처.
    ///
    /// 집계가 `HashMap` 을 돌지 않고 이 배열을 도는 덕분에 요약 순서가 항상 같다.
    pub const ALL: [FindingKind; 10] = [
        FindingKind::SectionCountChanged,
        FindingKind::ParagraphAdded,
        FindingKind::ParagraphRemoved,
        FindingKind::TextChanged,
        FindingKind::ParagraphStyleChanged,
        FindingKind::TableShapeChanged,
        FindingKind::ControlCountChanged,
        FindingKind::ControlKindChanged,
        FindingKind::StyleCountChanged,
        FindingKind::StyleChanged,
    ];

    /// 기계 가독 이름 — 봉투·로그의 안정 키다. **바꾸면 계약이 깨진다.**
    pub fn label(&self) -> &'static str {
        match self {
            FindingKind::SectionCountChanged => "sectionCountChanged",
            FindingKind::ParagraphAdded => "paragraphAdded",
            FindingKind::ParagraphRemoved => "paragraphRemoved",
            FindingKind::TextChanged => "textChanged",
            FindingKind::ParagraphStyleChanged => "paragraphStyleChanged",
            FindingKind::TableShapeChanged => "tableShapeChanged",
            FindingKind::ControlCountChanged => "controlCountChanged",
            FindingKind::ControlKindChanged => "controlKindChanged",
            FindingKind::StyleCountChanged => "styleCountChanged",
            FindingKind::StyleChanged => "styleChanged",
        }
    }
}

impl fmt::Display for FindingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// 차이 하나 — "어디가(`path`) 어떻게(`kind`) 얼마나(`detail`)".
///
/// `detail` 은 **사람이 읽는 한 줄**이다. 기계 판정은 `kind` 와 `path` 로 하고
/// `detail` 을 파싱하지 마라 — 문구는 계약이 아니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// 차이가 난 지점.
    pub path: NodePath,
    /// 차이의 종류.
    pub kind: FindingKind,
    /// 사람이 읽는 상세 — 미리보기는 문자 단위로 잘려 있다.
    pub detail: String,
}

impl Finding {
    /// 봉투 한 칸으로 쓸 JSON 값.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "path": self.path.to_string(),
            "kind": self.kind.label(),
            "detail": self.detail,
        })
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.path, self.kind.label(), self.detail)
    }
}

/// 비교 옵션.
///
/// `Default` 는 "아무것도 봐주지 않고, 아무것도 자르지 않는다" — 회귀 게이트가
/// 기본값으로 안전하게 쓰도록 한 선택이다.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiffOptions {
    /// 텍스트 비교에서 공백 차이를 무시한다.
    ///
    /// 켜면 연속 공백을 한 칸으로 접고 양끝을 다듬은 뒤 비교한다. 문단을 짝짓는
    /// 정렬에도 같은 정규화가 적용되므로, 들여쓰기만 바뀐 문서는 문단 추가·삭제가
    /// 아니라 **차이 없음**으로 읽힌다.
    pub ignore_whitespace: bool,
    /// 보고할 차이 개수 상한 — `None` 이면 무제한.
    ///
    /// 상한에 걸려도 순회는 끝까지 한다. 그래야
    /// [`DocumentDiff::truncated`] 가 "정말로 더 있었다"를 뜻한다.
    pub max_findings: Option<usize>,
}

impl DiffOptions {
    /// 공백 차이를 무시하도록 켠 사본.
    pub fn ignore_whitespace(mut self, yes: bool) -> Self {
        self.ignore_whitespace = yes;
        self
    }

    /// 보고 상한을 건 사본.
    pub fn max_findings(mut self, max: usize) -> Self {
        self.max_findings = Some(max);
        self
    }
}

/// 두 문서를 비교한 결과.
///
/// # 불변식
///
/// 1. `identical == true` 이면 `findings` 는 비어 있고 `truncated` 도 `false` 다.
/// 2. `truncated == true` 이면 상한 때문에 **버린 차이가 있다** — 결과는 부분 보고다.
/// 3. `findings` 의 순서는 문서 순서(구역 → 문단 → 컨트롤 → 셀, 끝으로 문서 정보)로
///    고정이다. 같은 두 문서는 언제나 같은 순서를 낸다.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentDiff {
    /// 차이가 하나도 없었는가 — 상한에 걸려 버린 것이 있으면 `false` 다.
    pub identical: bool,
    /// 발견한 차이(문서 순서).
    pub findings: Vec<Finding>,
    /// 상한 때문에 버린 차이가 있는가.
    pub truncated: bool,
}

impl DocumentDiff {
    /// 카테고리별 건수 집계.
    pub fn summary(&self) -> DiffSummary {
        super::summary::summarize(self)
    }

    /// 봉투 한 줄로 쓸 JSON 값 — `identical`/`truncated`/`findings`/`summary`.
    ///
    /// CLI·MCP 가 자기 봉투에 그대로 끼워 넣도록 **키만** 정하고 스키마 버전은
    /// 붙이지 않는다. 봉투의 주인은 이 엔진이 아니라 명령이다.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "identical": self.identical,
            "truncated": self.truncated,
            "findingCount": self.findings.len(),
            "findings": self.findings.iter().map(Finding::to_json).collect::<Vec<_>>(),
            "summary": self.summary().to_json(),
        })
    }
}

/// 카테고리별 건수 집계.
///
/// `by_kind` 는 [`FindingKind::ALL`] 선언 순서로 정렬돼 있고 **0 건은 빠진다**.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiffSummary {
    /// 보고된 차이 총 건수 — `findings.len()` 과 같다(버린 것은 안 센다).
    pub total: usize,
    /// 상한 때문에 버린 차이가 있는가.
    pub truncated: bool,
    /// (종류, 건수) — 종류 선언 순서, 0 건 제외.
    pub by_kind: Vec<(FindingKind, usize)>,
}

impl DiffSummary {
    /// 특정 종류의 건수 — 없으면 0.
    pub fn count_of(&self, kind: FindingKind) -> usize {
        self.by_kind
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, n)| *n)
            .unwrap_or(0)
    }

    /// 봉투 한 칸으로 쓸 JSON 값 — 종류 이름을 키로 한 객체.
    pub fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (kind, count) in &self.by_kind {
            map.insert(kind.label().to_string(), serde_json::json!(count));
        }
        serde_json::json!({
            "total": self.total,
            "truncated": self.truncated,
            "byKind": serde_json::Value::Object(map),
        })
    }
}
