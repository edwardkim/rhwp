//! [#7702] 합성 줄과 한/글 저장 줄이 섞인 쪽의 흐름 — 트림 소실·그리기 기준·쪽 끝 첫 줄.
//!
//! 입력 `samples/HWP5-nopassword-123456.hwpx` 는 한/글 2024 저장본이다. 일부 문단은 저장 줄이
//! 없어 rhwp 가 로드 때 줄을 합성하고(`TAG_IMPLEMENTATION_PROPERTY`), 그 뒤 저장 문단의 vpos 를
//! rhwp 누적 좌표로 옮긴다. 기준은 같은 파일의 한/글 2024 PDF
//! `pdf/HWP5-nopassword-123456-hwpx-2024.pdf`(24쪽)다.
//!
//! 세 결함이 겹쳐 있었다.
//!
//! 1. 조판: 합성 문단이 든 쪽에서 저장 문단의 끝 간격 트림이 «뒤 저장 문단 스냅이 되돌린다»
//!    는 전제로 흐름에서 빠졌다. 좌표계가 달라 스냅이 서지 않아 14쪽에서 트림 8회 ≈125px 가
//!    사라졌고, 그리기는 본문 아래 100px 를 넘쳤다.
//! 2. 그리기: 이어지는 부분 문단 뒤 기준 리셋 다음, 저장 문단이 기준을 세우지 않아 그 다음
//!    문단이 트림을 전제한 역산으로 15.6px(끝 줄간격 + 아래 간격) 아래에 놓였다(15쪽).
//! 3. 쪽 끝: 첫 줄만 이 쪽이고 둘째 줄부터 새 쪽(vpos 0)인 저장 문단의 «첫 줄이 본문 안인가»
//!    를 누적 좌표(128472)로 재 첫 줄을 다음 쪽으로 넘겼다. 한/글 원본 값은 64520(873.6px ≤
//!    본문 876.9px)이고, 한/글은 그 줄을 15쪽 끝에 둔다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/HWP5-nopassword-123456.hwpx");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 쪽 글줄을 위에서 아래로 (y, 공백 뺀 글) 로 모은다. 쪽 번호 줄은 뺀다.
fn page_lines(core: &DocumentCore, page: u32) -> Vec<(f64, String)> {
    fn text(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            text(child, out);
        }
    }
    fn walk(node: &RenderNode, out: &mut Vec<(f64, String)>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let mut s = String::new();
            text(node, &mut s);
            let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
            if !s.is_empty() && !s.starts_with('-') {
                out.push((node.bbox.y, s));
            }
            return;
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let tree = core.build_page_render_tree(page).expect("렌더 트리");
    let mut out = Vec::new();
    walk(&tree.root, &mut out);
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}

#[test]
fn page_count_matches_hangul() {
    assert_eq!(core().page_count(), 24, "정본 24쪽");
}

#[test]
fn page14_and_page15_end_like_hangul() {
    let core = core();
    let p14 = page_lines(&core, 13);
    let p15 = page_lines(&core, 14);
    assert!(
        p14.last()
            .is_some_and(|(_, s)| s.starts_with("사용자가지정한경로의그림이나색깔을")),
        "14쪽 끝은 정본처럼 `사용자가 지정한 경로의 그림이나 색깔을 …` 줄이다: {:?}",
        p14.last()
    );
    assert!(
        p15.last()
            .is_some_and(|(_, s)| s.starts_with("②끊어찍기:많은양의문서를출력할때")),
        "15쪽 끝은 정본처럼 `② 끊어 찍기: …` 첫 줄이다: {:?}",
        p15.last()
    );
}

/// 15쪽 `•원고지 보기 …`(저장 1줄) 와 다음 저장 문단 `일반 편집 화면에서 …` 의 간격은 저장
/// vpos 차 2168 HU = 28.9px 다(정본 기준선 193.9 → 222.9, 29.0px).
#[test]
fn stored_paragraph_after_continued_partial_keeps_ladder_gap() {
    let core = core();
    let p15 = page_lines(&core, 14);
    let y = |head: &str| {
        p15.iter()
            .find(|(_, s)| s.starts_with(head))
            .map(|(y, _)| *y)
            .unwrap_or_else(|| panic!("15쪽 `{head}` 줄: {p15:?}"))
    };
    let gap = y("일반편집화면에서작성한문서를") - y("•원고지보기기능으로작문이쉬워집니다");
    assert!(
        (gap - 28.9).abs() < 0.3,
        "저장 사다리 간격 28.9px(정본 29.0) 이어야 한다: {gap:.2}"
    );
}
