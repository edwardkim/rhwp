//! 고정형(쪽 번호·쪽수·쪽별 머리말)·관계형(조정 쪽 번호 = 앞 쪽 + 2) — [#8482] 홀/짝수 조정은 번호만 하나 건너뛴다
//! 주석 갱신: 2026-11-26 11:51 KST (새 파일).
//!
//! 홀/짝수 조정(`pgct` 홀수 쪽 시작)은 빈 쪽을 끼우지 않는다. 새 번호를 적용한 뒤 그 쪽 번호의
//! 홀짝이 요구와 다르면 번호 하나만 건너뛰고, 머리말 홀짝도 그 번호를 따른다.
//!
//! 측정일 2026-11-26. raw 측정: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`
//! 의 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣). source `0b54e47ed6`
//! (base upstream/devel `cbebdbeeca`). 앞 base `32014de6e0`·`fe431122f2` 의 측정과 전쪽 TSV·PNG 가 바이트 단위로 같다.
//!
//! 원본: `samples/issue8482/germany_law_ch3_resaved2020.hwp` (sha256 945183c9…401f) — 이슈의 HWP3
//! 문서(독일 입법심사, 264쪽)를 rhwp 로 HWP5 변환해 한/글 2020 으로 다시 저장한 뒤(한/글 서버는 HWP3
//! 를 열지 못한다), 131~141쪽만 `extract-pages` 로 남겨 한/글 2020(11.0.0.1623)으로 다시 저장한 11쪽
//! 문서. 기준 PDF: `pdf/issue8482/germany_law_ch3-2020.pdf` (한/글 2020 11.0.0.1623 Print, 11쪽,
//! sha256 f393c431…01ad). 발췌로 쪽 번호 위치 조판부호가 빠져 번호는 찍히지 않으며, 한/글이 6쪽에
//! 홀수 쪽 머리말을 찍는 것이 그 쪽 번호가 홀수(7)라는 독립 근거다.
//!
//! | 쪽 | Native base | Native head | fresh WASM base | fresh WASM head |
//! |---|---|---|---|---|
//! | 6 (제3장 시작, 번호 7) | 92.08 | 93.80 | 92.32 | 93.99 |
//! | 7 (짝수 쪽 머리말) | 93.02 | 94.60 | 93.09 | 94.64 |
//!
//! 전체 11/11쪽 일치. head 전쪽 최저: Native 90.58(11쪽), fresh WASM 90.81(11쪽). 90% 미만 없음
//! (base 11쪽은 Native 88.86 / WASM 89.10). 원 264쪽 문서(한/글 2020 Print, 발췌 전 저장본
//! sha256 98aaac35…, PDF sha256 068db0bb…; 저장소에 넣지 않음)는 base 쪽 번호 129쪽 불일치 → head 0쪽,
//! Native·WASM 모두 129쪽 상승·하락 0쪽. 210쪽(번호 215, 홀수 머리말)은 Native 42.94 → 92.30,
//! WASM 43.31 → 92.04. 그 문서의 head 90% 미만 29쪽(1·2·63·64·164~168·178~180·182~184·186·188·189·
//! 206·212·256~264)은 base 에서도 90% 미만이고 내려간 쪽은 없다 — 256~264쪽은 번호·머리말이 맞고 남은
//! 차이는 라틴 양쪽 정렬의 낱말 간격·하이픈(별도 축)이다.
//! 증거: `mydocs/pr/assets/issue_8482/{native,wasm,base}/` 의 전쪽 TSV 와 6·7쪽 review·overlay PNG,
//! `mydocs/pr/assets/issue_8482/full/` 의 264쪽 전쪽 TSV 와 210·260쪽 review·overlay PNG.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8482/germany_law_ch3_resaved2020.hwp";
/// 한/글 2020 Print 쪽수 — 홀/짝수 조정은 빈 쪽을 끼우지 않는다.
const HANCOM_PAGES: u32 = 11;
/// 제3장 시작 쪽(1부터 센 쪽 순번). 이 쪽 첫 문단에 「홀수 쪽 시작」 조정이 있다.
const CHAPTER3_PAGE: u32 = 6;
const ODD_HEADER_CH2: &str = "제5절독일의입법보좌기구";
const ODD_HEADER: &str = "제1절입법심사의체계와방법론";
const EVEN_HEADER: &str = "제3장독일의입법심사의체계와심사기준";

/// 한/글 2020 Print 의 쪽별 머리말. 제2장의 짝수 쪽 머리말은 발췌로 지워진 앞 문단에 있어
/// 2·4쪽은 비어 있다. 6쪽은 순번으로 짝수지만 홀수 쪽 머리말을 찍는다 — 번호가 7 이다.
const HANCOM_HEADERS: [&str; 11] = [
    ODD_HEADER_CH2,
    "",
    ODD_HEADER_CH2,
    "",
    ODD_HEADER_CH2,
    ODD_HEADER,
    EVEN_HEADER,
    ODD_HEADER,
    EVEN_HEADER,
    ODD_HEADER,
    EVEN_HEADER,
];

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open sample")
}

fn page_number(core: &DocumentCore, page: u32) -> u64 {
    let info: serde_json::Value =
        serde_json::from_str(&core.get_page_info_native(page).unwrap()).unwrap();
    info["pageNumber"].as_u64().unwrap()
}

/// 머리말 영역(`RenderNodeType::Header`) 안의 글만 모은다 — 본문의 장 제목과 섞지 않는다.
fn header_text(node: &RenderNode, inside: bool, out: &mut String) {
    let inside = inside || matches!(node.node_type, RenderNodeType::Header);
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if inside {
            out.push_str(run.display_or_text());
        }
    }
    for child in &node.children {
        header_text(child, inside, out);
    }
}

fn painted_header(core: &DocumentCore, page: u32) -> String {
    let mut out = String::new();
    header_text(
        &core.build_page_render_tree(page).unwrap().root,
        false,
        &mut out,
    );
    out.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn odd_start_skips_one_number_without_inserting_a_page() {
    let core = core();
    assert_eq!(
        core.page_count(),
        HANCOM_PAGES,
        "한/글 2020 Print 와 같은 쪽수 — 빈 쪽을 끼우지 않는다"
    );
    let numbers: Vec<u64> = (0..HANCOM_PAGES).map(|p| page_number(&core, p)).collect();
    // 1~5쪽은 1부터 하나씩. 짝수 차례(6)인 제3장 시작 쪽은 홀수 7 로 건너뛰고 뒤는 하나씩.
    assert_eq!(numbers, vec![1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12]);
    // 관계: 조정 쪽 번호는 홀수이고, 앞 쪽보다 2 크다(번호 하나만 건너뜀).
    let at = numbers[(CHAPTER3_PAGE - 1) as usize];
    assert_eq!(at % 2, 1, "홀수 쪽 시작 조정 쪽은 홀수 번호");
    assert_eq!(at, numbers[(CHAPTER3_PAGE - 2) as usize] + 2);
}

#[test]
fn header_follows_the_adjusted_page_parity() {
    let core = core();
    let painted: Vec<String> = (0..core.page_count())
        .map(|p| painted_header(&core, p))
        .collect();
    assert_eq!(
        painted, HANCOM_HEADERS,
        "쪽별 머리말이 한/글 2020 Print 와 같다 — 제3장 시작 쪽은 홀수 쪽 머리말"
    );
}
