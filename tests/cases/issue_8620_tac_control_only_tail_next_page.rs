//! [#8620] 관계형·고정형 회귀 — 본문 글자가 없는 글자처럼 취급 표 host 문단에서 표 줄 뒤의
//! 컨트롤 전용 줄(쪽 번호 감추기 등)은 표 줄 간격을 지난 자리가 쪽에 들어가지 않으면 다음
//! 쪽 맨 위에 놓인다. 그 줄만 있는 쪽도 한/글 출력의 한 쪽이다.
//!
//! 주석 갱신: 2026-11-05 (KST) — 측정값은 아래 표
//!
//! - 입력: `samples/issue8620/summary_tail_trimmed.hwpx` — 통계품질진단 2013 중간보고서
//!   (1인창조기업실태조사, HWP5, 한컴 96쪽)에서 구역 0 문단 199~203(요약문 머리 상자·제목·
//!   요약문 표 host·차례 머리 상자·「차 례」)만 남겨 한컴 2022 로 다시 저장한 축소본.
//!   문단 2 = 6×2 글자처럼 취급 표 줄(vpos 8100, 줄 높이 48675, 줄간격 38936 = 180%) +
//!   컨트롤 전용 줄(쪽 번호 감추기 ×2, 저장 vpos 0 = 다음 쪽). 문단 3 은 쪽 나누기.
//! - 독립 기준: 같은 입력의 한컴 2022 Print PDF `pdf/issue8620_summary_tail_trimmed-2020.pdf`
//!   3쪽 — 1쪽 요약문, 2쪽 빈 쪽(꼬리 줄), 3쪽 차례.
//! - 수정 전 rhwp(devel ed057276c1): 2쪽. 꼬리 줄과 표 줄 간격을 버려 빈 쪽이 없고 차례가
//!   2쪽이다(원본 전체 문서는 한컴 96쪽 대 95쪽).
//!
//! 측정(2026-11-05, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 이웃 내용 실루엣, 수정 코드):
//!
//! | 입력 | 전체 쪽수(한컴=rhwp) | Native 쪽별 | fresh WASM 쪽별 |
//! |---|---|---|---|
//! | summary_tail_trimmed | 3 | 98.93 100.00 99.92 | 100.00 100.00 100.00 |
//!
//! 두 경로 전쪽 최저: 98.93(1쪽 Native — 표 안 글자 폭 차이, 표 외곽·행 구성은 한컴과 같다). 수정 전 Native 는 2쪽으로 한컴과 쪽수가 달랐다(2쪽 0.00).
//!
//! 증거: `mydocs/pr/assets/issue_8620/{native,wasm}/*_silhouette.tsv`, `review/*.png`.
//! 검사는 표·문단의 쪽 소속과 꼬리 줄 쪽의 내용 관계(관계형)와 독립 PDF 전체 쪽수(고정형)다.
//! 절대 px 좌표는 고정하지 않는다. 같은 쪽에 이어지는 꼬리 줄은
//! `tac_host_control_only_tail_line.rs` 가 지킨다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

/// 본문 최상위 표의 (문단, 컨트롤).
fn top_tables(node: &RenderNode, out: &mut Vec<(usize, usize)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if let (Some(para), Some(control)) = (table.para_index, table.control_index) {
            out.push((para, control));
        }
        return;
    }
    for child in &node.children {
        top_tables(child, out);
    }
}

fn visible_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(run.display_or_text().trim());
    }
    for child in &node.children {
        visible_text(child, out);
    }
}

fn page(core: &DocumentCore, page: u32) -> (Vec<(usize, usize)>, String) {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let (mut tables, mut text) = (Vec::new(), String::new());
    top_tables(&tree.root, &mut tables);
    visible_text(&tree.root, &mut text);
    (tables, text)
}

#[test]
fn control_only_tail_line_past_page_end_starts_next_page() {
    let core = core("samples/issue8620/summary_tail_trimmed.hwpx");
    assert_eq!(core.page_count(), 3, "한컴 Print PDF 3쪽");

    let (first_tables, first_text) = page(&core, 0);
    assert!(
        first_tables.contains(&(2, 0)),
        "1쪽에 요약문 표: {first_tables:?}"
    );
    assert!(first_text.contains("최종보고서에"), "요약문 표 내용은 1쪽");

    // 꼬리 줄(쪽 번호 감추기만 있는 줄)만 있는 쪽: 표도 글자도 없다.
    let (tail_tables, tail_text) = page(&core, 1);
    assert!(tail_tables.is_empty(), "2쪽에 표가 없다: {tail_tables:?}");
    assert!(
        tail_text.is_empty(),
        "2쪽에 보이는 글자가 없다: {tail_text:?}"
    );

    let (toc_tables, toc_text) = page(&core, 2);
    assert!(
        toc_tables.contains(&(3, 0)) && !toc_tables.contains(&(2, 0)),
        "3쪽은 쪽 나누기 뒤 차례 머리 상자만: {toc_tables:?}"
    );
    assert!(toc_text.contains('차'), "차례 제목은 3쪽: {toc_text:?}");
}
