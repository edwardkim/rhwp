//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8461의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:37:17+09:00 (`f8c61ad2ffe5e8f9c1259f1cb4ec415d67f99f50`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8458/156759019-text-line-tac-logo.hwp`; 독립 Print `pdf/issue8458/156759019-text-line-tac-logo-2022.pdf`.
//! 전체 12쪽, 전쪽 최저 Native 93.11% / fresh WASM 92.87%.
//! 주장 2쪽 raw: Native 99.79% / fresh WASM 99.89%.
//! 주장 12쪽 raw: Native 93.11% / fresh WASM 92.87%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8461_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-02T12:27:55+09:00 (새 파일).
//! 2026-11-02 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`),
//! 기준은 독립 한컴 2022 12.0.0.4547 Print(pdf/issue8458/156759019-text-line-tac-logo-2022.pdf, 12쪽):
//! - 156759019(HWP, 12쪽): 로고 2쪽 Native 97.37% / fresh WASM 97.53%. 전 12쪽 최저 Native 93.42% /
//!   WASM 93.26%(둘 다 12쪽). 수정 전에는 로고만 3쪽으로 넘어가 13쪽(측정 불가).
//!
//! 증거: mydocs/pr/assets/issue_8458/{native,wasm}/156759019_silhouette.tsv.
//!
//! 기대값은 저장 LINE_SEG 와 독립 Print 로 정한다: 문단 29 줄1(vpos 65081, lh 3984) 끝 69065 HU 가
//! 쪽 본문 높이(70020 HU) 안이라 한컴은 공백 글줄과 그 안의 로고를 2쪽 하단에 둔다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect(child, out);
    }
}

fn bottom(b: &BoundingBox) -> f64 {
    b.y + b.height
}

/// 156759019(HWP, 12쪽): 문단 29 줄0 「담당 부서」 3×6 TAC 표, 줄1 공백 + TAC 로고 그림(높이 3984).
#[test]
fn tac_picture_in_placed_text_line_stays_on_the_line_page() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue8458/156759019-text-line-tac-logo.hwp"),
    )
    .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(core.page_count(), 12, "독립 한/글 2022 Print 전체 쪽수");

    let mut host_pages = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("쪽 render tree");
        let mut nodes = Vec::new();
        collect(&tree.root, &mut nodes);
        if nodes.iter().any(|n| {
            matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(29) && t.control_index == Some(0))
                || matches!(&n.node_type, RenderNodeType::TextLine(l) if l.para_index == Some(29))
        }) {
            host_pages.push(page);
        }
    }
    // 표와 줄1 글줄은 한 쪽(2쪽)에만 있다 — 로고만 남은 쪽이 없다.
    assert_eq!(host_pages, vec![1], "문단 29 의 표·글줄이 놓인 쪽");

    let tree = core.build_page_render_tree(1).expect("2쪽 render tree");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let table = nodes
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::Table(t) if t.para_index == Some(29) && t.control_index == Some(0)))
        .expect("담당 부서 표")
        .bbox;
    let line = nodes
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::TextLine(l) if l.para_index == Some(29) && l.line_index == Some(1)))
        .expect("문단 29 줄1 글줄")
        .bbox;
    let body = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .expect("본문 영역")
        .bbox;
    assert!(
        line.y >= bottom(&table) - 0.5,
        "줄1 {line:?} 이 표 {table:?} 위로 겹쳤다"
    );
    // 로고 그림은 줄1 글줄 상자 안에 있다(줄 높이 = 그림 높이 3984 HU).
    let logos: Vec<_> = nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Image(_)))
        .filter(|n| n.bbox.y >= line.y - 0.5 && bottom(&n.bbox) <= bottom(&line) + 0.5)
        .map(|n| n.bbox)
        .collect();
    assert!(!logos.is_empty(), "줄1 안의 로고 그림: {line:?}");
    let picture_h = 3984.0 * 96.0 / 7200.0;
    assert!(
        logos.iter().any(|b| (b.height - picture_h).abs() <= 1.0),
        "로고 높이 {picture_h:.2} 인 그림이 줄1 안에 없다: {logos:?}"
    );
    assert!(
        bottom(&line) <= bottom(&body) + 0.5,
        "줄1 이 본문 아래끝을 넘었다"
    );
}
