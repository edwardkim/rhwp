//! 한컴 정상 저장과 Print가 정하는 HWP3 이중선의 전체 굵기다.

/// HWP3 선 종류 4는 개별 stroke가 아니라 이중선 전체 0.5mm로 정규화한다.
/// 정상 HWPX의 DOUBLE_SLIM 폭과 원본 Print의 0.36pt stroke·1.08pt 중심 간격에 대응한다.
/// 쪽과 셀은 같은 기본 굵기를 사용하고 실제 선 배치는 공통 border geometry가 정한다.
pub(super) const DOUBLE_LINE_WIDTH: u8 = 7;
