//! 렌더러가 합성하는 한컴 PUA 기호의 테두리·안쪽 글자 메트릭을 글꼴 em으로 투영한다.
//! Haansoft Batang의 일반 글자와 같은 프로그램의 PUA 글리프 경계에서 정한다.
//! 사유 글꼴 바이트·윤곽선은 포함하지 않는다. 안쪽 글자는 공급된 일반 글리프로 표시한다.

use super::EnclosedPua;

#[derive(serde::Deserialize)]
struct InnerBounds {
    source: [f64; 4],
    target: [f64; 4],
}
#[derive(serde::Deserialize)]
struct BoxedDigit {
    frame: [f64; 5],
    digits: Vec<InnerBounds>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    units_per_em: f64,
    inner_face: String,
    boxed_digit: BoxedDigit,
}
fn metrics() -> &'static Metrics {
    static METRICS: std::sync::OnceLock<Metrics> = std::sync::OnceLock::new();
    METRICS.get_or_init(|| {
        serde_json::from_str(include_str!("../../assets/fonts/enclosed-pua-metrics.json"))
            .expect("동봉한 합성 PUA 메트릭")
    })
}

/// 안쪽 글자를 먼저 요청할 face. 공급되지 않으면 문서 글꼴 체인으로 내려간다.
pub(crate) fn inner_face() -> &'static str {
    &metrics().inner_face
}

pub(crate) struct EnclosedGeometry {
    pub(crate) frame_x: f64,
    pub(crate) frame_y: f64,
    pub(crate) frame_width: f64,
    pub(crate) frame_height: f64,
    pub(crate) stroke_width: f64,
    pub(crate) inner_scale_x: f64,
    pub(crate) inner_scale_y: f64,
    pub(crate) inner_x: f64,
    pub(crate) inner_y: f64,
}

/// 글자 원점·baseline 기준 기하. 동봉 메트릭이 없는 다자리 네모 숫자는 `None`이다.
pub(crate) fn geometry(glyph: EnclosedPua, font_size: f64) -> Option<EnclosedGeometry> {
    let metrics = metrics();
    let EnclosedPua::BoxedNumber(number) = glyph;
    let frame = &metrics.boxed_digit.frame;
    let InnerBounds { source, target } = metrics.boxed_digit.digits.get(number as usize)?;
    let unit = font_size / metrics.units_per_em;
    let scale_x = (target[2] - target[0]) / (source[2] - source[0]);
    let scale_y = (target[3] - target[1]) / (source[3] - source[1]);
    Some(EnclosedGeometry {
        // 바깥/안쪽 테두리 경계의 중간에 stroke 중심선을 둔다.
        frame_x: frame[0] * unit,
        frame_y: frame[1] * unit,
        frame_width: frame[2] * unit,
        frame_height: frame[3] * unit,
        stroke_width: frame[4] * unit,
        inner_scale_x: scale_x,
        inner_scale_y: scale_y,
        inner_x: (target[0] - source[0] * scale_x) * unit,
        inner_y: (-target[1] + source[1] * scale_y) * unit,
    })
}
