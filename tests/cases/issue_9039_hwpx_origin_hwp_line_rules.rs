//! 관계형·고정형 — [#9039] `/RhwpHwpxOrigin` = 1 HWP 를 HWPX 로 저장해도 다시 열 때 저장 줄을 같은 규칙(HWP)으로 읽는다.
//! 주석 갱신: 2026-11-22 20:45 KST (새 파일)
//!
//! 입력: `samples/issue2439_zero_offset_coanchored_float_exclusion.hwp` — `/RhwpHwpxOrigin` 값 `1`
//! (원본 HWPX 저장 줄 계약 #8310 이전의 rhwp 변환본). 본문 33문단의 저장 줄이 모두 `vertpos=0` 이다.
//! 값 `1` 의 HWP 는 HWP 규칙으로 저장 줄을 그대로 읽지만, 수정 전 HWPX 저장(devel `8a196ca7ae`)은
//! 이 계약을 싣지 않아 다시 연 HWPX 가 XML import 규칙(원점에 머문 본문 사다리 재조판)으로 줄을
//! 다시 짰다 — `--verify` 33건(`linesegs count expected=1 actual=0`), `ir-diff` 33문단 값 차이.
//! 수정: HWPX 에 `META-INF/rhwp-hwpx-origin`(값 그대로)을 싣고, 그 HWPX 는 HWP 규칙으로 줄을 읽으며,
//! HWPX → HWP 저장은 값을 `/RhwpHwpxOrigin` 으로 되돌린다.
//! - 관계형: 다시 연 HWPX 의 저장 줄·쪽별 본문이 원본 HWP 와 같고, `--verify` 와 같은 비교가 차이 0 이다.
//!   HWPX → HWP 로 되돌린 문서도 원본과 같은 줄·쪽별 본문이다.
//! - 고정형: 실리는 표지 값은 원본 스트림 값 `1` 이다. 반례로 값 `2`(#8310) HWP 와 한컴 HWPX 는 표지를 싣지 않는다.
//!
//! 측정: 2026-11-22, base `8a196ca7ae` / head `646515d993`. 이 표본의 독립 한컴 Print PDF 는 저장소에 없고
//! (`pdf/issue2439/` 는 `samples/issue2439/` 의 다른 원본용) 측정 환경(mac)에 한/글이 없어 raw 실루엣
//! 일치율은 측정하지 않았다. 이 검사는 쪽 좌표를 고정하지 않으며, 시각 근거는 같은 원본의 출력 대조로 대신한다.
//! - 원본 HWP SVG ↔ HWPX 저장 후 다시 연 SVG(값 `1` 표본 10건): base·head 모두 바이트 동일, 쪽수 동일
//!   (issue2439 1쪽). `--verify` 는 base exit 3(33건) → head exit 0.
//! - 저장소 표본 전체 SVG A/B(base ↔ head 직접 열기): 1877 파일 차이 0.
//! - 편람 HWPX Native 383쪽 `--handbook-gate` 통과.
//!
//! 증거: `output/pr-review/9039/final.log`(로컬, 커밋 안 함), PR 본문.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::document::{HWPX_ORIGIN_HWPX_MARKER_PATH, HWPX_ORIGIN_STREAM_PATH};
use rhwp::serializer::hwpx::roundtrip;

const SAMPLE: &str = "samples/issue2439_zero_offset_coanchored_float_exclusion.hwp";
/// #8310 값 `2` 의 HWP(원본 HWPX 저장 줄 계약) — 표지를 싣지 않아야 하는 반례.
const SOURCE_LINES_HWP: &str = "samples/issue8302/followup8438/followup-8333-worklife.hwp";
/// 한컴 HWPX — rhwp 출처 표식이 없어 표지를 싣지 않아야 하는 반례.
const HANCOM_HWPX: &str = "samples/hwpx/hwpx-02.hwpx";

fn load(path: &str) -> DocumentCore {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("fixture 누락 {path}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{path} 열기 실패: {e}"))
}

fn page_texts(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count() as usize)
        .map(|page| core.page_text(page).expect("쪽 글"))
        .collect()
}

type Line = (i32, i32, i32, i32, i32, i32, u32);

/// 본문 문단마다 저장 줄 — (vpos, 줄 높이, 글 높이, 기준선, 줄간격, 폭, 글 시작).
fn body_lines(core: &DocumentCore) -> Vec<Vec<Line>> {
    core.document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .map(|para| {
            para.line_segs
                .iter()
                .map(|l| {
                    (
                        l.vertical_pos,
                        l.line_height,
                        l.text_height,
                        l.baseline_distance,
                        l.line_spacing,
                        l.segment_width,
                        l.text_start,
                    )
                })
                .collect()
        })
        .collect()
}

fn reopen_as_hwpx(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwpx_native().expect("HWPX 저장"))
        .expect("HWPX 다시 열기")
}

fn reopen_as_hwp(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwp_with_adapter().expect("HWP 저장"))
        .expect("HWP 다시 열기")
}

fn origin_stream(core: &DocumentCore) -> Option<&[u8]> {
    core.document()
        .extra_streams
        .iter()
        .find(|(path, _)| path == HWPX_ORIGIN_STREAM_PATH)
        .map(|(_, value)| value.as_slice())
}

#[test]
fn hwp_line_rules_survive_hwpx_export_and_reopen() {
    let original = load(SAMPLE);
    assert_eq!(
        origin_stream(&original),
        Some(&b"1"[..]),
        "표본은 값 1 변환본이다"
    );
    let lines = body_lines(&original);
    assert_eq!(lines.len(), 33, "본문 33문단");
    assert!(
        lines.iter().flatten().all(|line| line.0 == 0),
        "표본의 본문 저장 줄은 모두 vpos 0 이다"
    );

    let hwpx = reopen_as_hwpx(&original);
    assert_eq!(
        body_lines(&hwpx),
        lines,
        "다시 연 HWPX 의 저장 줄은 원본 HWP 와 같아야 한다"
    );
    assert_eq!(page_texts(&hwpx), page_texts(&original), "쪽별 본문");

    // `export-hwpx --verify` 와 같은 비교 — 저장 줄 캐시 계약을 양쪽에 적용한다.
    let expected =
        roundtrip::hwpx_export_expected_document(&original.hwpx_export_expected_document());
    let reloaded = roundtrip::hwpx_export_expected_document(hwpx.document());
    let diff = roundtrip::strip_hwp_to_hwpx_noise(roundtrip::diff_exported_hwpx_documents(
        &expected, &reloaded,
    ));
    assert!(
        diff.is_empty(),
        "--verify 비교 차이 {}건: {:?}",
        diff.differences.len(),
        diff.differences.iter().take(3).collect::<Vec<_>>()
    );
    assert_eq!(
        hwpx.document().hwpx_aux_entry(HWPX_ORIGIN_HWPX_MARKER_PATH),
        Some(&b"1"[..]),
        "HWPX 에 원본 출처 값 1 을 실어야 한다"
    );
}

#[test]
fn hwpx_carrying_hwp_line_rules_returns_the_origin_value_to_hwp() {
    let original = load(SAMPLE);
    let roundtrip = reopen_as_hwp(&reopen_as_hwpx(&original));
    assert_eq!(
        origin_stream(&roundtrip),
        Some(&b"1"[..]),
        "HWPX → HWP 는 원래의 /RhwpHwpxOrigin 값 1 을 되돌려야 한다"
    );
    assert!(roundtrip.document().provenance.hwpx_lineage);
    assert_eq!(body_lines(&roundtrip), body_lines(&original), "저장 줄");
    assert_eq!(page_texts(&roundtrip), page_texts(&original), "쪽별 본문");
}

#[test]
fn source_line_contracts_do_not_carry_the_hwp_line_rule_marker() {
    let source_lines = load(SOURCE_LINES_HWP);
    assert_eq!(origin_stream(&source_lines), Some(&b"2"[..]));
    assert_eq!(
        reopen_as_hwpx(&source_lines)
            .document()
            .hwpx_aux_entry(HWPX_ORIGIN_HWPX_MARKER_PATH),
        None,
        "값 2 HWP 는 XML 규칙으로 읽으므로 표지를 싣지 않는다"
    );

    let hancom = load(HANCOM_HWPX);
    assert_eq!(
        reopen_as_hwpx(&hancom)
            .document()
            .hwpx_aux_entry(HWPX_ORIGIN_HWPX_MARKER_PATH),
        None,
        "한컴 HWPX 는 표지를 싣지 않는다"
    );
    let hwp = reopen_as_hwp(&hancom);
    assert_eq!(
        origin_stream(&hwp),
        Some(&b"2"[..]),
        "한컴 HWPX → HWP 는 종전대로 값 2"
    );
}
