//! 브라우저는 `<img>`·canvas·SVG `<image>`에서 JPEG EXIF 방향(Orientation)을 적용해 그림을
//! 돌려 그린다. 한/글은 그림 바이트를 방향과 상관없이 그대로 그리고, 돌림은 개체 회전으로만 둔다.
//! 브라우저 렌더러에는 방향 값만 1로 바꾼 사본을 넘겨 한/글과 같은 화소 배치로 그리게 한다.

use crate::renderer::image_header::find_jpeg_segment;
use image::metadata::Orientation;

/// 브라우저에 넘길 JPEG 사본의 EXIF 방향만 바로잡는다. 변환이 필요 없으면 복사하지 않는다.
pub fn upright_jpeg(data: &[u8]) -> Option<Vec<u8>> {
    const EXIF: &[u8] = b"Exif\0\0";
    let app1 = find_jpeg_segment(data, |marker, payload| {
        marker == 0xe1 && payload.starts_with(EXIF)
    })?;
    let tiff = app1.start + EXIF.len()..app1.end;
    if Orientation::from_exif_chunk(&data[tiff.clone()])? == Orientation::NoTransforms {
        return None;
    }

    let mut output = data.to_vec();
    Orientation::remove_from_exif_chunk(&mut output[tiff])?;
    Some(output)
}
