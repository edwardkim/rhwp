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

#[cfg(feature = "native-skia")]
#[test]
fn inferred_table_cannot_cover_its_labels_with_retained_positional_paint() {
    let grid = "0 0 0 RG 1 w 40 40 m 260 40 l S 40 100 m 260 100 l S 40 160 m 260 160 l S 40 40 m 40 160 l S 150 40 m 150 160 l S 260 40 m 260 160 l S ";
    let background = "0 0.5 1 rg 40 100 220 60 re f ";
    let text = "0 0 0 rg BT /F1 12 Tf 70 130 Td (H1) Tj ET BT /F1 12 Tf 180 130 Td (H2) Tj ET BT /F1 12 Tf 70 70 Td (B1) Tj ET BT /F1 12 Tf 180 70 Td (B2) Tj ET ";
    let decoration = "1 0 0 rg 10 180 10 10 re f ";
    let cases = [
        (
            "background-before-grid",
            format!("{background}{grid}{decoration}{text}"),
            true,
        ),
        (
            "background-after-grid",
            format!("{grid}{background}{text}"),
            true,
        ),
        ("plain-table", format!("{grid}{decoration}{text}"), false),
    ];
    let binary =
        std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").into());
    for (name, content, colored) in cases {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let scratch = Scratch(
            std::env::temp_dir().join(format!("rhwp-table-paint-{}-{stamp}", std::process::id())),
        );
        std::fs::create_dir_all(&scratch.0).unwrap();
        let source = scratch.0.join("source.pdf");
        let saved = scratch.0.join("saved.hwpx");
        std::fs::write(&source, pdf(&content)).unwrap();
        let result = Command::new(&binary)
            .arg("import-pdf")
            .arg(&source)
            .arg(&saved)
            .arg("--json")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let document = rhwp::parser::parse_document(&std::fs::read(&saved).unwrap()).unwrap();
        if !colored {
            assert_eq!(
                controls(&document)
                    .filter(|c| matches!(c, Control::Table(_)))
                    .count(),
                1,
                "disjoint decoration must not disable sound table inference"
            );
        }
        let output = scratch.0.join("png");
        let rendered = Command::new(&binary)
            .arg("export-png")
            .arg(&saved)
            .args(["--profile", "print", "--dpi", "96", "--json", "-o"])
            .arg(&output)
            .output()
            .unwrap();
        assert!(
            rendered.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&rendered.stderr)
        );
        let pngs: Vec<_> = std::fs::read_dir(&output)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "png"))
            .collect();
        assert_eq!(pngs.len(), 1, "{name}: all cells belong to one source page");
        let image = image::open(&pngs[0]).unwrap().into_rgba8();
        // Independent input geometry: MediaBox 300x200 pt, columns 40/150/260,
        // rows 40/100/160. Exclude the grid strokes and inspect each cell's
        // interior. No expected coordinate is taken from the lowered model.
        let scale = f64::from(image.width()) / 300.0;
        for (bottom, top) in [(40.0, 100.0), (100.0, 160.0)] {
            for (left, right) in [(40.0, 150.0), (150.0, 260.0)] {
                let x0 = ((left + 10.0) * scale).ceil() as u32;
                let x1 = ((right - 10.0) * scale).floor() as u32;
                let y0 = ((200.0 - top + 10.0) * scale).ceil() as u32;
                let y1 = ((200.0 - bottom - 10.0) * scale).floor() as u32;
                let mut ink = 0;
                let mut blue = 0;
                for y in y0..y1 {
                    for x in x0..x1 {
                        let p = image.get_pixel(x, y);
                        ink += usize::from(p[3] > 128 && p[0] < 80 && p[1] < 80 && p[2] < 80);
                        blue += usize::from(
                            p[3] > 250 && p[0] < 3 && p[1].abs_diff(128) < 3 && p[2] > 252,
                        );
                    }
                }
                assert!(ink > 0, "{name}: visible label missing inside source cell {left}..{right}/{bottom}..{top}");
                if colored && bottom == 100.0 {
                    assert!(blue > 0, "{name}: original blue cell background missing");
                }
            }
        }
    }
}
