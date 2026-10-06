//! 저장 줄 사다리의 문단 간격 복원 가능성 판정.
//!
//! 문단과 해석된 스타일을 읽어 기존 판정만 반환한다. 실제 spacing 트림과
//! dirty 전환·페이지 전진은 호출자가 수행한다. 기존 경험적 조건의 타당성을
//! 이번 구조 이동으로 새로 승인하는 것은 아니다.

use crate::model::paragraph::Paragraph;
use crate::renderer::hwpunit_to_px;
use crate::renderer::style_resolver::ResolvedStyleSet;

/// [#5801] 저장 사다리가 이 문단의 **문단 위 간격을 실제로 담고 있는가**.
///
/// `#2279 ①` 의 spacing 트림은 "저장 ladder 가 spacing 을 이미 반영한다"는 전제 위에 선다.
/// 그 게이트(`has_authoritative_seg`)는 lineseg 가 합성인지만 본다. 그런데 **합성이 아닌데도
/// 문단 위 간격을 안 담은** 사다리가 있다 — `156677324` 는 문단 간 delta 와 문단 내 줄
/// advance 가 똑같이 2240 HU 인데, 한글은 그 자리에 50.23px(≈ 줄간격 29.86 + 문단간격 20.4)
/// 를 그린다(한글 PDF 실측, rhwp 렌더 49.87px 과 일치).
///
/// 그런 사다리에 트림을 적용하면 typeset 이 쪽 채움을 문단마다 `sb` 만큼 짧게 세어, 1쪽 끝에서
/// 99px 짜리 착시가 생긴다(트림 합계 109.9px). 쪽이 다 찼는데 남았다고 보고 다음 문단을
/// 현재 쪽에 얹는다(#5755).
///
/// 판별은 데이터 안에 있다 — 앞 문단 마지막 줄 아래에서 이 문단 첫 줄까지의 **저장 간격**이
/// `줄 간격 + 문단 위 간격` 을 담고 있으면 권위 사다리, 줄 간격뿐이면 아니다.
pub(in crate::renderer::typeset) fn stored_ladder_encodes_spacing_before(
    paragraphs: &[Paragraph],
    para_idx: usize,
    spacing_before_px: f64,
    dpi: f64,
) -> bool {
    if spacing_before_px <= 0.5 {
        return true; // 담을 간격이 없다 — 판별 대상 아님.
    }
    let Some(para) = paragraphs.get(para_idx) else {
        return true;
    };
    let Some(first) = para.line_segs.first() else {
        return true;
    };
    let Some(prev) = para_idx
        .checked_sub(1)
        .and_then(|idx| paragraphs.get(idx))
        .filter(|prev| prev.controls.is_empty())
    else {
        return true; // 앞이 없거나 컨트롤 문단 — 사다리 비교가 성립하지 않는다.
    };
    let Some(prev_last) = prev.line_segs.last() else {
        return true;
    };
    // 합성(vpos 전부 0) 사다리는 이 판별의 대상이 아니다 — 기존 게이트가 처리한다.
    if first.vertical_pos == 0 || prev_last.vertical_pos == 0 {
        return true;
    }
    let stored_gap = first
        .vertical_pos
        .saturating_sub(prev_last.vertical_pos.saturating_add(prev_last.line_height));
    if stored_gap <= 0 {
        return true; // 쪽·단 경계 되감김 — 판별 불가.
    }
    // 비교 기준은 **같은 사다리 안의 줄 간 실제 delta** 다. 저장 `line_spacing` 필드를 쓰면
    // 문단 경계에서 줄 간격을 흡수한 사다리를 오탐한다(2990099·3249937 에서 +3·+1쪽 회귀).
    let intra_gap = stored_intra_line_gap(para).or_else(|| stored_intra_line_gap(prev));
    let Some(intra_gap) = intra_gap else {
        // [#6031] 한 줄짜리 문단 연속 구간 — 줄 간 delta 표본이 없다. 직전 문단
        // 마지막 줄의 저장 trailing ls 와 경계 gap 이 **정확히 일치**하면 사다리가
        // 경계를 줄바꿈처럼 적은 것(sb 미인코딩)이다 (3249937 p6: gap 1400 == ls
        // 1400, sb 1000 미반영 → 쪽-말미 2줄이 본문 밖 +40pt). ls 필드를 1차
        // 기준으로 쓰면 경계 흡수형 사다리를 오탐하지만(아래 주석, 2990099·
        // 3249937 +3·+1쪽 회귀), 표본 부재 시의 등가-일치 판별은 흡수형
        // (gap < ls)과 정상형(gap ≥ ls+sb) 어느 쪽에도 걸리지 않는다.
        // sb 하한 5px: 아주 작은 sb(예: hwp3-sample16 계보 285HU=3.8px)는 한글이
        // 저장 흐름을 신뢰하는 문서군과 겹친다(#2158 핀 64쪽 실측 — 누락 판정 시
        // 트림 철회 누적 +3.8px×282 경계로 65쪽 회귀). 여백 관통을 만드는 굵은
        // sb(6.7px+)만 누락 판정한다.
        // 추가 지문: 이 생성기 계열은 줄 피치를 lh·ls 에 양분해 적는다
        // (ls == lh, paraPr 160% 와도 모순 — 3249937 전 문단 1400/1400).
        // 정상 저장 사다리(ls < lh, hwp3-sample16 660/2000)는 한글이 저장
        // 흐름을 신뢰하는 문서군과 겹치므로 등가-일치만으로 누락 판정하지 않는다.
        if spacing_before_px > 5.0
            && prev_last.line_spacing == prev_last.line_height
            && prev_last.line_spacing > 0
            && (stored_gap - prev_last.line_spacing).abs() <= 2
            && hwpunit_to_px(prev_last.line_spacing, dpi) + spacing_before_px
                > hwpunit_to_px(stored_gap, dpi) + 0.5
        {
            return false;
        }
        return true; // 판별 불가 — 종전 보수 유지.
    };
    // 문단 경계 간격이 줄 간격과 같으면 사다리가 경계를 줄바꿈처럼 적은 것이다.
    hwpunit_to_px(stored_gap, dpi) + 0.5 >= hwpunit_to_px(intra_gap, dpi) + spacing_before_px
}

/// [#5801] 같은 문단 안에서 저장 사다리가 적은 줄 사이 실제 간격(HWPUNIT).
fn stored_intra_line_gap(para: &Paragraph) -> Option<i32> {
    para.line_segs.windows(2).find_map(|w| {
        let gap = w[1]
            .vertical_pos
            .saturating_sub(w[0].vertical_pos.saturating_add(w[0].line_height));
        (gap >= 0 && w[0].vertical_pos != 0).then_some(gap)
    })
}

/// [#2279 ①-3] spacing 트림의 복원 가능성 전방 판정.
///
/// 트림은 다음 authoritative(비합성 lineseg) anchor 에서 vpos-snap 이 좌표를
/// 복원한다는 전제다. 현 문단과 다음 anchor 사이에 합성/NO_LS 텍스트 문단이
/// 끼어 있으면 dirty 규칙(#2243 전방-스냅만)으로 복원이 차단되므로 트림하면
/// 그 spacing 이 흐름에서 소실된다 (36398700 pi6: 다음 anchor pi10 앞에
/// 합성 pi7~9 → −35.7px 소실 실측). 표 컨트롤 문단은 재앵커(#2243) 지점이라
/// 복원 가능으로 본다. 탐색은 32문단 한도(초과 시 보수적으로 복원 불가).
pub(in crate::renderer::typeset) fn spacing_trim_restorable(
    paragraphs: &[Paragraph],
    para_idx: usize,
) -> bool {
    use crate::model::paragraph::LineSeg;
    for para in paragraphs.iter().skip(para_idx + 1).take(32) {
        if !para.controls.is_empty() {
            return true; // 표/개체 재앵커 지점
        }
        match para.line_segs.first() {
            Some(seg) if seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0 => {
                return true; // authoritative anchor 도달 — 복원 가능
            }
            _ => {
                if !para.text.is_empty() {
                    return false; // 합성/NO_LS 텍스트 문단이 먼저 — 복원 불가
                }
                // 빈 문단은 계속 탐색
            }
        }
    }
    false
}

/// [#7196] 다음 문단 경계가 `#6031` 철회(dirty)에 걸려 트림 복원이 막히는가.
///
/// `#2279 ①` 트림은 다음 저장 anchor 의 vpos-snap 이 좌표를 되돌린다는 전제인데,
/// HWPX 저장 레이아웃에서 다음 문단이 굵은 문단 위 간격(> 5px)을 가졌고 그 경계의 저장
/// 사다리가 그 간격을 담지 않았으면(`stored_ladder_encodes_spacing_before` 거짓) `#6031` 이
/// 그 스냅을 되돌리고 사다리를 dirty 로 만든다. 그러면 이 문단에서 깎은 문단 위 간격과
/// 끝 줄 간격이 영영 복원되지 않아 조판이 렌더보다 짧게 센다 — 156760012 10쪽 첫 문단
/// pi=66: 트림 52.3px(sb 26.7 + ls 25.6) 미복원, 쪽 말미 표가 본문 바닥 +18.3px 넘침.
/// 판정식·게이트(`hwpx_stored_layout`, `!hwp3_layout`, sb > 5px)는 `#6031` 과 같게 둔다.
pub(in crate::renderer::typeset) fn next_boundary_reverts_spacing_trim(
    hwpx_stored_non_hwp3: bool,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    para_idx: usize,
    dpi: f64,
) -> bool {
    if !hwpx_stored_non_hwp3 {
        return false;
    }
    let next_idx = para_idx + 1;
    let Some(next) = paragraphs.get(next_idx) else {
        return false;
    };
    // `format_paragraph` 의 spacing_before 산출과 같은 축 — 저장 줄이 없는 텍스트 문단은 0.
    let spacing_before_px = if next.line_segs.is_empty() && !next.text.is_empty() {
        0.0
    } else {
        styles
            .para_styles
            .get(next.para_shape_id as usize)
            .map(|style| style.spacing_before)
            .unwrap_or(0.0)
    };
    spacing_before_px > 5.0
        && !stored_ladder_encodes_spacing_before(paragraphs, next_idx, spacing_before_px, dpi)
}
