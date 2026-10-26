//! #8160: 상위 장치 클립이 있는 비트맵도 공통 WMF 인쇄 격자를 유지한다.
//! MS-WMF의 상위 클립은 비트맵의 배치/원본 픽셀을 바꾸지 않고 표시 영역만 제한한다.
//! 실물 HWP 2022 18쪽의 독립 Print raw100 검증과 함께 사용하는 합성 경계 검사다.
#![cfg(not(target_arch = "wasm32"))]

use base64::Engine;
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

fn record(function: u16, parameters: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend((3 + parameters.len() as u32 / 2).to_le_bytes());
    bytes.extend(function.to_le_bytes());
    bytes.extend(parameters);
    bytes
}

fn bitmap_wmf(clipped: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(9u16.to_le_bytes());
    bytes.extend(0x0300u16.to_le_bytes());
    bytes.extend(0u32.to_le_bytes());
    bytes.extend(4u16.to_le_bytes());
    bytes.extend(0u32.to_le_bytes());
    bytes.extend(0u16.to_le_bytes());
    bytes.extend(record(
        0x020c,
        &[64i16.to_le_bytes(), 64i16.to_le_bytes()].concat(),
    ));
    if clipped {
        // META_INTERSECTCLIPRECT: bottom, right, top, left.
        bytes.extend(record(
            0x0416,
            &[
                4i16.to_le_bytes(),
                4i16.to_le_bytes(),
                0i16.to_le_bytes(),
                0i16.to_le_bytes(),
            ]
            .concat(),
        ));
    }
    let mut dib = Vec::new();
    dib.extend(40u32.to_le_bytes());
    dib.extend(8i32.to_le_bytes());
    dib.extend(8i32.to_le_bytes());
    dib.extend(1u16.to_le_bytes());
    dib.extend(24u16.to_le_bytes());
    dib.extend([0u8; 24]);
    // 64가지 불투명 색: 이진 마스크가 아닌 사진 전달 경로를 실제로 소비한다.
    for color in 0..64u8 {
        dib.extend([color * 3, color * 2, color]);
    }
    let mut params = Vec::new();
    params.extend(0x00cc0020u32.to_le_bytes()); // SRCCOPY
    for value in [0i16, 0, 8, 8, 0, 0] {
        params.extend(value.to_le_bytes());
    }
    params.extend(dib);
    // 공유 image/use 경로도 클립의 소유를 유지해야 한다.
    bytes.extend(record(0x0940, &params));
    bytes.extend(record(0x0940, &params));
    bytes.extend(record(0, &[]));
    let words = bytes.len() as u32 / 2;
    bytes[6..10].copy_from_slice(&words.to_le_bytes());
    bytes
}

fn print_svg(clipped: bool) -> String {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
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
        data: bitmap_wmf(clipped).into(),
    }];
    core.set_document(doc);
    let svg = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .unwrap();
    let document = roxmltree::Document::parse(&svg).unwrap();
    let payloads: Vec<_> = document
        .descendants()
        .filter_map(|node| {
            let href = node
                .attribute("href")
                .or_else(|| node.attribute(("http://www.w3.org/1999/xlink", "href")))?;
            let encoded = href.strip_prefix("data:image/svg+xml;base64,")?;
            Some(
                String::from_utf8(
                    base64::engine::general_purpose::STANDARD
                        .decode(encoded)
                        .unwrap(),
                )
                .unwrap(),
            )
        })
        .collect();
    assert_eq!(payloads.len(), 1, "그림 payload의 누락·중복");
    payloads.into_iter().next().unwrap()
}

#[test]
fn issue_8160_parent_clip_preserves_print_raster_sampling_and_shared_image_ownership() {
    for clipped in [false, true] {
        let svg = print_svg(clipped);
        let doc = roxmltree::Document::parse(&svg).unwrap();
        assert!(
            doc.root_element()
                .has_attribute("data-rhwp-print-wmf-frame"),
            "상위 클립 때문에 인쇄 재생 자료 전체를 일반 SVG로 되돌리지 않는다"
        );
        let images: Vec<_> = doc
            .descendants()
            .filter(|node| node.has_attribute("data-rhwp-print-wmf-image"))
            .collect();
        assert_eq!(images.len(), 2, "두 번의 실제 원본 blit을 보존한다");
        for image in images {
            let clip = image
                .ancestors()
                .find_map(|node| node.attribute("clip-path"));
            assert_eq!(
                clip.is_some(),
                clipped,
                "클립은 재생할 그림의 상위 그룹이 소유한다"
            );
            if let Some(reference) = clip {
                let id = reference
                    .strip_prefix("url(#")
                    .unwrap()
                    .strip_suffix(')')
                    .unwrap();
                let definition = doc
                    .descendants()
                    .find(|node| node.attribute("id") == Some(id))
                    .unwrap();
                let rect = definition
                    .children()
                    .find(|node| node.has_tag_name("rect"))
                    .unwrap();
                for (name, expected) in [("x", "0"), ("y", "0"), ("width", "4"), ("height", "4")] {
                    assert_eq!(
                        rect.attribute(name),
                        Some(expected),
                        "원본 장치 클립 {name}"
                    );
                }
            }
        }
    }
}
