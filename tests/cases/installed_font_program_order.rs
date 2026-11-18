//! 관계형: 실제 공급한 원본 프로그램의 TTF → TTC face → OTF → 대체 선택을 검사한다.
//! 주석 갱신: 2026-11-16 20:04:56 KST. 관계형이며 설치된 사설 글꼴에 의존하지 않는다.
//! 설치 글꼴이나 사설 바이너리 없이 원본 이름·형식·프로그램 신원의 소유를 확인한다.
//! 한양 계열 HFT와 HY 계열 TrueType은 별도 원본이다. 형식 순서는 같은 프로그램 안에만 적용한다.
//! 주석 갱신: 2026-11-18 21:18:50 KST. 관계형이며 좌표·설치된 사설 글꼴에 의존하지 않는다.
//! Native 컨테이너의 합성 RGBA 검사는 skia_native_glyph_replay.rs로 이관해 Native CI에서 실행한다.
//! 추가 주석 갱신: 2026-11-17 15:06:06 대한민국 표준시. 직전 변경: 2bb86f50d385ddef49433721c3f54da6c901ac7d,
//! 2026-11-16T22:13:09+09:00. 라이선스·합성 차이: fixtures/fonts/RHWPNativeContainer.LICENSE.md.
use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use serde_json::{json, Value};

fn environment(source: &str, programs: Value) -> Value {
    environment_for_type(source, programs, 1)
}

fn environment_for_type(source: &str, programs: Value, font_type: u8) -> Value {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    for fonts in &mut doc.doc_info.font_faces {
        for font in fonts {
            font.name = source.into();
            font.alt_type = font_type;
        }
    }
    let requests = FontEnvironment::document_requests(&doc.doc_info);
    assert!(requests
        .iter()
        .all(|request| request.primary.first().map(String::as_str) == Some(source)));
    serde_json::to_value(
        FontEnvironment::from_available_programs(&doc.doc_info, &programs.to_string()).unwrap(),
    )
    .unwrap()
}
fn program(query: &str, family: &str, id: &str, true_type: bool, collection: bool) -> Value {
    json!({"query":query,"family":family,"programId":id,"trueType":true_type,
        "openTypeCff":!true_type,"collectionFace":collection})
}
#[test]
fn original_otf_is_selected_before_compatibility_ttf() {
    let value = environment(
        "옥수수",
        json!([
            program("옥수수", "Corn", "original-otf", false, false),
            program("돋움", "Dotum", "substitute-ttf", true, false)
        ]),
    );
    assert_eq!(value["substitutions"]["옥수수"], "Corn");
    assert_eq!(
        value["selectedPrograms"]["옥수수"]["programId"],
        "original-otf"
    );
}
#[test]
fn original_program_aliases_use_ttf_then_ttc_then_otf() {
    let source = "KoPub바탕체 Light";
    let alias = "KoPubBatangLight";
    for (primary, secondary, expected) in [
        ((false, false), (true, false), "alias"),
        ((true, true), (true, false), "alias"),
        ((false, false), (true, true), "alias"),
        ((true, false), (true, true), "source"),
    ] {
        let value = environment(
            source,
            json!([
                program(source, "KoPubBatang Light", "source", primary.0, primary.1),
                program(
                    alias,
                    "KoPubBatang Light",
                    "alias",
                    secondary.0,
                    secondary.1
                )
            ]),
        );
        assert_eq!(value["selectedPrograms"][source]["programId"], expected);
    }
}

#[test]
fn hanyang_and_hy_keep_separate_original_programs() {
    for (hft, hy, hft_family, hy_family) in [
        ("한양중고딕", "HY중고딕", "HY Junggothic", "HYGothic-Medium"),
        (
            "한양신명조",
            "HY신명조",
            "HY Sinmyeongjo",
            "HYSinMyeongJo-Medium",
        ),
        ("한양견고딕", "HY견고딕", "HY Gyeongothic", "HYGothic-Extra"),
        (
            "한양견명조",
            "HY견명조",
            "HY Gyeonmyeongjo",
            "HYMyeongJo-Extra",
        ),
        ("한양그래픽", "HY그래픽", "HY Graphic", "HYGraphic-Medium"),
        ("한양궁서", "HY궁서", "HY Gungseo", "HY궁서"),
    ] {
        let mut programs = vec![
            program(hft, hft_family, "original-hft", false, false),
            program(hy, hy_family, "different-ttf", true, false),
        ];
        // 같은 질의에 서로 다른 파일 ID를 넣는 잘못된 공급 목록을 만들지 않는다.
        if hy != hy_family {
            programs.push(program(hy_family, hy_family, "different-ttc", true, true));
        }
        let value = environment_for_type(hft, json!(programs), 2);
        assert_eq!(
            value["selectedPrograms"][hft]["programId"], "original-hft",
            "{hft} 원본이 있으면 별도 {hy}의 형식을 우선하지 않는다"
        );
        let value = environment_for_type(hy, json!(programs), 1);
        assert_eq!(
            value["selectedPrograms"][hy]["programId"], "different-ttf",
            "{hy} 요청도 자기 원본을 선택한다"
        );
        let missing = environment_for_type(
            hft,
            json!([program(hy, hy_family, "different-ttf", true, false)]),
            2,
        );
        assert!(
            missing["selectedPrograms"][hft].is_null(),
            "{hft} 부재를 {hy}의 원본 설치로 인증하지 않는다"
        );
    }
}

#[test]
fn hanyang_svg_local_faces_do_not_register_hy_under_an_original_name() {
    use rhwp::renderer::{
        render_tree::{RenderNode, RenderNodeType},
        svg::{generate_font_style, FontEmbedMode, SvgRenderer},
    };
    fn use_family(node: &mut RenderNode, family: &str) {
        if let RenderNodeType::TextRun(run) = &mut node.node_type {
            run.style.font_family = family.into();
        }
        for child in &mut node.children {
            use_family(child, family);
        }
    }
    for (original, other) in [
        ("한양중고딕", "HY중고딕"),
        ("한양신명조", "HY신명조"),
        ("한양견고딕", "HY견고딕"),
        ("한양견명조", "HY견명조"),
        ("한양그래픽", "HY그래픽"),
        ("한양궁서", "HY궁서"),
    ] {
        let mut renderer = SvgRenderer::new();
        renderer.font_embed_mode = FontEmbedMode::Style;
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "A가═").unwrap();
        let mut tree = core.build_page_render_tree(0).unwrap();
        use_family(&mut tree.root, original);
        renderer.render_tree(&tree);
        let css = generate_font_style(&renderer, &[], &std::collections::HashMap::new());
        let own = css
            .lines()
            .find(|line| line.contains(&format!("font-family: \"{original}\"")))
            .unwrap_or_else(|| panic!("원본 family의 CSS가 있어야 한다: {css}"));
        assert!(
            !own.contains(&format!("local(\"{other}\")")),
            "{other}를 {original}의 원본 local face로 재등록하지 않는다: {own}"
        );
    }
}

#[test]
fn native_discovery_checks_installed_otf_even_for_a_ttf_declared_source() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    let source = "RHWP HFT Outline Fixture";
    for fonts in &mut doc.doc_info.font_faces {
        for font in fonts {
            font.name = source.into();
            font.alt_type = 1;
        }
    }
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts");
    let selected = FontEnvironment::from_native_document_substitutions(&doc.doc_info, &[directory])
        .unwrap()
        .expect("원본 OTF를 TTF 선언 때문에 조회 전에 건너뛰지 않는다");
    let value = serde_json::to_value(selected).unwrap();
    assert_eq!(value["selectedPrograms"][source]["openTypeCff"], true);
    assert_eq!(value["selectedPrograms"][source]["trueType"], false);
}
