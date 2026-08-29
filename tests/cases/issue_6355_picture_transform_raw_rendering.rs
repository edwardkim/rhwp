#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::image::Picture;

const SAMPLE: &str = "samples/ta-pic-001-r.hwp";
const CELL_PATH: &str = r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#;

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn cell_picture(core: &DocumentCore) -> &Picture {
    let table = match &core.document().sections[0].paragraphs[0].controls[2] {
        Control::Table(table) => table,
        _ => panic!("fixture coordinate is not a table"),
    };
    table.cells[2].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Picture(picture) => Some(picture.as_ref()),
            _ => None,
        })
        .expect("fixture cell has no picture")
}

fn loaded() -> (DocumentCore, Vec<u8>) {
    let bytes = read_fixture(SAMPLE);
    let core = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    let raw_rendering = cell_picture(&core).shape_attr.raw_rendering.clone();
    assert!(
        raw_rendering.len() >= 146,
        "fixture should carry Hancom rendering matrix bytes"
    );
    (core, raw_rendering)
}

#[test]
fn issue_6355_same_valued_transform_props_keep_raw_rendering() {
    let (mut core, original) = loaded();
    let bag = {
        let picture = cell_picture(&core);
        format!(
            r#"{{"width":{},"height":{},"horzOffset":{},"vertOffset":{},"horzFlip":{},"vertFlip":{}}}"#,
            picture.common.width,
            picture.common.height,
            picture.common.horizontal_offset as i32,
            picture.common.vertical_offset as i32,
            picture.shape_attr.horz_flip,
            picture.shape_attr.vert_flip,
        )
    };

    core.set_cell_picture_properties_by_path_native(0, 0, CELL_PATH, 0, &bag)
        .expect("reapply same-valued transform properties");

    assert_eq!(
        cell_picture(&core).shape_attr.raw_rendering,
        original,
        "same-valued transform properties should not discard original rendering matrix"
    );
}

#[test]
fn issue_6355_transform_key_inside_string_value_keeps_raw_rendering() {
    let (mut core, original) = loaded();

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        CELL_PATH,
        0,
        r#"{"note":"\"width\" mentioned in a string"}"#,
    )
    .expect("apply unrelated property");

    assert_eq!(
        cell_picture(&core).shape_attr.raw_rendering,
        original,
        "a transform key mentioned inside a string value should not invalidate raw_rendering"
    );
}

#[test]
fn issue_6355_non_transform_props_keep_raw_rendering() {
    let (mut core, original) = loaded();

    core.set_cell_picture_properties_by_path_native(0, 0, CELL_PATH, 0, r#"{"brightness":20}"#)
        .expect("set non-transform property");

    assert_eq!(
        cell_picture(&core).shape_attr.raw_rendering,
        original,
        "non-transform properties should not invalidate raw_rendering"
    );
}

#[test]
fn issue_6355_changed_transform_props_invalidate_raw_rendering() {
    let (mut core, _original) = loaded();
    let wider = cell_picture(&core).common.width + 2000;

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        CELL_PATH,
        0,
        &format!(r#"{{"width":{wider}}}"#),
    )
    .expect("set changed transform property");

    assert!(
        cell_picture(&core).shape_attr.raw_rendering.is_empty(),
        "changed transform properties should clear raw_rendering so the serializer regenerates it"
    );
}
