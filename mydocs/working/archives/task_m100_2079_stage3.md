# 단계 완료 보고 — Task M100 #2079 3단계: P6 판정 함수군 추출 (배치 A+B, 9건)

- 작성일: 2026-07-09 / 브랜치: `local/task2079` / goal 방식 (자체 검증)

## 수행 내용

P6 판정 연쇄 중 시뮬형/대형 9건을 `judge_*` &self 메서드로 추출 (전부 읽기 전용,
직접 파라미터 — 단일 호출부, emit_endnote_split 전례):

배치 A (예측기 연동 3건): split_head_render_overflows(53줄) /
last_column_visual_split(91줄, Option<usize>) / title_tail_body_advances_page(60줄)
배치 B (6건): tail_render_overflows(80) / last_column_question_title_tail_fits(73) /
zero_question_intro_tail_before_rewind_fits(57) / visible_separator_vpos_head_group_outside(54) /
default_between_large_below_head_group_outside(50) / last_column_vpos_head_group_outside(55)

- 판정 간 의존은 Copy 값 전달(this_first_offset/split_endnote_to_fit/
  large_between_question_title_render_y/head_inside_frame 등) — mut 캐리 0.
- 집계자(advance_for_fit 논리 76 / advance_for_new_endnote 42)는 지역 불리언 집계라
  잔류(구현계획서 설계대로). 미니 집계자 large_between_notes_vpos_head_outside 잔류.
- 컴파일러 검출 보정 5건: `paragraphs` 파라미터 누락 4건(E0425), `composed` 가 en_para
  의 ComposedParagraph 지역(슬라이스 아님)인 타입 오판 1건(E0308) — 사각지대 신규 아님.

## 게이트 (전수 통과, 자체 검증)

fmt ✓ / clippy **0** / `--tests` **2,945/0** / issue_1116 **13/13** / OVR 5샘플 회귀 **0건**.

## 계측 (표적 공식 CC 포함)

| 항목 | 시작 (r8) | 2단계 후 | 현재 |
|---|---|---|---|
| `typeset_endnote_paragraphs` 공식 CC | 138 | 129 | **122** |
| 동 함수 줄수 | 4,227 | 3,844 | **3,588** |
| 신규 judge_* 9건 | — | — | 최대 CC 24 (전부 ≤25) |

## 다음 단계

4단계 — 배치 C (잔여 중형 판정 ~8건) → 목표 100 내외 접근 후 재평가.
