//! 검사 유형: 프로필/글꼴/리플레이 8개는 관계형, 독립 Print 원점 대조 5개는 고정형(192DPI ±2px).
//! 주석 갱신: 2026-11-07 06:29 KST.
//! 부모 dbf9b54beb의 129쪽 raw: Native 94.76447% / fresh WASM 94.83414%(192DPI, 2px).
//! 129쪽의 미완료 글리프 좌표는 고정하지 않는다. 독립 A4/A5/혼재/채우기 뒤 40조건(68글리프)은 Native/fresh WASM 각 100%다.
//! 독립 한컴2024 Print의 A4 CTM은
//! [0.119935, 0, 0, -0.119869, 0, 841]이며 600DPI 원본 장치를 물리 종이에 투영한다.
//! source bbox·글자 크기를 변경하지 않고 같은 원본 glyph에만 이 프레임을 연결하는지 검사한다.

use rhwp::{
    paint::{GroupKind, LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    DocumentCore,
};

fn load() -> DocumentCore {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    DocumentCore::from_bytes(&std::fs::read(root.join("mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp")).unwrap()).unwrap()
}

fn collect(node: &LayerNode, inherited: &[[f64; 6]], out: &mut Vec<(String, Vec<[f64; 6]>)>) {
    match &node.kind {
        LayerNodeKind::Group {
            children,
            group_kind,
            ..
        } => {
            let mut frames = inherited.to_vec();
            if let GroupKind::Group(group) = group_kind {
                if let Some(frame) = group.paint_transform {
                    frames.push(frame);
                }
            }
            for child in children {
                collect(child, &frames, out);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            collect(child, inherited, out);
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Equation { equation, .. } = op {
                    out.push((equation.script.clone(), inherited.to_vec()));
                }
            }
        }
    }
}

fn logical_equation(node: &rhwp::renderer::render_tree::RenderNode) -> Option<serde_json::Value> {
    use rhwp::renderer::render_tree::RenderNodeType;
    if let RenderNodeType::Equation(equation) = &node.node_type {
        if equation.script.starts_with("P=460") {
            return Some(serde_json::json!({"bbox": node.bbox, "equation": equation}));
        }
    }
    node.children.iter().find_map(logical_equation)
}

fn paint_equation(node: &LayerNode) -> Option<serde_json::Value> {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => children.iter().find_map(paint_equation),
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            paint_equation(child)
        }
        LayerNodeKind::Leaf { ops } => ops.iter().find_map(|op| {
            if let PaintOp::Equation { bbox, equation } = op {
                if equation.script.starts_with("P=460") {
                    return Some(serde_json::json!({"bbox": bbox, "equation": equation}));
                }
            }
            None
        }),
    }
}

fn frames(compat2024: bool, profile: RenderProfile) -> Vec<[f64; 6]> {
    let mut core = load();
    core.set_hangul2024_compat(compat2024);
    let tree = core
        .build_page_layer_tree_with_profile(128, profile)
        .unwrap();
    let mut rows = Vec::new();
    collect(&tree.root, &[], &mut rows);
    rows.into_iter()
        .find(|(script, _)| script.starts_with("P=460"))
        .unwrap()
        .1
}

/// 관계형: 원본 표시와 프로필·글꼴의 프레임 소유를 검사하며 원문 좌표를 고정하지 않는다.
#[test]
fn original_hft_equation_uses_the_same_physical_print_paper_as_body_glyphs() {
    let mut core = load();
    core.set_hangul2024_compat(true);
    let source = core.build_page_render_tree(128).unwrap();
    let paint = core
        .build_page_layer_tree_with_profile(128, RenderProfile::Print)
        .unwrap();
    assert_eq!(
        logical_equation(&source.root).unwrap(),
        paint_equation(&paint.root).unwrap(),
        "복합 수식의 source bbox·배치·글자 크기를 수정하지 않고 물리 프레임만 발행한다"
    );
    let frames = frames(true, RenderProfile::Print);
    assert_eq!(frames.len(), 1, "원본 수식의 물리 투영을 한 번만 발행한다");
    let [a, b, c, d, e, f] = frames[0];
    assert!((0.9994..0.9996).contains(&a), "독립 Print의 가로 장치 비율");
    assert!((0.9988..0.9990).contains(&d), "독립 Print의 세로 장치 비율");
    assert_eq!([b, c, e, f], [0.0; 4]);
}

/// 관계형: 원본 표시와 프로필·글꼴의 프레임 소유를 검사하며 원문 좌표를 고정하지 않는다.
#[test]
fn screen_equations_keep_the_existing_logical_frame() {
    assert!(frames(true, RenderProfile::Screen).is_empty());
}

/// 관계형: 원본 표시와 프로필·글꼴의 프레임 소유를 검사하며 원문 좌표를 고정하지 않는다.
#[test]
fn previous_print_generation_keeps_the_existing_frame() {
    assert!(frames(false, RenderProfile::Print).is_empty());
}

/// 관계형: 원본 표시와 프로필·글꼴의 프레임 소유를 검사하며 원문 좌표를 고정하지 않는다.
#[test]
fn high_quality_export_keeps_its_existing_frame() {
    assert!(frames(true, RenderProfile::HighQuality).is_empty());
}

/// 관계형: 원본 표시와 프로필·글꼴의 프레임 소유를 검사하며 원문 좌표를 고정하지 않는다.
#[test]
fn modern_face_is_not_reclassified_as_an_original_hft_program() {
    use rhwp::model::{control::Control, paragraph::Paragraph};
    fn modernize(paragraphs: &mut [Paragraph]) {
        for para in paragraphs {
            for control in &mut para.controls {
                match control {
                    Control::Equation(value) => value.version_info = "Equation Version 60".into(),
                    Control::Table(table) => {
                        for cell in &mut table.cells {
                            modernize(&mut cell.paragraphs);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("tests/fixtures/issue1949/legacy-character-plane.hwpx")).unwrap(),
    )
    .unwrap();
    let mut document = core.document().clone();
    for section in &mut document.sections {
        modernize(&mut section.paragraphs);
    }
    core.set_document(document);
    core.set_hangul2024_compat(true);
    let tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let mut rows = Vec::new();
    collect(&tree.root, &[], &mut rows);
    assert!(
        !rows.is_empty(),
        "같은 입력의 현대 표시 대조군을 실제로 구성한다"
    );
    assert!(rows.iter().all(|(_, frames)| frames.is_empty()));
}

// 이 좌표는 후보 출력에서 뽑지 않았다. 독립 한컴2024 Save/Print PDF의
// 실제 Type3 원점이다. A4/A5 및 혼재 글줄 전쪽 Native/fresh WASM 각 100%를 확인했다.
fn original_line_glyphs(
    value: &rhwp::renderer::equation::layout::LayoutBox,
    x: f64,
    y: f64,
    out: &mut Vec<(char, f64, f64)>,
) {
    use rhwp::renderer::equation::layout::LayoutKind;
    let (x, y) = (x + value.x, y + value.y);
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            assert_eq!(run.font_family, "RHWP Hancom Legacy Equation");
            out.extend(
                run.glyphs
                    .chars()
                    .zip(&run.positions)
                    .map(|(ch, dx)| (ch, x + dx, y + value.baseline + run.baseline_shift)),
            );
        }
        LayoutKind::Row(children) => {
            for child in children {
                original_line_glyphs(child, x, y, out);
            }
        }
        LayoutKind::FontStyle { body, .. } => original_line_glyphs(body, x, y, out),
        other => panic!("독립 단일 글줄 대조군의 표시 구조가 바뀜: {other:?}"),
    }
}

fn printed_glyphs(
    node: &LayerNode,
    inherited: &[[f64; 6]],
    out: &mut std::collections::BTreeMap<usize, Vec<(char, f64, f64)>>,
    svg: bool,
) {
    match &node.kind {
        LayerNodeKind::Group {
            children,
            group_kind,
            ..
        } => {
            let mut frames = inherited.to_vec();
            if let GroupKind::Group(group) = group_kind {
                if let Some(frame) = group.paint_transform {
                    frames.push(frame);
                }
            }
            for child in children {
                printed_glyphs(child, &frames, out, svg);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            printed_glyphs(child, inherited, out, svg);
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Equation { bbox, equation } = op {
                    let mut glyphs = Vec::new();
                    if svg {
                        let fragment = format!("<svg>{}</svg>", equation.svg_content);
                        let document = roxmltree::Document::parse(&fragment).unwrap();
                        for text in document
                            .descendants()
                            .filter(|node| node.has_tag_name("text"))
                        {
                            assert_eq!(
                                text.attribute("font-family"),
                                Some("RHWP Hancom Legacy Equation")
                            );
                            let matrix = text
                                .attribute("transform")
                                .unwrap()
                                .strip_prefix("matrix(")
                                .unwrap()
                                .strip_suffix(')')
                                .unwrap();
                            let m: Vec<f64> = matrix
                                .split_whitespace()
                                .map(|v| v.parse().unwrap())
                                .collect();
                            assert_eq!(m.len(), 6);
                            let x: f64 = text.attribute("x").unwrap_or("0").parse().unwrap();
                            let y: f64 = text.attribute("y").unwrap_or("0").parse().unwrap();
                            let chars: Vec<char> = text.text().unwrap().chars().collect();
                            assert_eq!(
                                chars.len(),
                                1,
                                "수식 SVG가 실제 발행한 글리프 원점을 읽는다"
                            );
                            glyphs.push((
                                chars[0],
                                bbox.x + m[0] * x + m[2] * y + m[4],
                                bbox.y + m[1] * x + m[3] * y + m[5],
                            ));
                        }
                    } else {
                        original_line_glyphs(&equation.layout_box, bbox.x, bbox.y, &mut glyphs);
                    }
                    for (_, x, y) in &mut glyphs {
                        for [a, b, c, d, e, f] in inherited.iter().rev() {
                            (*x, *y) = (a * *x + c * *y + e, b * *x + d * *y + f);
                        }
                    }
                    assert!(out.insert(equation.para_index.unwrap(), glyphs).is_none());
                }
            }
        }
    }
}

fn check_independent_print_paper_mode(key: &str, svg: bool) {
    use sha2::{Digest, Sha256};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let data: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/issue4138/original-print-paper/print-oracle.json"
    ))
    .unwrap();
    let oracle = data
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["key"] == key)
        .unwrap();
    for name in ["input", "pdf"] {
        let bytes = std::fs::read(root.join(oracle[name].as_str().unwrap())).unwrap();
        assert_eq!(
            Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            oracle[format!("{name}_sha256")]
        );
    }
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join(oracle["input"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(
        core.page_count(),
        oracle["page_count"].as_u64().unwrap() as u32
    );
    let mut glyphs = std::collections::BTreeMap::new();
    for page in 0..core.page_count() {
        let tree = core
            .build_page_layer_tree_with_profile(page, RenderProfile::Print)
            .unwrap();
        if key == "after-fill" {
            let mut rows = Vec::new();
            collect(&tree.root, &[], &mut rows);
            assert_eq!(rows.len(), 4);
            assert!(
                rows.iter().all(|(_, frames)| frames.is_empty()),
                "독립 PDF의 채우기 뒤 CTM은 .120이며 정밀 종이 변환을 중복 적용하지 않는다"
            );
        }
        printed_glyphs(&tree.root, &[], &mut glyphs, svg);
    }
    assert_eq!(glyphs.len(), oracle["cases"].as_array().unwrap().len());
    for case in oracle["cases"].as_array().unwrap() {
        let para = case["para_index"].as_u64().unwrap() as usize;
        let actual = &glyphs[&para];
        let expected = case["glyphs"].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "{key} para{para}: 누락·중복");
        for ((ch, x, y), reference) in actual.iter().zip(expected) {
            assert_eq!(ch.to_string(), reference["character"]);
            let point = reference["point_origin"].as_array().unwrap();
            let (px, py) = (
                point[0].as_f64().unwrap() * 96.0 / 72.0,
                point[1].as_f64().unwrap() * 96.0 / 72.0,
            );
            // 정본 192DPI의 2px = 논리 96DPI의 1px. 점수를 맞춰 허용폭을 늘리지 않는다.
            assert!(
                (x - px).abs() <= 1.0 && (y - py).abs() <= 1.0,
                "{key} para{para} {ch}: physical ({x},{y}) != independent Print ({px},{py})"
            );
        }
    }
}

fn check_independent_print_paper(key: &str) {
    check_independent_print_paper_mode(key, false);
}

/// 고정형: 독립 한컴 Print 원점 대비 192DPI ±2px. 해당 대조군 Native/fresh WASM 전쪽 각 100%를 확인했다.
#[test]
fn a4_original_line_glyph_origins_match_independent_print_within_two_device_pixels() {
    check_independent_print_paper("a4");
}

/// 고정형: 독립 한컴 Print 원점 대비 192DPI ±2px. 해당 대조군 Native/fresh WASM 전쪽 각 100%를 확인했다.
#[test]
fn a5_original_line_glyph_origins_match_independent_print_within_two_device_pixels() {
    check_independent_print_paper("a5");
}

/// 고정형: 독립 한컴 Print 원점 대비 192DPI ±2px. 해당 대조군 Native/fresh WASM 전쪽 각 100%를 확인했다.
#[test]
fn mixed_original_line_glyph_origins_match_independent_print_within_two_device_pixels() {
    check_independent_print_paper("linear-mixed");
}

/// 고정형: 배경 채우기 뒤의 독립 Print 원점 대비 192DPI ±2px. 해당 대조군 Native/fresh WASM 전쪽 각 100%를 확인했다.
#[test]
fn original_line_after_solid_fill_keeps_its_own_glyph_baseline() {
    check_independent_print_paper("after-fill");
}

/// 고정형: SVG 소비자가 받는 실제 글리프 행렬을 독립 Print와 192DPI ±2px로 대조한다.
#[test]
fn original_print_svg_glyph_origins_match_the_independent_oracle() {
    for key in ["a4", "a5", "linear-mixed", "after-fill"] {
        check_independent_print_paper_mode(key, true);
    }
}

/// 관계형: SVG 공급 요청이 실제 수식 조각의 선언 face와 글리프를 보존한다.
#[test]
fn svg_font_collection_keeps_the_equation_declared_program() {
    use rhwp::renderer::{
        layer_renderer::LayerRenderer, svg::FontEmbedMode, svg_layer::SvgLayerRenderer,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("tests/fixtures/issue4138/original-print-paper/a4.hwp")).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(true);
    let tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let mut renderer = SvgLayerRenderer::new();
    renderer.inner_mut().font_embed_mode = FontEmbedMode::Subset;
    renderer.render_page(&tree).unwrap();
    let codepoints = renderer.inner().font_codepoints();
    assert!(
        codepoints
            .get("RHWP Hancom Legacy Equation")
            .is_some_and(|set| set.contains(&'P')),
        "표시 조각에 적힌 원본 HFT face를 실제 임베딩 요청에 전달한다"
    );
    assert!(
        !codepoints.contains_key("Latin Modern Math"),
        "이 대조군에 없는 face를 문자 종류로 추정하지 않는다"
    );
}

/// 합성 관계형: 같은 PageLayerTree의 명시적 affine이 실제 SVG의 수식 자손까지 상속된다.
#[test]
fn svg_replay_keeps_explicit_group_affine_around_equation_children() {
    use rhwp::paint::{CacheHint, GroupKind};
    use rhwp::renderer::{
        layer_renderer::LayerRenderer, render_tree::GroupNode, svg_layer::SvgLayerRenderer,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("tests/fixtures/issue4138/original-print-paper/a4.hwp")).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(true);
    let mut tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let frame = [1.0, 0.25, -0.5, 1.0, 17.0, 29.0];
    tree.root = LayerNode::group(
        tree.root.bounds,
        None,
        vec![tree.root],
        CacheHint::None,
        GroupKind::Group(GroupNode {
            paint_transform: Some(frame),
            section_index: None,
            para_index: None,
            control_index: None,
        }),
    );
    let mut renderer = SvgLayerRenderer::new();
    renderer.render_page(&tree).unwrap();
    let document = roxmltree::Document::parse(renderer.output()).unwrap();
    assert!(
        document.descendants().any(|node| node.has_tag_name("g")
            && node.attribute("transform") == Some("matrix(1 0.25 -0.5 1 17 29)")
            && node
                .descendants()
                .any(|child| child.has_tag_name("text") && child.text() == Some("P"))),
        "SVG의 좌표계 상속 계약에 따라 명시적 생산자 행렬을 실제 글리프 자손에 전달한다"
    );
}

/// 관계형: 명시적 글꼴 공급은 파일명이 아닌 내부 family로 연결하며 다른 family는 선택하지 않는다.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn svg_embeds_the_supplied_family_even_when_its_filename_is_unrelated() {
    use rhwp::renderer::svg::FontEmbedMode;
    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = Directory(std::env::temp_dir().join(format!(
        "rhwp-equation-font-supply-{}-{nonce}",
        std::process::id()
    )));
    std::fs::create_dir(&directory.0).unwrap();
    let font = include_bytes!("../fixtures/native_inline_placeholder/placeholder-contract.ttf");
    std::fs::write(directory.0.join("unrelated-filename.ttf"), font).unwrap();
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "A").unwrap();
    let mut document = core.document().clone();
    for fonts in &mut document.doc_info.font_faces {
        for face in fonts {
            face.name = "Rhwp Inline Placeholder Contract".into();
        }
    }
    core.set_document(document.clone());
    let paths = std::slice::from_ref(&directory.0);
    for mode in [FontEmbedMode::Full, FontEmbedMode::Subset] {
        let svg = core.render_page_svg_with_fonts(0, mode, paths).unwrap();
        let parsed = roxmltree::Document::parse(&svg).unwrap();
        let css = parsed
            .descendants()
            .find(|node| node.has_tag_name("style"))
            .unwrap()
            .text()
            .unwrap();
        assert!(
            css.contains("font-family: \"Rhwp Inline Placeholder Contract\"; src: url(\"data:"),
            "{mode:?}: 지정 폴더의 내부 family가 파일명과 무관하게 실제 데이터로 공급되어야 한다"
        );
    }
    for fonts in &mut document.doc_info.font_faces {
        for face in fonts {
            face.name = "Rhwp Missing Supplied Family".into();
        }
    }
    core.set_document(document);
    let svg = core
        .render_page_svg_with_fonts(0, FontEmbedMode::Full, paths)
        .unwrap();
    assert!(
        !svg.contains("data:font/"),
        "무관한 공급 파일을 다른 family로 선언하지 않는다"
    );
}
