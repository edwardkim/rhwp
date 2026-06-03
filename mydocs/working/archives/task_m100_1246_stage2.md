# Stage 2 완료 보고서 — Task #1246: HeightCursor 미주 min-gap (render)

- **이슈**: #1246 (M100) — #1238 흡수
- **브랜치**: `feature/issue-1246-endnote-vpos-anchor`
- **단계**: Stage 2 / 4 (구현)
- **작성일**: 2026-06-02

## 구현 내용

### 1. #1238 render-클램프 제거
`paragraph_layout.rs` 의 `layout_composed_paragraph` 진입부 min-gap 클램프 + `last_item_content_bottom`
병행 추적(`endnote_prev_content_bottom`) 제거. vpos 위치 시스템과 충돌하던 hack 폐기.

### 2. between-notes 마진 배선 (스칼라화)
- `typeset.rs`: 섹션 미주 between-notes 마진(HU)을 `endnote_between_notes_hu` 스칼라로 수집 →
  `PaginationResult` 전달 (#1238 의 `Vec<(local,hu)>` 맵 → 스칼라로 단순화).
- `layout.rs`: `set_endnote_between_notes_hu(hu)` 세터 + `endnote_between_notes_hu` Cell.
- `rendering.rs`: 섹션 렌더 셋업에서 전달.

### 3. HeightCursor min-gap (핵심)
- `HeightCursor` 에 `endnote_between_notes_hu: i32` 필드. `build_single_column` 이 미주 흐름
  컬럼에만 주입(본문 0).
- `vpos_adjust` 말미: 결과 y 계산 후, **새 미주 제목 & forward & 다줄 prev & stored vpos gap≈0
  (`[-0.5,4.0)`px)** 이면 `y_offset + prev_line_spacing_px`(직전 주입 between-notes)로 끌어올림.
  - **핵심 게이트**: stored gap 이 거의 0 일 때만(다줄 마지막 줄 trailing 누락=문22). 의도적
    소-gap(문13 ~10px, backtrack)은 존중 → over-lift 회피.
  - backtrack/rewind 류(result < y_offset)·단일줄 prev 제외 → #1209 무회귀.

## 검증

| 항목 | 결과 |
|------|------|
| **문22 gap** (3-11월 page14, 원래 버그) | **0 → 26.5px** ✅ |
| `issue_1189_2022_nov_pages10_12` (pi=475 overflow) | ✅ 통과 (이전 미해결분 해소) |
| `issue_1189_2022_nov_page17` (문28 수식) | ✅ 통과 |
| `issue_1209_2022_sep_page10_question12` (문12 backtrack) | ✅ 통과 (over-lift 회피) |
| HeightCursor 단위테스트 (신규 3 + 기존 26) | ✅ 29 통과 |
| **전체 `cargo test`** | ✅ **1945 passed, 0 failed** |

신규 단위테스트: `compact_endnote_min_gap_lifts_zero_gap_question_title`(gap≈0 → lift),
`..respects_existing_vpos_gap`(gap 10px → 무보정), `..skips_single_line_prev`(단일줄 → 무보정).

## Stage 3(pagination 정합) — Stage 2 에서 동시 달성

#1238 에서 pi=475 overflow 는 render-클램프가 pagination 과 desync 해 발생했다. Stage 2 의
min-gap 은 보정값으로 **`prev_line_spacing_px`(=pagination 의 `compute_en_metrics` 가 이미
`bottom_with_spacing` 으로 예약한 trailing)** 를 사용하므로 render·pagination 이 정합한다.
→ pi=475 overflow 가 별도 pagination 수정 없이 해소(전체 overflow/page-count 테스트 통과로 확인).
**구현계획서 Stage 3 의 목표(pagination 정합)는 Stage 2 설계로 충족.** 별도 `compute_en_metrics`
변경 불요.

## 다음 (Stage 4)

- 한컴 2022 PDF 시각 정합 확인(3-11월 page10/14/17, 3-09·10월), 골든 스냅샷 최종 점검.
- 최종 보고서 `report/task_m100_1246_report.md` + stream/devel 기준 squash PR. (closes #1238, #1246)
