//! 표 포맷을 위한 측정 결과 수용 Query.
//! 기존 높이 보정 정책을 유지하며 host 간격·각주 예약·분할/배치는 호출자가 소유한다.

use super::para_has_non_whitespace_text;
use crate::model::{paragraph::Paragraph, provenance::LayoutCompatibilityProfile, table::Table};
use crate::renderer::float_placement::is_para_topbottom_float;
use crate::renderer::height_measurer::{
    fit_measured_table_declared_tail_to_declared_height,
    fit_measured_table_nested_tail_to_declared_height, fit_measured_table_to_declared_height,
    MeasuredTable,
};

/// 기존 측정값에 대한 보정 후보를 반환한다. 원본 fallback과 결과 수명은 호출자가 관리한다.
/// profile 관측은 기존 guard의 단락 평가 위치를 보존한다.
pub(super) fn fit_measured_for_host(
    para: &Paragraph,
    table: &Table,
    mt: Option<&MeasuredTable>,
    dpi: f64,
    profile: impl Fn() -> LayoutCompatibilityProfile,
) -> Option<MeasuredTable> {
    // [#2195] 빈 앵커(자리차지 표 표준형)에도 선언높이 fit 적용 — 한글은 콘텐츠가
    // 선언보다 작아도 표 선언높이를 유지한다 (80168 pi=419 행 걷기 151.1 = 선언,
    // 콘텐츠 143.5 사용 시 페이지 끝 razor -1쪽). fit 자체의 0.75~1.35 가드(#1510)
    // 가 과대 압축/팽창을 차단한다.
    if is_para_topbottom_float(&table.common) {
        mt.map(|measured| {
            let fitted = fit_measured_table_to_declared_height(measured, table, dpi);
            // 빈 앵커는 **확대 방향만**: 한글 규칙 = max(선언, 콘텐츠) — 콘텐츠가
            // 선언보다 큰 표(pi=15 조문대비표 929.6>928.4)를 압축하면 분할 경계가
            // 당겨져 pi16 -1쪽. 압축(fit-down)은 종전대로 비공백 텍스트 앵커 한정.
            let shrunk = fitted.row_heights.iter().sum::<f64>()
                < measured.row_heights.iter().sum::<f64>() - 0.01;
            // 텍스트 앵커의 fit-down(선언 높이 압축)은 저장 시점 형상 전용
            // 보정이다 — 편집 세션(session_edited)이나 실제 성장 행(중첩 표
            // 없는 텍스트 행이 선언 행높이를 1.5배 초과) 판정 시 압축하지
            // 않는다. 압축하면 커진 행의 몫을 다른 행이 빼앗겨 내부가 위로
            // 밀리고 fit 판정이 과소해져 RowBreak 분할이 시작되지 않는다
            // (셀 Enter 재현: 한글은 행을 키우고 넘친 부분을 다음 쪽으로
            // 분할). 빈 host 는 아래 분기(측정 복원·tail-fit)가 종전대로
            // 처리한다.
            if shrunk
                && para_has_non_whitespace_text(para)
                && (profile().session_edited()
                    || crate::renderer::height_measurer::measured_table_has_grown_text_row(
                        measured, table, dpi,
                    ))
            {
                return measured.clone();
            }
            if shrunk && !para_has_non_whitespace_text(para) {
                // HWP5 빈 TopAndBottom host의 다행 RowBreak 표는 통상 콘텐츠가
                // 선언높이를 넘으면 축소하지 않는다. 다만 마지막 행 하나가 비-TAC
                // 1×1 자식 표이고, 그 parent viewport의 Center 정렬이 만든 작은
                // over-measure인 경우에는 한컴 PDF가 앞 행 경계는 보존한 채 마지막
                // 행만 선언 총높이에 맞춘다 (76076 p81→82). 전체 비율 축소는 정상
                // 헤더/짧은 행까지 줄이므로 금지하고, helper가 마지막 행만 줄일 수
                // 있는 구조·64px 이내 drift를 다시 확인한다.
                let native_empty_rowbreak_nested_tail = profile().hwp5_stored_pagination_layout()
                    && !table.common.treat_as_char
                    && matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    )
                    && table.row_count > 1
                    && table.cells.iter().all(|cell| cell.row_span == 1);
                // [#5906] 위 helper 가 못 잡는 형상이라도, 마지막 행이 저장
                // 선언(cellSz)으로만 잡혀 여유가 남아 있으면 그 행에서만 초과분을
                // 회수한다. 페인트 경로가 이미 반대 방향(부족분 → 마지막 행)으로
                // 하는 일과 같다 (float-stack-defer 2쪽 표 3쪽 분열).
                let native_empty_rowbreak = profile().hwp5_stored_pagination_layout()
                    && !table.common.treat_as_char
                    && matches!(
                        table.page_break,
                        crate::model::table::TablePageBreak::RowBreak
                    )
                    && table.row_count > 1;
                native_empty_rowbreak_nested_tail
                    .then(|| {
                        fit_measured_table_nested_tail_to_declared_height(measured, table, dpi)
                    })
                    .flatten()
                    .or_else(|| {
                        native_empty_rowbreak
                            .then(|| {
                                fit_measured_table_declared_tail_to_declared_height(
                                    measured, table, dpi,
                                )
                            })
                            .flatten()
                    })
                    .unwrap_or_else(|| measured.clone())
            } else {
                fitted
            }
        })
    } else {
        None
    }
}
