//! 흑백 예약색과 실제 Print 계조를 공통 그림 payload·조회·프로필 경로에서 보호한다.
//! 독립 2024 Print와 전쪽 Native/fresh WASM·직접 판독: stage2101, stage2122.
//! 합성 진단 입력이며 실물 시험지나 WMF의 완료 증거를 대신하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use base64::Engine;
use rhwp::paint::{LayerJsonOptions, LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::DocumentCore;
use serde_json::Value;
use std::collections::HashSet;

const DIRECTORY: &str = "pdf/issue7642-metafile-palette-visibility-20261017";

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
            out.extend(ops.iter().filter(|op| matches!(op, PaintOp::Image { .. })));
        }
    }
}

fn json_images<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(fields) => {
            if value["type"] == "image" {
                out.push(value);
            } else {
                for child in fields.values() {
                    json_images(child, out);
                }
            }
        }
        Value::Array(items) => {
            for child in items {
                json_images(child, out);
            }
        }
        _ => {}
    }
}

fn verify_profile(core: &DocumentCore, profile: RenderProfile, expanded: &[bool]) -> Vec<String> {
    let tree = core.build_page_layer_tree_with_profile(0, profile).unwrap();
    let mut ops = Vec::new();
    image_ops(&tree.root, &mut ops);
    assert_eq!(ops.len(), expanded.len(), "그림의 누락·중복 방지");
    let inline: Value = serde_json::from_str(&tree.to_json()).unwrap();
    let omitted: Value = serde_json::from_str(&tree.to_json_with_options(LayerJsonOptions {
        omit_image_bytes: true,
        ..LayerJsonOptions::default()
    }))
    .unwrap();
    let mut inline_ops = Vec::new();
    let mut omitted_ops = Vec::new();
    json_images(&inline, &mut inline_ops);
    json_images(&omitted, &mut omitted_ops);
    assert_eq!(inline_ops.len(), ops.len());
    assert_eq!(omitted_ops.len(), ops.len());

    let mut owners = HashSet::new();
    let mut keys = Vec::new();
    for (op, &changes) in ops.iter().zip(expanded) {
        let PaintOp::Image {
            image, resolved, ..
        } = op
        else {
            unreachable!()
        };
        let owner = (image.section_index, image.para_index, image.control_index);
        assert!(owners.insert(owner), "한 원본 그림의 중복 출력: {owner:?}");
        assert_eq!(image.section_index, Some(0));
        assert_eq!(image.para_index, Some(owners.len() - 1), "원본 문단 순서");
        let source_bytes = image.data.as_deref().expect("원본 그림 바이트 보존");
        let source = image::load_from_memory(source_bytes).unwrap().to_rgba8();
        let emitted_bytes = resolved.as_deref().map_or(source_bytes, |p| &p.data);
        let emitted = image::load_from_memory(emitted_bytes).unwrap().to_rgba8();
        assert_eq!(
            emitted.dimensions(),
            source.dimensions(),
            "원본 해상도 보존"
        );
        assert!(
            source
                .pixels()
                .zip(emitted.pixels())
                .all(|(a, b)| a[3] == b[3]),
            "알파 보존"
        );

        if changes {
            // 독립 Print의 JPEG 내부 표본은 128→131, 240→246이다.
            // JPEG 양자화 한 단계만 허용하며 위치를 골라 기대값을 만들지 않는다.
            for (source_gray, printed_gray) in [(128u8, 131u8), (240, 246)] {
                let mut count = 0;
                for (a, b) in source.pixels().zip(emitted.pixels()) {
                    if a.0[..3] == [source_gray; 3] {
                        count += 1;
                        assert!(b.0[..3].iter().all(|c| c.abs_diff(printed_gray) <= 1),
                            "독립 Print 계조 불일치: owner={owner:?}, 원본={source_gray}, 출력={b:?}");
                    }
                }
                assert!(count > 0, "Print의 대조 계조가 원본에 실제 존재해야 한다");
            }
        } else {
            assert_eq!(
                emitted, source,
                "팔레트 전달 또는 비대상 프로필의 원본색 보존"
            );
        }

        let index = keys.len();
        let key = inline_ops[index]["sourceImageKey"].as_str().unwrap();
        let lookup = core
            .get_source_image_bytes_native(key)
            .expect("바이트 생략 후 재조회");
        assert_eq!(lookup.0, inline_ops[index]["mime"].as_str().unwrap());
        let inline_bytes = base64::engine::general_purpose::STANDARD
            .decode(inline_ops[index]["base64"].as_str().unwrap())
            .unwrap();
        assert_eq!(lookup.1, inline_bytes, "직접 payload와 키 조회의 동일 출력");
        assert_eq!(
            lookup.1, emitted_bytes,
            "공통 paint payload와 JSON의 동일 출력"
        );
        assert_eq!(omitted_ops[index]["sourceImageKey"], key);
        assert_eq!(omitted_ops[index]["imageBytesOmitted"], true);
        assert!(omitted_ops[index].get("base64").is_none());
        keys.push(key.to_owned());
    }
    keys
}

fn verify(input: &str, expanded: &[bool]) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(DIRECTORY)
        .join(input);
    let bytes = std::fs::read(path).unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 1, "독립 Print와 전체 쪽수 일치");
    let unchanged = vec![false; expanded.len()];
    let screen = verify_profile(&core, RenderProfile::Screen, &unchanged);
    let print = verify_profile(&core, RenderProfile::Print, expanded);
    assert_eq!(
        screen,
        verify_profile(&core, RenderProfile::Screen, &unchanged)
    );
    assert_eq!(print, verify_profile(&core, RenderProfile::Print, expanded));
    for ((a, b), changes) in screen.iter().zip(&print).zip(expanded) {
        if *changes {
            assert_ne!(a, b, "원본색·Print 전달색의 캐시 신원 분리");
        }
    }
    core.set_hangul2024_compat(false);
    verify_profile(&core, RenderProfile::Print, &unchanged);
    core.set_hangul2024_compat(true);
    verify_profile(&core, RenderProfile::Print, expanded);
}

#[test]
fn verified_hancom2024_reserved_black_white_palette_keeps_print_and_source_paths_consistent() {
    // 흑백 포함 256 / 흰색 없음 256 / 검은색 없음 256 / 흑백 포함 유색 256 /
    // 흑백 없음 255 / 검은색만 없음 255의 독립 Print 순서이다.
    verify(
        "bitmap-white-palette-controls.hwpx",
        &[false, true, true, false, true, false],
    );
}

#[test]
fn verified_hancom2024_indexed_and_rgb_palettes_share_the_independent_print_contract() {
    // 각 팔레트에 RGB PNG·indexed PNG·indexed BMP가 같은 순서로 있다.
    verify(
        "bitmap-indexed-color-controls.hwpx",
        &[
            true, true, true, true, true, true, false, false, false, true, true, true,
        ],
    );
}
