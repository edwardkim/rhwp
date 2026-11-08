//! 관계형: 같은 형식·스타일의 대체 프로그램 열거가 OS가 선택한 실제 프로그램을 바꾸지 않는다.
//! 주석 갱신: 2026-11-08 20:09 KST. 저장소 MIT 합성 TTF 두 개만 별도 Fontconfig에 공급한다.
//! 호스트 설치 글꼴·실문서 px·golden에는 의존하지 않는 실제 Native 프로그램 선택 계약이다.
//! 독립 FontMgr의 선택을 명시 공급한 렌더링과 시스템 선택 렌더링을 비교한다. 실문서 raw는 비해당이다.
#![cfg(all(feature = "native-skia", target_os = "linux"))]
use rhwp::{
    paint::RenderProfile,
    renderer::{
        font_environment::FontEnvironment, layer_renderer::RasterRenderOptions,
        skia::SkiaLayerRenderer,
    },
    DocumentCore,
};
use std::path::PathBuf;
const FAMILY: &str = "RHWP Host Fixture";
const CHILD: &str = "RHWP_SYSTEM_PROGRAM_PREFERENCE_CHILD";
struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("이 검사만 만든 자료 정리");
    }
}
fn render(renderer: &SkiaLayerRenderer) -> Vec<u8> {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    for font in doc.doc_info.font_faces.iter_mut().flatten() {
        font.name = FAMILY.into();
        font.alt_type = 1;
        font.native_program = None;
        font.is_embedded = false;
        font.subst_font = None;
    }
    core.set_document(doc);
    core.insert_text_native(0, 0, 0, "A").unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(&format!(r#"{{"id":"fixture","substitutions":{{"{FAMILY}":"{FAMILY}"}},"trueTypeFaces":["{FAMILY}"],"preserveSourceMetrics":false}}"#)).unwrap())).unwrap();
    renderer.prepare_declared_font_fallbacks(&mut core).unwrap();
    renderer.prepare_native_pua_fonts(&mut core).unwrap();
    renderer.prepare_native_otf_fonts(&mut core).unwrap();
    let tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    renderer
        .render_raster_with_options(
            &tree,
            RasterRenderOptions {
                dpi: Some(96.0),
                ..Default::default()
            },
        )
        .unwrap()
        .bytes
}
fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub(super) fn check_system_program_preference() {
    if std::env::var_os(CHILD).is_some() {
        let manager = skia_safe::FontMgr::default();
        let names: Vec<_> = manager.family_names().collect();
        assert_eq!(
            names,
            vec![FAMILY.to_string()],
            "공개 합성 family 외의 시스템 폰트를 사용하지 않는다"
        );
        let chosen = manager
            .match_family_style(FAMILY, skia_safe::FontStyle::normal())
            .unwrap();
        assert_eq!(
            chosen.post_script_name().as_deref(),
            Some("RHWPHostFixture-Preferred"),
            "OS가 선택한 프로그램을 독립 확인"
        );
        let (data, index) = chosen.to_existing_font_data().unwrap();
        assert_eq!(index, 0);
        let actual = render(&SkiaLayerRenderer::new());
        let expected = render(
            &SkiaLayerRenderer::new()
                .try_with_font_data(&[data.as_bytes().to_vec()])
                .unwrap(),
        );
        assert!(
            actual == expected,
            "같은 형식의 다른 TTF가 OS의 프로그램 선택을 덮어쓰지 않는다"
        );
        return;
    }
    let directory = Directory(std::env::temp_dir().join(format!(
            "rhwp-program-preference-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    let fonts = directory.0.join("fonts");
    let cache = directory.0.join("cache");
    std::fs::create_dir_all(&fonts).unwrap();
    std::fs::create_dir(&cache).unwrap();
    std::fs::write(
        fonts.join("a-other.ttf"),
        include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf"),
    )
    .unwrap();
    let preferred = fonts.join("z-preferred.ttf");
    std::fs::write(
        &preferred,
        include_bytes!("../fixtures/fonts/RHWPHostFixture-Preferred.ttf"),
    )
    .unwrap();
    let config = directory.0.join("fonts.conf");
    std::fs::write(&config,format!(r#"<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd"><fontconfig><dir>{}</dir><cachedir>{}</cachedir><match target="pattern"><test name="family" compare="eq"><string>{FAMILY}</string></test><edit name="file" mode="assign"><string>{}</string></edit></match></fontconfig>"#,xml(&fonts.to_string_lossy()),xml(&cache.to_string_lossy()),xml(&preferred.to_string_lossy()))).unwrap();
    let name = std::thread::current().name().unwrap().to_string();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &name, "--nocapture"])
        .env(CHILD, "1")
        .env("FONTCONFIG_FILE", config)
        .env_remove("FONTCONFIG_PATH")
        .env_remove("RHWP_FONT_PATH")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "격리된 Native 프로그램 선택 실패:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
