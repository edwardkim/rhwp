//! 관계형: Native PNG의 명시 공급 글꼴이 WMF의 실제 paint에도 전달된다.
//! 주석 갱신: 2026-11-12T00:52:45+09:00. 합성 공급 계약이며 실물 좌표·golden을 고정하지 않는다.
//! 관련 실제 입력: samples/hwp3-sample16-hwp5-2022.hwp, 독립 한컴 2020 Print PDFium.
//! 2026-11-12 96 DPI·색상 보정 전 2px raw: 35쪽 Native/fresh WASM 모두 100.00000%.
//! 전 64쪽 최저 Native 99.99717%, fresh WASM 100.00000%.
//! 근거: mydocs/pr/assets/pr8896/protection/ 및 font-supply/README.md.
//! 합성 윤곽은 저장소 소유 MIT 합성 fixture만 사용하며 호스트 글꼴 설치를 요구하지 않는다.
#![cfg(feature = "native-skia")]

use rhwp::document_core::queries::png_export::NativePngExporter;
use rhwp::document_core::queries::rendering::PngExportOptions;
use rhwp::{
    model::{
        bin_data::BinDataContent,
        control::Control,
        image::Picture,
        paragraph::{CharShapeRef, LineSeg, Paragraph},
        shape::CommonObjAttr,
    },
    paint::RenderProfile,
    DocumentCore,
};
const SOURCE_FAMILY: &str = "RHWP Host Fixture";
const FONT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fonts/RHWPHostFixture-Regular.ttf"
));

// 이름만 고유하게 바꿔 시스템·다른 병렬 검사의 동일 family 설치와 격리한다.
// glyph·cmap·hmtx는 저장소 소유 합성 프로그램 그대로이며 정상 SFNT checksum을 유지한다.
fn isolated_program(family: &str) -> Vec<u8> {
    assert_eq!(family.len(), SOURCE_FAMILY.len());
    let mut bytes = FONT.to_vec();
    for (from, to) in [
        (
            SOURCE_FAMILY.as_bytes().to_vec(),
            family.as_bytes().to_vec(),
        ),
        (
            SOURCE_FAMILY
                .encode_utf16()
                .flat_map(u16::to_be_bytes)
                .collect(),
            family.encode_utf16().flat_map(u16::to_be_bytes).collect(),
        ),
    ] {
        let positions: Vec<_> = bytes
            .windows(from.len())
            .enumerate()
            .filter_map(|(index, data)| (data == from).then_some(index))
            .collect();
        for index in positions {
            bytes[index..index + to.len()].copy_from_slice(&to);
        }
    }
    fn checksum(data: &[u8]) -> u32 {
        data.chunks(4).fold(0u32, |sum, part| {
            let mut word = [0u8; 4];
            word[..part.len()].copy_from_slice(part);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
    }
    let tables = u16::from_be_bytes(bytes[4..6].try_into().unwrap()) as usize;
    let mut head = None;
    for index in 0..tables {
        let record = 12 + index * 16;
        let start = u32::from_be_bytes(bytes[record + 8..record + 12].try_into().unwrap()) as usize;
        let length =
            u32::from_be_bytes(bytes[record + 12..record + 16].try_into().unwrap()) as usize;
        if &bytes[record..record + 4] == b"head" {
            head = Some(start);
            bytes[start + 8..start + 12].fill(0);
        }
        if matches!(&bytes[record..record + 4], b"head" | b"name") {
            let value = checksum(&bytes[start..start + length]);
            bytes[record + 4..record + 8].copy_from_slice(&value.to_be_bytes());
        }
    }
    let head = head.unwrap();
    let adjustment = 0xB1B0_AFBAu32.wrapping_sub(checksum(&bytes));
    bytes[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    bytes
}

fn record(function: u16, parameters: &[u8]) -> Vec<u8> {
    assert!(parameters.len().is_multiple_of(2));
    [
        ((3 + parameters.len() / 2) as u32).to_le_bytes().as_slice(),
        function.to_le_bytes().as_slice(),
        parameters,
    ]
    .concat()
}

fn document(family: &str) -> DocumentCore {
    let mut wmf = [
        1u16.to_le_bytes().as_slice(),
        9u16.to_le_bytes().as_slice(),
        0x300u16.to_le_bytes().as_slice(),
        0u32.to_le_bytes().as_slice(),
        1u16.to_le_bytes().as_slice(),
        0u32.to_le_bytes().as_slice(),
        0u16.to_le_bytes().as_slice(),
    ]
    .concat();
    wmf.extend(record(
        0x020c,
        &[100i16.to_le_bytes(), 100i16.to_le_bytes()].concat(),
    ));
    let mut logfont = [
        (-30i16).to_le_bytes(),
        0i16.to_le_bytes(),
        0i16.to_le_bytes(),
        0i16.to_le_bytes(),
        400i16.to_le_bytes(),
    ]
    .concat();
    logfont.extend([0u8; 8]);
    let mut name = [0u8; 32];
    name[..family.len()].copy_from_slice(family.as_bytes());
    logfont.extend(name);
    wmf.extend(record(0x02fb, &logfont));
    wmf.extend(record(0x012d, &0u16.to_le_bytes()));
    wmf.extend(record(0x012e, &24u16.to_le_bytes()));
    wmf.extend(record(
        0x0521,
        &[
            1u16.to_le_bytes().as_slice(),
            b"A\0",
            60i16.to_le_bytes().as_slice(),
            10i16.to_le_bytes().as_slice(),
        ]
        .concat(),
    ));
    wmf.extend(record(0, &[]));
    let words = wmf.len() as u32 / 2;
    wmf[6..10].copy_from_slice(&words.to_le_bytes());
    let mut core = DocumentCore::from_bytes(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/saved/blank2010.hwp"
    )))
    .unwrap();
    core.set_hangul2024_compat(false);
    let mut doc = core.document().clone();
    let mut picture = Picture {
        common: CommonObjAttr {
            treat_as_char: true,
            width: 7500,
            height: 7500,
            ..Default::default()
        },
        ..Default::default()
    };
    picture.image_attr.bin_data_id = 1;
    doc.sections[0].paragraphs = vec![Paragraph {
        char_count: 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            line_height: 7500,
            text_height: 7500,
            baseline_distance: 6000,
            ..Default::default()
        }],
        controls: vec![Control::Picture(Box::new(picture))],
        ..Default::default()
    }];
    doc.bin_data_content = vec![BinDataContent {
        id: 1,
        extension: "wmf".into(),
        data: wmf.into(),
    }];
    core.set_document(doc);
    core
}

#[test]
fn png_exporter_supplies_wmf_only_font_from_its_snapshot_and_preserves_source() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let serial = NEXT.fetch_add(1, Ordering::Relaxed);
    let family = format!(
        "RHWPWMF{:010}",
        u64::from(std::process::id()) * 1000 + serial
    );
    let font = isolated_program(&family);
    let dir = Directory(std::env::temp_dir().join(format!(
        "rhwp-wmf-snapshot-{}-{}",
        std::process::id(),
        serial
    )));
    std::fs::create_dir_all(&dir.0).unwrap();
    std::fs::write(dir.0.join("fixture.ttf"), &font).unwrap();
    let exporter = NativePngExporter::new(
        RenderProfile::Print,
        PngExportOptions {
            font_paths: vec![dir.0.clone()],
            ..Default::default()
        },
    );
    let mut automatic = document(&family);
    let mut explicit = document(&family);
    let source = rhwp::serializer::serialize_document(automatic.document()).unwrap();
    let requests: serde_json::Value =
        serde_json::from_str(&automatic.get_metafile_font_requests_native().unwrap()).unwrap();
    assert!(
        requests.as_array().unwrap().iter().any(|x| x["families"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name.as_str() == Some(family.as_str()))),
        "WMF에만 존재하는 가족을 실제로 요청한다"
    );
    explicit
        .register_metafile_font_native(&family, &font, 0)
        .unwrap();
    let expected = exporter.render_page(&explicit, 0).unwrap();
    let missing = exporter.render_page(&automatic, 0).unwrap();
    assert_ne!(
        missing, expected,
        "원 프로그램 부재 대조군은 독립 합성 glyph와 다르다"
    );
    assert!(exporter.prepare_document(&mut automatic).unwrap());
    let supplied = exporter.render_page(&automatic, 0).unwrap();
    assert_eq!(
        supplied, expected,
        "exporter의 공급이 명시적으로 등록한 같은 원 프로그램을 소비한다"
    );
    assert!(
        !exporter.prepare_document(&mut automatic).unwrap(),
        "같은 snapshot의 중복 등록은 paint를 다시 바꾸지 않는다"
    );
    assert_eq!(
        rhwp::serializer::serialize_document(automatic.document()).unwrap(),
        source,
        "paint 공급이 저장 IR과 원래 WMF 바이트를 바꾸지 않는다"
    );
}
