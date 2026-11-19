//! 캡션 너비 변경 뒤 다시 줄 나누기, '여백 부분까지 너비 확대',
//! 글자처럼 취급 그림과 셀 안 그림의 캡션 자리를 한컴 12.30 기준으로 검사한다.

use crate::document_core::DocumentCore;
use crate::model::control::Control;
use crate::renderer::layout::CellContext;
use crate::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("빈 문서");
    core
}

fn json_usize(json: &str, key: &str) -> usize {
    let value: serde_json::Value = serde_json::from_str(json).expect("JSON");
    value[key].as_u64().expect(key) as usize
}

fn png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

/// 캡션 글줄 상자와 첫 글자 조각의 셀 문맥.
fn caption_lines(core: &DocumentCore) -> Vec<(BoundingBox, Option<CellContext>)> {
    fn walk(node: &RenderNode, out: &mut Vec<(BoundingBox, Option<CellContext>)>) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if line.caption_owner.is_some() {
                let ctx = node
                    .children
                    .iter()
                    .find_map(|child| match &child.node_type {
                        RenderNodeType::TextRun(run) => Some(run.cell_context.clone()),
                        _ => None,
                    });
                out.push((node.bbox, ctx.flatten()));
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let tree = core.build_page_tree(0).expect("쪽 트리");
    let mut out = Vec::new();
    walk(&tree.root, &mut out);
    out
}

fn image_boxes(core: &DocumentCore) -> Vec<BoundingBox> {
    fn walk(node: &RenderNode, out: &mut Vec<BoundingBox>) {
        if matches!(node.node_type, RenderNodeType::Image(_)) {
            out.push(node.bbox);
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let tree = core.build_page_tree(0).expect("쪽 트리");
    let mut out = Vec::new();
    walk(&tree.root, &mut out);
    out
}

fn floating_picture(core: &mut DocumentCore, props: &str) -> (usize, usize) {
    let result = core
        .insert_picture_native(
            0,
            0,
            0,
            &[],
            &png(),
            9600,
            9600,
            1,
            1,
            "png",
            "",
            None,
            None,
        )
        .expect("그림");
    let (para, ctrl) = (
        json_usize(&result, "paraIdx"),
        json_usize(&result, "controlIdx"),
    );
    core.set_picture_properties_native(
        0,
        para,
        ctrl,
        r#"{"treatAsChar":false,"textWrap":"InFrontOfText","horzRelTo":"Paper","vertRelTo":"Paper","horzOffset":8000,"vertOffset":12000}"#,
    )
    .expect("띄우기");
    core.set_picture_properties_native(0, para, ctrl, props)
        .expect("캡션");
    (para, ctrl)
}

fn picture(core: &DocumentCore, para: usize, ctrl: usize) -> &crate::model::image::Picture {
    match &core.document.sections[0].paragraphs[para].controls[ctrl] {
        Control::Picture(picture) => picture,
        other => panic!("그림이 아님: {other:?}"),
    }
}

#[test]
fn narrowing_a_side_caption_rewraps_its_text() {
    let mut core = blank();
    let (para, ctrl) = floating_picture(
        &mut core,
        r#"{"hasCaption":true,"captionDirection":"Right","captionVertAlign":"Top","captionWidth":8504,"captionSpacing":850}"#,
    );
    let end = core
        .get_cell_paragraph_length_native(0, para, ctrl, 0, 0)
        .expect("캡션 길이");
    core.insert_text_in_cell_native(
        0,
        para,
        ctrl,
        0,
        0,
        end,
        " 오른쪽 캡션은 너비를 줄이면 글을 다시 줄바꿈합니다.",
    )
    .expect("캡션 입력");
    let wide = picture(&core, para, ctrl)
        .caption
        .as_ref()
        .unwrap()
        .paragraphs[0]
        .line_segs
        .len();

    core.set_picture_properties_native(0, para, ctrl, r#"{"hasCaption":true,"captionWidth":2834}"#)
        .expect("캡션 크기");
    let caption = picture(&core, para, ctrl).caption.as_ref().unwrap();
    assert!(caption.paragraphs[0].line_segs.len() > wide);
    // 한컴은 줄 나눈 폭을 캡션 글 최대 폭으로 저장한다.
    assert_eq!(caption.max_width, 2834);
    let lines = caption_lines(&core);
    assert_eq!(lines.len(), caption.paragraphs[0].line_segs.len());
    assert!(lines
        .iter()
        .all(|(bbox, _)| (bbox.width - 37.8).abs() < 0.2));

    // 한컴 12.30처럼 옆 캡션은 개체 높이에서 잘라 그린다.
    let image = image_boxes(&core)[0];
    fn clips(node: &RenderNode, out: &mut Vec<Option<f64>>) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if line.caption_owner.is_some() {
                out.push(node.paint_clip.as_ref().map(|paths| {
                    paths[0]
                        .commands
                        .iter()
                        .filter_map(|command| match command {
                            crate::model::text_clip::ClipCommand::Line(_, y) => Some(*y),
                            _ => None,
                        })
                        .fold(f64::MIN, f64::max)
                }));
            }
        }
        for child in &node.children {
            clips(child, out);
        }
    }
    let mut bottoms = Vec::new();
    clips(
        &core.build_page_tree(0).expect("쪽 트리").root,
        &mut bottoms,
    );
    assert!(!bottoms.is_empty());
    for bottom in bottoms {
        let bottom = bottom.expect("옆 캡션 자름");
        assert!(
            (bottom - (image.y + image.height)).abs() < 0.5,
            "{bottom} {image:?}"
        );
    }
}

#[test]
fn include_margin_widens_a_bottom_caption_to_the_outer_margins() {
    let mut core = blank();
    let (para, ctrl) = floating_picture(
        &mut core,
        r#"{"outerMarginLeft":1500,"outerMarginRight":1500,"hasCaption":true,"captionDirection":"Bottom","captionVertAlign":"Top","captionWidth":8504,"captionSpacing":850}"#,
    );
    let image = image_boxes(&core)[0];
    let (plain, _) = caption_lines(&core).remove(0);
    assert!((plain.x - image.x).abs() < 0.1);
    assert!((plain.width - image.width).abs() < 0.1);

    core.set_picture_properties_native(
        0,
        para,
        ctrl,
        r#"{"hasCaption":true,"captionIncludeMargin":true}"#,
    )
    .expect("여백까지 너비 확대");
    let (wide, _) = caption_lines(&core).remove(0);
    // 1500 HWPUNIT = 20px(96dpi). 캡션이 왼쪽 바깥 여백부터 시작해 좌우 여백만큼 넓다.
    assert!((wide.x - (image.x - 20.0)).abs() < 0.1);
    assert!((wide.width - (image.width + 40.0)).abs() < 0.1);
    assert_eq!(
        picture(&core, para, ctrl)
            .caption
            .as_ref()
            .unwrap()
            .max_width,
        9600 + 3000
    );
}

#[test]
fn inline_picture_caption_starts_under_the_picture() {
    let mut core = blank();
    core.insert_text_native(0, 0, 0, "글자처럼 그림: 앞")
        .expect("본문");
    let result = core
        .insert_picture_native(
            0,
            0,
            8,
            &[],
            &png(),
            6000,
            6000,
            1,
            1,
            "png",
            "",
            None,
            None,
        )
        .expect("그림");
    let ctrl = json_usize(&result, "controlIdx");
    core.set_picture_properties_native(0, 0, ctrl, r#"{"treatAsChar":true}"#)
        .expect("글자처럼 취급");
    core.set_picture_properties_native(
        0,
        0,
        ctrl,
        r#"{"hasCaption":true,"captionDirection":"Bottom","captionVertAlign":"Top","captionWidth":8504,"captionSpacing":850}"#,
    )
    .expect("캡션");
    let image = image_boxes(&core)[0];
    let (line, _) = caption_lines(&core).remove(0);
    assert!(image.x > 100.0, "그림이 글 뒤에 있어야 한다: {image:?}");
    assert!((line.x - image.x).abs() < 0.1, "{line:?} {image:?}");
    assert!((line.width - image.width).abs() < 0.1, "{line:?} {image:?}");
}

#[test]
fn cell_picture_caption_lines_address_the_caption() {
    let mut core = blank();
    let table = core.create_table_native(0, 0, 0, 1, 2).expect("표");
    let table_ctrl = json_usize(&table, "controlIdx");
    let table_para = json_usize(&table, "paraIdx");
    let cell = [(table_ctrl, 0usize, 0usize)];
    let picture = core
        .insert_picture_in_cell_native(
            0,
            table_para,
            0,
            &cell,
            &png(),
            6000,
            6000,
            1,
            1,
            "png",
            "",
            None,
            None,
        )
        .expect("셀 그림");
    let inner = json_usize(&picture, "controlIdx");
    let path = format!(r#"[{{"controlIndex":{table_ctrl},"cellIndex":0,"cellParaIndex":0}}]"#);
    core.set_cell_picture_properties_by_path_native(
        0,
        table_para,
        &path,
        inner,
        r#"{"hasCaption":true,"captionDirection":"Bottom","captionVertAlign":"Top","captionWidth":8504,"captionSpacing":850}"#,
    )
    .expect("셀 그림 캡션");
    let lines = caption_lines(&core);
    assert!(!lines.is_empty());
    let ctx = lines[0].1.as_ref().expect("셀 문맥");
    let steps: Vec<_> = ctx
        .path
        .iter()
        .map(|entry| (entry.control_index, entry.cell_index, entry.cell_para_index))
        .collect();
    assert_eq!(steps, vec![(table_ctrl, 0, 0), (inner, 0, 0)]);
}
