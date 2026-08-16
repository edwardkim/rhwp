//! Issue #1608: `is_hwp3_origin` 오탐지 — 네이티브 한글2022 HWPX(head version 1.4)에
//! 부당한 HWP3 "마지막 줄" tolerance(1600 HU ≈ 21px)를 부여하던 회귀.
//!
//! `parser/hwpx/mod.rs`의 `is_hwp3_origin = (head version == "1.4")` 판정은 **HWPML 스키마
//! 버전**을 HWP3→HWPX 변환 지표로 오인했다. 네이티브 HWPX(`samples/hwpx/143E...`)는
//! version.xml 증거상 한글2022 직접 저장본(major=5 minor=1, application "Hancom Office
//! Hangul" appVersion 11)이며 head version 1.4 이므로 오탐지 대상이었다.
//!
//! 이 부당 tolerance 는 `page_layout::available_body_height()`에 +21px 를 더해 경계 문서를
//! 한글보다 1쪽 적게 렌더한다(Task #1600 통제셋 −1쪽 군집의 요인 A).
//!
//! 본 테스트는 파싱 결과 모든 섹션의 `pagination_bottom_tolerance` 가 0 임을 단언한다.
//! (이 값은 파일 포맷 필드가 아니라 렌더러 내부 보정치이므로 네이티브엔 0이어야 한다.)

use std::fs;

use rhwp::parser::hwpx::parse_hwpx;

#[test]
fn native_hwpx_v14_has_no_hwp3_pagination_tolerance() {
    let path = "samples/hwpx/143E433F503322BD33.hwpx";
    let data = fs::read(path).unwrap_or_else(|e| panic!("{path} 읽기 실패: {e}"));
    let doc = parse_hwpx(&data).expect("HWPX 파싱 실패");

    assert!(!doc.sections.is_empty(), "섹션이 비어 있음");
    for (si, section) in doc.sections.iter().enumerate() {
        let tol = section.section_def.page_def.pagination_bottom_tolerance;
        assert_eq!(
            tol, 0,
            "섹션 {si}: 네이티브 HWPX(head 1.4)에 부당한 HWP3 tolerance({tol} HU)가 부여됨 (#1608)"
        );
    }
}
