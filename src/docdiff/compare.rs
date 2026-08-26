//! 비교 엔진 — 두 `Document` IR 을 의미 수준에서 맞대 본다.
//!
//! ## 왜 자리끼리 맞대지 않는가
//!
//! 문단을 `a[i]` 대 `b[i]` 로 맞대면 **문단 하나가 앞에 끼어드는 순간** 뒤따르는 모든
//! 문단이 "텍스트 바뀜"으로 보고된다. 한 줄 삽입이 수백 건의 거짓 차이가 되는 것이다.
//! 그래서 이 엔진은 문단 목록을 먼저 **정렬(alignment)** 한다 — 공통 앞뒤를 깎아내고
//! 남은 가운데만 최장 공통 부분수열(LCS)로 짝지은 뒤, 서로 맞닿은 삭제·추가 덩어리를
//! 다시 짝지어 "바뀐 문단"으로 승격시킨다. 그 결과 한 글자 수정은 `TextChanged` 1 건,
//! 한 문단 삽입은 `ParagraphAdded` 1 건이 된다.
//!
//! ## 결정성
//!
//! 순회는 문서 순서로 고정이고, LCS 되짚기는 갈림길에서 **언제나 삭제를 먼저** 고른다.
//! 해시 자료구조를 돌지 않으므로 같은 두 문서는 언제나 같은 결과·같은 순서를 낸다.

use std::borrow::Cow;

use super::model::{DiffOptions, DocumentDiff, Finding, FindingKind, NodePath, PathStep};
use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;
use crate::model::style::Style;
use crate::model::table::Table;

/// LCS 표가 쓸 수 있는 최대 칸 수 — 넘으면 자리끼리 맞대는 방식으로 물러선다.
///
/// 칸당 `u32` 4 바이트라 최대 4 MB 다. 문단 1000 × 1000 쌍까지는 정렬이 돌고, 그보다
/// 큰 덩어리는 정렬 없이 자리끼리 비교한다(결과는 여전히 결정적이다).
const LCS_CELL_BUDGET: usize = 1_000_000;

/// 표 중첩 재귀의 안전 상한 — 실제 문서의 중첩 깊이를 한참 넘는 스택 보호용이다.
const MAX_DEPTH: usize = 32;

/// `detail` 미리보기의 최대 글자 수(바이트가 아니라 문자다 — 한글이 잘리면 안 된다).
const PREVIEW_CHARS: usize = 40;

/// 두 문서의 구조적 차이를 계산한다.
///
/// 입력은 이미 파싱된 IR 이다 — 이 함수는 **파일을 열지 않는다.** 어떤 포맷에서
/// 왔든(HWP5·HWPX·HWP3) 같은 `Document` 로 들어오면 같은 규칙으로 비교한다.
///
/// 결과의 불변식은 [`DocumentDiff`] 문서를 참고한다. 특히 `opts.max_findings` 에
/// 걸리더라도 **순회는 끝까지** 하므로 `truncated` 는 "정말로 더 있었다"를 뜻한다.
///
/// ```
/// use rhwp::docdiff::{diff_documents, DiffOptions};
/// use rhwp::model::document::Document;
///
/// let a = Document::default();
/// let b = Document::default();
/// let diff = diff_documents(&a, &b, &DiffOptions::default());
/// assert!(diff.identical);
/// assert!(diff.findings.is_empty());
/// ```
pub fn diff_documents(a: &Document, b: &Document, opts: &DiffOptions) -> DocumentDiff {
    let mut collector = Collector::new(opts.max_findings);
    let root = NodePath::root();

    if a.sections.len() != b.sections.len() {
        collector.push(
            &root,
            FindingKind::SectionCountChanged,
            format!("구역 수: A={} B={}", a.sections.len(), b.sections.len()),
        );
    }

    // 짝이 맞는 구역만 안으로 들어간다. 남는 구역은 위의 개수 차이가 이미 말한다.
    for (i, (sa, sb)) in a.sections.iter().zip(b.sections.iter()).enumerate() {
        let path = root.child(PathStep::Section(i));
        compare_paragraph_list(
            &mut collector,
            &path,
            &sa.paragraphs,
            &sb.paragraphs,
            opts,
            0,
        );
    }

    compare_styles(
        &mut collector,
        &root,
        &a.doc_info.styles,
        &b.doc_info.styles,
    );

    let truncated = collector.truncated;
    DocumentDiff {
        identical: collector.findings.is_empty() && !truncated,
        findings: collector.findings,
        truncated,
    }
}

/// 차이를 모으는 그릇 — 상한 관리와 `truncated` 표시를 한곳에 가둔다.
struct Collector {
    findings: Vec<Finding>,
    max: Option<usize>,
    truncated: bool,
}

impl Collector {
    fn new(max: Option<usize>) -> Self {
        Self {
            findings: Vec::new(),
            max,
            truncated: false,
        }
    }

    /// 차이 하나를 기록한다. 상한을 넘으면 **버렸다는 사실을 남기고** 조용히 지나간다.
    fn push(&mut self, path: &NodePath, kind: FindingKind, detail: String) {
        if let Some(max) = self.max {
            if self.findings.len() >= max {
                self.truncated = true;
                return;
            }
        }
        self.findings.push(Finding {
            path: path.clone(),
            kind,
            detail,
        });
    }
}

/// 텍스트 비교용 정규화 — `ignore_whitespace` 가 꺼져 있으면 원문을 그대로 빌려준다.
///
/// 켜져 있으면 연속 공백을 한 칸으로 접고 양끝 공백을 없앤다. 정렬(문단 짝짓기)과
/// 텍스트 비교가 **같은 정규화**를 쓰는 것이 중요하다 — 다르면 "짝은 지었는데 다르다"와
/// "짝을 못 지었다"가 어긋난다.
fn normalize(text: &str, ignore_whitespace: bool) -> Cow<'_, str> {
    if !ignore_whitespace {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            pending_space = !out.is_empty();
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        out.push(ch);
    }
    Cow::Owned(out)
}

/// 사람이 읽을 미리보기 — 문자 단위로 자르고 잘렸으면 말줄임표를 붙인다.
fn preview(text: &str) -> String {
    let mut out: String = text.chars().take(PREVIEW_CHARS).collect();
    if text.chars().nth(PREVIEW_CHARS).is_some() {
        out.push('…');
    }
    out
}

/// 문단 목록 정렬의 한 수 — 짝지음/추가/삭제.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlignOp {
    /// A 의 `i` 번과 B 의 `j` 번이 같은 문단이다(내용은 다를 수 있다).
    Pair(usize, usize),
    /// B 의 `j` 번만 있다.
    Added(usize),
    /// A 의 `i` 번만 있다.
    Removed(usize),
}

/// 두 문단 목록을 비교한다 — 본문·표 셀 안 어디에서나 같은 규칙을 쓴다.
fn compare_paragraph_list(
    collector: &mut Collector,
    base: &NodePath,
    a: &[Paragraph],
    b: &[Paragraph],
    opts: &DiffOptions,
    depth: usize,
) {
    let keys_a: Vec<Cow<'_, str>> = a
        .iter()
        .map(|p| normalize(&p.text, opts.ignore_whitespace))
        .collect();
    let keys_b: Vec<Cow<'_, str>> = b
        .iter()
        .map(|p| normalize(&p.text, opts.ignore_whitespace))
        .collect();

    for op in align(&keys_a, &keys_b) {
        match op {
            AlignOp::Pair(i, j) => {
                let path = base.child(PathStep::Paragraph(i));
                compare_paragraph(collector, &path, &a[i], &b[j], opts, depth);
            }
            AlignOp::Added(j) => {
                // 추가된 문단은 A 에 자리가 없다 — 경로 첨자는 B 기준이다.
                let path = base.child(PathStep::Paragraph(j));
                collector.push(
                    &path,
                    FindingKind::ParagraphAdded,
                    format!("B 에만 있는 문단: {:?}", preview(&b[j].text)),
                );
            }
            AlignOp::Removed(i) => {
                let path = base.child(PathStep::Paragraph(i));
                collector.push(
                    &path,
                    FindingKind::ParagraphRemoved,
                    format!("A 에만 있는 문단: {:?}", preview(&a[i].text)),
                );
            }
        }
    }
}

/// 짝지어진 문단 하나를 비교한다 — 텍스트·문단모양·컨트롤.
fn compare_paragraph(
    collector: &mut Collector,
    path: &NodePath,
    a: &Paragraph,
    b: &Paragraph,
    opts: &DiffOptions,
    depth: usize,
) {
    if normalize(&a.text, opts.ignore_whitespace) != normalize(&b.text, opts.ignore_whitespace) {
        collector.push(
            path,
            FindingKind::TextChanged,
            format!("A={:?} B={:?}", preview(&a.text), preview(&b.text)),
        );
    }

    if a.para_shape_id != b.para_shape_id || a.style_id != b.style_id {
        collector.push(
            path,
            FindingKind::ParagraphStyleChanged,
            format!(
                "para_shape_id: A={} B={}, style_id: A={} B={}",
                a.para_shape_id, b.para_shape_id, a.style_id, b.style_id
            ),
        );
    }

    if a.controls.len() != b.controls.len() {
        collector.push(
            path,
            FindingKind::ControlCountChanged,
            format!("컨트롤 수: A={} B={}", a.controls.len(), b.controls.len()),
        );
    }

    for (k, (ca, cb)) in a.controls.iter().zip(b.controls.iter()).enumerate() {
        let ctrl_path = path.child(PathStep::Control(k));
        let (label_a, label_b) = (control_label(ca), control_label(cb));
        if label_a != label_b {
            collector.push(
                &ctrl_path,
                FindingKind::ControlKindChanged,
                format!("컨트롤 종류: A={} B={}", label_a, label_b),
            );
            continue;
        }
        if let (Control::Table(ta), Control::Table(tb)) = (ca, cb) {
            compare_table(collector, &ctrl_path, ta, tb, opts, depth);
        }
    }
}

/// 표 하나를 비교한다 — 행렬 모양, 셀 병합, 그리고 셀 안 문단.
fn compare_table(
    collector: &mut Collector,
    path: &NodePath,
    a: &Table,
    b: &Table,
    opts: &DiffOptions,
    depth: usize,
) {
    if a.row_count != b.row_count || a.col_count != b.col_count {
        collector.push(
            path,
            FindingKind::TableShapeChanged,
            format!(
                "행렬: A={}x{} B={}x{}",
                a.row_count, a.col_count, b.row_count, b.col_count
            ),
        );
    }
    if a.cells.len() != b.cells.len() {
        collector.push(
            path,
            FindingKind::TableShapeChanged,
            format!("셀 수: A={} B={}", a.cells.len(), b.cells.len()),
        );
    }

    // 깊이 상한을 넘으면 셀 안으로는 들어가지 않는다. 표 자체의 모양 차이는 위에서
    // 이미 보고했으므로 여기서 멈춰도 "차이 없음"으로 오인되지 않는다.
    if depth >= MAX_DEPTH {
        return;
    }

    for (ca, cb) in a.cells.iter().zip(b.cells.iter()) {
        let cell_path = path.child(PathStep::TableCell {
            row: ca.row,
            col: ca.col,
        });
        if ca.row != cb.row || ca.col != cb.col {
            collector.push(
                &cell_path,
                FindingKind::TableShapeChanged,
                format!(
                    "셀 주소: A=(r{},c{}) B=(r{},c{})",
                    ca.row, ca.col, cb.row, cb.col
                ),
            );
        }
        if ca.row_span != cb.row_span || ca.col_span != cb.col_span {
            collector.push(
                &cell_path,
                FindingKind::TableShapeChanged,
                format!(
                    "셀 병합: A={}x{} B={}x{}",
                    ca.row_span, ca.col_span, cb.row_span, cb.col_span
                ),
            );
        }
        compare_paragraph_list(
            collector,
            &cell_path,
            &ca.paragraphs,
            &cb.paragraphs,
            opts,
            depth + 1,
        );
    }
}

/// 문서 정보의 스타일 목록을 비교한다.
///
/// 스타일 본문(문단모양·글자모양의 모든 필드)까지 파고들지 않는다 — 여기서 보는 것은
/// **문단이 가리키는 이름표가 달라졌는가**다. 필드 단위 충실도는 왕복 게이트의 몫이다.
fn compare_styles(collector: &mut Collector, root: &NodePath, a: &[Style], b: &[Style]) {
    if a.len() != b.len() {
        collector.push(
            root,
            FindingKind::StyleCountChanged,
            format!("스타일 수: A={} B={}", a.len(), b.len()),
        );
    }

    for (i, (sa, sb)) in a.iter().zip(b.iter()).enumerate() {
        let mut parts: Vec<String> = Vec::new();
        if sa.local_name != sb.local_name {
            parts.push(format!(
                "이름: A={:?} B={:?}",
                preview(&sa.local_name),
                preview(&sb.local_name)
            ));
        }
        if sa.english_name != sb.english_name {
            parts.push(format!(
                "영문이름: A={:?} B={:?}",
                preview(&sa.english_name),
                preview(&sb.english_name)
            ));
        }
        if sa.style_type != sb.style_type {
            parts.push(format!("종류: A={} B={}", sa.style_type, sb.style_type));
        }
        if sa.para_shape_id != sb.para_shape_id {
            parts.push(format!(
                "para_shape_id: A={} B={}",
                sa.para_shape_id, sb.para_shape_id
            ));
        }
        if sa.char_shape_id != sb.char_shape_id {
            parts.push(format!(
                "char_shape_id: A={} B={}",
                sa.char_shape_id, sb.char_shape_id
            ));
        }
        if !parts.is_empty() {
            collector.push(
                &root.child(PathStep::Style(i)),
                FindingKind::StyleChanged,
                parts.join("; "),
            );
        }
    }
}

/// 컨트롤 종류 이름 — 종류가 바뀌었는지 판정하는 단일 출처.
fn control_label(c: &Control) -> &'static str {
    match c {
        Control::SectionDef(_) => "sectionDef",
        Control::ColumnDef(_) => "columnDef",
        Control::Table(_) => "table",
        Control::Shape(_) => "shape",
        Control::Picture(_) => "picture",
        Control::Header(_) => "header",
        Control::Footer(_) => "footer",
        Control::Footnote(_) => "footnote",
        Control::Endnote(_) => "endnote",
        Control::AutoNumber(_) => "autoNumber",
        Control::NewNumber(_) => "newNumber",
        Control::PageNumberPos(_) => "pageNumberPos",
        Control::Bookmark(_) => "bookmark",
        Control::IndexMark(_) => "indexMark",
        Control::PageNumCtrl(_) => "pageNumCtrl",
        Control::Hyperlink(_) => "hyperlink",
        Control::Ruby(_) => "ruby",
        Control::CharOverlap(_) => "charOverlap",
        Control::PageHide(_) => "pageHide",
        Control::HiddenComment(_) => "hiddenComment",
        Control::Equation(_) => "equation",
        Control::Field(_) => "field",
        Control::Form(_) => "form",
        Control::Unknown(_) => "unknown",
    }
}

/// 두 키 목록을 정렬해 짝지음/추가/삭제의 수순을 낸다.
///
/// 공통 앞·뒤를 먼저 깎아내는 것이 요점이다 — 실제 회귀 검증에서 두 문서는 거의
/// 같으므로, LCS 를 돌려야 하는 "가운데"는 대개 몇 줄뿐이다.
fn align(a: &[Cow<'_, str>], b: &[Cow<'_, str>]) -> Vec<AlignOp> {
    let mut ops = Vec::with_capacity(a.len().max(b.len()));

    let max_prefix = a.len().min(b.len());
    let mut prefix = 0;
    while prefix < max_prefix && a[prefix] == b[prefix] {
        prefix += 1;
    }

    let max_suffix = max_prefix - prefix;
    let mut suffix = 0;
    while suffix < max_suffix && a[a.len() - 1 - suffix] == b[b.len() - 1 - suffix] {
        suffix += 1;
    }

    for i in 0..prefix {
        ops.push(AlignOp::Pair(i, i));
    }

    let mid_a = &a[prefix..a.len() - suffix];
    let mid_b = &b[prefix..b.len() - suffix];
    ops.extend(align_middle(mid_a, mid_b, prefix, prefix));

    for k in 0..suffix {
        ops.push(AlignOp::Pair(a.len() - suffix + k, b.len() - suffix + k));
    }

    ops
}

/// 공통 앞뒤를 깎아낸 "가운데"만 정렬한다. `off_a`/`off_b` 는 원래 목록에서의 시작 첨자.
fn align_middle(
    a: &[Cow<'_, str>],
    b: &[Cow<'_, str>],
    off_a: usize,
    off_b: usize,
) -> Vec<AlignOp> {
    if a.is_empty() && b.is_empty() {
        return Vec::new();
    }
    if a.is_empty() {
        return (0..b.len()).map(|j| AlignOp::Added(off_b + j)).collect();
    }
    if b.is_empty() {
        return (0..a.len()).map(|i| AlignOp::Removed(off_a + i)).collect();
    }

    let raw = if (a.len() + 1).saturating_mul(b.len() + 1) > LCS_CELL_BUDGET {
        positional_ops(a.len(), b.len(), off_a, off_b)
    } else {
        lcs_ops(a, b, off_a, off_b)
    };
    pair_adjacent(raw)
}

/// LCS 를 감당 못 할 만큼 큰 덩어리의 대비책 — 자리끼리 맞대고 남는 꼬리를 추가·삭제로.
fn positional_ops(len_a: usize, len_b: usize, off_a: usize, off_b: usize) -> Vec<AlignOp> {
    let pairs = len_a.min(len_b);
    let mut ops: Vec<AlignOp> = (0..pairs)
        .map(|k| AlignOp::Pair(off_a + k, off_b + k))
        .collect();
    ops.extend((pairs..len_a).map(|i| AlignOp::Removed(off_a + i)));
    ops.extend((pairs..len_b).map(|j| AlignOp::Added(off_b + j)));
    ops
}

/// 최장 공통 부분수열로 짝을 짓는다.
///
/// 되짚기는 갈림길에서 **삭제를 먼저** 고른다(`>=`). 이 한 줄이 결과 순서의 결정성을
/// 보장한다 — 같은 길이의 답이 여럿일 때 언제나 같은 답을 고르기 때문이다.
fn lcs_ops(a: &[Cow<'_, str>], b: &[Cow<'_, str>], off_a: usize, off_b: usize) -> Vec<AlignOp> {
    let n = a.len();
    let m = b.len();
    let stride = m + 1;
    let mut dp = vec![0u32; (n + 1) * stride];

    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i * stride + j] = if a[i] == b[j] {
                dp[(i + 1) * stride + j + 1] + 1
            } else {
                dp[(i + 1) * stride + j].max(dp[i * stride + j + 1])
            };
        }
    }

    let mut ops = Vec::with_capacity(n.max(m));
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if a[i] == b[j] {
            ops.push(AlignOp::Pair(off_a + i, off_b + j));
            i += 1;
            j += 1;
        } else if dp[(i + 1) * stride + j] >= dp[i * stride + j + 1] {
            ops.push(AlignOp::Removed(off_a + i));
            i += 1;
        } else {
            ops.push(AlignOp::Added(off_b + j));
            j += 1;
        }
    }
    ops.extend((i..n).map(|i| AlignOp::Removed(off_a + i)));
    ops.extend((j..m).map(|j| AlignOp::Added(off_b + j)));
    ops
}

/// 맞닿은 삭제 덩어리와 추가 덩어리를 짝지어 "바뀐 문단"으로 승격시킨다.
///
/// LCS 만으로는 한 글자 고친 문단이 `Removed` + `Added` 두 건으로 나온다. 사람이
/// 보기엔 그건 **수정**이다. 앞뒤로 맞닿은 두 덩어리를 앞에서부터 짝지어 `Pair` 로
/// 바꾸고, 남는 쪽만 순수 추가·삭제로 남긴다.
fn pair_adjacent(raw: Vec<AlignOp>) -> Vec<AlignOp> {
    let mut out = Vec::with_capacity(raw.len());
    let mut idx = 0;
    while idx < raw.len() {
        if !matches!(raw[idx], AlignOp::Removed(_)) {
            out.push(raw[idx]);
            idx += 1;
            continue;
        }

        let mut removed = Vec::new();
        while let Some(AlignOp::Removed(i)) = raw.get(idx) {
            removed.push(*i);
            idx += 1;
        }
        let mut added = Vec::new();
        while let Some(AlignOp::Added(j)) = raw.get(idx) {
            added.push(*j);
            idx += 1;
        }

        let paired = removed.len().min(added.len());
        for k in 0..paired {
            out.push(AlignOp::Pair(removed[k], added[k]));
        }
        out.extend(removed.iter().skip(paired).map(|i| AlignOp::Removed(*i)));
        out.extend(added.iter().skip(paired).map(|j| AlignOp::Added(*j)));
    }
    out
}
