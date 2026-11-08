//! [#7665] 미주 렌더의 「새 문항」 판정은 미주 소유 경계로 정한다 — 「문」 접두사가 아니다.
//!
//! 미주 첫 문단 앞에는 렌더가 번호 장식(`문1)` 등)을 붙인다(`prepend_endnote_marker_text`,
//! `ep_idx == 0`). 그래서 기존 갈래들(`height_cursor.rs` 제목 간격 압축·하단 되감기,
//! `layout.rs` 제목 간격 보존·다음 문항 판정, 조판 `paragraph.rs` 직전 제목 판정)은 「문」
//! 으로 시작하는 문단을 새 문항 제목으로 읽었다. 같은 미주 안 본문이 `문서…` 처럼 「문」
//! 으로 시작하면 제목으로 오인되어 위치가 바뀐다.
//!
//! 입력: 실제 저장본의 미주 본문 문단 맨 앞에 같은 폭의 두 글자를 넣은 두 문서 —
//! `문서 `(반례) / `가나 `(대조). 넣은 글자 말고는 같은 입력이므로 그 줄과 같은 쪽 뒤 줄의
//! 위치가 같아야 한다. 수정 전 관측(#7665 본문, PR #7591 head 41c64bd55):
//!
//! | 문서 | (구역,문단,컨트롤) · 미주 안 문단 | `문서 ` y | `가나 ` y |
//! | --- | --- | --- | --- |
//! | 2023 | (0,348,0) · 6 | 138.71 | 132.68 |
//! | 미주사이20 | (0,134,0) · 15 | 152.36 | 178.36 |
//! | 미주사이20 | (0,254,0) · 8 | 114.76 | 108.73 |
//! | 구분선위0미주사이20구분선아래2 | (0,229,0) · 3 | 182.01 | 175.99 |

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const S2023: &[u8] = include_bytes!("../../samples/3-09월_교육_통합_2023.hwp");
const S2024_GAP20: &[u8] = include_bytes!("../../samples/3-09월_교육_통합_2024-미주사이20.hwp");
const S2024_SEP: &[u8] =
    include_bytes!("../../samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp");

type Line = (u32, f64, String, Option<usize>);

fn line_text(node: &RenderNode) -> String {
    fn walk(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut text = String::new();
    walk(node, &mut text);
    text
}

/// 쪽마다 `(쪽, 상단 y, 텍스트, 문단 번호)` 로 줄을 모은다.
fn lines(core: &DocumentCore) -> Vec<Line> {
    fn walk(node: &RenderNode, page: u32, out: &mut Vec<Line>) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            out.push((page, node.bbox.y, line_text(node), line.para_index));
            return;
        }
        for child in &node.children {
            walk(child, page, out);
        }
    }
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        walk(&tree.root, page, &mut out);
    }
    out
}

/// 대상 문단의 앞 글. 표본이 바뀌어 전제(「문」 으로 시작하지 않는 같은 미주 본문)가
/// 깨지면 바로 실패한다.
fn target_head(core: &DocumentCore, (si, pi, ci, k): (usize, usize, usize, usize)) -> String {
    let Control::Endnote(en) = &core.document().sections[si].paragraphs[pi].controls[ci] else {
        panic!("대상 컨트롤이 미주가 아니다 — 표본이 바뀌었는지 확인")
    };
    assert!(k > 0, "대상은 미주의 첫 문단이 아니어야 한다");
    let head: String = en.paragraphs[k].text.chars().take(6).collect();
    assert!(
        head.chars().count() == 6 && !head.starts_with('문'),
        "대상은 글로 시작하는 미주 본문이어야 한다: {head:?}"
    );
    head
}

fn with_prefix(
    sample: &[u8],
    (si, pi, ci, k): (usize, usize, usize, usize),
    prefix: &str,
) -> DocumentCore {
    let mut core = DocumentCore::from_bytes(sample).unwrap();
    core.insert_text_in_footnote_native(si, pi, ci, k, 0, prefix)
        .unwrap();
    // 저장본 경로로 다시 열어 파서 → 조판 → 렌더 전체를 태운다.
    DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap()
}

fn find_line<'a>(lines: &'a [Line], starts: &str) -> (usize, &'a Line) {
    let hits: Vec<_> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.2.trim_start().starts_with(starts))
        .collect();
    assert_eq!(hits.len(), 1, "줄 `{starts}` 이 하나여야 한다: {hits:?}");
    hits[0]
}

fn assert_same_note_body_keeps_position(sample: &[u8], target: (usize, usize, usize, usize)) {
    let head = target_head(&DocumentCore::from_bytes(sample).unwrap(), target);
    let probe_lines = lines(&with_prefix(sample, target, "문서 "));
    let control_lines = lines(&with_prefix(sample, target, "가나 "));
    let (probe_idx, probe_line) = find_line(&probe_lines, &format!("문서 {head}"));
    let (control_idx, control_line) = find_line(&control_lines, &format!("가나 {head}"));
    assert_eq!(probe_idx, control_idx, "넣은 글자 앞 줄 구성이 같아야 한다");
    assert_eq!(
        (probe_line.0, (probe_line.1 * 100.0).round()),
        (control_line.0, (control_line.1 * 100.0).round()),
        "같은 미주 본문은 「문」 으로 시작해도 새 문항 제목 갈래를 타지 않는다 \
         (쪽, y×100: 문서 vs 대조) {target:?}"
    );
    // 같은 쪽 뒤 줄 — 제목 갈래는 뒤 줄의 저장 사다리 기준도 함께 옮긴다.
    let page = probe_line.0;
    let after = |lines: &[Line], from: usize| -> Vec<(f64, Option<usize>)> {
        lines[from..]
            .iter()
            .take_while(|line| line.0 == page)
            .map(|line| (line.1, line.3))
            .collect()
    };
    let probe_after = after(&probe_lines, probe_idx);
    let control_after = after(&control_lines, control_idx);
    assert_eq!(
        probe_after.len(),
        control_after.len(),
        "같은 쪽 뒤 줄 수 {target:?}"
    );
    for ((probe_y, para), (control_y, _)) in probe_after.iter().zip(&control_after) {
        assert!(
            (probe_y - control_y).abs() < 0.01,
            "같은 미주 본문 뒤 줄도 그대로다 {target:?}: 문단 {para:?} 문서 {probe_y:.2} vs \
             대조 {control_y:.2}"
        );
    }
}

#[test]
fn same_note_mun_body_2023_note_348() {
    assert_same_note_body_keeps_position(S2023, (0, 348, 0, 6));
}

#[test]
fn same_note_mun_body_gap20_note_134() {
    assert_same_note_body_keeps_position(S2024_GAP20, (0, 134, 0, 15));
}

#[test]
fn same_note_mun_body_gap20_note_254() {
    assert_same_note_body_keeps_position(S2024_GAP20, (0, 254, 0, 8));
}

#[test]
fn same_note_mun_body_separator_note_229() {
    assert_same_note_body_keeps_position(S2024_SEP, (0, 229, 0, 3));
}
