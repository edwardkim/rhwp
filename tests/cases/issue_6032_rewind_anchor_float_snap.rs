//! [Issue #6032] 직전 쪽 말미 anchor 의 자리차지 표가 다음 쪽으로 흘러넘친 뒤의
//! 빈-host 자리차지 표가 한글보다 6.1pt 아래에 그려져 하단 괘선이 다음 문단
//! 글줄을 관통한다 (`samples/issue6032/2912695_civil_petition_form.hwp` 2쪽
//! "- 신청서 작성요령 -" 1×1 표).
//!
//! 저장 사다리: pi=3(7×6 표 host)이 1쪽 말미 vpos=65212 에 남고 표만 2쪽으로
//! 흘러넘치며, pi=4(1×1 표의 빈 host)는 vpos=57794 로 **되감긴다**(쪽 경계 신호,
//! 2쪽 page-relative 좌표). 한글 정본 실측: 표 상단 666.0pt(=888.0px),
//! 다음 문단 "(용지규격…)"은 저장 vpos 64553(=969.2px) — 사다리 델타 6759HU 는
//! v_off(595)+outer_top(141)+선언높이(5882)+outer_bottom(141)과 정확히 일치한다.
//!
//! 현재 보정에서는 앞의 큰 표가 2쪽으로 이월된 뒤 호스트 줄간격 안에서 시작하는
//! 1×1 표를 겹침으로 오판해 3쪽으로 다시 이월하던 문제를 확인했다. 정상 한컴
//! 2020 PDF는 두 표·후속 용지규격 문장이 모두 2쪽에 있고 물리 상자가 겹치지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/issue6032/2912695_civil_petition_form.hwp";

#[test]
fn issue_6032_rewound_empty_anchor_table_snaps_to_saved_vpos() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        HwpDocument::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("원본 열기");

    assert_eq!(core.page_count(), 2, "한글 정본은 2쪽이다");

    let tree: Value = serde_json::from_str(&core.get_page_render_tree(1).expect("2쪽 렌더 트리"))
        .expect("렌더 트리 JSON");
    let table = |pi| {
        find_node(&tree, &|node| node["type"] == "Table" && node["pi"] == pi)
            .unwrap_or_else(|| panic!("2쪽에 원문 표 문단 {pi}이 없다"))
    };
    let first = table(3);
    let guide = table(4);
    let paper = find_node(&tree, &|node| {
        node["type"] == "TextRun"
            && node["text"]
                .as_str()
                .is_some_and(|text| text.contains("용지규격"))
    })
    .expect("2쪽에서 후속 용지규격 문장이 없다");
    let bottom = |node: &Value| {
        node["bbox"]["y"].as_f64().expect("상단") + node["bbox"]["h"].as_f64().expect("높이")
    };
    let guide_top = guide["bbox"]["y"].as_f64().expect("작성요령 표 상단");
    let paper_top = paper["bbox"]["y"].as_f64().expect("용지규격 문장 상단");
    assert!(
        bottom(first) <= guide_top,
        "큰 표와 작성요령 표가 겹치면 안 된다"
    );
    assert!(
        bottom(guide) <= paper_top,
        "작성요령 표가 뒤 문장을 덮으면 안 된다"
    );
}

fn find_node<'a>(value: &'a Value, predicate: &impl Fn(&Value) -> bool) -> Option<&'a Value> {
    if predicate(value) {
        return Some(value);
    }
    match value {
        Value::Array(items) => items.iter().find_map(|item| find_node(item, predicate)),
        Value::Object(fields) => fields.values().find_map(|item| find_node(item, predicate)),
        _ => None,
    }
}
