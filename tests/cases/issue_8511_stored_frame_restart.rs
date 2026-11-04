//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8514의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:42:52+09:00 (`825b92d6e54b8a06da5a82924b7327112310d479`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8511/wanju_goods_forms.hwp`; 독립 Print `pdf/issue8511_wanju_goods_forms-2020.pdf`.
//! 전체 26쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 6쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8514_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 문단 사이에 저장 원점이 이미 되감겼으면(새 저장 프레임 시작) 그 뒤의 저장 쪽 경계(문단 첫
//! 줄 vpos 0)는 옛 프레임의 «소비한 prefix» 가 아니다. 그 경계에서 새 쪽을 연다.
//!
//! - 입력: `samples/issue8511/wanju_goods_forms.hwp` — 완주군 «물품관리 조례 시행규칙 별지
//!   제1호서식~별지 제22호서식»(한/글 9.6 저장). `[별지 제7호서식]` 문단의 첫 줄이 vpos 0 이다.
//! - 독립 기준: 같은 HWP 의 한/글 2022 Print `pdf/issue8511_wanju_goods_forms-2020.pdf` — 26쪽.
//!   서식 머리 `[별지 제N호서식]` 의 쪽(0-based)은 아래 `HANCOM_FORM_PAGES` 와 같다.
//! - 수정 전 rhwp 는 `[별지 제7호서식]` 을 6쪽(0-based 5)에 `[별지 제6호서식]` 과 함께 두어
//!   7~10호 서식이 한 쪽씩 앞당겨졌다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 한/글 Print PDF 에서 서식 머리 `[별지 제N호서식]` 이 놓인 쪽(0-based).
/// 13호는 PDF 글자 추출에 머리가 나오지 않아 기준에서 뺀다.
const HANCOM_FORM_PAGES: &[(u32, usize)] = &[
    (1, 0),
    (2, 1),
    (3, 2),
    (4, 3),
    (5, 4),
    (6, 5),
    (7, 6),
    (8, 8),
    (9, 8),
    (10, 9),
    (11, 11),
    (12, 12),
    (14, 15),
    (15, 18),
    (16, 19),
    (17, 21),
    (18, 22),
    (19, 23),
    (20, 24),
    (21, 25),
];

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
fn stored_page_reset_after_new_source_frame_opens_the_next_page() {
    let core = core("samples/issue8511/wanju_goods_forms.hwp");
    let pages = page_texts(&core);
    assert_eq!(pages.len(), 26, "한/글 Print 쪽수");
    for &(form, page) in HANCOM_FORM_PAGES {
        let head = format!("[별지제{form}호서식]");
        let hits: Vec<usize> = pages
            .iter()
            .enumerate()
            .filter(|(_, text)| text.contains(&head))
            .map(|(index, _)| index)
            .collect();
        assert_eq!(hits, vec![page], "{head} 의 쪽");
    }
    // 7호 서식은 저장 쪽 경계에서 새 쪽을 열어 그 쪽의 첫 글자로 온다.
    assert!(
        pages[6].starts_with("[별지제7호서식]"),
        "7쪽은 [별지 제7호서식] 으로 시작한다"
    );
    assert!(
        !pages[5].contains("[별지제7호서식]"),
        "6쪽에 7호 서식을 붙이지 않는다"
    );
}
