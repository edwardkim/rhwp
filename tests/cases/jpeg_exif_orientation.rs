//! 관계형. 브라우저 렌더러는 그림 JPEG의 EXIF 방향을 1로 바꿔 내보낸다.
//!
//! 브라우저는 SVG `<image>`·캔버스에 그린 JPEG를 EXIF 방향대로 돌리지만 한/글은 바이트 그대로
//! 그린다. `upright_jpeg`는 방향 값만 1로 바꾸고 나머지 바이트는 그대로 둔다.
//! `samples/issue1921/59043_regulatory_analysis.hwp` 8쪽 사진(1321×2137px)은 방향 6이다.

use rhwp::renderer::image_resolver::upright_jpeg;

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
        let source = jpeg_with_orientation(6, little);
        let upright = upright_jpeg(&source).expect("방향 6은 바꾼다");
        assert_eq!(upright, jpeg_with_orientation(1, little));
        assert_eq!(upright_jpeg(&upright), None);
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

/// 59043 문서 8쪽 SVG에 실린 JPEG는 모두 방향이 1이거나 방향이 없다.
#[test]
fn svg_writes_the_rotated_sample_photo_upright() {
    use base64::Engine as _;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1921/59043_regulatory_analysis.hwp");
    let bytes = std::fs::read(&path).expect("59043 문서");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("문서 열기");
    let svg = doc.render_page_svg_native(7).expect("8쪽 SVG");
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
    assert!(!jpegs.is_empty(), "8쪽 SVG에 JPEG가 있어야 한다");
    assert!(
        jpegs.iter().all(|jpeg| upright_jpeg(jpeg).is_none()),
        "방향이 1이 아닌 JPEG를 그대로 실었다"
    );
}
