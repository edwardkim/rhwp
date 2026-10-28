//! Issue #7606 — 글자처럼 취급(TAC)으로 만든 표에 `setTableProperties` 를 부르면 HWP5 저장에서
//! 표 개체 공통 속성의 너비·높이 기준(attr bit 15–19)이 용지(Paper, 0)로 바뀐다.
//!
//! `set_table_properties_native` 는 끝에서 `table.attr` 전체를 `raw_ctrl_data` FLAGS 에 덮어썼다.
//! `create_table_ex_native` 의 TAC 경로는 FLAGS 에 개체 공통 속성 값(크기 기준 절대값 포함)을 넣지만
//! `Table.attr` 는 `0x04000006` 이라, 넘긴 키와 무관하게 크기 기준이 0 이 되고 `{}` 만 넘겨도 글자처럼
//! 취급·배치까지 초기화됐다. 파싱한 표는 `table.attr` 가 FLAGS 와 같아(`parse_table`) 드러나지 않았다.
//! 한/글 2024·한컴독스는 용지 기준 표를 여백을 포함한 용지 폭으로 늘려 표가 쪽 밖으로 나간다.
//!
//! 가드하는 축:
//!   ① TAC 표에 빈 속성(`{}`)을 넘겨도 저장된 개체 공통 속성이 넘기기 전과 같다
//!   ② 위치를 바꾼 TAC 표도 너비·높이 기준이 절대값으로 저장되고, 넘긴 위치는 반영된다
//!   ③ TAC 가 아닌 `create_table_native` 표도 빈 속성으로 저장된 개체 공통 속성이 바뀌지 않고,
//!      너비·높이 기준은 절대값이다
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::shape::{HorzRelTo, SizeCriterion, TextWrap, VertRelTo};
use rhwp::model::table::Table;
use rhwp::parser::parse_document;
use rhwp::wasm_api::HwpDocument;

const TAC_TABLE: &str =
    r#"{"sectionIdx":0,"paraIdx":0,"charOffset":0,"rowCount":2,"colCount":2,"treatAsChar":true}"#;
const FLOATING_POSITION: &str =
    r#"{"treatAsChar":false,"textWrap":"TopAndBottom","vertRelTo":"Para","horzRelTo":"Para"}"#;

/// 첫 표의 (문단, 컨트롤) 번호와 표.
fn first_table(doc: &Document) -> (u32, u32, &Table) {
    for (p, para) in doc.sections[0].paragraphs.iter().enumerate() {
        for (c, ctrl) in para.controls.iter().enumerate() {
            if let Control::Table(t) = ctrl {
                return (p as u32, c as u32, t);
            }
        }
    }
    panic!("표가 없다");
}

/// `edits` 를 차례로 `set_table_properties` 에 넘긴 뒤 HWP5 로 저장·재파싱한 첫 표.
fn edit_and_roundtrip(mut doc: HwpDocument, edits: &[&str]) -> Table {
    let (para, ctrl, _) = first_table(doc.document());
    for json in edits {
        doc.set_table_properties(0, para, ctrl, json)
            .unwrap_or_else(|e| panic!("set_table_properties({json}): {e:?}"));
    }
    let out = doc
        .export_hwp()
        .unwrap_or_else(|e| panic!("HWP5 저장: {e:?}"));
    let reparsed = parse_document(&out).unwrap_or_else(|e| panic!("HWP5 재파싱: {e:?}"));
    first_table(&reparsed).2.clone()
}

/// 빈 문서에 `createTableEx({treatAsChar:true})` 로 표를 만든 문서.
fn tac_table_doc() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document()
        .unwrap_or_else(|e| panic!("빈 문서: {e:?}"));
    doc.create_table_ex(TAC_TABLE)
        .unwrap_or_else(|e| panic!("createTableEx: {e:?}"));
    doc
}

fn size_criteria(t: &Table) -> (SizeCriterion, SizeCriterion) {
    (t.common.width_criterion, t.common.height_criterion)
}

const ABSOLUTE: (SizeCriterion, SizeCriterion) = (SizeCriterion::Absolute, SizeCriterion::Absolute);

#[test]
fn tac_table_empty_props_keep_common_attr_on_hwp5_save() {
    let untouched = edit_and_roundtrip(tac_table_doc(), &[]);
    assert_eq!(
        size_criteria(&untouched),
        ABSOLUTE,
        "전제 붕괴: 손대지 않은 TAC 표가 절대값으로 저장되지 않는다 (attr {:#010x})",
        untouched.common.attr,
    );

    let edited = edit_and_roundtrip(tac_table_doc(), &["{}"]);
    assert_eq!(
        edited.common.attr, untouched.common.attr,
        "빈 setTableProperties 가 저장된 개체 공통 속성을 바꿨다 ({:#010x} → {:#010x})",
        untouched.common.attr, edited.common.attr,
    );
}

#[test]
fn tac_table_position_edit_keeps_absolute_size_criteria_on_hwp5_save() {
    let t = edit_and_roundtrip(tac_table_doc(), &[FLOATING_POSITION]);
    assert_eq!(
        size_criteria(&t),
        ABSOLUTE,
        "위치를 바꾼 TAC 표의 너비·높이 기준이 절대값이 아니다 (attr {:#010x})",
        t.common.attr,
    );
    assert!(
        !t.common.treat_as_char,
        "넘긴 treatAsChar:false 가 저장되지 않았다"
    );
    assert_eq!(t.common.text_wrap, TextWrap::TopAndBottom);
    assert_eq!(t.common.vert_rel_to, VertRelTo::Para);
    assert_eq!(t.common.horz_rel_to, HorzRelTo::Para);
}

/// 빈 문서에 `create_table_native` 로 (TAC 가 아닌) 표를 만든 문서.
fn native_table_doc() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_table_native(0, 0, 0, 2, 2)
        .unwrap_or_else(|e| panic!("create_table_native: {e:?}"));
    doc
}

#[test]
fn native_table_empty_props_keep_common_attr_on_hwp5_save() {
    let untouched = edit_and_roundtrip(native_table_doc(), &[]);
    let edited = edit_and_roundtrip(native_table_doc(), &["{}"]);
    assert_eq!(
        size_criteria(&edited),
        ABSOLUTE,
        "TAC 가 아닌 표의 너비·높이 기준이 절대값이 아니다 (attr {:#010x})",
        edited.common.attr,
    );
    assert_eq!(
        edited.common.attr, untouched.common.attr,
        "빈 setTableProperties 가 저장된 개체 공통 속성을 바꿨다 ({:#010x} → {:#010x})",
        untouched.common.attr, edited.common.attr,
    );
}
