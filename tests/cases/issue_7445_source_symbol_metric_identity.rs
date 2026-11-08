//! 공개 글꼴/원본 기호의 프로그램 계약과 완료 페이지의 독립 Print 원점 보호.
use rhwp::renderer::{
    composer::{effective_text_for_metrics, ComposedTextRun},
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    TextStyle,
};

#[test]
fn declared_hft_bold_ascii_keeps_natural_width_and_separate_tracking() {
    // 독립 ASCII95 × 고딕/명조/산세리프 ×10/20pt Print:
    // 일반·굵게 /Widths500, 굵은 비공백 원점만 추가0.05em이다.
    // C144의 실제 Native/fresh WASM 원본 전64/151쪽도 모두90% 이상이다.
    let measurer = EmbeddedTextMeasurer;
    for face in ["고딕", "명조", "산세리프"] {
        let style = TextStyle {
            font_family: "unmeasured-display-program".into(),
            font_size: 16.0,
            bold: true,
            original_glyph_metric_family: Some(face.to_string().into()),
            ..Default::default()
        };
        let text: String = (' '..='~').collect();
        let positions = measurer.compute_char_positions(&text, &style);
        assert_eq!(positions.len(), 96);
        for (index, ch) in text.chars().enumerate() {
            let expected = if ch == ' ' { 8.0 } else { 8.8 };
            assert!(
                (positions[index + 1] - positions[index] - expected).abs() < 1e-8,
                "독립 원본 프로그램 {face} {ch:?}: {positions:?}"
            );
        }
        let regular = TextStyle {
            bold: false,
            ..style
        };
        let positions = measurer.compute_char_positions("iW. ", &regular);
        assert_eq!(positions, vec![0.0, 8.0, 16.0, 24.0, 32.0]);
    }
    // 실제 공개 TTF 프로그램 선택에는 원본 HFT 부분 메트릭·추적을 이식하지 않는다.
    let selected = TextStyle {
        font_family: "RHWP Public Batang".into(),
        font_size: 16.0,
        bold: true,
        font_metric_trusted: true,
        ..Default::default()
    };
    let positions = measurer.compute_char_positions("iW", &selected);
    assert!(
        positions[1] < positions[2] - positions[1],
        "공개 TTF의 비례 폭 유지"
    );
}

#[test]
fn source_pua_projection_keeps_original_advance() {
    // 독립 Print의 원본 HNC 선은 반각, 별도 Unicode 선 표본은 전각이다.
    let style = TextStyle {
        font_size: 16.0,
        font_family: "unmeasured-display-program".into(),
        original_glyph_metric_family: Some("명조".to_string().into()),
        ..Default::default()
    };
    // 12pt는 반각도1/1800인치 장치 격자에 정확히 놓여 프로그램 폭만 분리한다.
    let measurer = EmbeddedTextMeasurer;
    for source in ['\u{F080F}', '\u{A80F}'] {
        let positions = measurer.compute_char_positions(&source.to_string(), &style);
        assert!(
            (positions[1] / style.font_size - 0.5).abs() < 1e-9,
            "source HNC half-em: {source:?} -> {positions:?}"
        );
    }
    let literal = measurer.compute_char_positions("─", &style);
    assert!(
        (literal[1] / style.font_size - 1.0).abs() < 1e-9,
        "literal Unicode keeps its own full-em program"
    );
}

#[test]
fn source_pua_projection_preserves_metric_identity_and_field_values() {
    let symbol = ComposedTextRun {
        text: "\u{F080F}\u{F080F}".into(),
        display_text: Some("──".into()),
        ..Default::default()
    };
    assert_eq!(
        effective_text_for_metrics(&symbol),
        symbol.text,
        "one-to-one display projection keeps source program identity"
    );
    let field = ComposedTextRun {
        text: "\u{0015}".into(),
        display_text: Some("42".into()),
        ..Default::default()
    };
    assert_eq!(
        effective_text_for_metrics(&field),
        "42",
        "resolved field is measured by its visible value"
    );
}

#[test]
fn source_pua_projection_reaches_the_final_replay_positions() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    let mut core = rhwp::DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "\u{F080F}\u{F080F}")
        .unwrap();
    let mut doc = core.document().clone();
    for fonts in &mut doc.doc_info.font_faces {
        for face in fonts {
            face.name = "명조".into();
            face.alt_type = 1;
            face.is_embedded = false;
        }
    }
    for shape in &mut doc.doc_info.char_shapes {
        shape.base_size = 1200;
        shape.ratios = [100; 7];
        shape.relative_sizes = [100; 7];
        shape.spacings = [0; 7];
        shape.bold = false;
        shape.italic = false;
    }
    core.set_document(doc);
    let tree = core.build_page_render_tree(0).unwrap();
    fn collect<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(&n.node_type,RenderNodeType::TextRun(r) if r.text.contains('\u{F080F}')) {
            out.push(n);
        }
        for child in &n.children {
            collect(child, out);
        }
    }
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    assert_eq!(
        nodes.len(),
        1,
        "both source symbols remain in one run without omission/duplication"
    );
    let n = nodes[0];
    let RenderNodeType::TextRun(run) = &n.node_type else {
        unreachable!()
    };
    assert_eq!(run.text, "\u{F080F}\u{F080F}");
    assert_eq!(run.display_or_text(), "──");
    let positions = run
        .layout_positions
        .as_ref()
        .expect("source advance must reach every display replay backend");
    assert_eq!(positions.len(), 3);
    assert!((positions[1] / run.style.font_size - 0.5).abs() < 1e-9);
    assert!((positions[2] / run.style.font_size - 1.0).abs() < 1e-9);
    assert!(
        (positions[2] - n.bbox.width).abs() < 1e-9,
        "bbox and actual replay use the same source program result"
    );
}

#[test]
fn public_batang_metric_alias_matches_the_supplied_program() {
    // OFL 배포본을 이름만 분리한 프로그램이다. glyph/advance/cmap은 원본과 같다.
    // 이름이 새로워졌다는 이유로 일반 반각 추정에 빠지면 실제 paint와 조판이 갈린다.
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ttfs/opensource/RHWPPublicBatang-Regular.ttf"
    ));
    let face = ttf_parser::Face::parse(bytes, 0).expect("tracked public font");
    let selected =
        rhwp::renderer::font_metrics_data::find_metric("RHWP Public Batang", false, false)
            .expect("public program must retain its verified metric selection");
    assert_eq!(selected.metric.em_size, face.units_per_em());
    for ch in (' '..='~').chain('\u{ac00}'..='\u{d7a3}') {
        let glyph = face
            .glyph_index(ch)
            .expect("ASCII and modern Hangul coverage");
        assert_eq!(
            selected.metric.get_width(ch),
            face.glyph_hor_advance(glyph),
            "public program advance for {ch:?}"
        );
    }
    // 별도 공개 프로그램의 이름만 연결한다. 기존 선택은 유지한다.
    assert_eq!(
        rhwp::renderer::font_metrics_data::find_metric("Batang", false, false)
            .unwrap()
            .metric
            .name,
        "Batang"
    );
    assert_eq!(
        rhwp::renderer::font_metrics_data::find_metric("Haansoft Batang", false, false)
            .unwrap()
            .metric
            .name,
        "Haansoft Batang"
    );
}

// C202: 원본 PUA는 독립 Print의 Wingdings 프로그램, 실제 Unicode·는 CJK 프로그램이다.
#[test]
fn implicit_wingdings_dot_is_not_the_literal_cjk_middle_dot_program() {
    let measurer = EmbeddedTextMeasurer;
    let style = TextStyle {
        font_family: "굴림체".into(),
        font_size: 16.0,
        font_metric_trusted: true,
        ..Default::default()
    };
    for source in ['\u{F09E}', '\u{F0A0}'] {
        let text = format!("가{source}나");
        let positions = measurer.compute_char_positions(&text, &style);
        let em = (positions[2] - positions[1]) / style.font_size;
        assert!(
            (0.30..0.33).contains(&em),
            "independent Wingdings312/1000 program: {source:?} {em}"
        );
    }
    let literal = measurer.compute_char_positions("가·나", &style);
    assert!(
        (literal[2] - literal[1] - style.font_size).abs() < 0.03,
        "literal GulimChe middle dot keeps its independent full-em program"
    );
}

#[test]
fn source_symbol_advance_reaches_saved_cell_bounds_and_replay_positions() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(&node.node_type,RenderNodeType::TextRun(run) if run.text.contains('\u{F09E}')) {
            out.push(node);
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6032/2912695_civil_petition_form.hwp");
    let core = rhwp::DocumentCore::from_bytes(&std::fs::read(p).unwrap()).unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    assert_eq!(
        runs.len(),
        3,
        "all three source cells remain on independent Print page1"
    );
    for node in runs {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        assert!(run.cell_context.is_some(), "source cell ownership retained");
        assert_eq!(
            run.display_or_text(),
            run.text.replace('\u{F09E}', "·"),
            "paint projection preserves all neighboring content"
        );
        let positions = run
            .layout_positions
            .as_ref()
            .expect("source advance must reach actual replay");
        assert_eq!(positions.len(), run.text.chars().count() + 1);
        for (i, ch) in run.text.chars().enumerate() {
            if ch == '\u{F09E}' {
                let em = (positions[i + 1] - positions[i]) / run.style.font_size;
                assert!(
                    (0.30..0.33).contains(&em),
                    "source glyph program, not CJK fallback: {em}"
                );
            }
        }
        assert!(
            (positions.last().unwrap() - node.bbox.width).abs() < 1e-6,
            "saved cell extent and replay share one advance result"
        );
    }
}

#[test]
fn completed_civil_form_preserves_origins_after_each_source_wingdings_dot() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains('\u{F09E}'))
        {
            out.push(node);
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    // C202: 원본 전2쪽 Native/fresh WASM 최저99.98717%, 이 쪽은 양쪽100%.
    // review/overlay 직접 판독 후, 원본 독립 Print의 세 '퇴' 원점(pt)을
    // 96CSSpx로 환산한다. 글리프 폭 helper의 계산값을 기대 좌표로 쓰지 않는다.
    let expected_pt = [
        (225.5977325, 237.1008911),
        (112.2591553, 393.2901917),
        (101.1052017, 531.3792725),
    ];
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6032/2912695_civil_petition_form.hwp");
    let core = rhwp::DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    assert_eq!(
        nodes.len(),
        expected_pt.len(),
        "세 원문 셀의 기호 누락·중복 없음"
    );
    for (node, (expected_x, expected_y)) in nodes.into_iter().zip(expected_pt) {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        assert!(run.cell_context.is_some(), "원문 셀 소속 보존");
        let chars: Vec<_> = run.text.chars().collect();
        let dot = chars.iter().position(|ch| *ch == '\u{F09E}').unwrap();
        assert_eq!(chars[dot + 1], '퇴', "같은 원문의 바로 다음 글자");
        let positions = run.layout_positions.as_ref().expect("실제 replay 위치");
        assert_eq!(positions.len(), chars.len() + 1);
        let actual_x = node.bbox.x + positions[dot + 1];
        let actual_y = node.bbox.y + run.baseline;
        for (actual, expected) in [
            (actual_x, expected_x * 4.0 / 3.0),
            (actual_y, expected_y * 4.0 / 3.0),
        ] {
            assert!(
                (actual - expected).abs() <= 2.0,
                "독립 Print 원점 ±2CSSpx: 실제{actual}, 기준{expected}"
            );
        }
    }
}
