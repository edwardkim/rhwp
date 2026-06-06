use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::image::Picture;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::parse_document;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn hero_png() -> Vec<u8> {
    std::fs::read("/Users/phihu/ecrits/priv/static/images/landing/hero.png").expect("read hero.png")
}

fn first_body_picture(doc: &Document) -> &Picture {
    for para in &doc.sections[0].paragraphs {
        for ctrl in &para.controls {
            match ctrl {
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
    panic!("body picture not found")
}

// Find which paragraph/control index the picture lives at (mirrors NIF dispatch).
fn locate_body_picture(doc: &Document) -> (usize, usize) {
    for (pi, para) in doc.sections[0].paragraphs.iter().enumerate() {
        for (ci, ctrl) in para.controls.iter().enumerate() {
            if matches!(ctrl, Control::Picture(_)) {
                return (pi, ci);
            }
            if let Control::Shape(s) = ctrl {
                if matches!(s.as_ref(), ShapeObject::Picture(_)) {
                    return (pi, ci);
                }
            }
        }
    }
    panic!("body picture not found");
}

#[test]
fn repro_insert_picture_then_set_geometry_roundtrips() {
    let bytes = read_fixture("template/blank-batang.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load blank template");

    // title text on para 0
    core.insert_text_native(0, 0, 0, "이미지 삽입 테스트")
        .expect("insert title");

    // insert picture at end of title para
    let title_len = "이미지 삽입 테스트".chars().count();
    let png = hero_png();
    core.insert_picture_native(
        0,
        0,
        title_len,
        &[],
        &png,
        4000,
        2500,
        1024,
        1024,
        "png",
        "",
        None,
        None,
    )
    .expect("insert picture");

    let (pi, ci) = locate_body_picture(core.document());
    eprintln!("picture located at para={pi} ctrl={ci}");

    // Apply the supervisor's exact geometry edit.
    core.set_picture_properties_native(
        0,
        pi,
        ci,
        r#"{"width":4000,"height":2500,"horzOffset":1000,"vertOffset":800}"#,
    )
    .expect("set picture geometry");

    // In-memory check (pre-save).
    let pic = first_body_picture(core.document());
    eprintln!(
        "IN-MEM: width={} height={} horzOffset={} vertOffset={} tac={} horzRel={:?} vertRel={:?}",
        pic.common.width,
        pic.common.height,
        pic.common.horizontal_offset as i32,
        pic.common.vertical_offset as i32,
        pic.common.treat_as_char,
        pic.common.horz_rel_to,
        pic.common.vert_rel_to,
    );

    // Save -> reopen.
    let exported = core.export_hwp_native().expect("export edited HWP");
    let reparsed = parse_document(&exported).expect("reparse edited HWP");
    let pic = first_body_picture(&reparsed);

    eprintln!(
        "ROUNDTRIP: width={} height={} horzOffset={} vertOffset={} (u32 vert={})",
        pic.common.width,
        pic.common.height,
        pic.common.horizontal_offset as i32,
        pic.common.vertical_offset as i32,
        pic.common.vertical_offset,
    );

    assert_eq!(pic.common.width, 4000, "width must round-trip");
    assert_eq!(pic.common.height, 2500, "height must round-trip");
    assert_eq!(
        pic.common.horizontal_offset as i32, 1000,
        "horzOffset must round-trip"
    );
    assert_eq!(
        pic.common.vertical_offset as i32, 800,
        "vertOffset must round-trip"
    );
}

#[test]
fn rotation_getter_reports_signed_display_geometry_not_u32_garbage() {
    let bytes = read_fixture("template/blank-batang.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load blank template");
    core.insert_text_native(0, 0, 0, "이미지 삽입 테스트")
        .expect("insert title");
    let title_len = "이미지 삽입 테스트".chars().count();
    let png = hero_png();
    core.insert_picture_native(
        0,
        0,
        title_len,
        &[],
        &png,
        4000,
        2500,
        1024,
        1024,
        "png",
        "",
        None,
        None,
    )
    .expect("insert picture");
    let (pi, ci) = locate_body_picture(core.document());

    // size+offset first, THEN rotation (the coverage-matrix cell that broke).
    core.set_picture_properties_native(
        0,
        pi,
        ci,
        r#"{"width":4000,"height":2500,"horzOffset":1000,"vertOffset":800}"#,
    )
    .expect("set geometry");
    core.set_picture_properties_native(0, pi, ci, r#"{"rotationAngle":30}"#)
        .expect("rotate");

    {
        let pic = first_body_picture(core.document());
        eprintln!(
            "MODEL after rotation: common.w={} common.h={} cur.w={} cur.h={} orig.w={} orig.h={} hoff={} voff={} angle={}",
            pic.common.width, pic.common.height,
            pic.shape_attr.current_width, pic.shape_attr.current_height,
            pic.shape_attr.original_width, pic.shape_attr.original_height,
            pic.common.horizontal_offset as i32, pic.common.vertical_offset as i32,
            pic.shape_attr.rotation_angle,
        );
    }
    let json = core
        .get_picture_properties_native(0, pi, ci)
        .expect("get props");
    eprintln!("GETTER JSON after 30deg rotation:\n{json}");
    // Before the fix this read back width 4714 / height 4165 / horzOffset 643 /
    // vertOffset 4294967264 (rotated bbox + u32-underflow of a negative offset).
    // After the fix the getter reports the user's display geometry, signed.
    assert!(
        json.contains("\"width\":4000"),
        "width must report display size 4000, not rotated bbox: {json}"
    );
    assert!(
        json.contains("\"height\":2500"),
        "height must report display size 2500, not rotated bbox: {json}"
    );
    assert!(
        json.contains("\"horzOffset\":1000"),
        "horzOffset must round-trip to 1000: {json}"
    );
    assert!(
        json.contains("\"vertOffset\":800"),
        "vertOffset must round-trip to 800 (signed), not 4294967264: {json}"
    );
    assert!(
        !json.contains("4294967264"),
        "no u32-underflow garbage allowed: {json}"
    );
}

fn assert_hancom_openable(bytes: &[u8]) {
    use std::io::Cursor;
    // 1. Strict CFB structural check + required HWP streams present.
    let mut cfb = cfb::CompoundFile::open(Cursor::new(bytes.to_vec()))
        .expect("exported bytes must be a valid CFB/OLE compound file");
    let entries: Vec<String> = cfb
        .walk()
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    eprintln!("CFB entries: {entries:?}");
    let has = |needle: &str| entries.iter().any(|e| e.contains(needle));
    assert!(has("FileHeader"), "missing FileHeader stream: {entries:?}");
    assert!(has("DocInfo"), "missing DocInfo stream: {entries:?}");
    assert!(has("Section0"), "missing BodyText/Section0: {entries:?}");
    // 2. Strict reparse (fresh parser) must succeed.
    parse_document(bytes).expect("exported HWP must reparse cleanly");
}

#[test]
fn shape_doc_is_hancom_openable() {
    let bytes = read_fixture("template/blank-batang.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load blank template");
    core.insert_text_native(0, 0, 0, "주황색 직사각형 도형")
        .expect("insert title");
    let title_len = "주황색 직사각형 도형".chars().count();
    core.create_shape_control_native(
        0,
        0,
        title_len,
        6000,
        3000,
        0,
        0,
        true,
        "Square",
        "rectangle",
        false,
        false,
        &[],
    )
    .expect("create rectangle shape");

    let exported = core.export_hwp_native().expect("export shape HWP");
    assert_hancom_openable(&exported);

    // reparse and confirm a shape control survives
    let reparsed = parse_document(&exported).expect("reparse shape HWP");
    let mut found = false;
    for para in &reparsed.sections[0].paragraphs {
        for ctrl in &para.controls {
            if matches!(ctrl, Control::Shape(_)) {
                found = true;
            }
        }
    }
    assert!(found, "rectangle shape control must survive round-trip");
}

#[test]
fn picture_doc_is_hancom_openable() {
    let bytes = read_fixture("template/blank-batang.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load blank template");
    core.insert_text_native(0, 0, 0, "이미지 삽입 테스트")
        .expect("insert title");
    let title_len = "이미지 삽입 테스트".chars().count();
    let png = hero_png();
    core.insert_picture_native(
        0,
        0,
        title_len,
        &[],
        &png,
        4000,
        2500,
        1024,
        1024,
        "png",
        "",
        None,
        None,
    )
    .expect("insert picture");
    let exported = core.export_hwp_native().expect("export picture HWP");
    assert_hancom_openable(&exported);
    // BinStorage embedded image present.
    use std::io::Cursor;
    let mut cfb = cfb::CompoundFile::open(Cursor::new(exported.clone())).unwrap();
    let has_bin = cfb
        .walk()
        .any(|e| e.path().to_string_lossy().contains("BinData"));
    assert!(has_bin, "embedded image BinData storage must be present");
}

fn collect_nodes<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        collect_nodes(c, out);
    }
}

#[test]
fn inline_picture_lays_out_in_text_flow_not_floating_top_left() {
    // Authoritative layout proof on the current engine's page render tree: the
    // inserted picture produces an Image render node that flows inline with the
    // title text (top at/below the title line), NOT a floating box dumped at the
    // page origin that pushes the title aside.
    let bytes = read_fixture("template/blank-batang.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load blank template");
    core.insert_text_native(0, 0, 0, "이미지 삽입 테스트")
        .expect("insert title");
    let title_len = "이미지 삽입 테스트".chars().count();
    let png = hero_png();
    core.insert_picture_native(
        0,
        0,
        title_len,
        &[],
        &png,
        4000,
        2500,
        1024,
        1024,
        "png",
        "",
        None,
        None,
    )
    .expect("insert picture");

    let tree = core.build_page_render_tree(0).expect("build page 0 tree");
    let mut nodes = Vec::new();
    collect_nodes(&tree.root, &mut nodes);

    let mut image_box = None;
    let mut text_min_y = f64::MAX;
    for n in &nodes {
        match &n.node_type {
            RenderNodeType::Image(_) => image_box = Some(n.bbox),
            RenderNodeType::TextRun(_) | RenderNodeType::TextLine(_) => {
                text_min_y = text_min_y.min(n.bbox.y);
            }
            _ => {}
        }
    }

    let img = image_box.expect("inline picture must produce an Image render node");
    eprintln!(
        "IMAGE bbox x={:.1} y={:.1} w={:.1} h={:.1}; title top y={:.1}",
        img.x, img.y, img.width, img.height, text_min_y
    );
    assert!(
        img.width > 1.0 && img.height > 1.0,
        "image must have a real box"
    );
    assert!(
        img.y >= text_min_y - 1.0,
        "image top (y={:.1}) must be at/below the title line ({:.1}), not floating above",
        img.y,
        text_min_y
    );
    assert!(
        img.x > 1.0,
        "inline image must flow within the text body (x>{:.1}), not pinned to the page origin",
        img.x
    );
}

#[test]
fn repro_inserted_picture_default_placement_is_inline() {
    let bytes = read_fixture("template/blank-batang.hwp");
    let mut core = DocumentCore::from_bytes(&bytes).expect("load blank template");
    core.insert_text_native(0, 0, 0, "이미지 삽입 테스트")
        .expect("insert title");
    let title_len = "이미지 삽입 테스트".chars().count();
    let png = hero_png();
    core.insert_picture_native(
        0,
        0,
        title_len,
        &[],
        &png,
        4000,
        2500,
        1024,
        1024,
        "png",
        "",
        None,
        None,
    )
    .expect("insert picture");

    let pic = first_body_picture(core.document());
    eprintln!(
        "DEFAULT PLACEMENT: tac={} horzRel={:?} vertRel={:?} horzOffset={} vertOffset={} wrap={:?}",
        pic.common.treat_as_char,
        pic.common.horz_rel_to,
        pic.common.vert_rel_to,
        pic.common.horizontal_offset as i32,
        pic.common.vertical_offset as i32,
        pic.common.text_wrap,
    );
    assert!(
        pic.common.treat_as_char,
        "freshly inserted picture should default to inline (treat_as_char)"
    );
}
