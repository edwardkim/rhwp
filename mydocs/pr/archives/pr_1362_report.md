# PR #1362 처리 보고서 — 미주 제목 앞 gap 이중계상 정정 v2

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1362 |
| 작성자 | `planet6897` |
| 관련 이슈 | #1355 |
| 검토 브랜치 | `local/pr1362-merge-test` |
| 통합 방식 | 현재 `origin/devel` 기준 PR 단일 커밋 cherry-pick 검증 |
| 원 PR head | `5067f3f9` |
| 반영 커밋 | `7e9228f4` |
| 문서 정리 커밋 | `dccf4cf3` |
| PR close | `2026-06-10T15:20:25Z` |
| Issue #1355 close | `2026-06-10T15:21:49Z` |

## 2. 처리 내용

PR #1362는 #1355에서 보고된 해설(미주) 제목 앞 세로 여백 이중계상 문제를 정정한다.

증상:

- `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`
- p18 미주 영역에서 문30 제목 앞 여백이 과다
- 이후 문24 답안 본문이 하단을 초과하고 p19 문28 위치가 드리프트

원인과 수정:

- 직전 미주 문단이 수식 전용 tail이면 trailing line-spacing이 이미 흐름상 미주 사이 gap을 만든다.
- 그런데 현재 제목의 saved LINE_SEG vpos가 직전 bottom보다 크게 점프하면 `vpos_adjust`가 saved 기준 gap을 한 번 더 더한다.
- 이번 PR은 `current_is_endnote_question_title` + `endnote_flow` 조건 안에서 다음 조건을 모두 만족할 때만 제목 y를 흐름 위치로 되돌린다.
  - 직전 문단이 visible text가 없는 수식 전용 tail
  - `flow_advance`가 기존 미주 gap과 거의 같음
  - saved-vpos delta가 `5000HU`보다 큼
- v1에서 문제가 된 `flow_advance` 단독 판단은 사용하지 않는다.

변경 파일:

- `src/renderer/layout.rs`
- `tests/issue_1355_endnote_title_gap_double.rs`
- `mydocs/plans/task_m100_1355.md` → `mydocs/plans/archives/task_m100_1355.md`
- `mydocs/report/task_m100_1355_report.md` → `mydocs/report/archives/task_m100_1355_report.md`

## 3. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL Analyze (javascript-typescript) | pass |
| CodeQL Analyze (python) | pass |
| CodeQL Analyze (rust) | pass |
| CodeQL | pass |
| WASM Build | skipped |

로컬 검증:

| 명령 | 결과 |
|---|---|
| `git diff --check origin/devel..HEAD` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --test issue_1355_endnote_title_gap_double` | 통과, 1 passed |
| `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20` | 통과, 5 passed |
| `cargo test --test issue_1139_inline_picture_duplicate issue_1189_2022_oct_page11_endnote_question_gaps_match_pdf -- --exact` | 통과 |
| `cargo test --test issue_1139_inline_picture_duplicate issue_1274_2022_oct_page11_question20_equation_tail_stays_in_frame -- --exact` | 통과 |
| `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2022_oct` | 통과, 4 passed |
| `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2022_sep_page17_question27_starts_at_pdf_top -- --exact` | 통과 |
| `cargo test --lib` | 통과, 1622 passed / 0 failed / 6 ignored |
| `cargo clippy -- -D warnings` | 통과 |
| `cargo test --test svg_snapshot` | 통과, 8 passed |

시각 판정용 산출물:

- `output/poc/pr1362-endnote-gap/3-09월_교육_통합_2024-구분선아래20구분선위20_018.svg`
- `output/poc/pr1362-endnote-gap/3-09월_교육_통합_2024-구분선아래20구분선위20_019.svg`

## 4. 판정

**수용 가능**.

수정 범위는 미주 제목 배치 보정에 한정되어 있고, 기존 미주/수식 tail 회귀군 및 `svg_snapshot`이 모두 통과했다. v1 회귀 원인이었던 `flow_advance` 단독 판단을 피하고, `textless + saved-vpos jump` 복합 조건으로 좁혀 적용한 점도 타당하다.

주의점:

- `saved_delta_hu > 5000`은 계측 기반 임계값이다. 향후 다른 미주 문서에서 수식 전용 tail + 큰 saved-vpos jump가 정상 gap인 사례가 발견되면 재조정이 필요하다.
- 이번 PR은 p21→p22 페이지높이 오버플로(#1357)까지 해결하는 범위는 아니다.

## 5. 후속 절차

처리 완료:

- [x] 작업지시자 승인
- [x] `local/devel`에 PR 커밋 반영 — `7e9228f4`
- [x] contributor 작업 문서 archive 정리
- [x] 문서 정리 커밋 — `dccf4cf3`
- [x] `origin/devel` push — `dccf4cf3`
- [x] PR #1362에 메인테이너 코멘트 작성 — https://github.com/edwardkim/rhwp/pull/1362#issuecomment-4671749265
- [x] PR #1362 close — `2026-06-10T15:20:25Z`
- [x] Issue #1355 처리 코멘트 작성 — https://github.com/edwardkim/rhwp/issues/1355#issuecomment-4671758668
- [x] Issue #1355 completed close — `2026-06-10T15:21:49Z`
