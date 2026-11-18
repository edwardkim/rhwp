//! 관계형·고정형 — [#8894] 저장 줄 없는 글자처럼 취급 표 문단의 쪽 맨 위 문단 위 간격과 합성 사다리.
//! 주석 갱신: 2026-11-10 17:05 KST (새 파일)
//!
//! 입력: 공개 결재문서 36340478(한컴오피스 2020 저장) 원본. 셋째 구역의 1×3 제목 표 문단은
//! 모두 글자처럼 취급(TAC) 표 하나만 가진 문단이며, `pi=48` 뒤부터는 저장 줄(`linesegarray`)이 없다.
//! - 4쪽 맨 위 `pi=17`(위 간격 0, 저장 줄 있음) — 같은 모양의 기준 제목 표.
//! - 6쪽 맨 위 `pi=49`(위 간격 500HU, 쪽 나누기) · 7쪽 맨 위 `pi=56`(위 간격 1000HU, 자연 쪽 넘김).
//! - 7쪽 중간 `pi=60`(위 간격 1000HU) 뒤 `pi=61`(위 간격 500HU) 글 문단.
//!
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8894/36340478-unstored-tac-column-top-spacing-2020.pdf`(7쪽).
//! - 고정형: 전체 쪽수 7.
//! - 관계형: 제목 표 위끝 − 4쪽 제목 표 위끝 = 문단 위 간격. PDF 의 제목 글자 위끝은 4쪽 104.4px,
//!   6쪽 111.0px(+6.6), 7쪽 117.7px(+13.3)이다 — 한/글은 쪽 맨 위에서도 위 간격을 둔다(#8474).
//! - 관계형: 7쪽 (`pi=61` 줄 위끝 − `pi=60` 표 위끝) − (`pi=57` 줄 위끝 − `pi=56` 표 위끝)
//!   = `pi=61` 위 간격 500HU − 두 표 줄의 줄간격 차 32HU(1052 − 1020) = 468HU. PDF 의 「□」 글자 위끝
//!   간격 차는 55.60 − 49.36 = 6.24px(=468HU)다. 합성 사다리가 글 문단의 위 간격을 두 번 넣으면
//!   앞 TAC 줄의 닫힘 검사가 실패해 줄간격 절반만 남는다.
//!
//! 측정(2026-11-10, 코드 head d0dc3bd2e0(base devel 5f4e95c187), `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, 문서 face 굴림·HY헤드라인M·HY견고딕·HY견명조·맑은 고딕 공급,
//! 6쪽은 한양바탕을 더한 쪽 전용 글꼴 묶음으로도 같은 값):
//! - Native: 7/7쪽, 6쪽 97.01 · 7쪽 99.78, 전쪽 최저 97.01.
//! - fresh WASM: 7/7쪽, 6쪽 97.03 · 7쪽 99.99, 전쪽 최저 97.03.
//! - 수정 전 Native(devel 5f4e95c187 — b5064fae1e 와 SVG 동일): 6쪽 50.83 · 7쪽 27.18.
//! - 6쪽 남은 차이는 「예산과목」 칸 글줄의 줄 나눔(글자 폭)이다.
//! - 증거: `mydocs/pr/assets/issue_8894/`(TSV 원본·review·overlay PNG).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8894/36340478-unstored-tac-column-top-spacing.hwpx";
const SECTION: usize = 2;
const TOLERANCE_PX: f64 = 0.5;

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

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = Vec::new();
    walk(node, &mut out);
    out
}

fn title_table_top(core: &DocumentCore, page: u32, para: usize) -> f64 {
    let tree = core.build_page_render_tree(page).expect("쪽");
    nodes(&tree.root)
        .into_iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::Table(table)
                if table.section_index == Some(SECTION)
                    && table.para_index == Some(para)
                    && table.control_index == Some(0) =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("{}쪽 제목 표 pi={para}", page + 1))
}

fn body_line_top(core: &DocumentCore, page: u32, para: usize) -> f64 {
    let tree = core.build_page_render_tree(page).expect("쪽");
    nodes(&tree.root)
        .into_iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run)
                if run.section_index == Some(SECTION)
                    && run.para_index == Some(para)
                    && run.cell_context.is_none() =>
            {
                Some(node.bbox.y)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("{}쪽 pi={para} 글줄", page + 1))
}

#[test]
fn unstored_tac_title_keeps_spacing_before_at_page_top() {
    let core = document();
    assert_eq!(core.page_count(), 7, "한컴 engine 2020 Print PDF 는 7쪽");
    let reference = title_table_top(&core, 3, 17);
    for (page, para, before_hu) in [(5, 49, 500), (6, 56, 1000)] {
        let shift = title_table_top(&core, page, para) - reference;
        let expected = hu_to_px(before_hu);
        assert!(
            (shift - expected).abs() <= TOLERANCE_PX,
            "{}쪽 맨 위 제목 표 pi={para} 가 4쪽 제목 표보다 {shift:.2}px 아래다 — 문단 위 간격 {expected:.2}px 이어야 한다",
            page + 1
        );
    }
}

#[test]
fn unstored_text_after_tac_title_takes_its_spacing_before_once() {
    let core = document();
    let top_gap = body_line_top(&core, 6, 57) - title_table_top(&core, 6, 56);
    let middle_gap = body_line_top(&core, 6, 61) - title_table_top(&core, 6, 60);
    let difference = middle_gap - top_gap;
    let expected = hu_to_px(500 - (1052 - 1020));
    assert!(
        (difference - expected).abs() <= TOLERANCE_PX,
        "7쪽 pi=61 은 pi=60 제목 표 뒤에서 pi=57 보다 {difference:.2}px 더 내려갔다 — 위 간격 500HU − 줄간격 차 32HU = {expected:.2}px 이어야 한다"
    );
}
