//! 외부 소비자에게 전달할 문단 줄 캐시의 유효성.

use crate::model::paragraph::{tab_ext_is_placeholder, Paragraph};

/// 원본 저장 위치의 스냅샷은 현재 줄 수와 대응할 때만 유효하다.
pub(super) fn source_vertical_positions(para: &Paragraph) -> Option<&[i32]> {
    para.source_line_seg_vertical_pos
        .as_deref()
        .filter(|positions| positions.len() == para.line_segs.len())
}

/// 구역 본문 문단을 차례로 쓰며 줄 세로 위치를 한 좌표계로 맞춘다.
///
/// HWPX 에서 읽은 문단은 원본 위치 스냅샷(#5847)으로 나가고, 렌더는 구역 누적 좌표를 쓴다.
/// 본문 문단을 고치면 그 문단은 스냅샷을 잃고 다시 조판한 줄만 남는데, 그 줄의 세로 위치는
/// 렌더 좌표다. 그대로 실으면 이웃의 원본 좌표와 어긋난 값이 파일에 들어가, 다시 열 때 그
/// 어긋남이 저장된 쪽·단 경계로 읽혀 쪽 나눔이 편집 세션과 달라진다. 그런 문단은 앞 문단이
/// 파일에 실은 위치와 렌더 좌표의 차이만큼 옮겨, 앞 문단에 이어지는 원본 좌표로 싣는다.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BodyLineFrame {
    keeps_source: bool,
    /// 앞 문단이 파일에 실은 첫 줄 위치에서 렌더 좌표를 뺀 값.
    shift: Option<i32>,
}

impl BodyLineFrame {
    pub(crate) fn for_body(paragraphs: &[Paragraph]) -> Self {
        Self {
            keeps_source: paragraphs
                .iter()
                .any(|para| source_vertical_positions(para).is_some()),
            shift: None,
        }
    }

    /// 이 문단이 파일에 실을 줄 위치. `None` 이면 줄이 지닌 값을 그대로 싣는다.
    pub(crate) fn positions(&mut self, para: &Paragraph) -> Option<Vec<i32>> {
        let first = para.line_segs.first()?.vertical_pos;
        let positions = match source_vertical_positions(para) {
            Some(source) => Some(source.to_vec()),
            None if self.keeps_source => self.shift.map(|shift| {
                para.line_segs
                    .iter()
                    .map(|line| line.vertical_pos.saturating_add(shift))
                    .collect()
            }),
            None => None,
        };
        if let Some(stored) = positions.as_ref().and_then(|positions| positions.first()) {
            self.shift = Some(stored.saturating_sub(first));
        }
        positions
    }
}

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

/// 재구성된 축의 단독 표 줄은 문자 분할이 없고 원본 쪽 좌표도 아니다.
/// HWPX 소비자가 표의 물리 상자로 줄을 다시 구성하도록 해당 캐시만 생략한다.
pub(super) fn can_serialize_hwpx_line_partition(para: &Paragraph) -> bool {
    can_serialize_line_partition(para) && !para.rebased_single_table_row()
}
