//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8530의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-04T20:02:06+09:00 (`13ca465dfcfd5e42d1520fe801cd33c5b9a2b820`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8529/rowbreak_exact_fit.hwpx`; 독립 Print `pdf/issue8529_rowbreak_exact_fit-2020.pdf`.
//! 전체 3쪽, 전쪽 최저 Native 99.94% / fresh WASM 99.96%.
//! 주장 1쪽 raw: Native 99.94% / fresh WASM 99.96%.
//! 주장 2쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8530_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 조각 높이를 **정확히** 채우면 그 컷을 받는다. 같은 컷을 고른 비교 규칙(HU→px 반올림 잡음은
//! 정확히 맞음)과 다르게 쪽 면적 초과 검사가 부동소수 잡음으로 거부해 한 줄을 넘기지 않는다.
//!
//! - 입력: `samples/issue8529/rowbreak_exact_fit.hwpx` — 공개 문서 「(규제영향분석서) 먹는물관리법
//!   시행규칙 일부개정령(안)」 HWPX 에서 `trim_rowbreak_exact_fit.py` 로 구역 속성과 저장 쪽 머리
//!   두 문단(「< 규제 개요 >」·31×17 RowBreak 표)만 그대로 남긴 것. 저장 줄·표 선언 높이는 한/글 값.
//! - 독립 기준: 같은 HWPX 의 한/글 2022 Print `pdf/issue8529_rowbreak_exact_fit-2020.pdf` — 3쪽.
//!   1쪽 끝은 9행(9.도입목표) 셀의 둘째 문단 「□ 용출안전성 검사의 별도 행정처분기준 마련을 통한
//!   정수기 품질 향상」, 2쪽은 셋째 문단 「○ 이와 더불어 …」로 시작해 「일몰설정여부·일몰조문·
//!   연장여부」 머리 행으로 끝나고, 3쪽은 「미설정」으로 시작해 「15.규제정비 계획」으로 끝난다.
//! - 수정 전 rhwp 는 둘째 문단을 2쪽으로 넘겨 2·3쪽 경계가 한 행씩 밀렸다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn collect(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.extend(run.text.chars().filter(|c| !c.is_whitespace()));
    }
    for child in &node.children {
        collect(child, text);
    }
}

/// 쪽의 글자를 그린 순서대로 공백 없이 잇는다.
fn page_text(core: &DocumentCore, page: u32) -> String {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let mut text = String::new();
    collect(&tree.root, &mut text);
    text
}

#[test]
fn rowbreak_cut_that_exactly_fills_the_stored_first_fragment_stays() {
    let core = core("samples/issue8529/rowbreak_exact_fit.hwpx");
    assert_eq!(core.page_count(), 3, "한/글 Print 쪽수");
    let pages: Vec<String> = (0..3).map(|page| page_text(&core, page)).collect();

    const SECOND_PARAGRAPH: &str = "별도행정처분기준마련을통한정수기품질향상";
    const THIRD_PARAGRAPH: &str = "이와더불어용출안전성검사부적합정수기의";

    // 1쪽: 9행 셀의 첫째·둘째 문단까지, 셋째 문단은 없다.
    let first = &pages[0];
    let goal = first.find("9.도입목표").expect("1쪽에 9행 머리");
    let first_paragraph = first
        .find("먹는물수질검사결과거짓작성")
        .expect("1쪽에 9행 첫째 문단");
    let second_paragraph = first
        .find(SECOND_PARAGRAPH)
        .expect("1쪽 끝에 9행 둘째 문단(한/글 1쪽)");
    assert!(
        goal < second_paragraph && first_paragraph < second_paragraph,
        "둘째 문단은 9행 머리·첫째 문단 뒤"
    );
    assert!(!first.contains(THIRD_PARAGRAPH), "셋째 문단은 2쪽");

    // 2쪽: 셋째 문단으로 시작하고 둘째 문단을 다시 담지 않으며 일몰 머리 행으로 끝난다.
    let second = &pages[1];
    assert!(!second.contains(SECOND_PARAGRAPH), "둘째 문단 중복 없음");
    let third_paragraph = second.find(THIRD_PARAGRAPH).expect("2쪽에 셋째 문단");
    let sunset_header = second
        .find("일몰조문")
        .expect("2쪽에 일몰 머리 행(한/글 2쪽 끝)");
    assert!(
        third_paragraph < sunset_header,
        "셋째 문단이 일몰 머리 행 앞"
    );
    assert!(!second.contains("미설정"), "「미설정」 행은 3쪽");

    // 3쪽: 「미설정」 행부터 「15.규제정비 계획」까지, 일몰 머리 행은 다시 그리지 않는다.
    let third = &pages[2];
    assert!(!third.contains("일몰조문"), "일몰 머리 행 중복 없음");
    let unset = third.find("미설정").expect("3쪽에 「미설정」 행");
    let plan = third.find("15.규제정비계획").expect("3쪽에 마지막 행");
    assert!(unset < plan, "「미설정」 행이 마지막 행 앞");
}
