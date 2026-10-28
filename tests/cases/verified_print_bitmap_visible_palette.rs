//! 독립 한컴 2024 Print의 보이는 팔레트와 소유·원본 순서 및 재조회를 보호한다.
//! 동일 입력의 Native·fresh WASM 1쪽 raw 실루엣은 각각 100%다.
//! 그림 자원의 크기와 회색값은 독립 PDF의 이미지 자원에서 확인했다.
use base64::Engine;
use rhwp::paint::{
    LayerJsonOptions, LayerNode, LayerNodeKind, PaintOp, RenderProfile, ResolvedImageKind,
};
use rhwp::DocumentCore;
use serde_json::Value;

fn images<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(x) => {
            if x.get("type").and_then(Value::as_str) == Some("image") {
                out.push(v);
            } else {
                for v in x.values() {
                    images(v, out);
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                images(v, out);
            }
        }
        _ => {}
    }
}
fn image_ops<'a>(node: &'a LayerNode, out: &mut Vec<&'a PaintOp>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                image_ops(child, out);
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
fn sample() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7642-bitmap-crop-palette-print-20261019/bitmap-crop-palette-controls.hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 1);
    core
}

#[test]
fn print_picture_mask_and_area_sampling_follow_independent_output() {
    let core = sample();
    let tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let json: Value = serde_json::from_str(&tree.to_json()).unwrap();
    let mut ops = Vec::new();
    images(&json, &mut ops);
    assert_eq!(ops.len(), 5, "원문 그림의 누락·중복 방지");
    let modes: Vec<_> = ops
        .iter()
        .map(|op| op["imageReconstruction"].as_str())
        .collect();
    // 한컴 2024 Print는 세 번째 자원에만 별도 이진 마스크를 보유한다.
    assert_eq!(
        modes,
        [
            Some("printArea"),
            Some("printArea"),
            Some("printMaskedArea"),
            Some("printArea"),
            Some("printArea")
        ]
    );
}

#[test]
fn print_palette_payload_preserves_independent_samples_owners_and_source_order() {
    let core = sample();
    // 독립 Print PDF의 다섯 image stream 크기와 각 1/8, 3/8, 5/8, 7/8 내부 회색값.
    let expected_sizes = [(512, 512), (511, 512), (512, 511), (256, 256), (512, 511)];
    let print_gray = [
        [41u8, 108, 173, 238],
        [40, 102, 166, 230],
        [26, 99, 169, 234],
        [88, 120, 152, 184],
        [23, 88, 153, 216],
    ];
    for profile in [
        RenderProfile::Screen,
        RenderProfile::Print,
        RenderProfile::Screen,
        RenderProfile::Print,
    ] {
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
        assert_eq!((a.len(), b.len()), (5, 5));
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
                assert_eq!(&source, first, "crop만 다른 동일 원본 픽셀");
            } else {
                first_source = Some(source.clone());
            }
            let payload = resolved
                .as_deref()
                .map_or(image.data.as_deref().unwrap(), |p| p.data.as_slice());
            let actual = image::load_from_memory(payload).unwrap().to_rgba8();
            if profile == RenderProfile::Screen {
                assert_eq!(actual, source, "Screen 원본 RGBA 보존");
            } else {
                assert_eq!(
                    actual.dimensions(),
                    expected_sizes[i],
                    "독립 Print 자원의 크기"
                );
                let (w, h) = actual.dimensions();
                for (j, k) in [1u32, 3, 5, 7].into_iter().enumerate() {
                    let pixel = actual.get_pixel(w / 2, h * k / 8);
                    assert!(
                        pixel.0[..3]
                            .iter()
                            .all(|v| v.abs_diff(print_gray[i][j]) <= 1),
                        "독립 Print 계조 {i} {j}: {pixel:?}"
                    );
                    assert_eq!(pixel[3], 255, "내부 원본은 불투명");
                }
                let transparent = actual.pixels().filter(|p| p[3] == 0).count();
                assert_eq!(
                    transparent,
                    if i == 2 { 127 } else { 0 },
                    "독립 Print 이진 마스크"
                );
            }
            assert_eq!(b[i]["imageReconstruction"], a[i]["imageReconstruction"]);
            let encoded = base64::engine::general_purpose::STANDARD
                .decode(a[i]["base64"].as_str().unwrap())
                .unwrap();
            assert_eq!(encoded, payload, "공통 paint와 inline의 동일 바이트");
            if let Some(key) = a[i]["sourceImageKey"].as_str() {
                assert_eq!(b[i]["sourceImageKey"], key);
                assert_eq!(b[i]["imageBytesOmitted"], true);
                assert!(b[i].get("base64").is_none());
                let looked = core.get_source_image_bytes_native(key).unwrap();
                assert_eq!(looked.0, a[i]["mime"].as_str().unwrap());
                assert_eq!(looked.1, encoded, "inline과 source key 재조회 동일 바이트");
            } else {
                assert!(
                    resolved.as_ref().is_some_and(|p| matches!(
                        p.kind,
                        ResolvedImageKind::OutputDevice
                            | ResolvedImageKind::OutputDeviceWindow
                            | ResolvedImageKind::SourceWindow
                    )),
                    "배치 종속 자원은 실제 바이트를 보유"
                );
                assert_eq!(b[i]["base64"], a[i]["base64"]);
            }
        }
    }
}
