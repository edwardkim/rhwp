//! PNG 계약의 실제 제품 소스를 통합 suite에서 검사한다.
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
    mark_metafile_surface(&mut surface);
    assert!(metafile_surface(&surface));
    assert!(!print_bitmap_reconstruction(&surface));
    assert!(!metafile_surface(&printed));
    assert_eq!(interpolation(&surface), None);
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
