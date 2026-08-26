//! #5749 — 쪽 정보가 문서 쪽번호(`쪽 > 새 번호로 시작` 반영)를 내보내는지 고정한다.
//!
//! 상태 표시줄이 보여야 할 숫자는 물리 순번(pageIndex + 1)이 아니라 문서가 매기는 쪽번호다.
//! 둘은 NewNumber 컨트롤이 있을 때 갈라진다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/쪽기준.hwp")
        .to_string_lossy()
        .into_owned()
}

/// JSON 문자열에서 정수 필드 하나를 읽는다(테스트용 최소 파서).
fn json_int(json: &str, key: &str) -> i64 {
    let needle = format!("\"{}\":", key);
    let start = json
        .find(&needle)
        .unwrap_or_else(|| panic!("{key} 필드가 없다: {json}"))
        + needle.len();
    let end = json[start..]
        .find([',', '}'])
        .map(|idx| start + idx)
        .unwrap_or(json.len());
    json[start..end]
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("{key} 파싱 실패({e}): {json}"))
}

#[test]
fn page_info_reports_document_page_number() {
    let data = std::fs::read(sample()).expect("샘플을 읽을 수 있어야 한다");
    let mut core = DocumentCore::from_bytes(&data).expect("샘플을 열 수 있어야 한다");

    let before = core
        .get_page_info_native(0)
        .expect("쪽 정보를 조회할 수 있어야 한다");
    assert_eq!(json_int(&before, "pageIndex"), 0);
    assert_eq!(
        json_int(&before, "pageNumber"),
        1,
        "새 번호가 없으면 문서 쪽번호는 물리 순번과 같아야 한다: {before}"
    );

    // 쪽 > 새 번호로 시작 = 7
    core.insert_new_number_native(0, 0, 0, 7)
        .expect("새 번호 지정을 삽입할 수 있어야 한다");

    let after = core
        .get_page_info_native(0)
        .expect("쪽 정보를 조회할 수 있어야 한다");
    assert_eq!(
        json_int(&after, "pageIndex"),
        0,
        "물리 순번은 그대로여야 한다: {after}"
    );
    assert_eq!(
        json_int(&after, "pageNumber"),
        7,
        "문서 쪽번호는 새 번호를 따라야 한다: {after}"
    );
}
