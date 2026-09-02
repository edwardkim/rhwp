//! [#6310] `hc:imgBrush mode="ZOOM"` 은 칸 상자에 맞춰 종횡비를 지키며 그린다.
//!
//! 종전엔 ZOOM 이 매핑되지 않아 TILE 로 붕괴했고, 원본 픽셀을 96dpi 로 환산한
//! 크기로 그려 칸 clip 안에 흰 여백만 남았다 (156745900 로고 8.33배).
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::style::ImageFillMode;
use rhwp::parser::hwpx::header::parse_hwpx_header;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const SAMPLE: &str = "samples/tac-host-spacing.hwpx";

/// 1×1 RGB PNG. 크기는 중요하지 않다 — ZOOM 은 칸 bbox 에 meet 로 얹는다.
const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00,
    0x00, 0x00, 0x03, 0x00, 0x01, 0x00, 0x05, 0xfe, 0xd4, 0xef, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

fn zoom_header_xml() -> String {
    r##"<?xml version="1.0" encoding="UTF-8"?>
<hh:head xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head"
         xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
  <hh:refList>
    <hh:borderFills itemCnt="1">
      <hh:borderFill id="889">
        <hh:fillBrush>
          <hc:imgBrush mode="ZOOM">
            <hc:img binaryItemIDRef="image1" bright="0" contrast="0" effect="REAL_PIC" alpha="0"/>
          </hc:imgBrush>
        </hh:fillBrush>
      </hh:borderFill>
    </hh:borderFills>
  </hh:refList>
</hh:head>"##
        .to_string()
}

fn zoom_hwpx_bytes() -> Vec<u8> {
    let src = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("sample");
    let mut archive = ZipArchive::new(Cursor::new(&src)).expect("open zip");
    let mut out = Cursor::new(Vec::new());
    {
        let mut writer = ZipWriter::new(&mut out);
        let deflate = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).expect("zip entry");
            let name = entry.name().to_string();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).expect("read zip entry");
            if name == "Contents/header.xml" {
                let xml = String::from_utf8(data).expect("header utf-8");
                let xml = xml.replace(
                    "</hh:borderFills>",
                    r##"<hh:borderFill id="3" threeD="0" shadow="0" centerLine="NONE" breakCellSeparateLine="0"><hh:slash type="NONE" Crooked="0" isCounter="0"/><hh:backSlash type="NONE" Crooked="0" isCounter="0"/><hh:leftBorder type="NONE" width="0.1 mm" color="#000000"/><hh:rightBorder type="NONE" width="0.1 mm" color="#000000"/><hh:topBorder type="NONE" width="0.1 mm" color="#000000"/><hh:bottomBorder type="NONE" width="0.1 mm" color="#000000"/><hh:diagonal type="SOLID" width="0.1 mm" color="#000000"/><hh:fillBrush><hc:imgBrush mode="ZOOM"><hc:img binaryItemIDRef="image1" bright="0" contrast="0" effect="REAL_PIC" alpha="0"/></hc:imgBrush></hh:fillBrush></hh:borderFill></hh:borderFills>"##,
                );
                let xml = xml.replace("itemCnt=\"2\"", "itemCnt=\"3\"");
                writer.start_file(name, deflate).expect("start");
                writer.write_all(xml.as_bytes()).expect("write");
            } else if name == "Contents/section0.xml" {
                let xml = String::from_utf8(data).expect("section utf-8");
                let xml = xml.replace("borderFillIDRef=\"2\"", "borderFillIDRef=\"3\"");
                writer.start_file(name, deflate).expect("start");
                writer.write_all(xml.as_bytes()).expect("write");
            } else if name == "Contents/content.hpf" {
                let xml = String::from_utf8(data).expect("hpf utf-8");
                let xml = xml.replace(
                    "</opf:manifest>",
                    r#"<opf:item id="image1" href="BinData/image1.png" media-type="image/png"/></opf:manifest>"#,
                );
                writer.start_file(name, deflate).expect("start");
                writer.write_all(xml.as_bytes()).expect("write");
            } else if name == "mimetype" {
                writer.start_file(name, stored).expect("start");
                writer.write_all(&data).expect("write");
            } else {
                writer.start_file(name, deflate).expect("start");
                writer.write_all(&data).expect("write");
            }
        }
        writer
            .start_file("BinData/image1.png", deflate)
            .expect("png");
        writer.write_all(PNG_1X1).expect("png bytes");
        writer.finish().expect("finish zip");
    }
    out.into_inner()
}

#[test]
fn header_imgbrush_zoom_is_not_collapsed_to_tile() {
    let (doc_info, _) = parse_hwpx_header(&zoom_header_xml()).expect("header");
    let img = doc_info
        .border_fills
        .first()
        .and_then(|bf| bf.fill.image.as_ref())
        .expect("imgBrush");
    assert_eq!(
        img.fill_mode,
        ImageFillMode::Zoom,
        "ZOOM 이 TILE 로 붕괴하면 원본 픽셀 크기로 그려진다"
    );
    assert_eq!(img.bin_data_id, 1);
}

#[test]
fn zoom_cell_fill_svg_meets_the_cell_box() {
    let bytes = zoom_hwpx_bytes();
    let core = DocumentCore::from_bytes(&bytes).expect("open zoom hwpx");
    let fills: Vec<_> = core
        .document()
        .doc_info
        .border_fills
        .iter()
        .filter_map(|bf| bf.fill.image.as_ref())
        .collect();
    assert!(
        fills.iter().any(|img| img.fill_mode == ImageFillMode::Zoom),
        "문서의 imgBrush ZOOM 이 파싱되어야 한다: {:?}",
        fills.iter().map(|i| i.fill_mode).collect::<Vec<_>>()
    );

    let svg = core.render_page_svg_native(0).expect("svg");
    assert!(
        svg.contains("preserveAspectRatio=\"xMidYMid meet\""),
        "ZOOM 채우기는 칸 bbox 에 meet 로 얹혀야 한다. TILE 원본 픽셀 배치는 none+native size 다: {svg}"
    );
}
