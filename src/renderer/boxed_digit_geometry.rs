//! 실제 한컴 단자리 네모 숫자의 프레임·숫자 메트릭을 글꼴 em으로 투영한다.
//! Haansoft Batang의 일반 숫자와 같은 프로그램의 네모 숫자 경계에서 정한다.
//! 사유 글꼴 바이트·윤곽선은 포함하지 않는다. 숫자는 공급된 일반 글리프로 표시한다.

#[derive(serde::Deserialize)]
struct DigitBounds {
    source: [f64; 4],
    target: [f64; 4],
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    units_per_em: f64,
    number_face: String,
    frame: [f64; 5],
    digits: Vec<DigitBounds>,
}
fn metrics() -> &'static Metrics {
    static METRICS: std::sync::OnceLock<Metrics> = std::sync::OnceLock::new();
    METRICS.get_or_init(|| {
        serde_json::from_str(include_str!("../../assets/fonts/boxed-digit-metrics.json"))
            .expect("동봉한 네모 숫자 메트릭")
    })
}

pub(crate) fn number_face() -> &'static str {
    &metrics().number_face
}

pub(crate) struct SingleDigitGeometry {
    pub(crate) frame_x: f64,
    pub(crate) frame_y: f64,
    pub(crate) frame_width: f64,
    pub(crate) frame_height: f64,
    pub(crate) stroke_width: f64,
    pub(crate) number_scale_x: f64,
    pub(crate) number_scale_y: f64,
    pub(crate) number_x: f64,
    pub(crate) number_y: f64,
}

pub(crate) fn single_digit(number: u32, font_size: f64) -> Option<SingleDigitGeometry> {
    let metrics = metrics();
    let DigitBounds { source, target } = metrics.digits.get(number as usize)?;
    let unit = font_size / metrics.units_per_em;
    let scale_x = (target[2] - target[0]) / (source[2] - source[0]);
    let scale_y = (target[3] - target[1]) / (source[3] - source[1]);
    Some(SingleDigitGeometry {
        // 바깥/안쪽 사각 경계의 중간에 stroke 중심선을 둔다.
        frame_x: metrics.frame[0] * unit,
        frame_y: metrics.frame[1] * unit,
        frame_width: metrics.frame[2] * unit,
        frame_height: metrics.frame[3] * unit,
        stroke_width: metrics.frame[4] * unit,
        number_scale_x: scale_x,
        number_scale_y: scale_y,
        number_x: (target[0] - source[0] * scale_x) * unit,
        number_y: (-target[1] + source[1] * scale_y) * unit,
    })
}
