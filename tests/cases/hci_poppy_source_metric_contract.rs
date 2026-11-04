//! 관계형·고정형: 합성 문자열의 원본 글꼴 전진폭 비율과 독립 폭 관측을 검사한다.
//! 주석 갱신: 2026-11-01T11:28:30+09:00; 직전 파일 커밋: 2026-10-31T23:53:29+09:00 (af175315e7e16800e65e573c2a2ac23e8d9da82b).
//! 합성 문자 경계와 원본 프로그램 단위의 폭을 비교하며 실물 쪽의 시각 일치율을 판정하지 않는다.
//! 근거: mydocs/pr/assets/issue_8160/poppy-original-widths.json, mydocs/pr/assets/issue_8160/poppy-ascii-neutral-advances.tsv.
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

#[test]
fn unresolved_original_bold_program_keeps_its_advance_with_a_display_fallback() {
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    use rhwp::renderer::TextStyle;

    // 원본 HCI Poppy 굵은 I/D/C의 독립 프로그램 폭은 각각 199/426/370 ÷ 512em이다.
    // 표시 대체만 Palatino인 유형 미확정 원본에 보통 TTF 폭을 적용하지 않는다.
    let source = TextStyle {
        font_family: "Palatino Linotype".into(),
        font_size: 26.666666666666668,
        bold: true,
        original_glyph_metric_family: Some("HCI Poppy".to_string().into()),
        ..Default::default()
    };
    let measure = |style: &TextStyle| EmbeddedTextMeasurer.compute_char_positions("IDC", style);
    let positions = measure(&source);
    for (index, expected) in [199.0 / 512.0, 426.0 / 512.0, 370.0 / 512.0]
        .into_iter()
        .enumerate()
    {
        let em = (positions[index + 1] - positions[index]) / source.font_size;
        assert!(
            (em - expected).abs() < 0.002,
            "원본 굵은 전진 {index}: 실제={em}, 독립 원본={expected}"
        );
    }

    // 명시 TTF는 기존 #8079 계약을 유지한다. 글꼴 설치에 의존하지 않는다.
    let explicit = TextStyle {
        source_font_is_ttf: true,
        ..source.clone()
    };
    let regular = TextStyle {
        bold: false,
        ..explicit.clone()
    };
    assert_eq!(measure(&explicit), measure(&regular));
}

fn find<'a>(node: &'a RenderNode, text: &str) -> Option<&'a TextRunNode> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == text {
            return Some(run);
        }
    }
    node.children.iter().find_map(|child| find(child, text))
}

fn positions(face: &str, text: &str, size: i32, spacing: i8, bold: bool, italic: bool) -> Vec<f64> {
    let alt_type = if face == "HCI Poppy" { 2 } else { 1 };
    positions_as(face, alt_type, text, size, spacing, bold, italic)
}

/// `alt_type` — 선언 face 종류(1 TrueType · 2 HFT).
fn positions_as(
    face: &str,
    alt_type: u8,
    text: &str,
    size: i32,
    spacing: i8,
    bold: bool,
    italic: bool,
) -> Vec<f64> {
    let mut document = Document::default();
    document.doc_properties.section_count = 1;
    document.doc_info.font_faces = vec![
        vec![Font {
            name: face.to_owned(),
            alt_type,
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
            // [#8079] TrueType 로 선언된 굵은 글자는 보통 면 전진폭을 쓰므로, 표시 face 경로는
            // 같은 선언 종류(HFT)로 비교한다 — 원본 Type3 표를 쓰는지만 가른다.
            let display = positions_as("Palatino Linotype", 2, "O/O", size, 0, bold, true);
            assert_eq!(
                source, display,
                "확인하지 않은 기울임체는 기존 표시 face 경로 유지"
            );
        }
    }
}
