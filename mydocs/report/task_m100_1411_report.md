# Task M100 #1411 최종 보고서 — PR #1410 후속 공식 미주 모양 모델 잔여 검증

- 이슈: #1411 `PR #1410 후속: 공식 미주 모양 모델 잔여 검증`
- 브랜치: `local/task_m100_1411`
- 기준 브랜치: `upstream/devel` `a2a1b383`
- 작성일: 2026-06-15

## 1. 작업 요약

PR #1410 병합 뒤 최종 sweep에 남은 3개 target, 7개 후보를 재현하고 공식 미주 모양 모델 잔여인지 분류했다.

| target | baseline | 최종 | 판단 |
|--------|---------:|-----:|------|
| `2022-10` | `1/18` | `0/18` | p14 textless tall equation tail 뒤 문항 title gap 이중 보존을 실제 layout 결함으로 확인하고 보정했다. |
| `2024-09-below20-above20` | `3/23` | `3/23` | p19/p20/p22는 TAC shape/equation cluster split cascade 잔여로 분류했다. |
| `2024-11-practice-above0-between20-below2` | `3/22` | `3/22` | p17/p20/p21은 TAC shape/equation/table continuation split 잔여로 분류했다. |

최종 판단상 공식 `구분선 위`, `구분선 아래`, `미주 사이` 계산식 자체의 직접 잔여는 없다.

## 2. 구현 내용

- `src/renderer/layout.rs`
  - 하단부 textless equation tail 뒤 문항 제목에서 실제 content-bottom gap이 이미 확보된 경우, logical note title gap을 한 번 더 보존하지 않도록 제한했다.
  - 같은 미주 본문까지는 생략한 gap을 유지하되, 다음 미주 제목으로 넘어갈 때 vpos base를 지연 복원해 후속 미주 전체가 같이 당겨지지 않게 했다.
  - 하단부 조건을 둬 기존 p10 계열 title gap 회귀를 피했다.

## 3. 단계별 결과

- Stage 1: `output/task1411_stage1_baseline`
  - targeted sweep에서 `7/63` 후보를 재현했다.
- Stage 2: `2022-10` p14
  - 문24 tail 수식과 문25 첫 본문 쉼표 bbox overlap을 확인했다.
  - 공식 미주 모양값 문제가 아니라 하단부 textless/tall equation tail 뒤 title gap 이중 보존 문제로 분류했다.
- Stage 3: layout 보정
  - `2022-10` target이 `1/18`에서 `0/18`로 전환됐다.
  - 전체 3개 target 잔여는 `7/63`에서 `6/63`으로 감소했다.
- Stage 4: `2024-09-below20-above20`
  - p19/p20/p22는 문28/문29/문30 large TAC shape와 equation tail continuation cascade 잔여로 분류했다.
- Stage 5: `2024-11-practice-above0-between20-below2`
  - p17/p20/p21은 문26/문28 tall equation, large TAC shape, table continuation split 잔여로 분류했다.

## 4. 검증

- `cargo build --bin rhwp`: 통과
- `cargo test --lib compact_endnote_question_title_after_tall_tail_limited_backtrack`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- targeted visual sweep:
  - 명령: `python3 scripts/task1274_visual_sweep.py --target 2022-10 --target 2024-09-below20-above20 --target 2024-11-practice-above0-between20-below2 --out output/task1411_stage3_after_fix_v2 --rhwp-bin target/debug/rhwp`
  - 결과: `2022-10` `0/18`, `2024-09-below20-above20` `3/23`, `2024-11-practice-above0-between20-below2` `3/22`
- PR 전 로컬 필수 검증:
  - `cargo build --release`: 통과
  - `cargo test --release --lib`: 통과 (`1819 passed`, `6 ignored`)
  - `cargo test --profile release-test --tests`: 통과
  - `cargo fmt --check`: 통과

## 5. PR 준비와 CI 계획

- PR용 원격 브랜치: `task_m100_1411`
- PR 대상: `edwardkim/rhwp` `devel`
- PR 본문에는 `Closes #1411`을 포함한다.
- push 후 GitHub Actions required checks를 재확인한다.

## 6. 산출물

- 수행 계획서: `mydocs/plans/task_m100_1411.md`
- 구현 계획서: `mydocs/plans/task_m100_1411_impl.md`
- 단계 문서: `mydocs/working/task_m100_1411_stage1.md`부터 `stage5.md`
- 최종 보고서: `mydocs/report/task_m100_1411_report.md`
