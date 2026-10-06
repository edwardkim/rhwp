//! #6950: 표 속성창은 raw 바이너리가 아닌 공통 IR의 기하를 조회해야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/issue6025/3232693_employment_support_criteria.hwpx";

fn open(path: &str) -> HwpDocument {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .expect("fixture");
    HwpDocument::from_bytes(&bytes).expect("parse")
}

fn props(doc: &HwpDocument, si: usize, pi: usize, ci: usize) -> Value {
    serde_json::from_str(
        &doc.get_table_properties(si as u32, pi as u32, ci as u32)
            .expect("table properties"),
    )
    .expect("JSON")
}

fn assert_geometry(doc: &HwpDocument, si: usize, pi: usize, ci: usize) {
    let Control::Table(table) = &doc.document().sections[si].paragraphs[pi].controls[ci] else {
        panic!("table");
    };
    let p = props(doc, si, pi, ci);
    let common = &table.common;
    assert_eq!(p["horzOffset"], common.horizontal_offset as i32);
    assert_eq!(p["vertOffset"], common.vertical_offset as i32);
    assert_eq!(p["tableWidth"], common.width);
    assert_eq!(p["tableHeight"], common.height);
    assert_eq!(p["outerLeft"], common.margin.left);
    assert_eq!(p["outerRight"], common.margin.right);
    assert_eq!(p["outerTop"], common.margin.top);
    assert_eq!(p["outerBottom"], common.margin.bottom);
    assert_eq!(p["keepWithAnchor"], common.prevent_page_break != 0);
}

#[test]
fn issue_6950_hwpx_properties_preserve_original_geometry_without_mutation() {
    let doc = open(SAMPLE);
    let before = serde_json::to_value(&doc.document().sections[0].paragraphs[1].controls)
        .expect("IR before");
    let p = props(&doc, 0, 1, 0);
    assert_eq!(p["horzOffset"], 709);
    assert_eq!(p["vertOffset"], 4129);
    assert_eq!(p["tableWidth"], 47199);
    assert_eq!(p["tableHeight"], 69352);
    assert_eq!(p["outerLeft"], 141);
    assert_eq!(p["horzRelTo"], "Para");
    assert_eq!(p["vertRelTo"], "Para");
    assert_geometry(&doc, 0, 1, 0);
    assert_eq!(
        serde_json::to_value(&doc.document().sections[0].paragraphs[1].controls).expect("IR after"),
        before
    );
}

#[test]
fn issue_6950_empty_raw_properties_follow_signed_and_zero_offset_edits() {
    let mut doc = open(SAMPLE);
    for (h, v, keep) in [(-709, -4129, true), (0, 0, false)] {
        doc.set_table_properties_native(
            0,
            1,
            0,
            &format!(r#"{{"horzOffset":{h},"vertOffset":{v},"keepWithAnchor":{keep}}}"#),
        )
        .expect("edit");
        let p = props(&doc, 0, 1, 0);
        assert_eq!(p["horzOffset"], h);
        assert_eq!(p["vertOffset"], v);
        assert_eq!(p["keepWithAnchor"], keep);
        assert_geometry(&doc, 0, 1, 0);
        let Control::Table(table) = &doc.document().sections[0].paragraphs[1].controls[0] else {
            panic!("table");
        };
        assert!(
            table.raw_ctrl_data.is_empty(),
            "query must not synthesize raw"
        );
    }
}

#[test]
fn issue_6950_hwp_properties_match_ir_with_and_without_raw() {
    for (path, expect_raw) in [
        ("samples/hwp5-tbl-attr-1916.hwp", false),
        ("samples/synam-001.hwp", true),
    ] {
        let doc = open(path);
        let mut matched = 0;
        for (si, section) in doc.document().sections.iter().enumerate() {
            for (pi, para) in section.paragraphs.iter().enumerate() {
                for (ci, control) in para.controls.iter().enumerate() {
                    if let Control::Table(table) = control {
                        if table.raw_ctrl_data.is_empty() != expect_raw {
                            assert_geometry(&doc, si, pi, ci);
                            matched += 1;
                        }
                    }
                }
            }
        }
        assert!(matched > 0, "{path}: expected raw availability");
    }
}
