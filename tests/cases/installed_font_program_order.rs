//! 관계형: 실제 공급한 원본 프로그램의 TTF → TTC face → OTF → 대체 선택을 검사한다.
//! 주석 갱신: 2026-11-16 20:04:56 KST. 관계형이며 설치된 사설 글꼴에 의존하지 않는다.
//! 설치 글꼴이나 사설 바이너리 없이 원본 이름·형식·프로그램 신원의 소유를 확인한다.
//! 한양중고딕의 H2GTRM 설치 이름도 같은 원본 후보로 조회하며, 파일이 없으면 다음 형식을 선택한다.
//! Native 컨테이너 검사는 OFL 합성 TTF/TTC의 실제 RGBA 관계를 비교한다. 한컴 시각 점수가 아니다.
//! 추가 주석 갱신: 2026-11-17 15:06:06 대한민국 표준시. 직전 변경: 2bb86f50d385ddef49433721c3f54da6c901ac7d,
//! 2026-11-16T22:13:09+09:00. 라이선스·합성 차이: fixtures/fonts/RHWPNativeContainer.LICENSE.md.
use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use serde_json::{json, Value};

#[cfg(feature = "native-skia")]
mod native_containers {
    use super::*;
    use rhwp::paint::RenderProfile;
    use rhwp::renderer::{layer_renderer::RasterRenderOptions, skia::SkiaLayerRenderer};

    const TTF: &[u8] = include_bytes!("../fixtures/fonts/RHWPNativeContainer.ttf");
    const TTC: &[u8] = include_bytes!("../fixtures/fonts/RHWPNativeContainer.ttc");

    fn paint(renderer: SkiaLayerRenderer) -> Vec<u8> {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "AAAA").unwrap();
        let mut document = core.document().clone();
        for fonts in &mut document.doc_info.font_faces {
            fonts[0].name = "RHWP Native Container Probe".into();
            fonts[0].alt_type = 1;
        }
        for style in &mut document.doc_info.char_shapes {
            style.font_ids = [0; 7];
            style.base_size = 4800;
        }
        core.set_document(document);
        renderer.prepare_native_otf_fonts(&mut core).unwrap();
        let tree = core
            .build_page_layer_tree_with_profile(0, RenderProfile::Print)
            .unwrap();
        renderer
            .render_pixels_with_options(&tree, RasterRenderOptions::default())
            .unwrap()
            .bytes
    }

    #[test]
    fn native_supplied_bytes_prefer_ttf_over_nonzero_ttc_face_in_both_orders() {
        let render = |sources: &[Vec<u8>]| {
            paint(
                SkiaLayerRenderer::new()
                    .try_with_font_data(sources)
                    .unwrap(),
            )
        };
        let standalone = render(&[TTF.to_vec()]);
        let collection = render(&[TTC.to_vec()]);
        assert_ne!(
            standalone, collection,
            "서로 다른 A 윤곽이 실제로 그려지는 대조군"
        );
        for sources in [[TTC.to_vec(), TTF.to_vec()], [TTF.to_vec(), TTC.to_vec()]] {
            assert_eq!(
                render(&sources),
                standalone,
                "공급 순서와 무관하게 원본 TTF를 선택한다"
            );
        }
    }

    #[test]
    fn native_directory_order_does_not_promote_ttc_over_ttf() {
        let folder =
            std::env::temp_dir().join(format!("rhwp-native-container-{}", std::process::id()));
        std::fs::create_dir(&folder).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(folder.clone());
        let standalone = paint(
            SkiaLayerRenderer::new()
                .try_with_font_data(&[TTF.to_vec()])
                .unwrap(),
        );
        for ttc_first in [true, false] {
            let ttc_path = folder.join(if ttc_first { "0.ttc" } else { "1.ttc" });
            let ttf_path = folder.join(if ttc_first { "1.ttf" } else { "0.ttf" });
            std::fs::write(&ttc_path, TTC).unwrap();
            std::fs::write(&ttf_path, TTF).unwrap();
            assert_eq!(
                paint(SkiaLayerRenderer::new().with_font_paths(std::slice::from_ref(&folder))),
                standalone,
                "파일명 정렬과 무관하게 같은 family의 TTF를 선택한다"
            );
            std::fs::remove_file(ttc_path).unwrap();
            std::fs::remove_file(ttf_path).unwrap();
        }
    }
}

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
fn hanyang_junggothic_installed_names_use_ttf_then_ttc_then_cff() {
    for (with_ttf, with_ttc, expected) in [
        (true, true, "h2gtrm-ttf"),
        (false, true, "h2gtrm-ttc-face"),
        (false, false, "junggothic-cff"),
    ] {
        let mut programs = vec![program(
            "한양중고딕",
            "HY Junggothic",
            "junggothic-cff",
            false,
            false,
        )];
        if with_ttf {
            programs.push(program(
                "HY중고딕",
                "HYGothic-Medium",
                "h2gtrm-ttf",
                true,
                false,
            ));
        }
        if with_ttc {
            programs.push(program(
                "HYGothic-Medium",
                "HYGothic-Medium",
                "h2gtrm-ttc-face",
                true,
                true,
            ));
        }
        let value = environment_for_type("한양중고딕", json!(programs), 2);
        assert_eq!(
            value["selectedPrograms"]["한양중고딕"]["programId"], expected,
            "원본 HFT 선언도 설치된 한양중고딕의 실제 형식 순서를 따른다"
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
