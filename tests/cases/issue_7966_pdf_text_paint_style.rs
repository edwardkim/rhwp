//! PDF source paint styles must survive contiguous text assembly and both save formats.
//! Source p1: Website: is black, https://ostday.kr is RGB(5,99,193).
//! Independent Poppler SVG uses rgb(1.960754%,38.822937%,75.686646%).
#![cfg(feature = "pdf-import")]
use rhwp::{
    model::{control::Control, shape::ShapeObject},
    DocumentCore,
};
const PDF: &[u8] = include_bytes!("../../samples/issue7959/bounded-import-path.pdf");
fn word_colour<T: PartialEq + std::fmt::Debug>(records: &[(char, T)], word: &str, expected: &T) {
    let letters: Vec<char> = word.chars().collect();
    let matches: Vec<_> = records
        .windows(letters.len())
        .filter(|w| w.iter().map(|v| v.0).eq(letters.iter().copied()))
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "source word must occur exactly once: {word}"
    );
    for (_, colour) in matches[0] {
        assert_eq!(
            colour, expected,
            "source paint style was lost inside {word}"
        );
    }
}
fn check(core: &DocumentCore) {
    assert_eq!(core.page_count(), 4);
    let document = core.document();
    let mut records = Vec::new();
    for c in document.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
    {
        let Control::Shape(s) = c else { continue };
        let ShapeObject::Rectangle(r) = s.as_ref() else {
            continue;
        };
        let Some(text_box) = r.drawing.text_box.as_ref() else {
            continue;
        };
        for p in &text_box.paragraphs {
            for (i, c) in p.text.chars().enumerate() {
                let style_id = p.char_shape_id_at(i).unwrap();
                records.push((
                    c,
                    document.doc_info.char_shapes[style_id as usize].text_color & 0x00ff_ffff,
                ));
            }
        }
    }
    word_colour(&records, "Website:", &0u32);
    word_colour(&records, "https://ostday.kr", &0x00c1_6305u32);
    let svg = core.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let records: Vec<_> = xml
        .descendants()
        .filter(|n| n.is_text() && n.ancestors().any(|a| a.has_tag_name("text")))
        .flat_map(|n| {
            let fill = n
                .ancestors()
                .find_map(|a| a.attribute("fill"))
                .unwrap_or("#000000")
                .to_ascii_lowercase();
            n.text().unwrap().chars().map(move |c| (c, fill.clone()))
        })
        .collect();
    word_colour(&records, "Website:", &"#000000".to_string());
    word_colour(&records, "https://ostday.kr", &"#0563c1".to_string());
}
#[test]
fn mixed_text_colours_survive_positional_lowering_and_both_saved_formats() {
    let core = DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
    check(&core);
    for saved in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        check(&DocumentCore::from_bytes(&saved).unwrap());
    }
}
#[test]
fn cli_mixed_text_styles_preserve_each_source_colour() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let folder =
        std::env::temp_dir().join(format!("rhwp-pdf-style-{}-{stamp}", std::process::id()));
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    std::fs::create_dir(&folder).unwrap();
    let scratch = Scratch(folder);
    let input = scratch.0.join("input.pdf");
    let output = scratch.0.join("imported.hwpx");
    std::fs::write(&input, PDF).unwrap();
    let binary =
        std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
    let result = std::process::Command::new(binary)
        .arg("import-pdf")
        .arg(input)
        .arg(&output)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    check(&DocumentCore::from_bytes(&std::fs::read(output).unwrap()).unwrap());
}
