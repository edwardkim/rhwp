//! 이어받는 rowspan 시작 행은 앞 조각이 그린 몫을 다시 세지 않는다(#7685).
//!
//! # 규칙과 독립 기대값
//!
//! 앞 조각이 rowspan 에 걸친 행을 쪽 남은 높이(물리 띠)까지 그리고 끊으면, 다음 조각의
//! 시작 행은 «시작 컷 뒤 남은 내용» 과 «앞 조각이 넘긴 물리 꼬리(선언 − 그린 띠)» 중 큰
//! 쪽만 차지한다. 행 전체 선언 높이로 세면 앞 쪽에서 그린 띠가 한 번 더 들어간다.
//!
//! - `samples/task2287/1342000_edu_curriculum_map.hwp` 29→30쪽: 11행(선언 60.4px)이 29쪽에
//!   51.9px 띠로 그려지고, 30쪽 이월 꼬리는 8.5px 다. 종전 rhwp 는 30쪽에서 11행을 60.4px
//!   로 다시 세어 16행 분할 예산이 51.9px 줄었고, 30쪽 끝 세 줄(`■ 경기도교육청 성교육 진흥
//!   조례` 문단)이 31쪽으로 밀렸다.
//! - 정본 `pdf/task2287/1342000_edu_curriculum_map-hwp-2020.pdf`(한/글 2020): 30쪽 마지막
//!   두 글줄이 `■ 경기도교육청 성교육 진흥 조례` 와 그 `제3조` 첫 줄이고, 31쪽 첫 글줄이
//!   `② 교육감은 학교교육과정 운영에 성교육 시간을 …` 이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn load(path: &str) -> DocumentCore {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = std::fs::read(&full).unwrap_or_else(|e| panic!("재현체 {}: {e}", full.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 쪽의 글줄을 위에서 아래로 (y, 글) 로 모은다.
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
            if !s.trim().is_empty() {
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

fn compact(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn carried_rowspan_tail_is_not_charged_twice() {
    let core = load("samples/task2287/1342000_edu_curriculum_map.hwp");
    let p30: Vec<String> = page_lines(&core, 29)
        .into_iter()
        .map(|(_, s)| compact(&s))
        .filter(|s| !s.starts_with('-'))
        .collect();
    let p31: Vec<String> = page_lines(&core, 30)
        .into_iter()
        .map(|(_, s)| compact(&s))
        .filter(|s| !s.starts_with('-'))
        .collect();
    assert!(
        p30.iter()
            .any(|s| s.contains("■경기도교육청성교육진흥조례")),
        "30쪽이 `■ 경기도교육청 성교육 진흥 조례` 문단을 담아야 한다(정본 30쪽 끝): 끝 줄 {:?}",
        &p30[p30.len().saturating_sub(3)..]
    );
    assert!(
        p31.first()
            .is_some_and(|s| s.starts_with("②교육감은학교교육과정운영에성교육시간을")),
        "31쪽 첫 글줄은 정본처럼 `② 교육감은 …` 이어야 한다: {:?}",
        p31.first()
    );
}
