//! [#6574] 새 미주의 시작은 미주 소유 경계로 정한다 — 글자 모양(「문」 접두사)이 아니다.
//!
//! 렌더는 새 미주의 첫 문단을 직전 미주 마지막 줄 줄 상자 하단 + 미주 사이 간격에 둔다.
//! 이 갈래를 `문` 으로 시작하는 문단에 걸면 같은 미주 안의 `문서…` 본문도 새 미주로 오인해
//! 간격 자리로 옮긴다. 조판의 단 수용 판정은 이미 소유 경계(`ep_idx == 0`)를 쓰므로 렌더도
//! 같은 기준이어야 측정과 배치가 갈리지 않는다.
//!
//! 입력: `samples/3-09월_교육_통합_2024-미주사이20.hwp`(실제 저장본, 미주 사이 20mm) 의 구역 0
//! 문단 122 미주의 18번째 문단(글로 시작하는 본문). 그 앞에 같은 폭의 두 글자를 넣은 두 문서를
//! 만든다 — `문서 `(반례) 와 `가나 `(대조). 기대: 같은 미주의 본문이므로 그 줄과 같은 쪽 뒤
//! 줄의 위치가 두 문서에서 같다(넣은 글자 말고는 같은 입력이다). 수정 전에는 반례 줄이
//! 직전 줄 하단 + 미주 사이(75.6px) 로 옮겨져 69.6px 내려갔다(164.04 → 233.60px).
//! 정상 대조: 같은 문서에서 다음 미주의 첫 문단은 직전 미주 마지막 줄 줄 상자 하단 + 미주
//! 사이 간격에 놓인다(소유 경계 기준이 새 미주를 놓치지 않는다).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &[u8] = include_bytes!("../../samples/3-09월_교육_통합_2024-미주사이20.hwp");

/// (구역, 본문 문단, 컨트롤, 미주 안 문단).
const TARGET: (usize, usize, usize, usize) = (0, 122, 0, 17);

/// 대상 문단의 앞 글과 대상 미주의 문단 수. 표본이 바뀌어 전제가 깨지면 바로 실패한다.
fn target_endnote(core: &DocumentCore) -> (String, usize) {
    let (si, pi, ci, k) = TARGET;
    let Control::Endnote(en) = &core.document().sections[si].paragraphs[pi].controls[ci] else {
        panic!("대상 컨트롤이 미주가 아니다 — 표본이 바뀌었는지 확인")
    };
    let para = &en.paragraphs[k];
    let head: String = para.text.chars().take(6).collect();
    assert!(
        head.chars().count() == 6 && !head.starts_with('문') && en.paragraphs.len() > k + 1,
        "대상은 글로 시작하는 미주 본문이고 뒤에 같은 미주 문단이 있어야 한다: {head:?}"
    );
    (head, en.paragraphs.len())
}

type Line = (u32, f64, f64, String, Option<usize>);

fn line_text(node: &RenderNode) -> String {
    let mut text = String::new();
    fn walk(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    walk(node, &mut text);
    text
}

/// 쪽마다 `(쪽, 상단 y, 줄 높이, 텍스트, 문단 번호)` 로 줄을 모은다.
fn lines(core: &DocumentCore) -> Vec<Line> {
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        fn walk(node: &RenderNode, page: u32, out: &mut Vec<Line>) {
            if let RenderNodeType::TextLine(line) = &node.node_type {
                out.push((
                    page,
                    node.bbox.y,
                    line.line_height,
                    line_text(node),
                    line.para_index,
                ));
                return;
            }
            for child in &node.children {
                walk(child, page, out);
            }
        }
        walk(&tree.root, page, &mut out);
    }
    out
}

fn with_prefix(prefix: &str) -> DocumentCore {
    let (si, pi, ci, k) = TARGET;
    let mut core = DocumentCore::from_bytes(SAMPLE).unwrap();
    core.insert_text_in_footnote_native(si, pi, ci, k, 0, prefix)
        .unwrap();
    // 저장본 경로로 다시 열어 파서 → 조판 → 렌더 전체를 태운다.
    DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap()
}

fn find_line<'a>(lines: &'a [Line], starts: &str) -> (usize, &'a Line) {
    let hits: Vec<_> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.3.trim_start().starts_with(starts))
        .collect();
    assert_eq!(hits.len(), 1, "줄 `{starts}` 이 하나여야 한다: {hits:?}");
    hits[0]
}

#[test]
fn same_note_paragraph_starting_with_mun_keeps_its_position() {
    let original = DocumentCore::from_bytes(SAMPLE).unwrap();
    let (body, _) = target_endnote(&original);
    let probe = with_prefix("문서 ");
    let control = with_prefix("가나 ");

    let probe_lines = lines(&probe);
    let control_lines = lines(&control);
    let (probe_idx, probe_line) = find_line(&probe_lines, &format!("문서 {body}"));
    let (control_idx, control_line) = find_line(&control_lines, &format!("가나 {body}"));
    assert_eq!(probe_idx, control_idx, "넣은 글자 앞 줄 구성이 같아야 한다");
    assert_eq!(
        (probe_line.0, (probe_line.1 * 100.0).round()),
        (control_line.0, (control_line.1 * 100.0).round()),
        "같은 미주 본문은 「문」 으로 시작해도 새 미주 간격 자리로 옮기지 않는다 \
         (쪽, y×100: 문서 vs 대조)"
    );

    // 그 줄과 같은 쪽의 뒤 줄 전부 — 위치를 옮기는 갈래는 뒤 줄의 저장 사다리 기준도 함께 옮긴다.
    let page = probe_line.0;
    let after = |lines: &[Line], from: usize| -> Vec<(f64, Option<usize>)> {
        lines[from..]
            .iter()
            .take_while(|line| line.0 == page)
            .map(|line| (line.1, line.4))
            .collect()
    };
    let probe_after = after(&probe_lines, probe_idx);
    let control_after = after(&control_lines, control_idx);
    assert_eq!(probe_after.len(), control_after.len(), "같은 쪽 뒤 줄 수");
    for ((probe_y, para), (control_y, _)) in probe_after.iter().zip(&control_after) {
        assert!(
            (probe_y - control_y).abs() < 0.01,
            "같은 미주 본문 뒤 줄도 그대로다: 문단 {para:?} 문서 {probe_y:.2} vs 대조 \
             {control_y:.2}"
        );
    }
}

#[test]
fn next_note_first_paragraph_starts_after_between_notes_gap() {
    let original = DocumentCore::from_bytes(SAMPLE).unwrap();
    let (body, note_paras) = target_endnote(&original);
    let probe = with_prefix("문서 ");
    let si = TARGET.0;
    // 한컴 UI 「미주 사이」(구분선 아래 `note_spacing` 과 다른 값이다).
    let gap_hu = probe.document().sections[si]
        .section_def
        .endnote_shape
        .between_notes_margin_hu() as f64;
    let gap_px = gap_hu * 96.0 / 7200.0;

    let probe_lines = lines(&probe);
    let (_, probe_line) = find_line(&probe_lines, &format!("문서 {body}"));
    // 미주 문단 번호는 미주 흐름 안에서 이어진다: 대상 문단 + (남은 문단 수) = 다음 미주 첫 문단.
    let next_first = probe_line.4.expect("미주 줄의 문단 번호") + note_paras - TARGET.3;
    let idx = probe_lines
        .iter()
        .position(|line| line.4 == Some(next_first))
        .expect("다음 미주 첫 문단 줄");
    let head = &probe_lines[idx];
    let prev = &probe_lines[idx - 1];
    assert_eq!(
        prev.0, head.0,
        "직전 미주 마지막 줄과 같은 쪽이어야 정상 대조가 성립한다"
    );
    assert!(
        prev.4 < head.4,
        "직전 줄은 앞 미주의 줄이어야 한다: {prev:?}"
    );
    let prev_box_bottom = prev.1 + prev.2;
    assert!(
        (head.1 - (prev_box_bottom + gap_px)).abs() < 0.01,
        "새 미주 첫 문단 = 직전 줄 상자 하단 {prev_box_bottom:.2} + 미주 사이 {gap_px:.2}, \
         실제 {:.2} ({head:?})",
        head.1
    );
}
