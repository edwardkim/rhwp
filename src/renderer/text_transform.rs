//! 실제 기울임 face가 없을 때 사용하는 글리프 변환.
//!
//! 독립 7개 face·3개 크기의 2020/2024 Print는 합성 기울임 Tm에
//! 0.333을 기록하며, 실제 Times Italic 프로그램은 추가 shear가 없다.
//! 글줄의 원점·전진폭은 바꾸지 않고 실제 face 선택 뒤 paint만 소비한다.
pub(crate) const SYNTHETIC_ITALIC_SKEW: f64 = -0.333;
