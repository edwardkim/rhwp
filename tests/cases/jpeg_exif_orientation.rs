//! 관계형. 브라우저 렌더러는 그림 JPEG의 EXIF 방향을 1로 바꿔 내보낸다.
//! 주석 갱신 2026-11-25 00:02 KST. 직전 변경은 2026-11-22 11:06:31 +0900, 커밋 `a5884818331908d5d6085aac3f9ca3430f27a098`이다.
//!
//! 브라우저는 SVG `<image>`·캔버스에 그린 JPEG를 EXIF 방향대로 돌리지만 한/글은 바이트 그대로
//! 그린다. `upright_jpeg`는 방향 값만 1로 바꾸고 나머지 바이트는 그대로 둔다.
//! 규제영향분석서 8쪽의 JPEG(1321×2137px, 방향 6)만 담은 한 쪽 문서를 사용한다.
//! 2026-11-24 한컴 2024 Print와 96 DPI·2px raw로 대조한 devel의 전 1쪽 최저는
//! Native 99.98%, fresh WASM 99.98%다. 원사진과 채움 바이트 대조군의 독립 인쇄 픽셀은 같다.
//! 37쪽 원본과 보완 후보의 페이지별 수치·화면 비교는 `mydocs/pr/assets/pr9298/README.md`에 있다.

use rhwp::renderer::image_resolver::upright_jpeg;

fn original_photo_document() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/semanticist21-20261123/jpeg-exif-original-photo.hwpx");
    std::fs::read(path).expect("규제영향분석서 8쪽과 같은 JPEG가 든 한 쪽 문서")
}

/// SOI + APP1(Exif, IFD0에 방향 하나) + SOS 머리만 있는 JPEG 조각.
fn jpeg_with_orientation(value: u16, little: bool) -> Vec<u8> {
    let u16b = |v: u16| {
        if little {
            v.to_le_bytes()
        } else {
            v.to_be_bytes()
        }
    };
    let u32b = |v: u32| {
        if little {
            v.to_le_bytes()
        } else {
            v.to_be_bytes()
        }
    };
    let mut tiff = Vec::new();
    tiff.extend_from_slice(if little { b"II" } else { b"MM" });
    tiff.extend_from_slice(&u16b(42));
    tiff.extend_from_slice(&u32b(8));
    tiff.extend_from_slice(&u16b(1));
    tiff.extend_from_slice(&u16b(0x0112));
    tiff.extend_from_slice(&u16b(3));
    tiff.extend_from_slice(&u32b(1));
    tiff.extend_from_slice(&u16b(value));
    tiff.extend_from_slice(&[0, 0]);
    tiff.extend_from_slice(&u32b(0));
    let mut out = vec![0xFF, 0xD8, 0xFF, 0xE1];
    out.extend_from_slice(&((tiff.len() + 8) as u16).to_be_bytes());
    out.extend_from_slice(b"Exif\0\0");
    out.extend_from_slice(&tiff);
    out.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x02]);
    out
}

#[test]
fn 방향이_있는_jpeg는_방향_값만_1로_바꾼다() {
    for little in [true, false] {
        for value in 2..=8 {
            let source = jpeg_with_orientation(value, little);
            let upright = upright_jpeg(&source).expect("유효한 회전과 반사를 모두 해제한다");
            assert_eq!(upright, jpeg_with_orientation(1, little));
            assert_eq!(upright_jpeg(&upright), None);
        }
    }
}

#[test]
fn jpeg_marker_fill_and_other_app1_preserve_the_orientation_contract() {
    fn with_markers(value: u16, little: bool) -> Vec<u8> {
        let bytes = jpeg_with_orientation(value, little);
        let mut result = bytes[..2].to_vec();
        // 두 APP1 사이에 채움 바이트를 둔다. 앞 APP1은 EXIF가 아니다.
        result.extend_from_slice(b"\xff\xe1\x00\x06XMP!\xff");
        result.extend_from_slice(&bytes[2..]);
        result
    }
    for little in [true, false] {
        let source = with_markers(6, little);
        assert_eq!(upright_jpeg(&source).unwrap(), with_markers(1, little));
        assert!(upright_jpeg(&with_markers(1, little)).is_none());
    }
}

#[test]
fn 방향이_없거나_jpeg가_아니면_그대로_둔다() {
    assert_eq!(upright_jpeg(&jpeg_with_orientation(1, true)), None);
    assert_eq!(upright_jpeg(&[0xFF, 0xD8, 0xFF, 0xDA, 0x00, 0x02]), None);
    assert_eq!(upright_jpeg(b"\x89PNG\r\n\x1a\n"), None);
    // 길이가 잘린 EXIF도 넘어서 읽지 않는다.
    let mut short = jpeg_with_orientation(6, true);
    short.truncate(20);
    assert_eq!(upright_jpeg(&short), None);
}

/// 같은 사진을 담은 SVG의 JPEG에는 브라우저가 적용할 회전이 남지 않는다.
#[test]
fn svg_writes_the_rotated_sample_photo_upright() {
    use base64::Engine as _;
    let doc =
        rhwp::wasm_api::HwpDocument::from_bytes(&original_photo_document()).expect("문서 열기");
    let svg = doc.render_page_svg_native(0).expect("그림 대조군 SVG");
    let prefix = "data:image/jpeg;base64,";
    let jpegs: Vec<Vec<u8>> = svg
        .match_indices(prefix)
        .map(|(at, _)| {
            let start = at + prefix.len();
            let end = start + svg[start..].find('"').expect("data URI 끝");
            base64::engine::general_purpose::STANDARD
                .decode(&svg[start..end])
                .expect("base64")
        })
        .collect();
    assert_eq!(jpegs.len(), 1, "원래 JPEG 한 장을 SVG에 싣는다");
    assert!(
        jpegs.iter().all(|jpeg| upright_jpeg(jpeg).is_none()),
        "방향이 1이 아닌 JPEG를 그대로 실었다"
    );
}

#[test]
fn browser_image_key_is_upright_and_saved_jpeg_retains_its_orientation() {
    use image::codecs::jpeg::JpegDecoder;
    use image::metadata::Orientation;
    use image::ImageDecoder;
    use rhwp::wasm_api::HwpDocument;
    use std::io::Cursor;
    use std::sync::Arc;

    fn stored_jpeg(bytes: &[u8]) -> Arc<[u8]> {
        rhwp::parser::parse_document(bytes)
            .unwrap()
            .bin_data_content
            .iter()
            .map(|entry| entry.data.load_shared())
            .find(|data| data.starts_with(&[0xff, 0xd8]))
            .expect("fixture contains the original JPEG")
    }

    let input = original_photo_document();
    let original = stored_jpeg(&input);
    let orientation = |bytes: &[u8]| {
        JpegDecoder::new(Cursor::new(bytes))
            .unwrap()
            .orientation()
            .unwrap()
    };
    assert_eq!(orientation(&original), Orientation::Rotate90);
    let doc = HwpDocument::from_bytes(&input).unwrap();
    let keys: serde_json::Value =
        serde_json::from_str(&doc.get_page_source_image_keys_native(0).unwrap()).unwrap();
    let keys = keys["keys"].as_array().unwrap();
    assert_eq!(keys.len(), 1);
    let (mime, browser) = doc
        .get_source_image_bytes_native(keys[0].as_str().unwrap())
        .unwrap();
    assert_eq!(mime, "image/jpeg");
    assert_eq!(orientation(&browser), Orientation::NoTransforms);
    assert_eq!(browser.len(), original.len());
    assert_eq!(
        browser
            .iter()
            .zip(original.iter())
            .filter(|(a, b)| a != b)
            .count(),
        1
    );

    for saved in [
        doc.export_hwp_with_adapter_snapshot().unwrap(),
        doc.export_hwpx_native().unwrap(),
    ] {
        assert_eq!(stored_jpeg(&saved).as_ref(), original.as_ref());
    }
}
