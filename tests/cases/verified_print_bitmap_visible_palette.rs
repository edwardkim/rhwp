//! 독립 한컴 Print의 보이는 팔레트와 inline/재조회 결과를 보호한다.
//! 같은 BMP의 crop별 독립 기대값은 전체/위쪽만 색 확장, 나머지는 팔레트 전달이다.
//! 전쪽 시각·직접 판독 증거는 bitmap_visible_palette_20261019/README.md를 따른다.
use rhwp::paint::RenderProfile;
use rhwp::DocumentCore;
use serde_json::Value;
fn images<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(x) => {
            if x.get("type").and_then(Value::as_str) == Some("image") {
                out.push(v)
            } else {
                for v in x.values() {
                    images(v, out)
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                images(v, out)
            }
        }
        _ => {}
    }
}
#[test]
fn print_palette_selection_follows_visible_crop_instead_of_full_source_colors() {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7642-bitmap-crop-palette-print-20261019/bitmap-crop-palette-controls.hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(p).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 1);
    let tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let json: Value = serde_json::from_str(&tree.to_json()).unwrap();
    let mut ops = Vec::new();
    images(&json, &mut ops);
    assert_eq!(ops.len(), 5, "원문 그림의 누락·중복 방지");
    let observed: Vec<_> = ops
        .iter()
        .map(|op| op["imageReconstruction"].as_str() == Some("catmullRom"))
        .collect();
    // 독립 한컴 2024 Print: 전체/위1HU crop은 257색 이상, 오른쪽1HU/가운데/아래1HU는 팔레트 전달이다.
    assert_eq!(
        observed,
        [true, false, true, false, false],
        "같은 원본도 실제 보이는 창의 팔레트가 다르다"
    );
}

use base64::Engine;
use rhwp::paint::{LayerJsonOptions, LayerNode, LayerNodeKind, PaintOp};
fn image_ops<'a>(node: &'a LayerNode, out: &mut Vec<&'a PaintOp>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                image_ops(child, out)
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            image_ops(child, out)
        }
        LayerNodeKind::Leaf { ops } => {
            out.extend(ops.iter().filter(|op| matches!(op, PaintOp::Image { .. })))
        }
    }
}
#[test]
fn print_palette_payload_and_lazy_lookup_preserve_visible_colors_owners_and_source_order() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7642-bitmap-crop-palette-print-20261019/bitmap-crop-palette-controls.hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    for profile in [
        RenderProfile::Screen,
        RenderProfile::Print,
        RenderProfile::Screen,
        RenderProfile::Print,
    ] {
        let expected = if profile == RenderProfile::Print {
            [true, false, true, false, false]
        } else {
            [false; 5]
        };
        let tree = core.build_page_layer_tree_with_profile(0, profile).unwrap();
        let mut ops = Vec::new();
        image_ops(&tree.root, &mut ops);
        assert_eq!(ops.len(), 5);
        let inline: Value = serde_json::from_str(&tree.to_json()).unwrap();
        let omitted: Value = serde_json::from_str(&tree.to_json_with_options(LayerJsonOptions {
            omit_image_bytes: true,
            ..Default::default()
        }))
        .unwrap();
        let mut a = Vec::new();
        let mut b = Vec::new();
        images(&inline, &mut a);
        images(&omitted, &mut b);
        assert_eq!(a.len(), 5);
        assert_eq!(b.len(), 5);
        let mut owners = std::collections::HashSet::new();
        let mut previous = None;
        let mut first_source = None;
        for (i, op) in ops.iter().enumerate() {
            let PaintOp::Image {
                image, resolved, ..
            } = op
            else {
                unreachable!()
            };
            let owner = (image.section_index, image.para_index, image.control_index);
            assert!(owners.insert(owner), "원문 그림의 중복 출력");
            assert_eq!(image.section_index, Some(0));
            let ordinal = (
                image.para_index.expect("원문 문단 소속"),
                image.control_index.expect("원문 control 소속"),
            );
            if let Some(before) = previous {
                assert!(before < ordinal, "원문 그림 순서");
            }
            previous = Some(ordinal);
            let source = image::load_from_memory(image.data.as_deref().unwrap())
                .unwrap()
                .to_rgba8();
            if let Some(first) = &first_source {
                assert_eq!(
                    &source, first,
                    "crop만 다른 동일 원본 픽셀의 대조군이어야 한다"
                );
            } else {
                first_source = Some(source.clone());
            }
            let payload = resolved
                .as_deref()
                .map_or(image.data.as_deref().unwrap(), |p| p.data.as_slice());
            let actual = image::load_from_memory(payload).unwrap().to_rgba8();
            assert_eq!(source.dimensions(), actual.dimensions(), "원본 해상도 보존");
            assert!(
                source
                    .pixels()
                    .zip(actual.pixels())
                    .all(|(x, y)| x[3] == y[3]),
                "알파 보존"
            );
            if expected[i] {
                // 독립 Print의 평탄 영역 평균을 반올림한 계조다. JPEG의 한 단계만 허용한다.
                for (gray, printed) in [(128u8, 133u8), (217, 223), (228, 236), (240, 248)] {
                    let mut count = 0;
                    for (source, out) in source.pixels().zip(actual.pixels()) {
                        if source.0[..3] == [gray; 3] {
                            count += 1;
                            assert!(
                                out.0[..3].iter().all(|v| v.abs_diff(printed) <= 1),
                                "독립 Print 계조와 다름: {i} {gray} {out:?}"
                            );
                        }
                    }
                    assert!(count > 0, "독립 대조 계조가 실제 원본에 존재해야 한다");
                }
            } else {
                assert_eq!(actual, source, "팔레트 전달·Screen의 원본 RGBA 보존");
            }
            assert_eq!(
                a[i]["imageReconstruction"].as_str() == Some("catmullRom"),
                expected[i]
            );
            let key = a[i]["sourceImageKey"].as_str().unwrap();
            assert_eq!(b[i]["sourceImageKey"], key);
            assert_eq!(b[i]["imageReconstruction"], a[i]["imageReconstruction"]);
            assert_eq!(b[i]["imageBytesOmitted"], true);
            assert!(b[i].get("base64").is_none());
            let looked = core.get_source_image_bytes_native(key).unwrap();
            let encoded = base64::engine::general_purpose::STANDARD
                .decode(a[i]["base64"].as_str().unwrap())
                .unwrap();
            assert_eq!(looked.0, a[i]["mime"].as_str().unwrap());
            assert_eq!(looked.1, encoded, "inline과 source key 재조회 동일 바이트");
            assert_eq!(looked.1, payload, "공통 paint와 재조회 동일 바이트");
        }
    }
}
