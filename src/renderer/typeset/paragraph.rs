//! 문단 조판의 책임 경계.
//!
//! 구성된 줄 조회, 문단 구성 결과의 높이 조회, 저장 줄 간격 판정을 소유한다.
//! 구성 결과 생성·fit/분할 조정·상태 적용은 아직 상위 typeset에 남아 있다.
//! 이 하위 조회들은 원본 IR이나 페이지 상태를 변경하지 않는다.

pub(super) mod line_queries;
pub(super) mod metrics;
pub(super) mod stored_lines;
