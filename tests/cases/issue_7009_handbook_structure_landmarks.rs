//! [#7009] 편람 쪽수 핀이 **거짓 초록**이다 — 총합만 맞고 구조는 어긋난다.
//!
//! ## 무엇이 문제였나
//!
//! `oracle_page_count_baseline.tsv` 는 이 문서를 쪽수 **총합** 하나로 지킨다.
//!
//! ```text
//!   samples/2025 행정업무운영 편람(최종).hwp    정답지 384  rhwp 384   ← 통과
//!   samples/2025 행정업무운영 편람(최종).hwpx   정답지 384  rhwp 382   ← 알려진 격차
//! ```
//!
//! 그런데 `hwp` 의 384 는 **정답지의 384 가 아니다.** 본문이 3쪽 길고 부록이 3쪽 짧아
//! 총합만 맞는다. 쪽수 핀은 이 상쇄를 볼 수 없다.
//!
//! ## 실측 — 구조 랜드마크
//!
//! 정답지는 `pdf/2025 행정업무운영 편람(최종)-hwp-2024.pdf`(384쪽, KoPub 미설치 환경 —
//! 이 저장소의 실행 환경과 같다. 기준 선택 근거는 위 TSV 머리말 참조).
//!
//! ```text
//!                       정답지   rhwp hwp        rhwp hwpx
//!   총 쪽수               384     384  (±0)      382  (−2)
//!   부록 간지 쪽          310     313  (+3)      308  (−2)
//!   본문 마지막 쪽        308     311  (+3)      306  (−2)
//!   부록 간지 뒤 쪽수      74      71  (−3)       74  (±0)
//! ```
//!
//! `hwp` 는 `본문 +3` 과 `부록 −3` 이 정확히 상쇄한다. `hwpx` 는 총합이 2 모자라지만
//! **부록 구간 길이는 정답지와 정확히 같다** — 모자람이 본문에만 있다는 뜻이다.
//!
//! ## 이 시험이 잠그는 것
//!
//! 쪽수 총합으로는 안 보이는 **구조 편차**를 고정한다. 값을 정답지로 맞추라는 요구가
//! 아니라(그 수리는 `#6842` 축이다), **지금의 편차가 더 커지지 않게** 하고 `hwpx` 의
//! 부록 길이처럼 이미 맞는 속성은 정확히 잠근다.
//!
//! ## 잠그지 않는 것
//!
//! 편차를 0 으로 만드는 일은 이 파일의 범위가 아니다. 본문 `+3`/`−2` 의 원인은
//! `#6842`(제5장 질의 상자가 선언 셀 높이보다 +7~+27px 크다)로 귀속돼 있다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const HWP: &str = "samples/2025 행정업무운영 편람(최종).hwp";
const HWPX: &str = "samples/2025 행정업무운영 편람(최종).hwpx";

/// 정답지 `pdf/2025 행정업무운영 편람(최종)-hwp-2024.pdf` 실측.
const ORACLE_PAGES: usize = 384;
const ORACLE_APPENDIX_DIVIDER: usize = 310;
const ORACLE_BODY_LAST: usize = 308;
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

/// `hwp` 의 384 는 정답지의 384 가 아니다 — 본문 `+3` 과 부록 `−3` 의 상쇄다.
///
/// 쪽수 총합 핀이 못 보는 자리를 고정한다. 편차가 더 커지면 실패한다.
#[test]
fn hwp_total_matches_the_oracle_only_because_body_and_appendix_cancel() {
    let got = landmarks(HWP);
    assert_eq!(
        got.pages, ORACLE_PAGES,
        "총 쪽수는 정답지와 같아야 한다(기존 핀과 같은 계약)"
    );

    let body_gap = got.appendix_divider as i64 - ORACLE_APPENDIX_DIVIDER as i64;
    let appendix_gap = got.after_divider as i64 - ORACLE_AFTER_DIVIDER as i64;
    assert_eq!(
        (body_gap, appendix_gap),
        (3, -3),
        "편람 hwp 의 구조 편차가 움직였다 — 부록 간지 {}(정답지 {}) · 간지 뒤 {}쪽(정답지 {}쪽). \
         총 쪽수 {}는 이 둘의 상쇄로 맞는 값이라 쪽수 핀만으로는 회귀를 못 본다",
        got.appendix_divider,
        ORACLE_APPENDIX_DIVIDER,
        got.after_divider,
        ORACLE_AFTER_DIVIDER,
        got.pages,
    );
    assert_eq!(
        got.body_last as i64 - ORACLE_BODY_LAST as i64,
        3,
        "본문 마지막 쪽 편차가 움직였다 — {}(정답지 {})",
        got.body_last,
        ORACLE_BODY_LAST
    );
}

/// `hwpx` 는 총합이 2 모자라지만 **부록 구간 길이는 정답지와 정확히 같다**.
///
/// 모자람이 본문에만 있다는 뜻이고, 그 자리가 `#6842` 축이다. 이미 맞는 속성은 정확히 잠근다.
#[test]
fn hwpx_appendix_span_already_matches_the_oracle() {
    let got = landmarks(HWPX);
    assert_eq!(
        got.after_divider, ORACLE_AFTER_DIVIDER,
        "간지 뒤 쪽수는 정답지와 같아야 한다 — 이 문서에서 이미 맞는 속성이다"
    );
    assert_eq!(
        got.pages as i64 - ORACLE_PAGES as i64,
        -2,
        "편람 hwpx 의 쪽수 격차가 움직였다 — {}쪽(정답지 {}쪽)",
        got.pages,
        ORACLE_PAGES
    );
    assert_eq!(
        got.appendix_divider as i64 - ORACLE_APPENDIX_DIVIDER as i64,
        -2,
        "부록 간지 편차가 움직였다 — {}(정답지 {})",
        got.appendix_divider,
        ORACLE_APPENDIX_DIVIDER
    );
}

/// 두 포맷이 같은 문서인데 구조가 갈린다 — 그 비대칭을 숫자로 남긴다.
#[test]
fn the_two_formats_disagree_on_where_the_appendix_starts() {
    let hwp = landmarks(HWP);
    let hwpx = landmarks(HWPX);
    assert_eq!(
        hwp.appendix_divider as i64 - hwpx.appendix_divider as i64,
        5,
        "같은 문서의 부록 간지가 hwp {} · hwpx {} 로 갈린다(#6842 축)",
        hwp.appendix_divider,
        hwpx.appendix_divider
    );
}
