#![cfg(not(target_arch = "wasm32"))]

//! [#2792] 셀 안의 표(중첩 표) 텍스트가 검색·치환에서 통째로 누락되던 결함의 회귀 시험.
//!
//! 종전 `search_all` 은 본문 문단 → 표 셀·글상자까지 **한 단계만** 내려갔다. 셀 문단의
//! 컨트롤은 수식만 훑었으므로, 셀 안에 또 표가 있으면 그 텍스트는 검색 결과에 존재하지
//! 않았다. 그 결과 `replaceAll` 은 아무것도 바꾸지 못하고도 `{"ok":true,"count":0}` 을
//! 돌려줬다 — "문서 전체에서 치환한다"는 문서화된 계약의 위반이자, 호출자가 실패로
//! 판별할 수 없는 무음 결함이다(중첩 표를 쓰는 공문 서식에서 실제로 관측됐다).
//!
//! 표본은 중첩 표 픽스처(`issue1949_giant_cell_nested_tables_perf.hwpx`)를 재사용한다.
//! - `NESTED_ONLY` 는 **깊이 2**(셀 안의 표) 에만 있고 문서 전체에 한 번 나온다.
//! - `OUTER_ONLY` 는 **깊이 1**(본문 직속 표의 셀) 에만 있고 역시 한 번 나온다.
//!
//! 두 상수가 결과 JSON 하위호환의 대조군이다: 깊이 1 은 종전 `cellContext` 를 그대로 내고,
//! 깊이 2 는 그 표현에 담기지 않으므로 `cellPath` 로 싣는다.

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue1949_giant_cell_nested_tables_perf.hwpx";

/// 깊이 2(셀 안의 표)에만 있는 유일 문자열.
const NESTED_ONLY: &str = "스테인리스강 - 청동";

/// 깊이 1(본문 직속 표의 셀)에만 있는 유일 문자열.
const OUTER_ONLY: &str = "수면비행선박 안전시설요건";

const REPLACEMENT: &str = "치환확인";

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("표본 로드");
    DocumentCore::from_bytes(&bytes).expect("표본 파싱")
}

fn finds(core: &DocumentCore, query: &str) -> bool {
    core.search_all_text_native(query, true, true)
        .expect("검색 성공")
        != "[]"
}

/// 이 결함의 본체. 중첩 표 텍스트를 실제로 찾아 바꾸고, count 가 그 사실을 보고한다.
#[test]
fn replace_all_reaches_text_inside_a_table_nested_in_a_cell() {
    let mut core = load();
    assert!(
        finds(&core, NESTED_ONLY),
        "표본 전제: 중첩 표 텍스트가 검색돼야 한다"
    );

    let json = core
        .replace_all_native(NESTED_ONLY, REPLACEMENT, true)
        .expect("치환 성공");

    assert_eq!(
        json, r#"{"ok":true,"count":1}"#,
        "중첩 표를 못 보면 count 0 이 성공으로 보고된다"
    );
    assert!(
        !finds(&core, NESTED_ONLY),
        "원래 문자열이 남아 있으면 안 된다"
    );
    assert!(finds(&core, REPLACEMENT), "치환된 문자열이 있어야 한다");
}

/// 치환 결과가 직렬화까지 살아남는다. 중첩 셀은 평면 좌표 reflow 가 닿지 않아
/// path 판(`reflow_cell_paragraph_by_path`)에 연결되지 않으면 조판·raw 무효화가
/// 어긋난 채로 저장돼 결과가 유실된다(#1385 와 같은 계열의 사고).
#[test]
fn nested_replacement_survives_hwpx_export() {
    let mut core = load();
    core.replace_all_native(NESTED_ONLY, REPLACEMENT, true)
        .expect("치환 성공");

    let exported = core.export_hwpx_native().expect("export 성공");
    let reparsed = DocumentCore::from_bytes(&exported).expect("재파싱 성공");

    assert!(
        finds(&reparsed, REPLACEMENT),
        "치환이 저장 바이트에 반영되어야 한다"
    );
    assert!(
        !finds(&reparsed, NESTED_ONLY),
        "원래 문자열이 저장 바이트에 남아 있으면 안 된다"
    );
}

/// 깊이 2 는 평면 `cellContext` 로 안쪽/바깥쪽을 구분할 수 없으므로 `cellPath` 로 싣는다.
/// 배열 모양은 `parse_cell_path` 의 입력과 같아 by_path API 에 그대로 되먹일 수 있다.
#[test]
fn nested_cell_hits_are_addressed_by_path() {
    let core = load();

    let json = core
        .search_all_text_native(NESTED_ONLY, true, true)
        .expect("검색 성공");

    assert!(json.contains(r#""cellPath":[{"controlIndex":"#), "{json}");
    assert!(
        !json.contains("cellContext"),
        "깊이 2 를 평면 좌표로 실으면 안쪽/바깥쪽이 구분되지 않는다: {json}"
    );
}

/// 깊이 1 결과 JSON 은 종전 그대로다 — 중첩이 없는 문서에서 기존 소비자
/// (studio find-dialog 의 이동·단건 치환)가 무회귀여야 한다.
#[test]
fn outer_cell_hits_keep_the_flat_envelope() {
    let core = load();

    let json = core
        .search_all_text_native(OUTER_ONLY, true, true)
        .expect("검색 성공");

    assert!(json.contains(r#""cellContext":{"parentPara":"#), "{json}");
    assert!(!json.contains("cellPath"), "{json}");
}

/// Find/F3 는 중첩 히트를 아직 받으면 안 된다. 호출자(studio find-dialog)는
/// `cellContext` 가 없으면 본문 좌표 분기로 떨어져 표가 놓인 **바깥 문단**을 고친다
/// — #3865 가 경고한 오손이다. 이동·단건 치환이 path 를 받게 되면 이 게이트만 풀면 된다.
/// 전체 치환은 이 필터를 타지 않으므로 위 시험대로 중첩까지 이미 고친다.
#[test]
fn find_next_leaves_nested_cell_hits_out_until_navigation_takes_paths() {
    let core = load();

    let nested = core
        .search_text_native(NESTED_ONLY, 0, 0, 0, true, true, true)
        .expect("검색 성공");
    assert_eq!(nested, r#"{"found":false}"#, "{nested}");

    // 대조군: 깊이 1 은 종전대로 찾아진다(#3865 의 opt-in).
    let outer = core
        .search_text_native(OUTER_ONLY, 0, 0, 0, true, true, true)
        .expect("검색 성공");
    assert!(outer.contains(r#""found":true"#), "{outer}");
}
