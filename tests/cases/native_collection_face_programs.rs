//! Same source programs in a TTC and standalone fixtures must paint identically.
//! Synthetic font identity equivalence, not a Hancom document pixel golden.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::{native_host::NativeDocumentSession, DocumentCore};
use std::path::{Path, PathBuf};

fn paint(source: &Path, italic: bool) -> rhwp::renderer::raster_pixels::RasterPixels {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "A").unwrap();
    let mut document = core.document().clone();
    for language in &mut document.doc_info.font_faces {
        for font in language {
            font.name = "RHWP Host Fixture".into();
        }
    }
    for shape in &mut document.doc_info.char_shapes {
        shape.italic = italic;
    }
    core.set_document(document);
    let mut session = NativeDocumentSession::from_core(core);
    session
        .set_font_sources(&[PathBuf::from(source)], None)
        .unwrap();
    session.render_page(0, 1.0).unwrap().pixels
}

#[test]
fn collection_style_faces_paint_their_standalone_source_programs() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts");
    let collection = directory.join("RHWPHostFixture.ttc");
    for (italic, name) in [(false, "Regular"), (true, "Italic")] {
        let standalone = directory.join(format!("RHWPHostFixture-{name}.ttf"));
        // These fixtures retain the same A contours and horizontal metrics;
        // only the container and additional characters differ.
        assert_eq!(
            paint(&collection, italic),
            paint(&standalone, italic),
            "TTC {name} must preserve the selected source face, not replace it with face zero"
        );
    }
}
