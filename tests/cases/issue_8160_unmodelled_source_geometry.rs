//! #8160: sample16 전용 회귀. 이전 등록: tests/cases/issue_4961_font_decision_trace.rs
use rhwp::document_core::DocumentCore;

#[test]
fn issue_8160_issue4967_combined_evidence_keeps_unowned_legacy_geometry_unmodelled() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/hwp3-sample16.hwp"))
        .expect("public HWP3 fixture parses");
    let evidence: serde_json::Value = serde_json::from_str(
        &core
            .get_font_layout_evidence_native(0, r#"{"maxCharacters":4096}"#)
            .expect("same-snapshot HWP3 evidence"),
    )
    .expect("HWP3 evidence JSON");
    let lines = evidence["lines"].as_array().expect("line evidence");
    assert!(!lines.is_empty());
    for line in lines {
        assert_eq!(line["storedRow"]["disposition"], "unmodelled");
        let width = line["frame"]["width"].as_f64().expect("실제 줄 상자 폭");
        // 유효한 실제 상자도 HWP3의 저장 프레임 소유를 입증하지 못한다.
        // 폭이 없는 줄은 프레임 출처 판정에 앞서 실제 상자 자체가 무효다.
        let reason = if width.is_finite() && width > 0.0 {
            "frameProvenanceIncomplete"
        } else {
            "invalidActualFrame"
        };
        assert_eq!(line["storedRow"]["reason"], reason, "{line}");
    }
}
