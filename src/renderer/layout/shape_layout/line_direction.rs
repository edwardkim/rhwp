//! HWP5 표 92와 HWPX isReverseHV가 보존한 직선의 논리 방향.

/// 저장 좌표의 투영과 논리 시작·끝의 선택을 분리한다.
/// 원본 행렬의 부호나 현재 선의 축으로 방향 플래그를 추측하지 않는다.
pub(super) fn endpoints(
    reversed: bool,
    start: (f64, f64),
    end: (f64, f64),
) -> ((f64, f64), (f64, f64)) {
    if reversed {
        (end, start)
    } else {
        (start, end)
    }
}
