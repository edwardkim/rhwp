//! 관계형·고정형(쪽수): 쪽 하단 결재란은 앵커 문단 자신의 줄까지 결재란 위에 들어가야 같은 쪽에 흡수한다.
//! 주석 갱신: 2026-11-02T17:33:33+09:00 (새 파일).
//! 2026-11-02 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`),
//! 기준은 각 원본의 독립 한컴 2022 12.0.0.4547 Print(pdf/issue8467/*-2022.pdf, 각 2쪽):
//! - 36416725(HWPX): Native 100.00 / 100.00%, fresh WASM 100.00 / 100.00%. 수정 전 1쪽(측정 불가).
//! - 36412338(HWPX): Native 99.99 / 100.00%, fresh WASM 99.99 / 100.00%. 수정 전 1쪽(측정 불가).
//!
//! 같은 판정 문서는 한/글 2020·2024(MCP 변환)에서도 2쪽이다.
//!
//! 증거: mydocs/pr/assets/issue_8467/{native,wasm}/*_silhouette.tsv.
//!
//! 기대값은 독립 한컴 Print 와 저장 정보로 정한다. 두 원본은 빈 앵커 문단(저장 줄 vpos 0)에 쪽 기준·아래 정렬
//! 1×1 결재란을 둔다. 본문 끝 + 앞 줄 간격 + 앵커 줄(lh 1200 + ls 720 HU)이 결재란 띠 위끝을 넘으므로
//! 한/글 2020·2022·2024 Print 는 모두 결재란을 2쪽으로 넘긴다. 반례(같은 서식, 앵커 줄이 띠 안에 들어가
//! 한/글 1쪽)는 `issue_6535_page_anchored_block_vpos_zero` 의 36399617 이 지킨다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

/// 쪽마다 (문단, 컨트롤) 표 목록.
fn tables_per_page(core: &DocumentCore) -> Vec<Vec<(usize, usize)>> {
    fn walk(node: &RenderNode, out: &mut Vec<(usize, usize)>) {
        if let RenderNodeType::Table(t) = &node.node_type {
            if let (Some(p), Some(c)) = (t.para_index, t.control_index) {
                out.push((p, c));
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("쪽 render tree");
            let mut out = Vec::new();
            walk(&tree.root, &mut out);
            out
        })
        .collect()
}

fn verify(sample: &str, footer_para: usize) {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(core.page_count(), 2, "{sample}: 독립 한/글 Print 전체 쪽수");
    let pages = tables_per_page(&core);
    // 결재란(앵커 문단의 1×1 표)은 2쪽에만 있고, 1쪽 본문(첫 표 포함)은 1쪽에 남는다.
    let footer_pages: Vec<_> = pages
        .iter()
        .enumerate()
        .filter(|(_, tables)| tables.contains(&(footer_para, 0)))
        .map(|(page, _)| page)
        .collect();
    assert_eq!(footer_pages, vec![1], "{sample}: 결재란 쪽");
    assert!(
        pages[0].iter().any(|(para, _)| *para < footer_para),
        "{sample}: 1쪽 본문 표가 1쪽에 남아야 한다: {:?}",
        pages[0]
    );
}

/// 36416725(HWPX): 문단 14 결재란(376.9px). 본문 끝(저장) 593.6px + 앵커 줄 25.6px > 띠 위끝 613.3px.
#[test]
fn approval_footer_moves_when_anchor_line_overlaps_band() {
    verify("samples/issue8467/36416725-approval-footer-split.hwpx", 14);
}

/// 36412338(HWPX): 문단 13 결재란(357.2px), 다른 결재 서식(재난관리과).
#[test]
fn approval_footer_moves_when_anchor_line_overlaps_band_other_template() {
    verify("samples/issue8467/36412338-approval-footer-split.hwpx", 13);
}
