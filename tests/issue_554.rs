//! Task #554 회귀 테스트
//!
//! HWP3 → HWP5/HWPX 변환본 페이지네이션 정합 검증.
//!
//! ## 본질
//!
//! 한컴이 HWP3 → HWP5/HWPX 변환 시 한글97의 "마지막 줄 tolerance" (1600 HU)
//! 동작이 누락되어 페이지 수가 +1 ~ +4 증가했다. Task #554 에서 변환본
//! 식별 휴리스틱 + 조건부 `margin_bottom -= 1600` 보정으로 정정.
//!
//! ## 식별 휴리스틱
//!
//! - HWPX: `<hh:head version="1.4">`
//! - HWP5: `(ParaShape/Paragraph < 0.05) AND (CharShape/Paragraph < 0.15) AND (Paragraph > 50)`
//!
//! Task #554 Stage 1 진단 결과 (27 fixture 100% 정확).

use std::fs;
use std::path::Path;

fn page_count(rel_path: &str) -> usize {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"));
    doc.page_count() as usize
}

// ───────── HWP3 변환본 (Task #554 핵심 대상): 정답 정합 ─────────

#[test]
fn hwp3_sample4_hwp5_36p() {
    // HWP3 변환본 (36페이지 정답): 한컴 정답 일치
    assert_eq!(page_count("samples/hwp3-sample4-hwp5.hwp"), 36);
}

#[test]
fn hwp3_sample5_hwp5_64p() {
    // HWP3 변환본 (64페이지 정답)
    assert_eq!(page_count("samples/hwp3-sample5-hwp5.hwp"), 64);
}

#[test]
fn hwp3_sample5_hwpx_64p() {
    // HWPX 변환본 (hwpml=1.4)
    assert_eq!(page_count("samples/hwp3-sample5-hwpx.hwpx"), 64);
}

// ───────── 알려진 잔존 -1 over-correct (현재 본질적 한계) ─────────
//
// 단일 -1600 HU 보정의 한계로 sample 변환본은 정답 16 vs 결과 15.
// 향후 별도 task에서 typeset.rs 페이지 break 알고리즘 정밀화 예정.
// 현재는 -1 over-correct를 회귀 가드로 검증.

#[test]
fn hwp3_sample_hwp5_15p_known_limit() {
    // 정답 16, 현재 15 (잔존 -1)
    assert_eq!(page_count("samples/hwp3-sample-hwp5.hwp"), 15);
}

#[test]
fn hwp3_sample_hwpx_15p_known_limit() {
    // 정답 16, 현재 15 (잔존 -1)
    assert_eq!(page_count("samples/hwp3-sample-hwpx.hwpx"), 15);
}

// ───────── HWP3 원본 회귀 0 (Task #460 보정과 충돌 없음) ─────────

#[test]
fn hwp3_sample_hwp3_16p() {
    // HWP3 원본 — 기존 -1600 HWP3 파서 보정 (Task #460) 그대로 작동
    assert_eq!(page_count("samples/hwp3-sample.hwp"), 16);
}

#[test]
fn hwp3_sample5_hwp3_64p() {
    // HWP3 원본
    assert_eq!(page_count("samples/hwp3-sample5.hwp"), 64);
}

// ───────── 광범위 회귀 0 (휴리스틱 false positive 없음) ─────────

#[test]
fn task554_no_regression_2022_kuglip() {
    // 2022년 국립국어원: 단순 -1600 적용 시 -5 회귀였던 케이스
    // 휴리스틱 (PS/CS 비율) 로 변환본이 아니라 정확히 분류 → 보정 미적용
    // [Task #643] 페이지 분할 드리프트 정정으로 페이지당 컨텐츠 수용량 증가:
    //   pi=80 (page 6 마지막 문단) 이 다음 페이지로 분리되지 않고 페이지 6 에 fits
    //   → 후속 페이지 압축 → 40 → 38 페이지 (HWP 원본 정합 회복)
    assert_eq!(page_count("samples/2022년 국립국어원 업무계획.hwp"), 38);
}

#[test]
fn task554_no_regression_exam_kor() {
    // exam_kor: PS/Para=0.076 (휴리스틱 임계값 0.05 근처) — CS/Para=0.214 로 안전 분리
    assert_eq!(page_count("samples/exam_kor.hwp"), 20);
}

#[test]
fn task554_no_regression_aift() {
    // aift: 단순 -1600 적용 시 -1 회귀였던 케이스
    assert_eq!(page_count("samples/aift.hwp"), 77);
}

#[test]
fn task554_no_regression_2025_donations_hwpx() {
    // 2025년 기부·답례품 HWPX: hwpml=1.5 (직접 작성) — 휴리스틱 미적용
    assert_eq!(page_count("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx"), 30);
}

#[test]
fn task554_no_regression_exam_science() {
    // exam_science.hwp 4페이지 (Task #546 정합 유지)
    assert_eq!(page_count("samples/exam_science.hwp"), 4);
}
