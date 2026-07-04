use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::shape::CommonObjAttr;

const SIZE_PROTECT_BIT: u32 = 1 << 20;
const TARGET_PICTURE_PATH: &str = r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#;
const TARGET_SHAPE_PATH: &str = r#"[{"controlIdx":2,"cellIdx":5,"cellParaIdx":0}]"#;

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn load_picture_sample() -> DocumentCore {
    DocumentCore::from_bytes(&read_fixture("samples/ta-pic-001-r-쪽영역안제한.hwp"))
        .expect("load picture sample")
}

fn load_shape_sample() -> DocumentCore {
    DocumentCore::from_bytes(&read_fixture("samples/inner-table-01.hwp"))
        .expect("load shape sample")
}

fn target_picture_common(doc: &Document) -> &CommonObjAttr {
    let table = match &doc.sections[0].paragraphs[0].controls[2] {
        Control::Table(table) => table,
        other => panic!("expected target table, got {other:?}"),
    };
    match &table.cells[2].paragraphs[0].controls[0] {
        Control::Picture(pic) => &pic.common,
        other => panic!("expected target cell picture, got {other:?}"),
    }
}

fn target_shape_common(doc: &Document) -> &CommonObjAttr {
    let table = match &doc.sections[0].paragraphs[0].controls[2] {
        Control::Table(table) => table,
        other => panic!("expected target table, got {other:?}"),
    };
    match &table.cells[5].paragraphs[0].controls[0] {
        Control::Shape(shape) => shape.common(),
        other => panic!("expected target cell shape, got {other:?}"),
    }
}

#[test]
fn issue_1436_picture_properties_round_trip_size_protect() {
    let mut core = load_picture_sample();

    let before = core
        .get_cell_picture_properties_by_path_native(0, 0, TARGET_PICTURE_PATH, 0)
        .expect("initial picture properties");
    let before_v: serde_json::Value =
        serde_json::from_str(&before).expect("parse initial picture properties");
    assert_eq!(
        before_v["sizeProtect"].as_bool(),
        Some(false),
        "기본 샘플은 크기 고정 off 상태여야 함"
    );

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        TARGET_PICTURE_PATH,
        0,
        r#"{"sizeProtect":true}"#,
    )
    .expect("enable picture sizeProtect");

    let enabled = target_picture_common(core.document());
    assert!(enabled.size_protect, "모델 size_protect가 켜져야 함");
    assert_ne!(
        enabled.attr & SIZE_PROTECT_BIT,
        0,
        "HWP5 attr bit 20도 함께 켜져야 함"
    );

    let props = core
        .get_cell_picture_properties_by_path_native(0, 0, TARGET_PICTURE_PATH, 0)
        .expect("enabled picture properties");
    let props_v: serde_json::Value =
        serde_json::from_str(&props).expect("parse enabled picture properties");
    assert_eq!(props_v["sizeProtect"].as_bool(), Some(true));

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        TARGET_PICTURE_PATH,
        0,
        r#"{"sizeProtect":false}"#,
    )
    .expect("disable picture sizeProtect");

    let disabled = target_picture_common(core.document());
    assert!(!disabled.size_protect, "모델 size_protect가 꺼져야 함");
    assert_eq!(
        disabled.attr & SIZE_PROTECT_BIT,
        0,
        "HWP5 attr bit 20도 함께 꺼져야 함"
    );
}

#[test]
fn issue_1436_shape_properties_round_trip_size_protect() {
    let mut core = load_shape_sample();

    let before = core
        .get_cell_shape_properties_by_path_native(0, 0, TARGET_SHAPE_PATH, 0)
        .expect("initial shape properties");
    let before_v: serde_json::Value =
        serde_json::from_str(&before).expect("parse initial shape properties");
    assert_eq!(before_v["sizeProtect"].as_bool(), Some(false));

    core.set_cell_shape_properties_by_path_native(
        0,
        0,
        TARGET_SHAPE_PATH,
        0,
        r#"{"sizeProtect":true}"#,
    )
    .expect("enable shape sizeProtect");

    let enabled = target_shape_common(core.document());
    assert!(enabled.size_protect, "도형 모델 size_protect가 켜져야 함");
    assert_ne!(
        enabled.attr & SIZE_PROTECT_BIT,
        0,
        "도형 HWP5 attr bit 20도 함께 켜져야 함"
    );

    let props = core
        .get_cell_shape_properties_by_path_native(0, 0, TARGET_SHAPE_PATH, 0)
        .expect("enabled shape properties");
    let props_v: serde_json::Value =
        serde_json::from_str(&props).expect("parse enabled shape properties");
    assert_eq!(props_v["sizeProtect"].as_bool(), Some(true));
}
