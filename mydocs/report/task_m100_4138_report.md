---
kind: report
status: active
last_verified: 2026-08-07
---

# Task #4138 최종 보고서 — 셀 나누기 stale line_segs·vpos 사다리 붕괴 수정

Issue: #4138 (외부 리포트 "glyph truncation")
브랜치: `fix/issue-4138-split-cell-stale-linesegs`
단계 문서: [task_m100_4138_stage1.md](../working/task_m100_4138_stage1.md)

## 결론

`Table::split_cell*` 가 셀 폭을 바꾼 뒤 저장 line_segs 를 방치해 생기던 2단 결함
(옛 폭 줄의 셀 경계 잘림 + vpos 사다리 역행의 hard break 오판으로 인한 페이지
과소 적재)을, 분할 3형제 전부에 stale 셀 재래핑 + 셀 단위 vpos 사다리 단조
재구축을 배선해 수정했다. 수정 후 렌더는 이슈 첨부 한컴 기대 이미지와 1.1.2
문단 기준 줄 단위로 일치한다.

## 원인 사슬

1. `split_table_cell_into_native`(src/document_core/commands/table_ops.rs) →
   `Table::split_cell_into`(src/model/table.rs)가 셀 폭 44790→22395 변경, 저장
   line_segs 재계산 생략. 새 sibling 셀도 원본 line_segs 클론.
2. 렌더러 `LayoutEngine::layout_paragraph`(src/renderer/paragraph_layout.rs)는 저장
   줄을 그대로 그림 → 옛 폭(44508) 줄이 새 셀 클립 경계에서 잘림.
3. per-para `reflow_cell_paragraph` 만 배선하면 `reflow_line_segs` 가 문단 vpos
   원점을 보존한 채 내부 줄만 재래핑 → 줄 수 증가 문단 뒤로 사다리 역행 →
   컷 기계가 RowBreak hard break 로 오판 → 페이지 과소 적재(118→222쪽).

## 수정 내용 (commit `fix(core): #4138 …`)

- `reflow_stale_cells_after_split`(table_ops.rs): 저장 seg 폭 > 셀 폭(패딩 때문에
  정상 seg 는 항상 셀 폭 미만이므로 초과 = 확정 stale)인 셀의 전 문단 재래핑 후
  셀 사다리 재구축. `split_table_cell_native` / `split_table_cell_into_native` /
  `split_table_cells_in_range_native` 전부에 배선.
- `rebuild_table_cell_vpos_ladder_native` + `apply_cell_vpos_ladder`(text_editing.rs):
  `recalculate_cell_paragraph_vpos` 의 적용 루프를 분리 공유. 텍스트 편집 경로는
  기존대로 RowBreak reset 앞에서 정지, 전체 재래핑 직후 경로는 끝까지 재배치.
- undo: 분할은 studio snapshot undo(`save/restore_snapshot_native` 가 Document 통째
  복원)라 구 line_segs 자동 복원 — 코어 추가 작업 불요.

## 실측

| 구성 | stale | 사다리 역행 | 쪽수 |
| --- | --- | --- | --- |
| 수정 전 | 2,498문단/4,321seg | — | 118 |
| reflow만 | 0 | 발생 | 222 |
| 본 수정 | 0 | 0 | 195 |

성능(RHWP_2424_PROFILE): 본 수정 추가 비용 ≈0.1s. 분할 호출 3.43s 중 3.33s 는
분할 후 195쪽 거대 표 전체 재조판(`typeset_block_table_inner`, ~17ms/쪽) — Task #8
(fragment 단위 증분 재조판) 대상과 동일 병목이므로 본 과제 범위 밖.

## 검증

- 회귀: `tests/issue_4138_split_cell_stale_linesegs.rs` — stale 0 + 사다리 단조 +
  195쪽 핀, into/범위 분할 2경로. red→green(원복 시 stale 2,498/118쪽) 확인.
- 인접 focused 20 tests green (1073/1750/2158/2299/3236/3593/3595/3637 —
  2158/2299/3593 은 분리한 `apply_cell_vpos_ladder` 경유 reset-preserve 의미론 핀).
- 시각 증적: `mydocs/pr/assets/pr_4138_{presplit,before,after}_p{0,1}.png` —
  before 는 이슈 "실제" 스크린샷 재현, after 는 한컴 기대 이미지와 1.1.2 줄바꿈
  일치(1.1.1 은 1개 줄바꿈 근소 차이 — 작업지시자 판정 대상).
- PR #4122 merge(`accebdb20`) 후 devel 에 cherry-pick(`148bff3ef`)해 재검증 —
  전체 게이트(fmt/clippy/release-test/native-skia 3종/wasm-pack) 결과는 stage1
  문서 검증 기록에 기재.

## 미해결 (후속)

- (a) 중첩 표 host 문단의 잔존 stale seg — 한컴의 분할 시 중첩 표 리스케일 여부
  오라클 미확인(이슈 이미지 범위 밖). 테스트에서 계약 밖으로 명시 제외.
- 분할 후 전체 문서 쪽수(195)의 한컴 실측 대조 — 한컴 편집기 환경 필요.
- 병합(폭 확대) 경로의 reflow 부재 — 별도 이슈 후보.
