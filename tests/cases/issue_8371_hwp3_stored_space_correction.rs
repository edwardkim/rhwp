//! 관계형: 한글 3.0 저장 줄의 공백 보정과 편집 무효화 계약.
//! 주석 갱신: 2026-10-30T17:28:27+09:00; 신규 파일로 직전 파일 커밋 없음.
//! 2026-10-30 중간 측정 raw(96 DPI, PDFium, 2px 이웃): 28쪽 Native 99.06% / fresh WASM 99.02%,
//! 193쪽 Native 98.29% / fresh WASM 99.99%; 전 763쪽 최저 Native 98.29% / WASM 99.02%.
//! 근거: mydocs/pr/assets/issue_8371/sample10-native.tsv, mydocs/pr/assets/issue_8371/sample10-wasm.tsv,
//! mydocs/working/task_m100_8371_stage1.md. 최종 head 재측정 필요.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::DocumentCore;
use serde_json::Value;

fn space_advance(trace: &Value, paragraph: u64, offset: u64) -> i64 {
    trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            entry["source"]["paragraphIndex"] == paragraph
                && entry["source"]["charOffset"] == offset
                && entry["source"]["character"] == " "
        })
        .expect("원본 공백과 출력 글줄의 소유 관계")["layoutMetric"]["finalAdvanceHwpunit"]
        .as_i64()
        .unwrap()
}

#[test]
fn saved_hwp3_space_corrections_follow_their_rows_and_expire_on_edit() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/hwp3-sample10.hwp"))
        .expect("한글 3.0 원본");
    let paragraphs = &core.document().sections[0].paragraphs;
    let zero = &paragraphs[1194];
    assert_eq!(zero.hwp3_saved_space_corrections, [(0, 0), (73, 0)]);
    let corrected = &paragraphs[7414];
    assert_eq!(
        corrected.hwp3_saved_space_corrections,
        [(0, 4), (68, 13), (123, 14), (176, 0)]
    );

    let trace28: Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(27, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        space_advance(&trace28, 1194, 2),
        space_advance(&trace28, 1194, 75),
        "저장 보정이 0인 첫 줄과 마지막 줄의 같은 공백 전진은 같다"
    );
    let trace193: Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(192, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    assert!(
        space_advance(&trace193, 7414, 11) > space_advance(&trace193, 7414, 177),
        "양수 저장 보정이 있는 줄의 내부 공백은 보정 없는 줄보다 넓다"
    );
    let mut edited = zero.clone();
    edited.invalidate_layout_inputs();
    assert!(
        edited.hwp3_saved_space_corrections.is_empty(),
        "편집 후 이전 줄 보정은 무효다"
    );
}
