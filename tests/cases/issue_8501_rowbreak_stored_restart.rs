//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8505의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:42:35+09:00 (`624aeaed3d211188ba6d8894632f952ee16bb281`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8501/child_product_safety_rule.hwp`; 독립 Print `pdf/issue8501_child_product_safety_rule-2020.pdf`.
//! 전체 19쪽, 전쪽 최저 Native 99.78% / fresh WASM 99.80%.
//! 주장 6쪽 raw: Native 99.86% / fresh WASM 99.82%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8505_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 다시 시작하는 자리(한/글이 그 행을 나눈 기록)에서 나눈다. 셀 내용이 남은 예산에 겨우
//! 들어간다는 이유로 행 전체를 앞 쪽에 두지 않는다.
//!
//! - 입력: `samples/issue8501/child_product_safety_rule.hwp` — 산업통상자원부 «(법령안) 어린이제품
//!   안전 특별법 시행규칙 일부개정령(안)»(한/글 2020 저장). 1구역 20×5 표의 행 16 2열 셀
//!   「안전확인신고표시 | 사용금지 | 2개월」 은 셋째 문단 첫 줄이 vpos 0 으로 저장돼 있다.
//! - 독립 기준: 같은 HWP 의 한/글 2022 Print `pdf/issue8501_child_product_safety_rule-2020.pdf`
//!   — 19쪽. 6쪽은 행 16 을 3줄에서 끝내고, 7쪽 첫머리가 그 셀의 「2개월」 이다.
//! - 수정 전 rhwp 는 행 16 의 4줄을 모두 6쪽에 두어 7쪽이 행 17(「3) 국가기술표준원장이…」)로
//!   시작했다.
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

#[test]
fn page_end_row_splits_at_stored_paragraph_restart() {
    let core = core("samples/issue8501/child_product_safety_rule.hwp");
    let pages = page_texts(&core);
    assert_eq!(pages.len(), 19, "한/글 Print 쪽수");

    let row16 = "2)국가기술표준원장이고시";
    let row17 = "3)국가기술표준원장이고시";
    // 행 16 은 6쪽(0-based 5)에서 시작하고, 행 17 은 7쪽에 있다.
    assert!(pages[5].contains(row16), "행 16 은 6쪽에서 시작한다");
    assert!(!pages[5].contains(row17), "행 17 은 6쪽에 없다");
    assert!(pages[6].contains(row17), "행 17 은 7쪽에 있다");
    assert!(
        !pages[6].contains(row16),
        "행 16 의 첫 줄들은 7쪽에 다시 그리지 않는다"
    );

    // 행 16 의 저장 재시작 뒤 줄(「2개월」)은 7쪽 첫머리, 행 17 앞에 온다.
    let tail = pages[6].find("2개월").expect("7쪽에 행 16 꼬리 「2개월」");
    let next = pages[6].find(row17).unwrap();
    assert!(tail < next, "7쪽은 행 16 의 꼬리 줄로 시작한다");
    // 6쪽은 행 16 의 셋째 줄(「사용금지」)까지 담고, 그 뒤 꼬리 줄은 담지 않는다.
    let head = &pages[5][pages[5].find(row16).unwrap()..];
    assert!(
        head.contains("사용금지"),
        "6쪽에 행 16 의 저장 재시작 앞 줄"
    );
    assert!(
        !head.contains("2개월"),
        "저장 재시작 뒤 줄은 6쪽에 두지 않는다"
    );
}
