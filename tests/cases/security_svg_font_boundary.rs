//! Private security regression: document font names are data in CSS and SVG.
//! Synthetic IR preserves a real blank-document structure and a valid fixture font;
//! this checks serialization boundaries, not Hancom layout fidelity.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::bin_data::{BinData, BinDataContent, BinDataType};
use rhwp::paint::RenderProfile;
use rhwp::renderer::svg::FontEmbedMode;
use rhwp::DocumentCore;

#[cfg(unix)]
struct Directory(std::path::PathBuf);

#[cfg(unix)]
impl Directory {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("rhwp-private-font-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}

#[cfg(unix)]
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn document(face: &str, embedded: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "ABC 가나다").unwrap();
    let mut doc = core.document().clone();
    for fonts in &mut doc.doc_info.font_faces {
        for font in fonts {
            font.name = face.into();
            font.is_embedded = embedded;
            font.resolved_bin_data_id = embedded.then_some(1);
        }
    }
    if embedded {
        doc.doc_info.bin_data_list.push(BinData {
            data_type: BinDataType::Embedding,
            storage_id: 1,
            extension: Some("ttf".into()),
            ..Default::default()
        });
        doc.bin_data_content.push(BinDataContent {
            id: 1,
            extension: "ttf".into(),
            data: include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf")
                .to_vec()
                .into(),
        });
    }
    core.set_document(doc);
    core
}

fn evidence(name: &str, bytes: &[u8]) {
    if let Some(directory) = std::env::var_os("RHWP_PRIVATE_SECURITY_OUTPUT") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(name), bytes).unwrap();
    }
}

fn style(svg: &str) -> String {
    let parsed = roxmltree::Document::parse(svg).expect("export must remain valid SVG XML");
    assert_eq!(parsed.root_element().tag_name().name(), "svg");
    assert!(!parsed.descendants().any(|n| n.has_tag_name("script")));
    let styles: Vec<_> = parsed
        .descendants()
        .filter(|n| n.has_tag_name("style"))
        .collect();
    assert_eq!(
        styles.len(),
        1,
        "font metadata cannot introduce DOM elements"
    );
    assert!(
        parsed.descendants().any(|n| n.has_tag_name("text")),
        "body text retained"
    );
    styles[0].text().unwrap().to_string()
}

#[test]
fn embedded_font_names_cannot_close_svg_style_in_screen_or_print() {
    let face = "Safe</style><script>security_marker()</script><style>";
    let core = document(face, true);
    for profile in [RenderProfile::Screen, RenderProfile::Print] {
        let svg = core
            .render_page_svg_layer_with_profile_native(0, profile)
            .unwrap();
        evidence(&format!("embedded-{profile:?}.svg"), svg.as_bytes());
        let css = style(&svg);
        assert!(!css.contains("</style>"));
        assert!(
            css.contains("data:font/ttf;base64,"),
            "embedded font preserved"
        );
    }
}

#[test]
fn css_string_delimiters_are_encoded_in_every_embedding_mode() {
    // Independent CSS syntax expectation: quote/backslash/LF are hex escapes,
    // each terminated by whitespace so the next hex digit remains literal.
    let face = "A\"B\\C\nD";
    let expected = "font-family: \"A\\22 B\\5c C\\a D\";";
    for embedded in [false, true] {
        let core = document(face, embedded);
        for mode in [
            FontEmbedMode::Style,
            FontEmbedMode::Subset,
            FontEmbedMode::Full,
        ] {
            let svg = core.render_page_svg_with_fonts(0, mode, &[]).unwrap();
            evidence(&format!("strings-{mode:?}-{embedded}.svg"), svg.as_bytes());
            let css = style(&svg);
            assert!(
                css.contains(expected),
                "{mode:?}, embedded={embedded}: {css}"
            );
            if !embedded {
                assert!(css.contains("local(\"A\\22 B\\5c C\\a D\")"));
            }
        }
        assert_eq!(core.document().doc_info.font_faces[0][0].name, face);
    }
}

#[test]
fn normal_font_face_and_local_lookup_are_preserved() {
    let core = document("정상 글꼴 ABC", false);
    let svg = core
        .render_page_svg_with_fonts(0, FontEmbedMode::Style, &[])
        .unwrap();
    assert_eq!(
        style(&svg).trim(),
        "@font-face { font-family: \"정상 글꼴 ABC\"; src: local(\"정상 글꼴 ABC\"); }"
    );
    let svg = core
        .render_page_svg_with_fonts(0, FontEmbedMode::None, &[])
        .unwrap();
    let parsed = roxmltree::Document::parse(&svg).unwrap();
    assert!(!parsed.descendants().any(|n| n.has_tag_name("style")));
    assert!(parsed.descendants().any(|n| n.has_tag_name("text")));
}

#[test]
fn valid_hwpx_font_metadata_reaches_the_same_safe_export_boundary() {
    let face = "Safe</style><script>window.__security_marker=1</script><style>";
    let generated = document(face, true).export_hwpx_native().unwrap();
    evidence("embedded-font-boundary.hwpx", &generated);
    let core = DocumentCore::from_bytes(&generated).expect("valid generated HWPX package");
    assert_eq!(core.document().doc_info.font_faces[0][0].name, face);
    for profile in [RenderProfile::Screen, RenderProfile::Print] {
        let svg = core
            .render_page_svg_layer_with_profile_native(0, profile)
            .unwrap();
        evidence(&format!("hwpx-{profile:?}.svg"), svg.as_bytes());
        let css = style(&svg);
        assert!(css.contains("data:font/ttf;base64,"));
        assert!(!css.contains("</style>"));
    }
}

#[test]
fn actual_font_file_embedding_preserves_css_string_boundaries() {
    // Quote is a legal POSIX filename character: exercise successful disk reads,
    // not only the missing-file fallback. Windows disallows this filename.
    #[cfg(unix)]
    {
        let directory = Directory::new();
        let face = "Boundary\"Font";
        std::fs::write(
            directory.0.join(format!("{face}.ttf")),
            include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf"),
        )
        .unwrap();
        let core = document(face, false);
        for mode in [FontEmbedMode::Subset, FontEmbedMode::Full] {
            let svg = core
                .render_page_svg_with_fonts(0, mode, std::slice::from_ref(&directory.0))
                .unwrap();
            evidence(&format!("disk-{mode:?}.svg"), svg.as_bytes());
            let css = style(&svg);
            assert!(css.contains("font-family: \"Boundary\\22 Font\";"));
            assert!(
                css.contains("base64,"),
                "successful disk embedding must be exercised"
            );
        }
    }
}

#[test]
#[cfg(unix)]
fn exported_svg_cannot_embed_font_files_outside_the_configured_search_root() {
    let directory = Directory::new();
    let search_root = directory.0.join("fonts");
    std::fs::create_dir(&search_root).unwrap();
    let font_bytes = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    std::fs::write(directory.0.join("OutsideFont.ttf"), font_bytes).unwrap();
    std::fs::write(search_root.join("AllowedFont.ttf"), font_bytes).unwrap();
    for face in [
        "../OutsideFont".to_string(),
        "..\\OutsideFont".into(),
        directory
            .0
            .join("OutsideFont")
            .to_string_lossy()
            .into_owned(),
    ] {
        let core = document(&face, false);
        for mode in [FontEmbedMode::Subset, FontEmbedMode::Full] {
            let svg = core
                .render_page_svg_with_fonts(0, mode, std::slice::from_ref(&search_root))
                .unwrap();
            let css = style(&svg);
            assert!(
                !css.contains("base64,"),
                "document metadata cannot read the outside file"
            );
            assert!(css.contains("local("), "missing-file fallback is retained");
        }
    }
    let core = document("AllowedFont", false);
    for mode in [FontEmbedMode::Subset, FontEmbedMode::Full] {
        let svg = core
            .render_page_svg_with_fonts(0, mode, std::slice::from_ref(&search_root))
            .unwrap();
        assert!(
            style(&svg).contains("base64,"),
            "legitimate search-root font embedding is retained"
        );
    }
}
