//! 문단 줄 스캔 후보의 경계 보정. 상태를 전진하거나 항목을 배치하지 않는다.

use super::super::{is_synthetic_line_seg, LADDER_FIT_EPSILON_PX};
use super::metrics::FormattedParagraph;
use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::renderer::hwpunit_to_px;

/// 끝 줄과 그 줄까지의 누적 advance를 같은 보정 결과로 반환한다.
/// 시작 줄은 호출자가 소유하며 end_line은 exclusive다.
pub(in crate::renderer::typeset) struct SplitBoundary {
    pub end_line: usize,
    pub cumulative: f64,
}

/// 기존 최소 진행 → 다음 표 앵커 → 저장 되감김 순서를 보존한다.
/// 가용 줄 예산과 각주 미차감 본문 높이는 서로 다른 축이므로 합치지 않는다.
pub(in crate::renderer::typeset) fn refine_split_boundary(
    para: &Paragraph,
    fmt: &FormattedParagraph,
    next_para: Option<&Paragraph>,
    cursor_line: usize,
    line_count: usize,
    avail_for_lines: f64,
    body_height: f64,
    hwp5_stored_pagination_layout: bool,
    dpi: f64,
    candidate: SplitBoundary,
) -> SplitBoundary {
    let SplitBoundary {
        mut end_line,
        mut cumulative,
    } = candidate;
    if end_line <= cursor_line {
        end_line = cursor_line + 1;
    }

    let next_para_is_rowbreak_anchor_table = next_para
        .map(|next_para| {
            next_para.controls.iter().any(|ctrl| {
                if let Control::Table(table) = ctrl {
                    !table.common.treat_as_char
                        && matches!(
                            table.common.text_wrap,
                            crate::model::shape::TextWrap::TopAndBottom
                        )
                        && matches!(
                            table.common.vert_rel_to,
                            crate::model::shape::VertRelTo::Para
                        )
                        && matches!(
                            table.page_break,
                            crate::model::table::TablePageBreak::RowBreak
                        )
                } else {
                    false
                }
            })
        })
        .unwrap_or(false);
    if cursor_line == 0
        && end_line > cursor_line + 1
        && end_line < line_count
        && next_para_is_rowbreak_anchor_table
        // HWP가 다음 source line을 새 physical page top(vpos=0)으로
        // 기록했으면, 그 직전 줄들은 현 페이지의 마지막 본문이다. 표를
        // 다음 페이지의 page-top에 보존하려고 한 줄을 되돌리면 이 tail까지
        // 불필요하게 이월돼 이후 표 owner가 한 줄씩 밀린다.
        && para
            .line_segs
            .get(end_line)
            .map(|next| next.vertical_pos != 0)
            .unwrap_or(true)
    {
        end_line -= 1;
        cumulative = fmt.line_advances_sum(cursor_line..end_line);
    }

    // [#6542] 문단 **안**에서 저장 `vertical_pos` 가 되감기면 그 줄부터 다음 쪽이다.
    //
    // 위 `next_para_is_rowbreak_anchor_table` 갈래가 "다음 줄의 저장 vpos 가 0(새 쪽
    // 상단)" 을 근거로 `end_line` 을 되돌리는 것과 같은 종류의 증거인데, 되감김
    // (vpos 가 앞줄보다 **작아짐**) 은 배선돼 있지 않았다. `dump-pages` 는 이미
    // `[vpos-rewind@lineN]` 으로 검출해 찍지만 그것은 진단 문자열일 뿐이다.
    //
    // 실측(156678235 pi=59, 물리 6쪽): 저장 사다리가 `68896 → 5040` 으로 line1 에서
    // 되감기는데 lines 0..3 을 한 쪽에 얹어 `used 1008.3px > 본문 933.6px` — 세 줄이
    // 본문 하한 1028.1 을 넘어(+3.7 / +37.3 / +70.9px) 쪽번호 아래에 그려졌다.
    // line0 만 남기면 907.5px 로 들어간다.
    //
    // 발동을 두 겹으로 좁힌다.
    //   ① **실제로 넘칠 때만** — 되감김이 있어도 예산 안에 들어가는 문단은 종전
    //      배분을 그대로 둔다(쪽 경계 판정은 #2098·#2138·#2279 로 눈금이 맞춰진
    //      지점들과 얽혀 있다).
    //   ② **쪽 규모 되감김만** — 문단 안의 작은 국소 리셋(들여쓰기·조각 재시작 산물)
    //      까지 쪽 경계로 읽으면 안 된다. 넓게 켠 판(①만)은 `#2070` 시장구조조사
    //      핀을 315 → 316쪽으로 깨뜨렸다(한글 기준 PDF 정답 315). 되감김 폭이 남은
    //      본문 높이의 절반 이상일 때만 "다음 쪽 상단으로 돌아갔다"로 읽는다.
    // 저장 사다리가 권위인 native HWP5 조판에 한정한다.
    //   ③ [#6718] **되감김 목표가 정확히 `0`** 인 문서는 위 ②만으로는 못 받는다.
    //      `vpos == 0` 은 "되감김"이 아니라 **새 물리 쪽 상단** 표식이라 종전에는
    //      "별도 기계가 다룬다"며 통째로 배제했는데, 그 별도 기계 둘이
    //      **네이티브 HWP5 + 각주 없음** 조합에서는 모두 꺼진다
    //      (`internal_vpos_page_break_line` 은 호출부가 네이티브 HWP5 를 목록에
    //      넣지 않고, `native_hwp5_existing_footnote_reset_overlap_break_line` 은
    //      각주가 없으면 즉시 반환). 27469 는 6개 쪽에서 본문 16줄이 하한을 넘고
    //      8쪽 마지막 줄은 용지 밖(+121.2px)으로 나간다.
    //
    //      그래서 `0` 도 받되 두 겹으로 좁힌다.
    //        · **조각이 쪽 하단 30% 안에서 시작했을 때만** —
    //          `internal_vpos_page_break_line` 이 쓰는 것과 같은 술어다.
    //        · **사다리가 "이 쪽은 꽉 찼다"고 말할 때만** — 사다리대로 끊은 뒤
    //          한 줄을 더 얹으면 예산을 넘어야 한다.
    //      #2070 시장구조조사(pi=343·1088: 54444→0, 53265→0)는 첫 겹은 통과하고
    //      **둘째 겹에서 걸린다**(예산이 각각 1.48·1.10줄 남는 자리에서 끊으라고
    //      한다). 그대로 따르면 315 → 316쪽이 된다.
    //
    //      ⚠ #3817 이 반대 방향의 함정을 남겼다 — 승격 기준을 "정확히 0" 으로만
    //      잡으면 `300~1500HU` 로 되감기는 문서를 놓친다. 그래서 `0` 을 별도
    //      갈래로 두지 않고 ②의 쪽-규모 낙폭 검사에 그대로 태운다(`cur == 0` 이면
    //      낙폭 = `prev.vertical_pos` 라 자연히 통과한다).
    let page_scale_rewind_hu = (crate::renderer::px_to_hwpunit(body_height, dpi) / 2).max(1);
    let fragment_starts_in_page_tail = para
        .line_segs
        .get(cursor_line)
        .filter(|ls| !is_synthetic_line_seg(ls))
        .map(|ls| ls.vertical_pos > 0 && hwpunit_to_px(ls.vertical_pos, dpi) >= body_height * 0.7)
        .unwrap_or(false);
    let ladder_page_is_full = |k: usize| -> bool {
        let upto = fmt.line_advances_sum(cursor_line..k);
        let next = fmt.line_advances_sum(k..k + 1);
        upto + next > avail_for_lines - LADDER_FIT_EPSILON_PX
    };
    // [#6718 잔여] `ladder_page_is_full` 이 걷어내는 자리 중 **옳은 것**을 되살린다.
    //
    // 위 시험 주석이 남긴 미해결 축이다 — `27469` 의 `pi=47`(7쪽, 예산 1.39줄
    // 남음)은 따라야 하는 자리인데, `#2070` 의 `pi=343`(1.48줄)·`pi=1088`(1.10줄)
    // 과 슬랙 값이 줄 단위로 뒤섞여 **슬랙·시작위치·낙폭·직전 항목·문단 수·태그**
    // 어느 축으로도 갈리지 않았다.
    //
    // 갈리는 축은 **이 승격이 쪽 경계를 새로 만드는가**다. 문단이 이 조각으로
    // 끝나지 않는다면(= 남은 줄이 어차피 다음 쪽으로 넘어간다면) 경계는 이미
    // 서 있고, 사다리는 그 경계를 **어디에 둘지**만 말한다. 그때는 예산에
    // 여유가 남아도 파일이 적어 둔 자리를 따르는 것이 옳다.
    //
    //   27469 pi=47   end_line 5 < line_count 8   → 이미 쪼개진다  → 따른다
    //   #2070 pi=343  end_line 5 = line_count 5   → 여기서 새로 끊는 셈 → 안 따른다
    //   #2070 pi=1088 end_line 5 = line_count 5   → 같음
    //
    // 실측: `27469` 7·8쪽 넘침 2줄과 8쪽 마지막 줄의 용지 밖 이탈(+121.2px)이
    // 닫히고, `#2070` 은 315쪽으로 불변이다.
    let paragraph_is_already_splitting = end_line < line_count;
    if hwp5_stored_pagination_layout
        && end_line > cursor_line + 1
        && cumulative > avail_for_lines - LADDER_FIT_EPSILON_PX
    {
        let rewind_line = (cursor_line + 1..end_line).find(|&k| {
            match (para.line_segs.get(k - 1), para.line_segs.get(k)) {
                (Some(prev), Some(cur)) => {
                    !is_synthetic_line_seg(prev)
                        && !is_synthetic_line_seg(cur)
                        && prev.vertical_pos > 0
                        && if cur.vertical_pos == 0 {
                            fragment_starts_in_page_tail
                                && (ladder_page_is_full(k) || paragraph_is_already_splitting)
                        } else {
                            cur.vertical_pos > 0
                        }
                        && cur.vertical_pos < prev.vertical_pos
                        && i64::from(prev.vertical_pos) - i64::from(cur.vertical_pos)
                            >= i64::from(page_scale_rewind_hu)
                }
                _ => false,
            }
        });
        if let Some(k) = rewind_line {
            end_line = k;
            cumulative = fmt.line_advances_sum(cursor_line..end_line);
        }
    }

    SplitBoundary {
        end_line,
        cumulative,
    }
}
