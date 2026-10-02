//! [#7288] `setTableProperties({pageBreak})` 가 HWP5 저장으로 나가지 않는다.
//!
//! # 무엇이 깨져 있나
//!
//! `set_table_properties_native` 은 `table.page_break`(와 `repeat_header`)만 바꾸는데,
//! HWP5 저장기 `serialize_table_record` 는 **`raw_table_record_attr` 가 0 이 아니면 그 값을
//! 그대로 쓴다**(원본 보존 계약). 그래서 설정은 메모리에만 남고 저장본에는 옛 비트가 나간다.
//!
//! ```text
//!   setTableProperties({pageBreak: 1})  →  getTableProperties()  1   (메모리 OK)
//!                                        →  exportHwp() → 재파싱   2   ← 되돌아간다
//! ```
//!
//! 소비자 입장에서는 «쪽 경계에서» 를 파일에 심을 방법이 없다. 같은 레코드의 bit 2 인
//! `repeatHeader` 도 같은 이유로 새어 나가지 않는다.
//!
//! # 기대값의 출처
//!
//! HWP5 `HWPTAG_TABLE` 레코드 첫 UINT32 의 bit 0~1 이 «쪽 경계에서»(0 나누지 않음 ·
//! 1 셀 단위로 나눔 · 2 나눔), bit 2 가 제목 줄 자동 반복이다. 파서가 같은 비트를 읽으므로
//! (`parser/control.rs` 의 `attr & 0x03`), 저장 → 재파싱 왕복은 넣은 값을 그대로 돌려줘야 한다.
//!
//! 표본은 한컴이 저작한 `samples/issue2439_zero_offset_coanchored_float_exclusion.hwp` 로,
//! 첫 표의 raw TABLE attr 이 `0x0400_0002` 다 — 상위 비트를 달고 있어 **보존 계약**까지
//! 같은 문서에서 잠글 수 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::{Table, TablePageBreak};

const SAMPLE: &str = "samples/issue2439_zero_offset_coanchored_float_exclusion.hwp";
/// 표본 첫 표의 원본 raw TABLE attr — 상위 비트 0x0400_0000 + «나눔»(2).
const SAMPLE_RAW_ATTR: u32 = 0x0400_0002;

fn load() -> DocumentCore {
    let bytes = std::fs::read(SAMPLE).expect("표본을 읽지 못했습니다");
    DocumentCore::from_bytes(&bytes).expect("표본 파싱")
}

fn first_table(doc: &DocumentCore) -> &Table {
    doc.document()
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .expect("표를 찾지 못했습니다")
}

/// 문서에서 첫 표의 (구역, 문단, 컨트롤) 좌표.
fn first_table_path(doc: &DocumentCore) -> (usize, usize, usize) {
    for (si, section) in doc.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, control) in para.controls.iter().enumerate() {
                if matches!(control, Control::Table(_)) {
                    return (si, pi, ci);
                }
            }
        }
    }
    panic!("표를 찾지 못했습니다");
}

fn save_and_reload(doc: &DocumentCore) -> DocumentCore {
    let bytes = doc.export_hwp_native().expect("HWP5 저장");
    DocumentCore::from_bytes(&bytes).expect("저장본 재파싱")
}

/// 표본 전제 — 이 값이 바뀌면 아래 두 검사의 뜻이 달라진다.
#[test]
fn sample_starts_from_the_expected_raw_attr() {
    let doc = load();
    let table = first_table(&doc);
    assert_eq!(
        table.raw_table_record_attr, SAMPLE_RAW_ATTR,
        "표본의 raw TABLE attr 전제가 바뀌었습니다"
    );
    assert_eq!(table.page_break, TablePageBreak::RowBreak);
    assert!(!table.repeat_header);
}

/// 세 값 모두 저장 → 재파싱 왕복에서 살아남아야 한다.
#[test]
fn page_break_survives_hwp5_save() {
    for (value, want) in [
        (0u8, TablePageBreak::None),
        (1, TablePageBreak::CellBreak),
        (2, TablePageBreak::RowBreak),
    ] {
        let mut doc = load();
        let (sec, para, ctrl) = first_table_path(&doc);
        doc.set_table_properties_native(sec, para, ctrl, &format!("{{\"pageBreak\":{value}}}"))
            .expect("표 속성 설정");
        assert_eq!(
            first_table(&doc).page_break,
            want,
            "메모리에서부터 어긋납니다 (pageBreak={value})"
        );

        let reloaded = save_and_reload(&doc);
        assert_eq!(
            first_table(&reloaded).page_break,
            want,
            "저장 → 재파싱 뒤 «쪽 경계에서» 가 되돌아갔습니다 (pageBreak={value}, \
             raw attr=0x{:08x})",
            first_table(&reloaded).raw_table_record_attr
        );
    }
}

/// 같은 레코드 bit 2 인 제목 줄 반복도 함께 나가야 한다.
#[test]
fn repeat_header_survives_hwp5_save() {
    for want in [true, false] {
        let mut doc = load();
        let (sec, para, ctrl) = first_table_path(&doc);
        doc.set_table_properties_native(sec, para, ctrl, &format!("{{\"repeatHeader\":{want}}}"))
            .expect("표 속성 설정");

        let reloaded = save_and_reload(&doc);
        assert_eq!(
            first_table(&reloaded).repeat_header,
            want,
            "저장 → 재파싱 뒤 제목 줄 반복이 되돌아갔습니다 (want={want}, raw attr=0x{:08x})",
            first_table(&reloaded).raw_table_record_attr
        );
    }
}

/// **정의되지 않은 상위 비트는 건드리지 않는다.**
///
/// `raw_table_record_attr` 은 원본 보존 계약이라 bit 0~2 밖은 그대로 나가야 한다.
#[test]
fn setting_page_break_keeps_the_other_raw_bits() {
    let mut doc = load();
    let (sec, para, ctrl) = first_table_path(&doc);
    doc.set_table_properties_native(sec, para, ctrl, "{\"pageBreak\":1}")
        .expect("표 속성 설정");

    let reloaded = save_and_reload(&doc);
    let raw = first_table(&reloaded).raw_table_record_attr;
    assert_eq!(
        raw & !0x07,
        SAMPLE_RAW_ATTR & !0x07,
        "bit 0~2 밖의 원본 비트가 사라졌습니다 (0x{SAMPLE_RAW_ATTR:08x} -> 0x{raw:08x})"
    );
    assert_eq!(
        first_table(&reloaded).page_break,
        TablePageBreak::CellBreak,
        "«쪽 경계에서» 가 저장본에 안 실렸습니다 (raw attr=0x{raw:08x})"
    );
}

/// 뜻은 CellBreak 이지만 원본이 보존한 비표준 raw 값(3)은, 같은 뜻으로 다시 설정해도
/// 정규값 1로 바꾸면 안 된다. serializer는 raw 의미와 IR 의미가 달라질 때만 bit 0~1을
/// 동기화하므로 setter가 raw 비트를 미리 덮으면 이 계약을 깨뜨린다.
#[test]
fn setting_an_equivalent_page_break_keeps_nonstandard_raw_value() {
    let mut doc = load();
    let (sec, para, ctrl) = first_table_path(&doc);
    let Control::Table(table) =
        &mut doc.document_mut().sections[sec].paragraphs[para].controls[ctrl]
    else {
        panic!("첫 컨트롤이 표가 아니다");
    };
    table.raw_table_record_attr = (table.raw_table_record_attr & !0x03) | 0x03;
    table.page_break = TablePageBreak::CellBreak;

    doc.set_table_properties_native(sec, para, ctrl, "{\"pageBreak\":1}")
        .expect("동일한 표 속성 설정");
    let reloaded = save_and_reload(&doc);
    assert_eq!(
        first_table(&reloaded).raw_table_record_attr & 0x03,
        0x03,
        "뜻이 같은 pageBreak 설정이 원본 raw 값을 정규화했다"
    );
    assert_eq!(first_table(&reloaded).page_break, TablePageBreak::CellBreak);
}

/// **다른 속성만 바꿀 때는 이 비트를 건드리지 않는다.**
///
/// `pageBreak`·`repeatHeader` 키가 없으면 raw attr 은 원본 그대로여야 한다.
#[test]
fn unrelated_property_change_leaves_the_raw_attr_alone() {
    let mut doc = load();
    let (sec, para, ctrl) = first_table_path(&doc);
    doc.set_table_properties_native(sec, para, ctrl, "{\"cellSpacing\":42}")
        .expect("표 속성 설정");

    assert_eq!(
        first_table(&doc).raw_table_record_attr,
        SAMPLE_RAW_ATTR,
        "관계없는 속성 변경이 raw TABLE attr 을 건드렸습니다"
    );
}

/// **캡션을 켜도 «쪽 경계에서» 가 살아남아야 한다.**
///
/// 같은 뿌리의 두 번째 자리 — 캡션 갈래가 `raw_table_record_attr` 에 **개체 공통 속성**
/// (`table.attr` = `common.attr`)을 통째로 대입했다. 두 값은 비트 배치가 다른 별개
/// 레코드라, 캡션을 켜는 것만으로 표의 «쪽 경계에서»·제목 줄 반복이 엉뚱한 값으로 저장된다.
#[test]
fn adding_a_caption_does_not_rewrite_the_table_record_attr() {
    let mut doc = load();
    let (sec, para, ctrl) = first_table_path(&doc);
    doc.set_table_properties_native(sec, para, ctrl, "{\"hasCaption\":true}")
        .expect("캡션 설정");

    let raw = first_table(&doc).raw_table_record_attr;
    assert_eq!(
        raw & 0x07,
        SAMPLE_RAW_ATTR & 0x07,
        "캡션을 켜자 TABLE 레코드의 «쪽 경계에서»·제목 줄 반복 비트가 바뀌었습니다 \
         (0x{SAMPLE_RAW_ATTR:08x} -> 0x{raw:08x})"
    );

    let reloaded = save_and_reload(&doc);
    assert_eq!(
        first_table(&reloaded).page_break,
        TablePageBreak::RowBreak,
        "캡션을 켠 저장본에서 «쪽 경계에서» 가 달라졌습니다"
    );
}
