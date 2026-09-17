//! [Issue #7218] `insert-page-break` 를 문단 시작(offset 0)에 쓰면 빈 문단이 생긴다.
//!
//! `insert_page_break_native` 가 `char_offset` 과 관계없이 항상 `split_at(char_offset)`
//! 으로 문단을 갈랐다. `char_offset == 0` 이면 앞쪽이 원 문단의 ParaShape·스타일·개요
//! 수준을 그대로 물려받은 **빈 문단**으로 남는다. 개요 제목 앞에 쓰면 한/글이 그 빈
//! 문단에도 개요 번호를 매겨 항목 하나가 비어 보이고 뒤 번호가 밀린다. 문단 수도 늘어
//! 이후 문단 좌표로 편집하면 한 칸씩 어긋났다.
//!
//! # 기대값의 출처
//!
//! HWPX `hp:p/@pageBreak` 와 HWP5 문단 헤더 break 비트(0x04)는 *그 문단 앞에서* 쪽을
//! 넘기라는 **break-before** 속성이다(`parser/hwpx/section.rs` 스펙 표 59 주석). 따라서
//! "문단 P 앞에 쪽 나눔" 의 결과는 P 자신이 그 속성을 갖는 것이고 새 문단은 필요 없다.
//!
//! 저장소 정본 HWPX 85개 전수 실측도 같은 말을 한다 — `pageBreak="1"` 문단 712개 중
//! **664개(93%)가 글자를 가진 내용 문단**이다. 한/글은 이 속성을 빈 문단에 따로 붙이지
//! 않고 내용 문단에 붙인다.
//!
//! # 이 시험이 잠그는 것
//!
//! 1. offset 0 에서 문단 수가 변하지 않고, 대상 문단이 텍스트·문단모양을 그대로 유지한 채
//!    `ColumnBreakType::Page` 를 갖는다.
//! 2. 다른 축의 break 비트(구역 0x01·다단 0x02)를 지우지 않는다 — bitwise 합성.
//! 3. 반복 호출해도 문단이 누적되지 않는다(멱등).
//! 4. **문단 중간 오프셋의 분할 동작은 그대로다** — 이 수정이 좁혀야 할 자리는 offset 0 뿐이다.

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

/// offset 0 은 문단을 가르지 않고 그 문단에 break-before 속성만 준다.
#[test]
fn a_break_at_paragraph_start_does_not_split_the_paragraph() {
    let mut core = core();
    let before = paragraph_texts(&core);
    let before_shape = core.document().sections[0].paragraphs[HEADING_PARA].para_shape_id;

    core.insert_page_break_native(0, HEADING_PARA, 0)
        .expect("쪽 나눔 삽입");

    let after = paragraph_texts(&core);
    assert_eq!(
        after, before,
        "문단 수·텍스트가 변하면 안 된다 — 수정 전에는 개요 서식을 물려받은 빈 문단이 \
         {HEADING_PARA}번에 생겨 문단이 하나 늘었다",
    );

    let para = &core.document().sections[0].paragraphs[HEADING_PARA];
    assert_eq!(
        para.column_type,
        ColumnBreakType::Page,
        "대상 문단이 쪽 나눔을 가져야 한다",
    );
    assert_eq!(
        para.raw_break_type & 0x04,
        0x04,
        "HWP5 문단 헤더의 쪽 나눔 비트가 켜져야 한다",
    );
    assert_eq!(
        para.para_shape_id, before_shape,
        "대상 문단의 문단모양(개요 수준)은 그대로여야 한다",
    );
    assert!(
        !para.page_break_synthesized,
        "사용자가 명시한 쪽 나눔은 합성 표시를 남기면 안 된다 — 남으면 HWP5 저장기가 \
         이 바이트를 버린다",
    );
}

/// 다른 축의 break 비트를 지우지 않는다 — 구역 시작 문단에 적용해도 0x01 이 남는다.
#[test]
fn a_break_at_paragraph_start_keeps_the_other_break_axes() {
    let mut core = core();
    // 구역 시작 문단의 저장 계약을 재현한다(구역 나누기 비트 0x01).
    core.document_mut().sections[0].paragraphs[0].raw_break_type = 0x01;

    core.insert_page_break_native(0, 0, 0)
        .expect("쪽 나눔 삽입");

    let raw = core.document().sections[0].paragraphs[0].raw_break_type;
    assert_eq!(
        raw & 0x01,
        0x01,
        "구역 나누기 비트가 사라졌다(raw=0x{raw:02X}) — 덮어쓰기 대신 bitwise 합성이어야 한다",
    );
    assert_eq!(raw & 0x04, 0x04, "쪽 나눔 비트도 함께 켜져야 한다");
}

/// 반복 호출해도 문단이 누적되지 않는다.
#[test]
fn repeating_the_break_at_paragraph_start_is_idempotent() {
    let mut core = core();
    let before = paragraph_texts(&core);

    for _ in 0..3 {
        core.insert_page_break_native(0, HEADING_PARA, 0)
            .expect("쪽 나눔 삽입");
    }

    assert_eq!(
        paragraph_texts(&core),
        before,
        "반복 호출이 빈 문단을 누적하면 안 된다",
    );
    assert_eq!(
        core.document().sections[0].paragraphs[HEADING_PARA].column_type,
        ColumnBreakType::Page,
    );
}

/// 문단 중간 오프셋은 종전처럼 분할한다 — 좁힌 자리는 offset 0 뿐이다.
#[test]
fn a_break_inside_a_paragraph_still_splits_it() {
    let mut core = core();
    let before = paragraph_texts(&core);

    core.insert_page_break_native(0, HEADING_PARA, 3)
        .expect("쪽 나눔 삽입");

    let after = paragraph_texts(&core);
    assert_eq!(
        after.len(),
        before.len() + 1,
        "중간 오프셋은 문단을 하나 늘려야 한다",
    );
    assert_eq!(
        after[HEADING_PARA], "Sec",
        "앞 조각은 오프셋 앞 글자를 갖는다"
    );
    assert_eq!(after[HEADING_PARA + 1], "ond", "뒤 조각이 나머지를 갖는다");
    assert_eq!(
        core.document().sections[0].paragraphs[HEADING_PARA + 1].column_type,
        ColumnBreakType::Page,
        "쪽 나눔은 뒤 조각에 붙는다",
    );
    assert_eq!(
        core.document().sections[0].paragraphs[HEADING_PARA].column_type,
        ColumnBreakType::None,
        "앞 조각은 쪽 나눔을 갖지 않는다",
    );
}
