//! 저장 어울림 줄의 차선 증거를 조판과 실제 배치가 함께 소비한다.

use crate::model::{paragraph::Paragraph, shape::TextWrap, table::Table};

/// [#7548] 다음 문단 저장 첫 줄의 차선 증거 — 측정(typeset)과 배치(layout)가 같은 값을 쓴다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StoredLineLaneProbe {
    pub(crate) vertical_pos: i32,
    pub(crate) column_start: i32,
    pub(crate) segment_width: i32,
    /// 같은 문단 원본 저장 줄 중 가장 넓은 폭(전폭 줄 상자).
    pub(crate) full_width: i32,
}

/// 원본 저장 줄(구현 속성 bit31 이 아닌 줄)에서 첫 줄 차선 증거를 뽑는다.
pub(crate) fn stored_line_lane_probe(para: &Paragraph) -> Option<StoredLineLaneProbe> {
    use crate::model::paragraph::LineSeg;
    let original = |seg: &&LineSeg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0;
    let first = para.line_segs.first().filter(original)?;
    let full_width = para
        .line_segs
        .iter()
        .filter(original)
        .map(|seg| seg.segment_width as i32)
        .max()?;
    Some(StoredLineLaneProbe {
        vertical_pos: first.vertical_pos,
        column_start: first.column_start,
        segment_width: first.segment_width as i32,
        full_width,
    })
}

/// [#7548] 어울림(Square) 표 host 다음 문단의 저장 첫 줄이 표 옆 차선에서 시작하는가.
///
/// 한/글은 어울림 표로 흐름을 밀지 않는다. 표 띠와 겹치는 줄은 옆 공간이 있으면
/// 좁혀서 그 자리에 두고, 없으면 띠 아래로 넘긴다. 저장 LineSeg 가 그 결과다.
/// host 의 저장 첫 줄이 표 옆 차선(전폭보다 좁은 cs/sw)을 증언하고, 다음 문단의
/// 첫 줄이 그 차선 안에 있으면 그 줄은 표 옆에 놓인 것이다(21_언어 14쪽 pi=300:
/// host·다음 첫 줄 모두 cs=3455 sw=27581, 둘째 줄부터 전폭 cs=852 sw=30184).
/// host 차선이 없거나(빈 host sw=0) 다음 줄이 표와 가로로 겹치면 표 하단에서 잇는다.
pub(crate) fn square_successor_starts_beside_table(
    host: &Paragraph,
    next: Option<StoredLineLaneProbe>,
    table: &Table,
) -> bool {
    use crate::model::paragraph::LineSeg;
    if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
        eprintln!(
            "SQUARE_LANE_INPUT wrap={:?} tac={} first={:?} next={next:?}",
            table.common.text_wrap,
            table.common.treat_as_char,
            host.line_segs.first()
        );
    }
    if table.common.treat_as_char || !matches!(table.common.text_wrap, TextWrap::Square) {
        return false;
    }
    let (Some(host_first), Some(next)) = (
        host.line_segs
            .first()
            .filter(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0),
        next,
    ) else {
        return false;
    };
    let contains_successor = |line: &LineSeg| {
        line.segment_width > 0
            && next.vertical_pos >= line.vertical_pos
            && next.segment_width > 0
            && next.column_start >= line.column_start
            && i64::from(next.column_start) + i64::from(next.segment_width)
                <= i64::from(line.column_start) + i64::from(line.segment_width)
    };
    if (host_first.segment_width as i32) < next.full_width && contains_successor(host_first) {
        return true;
    }
    // 첫 줄이 전폭이어도 뒤 줄부터 개체 옆을 흐를 수 있다. 유효한 저장
    // 마지막 줄과 후속 줄이 같은 차선·연속 간격을 증명할 때만 수용한다.
    // 재조판·되감김·간격이 끊긴 입력을 저장 차선으로 추정하지 않는다.
    let Some(last) = host.line_segs.last() else {
        return false;
    };
    let continuous_lane = !host.stored_text_partition_is_dirty()
        && host.line_segs.iter().all(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0 && line.line_height > 0
        })
        && host.line_segs.windows(2).all(|pair| {
            pair[0]
                .vertical_pos
                .checked_add(pair[0].line_height)
                .and_then(|end| end.checked_add(pair[0].line_spacing))
                == Some(pair[1].vertical_pos)
        })
        && last
            .vertical_pos
            .checked_add(last.line_height)
            .and_then(|end| end.checked_add(last.line_spacing))
            == Some(next.vertical_pos)
        && host
            .line_segs
            .iter()
            .any(|line| i64::from(line.segment_width) > i64::from(last.segment_width))
        && contains_successor(last);
    if std::env::var("RHWP_TABLE_DRIFT").is_ok() {
        eprintln!(
            "SQUARE_LANE last={last:?} next={next:?} dirty={} continuous={continuous_lane}",
            host.stored_text_partition_is_dirty()
        );
    }
    continuous_lane
}
