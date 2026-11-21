//! 관계형·고정형 — [#9213] 여러 문단 캡션의 높이는 저장 줄 사다리의 끝이다.
//! 주석 갱신: 2026-11-20 04:55 KST (새 파일)
//!
//! 입력: 공개 보도자료(고용노동부 「2022년 상반기 직종별사업체노동력조사 결과」, HWPX, 32쪽)
//! `samples/issue9213/labor-survey-2022h1-caption-ladder.hwpx`.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue9213/labor-survey-2022h1-caption-ladder-2020.pdf`(32쪽).
//! 30쪽 구역 3 `pi=158` 은 글자처럼 취급한 25×14 표와 위 캡션(문단 둘, 간격 283HU)을 가진다.
//! 캡션 저장 줄은 `vpos 0·높이 1100·줄 간격 -552` 와 `vpos 548·높이 900` 이라 사다리 끝이 1448HU 이고,
//! 호스트 줄 높이 30730 = 표 28999 + 캡션 1448 + 간격 283 으로 닫힌다. 수정 전 rhwp 는 줄 높이를
//! 더한 2000HU 로 캡션을 재어 표를 552HU(7.36px) 아래에 두었다.
//! - 고정형: 전체 쪽수 32(= PDF).
//! - 관계형: 캡션 글은 표 위끝 위에 있다. 캡션 표와 다음 표(`pi=159`)의 틈은 두 호스트 줄의
//!   저장 간격(37043 − 35743 = 1300HU)이다.
//!
//! 측정(2026-11-20, 이 파일과 같은 커밋의 코드, base = upstream/devel `9f605a0d0a`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 32쪽 / 32쪽, 30쪽 99.99(base 89.30), 전쪽 최저 93.86(4쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 32쪽 / 32쪽, 30쪽 99.93, 전쪽 최저 91.36(1쪽)
//! - 증거: `mydocs/pr/assets/issue_9213/`(native·wasm 전쪽 TSV, 30쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9213/labor-survey-2022h1-caption-ladder.hwpx";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 32;
/// 대상 쪽(0 기반, 30쪽)과 두 표의 호스트 문단.
const PAGE: u32 = 29;
const CAPTIONED_TABLE_PARA: usize = 158;
const NEXT_TABLE_PARA: usize = 159;
/// 두 호스트 줄의 저장 간격: 다음 줄 vpos 37043 − (5013 + 30730).
const STORED_GAP_HU: f64 = 1300.0;

fn find<'a>(node: &'a RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

fn runs(node: &RenderNode, out: &mut Vec<(f64, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((node.bbox.y + node.bbox.height, run.text.clone()));
    }
    for child in &node.children {
        runs(child, out);
    }
}

fn table(root: &RenderNode, para: usize) -> &RenderNode {
    find(root, &|n| {
        matches!(&n.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(para) && t.cell_context.is_none())
    })
    .unwrap_or_else(|| panic!("30쪽에 pi={para} 표가 있어야 한다"))
}

#[test]
fn multi_paragraph_caption_height_follows_stored_ladder() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    let tree = core.build_page_render_tree(PAGE).expect("30쪽");
    let captioned = table(&tree.root, CAPTIONED_TABLE_PARA);
    let next = table(&tree.root, NEXT_TABLE_PARA);

    // 캡션 두 문단은 표 위에 있다.
    let mut all = Vec::new();
    runs(&tree.root, &mut all);
    let caption_bottom = all
        .iter()
        .filter(|(_, text)| text.contains("주요 항목별 동향") || text.contains("단위: 천명"))
        .map(|(bottom, _)| *bottom)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        caption_bottom.is_finite() && caption_bottom <= captioned.bbox.y + 0.5,
        "캡션 글은 표 위끝 위에 있어야 한다: 캡션 끝 {caption_bottom} / 표 위 {}",
        captioned.bbox.y
    );

    // 표는 호스트 줄 안에서 끝나므로 다음 표와의 틈은 두 호스트 줄의 저장 간격이다.
    let gap = next.bbox.y - (captioned.bbox.y + captioned.bbox.height);
    let stored = STORED_GAP_HU * 96.0 / 7200.0;
    assert!(
        (gap - stored).abs() <= 0.5,
        "캡션 표와 다음 표의 틈은 저장 간격 {stored:.2}px 이어야 한다: {gap:.2}"
    );
}
