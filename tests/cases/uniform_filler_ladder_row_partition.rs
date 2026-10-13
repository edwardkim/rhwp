//! 통짜 합성 저장 줄 사다리(`stored_line_ladder_is_uniform_filler`) 구역에서는 문단마다
//! 한 줄이라는 **저장 분할**도 조판 근거가 아니다.
//!
//! `samples/hwpx/hwpx-02.hwpx` 는 모든 본문 문단이 같은 한 줄 `<hp:lineseg>` 를 갖는다
//! (#5854). 문단 22 `   * ‘23년에 78.1% 감소하여 ’92년 이후 최초로 상위 5개
//! 투자대상국에서 제외` 는 본문 폭보다 약 1% 길다. 한/글 2022 정본
//! (`pdf/hwpx/hwpx-02-2022.pdf` 2쪽)은 이 문단을 `…투자대상국에서 제` / `외` 두 줄로
//! 나눈다. 수정 전 rhwp 는 저장 한 줄을 믿고 글자를 눌러 한 줄에 담았다.
//!
//! 반례: 저장 사다리가 실측 조판인 일반 문서는 이 판정에 걸리지 않는다
//! (`uniform_lineseg_ladder_reflow` 의 판정 조건 — 전 문단 동일 튜플·일정 걸음).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwpx/hwpx-02.hwpx";
const PARA: usize = 22;

fn collect_lines(node: &RenderNode, out: &mut Vec<(f64, String)>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(PARA) && line.section_index == Some(0) {
            let mut text = String::new();
            for child in &node.children {
                if let RenderNodeType::TextRun(run) = &child.node_type {
                    text.push_str(&run.text);
                }
            }
            out.push((node.bbox.y, text));
            return;
        }
    }
    for child in &node.children {
        collect_lines(child, out);
    }
}

#[test]
fn overfull_single_row_of_filler_ladder_is_rebroken_like_hangul() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = DocumentCore::from_bytes(&bytes).expect("parse sample");

    let mut lines = Vec::new();
    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("render tree");
        collect_lines(&tree.root, &mut lines);
        if !lines.is_empty() {
            break;
        }
    }
    lines.sort_by(|a, b| a.0.total_cmp(&b.0));
    let texts: Vec<&str> = lines.iter().map(|(_, t)| t.trim()).collect();
    assert_eq!(
        texts.len(),
        2,
        "한/글 2022 는 문단 {PARA} 를 두 줄로 나눈다(저장은 한 줄): {texts:?}"
    );
    assert!(
        texts[0].ends_with("투자대상국에서 제"),
        "첫 줄은 `…투자대상국에서 제` 로 끝나야 한다: {texts:?}"
    );
    assert_eq!(texts[1], "외", "둘째 줄은 `외` 한 글자다: {texts:?}");
}
