//! 검사 유형: 관계형. Native SVG와 PNG가 실제 공급 프로그램의 같은 조판 전진을 소비한다.
//! 주석 갱신: 2026-11-16 04:52 KST.
//! 공개 합성 TTF의 CLI/API 교차 계약이며 설치 글꼴·실문서 px 좌표에 의존하지 않는다.
//! 실물 PDF 일치율은 비해당이며 공급 전후를 서로 대조해 누락된 준비 단계를 검출한다.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use rhwp::paint::RenderProfile;
use rhwp::renderer::{font_environment::FontEnvironment, svg::FontEmbedMode};
use rhwp::DocumentCore;
use std::{path::PathBuf, process::Command};

#[test]
fn svg_cli_uses_the_same_original_program_advances_as_prepared_png() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let font = root.join("tests/fixtures/native_font_family_names/family-contract.ttf");
    let family = "Rhwp Family Contract";
    let fonts = ["Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User"]
        .iter()
        .map(|lang| format!("<FONTFACE Count=\"1\" Lang=\"{lang}\"><FONT Id=\"0\" Name=\"{family}\" Type=\"ttf\"/></FONTFACE>"))
        .collect::<String>();
    let xml = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><HWPML Version=\"2.91\"><HEAD SecCnt=\"1\"><MAPPINGTABLE><FACENAMELIST>{fonts}</FACENAMELIST><CHARSHAPELIST Count=\"1\"><CHARSHAPE Id=\"0\" Height=\"1200\"/></CHARSHAPELIST></MAPPINGTABLE></HEAD><BODY><SECTION Id=\"0\"><P ParaShape=\"0\"><TEXT CharShape=\"0\"><CHAR>AVI AV</CHAR></TEXT></P></SECTION></BODY></HWPML>");
    let mut core = DocumentCore::from_bytes(xml.as_bytes()).unwrap();
    let environment = FontEnvironment::from_native_document_substitutions(
        &core.document().doc_info,
        std::slice::from_ref(&font),
    )
    .unwrap();
    core.set_font_environment(environment).unwrap();
    let before = core
        .render_page_svg_with_fonts_and_profile(
            0,
            FontEmbedMode::Full,
            std::slice::from_ref(&font),
            RenderProfile::Print,
        )
        .unwrap();
    let renderer =
        rhwp::renderer::skia::SkiaLayerRenderer::new().with_font_paths(std::slice::from_ref(&font));
    assert!(renderer.prepare_native_otf_fonts(&mut core).unwrap());
    let expected = core
        .render_page_svg_with_fonts_and_profile(
            0,
            FontEmbedMode::Full,
            std::slice::from_ref(&font),
            RenderProfile::Print,
        )
        .unwrap();
    assert_ne!(
        before, expected,
        "실제 프로그램 준비가 조판 전진에 영향을 주는 대조군"
    );

    let work = std::env::temp_dir().join(format!("rhwp-7732-svg-program-{}", std::process::id()));
    std::fs::create_dir_all(&work).unwrap();
    let source = work.join("source.hml");
    std::fs::write(&source, xml).unwrap();
    let binary = std::env::var_os("CARGO_BIN_EXE_rhwp")
        .map(PathBuf::from)
        .or_else(|| option_env!("CARGO_BIN_EXE_rhwp").map(PathBuf::from))
        .expect("실제 CLI 실행 파일");
    let output = work.join("svg");
    let result = Command::new(binary)
        .arg("export-svg")
        .arg(&source)
        .args(["--embed-fonts=full", "--profile", "print", "--font-path"])
        .arg(&font)
        .args(["--page", "0", "--json", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let files: Vec<_> = std::fs::read_dir(&output)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "svg"))
        .collect();
    assert_eq!(files.len(), 1, "단일 페이지의 실제 SVG 파일");
    let svg = std::fs::read_to_string(&files[0]).unwrap();
    assert_eq!(
        svg, expected,
        "같은 공급 프로그램으로 준비한 API와 CLI의 실제 SVG"
    );
    std::fs::remove_dir_all(work).unwrap();
}
