# PR #1752 리뷰 — Task #1749 saved bounds 신뢰 페이지-마지막 증거 조건

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1752 |
| 작성자 | planet6897 |
| base | devel |
| head | pr/devel-1749 |
| 제목 | Task #1749: saved bounds 신뢰에 페이지-마지막 증거 조건 (쪽 경계 overfill 수정) |
| 문서 작성 시점 참고값 | draft=false, mergeable=MERGEABLE |
| 로컬 검토 브랜치 | `local/pr_page2_batch_check` |

## 체리픽 검토

- 오래된 순서 누적 검토: #1746 -> #1751 -> #1752 -> #1754
- 적용 커밋: `699208e240d3943ca8f891294b3f69e3921e474c`
- 로컬 체리픽 커밋: `5a349f5b0`
- #1746, #1751 적용 후 `upstream/devel` 기준 누적 체리픽 충돌 없음.

## 변경 범위

- `src/renderer/typeset.rs`: `saved_flow_marks_page_last`를 추가해 saved bounds 신뢰를 문서 끝, vpos 리셋, 명시적 쪽/구역나누기 증거가 있는 경우로 제한.
- `tests/issue_1749_saved_bounds_cumulative.rs`: 누적좌표 문서의 꼬리 공백 문단 overfill 방지 검증.
- `tests/issue_1749_saved_bounds_page_break.rs`: 누적좌표 문서라도 다음 문단이 명시적 쪽나누기이면 saved bounds를 신뢰하는 v2 케이스 검증.
- `samples/task1749/*`와 관련 문서 추가.

## 검토 결과

- `saved_flow_marks_page_last`는 synthetic line을 제외하고 다음 실줄을 탐색하므로 빈 문단을 건너뛰는 케이스를 처리한다.
- 다음 문단이 `ColumnBreakType::Page | Section`이면 누적 vpos라도 페이지 마지막 증거로 인정해 v2 회귀를 방지한다.
- #1750의 `stored_whole_para_reset`은 분할 직전 가드이고, #1752의 saved bounds 신뢰 제한은 전체 배치 fit 경로이므로 적용 위치가 분리되어 있다.

## 로컬 검증

- `rm -rf target/*`
- `env CARGO_INCREMENTAL=0 cargo fmt --check`: 통과
- `git diff --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1749_saved_bounds_cumulative --test issue_1749_saved_bounds_page_break -- --nocapture`: 2 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## PR 내용 기준 검증

- 이 PR의 핵심은 saved bounds를 무조건 신뢰하지 않고, 문서 끝/리셋/명시적 쪽·구역나누기처럼 페이지 마지막 증거가 있을 때만 신뢰하는지다.
- 두 통합 테스트가 누적좌표 overfill 방지와 명시적 쪽나누기 예외를 각각 직접 검증한다.
- 추가 HWP/HWPX 샘플도 `dump-pages` 기준으로 핵심 조건을 만족한다.
  - `saved_bounds_cumulative_vpos`: HWP/HWPX 모두 2쪽이며 pi18은 2쪽 시작으로 간다.
  - `saved_bounds_cumulative_page_break`: HWP/HWPX 모두 5쪽이며 pi26은 2쪽 마지막 문단으로 남는다.
- 기준 PDF visual sweep에서 HWP 기준은 두 샘플 모두 자동 후보 0건이다.
- HWPX `saved_bounds_cumulative_page_break`는 PR 핵심 위치 조건은 맞지만, 5쪽 하단 `render_tree_frame_tail_overflow` 후보가 남았다. 이는 pi26 saved-bounds 예외 처리 자체와는 다른 후속 시각 검토 후보로 분리한다.

## 시각 검증

- `samples/task1749/saved_bounds_cumulative_vpos.hwpx` vs `samples/task1749/saved_bounds_cumulative_vpos-2024.pdf`
  - SVG/PDF pages: 2/2
  - flagged: 0/2
  - review contact sheet: `output/pr-page2-visual/pr1752-saved-bounds-vpos-hwpx/review_contact_sheet.png`
  - `visual_accuracy_proxy_percent`: average 13.06509, worst 4.67736
- `samples/task1749/saved_bounds_cumulative_vpos.hwp` vs `samples/task1749/saved_bounds_cumulative_vpos-2024.pdf`
  - SVG/PDF pages: 2/2
  - flagged: 0/2
  - review contact sheet: `output/pr-page2-visual/pr1752-saved-bounds-vpos-hwp/review_contact_sheet.png`
  - `visual_accuracy_proxy_percent`: average 13.08623, worst 4.83585
- `samples/task1749/saved_bounds_cumulative_page_break.hwpx` vs `samples/task1749/saved_bounds_cumulative_page_break-2024.pdf`
  - SVG/PDF pages: 5/5
  - flagged: 1/5
  - flagged page: p5 (`render_tree_frame_tail_overflow`, `line_band_drift`, `column_line_band_drift`, `large_ink_region_drift`)
  - review page: `output/pr-page2-visual/pr1752-saved-bounds-page-break-hwpx/review/review_005.png`
  - review contact sheet: `output/pr-page2-visual/pr1752-saved-bounds-page-break-hwpx/review_contact_sheet.png`
  - `visual_accuracy_proxy_percent`: average 11.55304, worst 4.54105
- `samples/task1749/saved_bounds_cumulative_page_break.hwp` vs `samples/task1749/saved_bounds_cumulative_page_break-2024.pdf`
  - SVG/PDF pages: 5/5
  - flagged: 0/5
  - review contact sheet: `output/pr-page2-visual/pr1752-saved-bounds-page-break-hwp/review_contact_sheet.png`
  - `visual_accuracy_proxy_percent`: average 12.02853, worst 4.49959

## PR 코멘트 처리 시 PNG 위치

- 추가 보완 후보 대표 샘플 1장의 로컬 위치만 기록한다.
- 코멘트 처리 시 제시할 PNG: `mydocs/pr/assets/pr_1752_visual_review_p5_followup_candidate.png`
- 코멘트 요지: PR 핵심 조건(pi26 2쪽 마지막 문단 유지, 전체 5쪽 유지)은 맞지만, HWPX p5 하단 tail/line drift 후보가 남아 후속 이슈로 분리해 추적한다.
- 코멘트 요지에 다음 요청도 포함한다: 이후 시각 대조가 필요한 샘플을 추가할 때는 한컴 2020, 한컴 2024 등 기준 환경에서 저장한 PDF 파일도 함께 업로드해 달라고 요청한다.
- 정상 케이스 PNG는 별도로 첨부하지 않는다. 이 PR 코멘트는 보완 후보 1장만 첨부해 후속 범위를 명확히 한다.

## 결론

PR 핵심 조건인 saved bounds 신뢰 제한과 명시적 쪽나누기 예외는 로컬 테스트와 HWP/HWPX `dump-pages` 기준으로 맞다. `saved_bounds_cumulative_page_break.hwpx` 5쪽 하단의 tail/line drift 후보는 pi26 saved-bounds 예외 처리 자체와 다른 후속 시각 검토 범위이므로, PR 코멘트에서 후속 이슈로 분리해 추적하겠다고 안내한 뒤 merge 후보로 판단한다.

## 후속 처리 결과

- 통합 PR: #1810
- merge commit: `716fbca92ef4d5c67194ec6575bcc06413beacf6`
- 원 PR 코멘트: https://github.com/edwardkim/rhwp/pull/1752#issuecomment-4866610425
- 원 PR 상태: superseded close 완료
- 관련 이슈: #1749 close 완료, https://github.com/edwardkim/rhwp/issues/1749#issuecomment-4866621022
- 후속 이슈: #1811, https://github.com/edwardkim/rhwp/issues/1811
