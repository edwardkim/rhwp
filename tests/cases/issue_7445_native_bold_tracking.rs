//! 원본 프로그램의 굵은 글자 전진 간격과 자연 글리프 폭을 분리한다.
//! 독립 issue265 Print12쪽 T2/Widths의 한글은1000/1000em, TJ는약0.05em 추가 전진.
//! 원문 소유·내용 관계와 독립 Print 3·12쪽의 ±2CSSpx 좌표를 함께 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    paint::{LayerJsonOptions, RenderProfile},
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};
use serde_json::Value;

fn core() -> DocumentCore {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/issue_265.hwp")).expect("원본 HWP3"),
    )
    .unwrap();
    core.set_font_environment(Some(
        FontEnvironment::from_json(r#"{"id":"paired-empty-no-substitution","substitutions":{}}"#)
            .unwrap(),
    ))
    .unwrap();
    core
}

fn paragraph_runs<'a>(node: &'a RenderNode, para_index: usize, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.section_index == Some(0) && run.para_index == Some(para_index))
    {
        out.push(node);
    }
    for child in &node.children {
        paragraph_runs(child, para_index, out);
    }
}

fn text(node: &RenderNode) -> &TextRunNode {
    let RenderNodeType::TextRun(run) = &node.node_type else {
        panic!("원문 글줄");
    };
    run
}

fn source_ops<'a>(value: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(object) => {
            if value["type"] == "textRun" && value["source"]["stableSourceKey"] == key {
                out.push(value);
            }
            for child in object.values() {
                source_ops(child, key, out);
            }
        }
        Value::Array(array) => {
            for child in array {
                source_ops(child, key, out);
            }
        }
        _ => {}
    }
}

#[test]
fn native_bold_title_keeps_independent_origins_and_source_order() {
    let core = core();
    assert_title(&core);
    let bytes = core.export_hwpx_native().expect("HWPX 저장");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).unwrap();
    let mut section = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("Contents/section0.xml").unwrap(),
        &mut section,
    )
    .unwrap();
    assert!(
        !roxmltree::Document::parse(&section)
            .unwrap()
            .descendants()
            .filter(|node| node.has_tag_name(("http://www.hancom.co.kr/hwpml/2011/paragraph", "t")))
            .filter_map(|node| node.text())
            .any(|text| text.contains('\u{fffc}')),
        "이 원본의 제어문자 표식은 구조/그림 소유로 보존하며 인쇄할 OBJ 글자를 만들지 않는다"
    );
    let hwp = core.export_hwp_with_adapter().expect("HWP5 저장");
    for (format, saved) in [("HWPX", &bytes), ("HWP5", &hwp)] {
        let mut reparsed = DocumentCore::from_bytes(saved).expect("저장본 재열람");
        reparsed
            .set_font_environment(core.font_environment().cloned())
            .unwrap();
        assert_eq!(
            core.document()
                .doc_info
                .font_faces
                .iter()
                .map(Vec::len)
                .collect::<Vec<_>>(),
            reparsed
                .document()
                .doc_info
                .font_faces
                .iter()
                .map(Vec::len)
                .collect::<Vec<_>>(),
            "{format} 언어별 글꼴 소유"
        );
        for (before, after) in core
            .document()
            .doc_info
            .font_faces
            .iter()
            .flatten()
            .zip(reparsed.document().doc_info.font_faces.iter().flatten())
        {
            assert_eq!(
                before.native_program, after.native_program,
                "{format} 출처 왕복"
            );
        }
        assert_title(&reparsed);
    }
    // 프로그램 출처를 일반 HWPX의 글꼴명으로 추정하면 안 된다.
    // 외부 편집/엔트리 제거/알 수 없는 버전은 원래 글꼴 선언을 그대로 읽는다.
    for mutation in ["header", "remove", "version"] {
        let changed = change_program_metadata(&bytes, mutation);
        let doc = rhwp::parser::parse_document(&changed).unwrap();
        assert!(
            doc.doc_info
                .font_faces
                .iter()
                .flatten()
                .all(|font| font.native_program.is_none()),
            "{mutation}"
        );
        assert_eq!(doc.sections[0].paragraphs[121].text, para_text(&core));
    }
    for mutation in [
        "header",
        "remove",
        "version",
        "groups",
        "malformed",
        "oversized",
    ] {
        let changed = change_hwp_program_metadata(&hwp, mutation);
        let doc =
            rhwp::parser::parse_document(&changed).expect("보조 출처가 잘못돼도 표준 문서는 연다");
        assert!(
            doc.doc_info
                .font_faces
                .iter()
                .flatten()
                .all(|font| font.native_program.is_none()),
            "HWP5 {mutation}의 보조 출처를 추정해 적용하지 않는다"
        );
        assert_eq!(doc.sections[0].paragraphs[121].text, para_text(&core));
        assert!(doc
            .extra_streams
            .iter()
            .all(|(path, _)| path != "/RhwpNativeFontPrograms"));
    }
}

fn change_hwp_program_metadata(bytes: &[u8], mutation: &str) -> Vec<u8> {
    use std::io::{Cursor, Read, Write};
    let mut cfb = cfb::CompoundFile::open(Cursor::new(bytes.to_vec())).unwrap();
    let path = "/RhwpNativeFontPrograms";
    if mutation == "remove" {
        cfb.remove_stream(path).unwrap();
    } else {
        let mut contents = Vec::new();
        cfb.open_stream(path)
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        let mut value: Value = serde_json::from_slice(&contents).unwrap();
        match mutation {
            "header" => {
                value["header_sha256"][0] = (value["header_sha256"][0].as_u64().unwrap() ^ 1).into()
            }
            "version" => value["version"] = 2.into(),
            "groups" => {
                value["programs"].as_array_mut().unwrap().pop();
            }
            "malformed" | "oversized" => {}
            _ => panic!("알 수 없는 변형"),
        }
        contents = match mutation {
            "malformed" => b"{".to_vec(),
            "oversized" => vec![b' '; 1024 * 1024 + 1],
            _ => serde_json::to_vec(&value).unwrap(),
        };
        cfb.create_stream(path)
            .unwrap()
            .write_all(&contents)
            .unwrap();
    }
    cfb.into_inner().into_inner()
}

fn para_text(core: &DocumentCore) -> &str {
    &core.document().sections[0].paragraphs[121].text
}

fn change_program_metadata(bytes: &[u8], mutation: &str) -> Vec<u8> {
    use std::io::{Cursor, Read, Write};
    let mut source = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut destination = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        let name = entry.name().unwrap().to_string();
        let mut contents = Vec::new();
        entry.read_to_end(&mut contents).unwrap();
        if name == "Contents/header.xml" && mutation == "header" {
            contents.extend_from_slice(b"<!-- external edit -->");
        }
        if name == "META-INF/rhwp-native-font-programs.json" {
            if mutation == "remove" {
                continue;
            }
            if mutation == "version" {
                let mut value: Value = serde_json::from_slice(&contents).unwrap();
                value["version"] = 2.into();
                contents = serde_json::to_vec(&value).unwrap();
            }
        }
        destination
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        destination.write_all(&contents).unwrap();
    }
    destination.finish().unwrap().into_inner()
}

fn assert_title(core: &DocumentCore) {
    let rhwp::model::control::Control::Picture(picture) =
        &core.document().sections[0].paragraphs[41].controls[0]
    else {
        panic!("원문 그림의 소유 문단");
    };
    assert!(
        picture.common.flow_with_text && picture.common.allow_overlap,
        "독립 한컴 HWP3→HWP5 변환의 문단 앵커 계약: 그림과 호스트의 빈 줄을 함께 보존한다"
    );
    assert_eq!(
        picture.common.attr & ((1 << 13) | (1 << 14)),
        (1 << 13) | (1 << 14),
        "IR과 저장 속성의 앵커 계약 일치"
    );
    let following_page = core.build_page_render_tree(2).unwrap();
    let mut following = Vec::new();
    paragraph_runs(&following_page.root, 43, &mut following);
    let start = following
        .iter()
        .find(|node| text(node).char_start == Some(0))
        .expect("그림 뒤 본문의 첫 원문 문자");
    assert!(text(start).text.starts_with("실제 서버들은 고속"));
    let compact = |s: &str| {
        s.chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>()
    };
    assert_eq!(
        compact(
            &following
                .iter()
                .map(|node| text(node).text.as_str())
                .collect::<String>()
        ),
        compact(&core.document().sections[0].paragraphs[43].text),
        "그림 뒤 본문의 문자를 같은 쪽에서 한 번씩 원래 순서로 소유한다"
    );
    // 원본 Print p3의 첫 글리프 원점(72pt→96CSSpx). 한컴의 직접 HWP5 변환과
    // 두 저장 비트만 복원한 독립 Print도 같은 위치를 확인한다.
    for (actual, expected) in [
        (start.bbox.x, 113.4400024414),
        (start.bbox.y + text(start).baseline, 708.1599934896),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "그림 뒤 본문 독립 Print±2CSSpx: {actual} vs {expected}"
        );
    }
    let para = &core.document().sections[0].paragraphs[121];
    let source = para.text.clone();
    assert_eq!(source, "4. 작업 할당 방식(Scheduling Algorithms)");
    for reference in &para.char_shapes {
        let shape = &core.document().doc_info.char_shapes[reference.char_shape_id as usize];
        assert_eq!(shape.spacings, [0; 7], "원본 자간0을 고쳐 맞추지 않는다");
    }
    let fonts = serde_json::to_value(&core.document().doc_info.font_faces).unwrap();
    assert!(
        !fonts.to_string().contains("native_program"),
        "프로그램 출처는 원본 속성으로 직렬화하지 않는다"
    );
    assert_eq!(core.page_count(), 16);
    let tree = core.build_page_render_tree(11).unwrap();
    let mut nodes = Vec::new();
    paragraph_runs(&tree.root, 121, &mut nodes);
    assert_eq!(
        nodes
            .iter()
            .map(|node| text(node).text.as_str())
            .collect::<String>(),
        source
    );
    let korean: Vec<_> = nodes
        .iter()
        .filter(|node| text(node).char_start == Some(3))
        .collect();
    let english: Vec<_> = nodes
        .iter()
        .filter(|node| text(node).char_start == Some(12))
        .collect();
    assert_eq!(korean.len(), 1);
    assert_eq!(english.len(), 1);
    let (korean, english) = (korean[0], english[0]);
    assert_eq!(text(korean).text, "작업 할당 방식(");
    assert_eq!(text(english).text, "Scheduling Algorithms)");
    assert!(text(korean).style.bold && text(english).style.bold);
    assert_eq!(
        text(korean).style.font_family,
        text(english).style.font_family
    );
    assert_eq!(text(korean).style.letter_spacing, 0.0);
    assert_eq!(text(english).style.letter_spacing, 0.0);
    assert!(korean.bbox.x < english.bbox.x);
    assert!((korean.bbox.x + korean.bbox.width - english.bbox.x).abs() < 0.1);
    assert!(
        (korean.bbox.y + text(korean).baseline - english.bbox.y - text(english).baseline).abs()
            < 0.1
    );
    // 독립 Print72pt→96CSSpx. 원문 문단·scalar로 식별하며 좌표 영역으로 고르지 않는다.
    for (actual, expected) in [
        (korean.bbox.x, 145.7599995931),
        (english.bbox.x, 331.6799926758),
        (english.bbox.y + text(english).baseline, 152.7999979655),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print±2CSSpx: {actual} vs {expected}"
        );
    }
    let json: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                11,
                RenderProfile::Print,
                LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    let mut ops = Vec::new();
    source_ops(&json["root"], "section:0/para:121/char:12", &mut ops);
    assert_eq!(ops.len(), 1, "최종 paint의 영문 소유");
    let op = ops[0];
    let last = op["clusters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cluster| cluster["textRangeUtf16"]["start"] == 21)
        .expect("마지막 닫는 괄호의 원문 소속");
    let closing_x =
        op["placement"]["runToPage"]["e"].as_f64().unwrap() + last["origin"]["x"].as_f64().unwrap();
    assert!(
        (closing_x - 609.9208).abs() <= 2.0,
        "독립 Print의 끝 글자까지 누적 간격 보존"
    );
    assert_eq!(
        serde_json::to_value(&core.document().doc_info.font_faces).unwrap(),
        fonts
    );
    assert_eq!(core.document().sections[0].paragraphs[121].text, source);
}

#[cfg(feature = "native-skia")]
#[test]
fn native_bold_tracking_does_not_stretch_the_actual_glyph() {
    use rhwp::{
        paint::{LayerNode, PageLayerTree, PaintOp},
        renderer::{
            layer_renderer::RasterRenderOptions, render_tree::BoundingBox, skia::SkiaLayerRenderer,
        },
    };
    let core = core();
    let tree = core.build_page_render_tree(11).unwrap();
    let mut nodes = Vec::new();
    paragraph_runs(&tree.root, 121, &mut nodes);
    let source = text(
        nodes
            .into_iter()
            .find(|node| text(node).text == "작업 할당 방식(")
            .unwrap(),
    );
    let render = |natural: bool| {
        let mut run = source.clone();
        run.text = "작".into();
        run.layout_positions = None;
        if natural {
            run.style.metric_font_family = None;
        }
        let layer = PageLayerTree::new(
            64.0,
            48.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 48.0),
                None,
                vec![PaintOp::text_run(
                    BoundingBox::new(8.0, 8.0, 40.0, 32.0),
                    run,
                )],
            ),
        );
        SkiaLayerRenderer::new()
            .render_pixels_with_options(
                &layer,
                RasterRenderOptions {
                    scale: 4.0,
                    ..Default::default()
                },
            )
            .unwrap()
    };
    let natural = render(true);
    let tracked = render(false);
    assert!(natural.bytes.chunks_exact(4).any(|pixel| pixel[3] > 0));
    let changed = natural
        .bytes
        .chunks_exact(4)
        .zip(tracked.bytes.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        changed, 0,
        "추가 전진과 독립적으로 자연 글리프 모양을 유지한다"
    );
}
