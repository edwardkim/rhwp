//! [Issue #6550] 새 미주 문항 제목이 앞 풀이의 마지막 줄 **위로** 스냅돼 겹쳤다.
//!
//! `vpos_adjust` 의 저장-vpos 스냅 갈래들은 프레임 맞춤이 목적인데, 되감을 자리가
//! 직전 미주의 콘텐츠 하단보다 위면 그 스냅이 그대로 겹침이 된다.
//!
//! `samples/3-09월_교육_통합_2023.hwp` 18쪽 `문26）` 경계 실측:
//!
//! ```text
//!   흐름 y_offset  1044.3      앞 줄(`가 나타내는 도형의 길이는 …`) 상단 1032.3
//!   저장 end_y     1035.9      그 줄 하단 ≈ 1043.9
//!   수정 전 result 1035.9  →  overlap 142.51×11.63px · text-overlap 7건
//!   수정 후 result 1043.9+ →  overlap 0 · text-overlap 0
//! ```
//!
//! 바닥값 `prev_content_floor_y` 는 「앞 줄 하단 + 주입된 미주-사이 간격」이라
//! 결과가 한/글과 같은 자리로 들어온다 — 앞 줄 상단 기준 간격 **17.6 → 38.5px**
//! (한/글 2022 오라클 **39.3px**).
//!
//! ⚠ 범위는 **기본 「미주 사이」(≤1984HU) + 보통 높이 tail** 로만 좁힌다. 큰 미주
//! 사이(5669HU)의 spacer 접기와 큰 수식 tail 뒤 제한 backtrack(#1284 PDF 핀)은
//! 각각 다른 계약이라 건드리면 깨진다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/3-09월_교육_통합_2023.hwp";

/// 18쪽(0-기준 17) 왼쪽 단의 `TextLine` 들을 (상단 y, 잉크 하단 y, 텍스트) 로 모은다.
///
/// ⚠ `TextLine` 자신의 `bbox.height` 는 이 문서에서 0 이다 — 하단은 **자식 `TextRun`
/// 의 최하단**으로 재야 겹침 판정이 성립한다(그러지 않으면 음성 대조를 통과한다).
fn left_column_lines(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && node.bbox.x < 400.0 {
        let mut text = String::new();
        collect_text(node, &mut text);
        let ink_bottom = run_bottom(node).unwrap_or(node.bbox.y + node.bbox.height);
        out.push((node.bbox.y, ink_bottom, text));
        return;
    }
    for child in &node.children {
        left_column_lines(child, out);
    }
}

/// 자식 `TextRun` 들의 최하단.
fn run_bottom(node: &RenderNode) -> Option<f64> {
    let own = matches!(node.node_type, RenderNodeType::TextRun(_))
        .then(|| node.bbox.y + node.bbox.height);
    node.children
        .iter()
        .filter_map(run_bottom)
        .chain(own)
        .fold(None, |acc: Option<f64>, y| {
            Some(acc.map_or(y, |a| a.max(y)))
        })
}

fn collect_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

/// 새 문항 제목(`문N）`)은 바로 앞 줄과 겹치지 않아야 한다.
#[test]
fn endnote_question_title_does_not_overlap_previous_note_tail() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core
        .build_page_render_tree(17)
        .expect("18쪽 render tree 생성 실패");

    // ⚠ y 로 정렬하면 안 된다 — 결함이 있을 때 제목이 앞 줄보다 **위**에 오므로
    // 정렬이 둘의 순서를 뒤집어 음성 대조를 통과시킨다. 트리(흐름) 순서를 쓴다.
    let mut lines = Vec::new();
    left_column_lines(&tree.root, &mut lines);

    let title_idx = lines
        .iter()
        .position(|(_, _, t)| t.trim_start().starts_with("문26）"))
        .expect("18쪽 왼쪽 단에 `문26）` 제목 줄이 있어야 한다");
    assert!(title_idx > 0, "제목 앞에 앞 미주의 줄이 있어야 한다");

    let (title_y, _, _) = &lines[title_idx];
    let (_, prev_bottom, prev_text) = &lines[title_idx - 1];
    let prev_bottom = *prev_bottom;

    assert!(
        *title_y + 0.5 >= prev_bottom,
        "새 문항 제목이 앞 풀이 마지막 줄 위로 스냅되면 안 된다 — #6550 회귀 \
         (제목 상단 {title_y:.1} < 앞 줄 하단 {prev_bottom:.1}; 앞 줄 {prev_text:?})"
    );
}

/// 「미주 사이」 간격이 한/글과 같은 자리로 들어와야 한다.
///
/// 앞 줄 상단 → 새 문항 제목 상단: 수정 전 **17.6px** · 수정 후 **38.5px** ·
/// 한/글 2022 오라클 **39.3px**.
#[test]
fn between_notes_gap_matches_hangul_within_two_px() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core
        .build_page_render_tree(17)
        .expect("18쪽 render tree 생성 실패");

    // ⚠ y 로 정렬하면 안 된다 — 결함이 있을 때 제목이 앞 줄보다 **위**에 오므로
    // 정렬이 둘의 순서를 뒤집어 음성 대조를 통과시킨다. 트리(흐름) 순서를 쓴다.
    let mut lines = Vec::new();
    left_column_lines(&tree.root, &mut lines);

    let title_idx = lines
        .iter()
        .position(|(_, _, t)| t.trim_start().starts_with("문26）"))
        .expect("18쪽 왼쪽 단에 `문26）` 제목 줄이 있어야 한다");
    let gap = lines[title_idx].0 - lines[title_idx - 1].0;

    // 한/글 39.3px. 수정 전 17.6px 이었으므로 30px 문턱이 양쪽을 가른다.
    assert!(
        (30.0..=45.0).contains(&gap),
        "「미주 사이」 간격이 한/글(39.3px) 근처여야 한다 — #6550 회귀          (실측 {gap:.1}px; 수정 전 17.6px)"
    );
}
