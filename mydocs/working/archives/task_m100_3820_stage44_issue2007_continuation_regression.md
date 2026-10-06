---
kind: investigation
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 44 — issue2007 native RowBreak continuation 소유 회귀 분석

## 발생 계기와 범위 분리

Stage 43 H5의 focused gate를 통과한 뒤, 안정된 source에서 실행한
`cargo test --profile release-test --tests`는 `issue_2007_nested_cell_pagination`의 다음
기존 계약에서 실패했다.

```text
issue_2007_continuation_frame_restarts_and_drops_previous_page_residual
p10 must not paint the second line owned by p11
```

대상은 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`의 p10/p11 경계다.
Hancom PDF와 기존 gate의 계약은 p10이 `제50조의2`의 첫 줄만 owner로 가지고, 둘째 줄은 p11에서
시작해야 한다는 것이다. 현재 p10이 둘째 줄까지 paint한다.

이는 Stage 43의 `59043_regulatory_analysis.hwp` p11/p12 empty-TAC Picture slot 보정과 직접 같은
render path가 아니다. Stage 43 H5는 `table_partial.rs`의 **빈 run + TAC Picture + 두 개 이상
LINE_SEG** fallback에만 한정되며, issue2007 실패 문장은 text-bearing nested cell이다. 따라서 H5를
되돌리거나 p12 회귀 gate를 완화하는 것은 금지한다.

## 우선 가설 H1 — Stage 42의 native-HWP fragment viewport 확대가 nested continuation까지 번졌다

현재 worktree의 Stage 42 변경은 `table_layout.rs`에서 `fragment_cut_units`의
`hwpx_stored_layout()` 조건을 제거해 native HWP5 `RowBreak` single-row fragment에도 source-unit
viewport를 부여한다. 해당 변경의 목적은 overflow cell ink를 RenderTree에서 제거하는 것이며,
Stage 42 문서는 76076/86712/#3637 overflow fixture로 검증됐다.

issue2007은 native HWP5 RowBreak nested-cell continuation이다. 같은 viewport 규칙을 여기에
넓히면 p10의 physical cell clip과 line ownership cursor가 달라져, 본래 p11로 넘겨야 할 둘째
text line을 p10 source range에 포함시킬 수 있다. 실패 문구와 이 predicate의 format/profile
경계가 정확히 겹치므로, 먼저 H1을 검증한다.

다만 "native HWP5를 다시 모두 제외"하는 rollback은 Stage 42가 해결한 overflow fixture를
되살릴 수 있으므로 금지한다. 다음 단계는 p10 outer `pi=7, ci=1`의
`single_row_fragment`, `row_filter`, `split_terminal`, source cut end, `fragment_line_ranges`를
계측하여, **nested continuation인 경우만** source viewport가 owner를 앞당기는지 확인하는 것이다.

## 판정 기준과 다음 작업

1. issue2007 focused test를 source cut diagnostic과 함께 재현한다.
2. p10의 `cell_line_ranges_from_cut` 끝이 p11 소유 둘째 줄을 포함하는지 확인한다.
3. H1이 맞으면 Stage 42 overflow 보정의 narrow predicate에서 nested text continuation을 분리하고,
   Stage 42 overflow gate와 issue2007 p10/p11 gate를 모두 재실행한다.
4. H1이 틀리면 Stage 43 H1~H4의 `CellUnit` geometry가 p10 text unit을 바꿨는지 별도 비교한다.

코드 수정 전에는 위 provenance 표와 Hancom PDF p10/p11 raster를 이 문서에 추가한다.

## H1 1차 반증 — 동일 source의 단독/직렬 실행은 통과한다

전체 gate에서 실패한 뒤 같은 release-test binary를 다시 실행했다. 결과는 다음과 같다.

| 실행 범위 | 결과 | 의미 |
| --- | --- | --- |
| `issue_2007_continuation_frame_restarts_and_drops_previous_page_residual` 단독 | 통과 | p10/p11 layout의 단독 출력은 기존 계약을 만족 |
| `issue_2007_nested_cell_pagination --test-threads=1` | 9/9 통과 | source unit/fragment geometry가 직렬 순서에서는 안정적 |
| Stage 43 release-test binary 기본 실행 | 8/9, 같은 p10 owner assertion 실패 | p10 owner 경계가 안정적으로 잘못됨 |

따라서 Stage 42 native-HWP viewport 확대를 즉시 rollback하는 H1 보정은 중단한다. 그런 rollback은
Stage 42 overflow-cell 결함을 되살릴 위험이 있고, 같은 source에서 단독 p10 layout이 이미 정상인
관측과도 맞지 않는다.

## H2 — 실행 병렬성보다 최적화 바이너리/메모리 배치에 민감한 소유 범위 상태 (폐기)

위 1차 관측 뒤, 같은 테스트 binary를 별도 프로세스로 반복 실행해 병렬성 가설을 다시 검증했다.
Stage 43 binary는 6회 모두 같은 p10 owner assertion으로 실패했고, Stage 44 binary는 12회 모두
9/9 통과했다. Stage 44의 source 차이는 실제 layout 정책 변경이 아니라 source cut의 임시 계측
branch뿐이며, 계측 조건도 실행 중 충족되지 않아 로그를 내지 않았다.

| binary | 별도 프로세스 반복 | 결과 | 결론 |
| --- | ---: | --- | --- |
| `target/task-3820-3821-stage43/...issue_2007...` | 6회 | 6회 모두 p10 두 번째 줄 소유 실패 | 병렬 실행만의 문제 아님 |
| `target/task-3820-3821-stage44/...issue_2007...` | 12회 | 12회 모두 9/9 통과 | no-op branch가 codegen/stack 배치를 바꿔 증상을 숨길 가능성 |

따라서 앞 절의 “병렬 test binary만 실패”라는 표현은 **폐기**한다. 처음에는 반복 가능한
binary-sensitive renderer 상태로 보였으나, 이어진 source diff 확인에서 Stage 43과 Stage 44 사이에
실제 layout 변경이 있었음이 확인됐다. 따라서 아래 H3의 직접 산출물/코드 경로 증거로 H2도 폐기한다.

## H3 확정 — Stage 42의 nested continuation paint viewport 보정이 p10/p11 owner를 회복했다

Stage 43/44 release-test CLI로 같은 입력의 p10(0-based 9)을 각각 `export-svg` 및
`export-render-tree` 했다. Page count(17)와 outer p10 fragment bbox는 같았지만, source owner가
다르다.

| 산출물 | `제50조의2` 첫 줄 | 둘째 줄 `행하여야 하며…` | 판정 |
| --- | --- | --- | --- |
| Stage 43 RenderTree | `y=974.7`, 존재 | `y=990.7`, 존재 | p10이 p11 줄까지 paint — 실패 |
| Stage 44 RenderTree | `y=974.7`, 존재 | 없음 | Hancom PDF 및 기존 p10/p11 contract와 일치 |

두 binary가 단순히 계측 branch만 달랐다는 전제는 틀렸다. 현재 source에는 Stage 42가 추가한
`mixed_nested_split_from_cut`의 native HWP5 continuation 보정이 있다
(`src/renderer/layout/table_layout.rs`, `compensate_first_visible` 분기).
1×1 host 안의 1×1 nested continuation에서 이미 앞 page가 예약한 첫 unit만큼 content origin을
전진시키면, 같은 양을 child paint viewport에서도 빼야 한다. 그렇지 않으면 p10의 viewport 끝이
한 줄 길어져 p11 소유 둘째 줄을 함께 paint한다. 이 함수의 해당 분기는 정확히
`flow_visible - first_visible_content_height`를 사용하고, source 주석에도 `42065 p10/p11` 계약을
명시한다.

이는 Stage 42의 작업 중 이미 worktree에 있는 변경이며, Stage 43의 earlier binary에는 아직 포함되지
않았던 것으로 판정한다. 따라서 이 Stage에서 renderer 정책을 새로 바꾸거나 Stage 42 변경을
rollback하지 않는다. Stage 44의 역할은 이 owner 경계를 direct RenderTree로 재검증하고, focused 및
전체 gate를 다시 실행하는 것이다.

원래 H2에서 예정한 no-op branch 제거 실험은 근거가 사라졌으므로 수행하지 않는다. 현재 source에는
그 임시 branch 자체가 남아 있지 않다.

다음 단계에서는 Stage 42의 다른 focused overflow gate를 훼손하지 않는지 확인한 후, 현재 source로
`issue_2007` focused gate와 전체 integration gate를 순서대로 재실행한다. 이 단계에서도 Stage 42
overflow predicate와 Stage 43 H5 empty-TAC 보정은 수정하지 않는다.
