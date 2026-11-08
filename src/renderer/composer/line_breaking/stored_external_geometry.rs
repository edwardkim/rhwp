//! 저장 줄의 좁음이 실제 외부 개체의 어울림에 속하는지 확인한다.

use super::{LayoutFrame, Paragraph, FLOAT_CARVE_MATCH_TOLERANCE_HU, UNIFORM_INSET_MIN_DELTA_HU};

/// 저장된 물리 행이 세로 구간마다 달라지는 외부 어울림 영역을 요구하는지 확인한다.
///
/// FIRST..LAST 조각이 여러 개인 행과 수평 범위가 달라지는 완결 행은 직접 증거다.
/// 고정 수평 범위 하나만 가진 프레임으로는 외부 개체의 진입·이탈을 재현할 수 없다.
pub(super) fn stored_rows_require_external_geometry(
    para: &Paragraph,
    frame: &LayoutFrame,
    float_carve_evidence: &[crate::renderer::float_placement::FloatCarveEvidence],
    known_square_band: bool,
) -> bool {
    let line_segs = &para.line_segs;
    let split_or_varying = line_segs
        .iter()
        .any(|segment| !segment.is_first_segment() || !segment.is_last_segment())
        || line_segs.windows(2).any(|pair| {
            pair[0].column_start != pair[1].column_start
                || pair[0].segment_width != pair[1].segment_width
        });
    if split_or_varying {
        return true;
    }

    // [#6175] 문단 전체가 개체 옆에 들어가면 폭 변화가 사라져 위 증거가
    // 소멸한다. 그때는 문서에 실재하는 어울림 개체가 증거다 — 저장 행이 남긴
    // 결손 폭을 그 개체의 흐름 폭이 설명하면, 좁음의 출처는 이 문단 자신이
    // 아니라 외부 기하다.
    //
    // ⚠ 이 판별자는 개체 폭과 **같은 세로 band**의 대조여야 한다. 균일하게 좁다는 것만으로는 문단
    // 테두리 박스의 inset 과 구별되지 않아 #547·#1440 핀이 깨진다(#6129 에서
    // 국소 판별자 2종이 그렇게 반증됐다). 셀에서는 #5818 이 같은 혼동을 "같은
    // 셀에 Square float 실재"로 이미 갈랐고, 이것은 그 계약의 본문 판이다.
    //
    // 156655489 1쪽 실측: 본문 폭 48188, 저장 cs=0·sw=26692 → 결손 21496.
    // 용지 기준 Square 그림 폭 21212 (offset 32361 → 프레임 좌표 26692) 로,
    // 저장 사다리의 끝이 개체 왼쪽 변과 단위까지 맞는다.
    if !float_carve_evidence.is_empty() && !line_segs.is_empty() {
        // 실제 어울림 그림은 왼쪽과 오른쪽 어느 쪽에도 있을 수 있다.
        // 저장 lane이 프레임 한 변을 공유하고 모든 행의 원점·폭이 같으면,
        // 사라진 폭은 원점 이동량이 아니라 전체 폭과 lane 폭의 차이다.
        // 같은 세로 band의 실제 개체가 그 차이를 설명할 때만 이 프레임의
        // 소유 밖으로 남긴다. 근거 없는 문단 들여쓰기·양쪽 inset은 제외한다.
        let first = &line_segs[0];
        let lane_end = first.column_start.checked_add(first.segment_width);
        let uniform_narrow = first.segment_width > 0
            && first.column_start >= frame.horizontal.start
            && lane_end.is_some_and(|end| {
                end <= frame.horizontal.end
                    && (first.column_start == frame.horizontal.start || end == frame.horizontal.end)
            })
            && line_segs.iter().all(|segment| {
                segment.column_start == first.column_start
                    && segment.segment_width == first.segment_width
            });
        if uniform_narrow {
            let missing =
                (frame.horizontal.end - frame.horizontal.start).saturating_sub(first.segment_width);
            if missing > FLOAT_CARVE_MATCH_TOLERANCE_HU
                && float_carve_evidence.iter().any(|evidence| {
                    evidence.matches_stored_rows(missing, line_segs, FLOAT_CARVE_MATCH_TOLERANCE_HU)
                })
            {
                return true;
            }
        }
    }

    // 빈 운반 행도 외부 어울림 경로가 정한 좁은 원점을 가질 수 있다.
    // 텍스트·컨트롤이 없으면 이 문단 안에는 그 원점을 계산할 기하가 없다.
    // 전폭으로 재조판하지 않고 기존 외부 경로가 배치를 담당하게 한다.
    let differs_from_frame = |segment: &crate::model::paragraph::LineSeg| {
        segment.column_start != frame.horizontal.start
            || segment.column_start.checked_add(segment.segment_width) != Some(frame.horizontal.end)
    };
    let empty_carrier = para.controls.is_empty()
        && para
            .text
            .chars()
            .all(|ch| ch.is_whitespace() || ch == '\r' || ch == '\n');
    if empty_carrier {
        return line_segs.iter().any(differs_from_frame);
    }

    // 균일한 inset은 일반 문단 들여쓰기일 수도 있어 외부 어울림의 증거가 아니다.
    // 실제 비인라인 Square 그림·도형의 앵커가 확인된 경우에만 보존한다.
    // #6175 GroupShape 경로는 이 플래그를 설정하기 전에 앵커와 레인 폭을
    // 확인한다. 일반 본문 프레임의 행은 재조판 대상이다.
    let inset_inside_frame = |segment: &crate::model::paragraph::LineSeg| {
        let end = match segment.column_start.checked_add(segment.segment_width) {
            Some(end) => end,
            None => return false,
        };
        segment.column_start >= frame.horizontal.start
            && end <= frame.horizontal.end
            && (frame.horizontal.end - frame.horizontal.start) - segment.segment_width
                > UNIFORM_INSET_MIN_DELTA_HU
    };
    known_square_band
        && line_segs.iter().any(differs_from_frame)
        && line_segs.iter().all(inset_inside_frame)
}
