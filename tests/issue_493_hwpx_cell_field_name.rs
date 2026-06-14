//! Issue #493: HWPX 파서가 `hp:tc name`(셀 필드 이름)을 누락
//!
//! 직렬화기(serializer/hwpx/table.rs)는 `cell.field_name`을 `name` 속성으로
//! 방출하지만 파서(parser/hwpx/section.rs `parse_table_cell`)가 이 속성을
//! 읽지 않아, HWPX 로드 시 `getFieldList`가 셀 필드를 반환하지 못하고
//! HWPX 라운드트립에서 셀 필드 이름이 유실됐다.
//!
//! 본 테스트는 표 샘플의 셀에 필드 이름을 부여한 뒤 serialize → reparse
//! 라운드트립으로 보존을 단언한다. 직렬화기가 무명 셀에도 `name=""`를
//! 방출하므로, 무명 셀이 `Some("")`로 오염되지 않고 `None`을 유지하는지
//! (HWP5 파서 `parse_cell_field_name`과 동일 의미) 함께 단언한다.

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

/// 문서 첫 번째 표의 (섹션, 문단, 컨트롤) 위치를 찾는다.
fn find_first_table(doc: &Document) -> Option<(usize, usize, usize)> {
    for (si, section) in doc.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if matches!(ctrl, Control::Table(_)) {
                    return Some((si, pi, ci));
                }
            }
        }
    }
    None
}

fn table_cell_names(doc: &Document, pos: (usize, usize, usize)) -> Vec<Option<String>> {
    let (si, pi, ci) = pos;
    match &doc.sections[si].paragraphs[pi].controls[ci] {
        Control::Table(table) => table.cells.iter().map(|c| c.field_name.clone()).collect(),
        _ => panic!("expected table control at {pos:?}"),
    }
}

#[test]
fn hwpx_cell_field_name_survives_roundtrip() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/hwpx/basic-table-01.hwpx");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));

    let mut doc = parse_hwpx(&bytes).expect("parse basic-table-01.hwpx");
    let pos = find_first_table(&doc).expect("sample should contain a table");

    // 첫 셀에 필드 이름 부여 (누름틀 셀 필드 시나리오)
    {
        let (si, pi, ci) = pos;
        let Control::Table(table) = &mut doc.sections[si].paragraphs[pi].controls[ci] else {
            panic!("expected table control");
        };
        assert!(
            table.cells.len() >= 2,
            "test needs >=2 cells (named + unnamed)"
        );
        table.cells[0].field_name = Some("사업명칭".to_string());
        table.cells[1].field_name = None;
    }

    let serialized = serialize_hwpx(&doc).expect("serialize_hwpx");
    let reparsed = parse_hwpx(&serialized).expect("reparse serialized hwpx");

    let pos2 = find_first_table(&reparsed).expect("table should survive roundtrip");
    let names = table_cell_names(&reparsed, pos2);
    assert!(
        names.len() >= 2,
        "라운드트립 후 셀 수가 보존되어야 한다 (셀 {}개)",
        names.len()
    );

    assert_eq!(
        names[0].as_deref(),
        Some("사업명칭"),
        "셀 필드 이름이 HWPX 라운드트립에서 보존되어야 한다 (#493)"
    );
    assert_eq!(
        names[1], None,
        "무명 셀은 name=\"\" 방출에도 None을 유지해야 한다 (Some(\"\") 오염 금지)"
    );
}
