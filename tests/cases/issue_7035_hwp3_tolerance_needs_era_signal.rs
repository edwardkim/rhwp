//! [Issue #7035] 한글97 마지막 줄 허용치가 진짜 HWP5 문서에 붙어 본문 바닥을 넘긴다.
//!
//! HWP3 파서는 `pagination_bottom_tolerance = 1600 HU`(21.3px)를 세운다 — 한글97 이
//! 마지막 줄을 본문 밖으로 조금 넘겨 두던 동작을 흉내 내는 **렌더러 내부 값**이다.
//! HWP5 문서에도 "HWP3 변환본으로 의심되면" 같은 값을 붙이는데, 그 판정을 두 곳이
//! 서로 다르게 했다.
//!
//! ```text
//!   is_hwp3_variant  ([Task #1001])   summary_hwp3_era  AND  비율   -> 레이아웃 프로파일
//!   apply_hwp3_origin_fixup ([#554])  비율만                        -> 조판 예산(허용치)
//! ```
//!
//! 더 약한 쪽이 조판 예산을 쥐고 있었다. 진짜 HWP5 문서가 비율만으로 허용치를 받아
//! `available = 본문 + 21.3px` 이 되고, 쪽마다 마지막 한 줄이 본문 바닥을 넘겨 그려졌다
//! (속기자료 4문서 30건, `+2.9 ~ +11.4px`). 한/글 2020 정본은 그 줄들을 다음 쪽 첫머리에
//! 둔다.
//!
//! # 기대값의 출처
//!
//! 결정론 신호(`HwpSummaryInformation` 의 1990~2003년 표기)가 두 무리를 완전히 가른다.
//!
//! ```text
//!   저장소 HWP3 -> HWP5 변환본 16건   summary_hwp3_era = true    16/16
//!   속기자료 4문서 (진짜 HWP5)         summary_hwp3_era = false    4/4
//! ```
//!
//! 속기자료는 `version 5.0.0.6` · 저장 `5.7.9.3051` · HWP3 출처 마커 없음이다.
//!
//! # 이 시험이 잠그는 것
//!
//! **양성 방향** — 진짜 HWP3 변환본은 허용치를 그대로 받는다. 수정이 이 계약을 건드리지
//! 않았음을 저장소 표본으로 고정한다(`SO-SUEOP` 44쪽 · `issue_554` 등이 이 값에 매달려
//! 있다). 여기 5건은 저장소 HWP3→HWP5 변환본 중 `[#554]` 비율이 실제로 발화해 허용치를
//! 받는 전부다. 나머지 변환본(`hwp3-sample13/14/16/19-hwp5`)은 **수정 전에도 0** 이다 —
//! 두 판정의 비율 문턱이 다르기 때문이며(`[Task #1001]` 은 `< 0.20` 로 느슨해 lineage 만
//! 세운다) 이 이슈의 범위가 아니다.
//!
//! 음성 방향(결정론 신호가 없으면 허용치를 주지 않는다)은 판정 함수가 파서 모듈 내부라
//! 같은 모듈의 단위 시험으로 잠근다 —
//! `parser::tests::hwp3_origin_tolerance_requires_the_deterministic_era_signal`.
//! 결함을 드러내는 쪽은 그 시험이다(수정 전 `1600`, 수정 후 `0`). 이 파일은 그 문지기가
//! 진짜 변환본까지 막지 않는지를 실제 문서로 확인하는 반대편 잠금이다.
//!
//! # 잠그지 않는 것
//!
//! 허용치의 값(1600 HU)과 HWP3 **원본** 경로(파서가 직접 세운다)는 범위 밖이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::parse_document;
use std::path::Path;

fn tolerance_of(rel: &str) -> i32 {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let doc = parse_document(&std::fs::read(path).expect("표본 읽기")).expect("파싱");
    doc.sections
        .first()
        .expect("구역")
        .section_def
        .page_def
        .pagination_bottom_tolerance as i32
}

/// 진짜 HWP3→HWP5 변환본은 한글97 마지막 줄 허용치를 그대로 받는다.
#[test]
fn hwp3_converted_documents_keep_the_last_line_tolerance() {
    let mut checked = 0usize;
    for rel in [
        "samples/hwp3-sample-hwp5.hwp",
        "samples/hwp3-sample10-hwp5.hwp",
        "samples/hwp3-sample11-hwp5.hwp",
        "samples/hwp3-sample4-hwp5.hwp",
        "samples/hwp3-sample5-hwp5.hwp",
    ] {
        let tolerance = tolerance_of(rel);
        assert!(
            tolerance > 0,
            "{rel}: HWP3 변환본인데 한글97 마지막 줄 허용치를 잃었다 (= {tolerance})"
        );
        checked += 1;
    }
    assert_eq!(
        checked, 5,
        "표본 5건을 모두 재야 한다 — 실행 대상이 줄면 통과 증거가 아니다"
    );
}

/// 결정론 신호가 없는 진짜 HWP5 는 비율만으로 허용치를 받지 못한다.
///
/// `samples/issue7035/native_hwp5_low_style_ratio.hwp` 는 `#7035` 이 신고한 속기자료
/// (`148737458` · `version 5.0.0.6` · 저장 `5.7.9.3051`)에서 `extract-pages --from 1
/// --to 27` 으로 떼어낸 재현체다. `[#554]` 비율 휴리스틱의 발화 조건(문단 50개 초과 ·
/// ParaShape/문단 < 0.05 · CharShape/문단 < 0.15)을 그대로 만족하면서
/// `HwpSummaryInformation` 에 1990~2003년 표기가 **없다**.
///
/// 이 시험이 결함을 드러낸다 — 수정 전 `1600`, 수정 후 `0`.
#[test]
fn native_hwp5_without_the_era_signal_gets_no_tolerance() {
    let rel = "samples/issue7035/native_hwp5_low_style_ratio.hwp";
    let tolerance = tolerance_of(rel);
    assert_eq!(
        tolerance, 0,
        "{rel}: HWP3 시대 신호가 없는 진짜 HWP5 인데 한글97 마지막 줄 허용치를 받았다          (= {tolerance}) — 조판 예산이 본문보다 넓어져 쪽마다 마지막 줄이 바닥을 넘는다"
    );
}

/// 위 재현체가 **비율 조건은 실제로 만족**하는지 못 박는다.
///
/// 이것이 깨지면 위 시험은 "신호 문지기가 동작한다" 가 아니라 "비율이 애초에 발화하지
/// 않는다" 를 재는 공허한 시험이 된다.
#[test]
fn the_fixture_still_satisfies_the_ratio_heuristic() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7035/native_hwp5_low_style_ratio.hwp");
    let doc = parse_document(&std::fs::read(path).expect("표본 읽기")).expect("파싱");
    let paragraphs: usize = doc.sections.iter().map(|s| s.paragraphs.len()).sum();
    let ps = doc.doc_info.para_shapes.len() as f64 / paragraphs as f64;
    let cs = doc.doc_info.char_shapes.len() as f64 / paragraphs as f64;
    assert!(
        paragraphs > 50,
        "문단 {paragraphs}개 — 50개 초과여야 발화한다"
    );
    assert!(
        ps < 0.05,
        "ParaShape 비율 {ps:.4} — 0.05 미만이어야 발화한다"
    );
    assert!(
        cs < 0.15,
        "CharShape 비율 {cs:.4} — 0.15 미만이어야 발화한다"
    );
}
