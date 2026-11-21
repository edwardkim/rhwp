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
    pub(crate) line_height: i32,
    /// 첫 줄과 같은 `vertical_pos` 를 공유하는 저장 조각(개체 양옆 차선)의 가로 범위.
    /// 조각이 `FIRST_ROW_CAPACITY` 를 넘으면 `None` 이다.
    first_row: Option<([(i32, i32); FIRST_ROW_CAPACITY], usize)>,
}

const FIRST_ROW_CAPACITY: usize = 4;

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
    let mut row = [(0, 0); FIRST_ROW_CAPACITY];
    let mut row_len = 0;
    let mut row_fits = true;
    for seg in para
        .line_segs
        .iter()
        .take_while(|seg| original(seg) && seg.vertical_pos == first.vertical_pos)
    {
        match row.get_mut(row_len) {
            Some(slot) => *slot = (seg.column_start, seg.segment_width as i32),
            None => row_fits = false,
        }
        row_len += 1;
    }
    Some(StoredLineLaneProbe {
        vertical_pos: first.vertical_pos,
        column_start: first.column_start,
        segment_width: first.segment_width as i32,
        full_width,
        line_height: first.line_height,
        first_row: row_fits.then_some((row, row_len)),
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
    if flanking_successor_host_advance_hu(host, Some(next), table).is_some() {
        return true;
    }
    let contains_successor = |line: &LineSeg| {
        line.segment_width > 0
            && next.vertical_pos >= line.vertical_pos
            && next.segment_width > 0
            && next.column_start >= line.column_start
            && i64::from(next.column_start) + i64::from(next.segment_width)
                <= i64::from(line.column_start) + i64::from(line.segment_width)
    };
    if ((host_first.segment_width as i32) < next.full_width
        || host_line_clears_table_box(host_first, table))
        && contains_successor(host_first)
    {
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

/// 글자 없는 host 의 저장 줄과 같은 문단 기준 어울림 표 바깥 상자(단·문단 좌표, HWPUNIT).
struct StoredSquareTableBox {
    left: i64,
    right: i64,
    top: i64,
    bottom: i64,
    /// host 저장 줄의 끝 + 줄간격. 저장 사다리가 host 줄만 전진했다면 다음 줄의 시작이다.
    host_end: i64,
    host_advance: i32,
}

impl StoredSquareTableBox {
    fn resolve(host: &Paragraph, table: &Table) -> Option<Self> {
        use crate::model::paragraph::LineSeg;
        use crate::model::shape::{HorzAlign, HorzRelTo, VertAlign, VertRelTo};
        use crate::renderer::float_placement::signed_hwpunit;
        let host_first = host.line_segs.first()?;
        if table.common.treat_as_char
            || !matches!(table.common.text_wrap, TextWrap::Square)
            || table.common.vert_rel_to != VertRelTo::Para
            || table.common.vert_align != VertAlign::Top
            || table.common.horz_align != HorzAlign::Left
            || !matches!(
                table.common.horz_rel_to,
                HorzRelTo::Para | HorzRelTo::Column
            )
            || host
                .text
                .chars()
                .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}')
            || host.stored_text_partition_is_dirty()
            || host.line_segs.iter().any(|seg| {
                seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                    || seg.vertical_pos != host_first.vertical_pos
            })
            || host_first.line_height <= 0
            || host_first.line_spacing < 0
        {
            return None;
        }
        let host_top = i64::from(host_first.vertical_pos);
        let host_advance = host_first
            .line_height
            .checked_add(host_first.line_spacing)?;
        let left = i64::from(signed_hwpunit(table.common.horizontal_offset));
        let right = left
            + i64::from(table.outer_margin_left)
            + i64::from(table.common.width)
            + i64::from(table.outer_margin_right);
        let top = host_top + i64::from(signed_hwpunit(table.common.vertical_offset));
        let bottom = top
            + i64::from(table.outer_margin_top)
            + i64::from(table.common.height)
            + i64::from(table.outer_margin_bottom);
        Some(Self {
            left,
            right,
            top,
            bottom,
            host_end: host_top + i64::from(host_advance),
            host_advance,
        })
    }

    /// 저장 조각 하나가 상자와 세로로 겹치면서 가로로는 비켜 있는가.
    fn flanked_by(&self, vertical_pos: i32, line_height: i32, start: i32, width: i32) -> bool {
        let line_top = i64::from(vertical_pos);
        let (start, end) = (i64::from(start), i64::from(start) + i64::from(width));
        width > 0
            && line_height > 0
            && line_top >= self.host_end
            && line_top < self.bottom
            && line_top + i64::from(line_height) > self.top
            && (end <= self.left || start >= self.right)
    }
}

/// 글자 없는 host 의 저장 줄 바로 다음에 이어진 뒤 문단 첫 줄이, 표 바깥 상자와 세로로
/// 겹치면서 그 줄의 저장 조각이 상자 왼쪽·오른쪽에 나뉘어 있으면 host 줄의 전진량을 준다.
///
/// 한/글은 어울림 표 양옆에 글 자리가 있으면 한 줄을 같은 `vertical_pos` 의 조각 여럿으로
/// 저장한다(왼쪽 차선 `cs=0, sw=가로 오프셋`, 오른쪽 차선 `cs=상자 오른쪽 끝`). host 줄은
/// 앞 표의 띠에 걸려 다른 차선 폭을 가질 수 있으므로 host 줄의 차선과 견주지 않고, 뒤 줄의
/// 조각과 이 표의 상자를 직접 견준다. 저장 사다리가 host 줄만큼만 전진했다는 것(다음 줄
/// 시작 = host 줄 끝 + 줄간격)이 표 높이를 흐름에 싣지 않았다는 증거다. 이때 뒤 문단은
/// 표 옆 띠에 흡수되는 줄이 아니라 자기 높이를 소비하는 본문 줄이다.
pub(crate) fn flanking_successor_host_advance_hu(
    host: &Paragraph,
    next: Option<StoredLineLaneProbe>,
    table: &Table,
) -> Option<i32> {
    let next = next?;
    let (row, row_len) = next.first_row?;
    let frame = StoredSquareTableBox::resolve(host, table)?;
    let row = &row[..row_len];
    // 한 줄이 상자 왼쪽·오른쪽 조각으로 나뉜 경우만 다룬다. 한쪽 차선만 쓰는 줄은 띠 한 칸
    // (`cs/sw`)으로 표현되어 기존 차선 흡수 경로가 저장 좌표로 놓는다.
    let on_left = row
        .iter()
        .any(|&(start, width)| i64::from(start) + i64::from(width) <= frame.left);
    let on_right = row
        .iter()
        .any(|&(start, _)| i64::from(start) >= frame.right);
    (on_left
        && on_right
        && i64::from(next.vertical_pos) == frame.host_end
        && row.iter().all(|&(start, width)| {
            frame.flanked_by(next.vertical_pos, next.line_height, start, width)
        }))
    .then_some(frame.host_advance)
}

/// 뒤 문단의 모든 원본 저장 조각이 host 의 어울림 표 상자 옆(세로로 겹치고 가로로 비킴)에
/// 놓였는가. 표 옆을 흐르는 본문 문단은 띠를 닫지 않고 자기 높이만 소비한다.
pub(crate) fn stored_rows_flank_square_table(host: &Paragraph, para: &Paragraph) -> bool {
    use crate::model::control::Control;
    use crate::model::paragraph::LineSeg;
    let mut tables = host.controls.iter().filter_map(|control| match control {
        Control::Table(table) => Some(&**table),
        _ => None,
    });
    let (Some(table), None) = (tables.next(), tables.next()) else {
        return false;
    };
    let Some(frame) = StoredSquareTableBox::resolve(host, table) else {
        return false;
    };
    !para.line_segs.is_empty()
        && !para.stored_text_partition_is_dirty()
        && para.line_segs.iter().all(|seg| {
            seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && frame.flanked_by(
                    seg.vertical_pos,
                    seg.line_height,
                    seg.column_start,
                    seg.segment_width as i32,
                )
        })
}

/// host 저장 첫 줄이 같은 문단 기준 어울림 표의 바깥 상자와 세로로 겹치면서 가로로는
/// 비켜 있는가. 그렇다면 그 줄 자체가 표 옆 차선이다.
///
/// 다음 문단도 차선 폭의 줄만 가지면 "다음 문단의 가장 넓은 줄"로는 전폭을 알 수
/// 없다(36458608 pi=8: host·다음 문단 모두 cs=0 sw=31014, 표 가로 오프셋 31014).
/// 표 상자는 가로 오프셋에서 시작해 좌우 바깥 여백과 폭을 더한 곳에서 끝난다
/// (`stored_empty_square_successor_strip` 과 같은 단·문단 좌표).
fn host_line_clears_table_box(line: &crate::model::paragraph::LineSeg, table: &Table) -> bool {
    use crate::model::shape::{HorzRelTo, VertRelTo};
    use crate::renderer::float_placement::signed_hwpunit;
    if table.common.vert_rel_to != VertRelTo::Para
        || !matches!(
            table.common.horz_rel_to,
            HorzRelTo::Para | HorzRelTo::Column
        )
        || line.segment_width <= 0
        || line.line_height <= 0
    {
        return false;
    }
    let left = i64::from(signed_hwpunit(table.common.horizontal_offset));
    let right = left
        + i64::from(table.outer_margin_left)
        + i64::from(table.common.width)
        + i64::from(table.outer_margin_right);
    let line_left = i64::from(line.column_start);
    let line_right = line_left + i64::from(line.segment_width);
    let beside = line_right <= left || line_left >= right;
    // 표 위끝은 host 줄 원점 + 세로 오프셋, 아래끝은 바깥 여백까지다.
    let top = i64::from(signed_hwpunit(table.common.vertical_offset));
    let bottom = top
        + i64::from(table.outer_margin_top)
        + i64::from(table.common.height)
        + i64::from(table.outer_margin_bottom);
    let overlaps_vertically = top < i64::from(line.line_height) && bottom > 0;
    beside && overlaps_vertically
}
