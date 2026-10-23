//! 2024 Print의 bitmap 채움: 원본 픽셀 창과 물리 EMF 프레임의 대응.

use crate::renderer::render_tree::BoundingBox;

pub(super) struct BitmapFillFrame {
    pub image: BoundingBox,
    pub clip: BoundingBox,
}

/// source rect는 원본 bitmap 픽셀 수이고, EMF의 물리 프레임과는 다르다.
/// ENHMETAHEADER의 0.01mm 프레임과 포함 경계를 같은 단위로 환산한다.
/// 참조 장치는 2024 Print bitmap carrier의 계약이며 현재 화면 DPI가 아니다.
fn replay_extent(source: u32, destination: f64, pixels: u64, millimeters: u64) -> Option<f64> {
    let source = u64::from(source);
    let frame_himetric = source.checked_mul(millimeters)?.checked_mul(100)? / pixels;
    let frame_pixels = frame_himetric as f64 * pixels as f64 / (millimeters * 100) as f64;
    if source == 0 || frame_pixels <= 0.0 || destination <= 0.0 {
        return None;
    }
    // GDI+의 명시적 source rect는 bitmap 크기다. 전체 Metafile의 자동 Bounds를
    // 선택하면 이 포함 픽셀이 사라져 작은 원본과 큰 원본의 확대가 서로 달라진다.
    let projected = (destination + 2.0) * (frame_pixels + 1.0) / source as f64;
    // GDI의 28.4 장치 좌표에서 끝점을 정수화한 뒤, 포함 경계의 마지막 픽셀을
    // RECT의 제외 끝점으로 변환한다. 단순 floor는 1/16 픽셀 경계에서 달라진다.
    let fixed_endpoint = (projected * 16.0 + 0.5).floor() / 16.0;
    let playback = fixed_endpoint.ceil() - 1.0;
    let scale = (playback / frame_pixels) as f32;
    let extent = f64::from(scale) * source as f64;
    (extent.is_finite() && extent > 0.0).then_some(extent)
}

pub(super) fn map(
    device: BoundingBox,
    source: (u32, u32),
    device_page: (f64, f64),
    output_page: (f64, f64),
) -> Option<BitmapFillFrame> {
    if ![device.x, device.y, device.width, device.height]
        .into_iter()
        .all(f64::is_finite)
        || device.width <= 0.0
        || device.height <= 0.0
        || ![device_page.0, device_page.1, output_page.0, output_page.1]
            .into_iter()
            .all(|v| v.is_finite() && v > 0.0)
    {
        return None;
    }
    let x_unit = output_page.0 / device_page.0;
    let y_unit = output_page.1 / device_page.1;
    let width = replay_extent(source.0, device.width, 1024, 271)?;
    let height = replay_extent(source.1, device.height, 768, 203)?;
    Some(BitmapFillFrame {
        clip: BoundingBox::new(
            device.x * x_unit,
            device.y * y_unit,
            device.width * x_unit,
            device.height * y_unit,
        ),
        // 그림만 장치 한 픽셀씩 확장한다. 셀·문단의 원래 clip은 위에서 보존한다.
        image: BoundingBox::new(
            (device.x - 1.0) * x_unit,
            (device.y - 1.0) * y_unit,
            width * x_unit,
            height * y_unit,
        ),
    })
}
