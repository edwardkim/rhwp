//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8496 · PR #8505의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:41:06+09:00 (`5a091f479c46629e6991607ecac5c6d05fd0f9c5`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8493/table_tail.hwp`; 독립 Print `pdf/issue8493_table_tail-2020.pdf`.
//! 전체 3쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 2쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8496_review.md`.
//! 입력 `samples/issue8501/child_product_safety_rule.hwp`; 독립 Print `pdf/issue8501_child_product_safety_rule-2020.pdf`.
//! 전체 19쪽, 전쪽 최저 Native 99.78% / fresh WASM 99.80%.
//! 주장 6쪽 raw: Native 99.86% / fresh WASM 99.82%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8505_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 꼬리 쪽의 좌표다. 그 값을 표 host 줄(표가 시작한 앞 쪽의 좌표)과 비교한 «되감김» 으로
//! 문단을 통째로 넘기지 않는다. 이 쪽의 흐름 위치와 비교한다.
//!
//! - 입력: `samples/issue8493/table_tail.hwp` — `gen_table_tail.py` 의 입력을 한/글 2022
//!   (12.0.0.4547)가 다시 저장한 HWP(저장 줄 사다리는 한/글 값).
//! - 독립 기준: 같은 HWP 의 한/글 2022 Print `pdf/issue8493_table_tail-2020.pdf` — 3쪽.
//!   2쪽에 표 꼬리(`ROW 12`..`ROW 31`)와 문단 첫 줄 `LINE01` 이 함께 있다.
//! - 수정 전 rhwp 는 4쪽(2쪽에 표 꼬리만, 문단은 3쪽부터)이었다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn collect_runs(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

/// 쪽마다 그려진 글자(공백 제거)를 이어 붙인 문자열.
fn page_texts(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("렌더 트리");
            let mut text = String::new();
            collect_runs(&tree.root, &mut text);
            text.replace(' ', "")
        })
        .collect()
}

fn pages_with(pages: &[String], needle: &str) -> Vec<usize> {
    pages
        .iter()
        .enumerate()
        .filter(|(_, text)| text.contains(needle))
        .map(|(page, _)| page)
        .collect()
}

#[test]
fn paragraph_after_mid_page_table_tail_stays_on_the_tail_page() {
    let core = core("samples/issue8493/table_tail.hwp");
    let pages = page_texts(&core);
    assert_eq!(pages.len(), 3, "한/글 Print 쪽수");
    // 표 행은 순서대로 한 번씩, 꼬리는 2쪽에서 끝난다.
    for row in 1..=31 {
        let label = format!("ROW{row:02}");
        let hits = pages_with(&pages, &label);
        assert_eq!(
            hits.len(),
            1,
            "{label} 은 한 쪽에만 그려져야 한다: {hits:?}"
        );
    }
    assert_eq!(pages_with(&pages, "ROW31"), vec![1]);
    // 뒤 문단은 표 꼬리 쪽에서 시작한다.
    assert_eq!(
        pages_with(&pages, "LINE01"),
        vec![1],
        "표가 끝난 쪽의 vpos 를 표가 시작한 쪽의 host vpos 와 비교해 넘기지 않는다"
    );
    let tail = &pages[1];
    assert!(
        tail.find("ROW31").unwrap() < tail.find("LINE01").unwrap(),
        "2쪽에서 표 꼬리 뒤에 문단이 온다"
    );
    assert_eq!(pages_with(&pages, "END"), vec![2]);
    // 문단 줄은 누락·중복 없이 원본 순서로 이어진다.
    let mut last = (0usize, 0usize);
    for line in 1..=40 {
        let label = format!("LINE{line:02}");
        let hits = pages_with(&pages, &label);
        assert_eq!(hits.len(), 1, "{label}: {hits:?}");
        let at = (hits[0], pages[hits[0]].find(&label).unwrap());
        assert!(at > last, "{label} 순서");
        last = at;
    }
}
