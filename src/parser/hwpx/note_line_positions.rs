//! 각주·미주의 저장 줄 사다리 수용. 실제 되감김과 연속줄 아티팩트를 구분한다.
//! 미주의 양수→0 경계는 HWP 짝과 독립 한컴 Print로 확인한 내용 소속이다.
use super::{Paragraph, HWPX_HWP5_ORIGIN_SOURCE};

fn is_hwp5_stored_note_zero_vpos(paragraph: &Paragraph) -> bool {
    // HWP5 각주 하위 목록의 전 줄 0 원점 저장 패턴을 보존한다.
    paragraph.line_segs.len() > 1 && paragraph.line_segs.iter().all(|seg| seg.vertical_pos == 0)
}

pub(super) fn normalize_hwpx_note_line_vpos(paragraph: &mut Paragraph, preserve_all_zero: bool) {
    // [#4882] HWP5-origin HWPX는 note lineSeg 저장값 전체를 보존한다. marker가
    // 없는 HWP5 각주는 전 줄 0 원점 패턴을 보존한다. 일반 미주에서도 실제
    // 되감김은 보존하며, 전 줄 0인 연속줄 아티팩트만 기존처럼 복원한다.
    if HWPX_HWP5_ORIGIN_SOURCE.with(|c| c.get())
        || (preserve_all_zero && is_hwp5_stored_note_zero_vpos(paragraph))
    {
        return;
    }
    if paragraph.line_segs.len() <= 1 {
        return;
    }

    // 자기 쪽 원점에서 시작한 각주는 양수 위치 뒤 다시 시작할 수 있다
    // (예: 0/1172/0). 이는 각주 영역의 물리 페이지 경계다.
    // 첫 줄이 이미 양수에서 시작하는 후속0 연속줄 보정과 구분한다.
    // 반복 페이지 시작0/0/1172도 두 번째 줄을 다음 물리 쪽에 둔다.
    // 기존 미주 정규화와2344/0 연속줄 보정 계약은 유지한다.
    if preserve_all_zero
        && paragraph.line_segs[0].vertical_pos == 0
        && (paragraph.line_segs.windows(2).any(|lines| {
            lines[0].vertical_pos > 0
                && lines[1].vertical_pos == 0
                && lines.iter().all(|line| line.tag & 0x8000_0000 == 0)
        }) || (paragraph.line_segs[1].vertical_pos == 0
            && paragraph.line_segs[..2]
                .iter()
                .all(|line| line.tag & 0x8000_0000 == 0)))
    {
        return;
    }

    // 미주에서 양수 원점 뒤의 0은 실제 저장 단/쪽 되감김이다.
    // 모든 줄이 0인 연속줄 아티팩트와 구별하며 각주의 기존 계약은 유지한다.
    if !preserve_all_zero
        && !paragraph.stored_text_partition_is_dirty()
        && paragraph.line_segs.windows(2).any(|lines| {
            lines[0].vertical_pos > 0
                && lines[1].vertical_pos == 0
                && lines.iter().all(|line| line.tag & 0x8000_0000 == 0)
        })
    {
        return;
    }

    // 양수 원점들 사이의 기존 되감김도 보존한다. 빠진 0을 이전 줄
    // 전진으로 복원하면 원래의 컷을 한 줄 뒤로 옮길 수 있다.
    // 예: HWP 짝 65968/61499/63001, HWPX 저장 65968/0/63001.
    // 전 줄 0인 연속줄 아티팩트는 이 조건을 만족하지 않는다.
    let nonzero_rewinds = {
        let mut prev = None;
        paragraph.line_segs.iter().any(|seg| {
            if seg.vertical_pos <= 0 {
                return false;
            }
            let rewound = prev.is_some_and(|p| seg.vertical_pos < p);
            prev = Some(seg.vertical_pos);
            rewound
        })
    };
    if nonzero_rewinds {
        return;
    }

    let mut expected_vpos = None;
    for line_seg in &mut paragraph.line_segs {
        if let Some(expected) = expected_vpos {
            if line_seg.vertical_pos == 0 && expected > 0 {
                // HWPX 미주/각주 내부에는 실제 단/쪽 리셋이 아닌 후속 줄
                // vpos=0이 저장되는 사례가 있다. 본문 의미는 유지하고,
                // note 내부 연속줄만 이전 줄 advance 기준으로 복원한다.
                line_seg.vertical_pos = expected;
            }
        }

        expected_vpos = Some(
            line_seg
                .vertical_pos
                .saturating_add(line_seg.line_height)
                .saturating_add(line_seg.line_spacing),
        );
    }
}
