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

/// 평행·작게·크게 그림자의 기본 이동량(HWPUNIT). 저장 offset은 이 값에 부호를 보존해 더한다.
const PRESET_HU: f64 = 600.0;
/// 한/글 12.30 화면에서 잰 작게·크게 그림자의 배율.
const NARROW_SCALE: f64 = 0.75;
const ENLARGE_SCALE: f64 = 1.25;
/// 기울임 그림자는 본체 높이의 반이고, 먼 변이 본체 높이의 9/16만큼 옆으로 밀린다(한/글 12.30 화면에서 잰 값).
const SHEAR_HEIGHT: f64 = 0.5;
const SHEAR_SLOPE: f64 = 0.5625;

/// 평행 그림자의 기본 이동량. 저장 offset은 이 값에 부호를 보존해 더한다.
pub(crate) fn parallel_preset_hu(kind: u32) -> Option<(i32, i32)> {
    match kind {
        1 => Some((-600, -600)),
        2 => Some((600, -600)),
        3 => Some((-600, 600)),
        4 => Some((600, 600)),
        _ => None,
    }
}

/// 본체 상자(x, y, w, h) 위의 점을 그림자 자리로 옮기는 아핀 행렬 `[a, b, c, d, e, f]`
/// (x' = a·x + c·y + e, y' = b·x + d·y + f). `unit`은 HWPUNIT 하나의 길이, `offset`은 저장 offset이다.
/// 평행은 이동, 작게·크게는 왼쪽 위·오른쪽 아래 모서리를 기준으로 줄이고 늘린 뒤 왼쪽 위로 옮기며,
/// 기울임은 아래 변에서 위(뒤쪽 그림자) 또는 아래(앞쪽 그림자, 상하로 뒤집힘)로 반 높이만큼 눕힌다.
/// 원근(9~12)은 한/글 Mac에서 고를 수 없고 단순 이동으로 대체하지 않으므로 None이다.
pub(crate) fn matrix(
    kind: u32,
    [x, y, w, h]: [f64; 4],
    unit: f64,
    (offset_x, offset_y): (f64, f64),
) -> Option<[f64; 6]> {
    let d = PRESET_HU * unit;
    let bottom = y + h;
    let m = match kind {
        1..=4 => {
            let (px, py) = parallel_preset_hu(kind)?;
            [
                1.0,
                0.0,
                0.0,
                1.0,
                f64::from(px) * unit,
                f64::from(py) * unit,
            ]
        }
        13 | 14 => {
            let (s, ax, ay) = if kind == 13 {
                (NARROW_SCALE, x, y)
            } else {
                (ENLARGE_SCALE, x + w, bottom)
            };
            [s, 0.0, 0.0, s, ax - s * ax - d, ay - s * ay - d]
        }
        5..=8 => {
            // 5 왼쪽 위, 6 오른쪽 위, 7 왼쪽 아래, 8 오른쪽 아래
            let c = if matches!(kind, 5 | 7) {
                SHEAR_SLOPE
            } else {
                -SHEAR_SLOPE
            };
            let dy = if kind <= 6 {
                SHEAR_HEIGHT
            } else {
                -SHEAR_HEIGHT
            };
            [1.0, 0.0, c, dy, -c * bottom, bottom - dy * bottom]
        }
        _ => return None,
    };
    let m = [m[0], m[1], m[2], m[3], m[4] + offset_x, m[5] + offset_y];
    m.iter().all(|v| v.is_finite()).then_some(m)
}

/// 행렬로 옮긴 상자의 외곽(x, y, w, h).
pub(crate) fn transformed_box(m: [f64; 6], [x, y, w, h]: [f64; 4]) -> [f64; 4] {
    let corners = [(x, y), (x + w, y), (x, y + h), (x + w, y + h)]
        .map(|(px, py)| (m[0] * px + m[2] * py + m[4], m[1] * px + m[3] * py + m[5]));
    let min_x = corners.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let min_y = corners.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_x = corners
        .iter()
        .map(|p| p.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = corners
        .iter()
        .map(|p| p.1)
        .fold(f64::NEG_INFINITY, f64::max);
    [min_x, min_y, max_x - min_x, max_y - min_y]
}

/// 독립 개체의 선언 원점은 본체와 그림자를 합친 외곽의 왼쪽 위다(한/글 12.30 화면에서 확인).
/// 본체를 원점에서 얼마나 밀어야 하는지(HWPUNIT) 돌려준다.
/// 묶음 자식의 renderingInfo 원점과는 별개의 배치 계약이다.
pub(crate) fn body_origin_hu(kind: u32, x: i32, y: i32, w: f64, h: f64) -> Option<(f64, f64)> {
    let m = matrix(kind, [0.0, 0.0, w, h], 1.0, (f64::from(x), f64::from(y)))?;
    let [sx, sy, _, _] = transformed_box(m, [0.0, 0.0, w, h]);
    Some(((-sx).max(0.0), (-sy).max(0.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shadow_box(kind: u32, w: f64, h: f64) -> [f64; 4] {
        transformed_box(
            matrix(kind, [0.0, 0.0, w, h], 1.0, (0.0, 0.0)).unwrap(),
            [0.0, 0.0, w, h],
        )
    }

    /// 한/글 12.30이 12000×8000 사각형에 그린 그림자 외곽(HWPUNIT, 본체 왼쪽 위 기준).
    #[test]
    fn preset_shadow_boxes_match_hancom_screen() {
        assert_eq!(
            shadow_box(1, 12000.0, 8000.0),
            [-600.0, -600.0, 12000.0, 8000.0]
        );
        assert_eq!(
            shadow_box(13, 12000.0, 8000.0),
            [-600.0, -600.0, 9000.0, 6000.0]
        );
        assert_eq!(
            shadow_box(14, 12000.0, 8000.0),
            [-3600.0, -2600.0, 15000.0, 10000.0]
        );
        // 뒤쪽 기울임: 아래 변은 본체와 같고 위 변은 반 높이에서 4500만큼 밀린다.
        assert_eq!(
            shadow_box(5, 12000.0, 8000.0),
            [-4500.0, 4000.0, 16500.0, 4000.0]
        );
        assert_eq!(
            shadow_box(6, 12000.0, 8000.0),
            [0.0, 4000.0, 16500.0, 4000.0]
        );
        // 앞쪽 기울임: 본체 아래 변에서 아래로 반 높이.
        assert_eq!(
            shadow_box(7, 12000.0, 8000.0),
            [-4500.0, 8000.0, 16500.0, 4000.0]
        );
        assert_eq!(
            shadow_box(8, 6000.0, 12000.0),
            [0.0, 12000.0, 12750.0, 6000.0]
        );
        assert_eq!(matrix(9, [0.0, 0.0, 1.0, 1.0], 1.0, (0.0, 0.0)), None);
    }

    /// 앞쪽 기울임은 상하로 뒤집힌다: 직각삼각형의 위 꼭짓점이 그림자의 아래 먼 끝으로 간다.
    #[test]
    fn front_shear_mirrors_the_body() {
        let m = matrix(7, [0.0, 0.0, 12000.0, 8000.0], 1.0, (0.0, 0.0)).unwrap();
        let apex = (
            m[0] * 0.0 + m[2] * 0.0 + m[4],
            m[1] * 0.0 + m[3] * 0.0 + m[5],
        );
        assert_eq!(apex, (-4500.0, 12000.0));
    }

    #[test]
    fn body_origin_moves_past_left_and_top_shadow() {
        assert_eq!(
            body_origin_hu(1, 0, 0, 12000.0, 8000.0),
            Some((600.0, 600.0))
        );
        assert_eq!(body_origin_hu(4, 0, 0, 12000.0, 8000.0), Some((0.0, 0.0)));
        assert_eq!(
            body_origin_hu(14, 0, 0, 12000.0, 8000.0),
            Some((3600.0, 2600.0))
        );
        assert_eq!(
            body_origin_hu(7, 0, 0, 12000.0, 8000.0),
            Some((4500.0, 0.0))
        );
        assert_eq!(
            body_origin_hu(1, 100, -200, 12000.0, 8000.0),
            Some((500.0, 800.0))
        );
    }
}
