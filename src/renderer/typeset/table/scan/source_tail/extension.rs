//! 저장 프레임 후보의 경계 보정·확장 적합성 Query.
//! 페이지/스캔 상태를 쓰지 않고 보정 제안과 판정을 반환한다. 후보 반영·수용은 부모 소유다.

use super::SourceTailQuery;
use crate::renderer::layout::table_layout::RowCutResult;
use crate::renderer::typeset::table::scan::{row::RowScanQuery, RowBlockQuery};

pub(in crate::renderer::typeset) struct SourceTailCorrection {
    pub(in crate::renderer::typeset) end_cut: Vec<usize>,
    pub(in crate::renderer::typeset) consumed_height: f64,
}

pub(in crate::renderer::typeset) struct SourceTailFit {
    pub(in crate::renderer::typeset) extension: f64,
    pub(in crate::renderer::typeset) mid_extension_ok: bool,
    pub(in crate::renderer::typeset) source_tail_owns_this_page: bool,
}

impl SourceTailQuery<'_> {
    /// 가시 셀의 다음 경계가 일치할 때만 기존 컷을 한 번 복제해 보정안을 만든다.
    pub(in crate::renderer::typeset) fn mirrored_correction(
        &self,
        res: &RowCutResult,
        source_tail_cut: &RowCutResult,
        padding: f64,
    ) -> Option<SourceTailCorrection> {
        let Self {
            row, row_start_cut, ..
        } = *self;
        let RowScanQuery { rows, r, .. } = *row;
        let RowBlockQuery {
            layout_engine,
            table,
            styles,
            ..
        } = *rows;
        // [#6973] 파생 문단 꼬리 확장은 **저장된 물리 쪽 경계**를 넘지 않는다.
        //
        // 저장 `lineseg` 의 vpos 되감김(양수 → 0)은 한/글이 그 행 안에서 쪽을
        // 끊은 자리다. 83818 행 13 은 두 셀 모두 12줄이고 `li=9` 에서 되감긴다 —
        // 앞 9줄이 8쪽, 뒤 3줄이 9쪽이다. 그런데 `stored_source_frame` 은
        //   ① `row_has_single_visible_source_cell`  가시 셀 정확히 1개
        //   ② `direct_hwpx_cell_has_declared_stored_frame`  프레임 span ≤ 선언 cellSz
        // 를 함께 요구한다. 신·구조문대비표는 두 열이 다 글자를 가져 ①이 거짓이고,
        // 이 문서는 선언 높이가 **전 행 2416HU(한 줄 규모)** 로 유지되지 않아 행 13 의
        // span `21560 + 6440 = 28000HU` 가 ②를 구조적으로 통과할 수 없다. 그래서
        // 꼬리가 상한 없는 파생 경로(`paragraph_tail_cut_for_row`)로 떨어져 문단
        // 끝(12줄)까지 당기고, 잔여 299.8px 에 388.3px 를 실어 본문을 108.9px
        // 넘기면서 쪽 하나를 잃는다(한/글 2020 9쪽 · rhwp 8쪽).
        //
        // `paragraph_tail_cut_for_row` 는 이미 `stored_frame_break_before` 유닛에서
        // 멈추려 한다. 그 표지가 위 관문에 걸려 서지 않을 때 원시 되감김 인덱스로
        // 같은 의도를 세운다.
        //
        // ⭐⭐ 적용 조건은 **가시 셀이 둘 이상이고 전부 같은 줄 인덱스에서 되감기는
        // 것**이다. 독립된 두 셀이 같은 자리를 적었다는 것이 writer-local 커서
        // (②가 걸러내던 것)와 갈리는 증거이며, ①의 **정확한 여집합**이라 가시 셀
        // 1개인 기존 계약(#3930·#3931·#5584·#5801·#6025·#6549·#6790)은 이 분기에
        // 들어오지 않는다.
        //
        // ⚠ 크기·비율 축으로는 갈리지 않는다 — 편람 r=4(정상)의 잔여 초과 +81.8px 와
        // 83818 r=13(결함)의 +88.5px 는 6.7px 차이다. 그래서 잔여 기준 수용 판정
        // (지정 시험 10/54 실패)·이월 분기(12/54 실패)·되감김 무조건 클립(편람
        // 384→385)이 모두 깨졌다. 갈리는 것은 **저장 증거의 일치** 하나다.
        // 저장 되감김을 **CellUnit 경계로 투영해** 받는다 — 저장 `LineSeg`
        // 번호와 `end_cut` 은 같은 축이 아니다(좌우 분할 줄 병합·중첩 표 전개,
        // PR #6996 검토 지적). 투영이 성립하지 않는 줄은 목록에 아예 없다.
        let stored_rewinds = layout_engine.row_stored_rewind_unit_indices(table, r, styles);
        let visible = layout_engine.row_visible_source_cell_flags(table, r, styles);
        let visible_indices: Vec<usize> = visible
            .iter()
            .enumerate()
            .filter(|(_, shown)| **shown)
            .map(|(idx, _)| idx)
            .collect();
        // 이 컷 안에서 **현재 위치 이후 첫 경계**를 셀마다 고른다. 첫 되감김만
        // 보면 컷이 그 자리를 이미 지난 뒤의 경계를 놓친다(같은 검토 지적).
        let boundary_for = |idx: usize| -> Option<usize> {
            let pre = res.end_cut.get(idx).copied().unwrap_or(0);
            let end = source_tail_cut.end_cut.get(idx).copied()?;
            stored_rewinds
                .get(idx)?
                .iter()
                .copied()
                .find(|rewind| pre <= *rewind && *rewind < end)
        };
        // 독립된 두 셀 이상이 **같은 unit 경계**를 적었을 때만 물리 경계로 본다.
        let mirrored_boundary: Option<usize> = if visible_indices.len() >= 2 {
            let first = boundary_for(visible_indices[0]);
            match first {
                Some(b)
                    if visible_indices
                        .iter()
                        .all(|idx| boundary_for(*idx) == Some(b)) =>
                {
                    Some(b)
                }
                _ => None,
            }
        } else {
            None
        };
        if let Some(boundary) = mirrored_boundary {
            let mut clipped = source_tail_cut.end_cut.clone();
            let mut clipped_any = false;
            for idx in &visible_indices {
                if let Some(end) = clipped.get(*idx).copied() {
                    if boundary < end {
                        clipped[*idx] = boundary;
                        clipped_any = true;
                    }
                }
            }
            if clipped_any {
                let clipped_total = layout_engine.row_cut_content_height(
                    table,
                    r,
                    row_start_cut,
                    &clipped,
                    styles,
                    false,
                );
                return Some(SourceTailCorrection {
                    end_cut: clipped,
                    consumed_height: (clipped_total - padding).max(0.0),
                });
            }
        }

        None
    }

    /// 보정이 반영된 후보를 받는다. avail_for_rows를 잔여 예산으로 재해석하지 않는다.
    pub(in crate::renderer::typeset) fn extension_fit(
        &self,
        res: &RowCutResult,
        source_tail_cut: &RowCutResult,
        mid_frame_only: bool,
        avail_for_rows: f64,
    ) -> SourceTailFit {
        let Self { row, .. } = *self;
        let RowScanQuery { rows, r, .. } = *row;
        let RowBlockQuery {
            layout_engine,
            table,
            styles,
            ..
        } = *rows;
        // [#5584 ②] 중간 행 갈래는 near-miss(행 대부분을 담고 마지막
        // 한 유닛 규모만 부족)에 한정한다 — ① 확장 ≤24px(한 유닛 규모)
        // ② 확장 전 소비가 확장의 3배 이상(행을 거의 다 담은 상태).
        // ② 가 없으면 budget 이 0 에 가까운 행(그 행이 통째로 다음 쪽감)
        // 의 첫 유닛까지 강제로 당겨 382쪽 편람 계약(#4763, issue_3931/
        // 3930/5801 핀)이 381 로 무너진다.
        let extension = source_tail_cut.consumed_height - res.consumed_height;
        // 확장 안 하면 다음 조각이 "한 유닛 + 프레임 리셋 + 소량 꼬리"
        // sliver 가 되는 형상만 — 프레임 경계 뒤 같은 행의 잔여가 소량
        // (0.5, 64px] 이어야 그 sliver 형상이다. 잔여가 크면(382쪽 편람
        // r=5 실측: near-miss 시그니처는 동형이나 잔여 949.9px = 다음
        // 조각의 본체) 확장이 오히려 쪽 경계를 옮긴다(#4763 핀 —
        // issue_3931/3930/5801 이 381 로 무너짐). 3232693 은 잔여 38.4px.
        // [#6549] 어울림(Square) 자리차지 표도 같은 상한을 쓴다.
        //
        // 이 확장 계약(#5584 ②/#4763)은 **위아래 배치(TopAndBottom)** 표가
        // 쪽을 넘기는 형상에서 검증됐다 — 382쪽 편람 핀
        // (issue_3931/3930/5801)이 모두 `wrap=TopAndBottom` 이다. 어울림 표는
        // 옆으로 글이 흐르므로 프레임 회계가 달라, 상한 없는 확장이 그대로
        // 쪽 넘침이 된다.
        //
        // 실측 (원자력안전위 16418295, 어울림 RowBreak 표 r=6):
        //   budget 75.8 → 92.8 (확장 25.6) → 행이 통째로 수용돼 표가
        //   안 쪼개지고 본문 하한을 17.1px 넘는다. 한글은 2쪽, rhwp 1쪽.
        //   확장을 막으면 2쪽이 되어 한글과 맞는다.
        //
        // 편람 핀들의 확장은 15.3~107.4px 로 이 값(25.6)을 사이에 두고
        // 흩어져 있어 `extension`·`consumed` 비·`frame_tail_rest`·예산 초과율
        // 어느 축으로도 갈리지 않는다. 갈리는 것은 **배치 종류** 하나다.
        let bounded_extension_branch =
            mid_frame_only || table.common.text_wrap == crate::model::shape::TextWrap::Square;
        let frame_tail_rest = if bounded_extension_branch {
            layout_engine.row_cut_content_height(
                table,
                r,
                &source_tail_cut.end_cut,
                &[],
                styles,
                false,
            )
        } else {
            0.0
        };
        let mid_extension_ok = !bounded_extension_branch
            || (extension <= 24.0
                && res.consumed_height >= 3.0 * extension
                && frame_tail_rest > 0.5
                && frame_tail_rest <= 64.0);
        // [#6790] 확장된 저장 프레임 컷이 **이 조각의 행 예산 자체**를
        // 넘으면, 그 프레임은 이 쪽의 끊는 자리 증거가 될 수 없다.
        //
        // `#5584 ②`/`#4763` 의 확장은 "한글이 여기서 끊었다"는 저장 증거를
        // 따라 **거의 다 담은 행을 마저 담는** 조작이다. 그런데 확장된 컷
        // 하나가 이 조각이 행에 쓸 수 있는 전부보다 크면, 그 컷은 혼자서도
        // 이 쪽에 못 들어간다 — 그 프레임은 어차피 쪽을 넘기므로 이 쪽의
        // 경계를 정할 자격이 없다. 크기가 아니라 **쪽 소유(page ownership)**
        // 판정이다.
        //
        // 실측 (r = 확장이 일어난 행):
        //
        // ```text
        //   문서                                 tail    avail   소유
        //   17544911 (누에 사육기준) r=2        1178.5  1005.4   ✗ 못 넘김
        //   편람 r=1                              232.3   240.7   ✔
        //   편람 r=4 (여섯 갈래)             82.9~386.9  108.1~458.2 ✔ 전부
        //   3232693 (#5584/#6025) r=7            162.7   906.6   ✔
        //   16418295 (#6549) r=6                  92.8  1009.1   ✔
        // ```
        //
        // ⚠ 크기·비율로 가르지 않는다 — `#6549` 가 기록했듯 편람 핀들의
        // 확장(15.3~107.4px)과 예산 초과율(17.8~112.0px)은 어느 축으로도
        // 갈리지 않는다. 위 표에서 갈리는 것은 **부호 하나**이며 문턱이 없다.
        //
        // ⚠ 초판에는 `|| consumed > avail_for_rows + 0.5` 우회가 있었다
        // ("이미 예산을 넘긴 조각은 `#5057` 이 따로 판정한다"). **제거했다** —
        // 그 우회는 여기서 세운 쪽 수용 불변식을 무효화할 수 있고,
        // `#5057` 두 시험은 이 `source_frame_tail` 갈래를 실제로 실행하지
        // 않는다(PR #6792 검토 실측). 우회 없이 #3930·#3931·#5057·#5584·
        // #5801·#6025·#6549·#6790 선택 시험 19/19 가 통과한다.
        let source_tail_owns_this_page = source_tail_cut.consumed_height <= avail_for_rows + 0.5;

        SourceTailFit {
            extension,
            mid_extension_ok,
            source_tail_owns_this_page,
        }
    }
}
