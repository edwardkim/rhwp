//! 노드 주소와 노드 뷰 — 선택자가 고른 "어디"를 값으로 만든다.
//!
//! ## 왜 주소가 따로 필요한가
//!
//! 선택자의 결과는 `&Paragraph` 같은 참조로도 표현할 수 있다. 하지만 참조는
//! **문서를 고치는 순간 죽는다**. 커널의 요점은 고르고 → 고치고 → 확인하는
//! 것이므로, 고른 결과는 편집을 사이에 두고 살아남는 표현이어야 한다. 그래서
//! 선택 결과는 [`NodeId`] — 뿌리에서 내려오는 **구조 경로**다.
//!
//! ```text
//! /section[0]/para[3]/control[0]/cell[5]/para[0]
//!  └──┬───┘ └──┬──┘ └───┬────┘ └──┬──┘ └──┬──┘
//!   구역     문단      표(컨트롤)   셀     셀 안 문단
//! ```
//!
//! 이 경로는 사람이 읽을 수 있고, JSON 에 그대로 실리고, 파일을 다시 열어도 같은
//! 곳을 가리킨다(문서가 그대로라면). 편집으로 앞 형제가 사라지면 경로가 다른 것을
//! 가리키게 되는데 — 그 문제를 푸는 것이 앵커 층이고, 앵커가 무엇을 고정할지
//! 알려면 먼저 이 주소가 있어야 한다.
//!
//! ## 순번의 뜻
//!
//! `index` 는 **같은 종류의 형제 중** 순번이다. 한 문단이 컨트롤 셋과 구간 둘을
//! 가지면 컨트롤은 0·1·2, 구간은 0·1 로 각각 센다. 섞어서 세면
//! `control[index=1]` 이 "두 번째 컨트롤"이 아니라 "두 번째 자식"이 되어, 문단에
//! 글자 구간이 하나 늘 때마다 뜻이 바뀐다.
//!
//! ## 구간(run)은 어디서 오나
//!
//! `Paragraph::char_shapes` 가 나눈다. 글자 모양 변경점이 곧 구간 경계다.
//! `char_shapes` 가 비어 있으면 구간은 **0 개**다 — 텍스트가 있어도 그렇다.
//! 비었을 때 "전체를 덮는 구간 하나"를 지어내면 존재하지 않는 글자 모양 ID 0 을
//! 사실인 것처럼 싣게 된다. 없는 것은 없다고 하는 편이 정확하다.

use std::fmt;

use crate::model::control::Control;
use crate::model::document::Section;
use crate::model::paragraph::Paragraph;
use crate::model::table::Cell;

use super::ast::{Axis, AxisKind};

/// 트리 경로의 한 마디.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeStep {
    Section(u32),
    Para(u32),
    Control(u32),
    Cell(u32),
    Run(u32),
}

impl NodeStep {
    /// 경로 표기에 쓰는 이름.
    pub const fn axis_name(self) -> &'static str {
        match self {
            NodeStep::Section(_) => "section",
            NodeStep::Para(_) => "para",
            NodeStep::Control(_) => "control",
            NodeStep::Cell(_) => "cell",
            NodeStep::Run(_) => "run",
        }
    }

    /// 마디 종류의 구분 번호 — 형제 판정에 쓴다.
    ///
    /// 한 문단은 컨트롤과 구간을 함께 갖는다. `+`(다음 형제)가 종류를 보지 않으면
    /// 컨트롤 다음에 오는 구간이 "다음 형제"가 되어, 글자 모양이 하나 바뀔 때마다
    /// 같은 선택자가 다른 것을 고른다.
    pub const fn kind_ord(self) -> u8 {
        match self {
            NodeStep::Section(_) => 0,
            NodeStep::Para(_) => 1,
            NodeStep::Control(_) => 2,
            NodeStep::Cell(_) => 3,
            NodeStep::Run(_) => 4,
        }
    }

    /// 이 마디의 순번.
    pub const fn index(self) -> u32 {
        match self {
            NodeStep::Section(i)
            | NodeStep::Para(i)
            | NodeStep::Control(i)
            | NodeStep::Cell(i)
            | NodeStep::Run(i) => i,
        }
    }
}

impl fmt::Display for NodeStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "/{}[{}]", self.axis_name(), self.index())
    }
}

/// 문서 뿌리에서 노드까지의 구조 경로.
///
/// `Ord` 를 유도하는 이유: 선택 결과를 **문서 순서**로 정렬해야 하기 때문이다.
/// 경로의 사전식 순서가 곧 문서 순서다 — 앞 형제는 순번이 작고, 조상은 접두사다.
/// 이 성질 덕에 정렬에 별도 비교기가 필요 없다.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId {
    steps: Vec<NodeStep>,
}

impl NodeId {
    /// 문서 뿌리.
    pub fn root() -> NodeId {
        NodeId { steps: Vec::new() }
    }

    /// 마디를 덧붙인 새 경로.
    pub fn child(&self, step: NodeStep) -> NodeId {
        let mut steps = self.steps.clone();
        steps.push(step);
        NodeId { steps }
    }

    /// 경로 마디들.
    pub fn steps(&self) -> &[NodeStep] {
        &self.steps
    }

    /// 뿌리로부터의 깊이.
    pub fn depth(&self) -> usize {
        self.steps.len()
    }

    /// 부모 경로. 뿌리면 `None`.
    pub fn parent(&self) -> Option<NodeId> {
        if self.steps.is_empty() {
            return None;
        }
        Some(NodeId {
            steps: self.steps[..self.steps.len() - 1].to_vec(),
        })
    }

    /// `other` 가 이 경로의 조상인가. 자기 자신은 조상이 아니다.
    ///
    /// 자손 결합자(` `)와 `:has` 가 이 판정을 쓴다. 자기 자신을 조상으로 치면
    /// `table table` 이 같은 표를 두 번 고른다.
    pub fn is_descendant_of(&self, other: &NodeId) -> bool {
        self.steps.len() > other.steps.len() && self.steps.starts_with(&other.steps)
    }

    /// 같은 부모를 갖는가.
    pub fn is_sibling_of(&self, other: &NodeId) -> bool {
        self.steps.len() == other.steps.len()
            && !self.steps.is_empty()
            && self.steps[..self.steps.len() - 1] == other.steps[..other.steps.len() - 1]
    }
}

impl fmt::Display for NodeId {
    /// `/section[0]/para[3]` 형태. 뿌리는 `/`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.steps.is_empty() {
            return f.write_str("/");
        }
        for step in &self.steps {
            write!(f, "{step}")?;
        }
        Ok(())
    }
}

/// 문단 안의 글자 구간 하나.
///
/// 텍스트를 사본으로 들지 않고 문단과 바이트 범위만 든다 — 구간은 문단마다
/// 여러 개고, 선택자 하나가 문서 전체의 구간을 훑을 수 있어서 사본 비용이
/// 곱해진다.
#[derive(Debug, Clone, Copy)]
pub struct RunView<'d> {
    para: &'d Paragraph,
    byte_start: usize,
    byte_end: usize,
    char_shape_id: u32,
}

impl<'d> RunView<'d> {
    /// 구간 텍스트.
    pub fn text(&self) -> &'d str {
        // 경계는 생성 시점에 char_indices 로 구했으므로 항상 글자 경계다.
        &self.para.text[self.byte_start..self.byte_end]
    }

    /// 이 구간의 글자 모양 ID.
    pub const fn char_shape_id(&self) -> u32 {
        self.char_shape_id
    }

    /// 구간이 속한 문단.
    pub const fn paragraph(&self) -> &'d Paragraph {
        self.para
    }
}

/// 문단을 글자 모양 변경점으로 잘라 구간 목록을 만든다.
///
/// ## UTF-16 ↔ 글자 인덱스
///
/// `CharShapeRef::start_pos` 는 **UTF-16 코드 유닛** 위치다. 문단 텍스트는 Rust
/// `String`(UTF-8)이므로 그대로 자를 수 없다. `Paragraph::char_offsets[i]` 가
/// "i 번째 글자의 UTF-16 위치"를 주므로, 이분 탐색으로 되돌린다.
///
/// ## 손상 입력 방어
///
/// `char_offsets` 가 텍스트와 길이가 어긋나거나(손상 문서), `start_pos` 가
/// 증가하지 않거나, 범위를 넘는 경우가 실제로 있다. 그런 입력에서 패닉하지
/// 않는 것이 이 함수의 계약이다 — 자를 수 없으면 잘못 자르는 대신 **구간을
/// 만들지 않는다**. 선택자가 아무것도 못 고르는 것은 복구 가능하지만, 패닉은
/// 아니다.
pub fn runs_of(para: &Paragraph) -> Vec<RunView<'_>> {
    if para.char_shapes.is_empty() || para.text.is_empty() {
        return Vec::new();
    }

    // 글자 인덱스 → 바이트 오프셋 표. 끝에 전체 길이를 하나 더 달아 두면
    // 마지막 구간의 끝을 특수 처리하지 않아도 된다.
    let mut byte_at: Vec<usize> = para.text.char_indices().map(|(b, _)| b).collect();
    byte_at.push(para.text.len());
    let char_count = byte_at.len() - 1;

    // UTF-16 위치 → 글자 인덱스.
    let to_char_index = |utf16_pos: u32| -> usize {
        if para.char_offsets.len() == char_count {
            // 정상 경로 — 오름차순이라고 가정하되, 손상 시에도 partition_point 는
            // 임의의 값을 돌려줄 뿐 패닉하지 않는다.
            para.char_offsets.partition_point(|&o| o < utf16_pos)
        } else {
            // 표가 어긋난 문서. UTF-16 위치를 글자 위치로 그대로 보되 범위를 조인다.
            // 비ASCII 문서에서는 틀린 위치지만, 틀린 위치는 잘못된 선택일 뿐이고
            // 범위를 넘는 슬라이스는 패닉이다.
            (utf16_pos as usize).min(char_count)
        }
    };

    let mut starts: Vec<usize> = para
        .char_shapes
        .iter()
        .map(|cs| to_char_index(cs.start_pos).min(char_count))
        .collect();
    // 첫 구간은 반드시 0 에서 시작한다. 손상 문서에서 첫 start_pos 가 0 이
    // 아니면 앞부분이 어느 구간에도 속하지 않게 되므로 끌어내린다.
    starts[0] = 0;

    let mut runs = Vec::with_capacity(starts.len());
    for (i, &start) in starts.iter().enumerate() {
        let end = starts.get(i + 1).copied().unwrap_or(char_count);
        // 비단조(손상) 구간은 건너뛴다 — 뒤집힌 범위로 슬라이스하면 패닉이다.
        if end <= start {
            continue;
        }
        runs.push(RunView {
            para,
            byte_start: byte_at[start],
            byte_end: byte_at[end],
            char_shape_id: para.char_shapes[i].char_shape_id,
        });
    }
    runs
}

/// 선택 대상 노드 하나 — 종류별 뷰.
#[derive(Debug, Clone, Copy)]
pub enum Node<'d> {
    Section(&'d Section),
    Para(&'d Paragraph),
    Run(RunView<'d>),
    Control(&'d Control),
    Cell(&'d Cell),
}

impl<'d> Node<'d> {
    /// 이 노드가 축에 맞나.
    ///
    /// 컨트롤 특수화(`table`·`field` …)는 변종까지 본다 — `table` 은
    /// `control[kind=table]` 과 같은 것을 골라야 한다는 `ast` 의 계약이 여기서
    /// 실현된다.
    pub fn matches_axis(&self, axis: Axis) -> bool {
        let kind = match axis {
            Axis::Any => return true,
            Axis::Kind(k) => k,
        };
        match (kind, self) {
            (AxisKind::Section, Node::Section(_)) => true,
            (AxisKind::Para, Node::Para(_)) => true,
            (AxisKind::Run, Node::Run(_)) => true,
            (AxisKind::Cell, Node::Cell(_)) => true,
            (AxisKind::Control, Node::Control(_)) => true,
            (k, Node::Control(c)) => control_axis(c) == Some(k),
            _ => false,
        }
    }

    /// 이 노드의 축 이름 — 진단과 봉투 출력에 쓴다.
    pub fn axis_name(&self) -> &'static str {
        match self {
            Node::Section(_) => "section",
            Node::Para(_) => "para",
            Node::Run(_) => "run",
            Node::Cell(_) => "cell",
            Node::Control(c) => control_axis(c).map_or("control", AxisKind::name),
        }
    }

    /// 이 노드의 텍스트 — `:contains`·`:empty`·`text` 속성의 원천.
    ///
    /// 셀·각주처럼 문단을 담는 노드는 자손 문단 텍스트를 개행으로 잇는다.
    /// 컨트롤 일반은 텍스트가 없다(`None`) — 빈 문자열과 구분해야 `[text=""]`
    /// 이 "빈 텍스트"만 고르고 "텍스트 개념이 없는 것"은 고르지 않는다.
    pub fn text(&self) -> Option<String> {
        match self {
            Node::Section(s) => Some(join_paragraphs(&s.paragraphs)),
            Node::Para(p) => Some(p.text.clone()),
            Node::Run(r) => Some(r.text().to_string()),
            Node::Cell(c) => Some(join_paragraphs(&c.paragraphs)),
            Node::Control(c) => paragraphs_of_control(c).map(join_paragraphs),
        }
    }
}

/// 문단 목록의 텍스트를 개행으로 잇는다.
fn join_paragraphs(paras: &[Paragraph]) -> String {
    let mut out = String::new();
    for (i, p) in paras.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&p.text);
    }
    out
}

/// 컨트롤이 담은 문단 목록. 담지 않으면 `None`.
///
/// `Field::memo_paragraphs` 를 여기에 넣지 않는 이유: 메모 본문은 본문 흐름이
/// 아니라 주석이다. `field para` 가 메모 속 문단을 본문 문단과 같은 자격으로
/// 고르면, 본문만 훑으려던 선택자가 조용히 주석까지 고친다.
pub fn paragraphs_of_control(control: &Control) -> Option<&[Paragraph]> {
    match control {
        Control::Footnote(f) => Some(&f.paragraphs),
        Control::Endnote(e) => Some(&e.paragraphs),
        Control::Header(h) => Some(&h.paragraphs),
        Control::Footer(f) => Some(&f.paragraphs),
        _ => None,
    }
}

/// 컨트롤 변종에 대응하는 전용 축. 전용 축이 없으면 `None`.
pub fn control_axis(control: &Control) -> Option<AxisKind> {
    match control {
        Control::Table(_) => Some(AxisKind::Table),
        Control::Picture(_) => Some(AxisKind::Picture),
        Control::Equation(_) => Some(AxisKind::Equation),
        Control::Field(_) => Some(AxisKind::Field),
        Control::Footnote(_) => Some(AxisKind::Footnote),
        Control::Endnote(_) => Some(AxisKind::Endnote),
        Control::Header(_) => Some(AxisKind::Header),
        Control::Footer(_) => Some(AxisKind::Footer),
        Control::Bookmark(_) => Some(AxisKind::Bookmark),
        Control::Hyperlink(_) => Some(AxisKind::Hyperlink),
        Control::Shape(_) => Some(AxisKind::Shape),
        _ => None,
    }
}

/// `kind` 속성이 내는 이름 — 전용 축이 있으면 축 이름, 없으면 고유 이름.
///
/// 전용 축이 없는 컨트롤도 이름을 갖는다. 이름이 없으면
/// `control[kind=…]` 로 걸러 낼 방법이 사라져서, 축을 만들지 않은 컨트롤은
/// 아예 지목 불가능해진다.
pub fn control_kind_name(control: &Control) -> &'static str {
    if let Some(axis) = control_axis(control) {
        return axis.name();
    }
    match control {
        Control::SectionDef(_) => "sectionDef",
        Control::ColumnDef(_) => "columnDef",
        Control::AutoNumber(_) => "autoNumber",
        Control::NewNumber(_) => "newNumber",
        Control::PageNumberPos(_) => "pageNumberPos",
        Control::Ruby(_) => "ruby",
        Control::CharOverlap(_) => "charOverlap",
        Control::PageHide(_) => "pageHide",
        Control::HiddenComment(_) => "hiddenComment",
        Control::Form(_) => "form",
        Control::Unknown(_) => "unknown",
        // 전용 축이 있는 변종은 위에서 이미 돌아갔다.
        _ => "control",
    }
}

/// 주소가 붙은 노드 — 선택 결과의 단위.
///
/// ## 경로를 언제 만드나
///
/// 순회 중에는 만들지 않는다. 훑는 노드마다 `Vec<NodeStep>` 을 복제하면 문서
/// 크기에 비례해 할당이 쌓이는데, 실제로 경로가 필요한 것은 **후보로 살아남은**
/// 노드뿐이다. 그래서 순회기는 경로 스택 하나를 밀고 당기며 재사용하고
/// ([`PathStack`]), 후보가 확정되는 순간에만 [`PathStack::snapshot`] 으로 접는다.
#[derive(Debug, Clone)]
pub struct NodeRef<'d> {
    /// 뿌리로부터의 경로.
    pub id: NodeId,
    /// 노드 뷰.
    pub node: Node<'d>,
    /// 같은 종류 형제 중 순번 (0 기준).
    pub index: usize,
    /// 같은 종류 형제의 총수 — `:last` 와 음수 인덱스가 쓴다.
    pub sibling_count: usize,
}

/// 순회 중 경로를 재사용하는 스택.
///
/// 소유 `NodeId` 를 노드마다 만들지 않으려는 장치다. 깊이가 유한하고
/// (문서 중첩 깊이), 밀고 당기는 비용이 상수이므로 순회 전체가 할당 0 에
/// 가깝게 돈다.
#[derive(Debug, Clone, Default)]
pub struct PathStack {
    steps: Vec<NodeStep>,
}

impl PathStack {
    pub fn new() -> PathStack {
        PathStack { steps: Vec::new() }
    }

    pub fn push(&mut self, step: NodeStep) {
        self.steps.push(step);
    }

    pub fn pop(&mut self) {
        self.steps.pop();
    }

    /// 현재 경로를 소유 값으로 접는다.
    pub fn snapshot(&self) -> NodeId {
        NodeId {
            steps: self.steps.clone(),
        }
    }

    pub fn depth(&self) -> usize {
        self.steps.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::CharShapeRef;

    fn para_with(text: &str, shapes: &[(u32, u32)]) -> Paragraph {
        let mut p = Paragraph {
            text: text.to_string(),
            ..Default::default()
        };
        // char_offsets: 글자 i 의 UTF-16 위치.
        let mut utf16 = 0u32;
        for ch in text.chars() {
            p.char_offsets.push(utf16);
            utf16 += ch.len_utf16() as u32;
        }
        p.char_shapes = shapes
            .iter()
            .map(|(start_pos, char_shape_id)| CharShapeRef {
                start_pos: *start_pos,
                char_shape_id: *char_shape_id,
            })
            .collect();
        p
    }

    #[test]
    fn node_id_renders_as_a_path() {
        let id = NodeId::root()
            .child(NodeStep::Section(0))
            .child(NodeStep::Para(3))
            .child(NodeStep::Control(0))
            .child(NodeStep::Cell(5));
        assert_eq!(id.to_string(), "/section[0]/para[3]/control[0]/cell[5]");
        assert_eq!(NodeId::root().to_string(), "/");
    }

    #[test]
    fn lexicographic_order_is_document_order() {
        let a = NodeId::root()
            .child(NodeStep::Section(0))
            .child(NodeStep::Para(1));
        let b = NodeId::root()
            .child(NodeStep::Section(0))
            .child(NodeStep::Para(2));
        let c = NodeId::root()
            .child(NodeStep::Section(1))
            .child(NodeStep::Para(0));
        let mut v = vec![c.clone(), b.clone(), a.clone()];
        v.sort();
        assert_eq!(v, vec![a, b, c]);
    }

    #[test]
    fn ancestry_excludes_self() {
        let parent = NodeId::root().child(NodeStep::Section(0));
        let child = parent.child(NodeStep::Para(0));
        assert!(child.is_descendant_of(&parent));
        assert!(!parent.is_descendant_of(&parent));
        assert!(!parent.is_descendant_of(&child));
    }

    #[test]
    fn siblings_share_a_parent() {
        let base = NodeId::root().child(NodeStep::Section(0));
        let a = base.child(NodeStep::Para(0));
        let b = base.child(NodeStep::Para(1));
        let other = NodeId::root()
            .child(NodeStep::Section(1))
            .child(NodeStep::Para(0));
        assert!(a.is_sibling_of(&b));
        assert!(!a.is_sibling_of(&other));
        // 뿌리는 형제가 없다.
        assert!(!NodeId::root().is_sibling_of(&NodeId::root()));
    }

    #[test]
    fn runs_split_at_char_shape_boundaries() {
        let p = para_with("abcdef", &[(0, 10), (3, 20)]);
        let runs = runs_of(&p);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].text(), "abc");
        assert_eq!(runs[0].char_shape_id(), 10);
        assert_eq!(runs[1].text(), "def");
        assert_eq!(runs[1].char_shape_id(), 20);
    }

    #[test]
    fn runs_use_utf16_positions_not_byte_positions() {
        // 한글은 UTF-16 1 유닛, UTF-8 3 바이트. start_pos 2 는 세 번째 글자다.
        let p = para_with("가나다라", &[(0, 1), (2, 2)]);
        let runs = runs_of(&p);
        assert_eq!(runs[0].text(), "가나");
        assert_eq!(runs[1].text(), "다라");
    }

    #[test]
    fn surrogate_pair_counts_as_two_utf16_units() {
        // 😀 는 UTF-16 2 유닛이다. start_pos 2 는 그 다음 글자를 가리킨다.
        let p = para_with("😀ab", &[(0, 1), (2, 2)]);
        let runs = runs_of(&p);
        assert_eq!(runs[0].text(), "😀");
        assert_eq!(runs[1].text(), "ab");
    }

    #[test]
    fn empty_char_shapes_yield_no_runs() {
        let p = para_with("abc", &[]);
        assert!(runs_of(&p).is_empty());
    }

    #[test]
    fn corrupt_offsets_do_not_panic() {
        // char_offsets 가 텍스트와 어긋난 문서.
        let mut p = para_with("가나다", &[(0, 1), (99, 2)]);
        p.char_offsets.clear();
        let runs = runs_of(&p);
        // 자를 수 없으면 덜 자를 뿐, 패닉하지 않는다.
        assert!(runs.iter().all(|r| !r.text().is_empty()));
    }

    #[test]
    fn non_monotonic_shape_positions_are_skipped_not_panicked() {
        let p = para_with("abcdef", &[(0, 1), (5, 2), (2, 3)]);
        let runs = runs_of(&p);
        // 뒤집힌 구간은 버린다. 남은 구간은 전부 유효한 슬라이스여야 한다.
        assert!(runs.iter().all(|r| !r.text().is_empty()));
    }

    #[test]
    fn first_shape_is_pulled_down_to_zero() {
        // 손상 문서에서 첫 start_pos 가 0 이 아니면 앞부분이 유실된다.
        let p = para_with("abcdef", &[(2, 7)]);
        let runs = runs_of(&p);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text(), "abcdef");
    }

    #[test]
    fn every_control_specialization_is_addressable_as_a_kind() {
        // 전용 축을 가진 컨트롤은 `control[kind=<축이름>]` 로도 지목할 수 있어야
        // 한다. 어긋나면 `table` 과 `control[kind=table]` 이 다른 것을 고른다.
        use super::super::ast::AXIS_NAMES;
        for (_, kind) in AXIS_NAMES {
            if kind.specializes() == Some(AxisKind::Control) {
                assert!(
                    AXIS_NAMES.iter().any(|(n, _)| *n == kind.name()),
                    "`{}` 가 kind 어휘에 없다",
                    kind.name()
                );
            }
        }
    }

    #[test]
    fn path_stack_snapshots_the_current_path() {
        let mut st = PathStack::new();
        st.push(NodeStep::Section(2));
        st.push(NodeStep::Para(7));
        assert_eq!(st.snapshot().to_string(), "/section[2]/para[7]");
        assert_eq!(st.depth(), 2);
        st.pop();
        assert_eq!(st.snapshot().to_string(), "/section[2]");
    }
}
