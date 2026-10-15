//! 외부 소비자에게 전달할 문단 줄 캐시의 유효성.

use crate::model::paragraph::{tab_ext_is_placeholder, Paragraph};

/// 편집/재조판 결과의 미계산 탭은 소비자가 다시 계산하도록 캐시를 생략한다.
/// 원본에서 읽은 정상 줄 분할은 미계산 탭 하나만으로 폐기하지 않는다.
/// `layout_space_metrics`는 줄 구성 결과와 함께 게시되는 재조판 출처이며,
/// 원본 저장 행에는 없다. dirty 행과 실제 재생성 행의 계약을 구분한다.
pub(super) fn can_serialize_line_partition(para: &Paragraph) -> bool {
    if para.stored_text_partition_is_dirty() {
        return false;
    }
    if para.layout_space_metrics.is_empty() {
        return true;
    }
    !para
        .text
        .chars()
        .filter(|&ch| ch == '\t')
        .enumerate()
        .any(|(index, _)| {
            para.tab_extended
                .get(index)
                .is_none_or(tab_ext_is_placeholder)
        })
}
