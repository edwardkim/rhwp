//! 관계형: PNG 출처·CRC·보간·픽셀 및 WMFC Print 표면 소유 검사. 페이지 px 핀 없음.
//! 주석 갱신: 2026-11-03T11:47:31+09:00. 직전 변경 3739843004, 2026-11-03T11:29:07+09:00.
//! 독립 Print 96 DPI, 2024 호환, 전 64쪽: Native 최저 94.90%, fresh WASM 최저 94.92%.
//! 근거: mydocs/pr/assets/issue_8305/followup8438/policy-resave.hwpx,
//! pdf/issue8305/batch-8341-policy-resave-2024.pdf,
//! output/pr-review/planet6897-ordered-20261031/book-validation/chart-print-frame-all-{native,wasm}/chart-print-frame-all/silhouette.tsv.
//! PNG 계약의 실제 제품 소스와 Print paint 경로를 통합 suite에서 검사한다.
//! private 함수를 테스트용 공개 API로 늘리지 않고 같은 원본 모듈을 사용한다.
#[allow(dead_code)]
#[path = "../../src/renderer/png_sampling.rs"]
mod png_sampling;
use png_sampling::*;

fn encoded_color() -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([228, 217, 128, 173]));
    let mut output = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Png)
        .unwrap();
    output.into_inner()
}

#[test]
fn print_transfer_marker_keeps_source_color_alpha_and_dimensions() {
    let source = encoded_color();
    let mut printed = source.clone();
    mark_print_bitmap(&mut printed);
    assert!(print_bitmap_reconstruction(&printed));
    assert_eq!(interpolation(&printed), None);
    assert_eq!(
        image::load_from_memory(&printed).unwrap().to_rgba8(),
        image::load_from_memory(&source).unwrap().to_rgba8(),
        "표식은 실제 색·알파·해상도를 바꾸면 안 된다",
    );
    let mut surface = source.clone();
    mark_metafile_surface(&mut surface, false);
    assert!(metafile_surface(&surface));
    assert!(!intermediate_metafile_surface(&surface));
    assert!(!print_bitmap_reconstruction(&surface));
    assert!(!metafile_surface(&printed));
    assert_eq!(interpolation(&surface), None);
    let mut intermediate = source.clone();
    mark_metafile_surface(&mut intermediate, true);
    assert!(intermediate_metafile_surface(&intermediate));
    assert!(
        !metafile_surface(&intermediate),
        "중간 표면은 최종 장치 표면이 아니다"
    );
    assert_eq!(
        image::load_from_memory(&intermediate).unwrap().to_rgba8(),
        image::load_from_memory(&source).unwrap().to_rgba8()
    );
    assert_eq!(
        image::load_from_memory(&surface).unwrap().to_rgba8(),
        image::load_from_memory(&source).unwrap().to_rgba8(),
        "최종 장치 표본 표식도 색·알파·해상도를 보존한다",
    );
}

#[test]
fn invalid_crc_truncation_and_unknown_marker_version_never_select_reconstruction() {
    let mut printed = encoded_color();
    mark_print_bitmap(&mut printed);
    let mut corrupt = printed.clone();
    corrupt[46] ^= 1;
    assert!(!print_bitmap_reconstruction(&corrupt));
    assert!(!print_bitmap_reconstruction(&printed[..46]));
    printed[41] = 3;
    let crc = crc32fast::hash(&printed[37..43]);
    printed[43..47].copy_from_slice(&crc.to_be_bytes());
    assert!(!print_bitmap_reconstruction(&printed));
    assert_eq!(interpolation(&printed), None);
}

#[test]
fn explicit_source_sampling_overrides_print_transfer_marker_without_duplication() {
    let mut source = encoded_color();
    mark_print_bitmap(&mut source);
    source[41] = 1;
    source[42] = 0;
    let crc = crc32fast::hash(&source[37..43]);
    source[43..47].copy_from_slice(&crc.to_be_bytes());
    let mut printed = encoded_color();
    mark_print_bitmap(&mut printed);
    preserve(&source, &mut printed);
    assert_eq!(interpolation(&printed), Some(false));
    assert!(!print_bitmap_reconstruction(&printed));
    let once = printed.clone();
    preserve(&source, &mut printed);
    assert_eq!(printed, once, "같은 계약을 두 번 싣지 않는다");
    assert_eq!(
        image::load_from_memory(&printed).unwrap().to_rgba8(),
        image::load_from_memory(&source).unwrap().to_rgba8(),
    );
}

#[test]
fn pdf_colour_provenance_is_independent_of_optional_interpolation_and_crc_checked() {
    for (value, expected) in [(0, Some(false)), (1, Some(true)), (2, None)] {
        let mut source = encoded_color();
        mark_print_bitmap(&mut source);
        source[41] = 6;
        source[42] = value;
        let crc = crc32fast::hash(&source[37..43]);
        source[43..47].copy_from_slice(&crc.to_be_bytes());
        assert!(pdf_source(&source));
        assert_eq!(interpolation(&source), expected);
        assert!(!print_bitmap_reconstruction(&source));
        let mut copied = encoded_color();
        preserve(&source, &mut copied);
        assert_eq!(copied, source);
        preserve(&source, &mut copied);
        assert_eq!(copied, source, "출처 보존은 중복 chunk를 만들지 않는다");
        copied[46] ^= 1;
        assert!(!pdf_source(&copied), "손상 선언은 PDF 출처가 아니다");
    }
    let mut invalid = encoded_color();
    mark_print_bitmap(&mut invalid);
    assert!(!pdf_source(&invalid), "한컴 Print와 PDF 출처는 별개다");
    invalid[41] = 6;
    invalid[42] = 3;
    let crc = crc32fast::hash(&invalid[37..43]);
    invalid[43..47].copy_from_slice(&crc.to_be_bytes());
    assert!(!pdf_source(&invalid), "정의하지 않은 보간 값은 거부한다");
}

#[test]
fn wmfc_chart_print_frame_is_consumed_once_in_the_product_paint_path() {
    use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile, ResolvedImageKind};
    use rhwp::renderer::render_tree::BoundingBox;

    fn images<'a>(node: &'a LayerNode, out: &mut Vec<(BoundingBox, &'a [u8], ResolvedImageKind)>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    images(child, out);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                images(child, out);
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::Image {
                        bbox,
                        resolved: Some(payload),
                        ..
                    } = op
                    {
                        out.push((*bbox, &payload.data, payload.kind));
                    }
                }
            }
        }
    }

    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mydocs/pr/assets/issue_8305/followup8438/policy-resave.hwpx");
    let mut core = rhwp::DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let print = core
        .build_page_layer_tree_with_profile(27, RenderProfile::Print)
        .unwrap();
    let mut print_images = Vec::new();
    images(&print.root, &mut print_images);
    let printed: Vec<_> = print_images
        .iter()
        .filter(|(_, bytes, _)| metafile_surface(bytes))
        .collect();
    assert!(
        !printed.is_empty(),
        "원본 28쪽 Print에 WMFC 최종 표면이 있어야 한다: 이미지 {}개",
        print_images.len()
    );
    assert_eq!(printed.len(), 1, "원본 28쪽의 WMFC 차트를 한 번만 발행한다");
    assert!(
        print_images
            .iter()
            .all(|(_, bytes, _)| !intermediate_metafile_surface(bytes)),
        "중간 표면은 Print 소비자에게 남지 않는다"
    );
    for (bounds, bytes, kind) in printed {
        assert_eq!(*kind, ResolvedImageKind::OutputDevice);
        let raster = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
            .unwrap()
            .to_rgba8();
        assert_eq!(bounds.width, f64::from(raster.width()));
        assert_eq!(bounds.height, f64::from(raster.height()));
        assert!(bounds.x > 0.0 && bounds.y > 0.0);
    }
}
