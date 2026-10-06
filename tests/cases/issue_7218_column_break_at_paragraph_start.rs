//! CLI/MCP 문단 앞 단 나눔 속성과 Studio 사용자 분할 명령을 분리해서 검증한다.
//!
//! # 기대값의 출처
//!
//! HWPX `hp:p/@columnBreak` 와 HWP5 문단 헤더 break 비트(0x08)는 쪽 나눔(0x04)과 같은
//! **break-before** 속성이다(`parser/hwpx/section.rs` 스펙 표 59 주석:
//! `bit 0 구역 · bit 1 다단 · bit 2 쪽 · bit 3 단`).
//!
//! 저장소 정본 HWPX 85개 전수 실측: `columnBreak="1"` 최상위 문단 132개 중
//! **108개(82%)가 글자를 가진 내용 문단**이다. 그 앞에 빈 문단이 오는 경우도 흔하지만
//! 그 빈 문단은 `columnBreak="0"` 인 보통 빈 줄이고, 단 나눔은 **내용 문단**이 갖는다.
//!
//! # 이 시험이 잠그는 것
//!
//! 1. offset 0 에서 문단 수가 변하지 않고, 대상 문단이 텍스트·문단모양을 유지한 채
//!    `ColumnBreakType::Column` 을 갖는다.
//! 2. 다른 축의 break 비트를 지우지 않는다 — 쪽 나눔(0x04)이 이미 있으면 둘이 함께 남는다.
//! 3. 반복 호출해도 문단이 누적되지 않는다(멱등).
//! 4. 문단 중간 오프셋의 분할 동작은 그대로다.

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

/// offset 0 은 문단을 가르지 않고 그 문단에 단 나눔만 건다.
#[test]
fn a_column_break_at_paragraph_start_does_not_split_the_paragraph() {
    let mut core = core();
    let before = paragraph_texts(&core);
    let before_shape = core.document().sections[0].paragraphs[HEADING_PARA].para_shape_id;

    core.mark_column_break_at_paragraph_start_native(0, HEADING_PARA)
        .expect("단 나눔 삽입");

    assert_eq!(
        paragraph_texts(&core),
        before,
        "문단 수·텍스트가 변하면 안 된다 — 수정 전에는 개요 서식을 물려받은 빈 문단이 \
         {HEADING_PARA}번에 생겨 문단이 하나 늘었다",
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
fn a_column_break_at_paragraph_start_keeps_the_other_break_axes() {
    let mut core = core();
    // #7241 이후 사용자 쪽 나눔 명령은 문단을 분할한다. 이 검사는 기존 문단의
    // 두 속성 축 보존을 검사하므로 CLI/MCP의 명시적 속성 setter로 준비한다.
    core.mark_page_break_at_paragraph_start_native(0, HEADING_PARA)
        .expect("문단 앞 쪽 나눔 속성 설정");
    core.mark_column_break_at_paragraph_start_native(0, HEADING_PARA)
        .expect("단 나눔 삽입");

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
        "두 명령을 이어 써도 문단이 늘면 안 된다",
    );
}

/// 반복 호출해도 문단이 누적되지 않는다.
#[test]
fn repeating_the_column_break_at_paragraph_start_is_idempotent() {
    let mut core = core();
    let before = paragraph_texts(&core);

    for _ in 0..3 {
        core.mark_column_break_at_paragraph_start_native(0, HEADING_PARA)
            .expect("단 나눔 삽입");
    }

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

/// 문단 중간 오프셋은 종전처럼 분할한다.
#[test]
fn a_column_break_inside_a_paragraph_still_splits_it() {
    let mut core = core();
    let before = paragraph_texts(&core);

    core.insert_column_break_native(0, HEADING_PARA, 3)
        .expect("단 나눔 삽입");

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
        ColumnBreakType::Column,
        "단 나눔은 뒤 조각에 붙는다",
    );
    assert_eq!(
        core.document().sections[0].paragraphs[HEADING_PARA].column_type,
        ColumnBreakType::None,
        "앞 조각은 단 나눔을 갖지 않는다",
    );
}

/// 0x04/0x08는 직교 속성이다. HWPX의 두 속성과 HWP 헤더 모두 보존한다.
#[test]
fn page_and_column_flags_survive_both_formats_and_operation_orders() {
    for page_first in [true, false] {
        let mut doc = core();
        if page_first {
            doc.mark_page_break_at_paragraph_start_native(0, HEADING_PARA)
                .unwrap();
        }
        doc.mark_column_break_at_paragraph_start_native(0, HEADING_PARA)
            .unwrap();
        if !page_first {
            doc.mark_page_break_at_paragraph_start_native(0, HEADING_PARA)
                .unwrap();
        }
        for bytes in [
            doc.export_hwpx_native().unwrap(),
            doc.export_hwp_with_adapter().unwrap(),
        ] {
            let reopened = DocumentCore::from_bytes(&bytes).unwrap();
            let para = &reopened.document().sections[0].paragraphs[HEADING_PARA];
            assert_eq!(para.raw_break_type & 0x0c, 0x0c, "page_first={page_first}");
            assert_eq!(para.column_type, ColumnBreakType::Page);
        }
    }
}

/// Studio Ctrl+Shift+Enter는 시작에서도 문단을 분리하고 뒤 조각으로 이동한다.
#[test]
fn user_column_break_at_section_start_splits_and_moves_the_cursor() {
    let mut doc = core();
    let before = paragraph_texts(&doc);
    let result: serde_json::Value =
        serde_json::from_str(&doc.insert_column_break_native(0, 0, 0).unwrap()).unwrap();
    let after = paragraph_texts(&doc);
    assert_eq!(after.len(), before.len() + 1);
    assert!(after[0].is_empty());
    assert_eq!(&after[1..], before.as_slice());
    assert_eq!(result["paraIdx"], 1);
    assert_eq!(result["charOffset"], 0);
    for bytes in [
        doc.export_hwpx_native().unwrap(),
        doc.export_hwp_with_adapter().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(paragraph_texts(&reopened), after);
        assert_eq!(
            reopened.document().sections[0].paragraphs[1].raw_break_type & 8,
            8
        );
    }
}

#[test]
fn column_property_does_not_persist_a_synthesized_page_boundary() {
    let mut doc = core();
    let para = &mut doc.document_mut().sections[0].paragraphs[HEADING_PARA];
    para.column_type = ColumnBreakType::Page;
    para.raw_break_type = 0; // HWP3 자연 경계는 enum에만 있으며 raw 명시 비트가 없다.
    para.page_break_synthesized = true;
    assert!(doc
        .mark_column_break_at_paragraph_start_native(0, HEADING_PARA)
        .unwrap());
    assert!(!doc
        .mark_column_break_at_paragraph_start_native(0, HEADING_PARA)
        .unwrap());
    for bytes in [
        doc.export_hwpx_native().unwrap(),
        doc.export_hwp_with_adapter().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            reopened.document().sections[0].paragraphs[HEADING_PARA].raw_break_type & 0x0c,
            0x08
        );
    }
}
