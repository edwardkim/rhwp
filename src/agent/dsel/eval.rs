//! DSEL 평가기 — 선택자를 실제 `Document` 에 먹인다.
//!
//! ## 왜 문서를 한 번 펼치나
//!
//! 평가는 두 단계다. 먼저 문서를 **문서 순서의 평평한 목록**으로 펼치고
//! (`flatten`), 그 위에서 선택자를 접는다. 트리를 직접 재귀하며 맞추는 구현도
//! 가능하지만, 결합자 네 종(자손·직계·다음 형제·이후 형제)을 트리 재귀 안에서
//! 구현하면 각 결합자마다 다른 순회 방향이 필요해 코드가 네 갈래로 갈라진다.
//!
//! 펼쳐 두면 결합자가 전부 **부모 색인 비교**로 환원된다.
//!
//! | 결합자 | 판정 |
//! | --- | --- |
//! | `>` | `flat[n].parent == Some(c)` |
//! | ` ` | `c` 가 `n` 의 조상 사슬에 있다 |
//! | `+` | 같은 부모·같은 종류·`n.index == c.index + 1` |
//! | `~` | 같은 부모·같은 종류·`n.index > c.index` |
//!
//! 네 줄이 전부이고, 각각을 따로 검증할 수 있다. 대가는 노드 수만큼의 메모리인데
//! 상한([`EvalLimits::max_nodes`])으로 유계이므로 손상·거대 입력에서도 예측 가능한
//! 실패(에러)로 끝난다 — 예측 불가능한 실패(OOM)가 아니라.
//!
//! ## 위치 의사 선택자의 기준점
//!
//! `:first`·`:last`·`:nth`·`:range` 는 **그 스텝의 결과 집합**에서 센다. 형제
//! 중에서 세지 않는다. 즉 `table:last` 는 "문서에서 마지막으로 걸린 표"이지
//! "자기 문단의 마지막 표"가 아니다.
//!
//! CSS 의 `:last-child` 는 후자지만, 에이전트가 쓰는 표현은 거의 언제나 전자다
//! ("마지막 표의 합계 행"). 형제 기준이 필요하면 `[index]` 속성이 그대로 남아
//! 있다 — `table[index=0]` 은 자기 문단의 첫 표다. 두 기준을 **다른 문법에**
//! 두어 어느 쪽인지 항상 눈에 보이게 했다.
//!
//! ## 없는 값
//!
//! 속성이 없으면 그 술어는 **어떤 연산자로도 참이 되지 않는다** — `!=` 도
//! 마찬가지다. `cell[name!="합계"]` 가 이름 없는 셀을 전부 고르면, 이름을 붙이지
//! 않은 셀이 갑자기 대상이 되어 편집이 새어 나간다. 없는 것은 비교의 대상이
//! 아니라는 규칙이 편집 안전에서 더 옳다.

use std::collections::HashSet;

use crate::model::control::{Control, FieldType};
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;

use super::ast::{AttrPred, Axis, CmpOp, Combinator, Literal, Path, Pred, Pseudo, Selector, Step};
use super::error::SelectorError;
use super::glob::Glob;
use super::node::{
    control_kind_name, paragraphs_of_control, runs_of, Node, NodeId, NodeRef, NodeStep, PathStack,
};

/// 평가 상한.
///
/// 상한을 기본값으로 두는 이유: 호출부가 상한을 **잊을 수 있기** 때문이다.
/// 기본이 무한이면 잊은 호출부가 곧 취약점이 된다. 기본이 유한하면 잊은 호출부는
/// 그냥 정상 동작한다.
#[derive(Debug, Clone, Copy)]
pub struct EvalLimits {
    /// 펼칠 수 있는 최대 노드 수.
    pub max_nodes: usize,
    /// 최대 트리 깊이 — 표 안의 표 안의 표…를 막는다.
    pub max_depth: usize,
    /// 돌려줄 수 있는 최대 결과 수.
    pub max_results: usize,
}

impl Default for EvalLimits {
    fn default() -> Self {
        EvalLimits {
            // 실측 기준: 300쪽 공문서가 문단 1만 개 안쪽이다. 20만이면 그보다
            // 한 자릿수 위이므로 정상 문서를 막지 않으면서 폭주는 잡는다.
            max_nodes: 200_000,
            // 표 중첩은 한글에서도 실질 한계가 있다. 64 는 그보다 훨씬 깊다.
            max_depth: 64,
            max_results: 50_000,
        }
    }
}

/// 펼쳐진 노드 하나.
struct Flat<'d> {
    id: NodeId,
    node: Node<'d>,
    step: NodeStep,
    /// 부모의 `flat` 색인. 구역은 부모가 없다.
    parent: Option<usize>,
    /// 같은 종류 형제 중 순번.
    index: usize,
    /// 같은 종류 형제의 총수.
    sibling_count: usize,
}

/// 선택자를 기본 상한으로 평가한다.
pub fn select<'d>(sel: &Selector, doc: &'d Document) -> Result<Vec<NodeRef<'d>>, SelectorError> {
    select_with(sel, doc, EvalLimits::default())
}

/// 선택자를 주어진 상한으로 평가한다.
///
/// 결과는 **문서 순서**로 정렬되고 중복이 없다. 합집합 가지 여럿이 같은 노드를
/// 골라도 한 번만 나온다 — 편집이 같은 대상에 두 번 적용되는 사고를 여기서 막는다.
pub fn select_with<'d>(
    sel: &Selector,
    doc: &'d Document,
    limits: EvalLimits,
) -> Result<Vec<NodeRef<'d>>, SelectorError> {
    let flat = flatten(doc, &limits)?;
    let hits = eval_paths(&sel.paths, &flat, &limits, 0)?;

    if hits.len() > limits.max_results {
        return Err(SelectorError::limit(
            0,
            format!(
                "결과가 너무 많다 ({}건, 상한 {}건)",
                hits.len(),
                limits.max_results
            ),
        )
        .hinting("술어를 붙여 범위를 좁힌다"));
    }

    Ok(hits
        .into_iter()
        .map(|i| NodeRef {
            id: flat[i].id.clone(),
            node: flat[i].node,
            index: flat[i].index,
            sibling_count: flat[i].sibling_count,
        })
        .collect())
}

/// 선택자가 고른 노드 수만 센다 — 사전조건 검사가 쓴다.
///
/// 결과를 만들지 않으므로 `max_results` 상한에 걸리지 않는다. "몇 개인지"를
/// 알아야 상한을 넘겼는지 판단할 수 있는데, 세는 것조차 상한에 막히면 진단이
/// "너무 많다"에서 멈춰 몇 개인지 영영 알 수 없다.
pub fn count<'d>(
    sel: &Selector,
    doc: &'d Document,
    limits: EvalLimits,
) -> Result<usize, SelectorError> {
    let flat = flatten(doc, &limits)?;
    Ok(eval_paths(&sel.paths, &flat, &limits, 0)?.len())
}

// ---------------------------------------------------------------------------
// 펼치기
// ---------------------------------------------------------------------------

struct Walker<'d> {
    out: Vec<Flat<'d>>,
    stack: PathStack,
    limits: EvalLimits,
}

impl<'d> Walker<'d> {
    /// 노드 하나를 밀어 넣고 그 색인을 돌려준다.
    fn emit(
        &mut self,
        step: NodeStep,
        node: Node<'d>,
        parent: Option<usize>,
        index: usize,
        sibling_count: usize,
    ) -> Result<usize, SelectorError> {
        if self.out.len() >= self.limits.max_nodes {
            return Err(SelectorError::limit(
                0,
                format!("문서 노드가 너무 많다 (상한 {})", self.limits.max_nodes),
            ));
        }
        self.stack.push(step);
        let id = self.stack.snapshot();
        self.out.push(Flat {
            id,
            node,
            step,
            parent,
            index,
            sibling_count,
        });
        Ok(self.out.len() - 1)
    }

    fn check_depth(&self) -> Result<(), SelectorError> {
        if self.stack.depth() > self.limits.max_depth {
            return Err(SelectorError::limit(
                0,
                format!("문서 중첩이 너무 깊다 (상한 {})", self.limits.max_depth),
            ));
        }
        Ok(())
    }

    fn walk_paragraphs(
        &mut self,
        paras: &'d [Paragraph],
        parent: Option<usize>,
    ) -> Result<(), SelectorError> {
        self.check_depth()?;
        let total = paras.len();
        for (i, para) in paras.iter().enumerate() {
            let me = self.emit(NodeStep::Para(i as u32), Node::Para(para), parent, i, total)?;
            self.walk_para_children(para, me)?;
            self.stack.pop();
        }
        Ok(())
    }

    /// 문단의 자식 — 컨트롤과 구간. 순번은 종류별로 따로 센다.
    fn walk_para_children(
        &mut self,
        para: &'d Paragraph,
        parent: usize,
    ) -> Result<(), SelectorError> {
        self.check_depth()?;

        let control_total = para.controls.len();
        for (i, control) in para.controls.iter().enumerate() {
            let me = self.emit(
                NodeStep::Control(i as u32),
                Node::Control(control),
                Some(parent),
                i,
                control_total,
            )?;
            self.walk_control_children(control, me)?;
            self.stack.pop();
        }

        let runs = runs_of(para);
        let run_total = runs.len();
        for (i, run) in runs.into_iter().enumerate() {
            self.emit(
                NodeStep::Run(i as u32),
                Node::Run(run),
                Some(parent),
                i,
                run_total,
            )?;
            self.stack.pop();
        }

        Ok(())
    }

    fn walk_control_children(
        &mut self,
        control: &'d Control,
        parent: usize,
    ) -> Result<(), SelectorError> {
        self.check_depth()?;

        if let Control::Table(table) = control {
            let total = table.cells.len();
            for (i, cell) in table.cells.iter().enumerate() {
                let me = self.emit(
                    NodeStep::Cell(i as u32),
                    Node::Cell(cell),
                    Some(parent),
                    i,
                    total,
                )?;
                self.walk_paragraphs(&cell.paragraphs, Some(me))?;
                self.stack.pop();
            }
            return Ok(());
        }

        if let Some(paras) = paragraphs_of_control(control) {
            self.walk_paragraphs(paras, Some(parent))?;
        }
        Ok(())
    }
}

/// 문서를 문서 순서의 평평한 목록으로 펼친다.
///
/// 결과가 문서 순서로 **이미 정렬되어 있다**는 것이 뒤 단계의 전제다. 전위
/// 순회로 밀어 넣으므로 조상은 항상 자손보다 앞서고, 앞 형제는 뒤 형제보다
/// 앞선다 — 이는 `NodeId` 의 사전식 순서와 정확히 같다(`node` 모듈 참조).
fn flatten<'d>(doc: &'d Document, limits: &EvalLimits) -> Result<Vec<Flat<'d>>, SelectorError> {
    let mut w = Walker {
        out: Vec::new(),
        stack: PathStack::new(),
        limits: *limits,
    };
    let total = doc.sections.len();
    for (i, section) in doc.sections.iter().enumerate() {
        let me = w.emit(
            NodeStep::Section(i as u32),
            Node::Section(section),
            None,
            i,
            total,
        )?;
        w.walk_paragraphs(&section.paragraphs, Some(me))?;
        w.stack.pop();
    }
    Ok(w.out)
}

// ---------------------------------------------------------------------------
// 접기
// ---------------------------------------------------------------------------

/// 합집합 가지 전체를 평가하고 문서 순서로 합친다.
fn eval_paths(
    paths: &[Path],
    flat: &[Flat<'_>],
    limits: &EvalLimits,
    depth: usize,
) -> Result<Vec<usize>, SelectorError> {
    // 파서가 중첩 깊이를 이미 막지만, 평가기도 스스로를 지킨다. 두 곳 다
    // 막는 이유는 `eval` 이 파서를 거치지 않은 AST 로도 불릴 수 있기 때문이다
    // (계획서에서 역직렬화한 선택자 등).
    if depth > super::parse::MAX_NESTING {
        return Err(SelectorError::limit(
            0,
            format!(
                "중첩 선택자가 너무 깊다 (상한 {})",
                super::parse::MAX_NESTING
            ),
        ));
    }

    let mut seen = vec![false; flat.len()];
    for path in paths {
        for i in eval_path(path, flat, limits, depth)? {
            seen[i] = true;
        }
    }
    Ok((0..flat.len()).filter(|&i| seen[i]).collect())
}

/// 한 경로를 왼쪽에서 오른쪽으로 접는다.
fn eval_path(
    path: &Path,
    flat: &[Flat<'_>],
    limits: &EvalLimits,
    depth: usize,
) -> Result<Vec<usize>, SelectorError> {
    let mut current: Vec<usize> = Vec::new();

    for (si, step) in path.steps.iter().enumerate() {
        let candidates = if si == 0 {
            // 첫 스텝은 문서 어디서든 시작한다 — 뿌리의 자손 전부가 후보다.
            (0..flat.len())
                .filter(|&i| flat[i].node.matches_axis(step.axis))
                .collect()
        } else {
            reachable(step.combinator, &current, flat, step.axis)
        };

        current = apply_preds(candidates, step, flat, limits, depth)?;
        if current.is_empty() {
            break;
        }
    }

    Ok(current)
}

/// 결합자로 도달 가능한 후보들.
fn reachable(comb: Combinator, current: &[usize], flat: &[Flat<'_>], axis: Axis) -> Vec<usize> {
    let mut in_current = vec![false; flat.len()];
    for &i in current {
        in_current[i] = true;
    }

    // 형제 결합자는 (부모, 종류) 별로 현재 집합의 순번을 봐야 한다. 후보마다
    // `current` 를 전부 훑으면 O(후보 × 현재)가 되므로, 한 번만 접어 둔다.
    let mut sib_exact: HashSet<(usize, u8, usize)> = HashSet::new();
    let mut sib_min: std::collections::HashMap<(usize, u8), usize> =
        std::collections::HashMap::new();
    if matches!(comb, Combinator::NextSibling | Combinator::FollowingSibling) {
        for &c in current {
            let Some(p) = flat[c].parent else { continue };
            let key = (p, flat[c].step.kind_ord());
            sib_exact.insert((p, flat[c].step.kind_ord(), flat[c].index));
            sib_min
                .entry(key)
                .and_modify(|m| *m = (*m).min(flat[c].index))
                .or_insert(flat[c].index);
        }
    }

    (0..flat.len())
        .filter(|&n| flat[n].node.matches_axis(axis))
        .filter(|&n| match comb {
            Combinator::Root => true,
            Combinator::Child => flat[n].parent.is_some_and(|p| in_current[p]),
            Combinator::Descendant => {
                let mut cur = flat[n].parent;
                while let Some(p) = cur {
                    if in_current[p] {
                        return true;
                    }
                    cur = flat[p].parent;
                }
                false
            }
            Combinator::NextSibling => {
                let Some(p) = flat[n].parent else {
                    return false;
                };
                flat[n].index > 0
                    && sib_exact.contains(&(p, flat[n].step.kind_ord(), flat[n].index - 1))
            }
            Combinator::FollowingSibling => {
                let Some(p) = flat[n].parent else {
                    return false;
                };
                sib_min
                    .get(&(p, flat[n].step.kind_ord()))
                    .is_some_and(|&m| m < flat[n].index)
            }
        })
        .collect()
}

/// 술어를 적용한다 — 값 술어 먼저, 위치 술어 나중.
///
/// 순서가 뒤바뀌면 `table:last[rows>2]` 가 "마지막 표를 고른 뒤 행 수를 본다"가
/// 되어, 마지막 표의 행이 둘 이하면 결과가 빈다. 사람이 뜻한 것은 거의 언제나
/// "행이 셋 이상인 표들 중 마지막"이다.
fn apply_preds(
    candidates: Vec<usize>,
    step: &Step,
    flat: &[Flat<'_>],
    limits: &EvalLimits,
    depth: usize,
) -> Result<Vec<usize>, SelectorError> {
    let mut survivors = candidates;

    // 1단계 — 값 술어.
    for pred in &step.preds {
        if survivors.is_empty() {
            break;
        }
        match pred {
            Pred::Attr(attr) => {
                let glob = compile_glob_for(attr)?;
                survivors.retain(|&n| attr_matches(&flat[n], attr, glob.as_ref()));
            }
            Pred::Pseudo(Pseudo::Contains(needle)) => {
                survivors.retain(|&n| {
                    flat[n]
                        .node
                        .text()
                        .is_some_and(|t| visible(&t).contains(needle.as_str()))
                });
            }
            Pred::Pseudo(Pseudo::Matches(pattern)) => {
                let g = Glob::compile(pattern, step.offset)?;
                survivors.retain(|&n| {
                    flat[n]
                        .node
                        .text()
                        .is_some_and(|t| g.is_match(&visible(&t)))
                });
            }
            Pred::Pseudo(Pseudo::Empty) => {
                // 텍스트 개념이 없는 노드(그림 등)는 "비었다"가 성립하지 않는다.
                survivors.retain(|&n| flat[n].node.text().is_some_and(|t| visible(&t).is_empty()));
            }
            Pred::Pseudo(Pseudo::Not(inner)) => {
                let hits = eval_paths(&inner.paths, flat, limits, depth + 1)?;
                let set: HashSet<usize> = hits.into_iter().collect();
                survivors.retain(|n| !set.contains(n));
            }
            Pred::Pseudo(Pseudo::Has(inner)) => {
                let hits = eval_paths(&inner.paths, flat, limits, depth + 1)?;
                // 후보마다 전체 결과를 훑지 않도록, 결과의 조상 사슬을 한 번에
                // 접어 "자손을 가진 노드"의 집합으로 만든다.
                let mut has_desc = vec![false; flat.len()];
                for h in hits {
                    let mut cur = flat[h].parent;
                    while let Some(p) = cur {
                        if has_desc[p] {
                            break; // 위쪽은 이미 표시됐다.
                        }
                        has_desc[p] = true;
                        cur = flat[p].parent;
                    }
                }
                survivors.retain(|&n| has_desc[n]);
            }
            // 위치 술어는 2단계에서.
            Pred::Pseudo(Pseudo::First | Pseudo::Last | Pseudo::Nth(_) | Pseudo::Range { .. }) => {}
        }
    }

    // 2단계 — 위치 술어. 선언 순서대로 차례로 좁힌다.
    for pred in &step.preds {
        if survivors.is_empty() {
            break;
        }
        let Pred::Pseudo(p) = pred else { continue };
        let len = survivors.len();
        match p {
            Pseudo::First => survivors.truncate(1),
            Pseudo::Last => {
                let last = survivors[len - 1];
                survivors.clear();
                survivors.push(last);
            }
            Pseudo::Nth(n) => {
                survivors = match resolve_index(*n, len) {
                    Some(i) => vec![survivors[i]],
                    None => Vec::new(),
                };
            }
            Pseudo::Range { from, to } => {
                let lo = clamp_index(*from, len);
                let hi = clamp_index(*to, len);
                survivors = if lo < hi {
                    survivors[lo..hi].to_vec()
                } else {
                    Vec::new()
                };
            }
            _ => {}
        }
    }

    Ok(survivors)
}

/// 음수 인덱스를 뒤에서 센 위치로 바꾼다. 범위를 벗어나면 `None`.
fn resolve_index(n: i64, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let len_i = len as i64;
    let idx = if n < 0 { len_i + n } else { n };
    if idx < 0 || idx >= len_i {
        return None;
    }
    Some(idx as usize)
}

/// 범위 끝점을 0..=len 으로 조인다.
///
/// `resolve_index` 와 달리 범위를 벗어나도 실패하지 않는다 — `:range(0..999)` 는
/// "있는 만큼 전부"라는 뜻으로 읽는 편이 쓰는 사람의 의도에 맞는다.
fn clamp_index(n: i64, len: usize) -> usize {
    let len_i = len as i64;
    let idx = if n < 0 { len_i + n } else { n };
    idx.clamp(0, len_i) as usize
}

/// 글롭 비교자라면 패턴을 미리 컴파일한다.
fn compile_glob_for(attr: &AttrPred) -> Result<Option<Glob>, SelectorError> {
    match &attr.compare {
        Some((CmpOp::Glob, Literal::Str(pattern))) => {
            Ok(Some(Glob::compile(pattern, attr.offset)?))
        }
        _ => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// 속성
// ---------------------------------------------------------------------------

/// 속성값 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
enum AttrValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

/// 제어문자를 뺀 텍스트.
///
/// HWP 본문에는 컨트롤 자리를 표시하는 제어문자가 섞여 있다. 그대로 두면
/// `[text="합계"]` 가 눈에 보이기로는 "합계"인 문단에 걸리지 않는다 — 보이지 않는
/// 글자 때문에 선택자가 빗나가는 것은 진단조차 어렵다.
fn visible(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).collect()
}

/// 노드에서 속성값을 뽑는다. 없으면 `None`.
fn attr_of(flat: &Flat<'_>, name: &str) -> Option<AttrValue> {
    if name == "index" {
        return Some(AttrValue::Int(flat.index as i64));
    }

    match flat.node {
        Node::Section(s) => match name {
            "paras" => Some(AttrValue::Int(s.paragraphs.len() as i64)),
            _ => None,
        },
        Node::Para(p) => match name {
            "text" => Some(AttrValue::Str(visible(&p.text))),
            "len" => Some(AttrValue::Int(visible(&p.text).chars().count() as i64)),
            "styleId" => Some(AttrValue::Int(i64::from(p.style_id))),
            "shapeId" => Some(AttrValue::Int(i64::from(p.para_shape_id))),
            "empty" => Some(AttrValue::Bool(visible(&p.text).trim().is_empty())),
            "controls" => Some(AttrValue::Int(p.controls.len() as i64)),
            _ => None,
        },
        Node::Run(r) => match name {
            "text" => Some(AttrValue::Str(visible(r.text()))),
            "len" => Some(AttrValue::Int(visible(r.text()).chars().count() as i64)),
            "charShapeId" => Some(AttrValue::Int(i64::from(r.char_shape_id()))),
            _ => None,
        },
        Node::Cell(c) => match name {
            "row" => Some(AttrValue::Int(i64::from(c.row))),
            "col" => Some(AttrValue::Int(i64::from(c.col))),
            "rowSpan" => Some(AttrValue::Int(i64::from(c.row_span))),
            "colSpan" => Some(AttrValue::Int(i64::from(c.col_span))),
            "header" => Some(AttrValue::Bool(c.is_header)),
            "name" => c.field_name.as_ref().map(|n| AttrValue::Str(n.clone())),
            "text" => Some(AttrValue::Str(visible(
                &c.paragraphs
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n"),
            ))),
            _ => None,
        },
        Node::Control(control) => attr_of_control(control, name),
    }
}

fn attr_of_control(control: &Control, name: &str) -> Option<AttrValue> {
    match name {
        "kind" => return Some(AttrValue::Str(control_kind_name(control).to_string())),
        "inline" => return Some(AttrValue::Bool(control.is_treat_as_char_object())),
        _ => {}
    }

    match control {
        Control::Table(t) => match name {
            "rows" => Some(AttrValue::Int(i64::from(t.row_count))),
            "cols" => Some(AttrValue::Int(i64::from(t.col_count))),
            _ => None,
        },
        Control::Field(f) => match name {
            // 이름은 CTRL_DATA 쪽이 우선이다 — 누름틀 고치기가 여기에 쓰고,
            // `command` 는 안내문이라 사용자가 보는 이름과 다를 수 있다.
            "name" => {
                let raw = f.ctrl_data_name.as_deref().unwrap_or(f.command.as_str());
                // 빈 이름은 이름이 없는 것과 같다 — 빈 문자열을 값으로 내면
                // `[name]` 존재 검사가 참이 되어 "이름 있는 필드"를 잘못 센다.
                if raw.is_empty() {
                    None
                } else {
                    Some(AttrValue::Str(raw.to_string()))
                }
            }
            "type" => Some(AttrValue::Str(field_type_name(f.field_type).to_string())),
            _ => None,
        },
        Control::Bookmark(b) => match name {
            "name" if !b.name.is_empty() => Some(AttrValue::Str(b.name.clone())),
            _ => None,
        },
        _ => None,
    }
}

/// 필드 타입의 안정 이름.
///
/// `Debug` 로 찍지 않는 이유: `Debug` 출력은 계약이 아니다. 변종 이름을 리팩터링
/// 하면 선택자가 조용히 안 맞게 된다. 여기 적힌 문자열이 계약이다.
fn field_type_name(ty: FieldType) -> &'static str {
    match ty {
        FieldType::Unknown => "unknown",
        FieldType::Date => "date",
        FieldType::DocDate => "docDate",
        FieldType::Path => "path",
        FieldType::Bookmark => "bookmark",
        FieldType::MailMerge => "mailMerge",
        FieldType::CrossRef => "crossRef",
        FieldType::Formula => "formula",
        FieldType::ClickHere => "clickHere",
        FieldType::Summary => "summary",
        FieldType::UserInfo => "userInfo",
        FieldType::Hyperlink => "hyperlink",
        FieldType::Memo => "memo",
        FieldType::PrivateInfoSecurity => "privateInfoSecurity",
        FieldType::TableOfContents => "tableOfContents",
    }
}

/// 속성 술어 하나를 판정한다.
fn attr_matches(flat: &Flat<'_>, pred: &AttrPred, glob: Option<&Glob>) -> bool {
    let Some(value) = attr_of(flat, &pred.name) else {
        // 없는 값은 어떤 조건도 만족하지 않는다 — `!=` 도 포함. 모듈 문서 참조.
        return false;
    };

    let Some((op, literal)) = &pred.compare else {
        // 존재 검사. 불리언은 값 자체가 판정이다 — `[header]` 는 `[header=true]`.
        return match value {
            AttrValue::Bool(b) => b,
            _ => true,
        };
    };

    match (&value, literal) {
        (AttrValue::Int(a), Literal::Int(b)) => match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            CmpOp::Gt => a > b,
            CmpOp::Lt => a < b,
            CmpOp::Ge => a >= b,
            CmpOp::Le => a <= b,
            // 파서가 이미 막지만, 파서를 거치지 않은 AST 도 안전해야 한다.
            _ => false,
        },
        (AttrValue::Bool(a), Literal::Bool(b)) => match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            _ => false,
        },
        (AttrValue::Str(a), Literal::Str(b)) => match op {
            CmpOp::Eq => a == b,
            CmpOp::Ne => a != b,
            CmpOp::Prefix => a.starts_with(b.as_str()),
            CmpOp::Suffix => a.ends_with(b.as_str()),
            CmpOp::Substr => a.contains(b.as_str()),
            CmpOp::Glob => glob.is_some_and(|g| g.is_match(a)),
            _ => false,
        },
        // 타입이 어긋난 비교는 참이 될 수 없다.
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::dsel::parse;
    use crate::model::control::{Bookmark, Field, FieldType};
    use crate::model::document::Section;
    use crate::model::paragraph::CharShapeRef;
    use crate::model::table::{Cell, Table};

    fn para(text: &str) -> Paragraph {
        let mut p = Paragraph {
            text: text.to_string(),
            ..Default::default()
        };
        let mut utf16 = 0u32;
        for ch in text.chars() {
            p.char_offsets.push(utf16);
            utf16 += ch.len_utf16() as u32;
        }
        p
    }

    fn para_styled(text: &str, style_id: u8) -> Paragraph {
        let mut p = para(text);
        p.style_id = style_id;
        p
    }

    fn cell(row: u16, col: u16, text: &str) -> Cell {
        Cell {
            row,
            col,
            row_span: 1,
            col_span: 1,
            paragraphs: vec![para(text)],
            ..Default::default()
        }
    }

    /// 2×2 표 하나를 담은 문단.
    fn table_para(rows: u16, cols: u16, texts: &[&str]) -> Paragraph {
        let mut t = Table {
            row_count: rows,
            col_count: cols,
            ..Default::default()
        };
        for (i, text) in texts.iter().enumerate() {
            let r = (i as u16) / cols;
            let c = (i as u16) % cols;
            t.cells.push(cell(r, c, text));
        }
        let mut p = para("");
        p.controls.push(Control::Table(Box::new(t)));
        p
    }

    fn doc_with(paragraphs: Vec<Paragraph>) -> Document {
        Document {
            sections: vec![Section {
                paragraphs,
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn sel(src: &str, doc: &Document) -> Vec<String> {
        let s = parse(src).unwrap_or_else(|e| panic!("{}", e.render(src)));
        select(&s, doc)
            .unwrap_or_else(|e| panic!("{e}"))
            .into_iter()
            .map(|n| n.id.to_string())
            .collect()
    }

    fn texts(src: &str, doc: &Document) -> Vec<String> {
        let s = parse(src).unwrap();
        select(&s, doc)
            .unwrap()
            .into_iter()
            .filter_map(|n| n.node.text())
            .collect()
    }

    #[test]
    fn selects_paragraphs_in_document_order() {
        let doc = doc_with(vec![para("가"), para("나"), para("다")]);
        assert_eq!(
            sel("para", &doc),
            vec![
                "/section[0]/para[0]",
                "/section[0]/para[1]",
                "/section[0]/para[2]"
            ]
        );
    }

    #[test]
    fn first_step_reaches_any_depth() {
        // 셀 안 문단도 `para` 로 걸린다 — CSS 의 타입 선택자와 같은 규칙.
        let doc = doc_with(vec![para("바깥"), table_para(1, 2, &["안1", "안2"])]);
        let all = texts("para", &doc);
        assert!(all.contains(&"바깥".to_string()));
        assert!(all.contains(&"안1".to_string()));
    }

    #[test]
    fn child_combinator_does_not_cross_levels() {
        let doc = doc_with(vec![para("바깥"), table_para(1, 2, &["안1", "안2"])]);
        // 구역의 직계 문단만 — 셀 안 문단은 제외.
        let direct = texts("section > para", &doc);
        assert!(direct.contains(&"바깥".to_string()));
        assert!(!direct.contains(&"안1".to_string()));
    }

    #[test]
    fn descendant_combinator_crosses_levels() {
        let doc = doc_with(vec![table_para(1, 2, &["안1", "안2"])]);
        let inner = texts("table para", &doc);
        assert_eq!(inner, vec!["안1".to_string(), "안2".to_string()]);
    }

    #[test]
    fn cell_addressing_by_row_and_col() {
        let doc = doc_with(vec![table_para(2, 2, &["A", "B", "C", "D"])]);
        assert_eq!(texts("cell[row=1][col=0]", &doc), vec!["C".to_string()]);
    }

    #[test]
    fn sibling_combinators_respect_node_kind() {
        let doc = doc_with(vec![para("가"), para("나"), para("다")]);
        // `+` 는 바로 다음 형제 하나.
        assert_eq!(texts("para[index=0] + para", &doc), vec!["나".to_string()]);
        // `~` 는 이후 형제 전부.
        assert_eq!(
            texts("para[index=0] ~ para", &doc),
            vec!["나".to_string(), "다".to_string()]
        );
    }

    #[test]
    fn positional_pseudos_count_over_the_result_set() {
        let doc = doc_with(vec![
            table_para(1, 1, &["첫"]),
            table_para(1, 1, &["둘"]),
            table_para(1, 1, &["셋"]),
        ]);
        // 표 셋은 각각 다른 문단에 있으므로 형제 기준이라면 전부 `index=0` 이다.
        // 결과 집합 기준이므로 `:last` 는 세 번째 표를 고른다.
        assert_eq!(texts("table:last cell", &doc), vec!["셋".to_string()]);
        assert_eq!(texts("table:nth(1) cell", &doc), vec!["둘".to_string()]);
        assert_eq!(texts("table:first cell", &doc), vec!["첫".to_string()]);
    }

    #[test]
    fn negative_nth_counts_from_the_end() {
        let doc = doc_with(vec![para("가"), para("나"), para("다")]);
        assert_eq!(texts("para:nth(-1)", &doc), vec!["다".to_string()]);
        assert_eq!(texts("para:nth(-3)", &doc), vec!["가".to_string()]);
        // 범위를 벗어나면 빈 결과 — 패닉이 아니다.
        assert!(texts("para:nth(-9)", &doc).is_empty());
        assert!(texts("para:nth(9)", &doc).is_empty());
    }

    #[test]
    fn range_clamps_instead_of_failing() {
        let doc = doc_with(vec![para("가"), para("나"), para("다")]);
        assert_eq!(
            texts("para:range(1..99)", &doc),
            vec!["나".to_string(), "다".to_string()]
        );
        assert_eq!(texts("para:range(0..1)", &doc), vec!["가".to_string()]);
        assert!(texts("para:range(5..9)", &doc).is_empty());
    }

    #[test]
    fn value_predicates_run_before_positional_ones() {
        let doc = doc_with(vec![
            para_styled("가", 1),
            para_styled("나", 2),
            para_styled("다", 1),
        ]);
        // 스타일 1 인 문단들 중 마지막 = "다". 위치를 먼저 적용했다면 "다"를
        // 고른 뒤 스타일을 봐서 결과가 같겠지만, 스타일 2 로 물으면 갈린다.
        assert_eq!(texts("para[styleId=2]:last", &doc), vec!["나".to_string()]);
    }

    #[test]
    fn contains_and_matches_use_visible_text() {
        // 제어문자가 섞여 있어도 사람이 보는 글자로 걸려야 한다.
        let doc = doc_with(vec![para("합\u{0003}계")]);
        assert_eq!(texts(r#"para:contains("합계")"#, &doc).len(), 1);
        assert_eq!(texts(r#"para:matches("합*")"#, &doc).len(), 1);
        assert_eq!(texts(r#"para[text="합계"]"#, &doc).len(), 1);
    }

    #[test]
    fn empty_only_matches_nodes_that_have_text() {
        let mut p = para("");
        p.controls.push(Control::Bookmark(Bookmark {
            name: "표시".into(),
        }));
        let doc = doc_with(vec![p]);
        // 빈 문단은 걸린다.
        assert_eq!(sel("para:empty", &doc).len(), 1);
        // 텍스트 개념이 없는 책갈피는 걸리지 않는다.
        assert!(sel("bookmark:empty", &doc).is_empty());
    }

    #[test]
    fn not_excludes_the_inner_result_set() {
        let doc = doc_with(vec![para("가"), para(""), para("다")]);
        assert_eq!(
            texts("para:not(:empty)", &doc),
            vec!["가".to_string(), "다".to_string()]
        );
    }

    #[test]
    fn has_matches_ancestors_of_the_inner_result() {
        let doc = doc_with(vec![
            table_para(1, 1, &["합계"]),
            table_para(1, 1, &["기타"]),
        ]);
        let hits = sel(r#"table:has(cell:contains("합계"))"#, &doc);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].ends_with("/control[0]"));
        assert!(hits[0].starts_with("/section[0]/para[0]"));
    }

    #[test]
    fn union_paths_are_merged_in_document_order_without_duplicates() {
        let doc = doc_with(vec![para("가"), para("나")]);
        // 두 가지가 겹쳐도 한 번만 나온다.
        assert_eq!(sel("para, para[index=0]", &doc).len(), 2);
    }

    #[test]
    fn missing_attribute_never_matches_even_with_ne() {
        let doc = doc_with(vec![table_para(1, 1, &["A"])]);
        // 셀에 field_name 이 없다. `!=` 로도 걸리면 안 된다.
        assert!(sel(r#"cell[name!="합계"]"#, &doc).is_empty());
        assert!(sel("cell[name]", &doc).is_empty());
    }

    #[test]
    fn bare_boolean_attribute_means_true() {
        let mut t = Table {
            row_count: 1,
            col_count: 1,
            ..Default::default()
        };
        let mut c = cell(0, 0, "제목");
        c.is_header = true;
        t.cells.push(c);
        t.cells.push(cell(0, 1, "값"));
        let mut p = para("");
        p.controls.push(Control::Table(Box::new(t)));
        let doc = doc_with(vec![p]);
        assert_eq!(texts("cell[header]", &doc), vec!["제목".to_string()]);
        assert_eq!(texts("cell[header=false]", &doc), vec!["값".to_string()]);
    }

    #[test]
    fn field_name_prefers_ctrl_data_and_ignores_empty() {
        let mut p = para("");
        p.controls.push(Control::Field(Field {
            field_type: FieldType::ClickHere,
            command: "안내문".into(),
            ctrl_data_name: Some("수급자성명".into()),
            ..Default::default()
        }));
        p.controls.push(Control::Field(Field {
            field_type: FieldType::ClickHere,
            command: String::new(),
            ctrl_data_name: None,
            ..Default::default()
        }));
        let doc = doc_with(vec![p]);
        assert_eq!(sel(r#"field[name="수급자성명"]"#, &doc).len(), 1);
        // 이름이 빈 필드는 `[name]` 에 걸리지 않는다.
        assert_eq!(sel("field[name]", &doc).len(), 1);
        assert_eq!(sel(r#"field[type=clickHere]"#, &doc).len(), 2);
    }

    #[test]
    fn table_axis_and_control_kind_select_the_same_thing() {
        let doc = doc_with(vec![table_para(1, 1, &["A"])]);
        assert_eq!(sel("table", &doc), sel("control[kind=table]", &doc));
    }

    #[test]
    fn runs_are_selectable_by_char_shape() {
        let mut p = para("가나다라");
        p.char_shapes = vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 7,
            },
            CharShapeRef {
                start_pos: 2,
                char_shape_id: 9,
            },
        ];
        let doc = doc_with(vec![p]);
        assert_eq!(texts("run[charShapeId=9]", &doc), vec!["다라".to_string()]);
    }

    #[test]
    fn node_limit_is_enforced_as_an_error_not_an_oom() {
        let doc = doc_with((0..50).map(|i| para(&format!("문단{i}"))).collect());
        let s = parse("para").unwrap();
        let limits = EvalLimits {
            max_nodes: 10,
            ..Default::default()
        };
        let err = select_with(&s, &doc, limits).unwrap_err();
        assert!(err.message.contains("노드가 너무 많다"));
    }

    #[test]
    fn result_limit_is_enforced() {
        let doc = doc_with((0..20).map(|i| para(&format!("문단{i}"))).collect());
        let s = parse("para").unwrap();
        let limits = EvalLimits {
            max_results: 5,
            ..Default::default()
        };
        let err = select_with(&s, &doc, limits).unwrap_err();
        assert!(err.message.contains("결과가 너무 많다"));
        // 세는 것은 상한에 막히지 않는다.
        assert_eq!(count(&s, &doc, limits).unwrap(), 20);
    }

    #[test]
    fn empty_document_selects_nothing_without_error() {
        let doc = Document::default();
        assert!(sel("para", &doc).is_empty());
        assert!(sel("table cell", &doc).is_empty());
    }

    #[test]
    fn index_attribute_stays_sibling_relative() {
        // 위치 의사 선택자는 결과 집합 기준이지만 `index` 는 형제 기준이다.
        // 두 기준이 같은 문법을 쓰면 어느 쪽인지 알 수 없게 된다.
        let doc = doc_with(vec![table_para(1, 1, &["첫"]), table_para(1, 1, &["둘"])]);
        // 표 둘 다 자기 문단의 첫 컨트롤이므로 index=0 이 둘 다 걸린다.
        assert_eq!(sel("table[index=0]", &doc).len(), 2);
        // 결과 집합 기준인 :first 는 하나만 고른다.
        assert_eq!(sel("table:first", &doc).len(), 1);
    }
}
