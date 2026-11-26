//! 관계형·고정형 — [#9419] 한/글이 저장한 HWPX 의 쪽 첫 글자처럼 취급 표는 저장 첫 줄이 증언하는 문단 위 간격 아래에 그리며, 같은 문서의 HWP 와 같은 자리다.
//! 주석 갱신: 2026-11-26 15:08 KST (처음 작성)
//!
//! 입력:
//! - `samples/issue9419/kaf_appendix_2020.hwpx` — `samples/issue9379/kaf_appendix.hwp` 를 한/글 2020(11.0.0.1623)으로
//!   HWPX 로 다시 저장한 파일(`samples/issue9419/README.md`).
//! - `samples/issue9379/kaf_appendix.hwp` — 같은 문서의 HWP. 대상 문단의 문단 모양·저장 줄·표는 두 형식이 같다.
//!
//! 21·23쪽은 쪽 나누기 뒤 빈 문단(구역 1 문단 12·14, 위·아래 간격 600 HU = 8px)의 글자처럼 취급 표가 단의
//! 첫 항목이다. 저장 첫 줄 vpos 는 600 으로, 쪽 맨 위에서도 적용한 위 간격이다. 한/글 2020 Print 는 HWP·HWPX 모두 표
//! 테두리를 단 위끝 + 위 간격 + 바깥 위 여백에 그린다(96 DPI 래스터 내용 윗변 122px, 두 PDF 같음).
//!
//! 수정 전 rhwp 는 HWPX 에서만 표를 위 간격만큼 올려(115.3px, HWP 123.3px) 그렸다. HWPX 전용 #2243 표 경로
//! 앵커가 위 간격 앞의 흐름 위치를 저장 줄 윗변 vpos 에 대응시켜 단 원점을 600 HU 어긋나게 했고, HWPX 전용
//! #2279 host 간격 가산(위+아래 간격)이 끝 복원과 아래 간격을 두 번 세어 흐름 끝이 8px 길어졌다. 그래서 같은
//! 저장 줄 원점 수용이 실패하고 표가 단 위끝에 붙었다. 수정 후에는 두 경로가 저장 첫 줄이 증언하는 쪽 위
//! 간격(#8474 `stored_column_top_spacing`)을 함께 소비한다.
//! - 반례: 위 간격이 저장 사다리에 없는(첫 줄 vpos 가 0 이거나 위 간격보다 큰) 생성기 문서는 증언 값이 0 이라
//!   종전과 같다(10k 코퍼스 A/B, PR 본문).
//!
//! 측정(2026-11-26, head f70f6fe602 위 이 검사, `scripts/visual_sweep.py` silhouette_raw_match_percent, 96 DPI,
//! 기준 `pdf/issue9419/kaf_appendix_2020-2020.pdf` 54쪽):
//! - HWPX: 21쪽 Native 98.35 · fresh WASM 99.30, 23쪽 Native 98.67 · fresh WASM 99.91(수정 전 Native 61.63 · 62.25).
//!   전체 54쪽 = PDF 54쪽. 전쪽 최저 Native 94.30 · fresh WASM 92.56(1쪽, 표지 글꼴 대체 NotoSansKRThin-Bold→H2hdrM,
//!   HWP 와 같은 값). 98 미만은 1쪽·24쪽(글꼴 대체, Native 97.40)·36쪽(글상자, Native 95.96 · WASM 96.04, 별건 #9422).
//! - HWP(대조군 `samples/issue9379/kaf_appendix.hwp`): 54쪽, Native 전쪽 값이 수정 전과 같다(21쪽 98.30 · 23쪽 98.57).
//! - 증거: `mydocs/pr/assets/issue_9419/` 의 Native·fresh WASM·수정 전 Native TSV 와 21·23쪽 review·overlay PNG.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const HWPX: &str = "samples/issue9419/kaf_appendix_2020.hwpx";
const HWP: &str = "samples/issue9379/kaf_appendix.hwp";
/// 한컴 PDF 의 전체 쪽수(두 형식 같음).
const HANCOM_PAGES: u32 = 54;
/// (0 기반 쪽, 구역 1 의 host 문단).
const CASES: [(u32, usize); 2] = [(20, 12), (22, 14)];
/// host 문단 위 간격 600 HU.
const SPACING_BEFORE_PX: f64 = 600.0 * 96.0 / 7200.0;

fn open(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {sample}: {e:?}"));
    assert_eq!(
        core.page_count(),
        HANCOM_PAGES,
        "{sample}: 한컴 PDF 의 전체 쪽수"
    );
    core
}

fn column_top(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Column(_)) {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(column_top)
}

/// 쪽의 표 노드를 (host 문단, 윗변) 순서대로 모은다.
fn tables(node: &RenderNode, out: &mut Vec<(Option<usize>, f64)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        out.push((table.para_index, node.bbox.y));
    }
    for child in &node.children {
        tables(child, out);
    }
}

struct PageTop {
    column_top: f64,
    table_top: f64,
}

fn page_top(core: &DocumentCore, sample: &str, page: u32, para: usize) -> PageTop {
    let tree = core.build_page_render_tree(page).unwrap();
    let mut found = Vec::new();
    tables(&tree.root, &mut found);
    assert_eq!(
        found.first().map(|(owner, _)| *owner),
        Some(Some(para)),
        "{sample} {}쪽: 첫 표는 구역 1 문단 {para} 의 표다",
        page + 1
    );
    PageTop {
        column_top: column_top(&tree.root).expect("단"),
        table_top: found[0].1,
    }
}

#[test]
fn hwpx_page_top_tac_table_sits_below_stored_spacing_before_like_hwp() {
    let hwpx = open(HWPX);
    let hwp = open(HWP);
    for (page, para) in CASES {
        let x = page_top(&hwpx, HWPX, page, para);
        let h = page_top(&hwp, HWP, page, para);
        assert!(
            (x.table_top - h.table_top).abs() <= 0.01,
            "{}쪽: HWPX 표 윗변 {:.2} 는 같은 문서 HWP 의 {:.2} 와 같다",
            page + 1,
            x.table_top,
            h.table_top
        );
        assert!(
            x.table_top - x.column_top >= SPACING_BEFORE_PX - 0.01,
            "{}쪽: 표는 단 위끝 {:.2} 에서 위 간격 {:.2}px 아래에 있어야 한다 (윗변 {:.2})",
            page + 1,
            x.column_top,
            SPACING_BEFORE_PX,
            x.table_top
        );
    }
}
