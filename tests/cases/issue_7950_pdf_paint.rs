//! PDF paint contracts exercised through the CLI and saved HWPX.
//! Independent PDF/Poppler and Native/fresh WASM evidence: task_m100_7949_stage6.
#![cfg(all(feature = "pdf-import", not(target_arch = "wasm32")))]

use rhwp::model::{control::Control, document::Document};
use std::process::Command;

fn pdf(content: &str) -> Vec<u8> {
    let mut objects: Vec<Vec<u8>> = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Resources << /Font << /F1 5 0 R >> /ExtGState << /Hidden 6 0 R /Visible 7 0 R >> /XObject << /Im 8 0 R >> >> /Contents 4 0 R >>".to_owned(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        "<< /Type /ExtGState /ca 0 /CA 0 >>".to_owned(),
        "<< /Type /ExtGState /ca 1 /CA 1 >>".to_owned(),
    ].into_iter().map(String::into_bytes).collect();
    objects.push([b"<< /Subtype /Image /Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Length 3 >>\nstream\n".as_slice(), b"\xff\0\0", b"\nendstream"].concat());
    let mut bytes = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        bytes.extend_from_slice(object);
        bytes.extend_from_slice(b"\nendobj\n");
    }
    let start = bytes.len();
    bytes.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    bytes
}

struct Scratch(std::path::PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn imported(content: &str) -> Document {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let scratch = Scratch(
        std::env::temp_dir().join(format!("rhwp-pdf-paint-{}-{stamp}", std::process::id())),
    );
    std::fs::create_dir_all(&scratch.0).unwrap();
    let input = scratch.0.join("paint.pdf");
    let output = scratch.0.join("paint.hwpx");
    std::fs::write(&input, pdf(content)).unwrap();
    let binary =
        std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
    let result = Command::new(binary)
        .arg("import-pdf")
        .arg(&input)
        .arg(&output)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    rhwp::parser::parse_document(&std::fs::read(output).unwrap()).unwrap()
}

fn controls(doc: &Document) -> impl Iterator<Item = &Control> {
    doc.sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
}

#[test]
fn fill_and_visible_text_survive_save_while_zero_alpha_paint_does_not_reappear() {
    let doc = imported("0 0.2 0.8 rg 10 10 280 180 re f q /Hidden gs BT /F1 24 Tf 30 100 Td (HIDDEN) Tj ET Q /Visible gs 1 1 1 rg BT /F1 24 Tf 30 130 Td (VISIBLE) Tj ET q /Hidden gs 0 0 0 rg 15 15 100 100 re f Q");
    let text: String = controls(&doc)
        .filter_map(|c| match c {
            Control::Shape(s) => s.drawing().and_then(|d| d.text_box.as_ref()),
            _ => None,
        })
        .flat_map(|b| &b.paragraphs)
        .map(|p| p.text.as_str())
        .collect();
    assert!(text.contains("VISIBLE"));
    assert!(
        !text.contains("HIDDEN"),
        "zero-alpha OCR text must not become visible text"
    );
    let colors: Vec<_> = controls(&doc)
        .filter_map(|c| match c {
            Control::Shape(s) => s
                .drawing()
                .and_then(|d| d.fill.solid.as_ref())
                .map(|f| f.background_color),
            _ => None,
        })
        .collect();
    assert!(
        colors.contains(&0x00cc3300),
        "the original blue fill must survive the CLI save"
    );
    assert!(
        !colors.contains(&0),
        "the hidden black rectangle must not cover visible paint"
    );
}

#[test]
fn compound_even_odd_hole_and_bezier_stroke_keep_alpha_in_saved_pictures() {
    let doc = imported("0.2 0.4 0.8 rg 10 10 280 180 re f 1 1 1 rg 40 40 100 100 re 60 60 60 60 re f* 0 0 0 RG 2 w 170 50 m 170 150 260 150 260 50 c S");
    let pictures: Vec<_> = controls(&doc)
        .filter_map(|c| match c {
            Control::Picture(p) => Some(p),
            _ => None,
        })
        .collect();
    assert_eq!(
        pictures.len(),
        2,
        "compound fill and curve are distinct source paints"
    );
    let images: Vec<_> = pictures
        .iter()
        .map(|picture| {
            let descriptor =
                &doc.doc_info.bin_data_list[usize::from(picture.image_attr.bin_data_id) - 1];
            let data = doc
                .bin_data_content
                .iter()
                .find(|c| c.id == descriptor.storage_id)
                .unwrap();
            image::load_from_memory(&data.data.load())
                .unwrap()
                .into_rgba8()
        })
        .collect();
    let fill = images
        .iter()
        .find(|im| im.pixels().any(|p| p.0 == [255, 255, 255, 255]))
        .expect("white compound fill retained");
    assert_eq!(
        fill.get_pixel(fill.width() / 2, fill.height() / 2)[3],
        0,
        "even-odd centre is transparent"
    );
    assert_eq!(
        fill.get_pixel(fill.width() / 10, fill.height() / 10).0,
        [255, 255, 255, 255],
        "outer ring is opaque"
    );
    assert!(
        images.iter().any(|im| im
            .pixels()
            .any(|p| p[3] > 0 && p[0] == 0 && p[1] == 0 && p[2] == 0)),
        "Bezier stroke retained"
    );
}

#[test]
fn path_image_path_source_order_survives_lowering_and_save() {
    let doc = imported("0 0 1 rg 10 10 280 180 re f q 100 0 0 100 50 50 cm /Im Do Q 1 1 1 rg 80 80 m 140 80 l 140 140 l 80 140 l h f");
    let color_order = |color| {
        controls(&doc)
            .find_map(|c| match c {
                Control::Shape(s)
                    if s.drawing()
                        .and_then(|d| d.fill.solid.as_ref())
                        .is_some_and(|f| f.background_color == color) =>
                {
                    Some(s.common().z_order)
                }
                _ => None,
            })
            .expect("source fill retained")
    };
    let image_order = controls(&doc)
        .find_map(|c| match c {
            Control::Picture(p) => Some(p.common.z_order),
            _ => None,
        })
        .expect("source image retained");
    assert!(
        color_order(0x00ff0000) < image_order,
        "blue backdrop precedes red image"
    );
    assert!(
        image_order < color_order(0x00ffffff),
        "white foreground follows red image"
    );
}
