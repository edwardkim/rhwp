//! 한 문단에 빈 누름틀이 둘 이상일 때, 값을 채워 저장한 뒤 되읽어도 필드 범위가 유지되는가.
//!
//! ## 무엇이 깨졌었나
//!
//! `samples/issue-986-receipt.hwp` 의 "진료기간" 칸은 한 표 셀 문단(`'부터 까지'`)에 빈
//! ClickHere 누름틀 두 개(`med_str_dt`·`med_end_dt`)를 담는다. 앞 필드에 값을 넣고 저장하면
//! 두 FIELD_BEGIN 이 **모두 위치 0 에 붙어** 방출됐다. 파서는 스택(LIFO)으로 짝지으므로
//! (`parser/body_text.rs`) 되읽을 때 범위가 뒤엉켜, 옆 필드가 남의 값을 갖고 자기 값은 뒤
//! 텍스트까지 삼켰다.
//!
//! ```text
//! 한글2022(정답)  med_str_dt='2026-08-07'      med_end_dt=''
//! 수정 전 rhwp    med_str_dt='2026-08-07부터 '  med_end_dt='2026-08-07'
//! ```
//!
//! ## 이 테스트가 지키는 것
//!
//! 1. **값의 귀속** — 채운 필드만 값을 갖는다.
//! 2. **필드 개수 보존** — 조기 방출을 막기만 하고 시작 위치에서 강제 방출하지 않으면
//!    그 FIELD_BEGIN 이 갈 자리를 못 찾아 필드가 통째로 사라진다(실제로 165→164 로 줄었다).
//!    개수를 함께 보지 않으면 이 실패가 "값이 정확해졌다"로 위장된다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const FIXTURE: &str = "samples/issue-986-receipt.hwp";
const FILLED_FIELD: &str = "med_str_dt";
const SIBLING_FIELD: &str = "med_end_dt";
const VALUE: &str = "2026-08-07";

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = fs::read(&path).expect("fixture read");
    DocumentCore::from_bytes(&bytes).expect("fixture parse")
}

/// 이름 → 값 목록. 같은 이름이 여러 번 나오는 서식이 있으므로 목록으로 모은다.
fn field_values(core: &DocumentCore, name: &str) -> Vec<String> {
    core.collect_all_fields()
        .into_iter()
        .filter(|f| f.field.field_name().map(|n| n == name).unwrap_or(false))
        .map(|f| f.value)
        .collect()
}

#[test]
fn sibling_field_in_same_paragraph_keeps_its_range_after_roundtrip() {
    let mut core = load();
    let before_count = core.collect_all_fields().len();
    assert!(
        before_count > 0,
        "fixture 에 필드가 없다 — 표본이 바뀌었는지 확인하라"
    );

    core.set_field_value_by_name(FILLED_FIELD, VALUE)
        .expect("필드 값 설정");

    // 저장 → 재적재. 결함은 메모리가 아니라 이 경계에서만 드러난다.
    let saved = core.export_hwp_native().expect("HWP5 저장");
    let reloaded = DocumentCore::from_bytes(&saved).expect("저장본 재적재");

    assert_eq!(
        field_values(&reloaded, FILLED_FIELD),
        vec![VALUE.to_string()],
        "채운 필드가 자기 값만 갖지 않는다 — 범위가 뒤 텍스트까지 넘어갔다"
    );
    assert_eq!(
        field_values(&reloaded, SIBLING_FIELD),
        vec![String::new()],
        "같은 문단의 옆 필드가 남의 값을 가졌다 — FIELD_BEGIN 이 겹쳐 방출됐다"
    );
    assert_eq!(
        reloaded.collect_all_fields().len(),
        before_count,
        "왕복 후 필드 개수가 달라졌다 — FIELD_BEGIN 이 방출되지 못해 필드가 사라졌다"
    );
}
