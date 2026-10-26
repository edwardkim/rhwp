//! 원본 HFT 전진폭과 표시용 대체 face를 구분하는 합성 계약이다.
//! 독립 Print Type3 T3의 /Widths: O=785, /=605, FontMatrix=0.001.
//! 실물 쪽의 좌표·이미지·baseline을 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{
    document::{Document, Section},
    paragraph::{CharShapeRef, Paragraph},
    style::{CharShape, Font, ParaShape},
};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode};
use rhwp::DocumentCore;

fn find<'a>(node: &'a RenderNode, text: &str) -> Option<&'a TextRunNode> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == text {
            return Some(run);
        }
    }
    node.children.iter().find_map(|child| find(child, text))
}

fn positions(face: &str, text: &str, size: i32, spacing: i8, bold: bool, italic: bool) -> Vec<f64> {
    let mut document = Document::default();
    document.doc_properties.section_count = 1;
    document.doc_info.font_faces = vec![
        vec![Font {
            name: face.to_owned(),
            alt_type: if face == "HCI Poppy" { 2 } else { 1 },
            ..Default::default()
        }];
        7
    ];
    document.doc_info.char_shapes.push(CharShape {
        base_size: size,
        spacings: [spacing; 7],
        bold,
        italic,
        ..Default::default()
    });
    document.doc_info.para_shapes.push(ParaShape::default());
    let mut section = Section::default();
    section.paragraphs.push(Paragraph {
        text: text.to_owned(),
        char_count: text.len() as u32 + 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        ..Default::default()
    });
    document.sections.push(section);
    let bytes = rhwp::serializer::hwpx::serialize_hwpx(&document).expect("합성 원본 HWPX");
    let core = DocumentCore::from_bytes(&bytes).expect("공개 파싱·조판 경로");
    let page = core.build_page_render_tree(0).expect("공개 렌더 트리");
    let run = find(&page.root, text).expect("원문을 보존한 run");
    assert!(run.style.font_family.contains("Palatino"));
    let layout: serde_json::Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).expect("공개 문자 경계"))
            .expect("layout JSON");
    layout["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .find(|run| run["text"].as_str() == Some(text))
        .expect("원문을 보존한 공개 run")["charX"]
        .as_array()
        .expect("charX")
        .iter()
        .map(|v| v.as_f64().expect("문자 경계"))
        .collect()
}

fn middle_to_first_advance(values: &[f64]) -> f64 {
    assert_eq!(values.len(), 4);
    assert!(values.windows(2).all(|pair| pair[1] > pair[0]));
    (values[2] - values[1]) / (values[1] - values[0])
}

#[test]
fn hci_poppy_regular_uses_original_advance_without_changing_display_font() {
    for size in [1000, 2000] {
        for spacing in [0, -20] {
            let source = positions("HCI Poppy", "O/O", size, spacing, false, false);
            let ratio = middle_to_first_advance(&source);
            assert!(
                // 공개 charX는 소수 한 자리로 직렬화된다. 그 양자화 여유이며
                // 특정 쪽의 절대 위치를 고정하는 허용치가 아니다.
                (ratio - 605.0 / 785.0).abs() < 0.02,
                "원본 Type3 글자 전진의 비율: size={size}, spacing={spacing}, ratio={ratio}"
            );
            let display = positions("Palatino Linotype", "O/O", size, spacing, false, false);
            assert!(
                middle_to_first_advance(&display) < ratio,
                "실제 TTF 폭은 유지한다"
            );
        }
    }
}

#[test]
fn hci_poppy_measured_ascii_styles_use_source_advances_and_italic_keeps_display_metric() {
    // 원본 HFT em=512와 새 독립 Print의 12·20·52pt 전진 관측을 교차 확인했다.
    // mydocs/pr/assets/issue_8160/poppy-original-widths.json 및
    // poppy-ascii-neutral-advances.tsv. 기대값은 renderer 메트릭 표에서 읽지 않는다.
    for size in [1200, 2000, 5200] {
        for (bold, text, widths) in [
            (false, "O!O", [402, 142, 402]),
            (true, "O/O", [426, 151, 426]),
            (true, "O!O", [426, 142, 426]),
        ] {
            let source = positions("HCI Poppy", text, size, 0, bold, false);
            assert_eq!(source.len(), widths.len() + 1);
            let scale = f64::from(size) / 100.0 * 96.0 / 72.0 / 512.0;
            let mut expected = 0.0;
            for (index, actual) in source.iter().enumerate() {
                // 공개 charX의 0.1px 직렬화와 원본 전진의 1800dpi 격자 정밀도.
                assert!(
                    (actual - expected).abs() <= 0.11,
                    "확인한 원본 전진: size={size}, bold={bold}, text={text}, index={index}, actual={actual}, expected={expected}"
                );
                if let Some(width) = widths.get(index) {
                    expected += f64::from(*width) * scale;
                }
            }
        }
        for bold in [false, true] {
            let source = positions("HCI Poppy", "O/O", size, 0, bold, true);
            let display = positions("Palatino Linotype", "O/O", size, 0, bold, true);
            assert_eq!(
                source, display,
                "확인하지 않은 기울임체는 기존 표시 face 경로 유지"
            );
        }
    }
}
