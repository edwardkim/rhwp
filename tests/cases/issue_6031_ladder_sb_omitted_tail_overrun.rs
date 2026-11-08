//! #6031 자산관리규정의 말미 문단은 독립 PDF와 같은 쪽에 속해야 한다.
//! 동일 원본과 한컴 Print PDF의 Native/fresh WASM 전60쪽 최저90.25013%다.
//! 증적: asset6031-stored-cell-frame-fresh-wasm-validation.json.
//! 본문 바닥 좌표나 첫줄 픽셀 밴드를 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue6031/3249937_asset_management_rules.hwpx";

fn page_text(core: &DocumentCore, page: u32) -> String {
    let raw = core.get_page_text_layout_native(page - 1).expect("쪽 글줄");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("글줄 JSON");
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
fn issue_6031_page_tail_lines_stay_inside_body_bottom() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("정식 원본")).expect("자산관리규정");
    assert_eq!(core.page_count(), 60, "독립 한컴 Print PDF는60쪽이다");

    // 정본3쪽은 '마. … 채권은 연체가…'로 끝나며 '바.' 문단은4쪽 첫 문단이다.
    let next_paragraph = "바.매립지등관리자는기금예산에해당연도";
    let p3 = page_text(&core, 3);
    let p4 = page_text(&core, 4);
    assert!(
        !p3.contains(next_paragraph),
        "다음 쪽 문단을 말미에 붙들면 안 된다"
    );
    assert!(
        p4.starts_with(next_paragraph),
        "4쪽은 정본의 '바.' 문단으로 시작한다"
    );
    assert_eq!(
        p4.matches(next_paragraph).count(),
        1,
        "문단을 중복하지 않는다"
    );

    // 정본6쪽의 '나. 신고의무자는…' 다음 두 항목은7쪽에 속한다.
    let p6 = page_text(&core, 6);
    let p7 = page_text(&core, 7);
    let first = "1)취득세과세대상통보:매립지등관리자";
    let second = "2)부동산거래신고:매매당사자공동";
    assert!(
        !p6.contains(first) && !p6.contains(second),
        "다음 쪽 항목의 역류"
    );
    assert!(p7.starts_with(first), "7쪽 첫 항목 누락");
    assert_eq!(p7.matches(first).count(), 1);
    assert_eq!(p7.matches(second).count(), 1);
    assert!(
        p7.find(first).unwrap() < p7.find(second).unwrap(),
        "항목 순서"
    );

    // 각 쪽의 실제 본문 영역을 사용한다. 특정 용지의 고정 바닥 좌표는 없다.
    let options = AnomalyOptions::default();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("실제 배치");
        let anomalies = scan_page(page, &tree.root, core.page_count(), &options);
        let escaped: Vec<_> = anomalies
            .overflow
            .iter()
            .filter(|item| item.over_bottom > options.overflow_tolerance_px)
            .collect();
        assert!(
            escaped.is_empty(),
            "{}쪽 본문 밖에 남은 요소: {escaped:?}",
            page + 1
        );
    }
}
