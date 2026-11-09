//! 마커의 원본 머리 속성과 줄 구성·배치가 공유하는 예약 폭.

/// 목록 마커의 문단 머리 속성. 저장 형식이 아니라 사전 계산한 마커의 세션 값이다.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MarkerHead {
    /// HWP5 표 40: bit0-1 정렬, bit2 인스턴스 폭, bit3 자동 내어쓰기,
    /// bit4 본문과의 거리 단위(0 글자 크기 비율 · 1 HWPUNIT).
    pub attr: u32,
    /// 너비 보정값(HWPUNIT).
    pub width_adjust: i16,
    /// 본문과의 거리(bit4에 따라 % 또는 HWPUNIT).
    pub text_distance: i16,
    /// 마커 글자 모양. `u32::MAX`는 참조 없이 문단 글자 모양을 따른다.
    pub char_shape_id: u32,
    /// 번호 형식의 한 자리 참조 예약 폭(em). ASCII 반각 칸도 보존하며 실제 계수기의
    /// 자릿수·글꼴 전진과 구분한다. 내장 `^N`은 수준당 1em이다.
    /// 글머리표에는 부여하지 않으며 bit2가 켜지면 실제 번호 폭을 사용한다.
    pub reference_width_em: Option<f32>,
}
crate::storage::clone_copy!(MarkerHead);
