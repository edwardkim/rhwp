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
