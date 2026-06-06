use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::image::Picture;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::parse_document;

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn read_f64(raw: &[u8], offset: usize) -> f64 {
    f64::from_le_bytes(
        raw.get(offset..offset + 8)
            .unwrap_or_else(|| panic!("raw rendering offset {offset} out of range"))
            .try_into()
            .expect("f64 bytes"),
    )
}

fn first_cell_picture(doc: &Document) -> &Picture {
    for para in &doc.sections[0].paragraphs {
        for ctrl in &para.controls {
            if let Control::Table(table) = ctrl {
                for cell in &table.cells {
                    for cell_para in &cell.paragraphs {
                        for inner in &cell_para.controls {
                            match inner {
                                Control::Picture(pic) => return pic,
                                Control::Shape(shape) => {
                                    if let ShapeObject::Picture(pic) = shape.as_ref() {
                                        return pic;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    panic!("cell picture not found")
}

fn assert_rotated_picture_contract(pic: &Picture, expected_angle: i16) {
    assert_eq!(pic.shape_attr.rotation_angle, expected_angle);
    assert_ne!(
        pic.common.width, pic.shape_attr.current_width,
        "rotated picture common width must be the rotated bbox, not curSz"
    );
    assert_ne!(
        pic.common.height, pic.shape_attr.current_height,
        "rotated picture common height must be the rotated bbox, not curSz"
    );
    assert!(
        pic.shape_attr.raw_rendering.len() >= 146,
        "rotated picture must carry HWP5 rendering matrix"
    );

    let raw = &pic.shape_attr.raw_rendering;
    let count = u16::from_le_bytes([raw[0], raw[1]]);
    assert_eq!(count, 1, "expected one scale/rotation rendering pair");

    let scale_start = 2 + 48;
    let rot_start = scale_start + 48;
    let sx = read_f64(raw, scale_start);
    let sy = read_f64(raw, scale_start + 4 * 8);
    let rot_a = read_f64(raw, rot_start);
    let rot_b = read_f64(raw, rot_start + 8);
    let rot_c = read_f64(raw, rot_start + 3 * 8);
    let rot_d = read_f64(raw, rot_start + 4 * 8);
    let rot_tx = read_f64(raw, rot_start + 2 * 8);

    assert!(sx > 0.0 && sy > 0.0, "scale matrix must be materialized");
    let theta = (expected_angle as f64).to_radians();
    assert!(
        (rot_a - theta.cos()).abs() < 0.000_001,
        "rotMatrix e1 must match cos(angle)"
    );
    assert!(
        (rot_b + theta.sin()).abs() < 0.000_001,
        "rotMatrix e2 must match -sin(angle)"
    );
    assert!(
        (rot_c - theta.sin()).abs() < 0.000_001,
        "rotMatrix e4 must match sin(angle)"
    );
    assert!(
        (rot_d - theta.cos()).abs() < 0.000_001,
        "rotMatrix e5 must match cos(angle)"
    );
    assert!(
        rot_tx.abs() > 1.0,
        "rotated picture needs a positive-bounds translation in rotMatrix"
    );
}

#[test]
fn issue_1279_reference_hwp_and_hwpx_preserve_picture_rotation_matrix() {
    for (path, expected_flip_mask) in [
        ("samples/ta-pic-001-r.hwp", 0x0008_0000),
        ("samples/hwpx/ta-pic-001-r.hwpx", 0x0008_0000),
    ] {
        let bytes = read_fixture(path);
        let doc = parse_document(&bytes).unwrap_or_else(|e| panic!("parse {path}: {e}"));
        let pic = first_cell_picture(&doc);
        assert_eq!(pic.common.width, 18425, "{path}: rotated bbox width");
        assert_eq!(pic.common.height, 18160, "{path}: rotated bbox height");
        assert_eq!(pic.shape_attr.current_width, 13668, "{path}: curSz width");
        assert_eq!(pic.shape_attr.current_height, 12686, "{path}: curSz height");
        assert_eq!(
            pic.shape_attr.flip & expected_flip_mask,
            expected_flip_mask,
            "{path}: rotate-image storage bit must be preserved"
        );
        assert_rotated_picture_contract(pic, 34);
    }
}

#[test]
fn issue_1279_hwpx_to_hwp_export_preserves_picture_rotation_contract() {
    let bytes = read_fixture("samples/hwpx/ta-pic-001-r.hwpx");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load HWPX fixture");

    let exported = core
        .export_hwp_with_adapter()
        .expect("export HWPX source to HWP");
    let reparsed = parse_document(&exported).expect("reparse exported HWP");
    let pic = first_cell_picture(&reparsed);

    assert_eq!(pic.common.width, 18425, "exported rotated bbox width");
    assert_eq!(pic.common.height, 18160, "exported rotated bbox height");
    assert_eq!(pic.shape_attr.current_width, 13668, "exported curSz width");
    assert_eq!(
        pic.shape_attr.current_height, 12686,
        "exported curSz height"
    );
    assert_eq!(
        pic.shape_attr.flip & 0x0008_0000,
        0x0008_0000,
        "exported rotate-image storage bit must be preserved"
    );
    assert_rotated_picture_contract(pic, 34);
}

#[test]
fn issue_1279_cell_picture_rotation_edit_exports_hancom_rendering_matrix() {
    let bytes = read_fixture("samples/ta-pic-001-r.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load HWP fixture");
    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        r#"{"rotationAngle":44}"#,
    )
    .expect("set cell picture rotation");

    let exported = core.export_hwp_native().expect("export edited HWP");
    let reparsed = parse_document(&exported).expect("reparse edited HWP");
    let pic = first_cell_picture(&reparsed);

    assert_eq!(pic.shape_attr.current_width, 13668);
    assert_eq!(pic.shape_attr.current_height, 12686);
    assert_rotated_picture_contract(pic, 44);
}

#[test]
fn issue_1279_resizing_rotated_cell_picture_preserves_scaled_current_size() {
    let bytes = read_fixture("samples/ta-pic-001-r.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load HWP fixture");

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        r#"{"width":20000,"height":19000}"#,
    )
    .expect("resize rotated cell picture");

    let resized = first_cell_picture(core.document());
    assert_eq!(resized.common.width, 20000);
    assert_eq!(resized.common.height, 19000);
    assert_eq!(
        resized.shape_attr.current_width, 14836,
        "display bbox width must scale curSz instead of replacing it"
    );
    assert_eq!(
        resized.shape_attr.current_height, 13273,
        "display bbox height must scale curSz instead of replacing it"
    );

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        r#"{"rotationAngle":44}"#,
    )
    .expect("rotate resized cell picture");

    let exported = core.export_hwp_native().expect("export edited HWP");
    let reparsed = parse_document(&exported).expect("reparse edited HWP");
    let pic = first_cell_picture(&reparsed);

    assert_eq!(
        pic.shape_attr.current_width, 14836,
        "rotation must not restore the pre-resize image width"
    );
    assert_eq!(
        pic.shape_attr.current_height, 13273,
        "rotation must not restore the pre-resize image height"
    );
    assert_rotated_picture_contract(pic, 44);
}
