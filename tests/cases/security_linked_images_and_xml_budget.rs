//! 관계형 보안 계약: 편집용 연결 그림 표시는 basename만 노출하고 인쇄에는 노출하지 않는다.
//! 주석 갱신: 2026-11-12T17:04:43+09:00; 직전 변경 17a69fa4e789226d529ef90caaae5a58655a1eec (2026-10-06T11:49:12+09:00).
//! 화면/인쇄 분리 근거: #8957 cover-linked·cover-mixed-images, 한글 2024 독립 Print.
//! 2026-11-12, PDFium 96 DPI raw 2px: 각 전체 1쪽 Native/fresh WASM 100.00%, 전쪽 최저 100.00%.
//! mydocs/pr/assets/issue8957/linked-print/의 확대 판독을 완료했다. 이 파일의 합성 입력은 보안 계약이며
//! 그 입력을 실물 조판 golden으로 주장하지 않는다. XML 예산과 비이미지 거부 계약은 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::image::{ImageAttr, Picture};
use rhwp::model::shape::CommonObjAttr;
use rhwp::parser::hwpx::reader::HwpxReader;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::svg::SvgRenderer;
use rhwp::DocumentCore;
use std::io::{Cursor, Write};

struct Directory(std::path::PathBuf);
impl Directory {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("rhwp-private-image-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn linked(path: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "X").unwrap();
    let mut doc = core.document().clone();
    doc.sections[0].paragraphs[0]
        .controls
        .push(Control::Picture(Box::new(Picture {
            common: CommonObjAttr {
                width: 7200,
                height: 7200,
                treat_as_char: true,
                ..Default::default()
            },
            image_attr: ImageAttr {
                external_path: Some(path.into()),
                bin_data_id: 1,
                ..Default::default()
            },
            ..Default::default()
        })));
    core.set_document(doc);
    core
}

fn png() -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(1, 1)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    bytes.into_inner()
}

#[test]
fn nonimage_sidecar_is_never_loaded_or_exported_as_a_linked_picture() {
    let directory = Directory::new();
    std::fs::write(directory.0.join("secret.png"), b"PRIVATE_NONIMAGE_CONTENT").unwrap();
    let mut core = linked(r"C:\OriginalOwner\secret.png");
    assert_eq!(core.populate_external_images_from_dir(&directory.0), 0);
    assert!(core.document().bin_data_content.is_empty());
    let svg = core.render_page_svg_native(0).unwrap();
    assert!(!svg.contains("PRIVATE_NONIMAGE_CONTENT"));
    assert!(!svg.contains("data:application/octet-stream"));
}

#[test]
fn valid_linked_picture_is_loaded_and_retained_in_the_final_svg() {
    let directory = Directory::new();
    std::fs::write(directory.0.join("photo.png"), png()).unwrap();
    let mut core = linked(r"C:\OriginalOwner\photo.png");
    assert_eq!(core.populate_external_images_from_dir(&directory.0), 1);
    let svg = core.render_page_svg_native(0).unwrap();
    assert!(
        svg.contains("data:image/png;base64,"),
        "normal picture remains visible"
    );
}

fn mark_missing(node: &mut RenderNode, path: &str) -> usize {
    let mut count = 0;
    if let RenderNodeType::Image(image) = &mut node.node_type {
        image.data = None;
        image.external_path = Some(path.into());
        count += 1;
    }
    count
        + node
            .children
            .iter_mut()
            .map(|child| mark_missing(child, path))
            .sum::<usize>()
}

#[test]
fn missing_picture_label_exposes_basename_without_author_directory() {
    let directory = Directory::new();
    std::fs::write(directory.0.join("photo.png"), png()).unwrap();
    let mut core = linked("photo.png");
    core.populate_external_images_from_dir(&directory.0);
    for path in [
        r"C:\PrivateOwner\Project\photo.png",
        "/home/PrivateOwner/Project/photo.png",
    ] {
        let mut tree = core.build_page_render_tree(0).unwrap();
        assert!(mark_missing(&mut tree.root, path) > 0);
        let mut renderer = SvgRenderer::new();
        renderer.show_missing_picture_placeholder = true;
        renderer.render_tree(&tree);
        let svg = renderer.output();
        assert!(!svg.contains("PrivateOwner"));
        assert!(svg.contains("[외부: photo.png]"));
        roxmltree::Document::parse(svg).unwrap();
        let mut print = SvgRenderer::new();
        print.render_tree(&tree);
        assert!(!print.output().contains("PrivateOwner"));
        assert!(!print.output().contains("[외부:"));
    }
}

#[test]
fn xml_reads_cannot_repeat_the_per_entry_allowance_without_a_document_budget() {
    // Independent policy: 512MiB cumulative XML, retaining the existing 256MiB
    // per-entry allowance. Stream the generator; each XML is only 64MiB.
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(1));
    let chunk = vec![b' '; 1024 * 1024];
    for entry in 0..9 {
        writer
            .start_file(format!("section{entry}.xml"), options)
            .unwrap();
        for _ in 0..64 {
            writer.write_all(&chunk).unwrap();
        }
    }
    let package = writer.finish().unwrap().into_inner();
    let mut reader = HwpxReader::open(&package).unwrap();
    for entry in 0..8 {
        assert_eq!(
            reader
                .read_file(&format!("section{entry}.xml"))
                .unwrap()
                .len(),
            64 * 1024 * 1024
        );
    }
    match reader.read_file("section8.xml") {
        Err(error) => assert!(error.to_string().contains("XML read budget"), "{error}"),
        Ok(_) => panic!("ninth entry exceeds the 512MiB document XML allowance"),
    }
}
