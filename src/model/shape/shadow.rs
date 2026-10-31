//! HWP 도형 그림자와 HWPX의 공식 이름 대응.

const HWPX_NAMES: [&str; 15] = [
    "NONE",
    "PARELLEL_LEFTTOP",
    "PARELLEL_RIGHTTOP",
    "PARELLEL_LEFTBOTTOM",
    "PARELLEL_RIGHTBOTTOM",
    "SHEAR_LEFTTOP",
    "SHEAR_RIGHTTOP",
    "SHEAR_LEFTBOTTOM",
    "SHEAR_RIGHTBOTTOM",
    "PERS_LEFTTOP",
    "PERS_RIGHTTOP",
    "PERS_LEFTBOTTOM",
    "PERS_RIGHTBOTTOM",
    "SCALE_NARROW",
    "SCALE_ENLARGE",
];

pub(crate) fn from_hwpx(name: &str) -> u32 {
    HWPX_NAMES
        .iter()
        .position(|candidate| *candidate == name)
        .map(|index| index as u32)
        .unwrap_or_else(|| match name {
            // 공식 모델의 이전 버전 읽기 별칭과 과거 rhwp 저장본을 수용한다.
            "DROP" | "LEFT_TOP" => 1,
            "CONTINUOUS" | "RIGHT_TOP" => 2,
            "LEFT_BOTTOM" => 3,
            "RIGHT_BOTTOM" => 4,
            "CENTER" | "INSIDE" | "OUTSIDE" => 5,
            _ => 0,
        })
}

pub(crate) fn to_hwpx(kind: u32) -> &'static str {
    HWPX_NAMES.get(kind as usize).copied().unwrap_or("NONE")
}

/// 평행 그림자의 기본 이동량. 저장 offset은 이 값에 부호를 보존해 더한다.
/// 다른 계열의 기울임·원근·배율을 단순 이동으로 대체하지 않는다.
pub(crate) fn parallel_preset_hu(kind: u32) -> Option<(i32, i32)> {
    match kind {
        1 => Some((-600, -600)),
        2 => Some((600, -600)),
        3 => Some((-600, 600)),
        4 => Some((600, 600)),
        _ => None,
    }
}

/// 독립 개체의 선언 원점은 본체와 그림자를 합친 외곽의 왼쪽 위다.
/// 묶음 자식의 renderingInfo 원점과는 별개의 배치 계약이다.
pub(crate) fn parallel_body_origin_hu(kind: u32, x: i32, y: i32) -> Option<(f64, f64)> {
    let (base_x, base_y) = parallel_preset_hu(kind)?;
    Some((
        (-(f64::from(base_x) + f64::from(x))).max(0.0),
        (-(f64::from(base_y) + f64::from(y))).max(0.0),
    ))
}
