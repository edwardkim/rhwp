//! 묶기·풀기 뒤 활성 셀 필드는 같은 본문 소유자와 지역 필드 주소를 유지한다.
//! 저장본 전쪽 Native/fresh raw100% 및 독립 Print·대표 직접 판독을 먼저 확보했다.
//! mydocs/pr/assets/active_field_group_owner_20261021/README.md
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    document_core::DocumentCore,
    model::{control::Control, paragraph::Paragraph, shape::ShapeObject},
};
use serde_json::{json, Value};
use std::path::Path;

fn table_owner(core: &DocumentCore) -> usize {
    let owners: Vec<_> = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter_map(|(index, control)| matches!(control, Control::Table(_)).then_some(index))
        .collect();
    assert_eq!(owners.len(), 1, "원본의 표 소유자를 한 번만 보존한다");
    owners[0]
}

fn cell(core: &DocumentCore) -> &Paragraph {
    let Control::Table(table) =
        &core.document().sections[0].paragraphs[0].controls[table_owner(core)]
    else {
        unreachable!()
    };
    assert_eq!(table.cells.len(), 1, "누름틀의 원래 셀 보존");
    &table.cells[0].paragraphs[0]
}

fn field_state(core: &DocumentCore) -> Value {
    let paragraph = cell(core);
    json!({"text": paragraph.text, "ranges": paragraph.field_ranges.iter().map(|range|
        json!([range.control_idx, range.start_char_idx, range.end_char_idx])).collect::<Vec<_>>(),
        "fields": paragraph.controls.iter().filter_map(|control| match control {
            Control::Field(field) => Some(json!([field.field_id, field.field_name(), field.guide_text()])),
            _ => None,
        }).collect::<Vec<_>>()})
}

fn value(core: &DocumentCore) -> String {
    let fields: Vec<_> = core
        .collect_all_fields()
        .into_iter()
        .filter(|entry| entry.field.field_name() == Some("셀 필드"))
        .collect();
    assert_eq!(
        fields.len(),
        1,
        "이름이 같은 원문 누름틀을 누락·중복하지 않는다"
    );
    fields[0].value.clone()
}

fn check(group: bool, format: &str) {
    let action = if group { "group" } else { "ungroup" };
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "mydocs/pr/assets/active_field_group_owner_20261021/inputs/{action}-before.{format}"
    ));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    let body = core.document().sections[0].paragraphs[0].text.clone();
    let original_field = field_state(&core);
    let before = table_owner(&core);
    assert_eq!(cell(&core).field_ranges.len(), 1);
    let start = cell(&core).field_ranges[0].start_char_idx as usize;
    assert!(core.set_active_field_in_cell(0, 0, before, 0, 0, start, false));
    assert!(
        !core.set_active_field_in_cell(0, 0, before, 0, 0, start, false),
        "정상 대조: 같은 활성 주소의 설정은 상태를 바꾸지 않는다"
    );
    if group {
        let targets: Vec<_> = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .enumerate()
            .filter_map(|(index, control)| {
                matches!(control, Control::Shape(_)).then_some((0, index))
            })
            .collect();
        assert_eq!(targets.len(), 2);
        assert!(
            targets.iter().all(|(_, index)| *index < before),
            "선택 도형이 누름틀을 담은 표보다 앞에 있다"
        );
        core.group_shapes_native(0, &targets).unwrap();
    } else {
        let groups: Vec<_> = core.document().sections[0].paragraphs[0].controls.iter().enumerate()
            .filter_map(|(index, control)| matches!(control, Control::Shape(shape) if matches!(shape.as_ref(), ShapeObject::Group(_))).then_some(index)).collect();
        assert_eq!(groups.len(), 1);
        assert!(groups[0] < before);
        core.ungroup_shape_native(0, 0, groups[0]).unwrap();
    }
    let after = table_owner(&core);
    assert!(
        if group {
            after < before
        } else {
            after > before
        },
        "컨트롤 수 변화로 표의 본문 소유 번호가 실제 이동했다"
    );
    assert_eq!(
        core.document().sections[0].paragraphs[0].text,
        body,
        "본문 내용 순서·보존"
    );
    assert_eq!(
        field_state(&core),
        original_field,
        "셀 안의 지역 필드 번호·범위·내용은 움직이지 않는다"
    );
    assert!(
        !core.set_active_field_in_cell(0, 0, after, 0, 0, start, false),
        "원래 활성 셀 필드의 본문 소유 번호만 같은 편집 결과로 이동해야 한다: {action}/{format}"
    );
    core.insert_text_in_cell_native(0, 0, after, 0, 0, start, "입력")
        .unwrap();
    assert_eq!(value(&core), "입력", "같은 활성 누름틀 안에 입력한다");
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_eq!(
            value(&DocumentCore::from_bytes(&bytes).unwrap()),
            "입력",
            "두 저장 형식에 셀 누름틀 내용 보존"
        );
    }
}

#[test]
fn group_before_active_cell_field_preserves_hwp_owner() {
    check(true, "hwp");
}
#[test]
fn group_before_active_cell_field_preserves_hwpx_owner() {
    check(true, "hwpx");
}
#[test]
fn ungroup_before_active_cell_field_preserves_hwp_owner() {
    check(false, "hwp");
}
#[test]
fn ungroup_before_active_cell_field_preserves_hwpx_owner() {
    check(false, "hwpx");
}
