//! 셀·중첩 셀·글상자·머리말 차트 문서를 HWP와 HWPX로 다시 저장해도 쪽 배치가 멈추지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn ole_count(core: &DocumentCore) -> usize {
    (0..core.page_count())
        .map(|page| {
            let layout: serde_json::Value =
                serde_json::from_str(&core.get_page_control_layout_native(page).unwrap()).unwrap();
            layout["controls"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|control| control["type"] == "ole")
                .count()
        })
        .sum()
}

#[test]
fn chart_hosts_lay_out_after_hwp_then_hwpx_resave() {
    let source = DocumentCore::from_bytes(include_bytes!(
        "../fixtures/container_charts/container-charts.hwpx"
    ))
    .unwrap();
    let hwp = DocumentCore::from_bytes(&source.export_hwp_native().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&hwp.export_hwpx_native().unwrap()).unwrap();
    assert!(ole_count(&source) > 0);
    // 두 저장본 모두 모든 쪽의 개체 배치를 끝까지 계산한다(이전에는 HWPX 재저장본이 멈췄다).
    ole_count(&hwp);
    ole_count(&hwpx);
}

#[test]
fn legacy_resaved_chart_host_keeps_text_before_its_composed_table_line() {
    // 이 저장본은 표의 선언 높이가 셀 내용의 재조판 높이보다 작다.
    // 높이 대조로 표 줄을 추측하면 표 앞의 본문을 그리지 못한다.
    let core = DocumentCore::from_bytes(include_bytes!(
        "../fixtures/container_charts/legacy-resaved.hwpx"
    ))
    .unwrap();
    assert_eq!(core.page_count(), 1);
    let layout: serde_json::Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    let text: String = layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|run| run["text"].as_str())
        .collect();
    for marker in [
        "앞 본문 🦦",
        "이웃 보존-0 한글🦦",
        "이웃 보존-1 한글🦦",
        "HDR",
        "TBX",
        "뒤 본문 🦦 ABC 999",
    ] {
        assert!(text.contains(marker), "missing {marker:?}: {text}");
    }
}

fn verify_anchor_rows(case: &str) {
    use std::path::Path;

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/fixtures/container_charts/inline-anchor");
    let input = if case == "source" {
        root.join("tests/fixtures/container_charts/container-charts.hwpx")
    } else {
        fixtures.join(format!("{case}.hwpx"))
    };
    // rhwp 출력에서 기대값을 만들지 않는다. 같은 공개 입력을 한/글 2020에서
    // 저장한 원본 LineSeg가 독립 Print의 본문 줄 원점을 기록한다.
    let oracle = rhwp::parser::parse_document(
        &std::fs::read(fixtures.join(format!("{case}-hancom.hwp"))).unwrap(),
    )
    .unwrap();
    let rows = &oracle.sections[0].paragraphs;
    let expected = f64::from(rows[1].line_segs[0].vertical_pos - rows[0].line_segs[0].vertical_pos)
        * 96.0
        / 7200.0;
    let source = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    let before = serde_json::to_value(&source.document().sections[0].paragraphs).unwrap();
    let hwp = DocumentCore::from_bytes(&source.export_hwp_native().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&hwp.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(&source.document().sections[0].paragraphs).unwrap(),
        before,
        "저장으로 원본의 줄/제어 슬롯을 변경하지 않는다"
    );
    for (label, core) in [("source", source), ("hwp", hwp), ("hwpx", hwpx)] {
        assert_eq!(core.page_count(), 1, "{case}/{label}: 독립 Print 쪽수");
        let layout: serde_json::Value =
            serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
        let runs = layout["runs"].as_array().unwrap();
        let body_y = |paragraph: u64| {
            runs.iter()
                .find(|run| {
                    run["paraIdx"].as_u64() == Some(paragraph)
                        && run.get("cellPath").is_none()
                        && run["charStart"].as_u64() == Some(0)
                })
                .and_then(|run| run["y"].as_f64())
                .expect("표 앞뒤 본문 첫 글자")
        };
        let actual = body_y(1) - body_y(0);
        assert!(
            (actual - expected).abs() <= 1.0,
            "{case}/{label}: 본문 원점 차 {actual}px, 독립 한컴 저장 줄 {expected}px"
        );
    }
}

#[test]
fn overflowing_table_keeps_following_header_anchor_row() {
    verify_anchor_rows("source");
}

#[test]
fn removing_horizontal_margin_keeps_anchor_on_table_row() {
    verify_anchor_rows("zero-outer-margin");
}

#[test]
fn narrower_table_keeps_anchor_on_table_row() {
    verify_anchor_rows("narrow-table");
}

#[test]
fn overflowing_terminal_table_without_anchor_has_no_extra_row() {
    verify_anchor_rows("no-header");
}

#[test]
fn overflowing_table_keeps_following_footer_anchor_row() {
    verify_anchor_rows("footer-anchor");
}
