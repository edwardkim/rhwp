//! 관계형·고정형 검사. 그림과 캡션의 줄·여백·셀 소유 및 합성 입력에서 그림의 시작 영역을 검사한다.
//! 주석 갱신 2026-11-21T15:21:50+09:00. 직전 파일 변경 82f43831f492beb7f3ec8c3d4cde946225b0f150 2026-11-21T10:04:32+09:00.
//! 2026-11-21 한컴 2020 독립 인쇄본과 96 DPI에서 색상 경계 보정 전 2px 실루엣을 측정했다.
//! 아래는 쪽 순서대로 Native Skia와 fresh WASM Canvas2D의 raw 일치율(%)이다.
//! `caption-captioned_picture_inside_a_caption_survives_hwp_round_trip.hwp` 1쪽. Native [100.00000], WASM [100.00000].
//! `caption-cell_picture_caption_lines_address_the_caption.hwp` 1쪽. Native [99.60976], WASM [99.90272].
//! `caption-header-false-false.hwp` 1쪽. Native [100.00000], WASM [100.00000].
//! `caption-header-true-false.hwp` 1쪽. Native [100.00000], WASM [100.00000].
//! `caption-header-true-true.hwp` 1쪽. Native [100.00000], WASM [100.00000].
//! `caption-include_margin_widens_a_bottom_caption_to_the_outer_margins.hwp` 1쪽. Native [100.00000], WASM [100.00000].
//! `caption-inline_picture_caption_starts_under_the_picture.hwp` 1쪽. Native [100.00000], WASM [100.00000].
//! `caption-narrowing_a_side_caption_rewraps_its_text.hwp` 1쪽. Native [99.53757], WASM [99.59184].
//! 이 범위의 전쪽 최저는 Native 99.53757%, WASM 99.59184%다.
//! 입력·PDF 해시와 쪽별 근거는 `mydocs/pr/assets/semanticist21-20261121/captions/pages.tsv`에 있다.
//!
//! 캡션 너비 변경 뒤 다시 줄 나누기, '여백 부분까지 너비 확대',
//! 글자처럼 취급 그림과 셀 안 그림의 캡션 자리를 한컴 12.30 기준으로 검사한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::layout::CellContext;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

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
    let tree = core.build_page_render_tree(0).expect("쪽 트리");
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
    let tree = core.build_page_render_tree(0).expect("쪽 트리");
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

fn picture(core: &DocumentCore, para: usize, ctrl: usize) -> &rhwp::model::image::Picture {
    match &core.document().sections[0].paragraphs[para].controls[ctrl] {
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
                            rhwp::model::text_clip::ClipCommand::Line(_, y) => Some(*y),
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
        &core.build_page_render_tree(0).expect("쪽 트리").root,
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

    // 한컴 12.30처럼 셀이 아래 캡션까지 커져 캡션 글줄이 셀 안에 든다.
    let cell_box = first_cell_box(&core);
    let last = lines.last().expect("캡션 글줄").0;
    assert!(
        last.y + last.height <= cell_box.y + cell_box.height + 0.5,
        "{last:?} {cell_box:?}"
    );
    // 캡션 글줄을 누르면 그 그림 캡션 자리가 된다.
    let hit = core
        .hit_test_native(0, last.x + 2.0, last.y + last.height / 2.0)
        .expect("누르기");
    let hit: serde_json::Value = serde_json::from_str(&hit).expect("JSON");
    let path = hit["cellPath"].as_array().expect("셀 경로");
    assert_eq!(path.len(), 2, "{hit}");
    assert_eq!(path[1]["controlIndex"], inner);
}

fn first_cell_box(core: &DocumentCore) -> BoundingBox {
    fn walk(node: &RenderNode) -> Option<BoundingBox> {
        if matches!(node.node_type, RenderNodeType::TableCell(_)) {
            return Some(node.bbox);
        }
        node.children.iter().find_map(walk)
    }
    walk(&core.build_page_render_tree(0).expect("쪽 트리").root).expect("셀")
}

#[test]
fn header_picture_caption_line_hits_its_caption() {
    header_picture_caption(false, false);
}

/// 한컴 12.30은 머리말에 넣은 그림을 글자처럼 취급하고 그 아래 캡션을 그림 바로 아래에 그린다.
#[test]
fn header_inline_picture_caption_is_drawn_and_hits_its_caption() {
    header_picture_caption(true, false);
}

/// 한컴 12.30처럼 머리말 그림에도 캡션을 새로 단다.
#[test]
fn header_picture_gets_a_new_caption() {
    header_picture_caption(true, true);
}

fn header_picture_caption(treat_as_char: bool, new_caption: bool) {
    const CAPTION: &str = r#""hasCaption":true,"captionDirection":"Bottom","captionVertAlign":"Top","captionWidth":8504,"captionSpacing":850"#;
    let mut core = blank();
    let (para, ctrl) = floating_picture(
        &mut core,
        &if new_caption {
            format!(r#"{{"treatAsChar":{treat_as_char}}}"#)
        } else {
            format!(r#"{{"treatAsChar":{treat_as_char},{CAPTION}}}"#)
        },
    );
    core.copy_control_native(0, para, &[], ctrl).unwrap();
    core.create_header_footer_native(0, true, 0)
        .expect("머리말");
    let info: serde_json::Value = serde_json::from_str(
        &core
            .get_header_footer_native(0, true, 0)
            .expect("머리말 정보"),
    )
    .expect("JSON");
    let host = info["paraIndex"].as_u64().expect("paraIndex") as usize;
    let hf = info["controlIndex"].as_u64().expect("controlIndex") as usize;
    // 한컴 12.30에서 머리말에 넣은 그림처럼, 캡션 달린 그림을 머리말 첫 문단 맨 앞 컨트롤로 둔다.
    core.paste_internal_in_cell_by_path_native(0, host, &[(hf, 0, 0)], 0)
        .expect("머리말에 그림 붙여넣기");
    if new_caption {
        core.set_header_footer_picture_properties_native(
            0,
            host,
            hf,
            0,
            0,
            &format!("{{{CAPTION}}}"),
        )
        .expect("머리말 그림 캡션 넣기");
    }

    // 머리말 캡션 글줄의 셀 문맥은 머리말 표 칸처럼 머리말 문단이 뿌리인 [(그림, 0, 0)]이다.
    // 머리말 글줄은 본문 캡션 소유자(caption_owner)를 싣지 않으므로 글자 조각의 셀 문맥으로 찾는다.
    fn header_caption(node: &RenderNode) -> Option<(BoundingBox, CellContext)> {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let ctx = node
                .children
                .iter()
                .find_map(|child| match &child.node_type {
                    RenderNodeType::TextRun(run) => run.cell_context.clone(),
                    _ => None,
                });
            if let Some(ctx) = ctx.filter(|ctx| ctx.path.len() == 1) {
                return Some((node.bbox, ctx));
            }
        }
        node.children.iter().find_map(header_caption)
    }
    let tree = core.build_page_render_tree(0).expect("쪽 트리");
    let header_node = tree
        .root
        .children
        .iter()
        .find(|child| matches!(child.node_type, RenderNodeType::Header))
        .expect("머리말 노드");
    let (line, ctx) = header_caption(header_node).expect("머리말 캡션 글줄");
    assert_eq!(ctx.parent_para_index, 0);
    let steps: Vec<_> = ctx
        .path
        .iter()
        .map(|entry| (entry.control_index, entry.cell_index, entry.cell_para_index))
        .collect();
    assert_eq!(steps, vec![(0, 0, 0)]);

    // 머리말 편집에서 그 글줄을 누르면 그림 캡션 자리가 된다.
    let hit = core
        .hit_test_in_header_footer_target_native(
            0,
            0,
            true,
            0,
            line.x + 2.0,
            line.y + line.height / 2.0,
        )
        .expect("누르기");
    let hit: serde_json::Value = serde_json::from_str(&hit).expect("JSON");
    let path = hit["cellPath"].as_array().expect("셀 경로");
    assert_eq!(path.len(), 2, "{hit}");
    assert_eq!(hit["parentParaIndex"], host);
    assert_eq!(path[0]["controlIndex"], hf);
    assert_eq!(path[1]["controlIndex"], 0);
}

/// 한컴 12.30은 캡션 글에 캡션 달린 그림을 넣을 수 있다. HWP로 저장해 다시 열어도 안쪽 그림은
/// 그림과 그 캡션으로 남는다(바깥 그림의 캡션이 안쪽 그림의 SHAPE_COMPONENT 앞에서 잘리면 안쪽
/// 그림이 사각형 글상자가 되고, 한컴은 그 파일을 열지 못한다).
#[test]
fn captioned_picture_inside_a_caption_survives_hwp_round_trip() {
    let caption = r#"{"hasCaption":true,"captionDirection":"Bottom","captionVertAlign":"Top","captionWidth":8504,"captionSpacing":850}"#;
    let mut core = blank();
    let (para, ctrl) = floating_picture(&mut core, caption);
    core.set_picture_properties_native(0, para, ctrl, r#"{"treatAsChar":true}"#)
        .unwrap();
    core.copy_control_native(0, para, &[], ctrl).unwrap();
    core.set_picture_properties_native(0, para, ctrl, r#"{"treatAsChar":false}"#)
        .unwrap();
    let end = core
        .get_cell_paragraph_length_native(0, para, ctrl, 0, 0)
        .unwrap();
    core.paste_internal_in_cell_by_path_native(0, para, &[(ctrl, 0, 0)], end)
        .unwrap();
    let reopened = DocumentCore::from_bytes(&core.export_hwp_native().expect("HWP")).expect("열기");
    let Control::Picture(outer) = &reopened.document().sections[0].paragraphs[para].controls[ctrl]
    else {
        panic!("바깥 그림이 아니다");
    };
    let controls = &outer.caption.as_ref().expect("바깥 캡션").paragraphs[0].controls;
    let inner = controls
        .iter()
        .find_map(|control| match control {
            Control::Picture(pic) => Some(pic),
            _ => None,
        })
        .expect("캡션 안 그림");
    assert!(inner.caption.is_some(), "안쪽 그림 캡션");
}
