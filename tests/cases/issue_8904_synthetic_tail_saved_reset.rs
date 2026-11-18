//! 관계형·고정형 — [#8904] 저장 줄 없는 문단의 합성 vpos 를 저장 되감김 쪽 경계로 읽지 않는다.
//! 주석 갱신: 2026-11-10 18:45 KST (새 파일)
//!
//! 입력: 공개 결재문서 36376635(한컴오피스 2020 저장) 원본. 2쪽 끝 `pi=34`「- 다만, ***」은
//! 익명화로 저장 줄(`linesegarray`)이 없어 로드 때 3줄을 합성하고, 다음 빈 문단 `pi=35` 에는
//! 원문의 저장 vpos 0 이 남아 있다. 합성 줄 vpos 와 그 0 을 비교해 쪽 경계를 추론하면
//! 글꼴 drift 꼬리 넘침 허용이 걸려 셋째 줄이 2쪽 본문 아래끝 밖에 남는다.
//!
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8904/36376635-synthetic-tail-saved-reset-2020.pdf`(3쪽).
//! - 고정형: 전체 쪽수 3.
//! - 관계형: `pi=34` 는 2쪽과 3쪽에 나뉜다. 2쪽 본문 글줄은 모두 본문 아래끝(용지 84188 −
//!   아래 여백 4252HU) 안에 있고, 3쪽 첫 본문 글줄은 `pi=34` 의 넘어온 줄이며 `pi=36` 은 그 뒤다.
//!   PDF 3쪽 첫 줄은 「****니다.」, 그 뒤 빈 줄, 「- 따라서」 순이다.
//!
//! 측정(2026-11-10, 코드 head 1ce9842468(base devel 411d4a91fd),
//! `scripts/visual_sweep.py --silhouette-only --dpi 96`, raw = `silhouette_raw_match_percent`,
//! 문서 face 만 모은 글꼴 폴더 공급):
//! - Native: 3/3쪽, 1쪽 99.92 · 2쪽 99.69 · 3쪽 100.00, 전쪽 최저 99.69.
//! - fresh WASM: 3/3쪽, 1쪽 100.00 · 2쪽 99.78 · 3쪽 100.00, 전쪽 최저 99.78.
//! - 수정 전 Native(base devel 411d4a91fd): 2쪽 97.71 · 3쪽 62.30.
//! - 증거: `mydocs/pr/assets/issue_8904/`(TSV 원본·review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8904/36376635-synthetic-tail-saved-reset.hwpx";
const TOLERANCE_PX: f64 = 0.5;
/// 원본 구역 용지 높이 84188HU − 아래 여백 4252HU(머리말·꼬리말 0).
const BODY_BOTTOM_HU: i32 = 84188 - 4252;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn document() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 쪽의 본문(표 칸 밖) 글줄을 (문단, 위끝, 아래끝)으로 위에서부터 돌려준다.
fn body_runs(core: &DocumentCore, page: u32) -> Vec<(usize, f64, f64)> {
    let tree = core.build_page_render_tree(page).expect("쪽");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let mut runs: Vec<(usize, f64, f64)> = nodes
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.section_index == Some(0)
                    && run.cell_context.is_none()
                    && !run.text.trim().is_empty() =>
            {
                run.para_index
                    .map(|para| (para, node.bbox.y, node.bbox.y + node.bbox.height))
            }
            _ => None,
        })
        .collect();
    runs.sort_by(|a, b| a.1.total_cmp(&b.1));
    runs
}

#[test]
fn synthetic_tail_line_moves_to_next_page_instead_of_overflowing_body() {
    let core = document();
    assert_eq!(core.page_count(), 3, "한컴 engine 2020 Print PDF 는 3쪽");

    let page2 = body_runs(&core, 1);
    let page3 = body_runs(&core, 2);
    assert!(
        page2.iter().any(|(para, _, _)| *para == 34),
        "pi=34 앞 두 줄은 2쪽에 남아야 한다"
    );
    let body_bottom = hu_to_px(BODY_BOTTOM_HU);
    for (para, _, bottom) in &page2 {
        assert!(
            *bottom <= body_bottom + TOLERANCE_PX,
            "2쪽 pi={para} 글줄 아래끝 {bottom:.2}px 가 본문 아래끝 {body_bottom:.2}px 밖이다"
        );
    }

    let first = page3.first().expect("3쪽 본문 글줄");
    assert_eq!(first.0, 34, "3쪽 첫 본문 글줄은 넘어온 pi=34 줄이어야 한다");
    let next_para_top = page3
        .iter()
        .find(|(para, _, _)| *para == 36)
        .map(|(_, top, _)| *top)
        .expect("3쪽 pi=36 글줄");
    assert!(
        first.2 <= next_para_top,
        "3쪽 pi=36 은 넘어온 pi=34 줄 뒤에 와야 한다"
    );
}
