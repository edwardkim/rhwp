//! [Issue #5019] 문단 시작의 단 나눔 — 쪽 나눔(`#7218`)과 같은 계약으로 맞춘다.
//!
//! `edit insert-column-break --offset 0` 이 문단을 갈라, 앞쪽에 원 문단 모양(개요 수준
//! 포함)을 물려받은 **빈 문단**을 남겼다. 개요 제목 앞이면 한글이 그 빈 문단에도 번호를
//! 붙여 항목이 비어 보이고 뒤 번호가 밀린다.
//!
//! # 두 연산을 구분한다 — `#7218` 이 세운 구조를 그대로 따른다
//!
//! * **속성 setter** `mark_column_break_at_paragraph_start_native` — 문단을 보존하고
//!   break-before 저장 비트만 갱신한다. 선언적 편집(CLI·MCP)이 쓴다.
//! * **사용자 명령** `insert_column_break_native` — 커서 위치에서 문단을 가르는 한/글 키
//!   입력(Ctrl+Shift+Enter) 계약이다. 저장 속성의 의미만으로 이 명령을 바꾸지 않는다.
//!   쪽 나눔 쪽에서 COM 11.0.0.9136 실측으로 확인된 구분이며(첫 문단 시작의 `BreakPage`
//!   는 빈 선행 문단과 새 쪽을 만들었다), 단 나눔도 같은 계층 구분을 따른다.
//!
//! # 기대값의 출처
//!
//! HWPX `hp:p/@columnBreak` 와 HWP5 문단 헤더 비트 `0x08` 은 쪽 나눔 `0x04` 와 같은
//! **break-before** 축이다(`parser/hwpx/section.rs` 스펙 표 59 주석:
//! `bit 0 구역 · bit 1 다단 · bit 2 쪽 · bit 3 단`).
//!
//! 저장소 정본 HWPX 85개 전수 실측: `columnBreak="1"` 최상위 문단 132개 중
//! **108개(82%)가 글자를 가진 내용 문단**이다. 그 앞에 빈 문단이 오는 경우도 흔하지만
//! 그 빈 문단은 `columnBreak="0"` 인 보통 빈 줄이고, 단 나눔은 **내용 문단**이 갖는다.
//!
//! # 이 시험이 잠그는 것
//!
//! 1. setter 가 문단을 보존하고 대상 문단에 `Column`·`0x08` 을 건다.
//! 2. 다른 축의 break 비트를 지우지 않는다 — 쪽 나눔(`0x04`)이 있으면 둘이 함께 남는다.
//! 3. setter 는 멱등이다(두 번째 호출은 `false`).
//! 4. **사용자 명령은 종전 분할 계약을 유지한다** — 문단 시작에서도 가른다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::ColumnBreakType;
use rhwp::scaffold::{build_scaffold, ScaffoldSpec};

const SPEC: &str = r#"{"version":"1","title":"repro","blocks":[
 {"type":"heading","level":1,"text":"First"},
 {"type":"paragraph","text":"body 1"},
 {"type":"heading","level":1,"text":"Second"},
 {"type":"paragraph","text":"body 2"}
]}"#;

/// 제목 문단 + 4블록 = 5문단. 문단 3 이 두 번째 개요 제목 `Second` 다.
const HEADING_PARA: usize = 3;

fn core() -> DocumentCore {
    let spec: ScaffoldSpec = serde_json::from_str(SPEC).expect("scaffold spec");
    let bytes = rhwp::serializer::serialize_hwpx(&build_scaffold(&spec)).expect("HWPX 직렬화");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn paragraph_texts(core: &DocumentCore) -> Vec<String> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.clone())
        .collect()
}

/// setter 는 문단을 가르지 않고 그 문단에 break-before 속성만 준다.
#[test]
fn the_setter_does_not_split_the_paragraph() {
    let mut core = core();
    let before = paragraph_texts(&core);
    let before_shape = core.document().sections[0].paragraphs[HEADING_PARA].para_shape_id;

    assert!(
        core.mark_column_break_at_paragraph_start_native(0, HEADING_PARA)
            .expect("단 나눔 속성"),
        "처음 걸 때는 true 를 돌려준다",
    );

    assert_eq!(
        paragraph_texts(&core),
        before,
        "문단 수·텍스트가 변하면 안 된다 — 종전 CLI 는 개요 서식을 물려받은 빈 문단을 \
         {HEADING_PARA}번에 만들어 문단이 하나 늘었다",
    );

    let para = &core.document().sections[0].paragraphs[HEADING_PARA];
    assert_eq!(
        para.column_type,
        ColumnBreakType::Column,
        "대상 문단이 단 나눔을 가져야 한다",
    );
    assert_eq!(
        para.raw_break_type & 0x08,
        0x08,
        "HWP5 문단 헤더의 단 나눔 비트가 켜져야 한다",
    );
    assert_eq!(
        para.para_shape_id, before_shape,
        "대상 문단의 문단모양(개요 수준)은 그대로여야 한다",
    );
}

/// 이미 쪽 나눔이 있는 문단에 단 나눔을 더해도 두 비트가 함께 남는다.
#[test]
fn the_setter_keeps_the_other_break_axes() {
    let mut core = core();
    core.mark_page_break_at_paragraph_start_native(0, HEADING_PARA)
        .expect("쪽 나눔 속성");
    core.mark_column_break_at_paragraph_start_native(0, HEADING_PARA)
        .expect("단 나눔 속성");

    let raw = core.document().sections[0].paragraphs[HEADING_PARA].raw_break_type;
    assert_eq!(
        raw & 0x04,
        0x04,
        "쪽 나눔 비트가 사라졌다(raw=0x{raw:02X}) — 덮어쓰기 대신 bitwise 합성이어야 한다",
    );
    assert_eq!(raw & 0x08, 0x08, "단 나눔 비트도 함께 켜져야 한다");
    assert_eq!(
        paragraph_texts(&core).len(),
        5,
        "두 속성을 이어 걸어도 문단이 늘면 안 된다",
    );
}

/// setter 는 멱등이다 — 두 번째 호출은 아무것도 하지 않는다.
#[test]
fn the_setter_is_idempotent() {
    let mut core = core();
    let before = paragraph_texts(&core);

    assert!(core
        .mark_column_break_at_paragraph_start_native(0, HEADING_PARA)
        .expect("단 나눔 속성"));
    assert!(
        !core
            .mark_column_break_at_paragraph_start_native(0, HEADING_PARA)
            .expect("단 나눔 속성"),
        "같은 속성이 이미 있으면 false 여야 한다",
    );

    assert_eq!(
        paragraph_texts(&core),
        before,
        "반복 호출이 빈 문단을 누적하면 안 된다",
    );
    assert_eq!(
        core.document().sections[0].paragraphs[HEADING_PARA].column_type,
        ColumnBreakType::Column,
    );
}

/// 사용자 명령은 종전 분할 계약을 유지한다 — 문단 시작에서도 가른다.
///
/// 저장 속성의 의미로 이 명령을 바꾸지 않는다. 쪽 나눔 쪽 `core_contract` 와 같은 계약이다.
#[test]
fn the_user_command_still_splits_at_paragraph_start() {
    let mut core = core();
    let before = paragraph_texts(&core);

    core.insert_column_break_native(0, HEADING_PARA, 0)
        .expect("단 나눔 명령");

    let after = paragraph_texts(&core);
    assert_eq!(
        after.len(),
        before.len() + 1,
        "사용자 명령은 문단 시작에서도 문단을 가른다",
    );
    assert_eq!(
        core.document().sections[0].paragraphs[HEADING_PARA + 1].column_type,
        ColumnBreakType::Column,
        "단 나눔은 뒤 조각에 붙는다",
    );
}

/// 문단 중간 오프셋은 종전처럼 분할한다.
#[test]
fn the_user_command_splits_inside_a_paragraph() {
    let mut core = core();
    let before = paragraph_texts(&core);

    core.insert_column_break_native(0, HEADING_PARA, 3)
        .expect("단 나눔 명령");

    let after = paragraph_texts(&core);
    assert_eq!(after.len(), before.len() + 1);
    assert_eq!(
        after[HEADING_PARA], "Sec",
        "앞 조각은 오프셋 앞 글자를 갖는다"
    );
    assert_eq!(after[HEADING_PARA + 1], "ond", "뒤 조각이 나머지를 갖는다");
    assert_eq!(
        core.document().sections[0].paragraphs[HEADING_PARA + 1].column_type,
        ColumnBreakType::Column,
    );
    assert_eq!(
        core.document().sections[0].paragraphs[HEADING_PARA].column_type,
        ColumnBreakType::None,
        "앞 조각은 단 나눔을 갖지 않는다",
    );
}
