//! HWP3 그림 정보의 저장 캡션 띠를 공통 캡션 간격으로 옮긴다.
//!
//! 문서 구조 3.0 표 43의 캡션 세로 크기는 글줄 길이와 그림 사이의
//! 간격을 함께 포함한다. 한컴 정상 저장의 gap은 그 두 저장값의 차이다.
//! 문단의 마지막 줄간격은 별도의 호스트 흐름이므로 여기서 더하지 않는다.

use crate::model::shape::CaptionDirection;

pub(super) fn stored_gap(info: &[u8], direction: CaptionDirection) -> i16 {
    if !matches!(direction, CaptionDirection::Top | CaptionDirection::Bottom) {
        // 옆 캡션은 가로 띠 계약을 확인하기 전까지 세로 저장값으로 추정하지 않는다.
        return 0;
    }
    let Some(frame) = info.get(48..50) else {
        return 0;
    };
    let Some(content) = info.get(50..52) else {
        return 0;
    };
    let frame = u32::from(u16::from_le_bytes([frame[0], frame[1]])) * 4;
    let content = u32::from(u16::from_le_bytes([content[0], content[1]])) * 4;
    frame.saturating_sub(content).min(i16::MAX as u32) as i16
}
