//! `ColorRef` 도메인 규칙.
//!
//! HWP 계열은 Windows `COLORREF` 규약을 따른다 — 하위 3바이트가 `0x00BBGGRR` 이고
//! **상위 바이트가 0이 아니면 "색 없음/자동"**(`CLR_INVALID`/`CLR_DEFAULT`)이다.
//! 이 규칙이 파서·렌더러·판정기에 흩어져 조금씩 다르게 재구현되면서
//! #3546·#3557·#4155 가 반복됐다. 정의는 여기 하나뿐이다.

use super::ColorRef;

/// "색 없음/자동" sentinel.
///
/// 추정이 아니라 실측 정본이다 — HWPX `shadeColor="none"` 이 파싱되는 값
/// (`parser/hwpx/utils.rs` `parse_color_str`), 한/글이 HML 에 쓰는 `4294967295`
/// (`samples/hml/*.hml` 2/2), 한컴산 HWP5 코퍼스 380건의 `CHAR_SHAPE` 음영색
/// `0xffffffff` × 22,189 (검정 `0x00000000` 은 전수에서 0건, #4155 실측).
pub const NONE: ColorRef = 0xFFFF_FFFF;
