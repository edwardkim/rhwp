# PR #1754 리뷰 — Task #1753 지연 자리차지 표 후속 텍스트 선행 채움

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1754 |
| 작성자 | planet6897 |
| base | devel |
| head | pr/devel-1753 |
| 제목 | Task #1753: 지연 자리차지 표의 후속 텍스트 선행 채움 (부분) |
| 문서 작성 시점 참고값 | draft=false, mergeable=MERGEABLE |
| 로컬 검토 브랜치 | `local/pr_page2_batch_check` |

## 체리픽 검토

- 오래된 순서 누적 검토: #1746 -> #1751 -> #1752 -> #1754
- 적용 커밋: `6e07bb98134a5d88959de22102bdbda143ad5c6e`
- 로컬 체리픽 커밋: `b76668bbf`
- 선행 세 PR 적용 후 `upstream/devel` 기준 누적 체리픽 충돌 없음.

## 변경 범위

- `src/renderer/typeset.rs`: `prefill_before_deferred_table`로 visible-host TopAndBottom RowBreak 표가 다음 쪽으로 이월되기 직전 후속 control-free 문단을 현재 쪽 잔여 공간에 선행 배치.
- `TypesetState.prefilled_paras`: 선행 배치된 문단이 메인 루프에서 중복 배치되지 않도록 추적.
- `tests/issue_1753_deferred_table_fill_ahead.rs`: pi=52/53 선행 채움, 표 첫 fragment 유지, 중복/잔류 방지 검증.
- `samples/task1753/deferred_takeplace_fill_ahead.hwpx`와 관련 문서 추가.

## 검토 결과

- prefill 가드는 단일 단, 현재 쪽 항목 존재, visible host text, TopAndBottom float, v_off >= 0, RowBreak table을 모두 요구해 범위가 제한적이다.
- 후보 문단은 control-free 문단으로 제한되고 저장 vpos가 host 이후 같은 쪽 범위에 있어야 하므로 표/그림 포함 후속 문단이나 누적좌표 문서에 과적용될 가능성을 줄였다.
- `prefilled_paras` 스킵으로 중복 배치 방지가 명시되어 있다.
- 문서 자체가 "부분" 수정임을 밝히고 있으며 pi51 host 제목 줄 잔여 이슈는 후속 범위로 남긴다. 이 한계는 blocker가 아니라 범위 명시로 판단한다.

## 로컬 검증

- `rm -rf target/*`
- `env CARGO_INCREMENTAL=0 cargo fmt --check`: 통과
- `git diff --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1753_deferred_table_fill_ahead -- --nocapture`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## PR 내용 기준 검증

- 이 PR의 핵심은 지연 자리차지 RowBreak 표가 다음 쪽으로 밀릴 때 후속 control-free 문단을 현재 쪽 잔여 공간에 선행 배치하는지다.
- `tests/issue_1753_deferred_table_fill_ahead.rs`가 pi=52/53 선행 채움, 표 fragment 유지, 중복 배치 방지를 직접 검증한다.
- PR 자체가 "부분" 수정임을 밝히므로, 남은 host 제목 줄 이슈는 merge blocker가 아니라 후속 범위로 본다.
- `samples/task1753/deferred_takeplace_fill_ahead-2024.pdf` 기준 visual sweep에서 HWP/HWPX 모두 자동 후보 0건이다.

## 시각 검증

- `samples/task1753/deferred_takeplace_fill_ahead.hwpx` vs `samples/task1753/deferred_takeplace_fill_ahead-2024.pdf`
  - SVG/PDF pages: 21/21
  - flagged: 0/21
  - review contact sheet: `output/pr-page2-visual/pr1754-deferred-fill-hwpx/review_contact_sheet.png`
  - `visual_accuracy_proxy_percent`: average 9.82738, worst 6.42037
- `samples/task1753/deferred_takeplace_fill_ahead.hwp` vs `samples/task1753/deferred_takeplace_fill_ahead-2024.pdf`
  - SVG/PDF pages: 21/21
  - flagged: 0/21
  - review contact sheet: `output/pr-page2-visual/pr1754-deferred-fill-hwp/review_contact_sheet.png`
  - `visual_accuracy_proxy_percent`: average 9.82738, worst 6.42037

## PR 코멘트 처리 시 PNG 위치

- 정상 대표 샘플 1장의 로컬 위치만 기록한다.
- 코멘트 처리 시 제시할 PNG: `mydocs/pr/assets/pr_1754_visual_review_p9.png`
- 코멘트 요지: p9에서 pi=52/53 후속 control-free 문단이 현재 쪽 잔여 공간에 선행 배치되고, HWP/HWPX visual sweep 자동 후보는 모두 0/21이다.
- 코멘트 요지에 다음 요청도 포함한다: 이후 시각 대조가 필요한 샘플을 추가할 때는 한컴 2020, 한컴 2024 등 기준 환경에서 저장한 PDF 파일도 함께 업로드해 달라고 요청한다.

## 결론

PR 내용 기준 로컬 검증과 기준 PDF visual sweep 모두 통과했다. #1746 -> #1751 -> #1752 이후 merge 후보로 판단한다.

## 후속 처리 결과

- 통합 PR: #1810
- merge commit: `716fbca92ef4d5c67194ec6575bcc06413beacf6`
- 원 PR 코멘트: https://github.com/edwardkim/rhwp/pull/1754#issuecomment-4866610928
- 원 PR 상태: superseded close 완료
- 관련 이슈: #1753 close 완료, https://github.com/edwardkim/rhwp/issues/1753#issuecomment-4866621858
