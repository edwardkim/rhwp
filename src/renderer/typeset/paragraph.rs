//! 문단 조판의 책임 경계.
//!
//! 구성된 줄 조회, 문단 구성 결과의 높이 조회, 저장 줄 간격 판정을 소유한다.
//! 문단 구성은 필요한 관측값을 읽고 결과만 반환한다.
//! fit 예산의 읽기 전용 계산은 fit에, 1회성 보정 소비는 state에 있다.
//! 줄 스캔 뒤의 경계 보정은 split에 있다.
//! fit/분할의 실행 순서 조정은 아직 상위 typeset에 남아 있다.
//! 이 하위 조회들은 원본 IR이나 페이지 상태를 변경하지 않는다.

pub(super) mod context;
pub(super) mod fit;
pub(super) mod format;
pub(super) mod line_queries;
pub(super) mod metrics;
pub(super) mod split;
pub(super) mod stored_lines;
