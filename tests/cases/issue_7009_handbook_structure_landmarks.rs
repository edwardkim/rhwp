//! #7009: 총 쪽수 상쇄를 부록 간지·본문·부록 길이로 검출한다.
//!
//! 원 PR #7331과 통합 #7332는 과거 KoPub 미설치 출력의 구조 편차를 기록했다.
//! #7540의 독립 한컴 2024+KoPub Print와 이슈의 최종 정정에 따라 현재 정상 계약은
//! 전체 383쪽, 부록 간지 309쪽, 간지 뒤 74쪽이다. #7964의 HWP 전체 보정과 현재
//! 고정 base에서도 이 부록 길이를 확인했다. 과거 미설치 환경의 384쪽·편차 핀은
//! 재도입하지 않으며, 정상 KoPub 환경에서 이미 맞는 부록 구간과 내용 소속을 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const HWP: &str = "samples/2025 행정업무운영 편람(최종).hwp";
const HWPX: &str = "samples/2025 행정업무운영 편람(최종).hwpx";

/// 정답지 `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf` 실측.
const ORACLE_AFTER_DIVIDER: usize = 74;

/// 문서 구조 랜드마크.
#[derive(Debug, PartialEq, Eq)]
struct Landmarks {
    pages: usize,
    /// `부 록` 간지 쪽(1-기반).
    appendix_divider: usize,
    /// 간지 앞에서 내용이 있는 마지막 쪽(1-기반).
    body_last: usize,
    /// 간지 뒤 쪽수.
    after_divider: usize,
}

fn landmarks(sample: &str) -> Landmarks {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let pages = core.page_count() as usize;
    let flat: Vec<String> = (0..pages as u32)
        .map(|page| {
            core.extract_page_text_native(page)
                .unwrap_or_default()
                .split_whitespace()
                .collect::<String>()
        })
        .collect();

    // 간지는 `부록` 으로 시작하고 내용이 거의 없는 쪽이다(목차 항목과 구분하려 길이를 본다).
    let appendix_divider = flat
        .iter()
        .enumerate()
        .filter(|(_, text)| text.starts_with("부록") && text.chars().count() < 40)
        .map(|(index, _)| index + 1)
        .next_back()
        .expect("부록 간지 쪽");
    let body_last = (0..appendix_divider - 1)
        .rev()
        .find(|&index| flat[index].chars().count() > 10)
        .map(|index| index + 1)
        .expect("본문 마지막 쪽");

    Landmarks {
        pages,
        appendix_divider,
        body_last,
        after_divider: pages - appendix_divider,
    }
}

/// HWP의 정상 부록 길이를 보호한다. 전체 쪽수·완료 페이지는 #7751 검사도 대조한다.
/// 정상 부록 74쪽과 간지·본문 존재 검사를 전체 피델리티 통과로 해석하지 않는다.
#[test]
fn hwp_total_matches_the_oracle_only_because_body_and_appendix_cancel() {
    let got = landmarks(HWP);
    let appendix_gap = got.after_divider as i64 - ORACLE_AFTER_DIVIDER as i64;
    // 부록 간지 뒤 PageHide 빈 쪽(정본 310쪽)을 되살려 편차가 −3 → −2 로 줄었고,
    // 부록 표의 저장 쪽 되감김 경계를 지켜 −2 → 0 이 됐다. 간지 뒤 쪽수가 정본
    // (`pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`, 383쪽, 간지 309쪽 뒤 74쪽)과
    // 같다 — 이미 맞는 속성으로 정확히 잠근다.
    assert_eq!(appendix_gap, 0, "부록 구간 길이는 정답지와 같아야 한다");
}

/// HWPX의 정상 부록 길이도 같은 독립 KoPub Print와 정확히 일치해야 한다.
#[test]
fn hwpx_appendix_span_already_matches_the_oracle() {
    let got = landmarks(HWPX);
    assert_eq!(
        got.after_divider, ORACLE_AFTER_DIVIDER,
        "간지 뒤 쪽수는 정답지와 같아야 한다 — 이 문서에서 이미 맞는 속성이다"
    );
}

// 과거 미설치 환경의 포맷 비대칭·쪽수 핀은 재도입하지 않는다.
// HWP/HWPX의 정상 부록 구간과 랜드마크 존재 계약은 유지한다.
