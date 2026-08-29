//! 하위 호환성을 위한 재내보내기
//!
//! DocumentEvent는 도메인 개념이므로 model::event로 이동하였다.
//! 기존 코드의 `use super::super::event::*` 임포트를 유지하기 위해 재내보내기한다.
