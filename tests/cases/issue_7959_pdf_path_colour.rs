//! PDF solid-colour operators: input-defined RGB and graphics-state ownership.
//! Generated 3-page PDF, independent Poppler baseline; no coordinate/hash pins.
#![cfg(feature = "pdf-import")]

use pdf_oxide::{elements::PathContent, layout::Color, PdfDocument};

const PDF: &[u8] = include_bytes!("../../pdf/issue7959/path-colour-controls.pdf");

fn color(c: Option<Color>) -> Option<(u8, u8, u8)> {
    c.map(|c| {
        (
            (c.r * 255.0).round() as u8,
            (c.g * 255.0).round() as u8,
            (c.b * 255.0).round() as u8,
        )
    })
}

fn check_paths(
    paths: &[PathContent],
    fills: &[Option<(u8, u8, u8)>],
    strokes: &[Option<(u8, u8, u8)>],
) {
    assert_eq!(
        paths.len(),
        fills.len(),
        "preserve each painted object's ownership/order"
    );
    assert_eq!(
        paths
            .iter()
            .map(|p| color(p.fill_color))
            .collect::<Vec<_>>(),
        fills
    );
    assert_eq!(
        paths
            .iter()
            .map(|p| color(p.stroke_color))
            .collect::<Vec<_>>(),
        strokes
    );
}

#[test]
fn named_path_spaces_preserve_device_resets_and_q_q_in_page_and_form_scopes() {
    let black = Some((0, 0, 0));
    let white = Some((255, 255, 255));
    let red = Some((255, 0, 0));
    let green = Some((0, 255, 0));
    let blue = Some((0, 0, 255));
    // Both entry points must retain cs/sc inside BT/ET as well as page paint.
    for bounded in [false, true] {
        let doc = if bounded {
            PdfDocument::from_bytes_with_limits(PDF.to_vec(), Default::default())
        } else {
            PdfDocument::from_bytes(PDF.to_vec())
        }
        .unwrap();
        check_paths(
            &doc.extract_paths(0).unwrap(),
            &[
                black,
                white,
                Some((128, 128, 128)),
                red,
                Some((64, 64, 64)),
                green,
                Some((191, 191, 191)),
                red,
            ],
            &[None; 8],
        );
        check_paths(
            &doc.extract_paths(1).unwrap(),
            &[black, white, red, white, None],
            &[None, None, None, None, red],
        );
        // ICC N=3 + declared DeviceRGB Alternate. Inner Gray space must not
        // change the colour space restored by Q; SCN must reach stroke paint.
        check_paths(
            &doc.extract_paths(2).unwrap(),
            &[black, white, green, black, red, None],
            &[None, None, None, None, None, blue],
        );
        doc.check_import_budget().unwrap();
    }
}

#[test]
fn named_path_colours_survive_lowering_and_final_svg_paint() {
    let core = rhwp::DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
    assert_eq!(core.page_count(), 3);
    let page = core.render_page_svg_native(0).unwrap();
    for expected in [
        "#000000", "#ffffff", "#808080", "#ff0000", "#404040", "#00ff00", "#bfbfbf",
    ] {
        assert!(
            page.to_ascii_lowercase().contains(expected),
            "missing input-defined paint {expected}"
        );
    }
    let form = core.render_page_svg_native(1).unwrap().to_ascii_lowercase();
    assert!(
        form.contains("#ff0000"),
        "Form's Gray name scope leaked into parent RGB space"
    );
    let icc = core.render_page_svg_native(2).unwrap().to_ascii_lowercase();
    assert!(icc.contains("#00ff00") && icc.contains("#ff0000"));
}
