use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::image::Picture;
use rhwp::model::table::{Cell, Table};
use rhwp::parser::parse_document;
use rhwp::renderer::composer::compose_paragraph;
use rhwp::renderer::height_measurer::HeightMeasurer;
use rhwp::renderer::style_resolver::resolve_styles_with_variant;
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn target_table(doc: &Document) -> &Table {
    match &doc.sections[0].paragraphs[0].controls[2] {
        Control::Table(table) => table,
        other => panic!("expected target table at para 0 control 2, got {other:?}"),
    }
}

fn target_cell(doc: &Document) -> &Cell {
    &target_table(doc).cells[2]
}

fn target_picture(doc: &Document) -> &Picture {
    let cell = target_cell(doc);
    match &cell.paragraphs[0].controls[0] {
        Control::Picture(pic) => pic,
        other => panic!("expected target cell picture, got {other:?}"),
    }
}

fn measured_target_row_heights(doc: &Document) -> Vec<f64> {
    let section = &doc.sections[0];
    let styles = resolve_styles_with_variant(&doc.doc_info, DEFAULT_DPI, doc.is_hwp3_variant);
    let composed = section
        .paragraphs
        .iter()
        .map(compose_paragraph)
        .collect::<Vec<_>>();
    let measured = HeightMeasurer::new(DEFAULT_DPI)
        .with_hwp3_variant(doc.is_hwp3_variant)
        .measure_section(&section.paragraphs, &composed, &styles, None);
    measured
        .tables
        .iter()
        .find(|t| t.para_index == 0 && t.control_index == 2)
        .expect("measured target table")
        .row_heights
        .clone()
}

fn required_cell_height_for_picture(cell: &Cell, pic: &Picture) -> u32 {
    let angle = (pic.shape_attr.rotation_angle as f64).to_radians();
    let visual_height = if pic.shape_attr.rotation_angle.rem_euclid(360) != 0
        && pic.shape_attr.current_width > 0
        && pic.shape_attr.current_height > 0
    {
        pic.common.height
    } else if pic.shape_attr.rotation_angle.rem_euclid(360) == 0 {
        pic.common.height
    } else {
        let sin = angle.sin().abs();
        let cos = angle.cos().abs();
        ((pic.common.width as f64 * sin + pic.common.height as f64 * cos)
            .round()
            .max(1.0)) as u32
    };
    let vert_offset = (pic.common.vertical_offset as i32).max(0) as u32;
    vert_offset
        .saturating_add(visual_height)
        .saturating_add(cell.padding.top as u32)
        .saturating_add(cell.padding.bottom as u32)
}

fn max_horz_offset_inside_cell(cell: &Cell, pic: &Picture) -> u32 {
    let inner_width = cell
        .width
        .saturating_sub(cell.padding.left.max(0) as u32)
        .saturating_sub(cell.padding.right.max(0) as u32);
    inner_width.saturating_sub(pic.common.width)
}

#[test]
fn issue_1282_page_area_limit_samples_drive_table_height_only_when_enabled() {
    let restricted = parse_document(&read_fixture("samples/ta-pic-001-r-쪽영역안제한.hwp"))
        .expect("parse restricted sample");
    let unrestricted = parse_document(&read_fixture("samples/ta-pic-001-r-쪽영역안제한no.hwp"))
        .expect("parse unrestricted sample");

    let restricted_pic = target_picture(&restricted);
    let unrestricted_pic = target_picture(&unrestricted);
    assert!(restricted_pic.common.flow_with_text);
    assert!(!unrestricted_pic.common.flow_with_text);

    let restricted_rows = measured_target_row_heights(&restricted);
    let unrestricted_rows = measured_target_row_heights(&unrestricted);
    let restricted_required = hwpunit_to_px(
        (restricted_pic.common.vertical_offset + restricted_pic.common.height) as i32,
        DEFAULT_DPI,
    );
    let unrestricted_saved_row =
        hwpunit_to_px(target_cell(&unrestricted).height as i32, DEFAULT_DPI);

    assert!(
        restricted_rows[1] >= restricted_required,
        "쪽 영역 제한 on은 문단 기준 그림의 vertOffset+height를 행 높이에 반영해야 함: row={:.1}, required={:.1}",
        restricted_rows[1],
        restricted_required
    );
    assert!(
        unrestricted_rows[1] <= unrestricted_saved_row + 1.0,
        "쪽 영역 제한 off는 문단 기준 그림으로 행 높이를 확장하지 않아야 함: row={:.1}, saved={:.1}",
        unrestricted_rows[1],
        unrestricted_saved_row
    );
}

#[test]
fn issue_1282_restrict_in_page_clamps_cell_picture_move_offsets() {
    let bytes = read_fixture("samples/ta-pic-001-r-쪽영역안제한.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load restricted sample");

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        r#"{"horzOffset":-5000,"vertOffset":-5000}"#,
    )
    .expect("move restricted picture outside cell start");

    let doc = core.document();
    let cell = target_cell(doc);
    let pic = target_picture(doc);
    assert!(pic.common.flow_with_text);
    assert_eq!(
        pic.common.horizontal_offset as i32, 0,
        "쪽 영역 제한 on 셀 그림은 셀 왼쪽 밖으로 이동하면 안 됨"
    );
    assert_eq!(
        pic.common.vertical_offset as i32, 0,
        "쪽 영역 제한 on 셀 그림은 셀 위쪽 밖으로 이동하면 안 됨"
    );

    let too_far_right = cell.width.saturating_mul(2);
    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        &format!(r#"{{"horzOffset":{}}}"#, too_far_right),
    )
    .expect("move restricted picture outside cell end");

    let doc = core.document();
    let cell = target_cell(doc);
    let pic = target_picture(doc);
    assert_eq!(
        pic.common.horizontal_offset,
        max_horz_offset_inside_cell(cell, pic),
        "쪽 영역 제한 on 셀 그림은 셀 오른쪽 밖으로 이동하면 안 됨"
    );
}

#[test]
fn issue_1282_turning_off_restrict_in_page_releases_picture_from_cell_flow() {
    let bytes = read_fixture("samples/ta-pic-001-r-쪽영역안제한.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load restricted sample");
    let before_cell_height = target_cell(core.document()).height;
    let before_table_height = target_table(core.document()).common.height;
    let no_sample = parse_document(&read_fixture("samples/ta-pic-001-r-쪽영역안제한no.hwp"))
        .expect("parse unrestricted oracle sample");

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        r#"{"restrictInPage":false}"#,
    )
    .expect("turn off restrictInPage");

    let doc = core.document();
    let table = target_table(doc);
    let cell = target_cell(doc);
    let pic = target_picture(doc);
    let outer_line_seg = doc.sections[0].paragraphs[0]
        .line_segs
        .first()
        .expect("outer paragraph line seg");
    let no_table = target_table(&no_sample);
    let no_cell = target_cell(&no_sample);
    let no_line_seg = no_sample.sections[0].paragraphs[0]
        .line_segs
        .first()
        .expect("oracle outer paragraph line seg");

    assert!(
        !pic.common.flow_with_text,
        "쪽 영역 제한 off 토글은 flow_with_text를 꺼야 함"
    );
    assert_eq!(
        cell.height, before_cell_height,
        "쪽 영역 제한 off 토글은 저장 샘플처럼 소유 셀 높이를 직접 줄이지 않아야 함"
    );
    assert_eq!(
        cell.height, no_cell.height,
        "쪽 영역 제한 off 토글 후 소유 셀 높이는 no 샘플과 같아야 함"
    );
    assert!(
        table.common.height < before_table_height,
        "쪽 영역 제한 off 토글 후 표 높이도 줄어야 함: before={}, after={}",
        before_table_height,
        table.common.height
    );
    assert_eq!(
        table.common.height, no_table.common.height,
        "쪽 영역 제한 off 토글 후 표 높이는 no 샘플과 같아야 함"
    );
    assert_eq!(
        outer_line_seg.vertical_pos, no_line_seg.vertical_pos,
        "쪽 영역 제한 off 토글 후 표 문단 vertical_pos는 no 샘플과 같아야 함"
    );
    assert_eq!(
        outer_line_seg.line_height, no_line_seg.line_height,
        "쪽 영역 제한 off 토글 후 표 문단 line_height는 no 샘플과 같아야 함"
    );
}

#[test]
fn issue_1282_unrestricted_cell_picture_move_offsets_are_not_clamped() {
    let bytes = read_fixture("samples/ta-pic-001-r-쪽영역안제한no.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load unrestricted sample");

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        r#"{"horzOffset":-5000,"vertOffset":-5000}"#,
    )
    .expect("move unrestricted picture outside cell start");

    let pic = target_picture(core.document());
    assert!(!pic.common.flow_with_text);
    assert_eq!(
        pic.common.horizontal_offset as i32, -5000,
        "쪽 영역 제한 off 셀 그림은 기존 자유 이동 의미를 유지해야 함"
    );
    assert_eq!(
        pic.common.vertical_offset as i32, -5000,
        "쪽 영역 제한 off 셀 그림은 기존 자유 이동 의미를 유지해야 함"
    );

    let props_json = core
        .get_cell_picture_properties_by_path_native(
            0,
            0,
            r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
            0,
        )
        .expect("query unrestricted picture props");
    assert!(
        props_json.contains(r#""horzOffset":-5000"#)
            && props_json.contains(r#""vertOffset":-5000"#),
        "속성창 JSON은 음수 offset을 u32 래핑값이 아니라 signed 값으로 노출해야 함: {props_json}"
    );
}

#[test]
fn issue_1282_unrestricted_take_place_negative_offset_pulls_table_up() {
    let bytes = read_fixture("samples/ta-pic-001-r-쪽영역안제한.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load restricted sample");

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        r#"{"restrictInPage":false}"#,
    )
    .expect("turn off restrictInPage");
    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        r#"{"vertOffset":-5890,"horzOffset":2030}"#,
    )
    .expect("move unrestricted take-place picture upward");

    let doc = core.document();
    let pic = target_picture(doc);
    let line_seg = doc.sections[0].paragraphs[0]
        .line_segs
        .first()
        .expect("outer paragraph line seg");
    assert_eq!(
        pic.common.vertical_offset as i32, -5890,
        "제한 off 자리차지 그림은 상단 이동 음수 offset을 보존해야 함"
    );
    assert_eq!(
        line_seg.vertical_pos,
        pic.common.vertical_offset as i32 + pic.common.height as i32,
        "자리차지 그림의 부모 line_seg는 vertOffset+height 기준으로 아래 표를 당겨야 함"
    );
    assert!(
        line_seg.vertical_pos > 0,
        "위로 이동해도 flow 높이는 양수 범위여야 함: {}",
        line_seg.vertical_pos
    );
}

fn picture_center(pic: &Picture) -> (i64, i64) {
    (
        pic.common.horizontal_offset as i32 as i64 + pic.common.width as i64 / 2,
        pic.common.vertical_offset as i32 as i64 + pic.common.height as i64 / 2,
    )
}

#[test]
fn issue_1282_resizing_rotated_cell_picture_grows_owner_cell_height() {
    let bytes = read_fixture("samples/ta-pic-001-r.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load HWP fixture");
    let original_width = target_picture(core.document()).common.width;
    let original_cell_width = target_cell(core.document()).width;

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        r#"{"height":30000}"#,
    )
    .expect("resize rotated cell picture");

    let doc = core.document();
    let table = target_table(doc);
    let cell = target_cell(doc);
    let pic = target_picture(doc);
    let required_height = required_cell_height_for_picture(cell, pic);

    assert!(
        cell.height >= required_height,
        "owner cell must grow to contain resized rotated picture: cell.height={}, required={}, pic.vertOffset={}, pic.height={}, pad=({}, {})",
        cell.height,
        required_height,
        pic.common.vertical_offset,
        pic.common.height,
        cell.padding.top,
        cell.padding.bottom
    );
    assert!(
        table.common.height >= cell.height,
        "table common height must follow grown cell height: table.common.height={}, cell.height={}",
        table.common.height,
        cell.height
    );
    let grown_cell_height = cell.height;
    let grown_current_width = pic.shape_attr.current_width;
    let grown_current_height = pic.shape_attr.current_height;
    let grown_center = picture_center(pic);

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        r#"{"rotationAngle":0}"#,
    )
    .expect("unrotate cell picture");

    let doc = core.document();
    let table = target_table(doc);
    let cell = target_cell(doc);
    let pic = target_picture(doc);
    let required_height = required_cell_height_for_picture(cell, pic);
    let unrotated_center = picture_center(pic);

    assert_eq!(
        pic.common.width, grown_current_width,
        "rotation 0 edit must use the current image width"
    );
    assert_eq!(
        pic.common.height, grown_current_height,
        "rotation 0 edit must use the current image height"
    );
    assert!(
        (unrotated_center.0 - grown_center.0).abs() <= 1
            && (unrotated_center.1 - grown_center.1).abs() <= 1,
        "rotation edit must preserve visual center: grown={:?}, unrotated={:?}",
        grown_center,
        unrotated_center
    );
    assert!(
        cell.height < grown_cell_height,
        "owner cell height must shrink when rotation angle alone removes visual hull: grown={}, current={}",
        grown_cell_height,
        cell.height
    );
    assert_eq!(
        cell.height, required_height,
        "owner cell height must sync after rotation-only edit: cell.height={}, required={}",
        cell.height, required_height
    );
    assert!(
        table.common.height >= cell.height,
        "table common height must follow rotation-only cell height: table.common.height={}, cell.height={}",
        table.common.height,
        cell.height
    );

    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        &format!(
            r#"{{"width":{},"height":12000,"rotationAngle":0}}"#,
            original_width
        ),
    )
    .expect("shrink and unrotate cell picture");

    let doc = core.document();
    let table = target_table(doc);
    let cell = target_cell(doc);
    let pic = target_picture(doc);
    let required_height = required_cell_height_for_picture(cell, pic);

    assert!(
        cell.height < grown_cell_height,
        "owner cell height must shrink after picture shrinks/unrotates: grown={}, current={}",
        grown_cell_height,
        cell.height
    );
    assert_eq!(
        cell.height, required_height,
        "owner cell height must sync to shrunken unrotated picture: cell.height={}, required={}",
        cell.height, required_height
    );
    assert!(
        table.common.height >= cell.height,
        "table common height must follow shrunken cell height: table.common.height={}, cell.height={}",
        table.common.height,
        cell.height
    );

    let original_table_width = table.common.width;
    let oversized_width = cell.width.saturating_mul(3);
    core.set_cell_picture_properties_by_path_native(
        0,
        0,
        r#"[{"controlIdx":2,"cellIdx":2,"cellParaIdx":0}]"#,
        0,
        &format!(
            r#"{{"width":{},"height":36000,"rotationAngle":34}}"#,
            oversized_width
        ),
    )
    .expect("oversized resize must preserve picture width and sync owner cell height");

    let doc = core.document();
    let table = target_table(doc);
    let cell = target_cell(doc);
    let pic = target_picture(doc);
    let required_height = required_cell_height_for_picture(cell, pic);

    assert!(
        pic.common.width > original_cell_width,
        "oversized picture frame must remain larger than the original owner cell width: frame={}, original_cell={}",
        pic.common.width,
        original_cell_width
    );
    assert_eq!(
        cell.width, original_cell_width,
        "horizontal resize must not change owner cell width"
    );
    assert_eq!(
        table.common.width, original_table_width,
        "horizontal resize must not change table width"
    );
    assert_eq!(
        cell.height, required_height,
        "owner cell height must sync after oversized resize: cell.height={}, required={}",
        cell.height, required_height
    );

    let exported = core.export_hwp_native().expect("export edited HWP");
    let reparsed = parse_document(&exported).expect("reparse edited HWP");
    let reparsed_table = target_table(&reparsed);
    let reparsed_cell = target_cell(&reparsed);
    let reparsed_pic = target_picture(&reparsed);
    let reparsed_required = required_cell_height_for_picture(reparsed_cell, reparsed_pic);

    assert!(
        reparsed_cell.height >= reparsed_required,
        "exported HWP must preserve grown owner cell height: cell.height={}, required={}",
        reparsed_cell.height,
        reparsed_required
    );
    assert!(
        reparsed_table.common.height >= reparsed_cell.height,
        "exported HWP table height must preserve grown cell height"
    );
}
