//! [Issue #5946] 행정업무운영편람 2025 141쪽 쪽나눔.
//!
//! `pageBreak=CELL` · `repeatHeader=1` 인 설계 기준 표에서 '3. 쪽' 칸의
//! 중첩 예시 서식(접수번호)이 다음 쪽 '4. 항목란' 위로 포개진다. 쪽 분할이
//! 행 높이 override 로만 이어질 때 조각 셀을 clip 해 다음 행을 침범하지 않게 한다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/2025 행정업무운영 편람(최종).hwpx";
/// 뷰어 141쪽 = 0-based 140. #5947 은 137쪽을 `-p 136` 으로 열었다.
const PAGE_INDEX: u32 = 140;

#[test]
fn issue_5946_p141_hangmok_is_not_covered_by_form_header() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse handbook");
    let page_count = core.page_count();
    // 뷰어 141쪽 근처. 조판이 밀리고 렌더 트리는 '4. ' / '항목란' 을 나눠 그린다.
    let search_lo = PAGE_INDEX.saturating_sub(4);
    let search_hi = (PAGE_INDEX + 20).min(page_count.saturating_sub(1));
    let mut hangmok = Vec::new();
    let mut form = Vec::new();
    let mut found_page = PAGE_INDEX;
    for page_idx in search_lo..=search_hi {
        hangmok.clear();
        form.clear();
        let page = core
            .build_page_render_tree(page_idx)
            .unwrap_or_else(|e| panic!("page {} render tree: {e}", page_idx + 1));
        collect(&page.root, None, &mut hangmok, &mut form);
        if !hangmok.is_empty() {
            found_page = page_idx;
            break;
        }
    }

    assert!(
        !hangmok.is_empty(),
        "{}쪽 근처에 '항목란' 이 없다 — 검색 {}..{}: 표본 쪽 번호가 바뀌었는지 확인하라",
        PAGE_INDEX + 1,
        search_lo + 1,
        search_hi + 1
    );
    for (hx, hy, hw, hh, _) in &hangmok {
        for (fx, fy, fw, fh, ftext) in &form {
            let overlap = hx < &(fx + fw) && fx < &(hx + hw) && hy < &(fy + fh) && fy < &(hy + hh);
            assert!(
                !overlap,
                "쪽 {} 예시 서식 '{ftext}' 가 4. 항목란 과 겹친다 ({fx:.1},{fy:.1})×({fw:.1}x{fh:.1}) vs 항목란 ({hx:.1},{hy:.1})",
                found_page + 1
            );
        }
    }
}

fn collect(
    node: &RenderNode,
    cell_clip: Option<bool>,
    hangmok: &mut Vec<(f64, f64, f64, f64, String)>,
    form: &mut Vec<(f64, f64, f64, f64, String)>,
) {
    let cell_clip = match &node.node_type {
        RenderNodeType::TableCell(cell) => Some(cell.clip),
        _ => cell_clip,
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let t = run.text.as_str();
        let bb = (
            node.bbox.x,
            node.bbox.y,
            node.bbox.width,
            node.bbox.height,
            t.to_string(),
        );
        // 레이아웃이 '4. 항목란' 을 줄/런 단위로 나눠 '항목란' 만 남긴다.
        if t.contains("항목란") {
            hangmok.push(bb.clone());
        }
        if (t.contains("접수번호") || t.contains("3쪽 중 1쪽")) && cell_clip != Some(true) {
            form.push(bb);
        }
    }
    for child in &node.children {
        collect(child, cell_clip, hangmok, form);
    }
}
