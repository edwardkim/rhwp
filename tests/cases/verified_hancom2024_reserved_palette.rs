//! 흑백 예약색과 실제 Print 계조를 공통 그림 payload·조회·프로필 경로에서 보호한다.
//! 독립 2024 Print와 전쪽 Native/fresh WASM·직접 판독: stage2101, stage2122.
//! 합성 진단 입력이며 실물 시험지나 WMF의 완료 증거를 대신하지 않는다.
//! 새 독립 PDF를 직접 읽으므로 pdf-import 기능에서 실행한다.
#![cfg(all(not(target_arch = "wasm32"), feature = "pdf-import"))]

use base64::Engine;
use rhwp::paint::{
    LayerJsonOptions, LayerNode, LayerNodeKind, PaintOp, RenderProfile, ResolvedImageKind,
};
use rhwp::DocumentCore;
use serde_json::Value;
use sha2::{Digest, Sha256};
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

fn verify_profile(
    core: &DocumentCore,
    profile: RenderProfile,
    expanded: &[bool],
    reference: Option<&[image::RgbaImage]>,
) -> Vec<String> {
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
    let source_keys: Value =
        serde_json::from_str(&core.get_page_source_image_keys_native(0).unwrap()).unwrap();
    assert_eq!(source_keys["keys"].as_array().unwrap().len(), ops.len());

    let mut owners = HashSet::new();
    let mut digests = Vec::new();
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
        let index = digests.len();
        let original_key = source_keys["keys"][index].as_str().unwrap();
        let original_lookup = core.get_source_image_bytes_native(original_key).unwrap();
        assert_eq!(
            image::load_from_memory(&original_lookup.1)
                .unwrap()
                .to_rgba8(),
            source,
            "Print 전달이 원본 bin 조회의 색을 덮어쓰지 않는다"
        );
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

        let index = digests.len();
        if let Some(reference) = reference {
            let expected = &reference[index];
            assert_eq!(emitted.dimensions(), expected.dimensions());
            let maximum_error = emitted
                .as_raw()
                .iter()
                .zip(expected.as_raw())
                .map(|(actual, expected)| actual.abs_diff(*expected))
                .max()
                .unwrap();
            assert!(maximum_error <= 1,
                "새 독립 Print JPEG의 전체 계조·색상: owner={owner:?}, maximum_error={maximum_error}");
            assert_eq!(
                inline_ops[index]["imageReconstruction"].as_str() == Some("catmullRom"),
                changes
            );
        } else {
            assert_eq!(emitted.as_raw(), source.as_raw(), "Screen 원본 RGBA 보존");
        }

        let inline_bytes = base64::engine::general_purpose::STANDARD
            .decode(inline_ops[index]["base64"].as_str().unwrap())
            .unwrap();
        assert_eq!(
            inline_bytes, emitted_bytes,
            "공통 paint payload와 JSON의 동일 출력"
        );
        if let Some(key) = inline_ops[index]["sourceImageKey"].as_str() {
            let lookup = core
                .get_source_image_bytes_native(key)
                .expect("바이트 생략 후 재조회");
            assert_eq!(lookup.0, inline_ops[index]["mime"].as_str().unwrap());
            assert_eq!(lookup.1, inline_bytes, "직접 payload와 키 조회의 동일 출력");
            assert_eq!(omitted_ops[index]["sourceImageKey"], key);
            assert_eq!(omitted_ops[index]["imageBytesOmitted"], true);
            assert!(omitted_ops[index].get("base64").is_none());
        } else {
            assert!(
                resolved.as_ref().is_some_and(|p| matches!(
                    p.kind,
                    ResolvedImageKind::OutputDevice | ResolvedImageKind::SourceWindow
                )),
                "배치별 payload만 원본 bin 키를 생략할 수 있다"
            );
            assert!(omitted_ops[index].get("sourceImageKey").is_none());
            assert_ne!(omitted_ops[index]["imageBytesOmitted"], true);
            let retained = base64::engine::general_purpose::STANDARD
                .decode(
                    omitted_ops[index]["base64"]
                        .as_str()
                        .expect("재조회 불가 payload의 inline 보존"),
                )
                .unwrap();
            assert_eq!(retained, emitted_bytes);
            assert_eq!(omitted_ops[index]["mime"], inline_ops[index]["mime"]);
            let digest: String = sha2::Sha256::digest(emitted_bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
        }
        digests.push(sha256(&inline_bytes));
    }
    digests
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn independent_print(input: &str, engine: &str, count: usize) -> Vec<image::RgbaImage> {
    let stem = input.strip_suffix(".hwpx").unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("pdf/issue8160/{stem}-{engine}.pdf"));
    let expected_sha = match (stem, engine) {
        ("bitmap-white-palette-controls", "2020") => {
            "55b213c8164c59f4bdda482a4c7bd0756e5b1490db5df8d6f9f47a13af40a99c"
        }
        ("bitmap-white-palette-controls", "2024") => {
            "8cf7dbe4ce8b1cd52c72ddbf9f9177e8c1a70a4fa49bae5e9971510ac84a8750"
        }
        ("bitmap-indexed-color-controls", "2020") => {
            "40e2a63219f62a1bef22c229205e169261bdaeb2b070f2e29b4128ee51325859"
        }
        ("bitmap-indexed-color-controls", "2024") => {
            "7a09f328926e0dccbc5b2a61f0cb8a763cbdaaf0455cc86aca3f789af0ccef63"
        }
        _ => panic!("독립 Print 계약에 없는 원본·engine"),
    };
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(sha256(&bytes), expected_sha, "새 MCP Print 원본 해시");
    let pdf = pdf_oxide::PdfDocument::from_bytes(bytes).unwrap();
    assert_eq!(pdf.page_count().unwrap(), 1);
    let handles = pdf.page_image_handles(0).unwrap();
    assert_eq!(
        handles.len(),
        count,
        "독립 Print의 전체 그림 순서·누락·중복"
    );
    handles
        .iter()
        .enumerate()
        .map(|(index, handle)| {
            assert_eq!(handle.paint_order, index);
            image::load_from_memory(&handle.raw_compressed_bytes().unwrap())
                .unwrap()
                .to_rgba8()
        })
        .collect()
}

fn verify(input: &str, expanded: &[bool]) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(DIRECTORY)
        .join(input);
    let bytes = std::fs::read(path).unwrap();
    let expected_source_sha = match input {
        "bitmap-white-palette-controls.hwpx" => {
            "0460afdbe6ed8f9a4272cac1828645760038421a6aa03338c96d6e730a8cab55"
        }
        "bitmap-indexed-color-controls.hwpx" => {
            "b6e326a5a7b486a85cb49c599611ec80f39e516b3ac2c6a93efaf6241542e04b"
        }
        _ => panic!("독립 Print 계약에 없는 입력"),
    };
    assert_eq!(sha256(&bytes), expected_source_sha);
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 1, "독립 Print와 전체 쪽수 일치");
    let reference2024 = independent_print(input, "2024", expanded.len());
    let reference2020 = independent_print(input, "2020", expanded.len());
    let unchanged = vec![false; expanded.len()];
    let screen = verify_profile(&core, RenderProfile::Screen, &unchanged, None);
    let print = verify_profile(&core, RenderProfile::Print, expanded, Some(&reference2024));
    assert_eq!(
        screen,
        verify_profile(&core, RenderProfile::Screen, &unchanged, None)
    );
    assert_eq!(
        print,
        verify_profile(&core, RenderProfile::Print, expanded, Some(&reference2024))
    );
    for ((a, b), changes) in screen.iter().zip(&print).zip(expanded) {
        if *changes {
            assert_ne!(a, b, "원본색·Print 전달색의 캐시 신원 분리");
        }
    }
    core.set_hangul2024_compat(false);
    verify_profile(
        &core,
        RenderProfile::Print,
        &unchanged,
        Some(&reference2020),
    );
    core.set_hangul2024_compat(true);
    verify_profile(&core, RenderProfile::Print, expanded, Some(&reference2024));
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
