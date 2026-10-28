//! 반복 비트맵의 마스크를 원래 표본 격자에서 합성한 뒤 축소한다.
use resvg::{tiny_skia, usvg};

fn contains_image(group: &usvg::Group) -> bool {
    group.children().iter().any(|node| match node {
        usvg::Node::Image(_) => true,
        usvg::Node::Group(group) => contains_image(group),
        _ => false,
    })
}

fn uses_image_pattern(group: &usvg::Group) -> bool {
    group.children().iter().any(|node| match node {
        usvg::Node::Group(group) => uses_image_pattern(group),
        usvg::Node::Text(text) => uses_image_pattern(text.flattened()),
        usvg::Node::Path(path) => path
            .fill()
            .map(|fill| fill.paint())
            .into_iter()
            .chain(path.stroke().map(|stroke| stroke.paint()))
            .any(|paint| matches!(paint, usvg::Paint::Pattern(pattern) if contains_image(pattern.root()))),
        _ => false,
    })
}

/// 반환 PNG의 크기는 종전 요청과 같다. crop·fill·최종 배치의 좌표 계약은 유지한다.
/// `transform`은 문서 좌표를 `width`×`height` 목적지 픽셀로 옮긴다.
pub(super) fn rasterize_before_reduction(
    tree: &usvg::Tree,
    width: u32,
    height: u32,
    transform: tiny_skia::Transform,
    pixel_limit: u64,
) -> Option<Vec<u8>> {
    let size = tree.size();
    if (size.width() <= width as f32 && size.height() <= height as f32)
        || !tree
            .masks()
            .iter()
            .any(|mask| uses_image_pattern(mask.root()))
    {
        return None;
    }
    if !(transform.sx > 0.0 && transform.sy > 0.0) {
        return None;
    }
    // 목적지 창이 덮는 문서 범위를 문서 1단위=1표본으로 그린다. 위상도 정수 표본으로 옮겨
    // 8px 무늬 타일이 소수 배율·소수 이동으로 재표본되어 옅어지지 않게 한다.
    let source_width = ((width as f32 / transform.sx).ceil() as u64).max(u64::from(width));
    let source_height = ((height as f32 / transform.sy).ceil() as u64).max(u64::from(height));
    if source_width.checked_mul(source_height)? > pixel_limit {
        return None;
    }
    let source_width = u32::try_from(source_width).ok()?;
    let source_height = u32::try_from(source_height).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(source_width, source_height)?;
    let transform = tiny_skia::Transform::from_translate(
        (transform.tx / transform.sx).round(),
        (transform.ty / transform.sy).round(),
    );
    // 반복 타일을 목적지의 작은 정수 격자로 먼저 뭉개지 않는다.
    // 흑백 마스크와 앞뒤 칠을 원래 표본에서 합성한 뒤에만 재표본한다.
    resvg::render(tree, transform, &mut pixmap.as_mut());
    encode_reduced(&pixmap, width, height)
}

/// 큰 격자의 합성 결과를 면적으로 저역 통과해 작은 글자 획과 premultiplied 색을 보존한다.
pub(super) fn encode_reduced(
    pixmap: &tiny_skia::Pixmap,
    width: u32,
    height: u32,
) -> Option<Vec<u8>> {
    let source = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        pixmap.width(),
        pixmap.height(),
        pixmap.data(),
    )?;
    let mut reduced = image::imageops::resize(
        &source,
        width,
        height,
        image::imageops::FilterType::Lanczos3,
    );
    // Lanczos의 작은 overshoot만 premultiplied RGBA 불변식 안으로 제한한다.
    for pixel in reduced.pixels_mut() {
        for channel in 0..3 {
            pixel[channel] = pixel[channel].min(pixel[3]);
        }
    }
    let mut output = tiny_skia::Pixmap::new(width, height)?;
    output.data_mut().copy_from_slice(reduced.as_raw());
    output.encode_png().ok()
}
