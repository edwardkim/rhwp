//! [#8786] 관계형·고정형 회귀 — 저장 끝을 준 문단과 현재 문단 사이에 재조판 줄로 소비한 문단이
//! 있으면, 그 저장 끝으로 낡은 vpos 되감김을 쪽 경계로 인정하지 않는다.
//!
//! 주석 갱신: 2026-11-08 22:22 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8786/reflowed_gap_stored_rewind.hwpx` — 공개 결재문서 36431892(HWPX, 3쪽).
//!   문단 16·17 은 결재 시스템이 가림표로 다시 쓴 합성 줄(tag 최상위 비트), 문단 15 가 마지막 유효
//!   저장 줄, 문단 18 의 저장 vpos 3152 는 옛 조판의 「다음 쪽 시작」 되감김이다.
//! - 독립 기준: `pdf/issue8786/reflowed_gap_stored_rewind-2020.pdf`(한/글 2022 12.0.0.4547 Print,
//!   3쪽). 문단 18~20(「※ 규격 변경 내용」과 별표 두 줄)을 2쪽에 두고 3쪽은 문단 21 부터다.
//! - 수정 전 rhwp(devel `80fc0c6b44`)는 문단 15 의 저장 끝 817.9px 를 재조판 문단 16·17 소비 뒤
//!   흐름 838.6px 와 견줘 「일치」로 보고, 낡은 되감김으로 문단 18 부터 3쪽에 넘겼다.
//!
//! 측정: 2026-11-08, `scripts/visual_sweep.py --silhouette-only --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 devel `80fc0c6b44` + 이 수정.
//! 1쪽 Native 99.99 / fresh WASM 99.99, 2쪽 99.61 / 99.61(수정 전 Native 89.32), 3쪽 99.70 / 99.71
//! (수정 전 13.01). 전체 3쪽 = PDF 3쪽, 전쪽 최저 Native 99.61(2쪽)·fresh WASM 99.61(2쪽).
//! 증거: `mydocs/pr/assets/issue_8786/{native,wasm,native-base}/`(TSV·review·overlay PNG).
//!
//! 검사: 원본 계약(합성 줄 문단 16·17, 문단 18 되감김), 쪽수 3 = 한/글(고정형, 양쪽 raw 99.61%
//! 이상), 문단 16~20 은 2쪽·문단 21 은 3쪽·중복 없음·2쪽 안 순서 17→18→19→20(관계형).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::LineSeg;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8786/reflowed_gap_stored_rewind.hwpx";

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

/// 본문 흐름의 글줄이 속한 문단 번호(표 칸 안 문단은 제외).
fn body_paragraphs(node: &RenderNode, out: &mut Vec<usize>) {
    match &node.node_type {
        RenderNodeType::Table(_) => return,
        RenderNodeType::TextLine(line) => {
            if let Some(para) = line.para_index {
                if !out.contains(&para) {
                    out.push(para);
                }
            }
        }
        _ => {}
    }
    for child in &node.children {
        body_paragraphs(child, out);
    }
}

fn synthetic(seg: &LineSeg) -> bool {
    seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
}

#[test]
fn stale_rewind_after_reflowed_paragraphs_does_not_break_the_page() {
    let core = load();
    let paragraphs = &core.document().sections[0].paragraphs;

    // 원본 계약: 문단 16·17 은 재조판(합성) 줄, 문단 15 는 마지막 유효 저장 줄, 문단 18 의 저장
    // vpos 는 문단 15 보다 작아지는 되감김이다.
    for index in [16, 17] {
        assert!(
            paragraphs[index].line_segs.iter().all(synthetic),
            "문단 {index} 은 재조판 줄"
        );
    }
    assert!(paragraphs[15].line_segs.iter().all(|seg| !synthetic(seg)));
    let rewind = paragraphs[18].line_segs[0].vertical_pos;
    let previous = paragraphs[15].line_segs.last().unwrap().vertical_pos;
    assert!(
        !synthetic(&paragraphs[18].line_segs[0]) && rewind < previous,
        "문단 18 저장 vpos {rewind} < 문단 15 {previous}"
    );

    // 쪽수는 한/글 2022 Print 와 같은 3쪽이다(고정형).
    assert_eq!(core.page_count(), 3, "전체 쪽수 = 한/글 3쪽");

    // 쪽 소속(관계형): 문단 18·19·20 은 문단 17 과 같은 2쪽, 문단 21 부터 3쪽.
    let mut second = Vec::new();
    body_paragraphs(
        &core.build_page_render_tree(1).expect("2쪽").root,
        &mut second,
    );
    let mut third = Vec::new();
    body_paragraphs(
        &core.build_page_render_tree(2).expect("3쪽").root,
        &mut third,
    );
    for para in [16, 17, 18, 19, 20] {
        assert!(second.contains(&para), "문단 {para} 은 2쪽: {second:?}");
        assert!(
            !third.contains(&para),
            "문단 {para} 을 3쪽에 다시 놓지 않는다"
        );
    }
    assert!(third.contains(&21), "문단 21 은 3쪽: {third:?}");
    assert!(!second.contains(&21), "문단 21 은 2쪽에 없다");
    // 내용 순서: 2쪽 안에서 17 → 18 → 19 → 20.
    let order: Vec<_> = second
        .iter()
        .copied()
        .filter(|para| (17..=20).contains(para))
        .collect();
    assert_eq!(order, vec![17, 18, 19, 20], "2쪽 문단 순서");
}
