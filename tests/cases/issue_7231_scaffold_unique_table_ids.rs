//! [Issue #7231] scaffold 가 만든 모든 표의 개체 id 가 `0` 으로 중복된다.
//!
//! `scaffold/builder.rs` 가 instance_id 를 행·열 수와 전체 폭·높이로 만든 해시로 계산해
//! `raw_ctrl_data` 에만 기록하고 `table.common.instance_id` 는 기본값 `0` 으로 남겼다.
//! HWPX 저장기(`serializer/hwpx/table.rs`)는 `<hp:tbl id>` 를 `common.instance_id` 에서
//! 만들므로 문서 안 모든 표가 `id="0"` 이 됐다. 부수적으로 **크기가 같은 표끼리는 raw 쪽
//! id 도 같았다**.
//!
//! # 기대값의 출처 — 저장소 자신의 identity 계약
//!
//! `model/identity.rs` 의 머리 주석이 이 함정을 이미 적어 뒀다 —
//! *"Existing objects … stay untouched. Walk the owned IR; **dimensions, clock time and
//! wrapping hashes cannot establish uniqueness**."* 그 모듈의 `Allocator` 는 문서에서 쓰인
//! id 를 전부 모은 뒤 사용되지 않은 양수를 내주며, 편집 경로(`serializer/form_identity.rs`)가
//! 이미 그것을 쓴다. scaffold 와 HTML 붙여넣기만 자기 해시를 굴리고 있었다.
//!
//! 같은 모듈은 **IR 과 raw 두 namespace 를 함께 예약**한다 — *"Reserve both table IR and
//! retained HWP raw IDs when they disagree: either may be observed by the corresponding
//! serializer."* 그래서 두 곳에 같은 값을 넣어야 한다.
//!
//! # 이 시험이 잠그는 것
//!
//! 1. scaffold 표의 `common.instance_id` 가 서로 다르고 `0` 이 아니다.
//! 2. `raw_ctrl_data` 의 INSTANCE_ID 가 `common.instance_id` 와 같다.
//! 3. **크기가 같은 두 표도 서로 다른 id 를 받는다** — 해시 방식의 반례.
//! 4. HWPX 저장 → 재파싱 왕복에서 id 가 보존된다(저장기가 실제로 그 값을 쓴다).
//! 5. HTML 붙여넣기 경로도 한 번의 호출에서 만든 표들이 서로 다른 id 를 받는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::scaffold::{build_scaffold, ScaffoldSpec};

/// 표 4개 — 1번과 4번은 **치수가 완전히 같다**(해시 충돌 반례).
const SPEC: &str = r#"{"version":"1","title":"id 재현","blocks":[
  {"type":"table","rows":[["a","b"],["c","d"]]},
  {"type":"table","rows":[["x","y","z"]]},
  {"type":"table","rows":[["1"]]},
  {"type":"table","rows":[["a","b"],["c","d"]]}
]}"#;

/// 본문 최상위 표의 `(common.instance_id, raw_ctrl_data 의 INSTANCE_ID)`.
fn table_ids(document: &Document) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for section in &document.sections {
        for para in &section.paragraphs {
            for control in &para.controls {
                if let Control::Table(table) = control {
                    // CommonObjAttr 의 instance_id 는 offset 32..36 이다
                    // (`model/shape.rs` 의 `common_obj_offsets` — crate 내부 모듈이라
                    // 통합 시험에서는 같은 오프셋을 직접 읽는다).
                    let raw = table
                        .raw_ctrl_data
                        .get(32..36)
                        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
                        .unwrap_or(0);
                    out.push((table.common.instance_id, raw));
                }
            }
        }
    }
    out
}

fn scaffold_document() -> Document {
    let spec: ScaffoldSpec = serde_json::from_str(SPEC).expect("scaffold spec");
    build_scaffold(&spec)
}

/// 표마다 고유한 개체 id 를 받고, IR 과 raw 가 같은 값이다.
#[test]
fn scaffold_tables_get_unique_nonzero_object_ids() {
    let ids = table_ids(&scaffold_document());
    assert_eq!(ids.len(), 4, "표 4개가 나와야 한다: {ids:?}");

    for (ir, raw) in &ids {
        assert_ne!(
            *ir, 0,
            "common.instance_id 가 0 이면 HWPX `<hp:tbl id>` 가 \"0\" 이 된다: {ids:?}",
        );
        assert_eq!(
            ir, raw,
            "IR 과 raw 가 어긋나면 같은 표가 형식마다 다른 id 로 저장된다: {ids:?}",
        );
    }

    let mut unique: Vec<u32> = ids.iter().map(|(ir, _)| *ir).collect();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), ids.len(), "개체 id 가 중복됐다: {ids:?}");
}

/// 치수가 같은 두 표도 서로 다른 id 를 받는다 — 해시 방식의 반례.
#[test]
fn tables_with_identical_dimensions_still_differ() {
    let ids = table_ids(&scaffold_document());
    // 0번과 3번은 같은 2×2, 같은 폭·높이다.
    assert_ne!(
        ids[0].0, ids[3].0,
        "치수가 같은 두 표가 같은 id 를 받았다 — 크기 기반 해시로는 고유성을 세울 수 없다: {ids:?}",
    );
    assert_ne!(ids[0].1, ids[3].1, "raw 쪽도 달라야 한다: {ids:?}");
}

/// HWPX 저장 → 재파싱 왕복에서 id 가 보존된다.
#[test]
fn the_ids_survive_the_hwpx_roundtrip() {
    let bytes = rhwp::serializer::serialize_hwpx(&scaffold_document()).expect("HWPX 직렬화");
    let reopened = DocumentCore::from_bytes(&bytes).expect("재파싱");
    let ids = table_ids(reopened.document());
    assert_eq!(ids.len(), 4, "왕복 후 표 4개: {ids:?}");

    let mut unique: Vec<u32> = ids.iter().map(|(ir, _)| *ir).collect();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        ids.len(),
        "왕복 후 id 가 중복됐다 — 저장기가 common.instance_id 를 쓰지 않은 것이다: {ids:?}",
    );
    assert!(
        ids.iter().all(|(ir, _)| *ir != 0),
        "왕복 후 id 가 0 이다: {ids:?}",
    );
}

/// HTML 붙여넣기 경로도 한 번의 호출에서 만든 표들이 서로 다른 id 를 받는다.
///
/// `parse_table_html` 은 HTML 하나에 표가 여러 개면 반복 호출되고, 만든 표는 아직
/// 문서에 들어가지 않은 문단 목록에 쌓인다 — 문서만 훑으면 같은 id 가 나온다.
#[test]
fn html_import_tables_get_unique_object_ids() {
    let html = concat!(
        "<table><tr><td>a</td><td>b</td></tr><tr><td>c</td><td>d</td></tr></table>",
        "<table><tr><td>a</td><td>b</td></tr><tr><td>c</td><td>d</td></tr></table>",
    );
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("빈 문서");
    core.paste_html_native(0, 0, 0, html)
        .expect("HTML 붙여넣기");

    let ids = table_ids(core.document());
    assert_eq!(ids.len(), 2, "표 2개가 나와야 한다: {ids:?}");
    for (ir, raw) in &ids {
        assert_ne!(*ir, 0, "common.instance_id 가 0 이다: {ids:?}");
        assert_eq!(ir, raw, "IR 과 raw 가 어긋났다: {ids:?}");
    }
    assert_ne!(
        ids[0].0, ids[1].0,
        "치수가 같은 두 표가 같은 id 를 받았다: {ids:?}",
    );
}
