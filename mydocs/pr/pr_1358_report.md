# PR #1358 처리 보고서 — 미주 다단 오버플로 조사 + 시각 회귀 하니스 확장

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1358 |
| 작성자 | `planet6897` |
| 관련 이슈 | #1357, #1336, #1082 |
| 검토 브랜치 | `local/pr1358-merge-test` |
| 통합 방식 | 현재 `origin/devel` 기준 PR 단일 커밋 cherry-pick 검증 |
| 원 PR head | `5ffa0f1b` |
| 반영 커밋 | `6918f2e0` |
| 문서 정리 커밋 | `0950eb41` |
| PR close | `2026-06-10T14:56:59Z` |
| Issue #1357 close | `2026-06-10T14:57:03Z` |

## 2. 처리 내용

PR #1358은 미주 다단 페이지네이션 잔여 문제(#1336/#1357)를 직접 수정하지 않고, 조사 결과와 회귀 하니스 확장을 보존한다.

변경 내용:

- `scripts/task1274_visual_sweep.py`
  - 새 target `2024-09-below20above20` 추가
  - HWP: `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`
  - PDF: `pdf/3-09월_교육_통합_2024-구분선아래20구분선위20.pdf`
- contributor 작업 문서 추가 후 archive 정리
  - `mydocs/plans/archives/task_m100_1357.md`
  - `mydocs/plans/archives/task_m100_1357_impl.md`
  - `mydocs/report/archives/task_m100_1357_report.md`
  - `mydocs/working/archives/task_m100_1357_stage1.md`
  - `mydocs/working/archives/task_m100_1357_stageA.md`
  - `mydocs/working/archives/task_m100_1357_stageBC.md`

조사 결론:

- p21 col0 overflow를 줄이는 정밀 수정 시도는 overflow를 col1로 전이시켜 전체 회귀를 악화시켰다.
- 전 문서 오버플로 총합이 50.1px에서 72.9px로 악화되어 `issue_1082` 바운드 테스트가 실패하는 경로임을 확인했다.
- 따라서 현재는 `issue_1082_endnote_multicolumn_drift`의 60px 바운드로 추적하고, 근본 해결은 typeset 누적기 정합 전용 대형 타스크로 분리하는 것이 타당하다.

## 3. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

로컬 검증:

| 명령 | 결과 |
|---|---|
| `git diff --check` | 통과 |
| `python3 -m py_compile scripts/task1274_visual_sweep.py` | 통과 |
| `python3 scripts/task1274_visual_sweep.py --help` | 통과, 새 target 노출 확인 |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과, 5 passed |
| `cargo fmt --all -- --check` | 통과 |
| `python3 scripts/task1274_visual_sweep.py --target 2024-09-below20above20 --out output/poc/pr1358-visual-sweep --rhwp-bin target/debug/rhwp` | 통과 |

새 target sweep 산출:

- summary: `output/poc/pr1358-visual-sweep/summary.json`
- manifest: `output/poc/pr1358-visual-sweep/2024-09-below20above20/manifest.json`
- contact sheet: `output/poc/pr1358-visual-sweep/2024-09-below20above20/contact_sheet.png`
- flagged annotated pages: 7/23

sweep가 확인한 주요 잔여:

- p21 col0 `para=1156~1159`, 최대 overflow `82.1px`
- `render_tree_frame_tail_overflow` pages: 9, 17, 18, 19, 22
- line/column drift 후보 산출

## 4. 판정

**수용 가능**.

이번 PR은 런타임 렌더러 소스를 변경하지 않으며, 기존 문제를 "고친 척"하지 않고 실패한 정밀 수정 경로와 검증 기준을 문서화한다. 새 sweep target은 실제 샘플/PDF로 실행되어 분석 산출물을 생성했고, #1082 바운드 테스트도 통과했다.

이 PR의 가치는 다음과 같다.

- #1357 재착수 시 동일한 원인 조사와 실패 경로를 반복하지 않게 한다.
- `2024-09-below20above20` 샘플을 시각 회귀 세트에 넣어 cascade 회귀를 더 빨리 감지할 수 있게 한다.
- 현재 한계는 `deferred-tracked` 성격으로 명시되어 있어 후속 대형 타스크 범위를 분리하기 쉽다.

주의점:

- #1357의 근본 해결은 아니다.
- Issue #1357을 닫을 경우 "해결 완료"가 아니라 "바운드 추적으로 전환 / 후속 대형 타스크 분리"로 설명해야 한다.

## 5. 후속 절차

처리 완료:

- [x] `local/devel` 문서 정리 커밋 — `0950eb41`
- [x] `origin/devel` push — `0950eb41`
- [x] PR #1358에 메인테이너 코멘트 작성 — https://github.com/edwardkim/rhwp/pull/1358#issuecomment-4671568633
- [x] PR #1358 close — `2026-06-10T14:56:59Z`
- [x] Issue #1357 deferred-tracked 설명과 함께 close — `2026-06-10T14:57:03Z`
