//! #6409 앞쪽의 남은 공간이 다음 쪽 신고서를 소유하지 않는다.
//! 같은 원본/독립 Print PDF의 Native/fresh WASM 전60쪽 최저90.25013%다.
//! 표 시작 좌표 대신 정본41·42쪽의 서식 소속과 내용 보존을 확인한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::DocumentCore;

fn page_text(core: &DocumentCore, page: u32) -> String {
    let value: serde_json::Value =
        serde_json::from_str(&core.get_page_text_layout_native(page - 1).expect("쪽 글줄"))
            .expect("글줄 JSON");
    value["runs"]
        .as_array()
        .expect("실제 출력 글줄")
        .iter()
        .filter_map(|run| run["text"].as_str())
        .flat_map(str::chars)
        .filter(|ch| !ch.is_whitespace())
        .collect()
}

#[test]
fn issue_6409_form_table_starts_on_next_page_not_leftover() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6031/3249937_asset_management_rules.hwpx");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("정식 원본")).expect("자산관리규정");
    assert_eq!(core.page_count(), 60, "독립 Print PDF 쪽수");
    let p41 = page_text(&core, 41);
    let p42 = page_text(&core, 42);
    for marker in ["붙임4", "행정기관명", "취득세과세물건의매각통보(신고)"] {
        assert!(p41.contains(marker), "41쪽 서식 내용 누락: {marker}");
    }
    let report = "부동산거래계약신고서";
    assert!(
        !p41.contains(report),
        "다음 쪽 신고서가 남은 공간으로 역류했다"
    );
    assert!(p42.contains("붙임5"), "42쪽은 붙임5 서식이다");
    assert_eq!(
        p42.matches(report).count(),
        1,
        "42쪽 신고서 제목의 누락/중복"
    );
}
