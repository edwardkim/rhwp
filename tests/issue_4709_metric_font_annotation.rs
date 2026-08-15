//! [#4709] renderPageSvg 메트릭 face 주석 계약.
//!
//! 옵트인(`setAnnotateMetricFont(true)`) 시에만 각 `<text>` 에 `data-metric-font`,
//! 루트 `<svg>` 에 `data-rhwp-metric-fonts`(쉼표 목록)가 붙는다. 기본값(꺼짐)의
//! 출력은 종전과 바이트 단위로 같아야 한다 — 골든/스냅샷 불변 계약.

fn load(path: &str) -> rhwp::wasm_api::HwpDocument {
    let full = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path);
    let bytes = std::fs::read(&full).expect("fixture 읽기");
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("fixture 파싱")
}

#[test]
fn annotation_is_optin_and_lists_metric_faces() {
    let mut doc = load("samples/byeolpyo1.hwp");

    let off = doc.render_page_svg_native(0).expect("기본 렌더");
    assert!(
        !off.contains("data-metric-font"),
        "기본값(꺼짐)에서는 메트릭 주석이 없어야 한다 — 골든 출력 불변"
    );

    doc.set_annotate_metric_font(true);
    let on = doc.render_page_svg_native(0).expect("주석 렌더");
    assert!(
        on.contains("data-metric-font=\""),
        "옵트인 시 <text> 에 data-metric-font 가 붙어야 한다"
    );
    assert!(
        on.contains("data-rhwp-metric-fonts=\""),
        "옵트인 시 루트 <svg> 에 data-rhwp-metric-fonts 목록이 붙어야 한다"
    );

    // 주석 제거 시 기본 출력과 동일해야 한다 (주석은 뷰 전용, 레이아웃 불변).
    doc.set_annotate_metric_font(false);
    let off_again = doc.render_page_svg_native(0).expect("재렌더");
    assert_eq!(
        off, off_again,
        "플래그를 끄면 종전 출력으로 되돌아와야 한다"
    );
}
