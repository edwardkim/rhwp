//! PDF source-sample masks and sampling survive the public import/save path.
//! Independent input PDF/MuPDF/Native/Docker WASM evidence: #7974 stage 1.
#![cfg(all(feature = "pdf-import", not(target_arch = "wasm32")))]
use rhwp::DocumentCore;

fn stream(fields: &str, bytes: &[u8]) -> Vec<u8> {
    [
        format!("<< {fields} /Length {} >>\nstream\n", bytes.len()).as_bytes(),
        bytes,
        b"\nendstream",
    ]
    .concat()
}
fn pdf(content: &[u8], resources: &str, extra: Vec<Vec<u8>>) -> Vec<u8> {
    let mut objects=vec![b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_vec(),format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Resources << {resources} >> /Contents 4 0 R >>").into_bytes(),stream("",content)];
    objects.extend(extra);
    let mut bytes = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend(format!("{} 0 obj\n", i + 1).as_bytes());
        bytes.extend(o);
        bytes.extend(b"\nendobj\n");
    }
    let xref = bytes.len();
    bytes.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for n in offsets {
        bytes.extend(format!("{n:010} 00000 n \n").as_bytes());
    }
    bytes.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    bytes
}
fn image(fields: &str, pixels: &[u8]) -> Vec<u8> {
    pdf(
        b"0 1 1 rg 0 0 300 200 re f q 180 0 0 120 40 40 cm /Im Do Q",
        "/XObject << /Im 5 0 R >>",
        vec![stream(
            &format!("/Type /XObject /Subtype /Image {fields}"),
            pixels,
        )],
    )
}

fn fixture(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7974")
        .join(format!("{name}.pdf"));
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn textures(core: &DocumentCore) -> Vec<image::RgbaImage> {
    core.document()
        .bin_data_content
        .iter()
        .map(|b| {
            image::load_from_memory(&b.data.load())
                .unwrap()
                .into_rgba8()
        })
        .collect()
}
fn saved(core: &DocumentCore) -> Vec<DocumentCore> {
    [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ]
    .iter()
    .map(|b| DocumentCore::from_bytes(b).unwrap())
    .collect()
}
#[test]
fn colour_key_compares_packed_samples_before_decode_and_preserves_alpha_on_save() {
    let core = fixture("gray-packed-decode");
    for doc in std::iter::once(&core).chain(saved(&core).iter()) {
        let t = textures(doc);
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].dimensions(), (4, 1));
        assert_eq!(
            t[0].pixels().map(|p| p[3]).collect::<Vec<_>>(),
            [255, 0, 0, 255]
        );
        assert_eq!(t[0].get_pixel(0, 0).0, [255, 255, 255, 255]);
        assert_eq!(t[0].get_pixel(3, 0).0, [0, 0, 0, 255]);
    }
}
#[test]
fn indexed_colour_key_does_not_mask_another_index_with_the_same_rgb() {
    let core = fixture("indexed-key");
    let t = textures(&core);
    assert_eq!(
        t[0].pixels().map(|p| p[3]).collect::<Vec<_>>(),
        [255, 0, 255, 255]
    );
    assert_eq!(&t[0].get_pixel(1, 0).0[..3], &t[0].get_pixel(2, 0).0[..3]);
}
#[test]
fn sixteen_bit_colour_key_distinguishes_samples_that_round_to_the_same_gray() {
    let core = fixture("gray16-key");
    let t = textures(&core);
    assert_eq!(t[0].pixels().map(|p| p[3]).collect::<Vec<_>>(), [255, 0]);
}
#[test]
fn stencil_form_reuse_retains_each_invocations_current_colour() {
    let core = fixture("stencil-form-reuse");
    for doc in std::iter::once(&core).chain(saved(&core).iter()) {
        let t = textures(doc);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(t[1].get_pixel(0, 0).0, [0, 255, 0, 255]);
        assert_eq!(t[0].get_pixel(1, 0)[3], 0);
        assert_eq!(t[1].get_pixel(1, 0)[3], 0);
    }
}
#[test]
fn stencil_reverse_decode_preserves_non_byte_width_rows_and_alpha() {
    let reversed = fixture("stencil-reverse-padding");
    let t = textures(&reversed);
    assert_eq!(t[0].dimensions(), (9, 2));
    assert_eq!(t[0].get_pixel(0, 0).0, [255, 0, 255, 255]);
    assert_eq!(t[0].get_pixel(1, 1).0, [255, 0, 255, 255]);
    assert_eq!(t[0].pixels().filter(|p| p[3] != 0).count(), 2);
    let half = fixture("stencil-alpha");
    assert_eq!(textures(&half)[0].get_pixel(0, 0).0, [255, 0, 255, 128]);
}
#[test]
fn malformed_masks_are_rejected_without_breaking_the_next_document() {
    for fields in [
        "/Width 2 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray /Mask [0]",
        "/Width 2 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray /Mask [256 256]",
        "/Width 2 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray /Mask [255 0]",
        "/Width 1000000000 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray /Mask [0 0]",
    ] {
        assert!(DocumentCore::from_bytes(&image(fields, &[0, 255])).is_err());
    }
    assert!(DocumentCore::from_bytes(&image(
        "/Width 2 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray /Mask [0 0]",
        &[255]
    ))
    .is_err());
    assert_eq!(
        DocumentCore::from_bytes(&image(
            "/Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray",
            &[255]
        ))
        .unwrap()
        .page_count(),
        1
    );
}
#[test]
fn sampling_declaration_survives_both_save_formats_and_svg_replay() {
    for (name, rendering) in [
        ("unmasked-control", "pixelated"),
        ("linear-control", "auto"),
    ] {
        let core = fixture(name);
        for doc in std::iter::once(&core).chain(saved(&core).iter()) {
            let svg = doc.render_page_svg_native(0).unwrap();
            assert!(
                svg.contains(&format!("image-rendering:{rendering}")),
                "{rendering} lost through import/save/reopen"
            );
            assert_eq!(textures(doc)[0].dimensions(), (2, 2));
        }
    }
    #[cfg(feature = "native-skia")]
    {
        // Every interior destination footprint covers both black and white
        // source samples. Point sampling loses one of them; coverage filtering
        // produces intermediate ink. Do not pin a backend-specific gray value
        // or document pixel coordinate (PDF 32000-1 8.9.5.3 leaves the kernel open).
        let core = fixture("minification-control");
        for doc in std::iter::once(&core).chain(saved(&core).iter()) {
            let bytes = doc
                .render_page_png_native_with_profile_and_export_options(
                    0,
                    rhwp::paint::RenderProfile::Print,
                    &rhwp::document_core::queries::rendering::PngExportOptions {
                        dpi: Some(96.),
                        ..Default::default()
                    },
                )
                .unwrap();
            let output = image::load_from_memory(&bytes).unwrap().into_rgb8();
            let ink = output.pixels().filter(|p| p.0 != [255; 3]).count();
            let mixed = output
                .pixels()
                .filter(|p| p[0] > 0 && p[0] < 255 && p[0] == p[1] && p[1] == p[2])
                .count();
            assert!(
                ink > 0 && mixed > ink / 2,
                "minification discarded source coverage"
            );
        }
    }
}

#[test]
fn smask_has_precedence_over_colour_key_and_rgb_uses_all_components() {
    let core = fixture("smask-overrides-key");
    assert_eq!(
        textures(&core)[0]
            .pixels()
            .map(|p| p[3])
            .collect::<Vec<_>>(),
        [0, 255]
    );
    let core = fixture("rgb-all-components");
    assert_eq!(
        textures(&core)[0]
            .pixels()
            .map(|p| p[3])
            .collect::<Vec<_>>(),
        [0, 255]
    );
    assert_eq!(textures(&core)[0].get_pixel(1, 0).0, [255, 255, 0, 255]);
}
