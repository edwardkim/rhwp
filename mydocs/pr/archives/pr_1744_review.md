# PR #1744 리뷰 — RowBreak continuation footer/spacing 부분 보정

- PR: #1744 `Task #1728 (부분): 자동 쪽번호 세로 위치 margin_footer 정합`
- 작성자: @planet6897
- 기준 브랜치: `devel`
- PR head: `49d9e858c2108c76aee8f89ea2d2d7addaa10ba5` (문서 작성 시점 참고값)
- merge commit: `4da379b4385570be5b73d75fa9facc41bf5717dd`
- 관련 이슈: #1728
- 규모: 8 files, +219/-5
- 처리 결과: GitHub Actions 통과 후 admin merge 완료
- PR 후속 코멘트: https://github.com/edwardkim/rhwp/pull/1744#issuecomment-4865221629
- 이슈 후속 코멘트: https://github.com/edwardkim/rhwp/issues/1728#issuecomment-4865223824
- 관련 이슈 처리: #1728은 부분 해결이므로 open 유지

## 변경 요약

#1728의 RowBreak continuation PDF 시각 차이 중 두 갈래를 부분 보정하는 PR이다.

핵심 변경:

- `src/renderer/layout.rs`
  - footer 쪽번호 y 계산을 `footer_area.height / 2` 기준에서 `margin_footer / 2` 기준으로 변경
  - RowBreak continuation 첫 가시 문단의 `spacing_before` 보존용 engine flag 추가
- `src/renderer/layout/paragraph_layout.rs`
  - `keep_continuation_column_top_spacing_before`가 켜진 문단은 column-top이어도 `spacing_before`를 적용
- `src/renderer/layout/table_partial.rs`
  - cut unit이 있는 continuation 조각의 첫 가시 문단에만 flag를 set/reset
  - 1x1 linear 셀은 기존 정합을 보존하기 위해 대상에서 제외
- `tests/golden_svg/form-002/page-0.svg`
  - footer 쪽번호 위치 변화에 따른 golden 갱신
- 문서
  - `task_m100_1728_*` 계획/보고 문서 추가

## conflict 해소

#1743 merge 이후 `src/renderer/layout.rs`의 `LayoutEngine` 필드 추가 위치에서 충돌이 발생했다.

- #1744: `keep_continuation_column_top_spacing_before`
- #1743: `hwpx_page_preview`

두 필드는 독립 상태값이므로 둘 다 유지하는 방향으로 메인터너 권한으로 conflict를 해소했다.

- conflict 해소 commit: `49d9e858c2108c76aee8f89ea2d2d7addaa10ba5`
- push 대상: `planet6897/rhwp:pr/devel-1728`
- push 후 상태: `MERGEABLE`, GitHub Actions 통과

## 코드 리뷰 결과

Blocking finding 없음.

footer 쪽번호 y 변경은 `page_number_baseline_y`와 `build_page_number` 경로 모두 같은 helper를 통하도록 유지되어 있다.
`margin_footer == margin_bottom`인 aift 계열은 위치가 변하지 않고, `margin_footer != margin_bottom`인 문서에서만 y가
달라지는 구조다.

RowBreak continuation spacing 보정은 `table_partial.rs`에서 `layout_composed_paragraph` 호출 직전 set, 호출 직후 reset
되는 좁은 scope다. 조건도 `cut_units.su > 0`, 첫 가시 문단, `start_line == 0`, 1x1 셀 제외로 제한되어 있어 기존
1x1 container/textbox 회귀를 피하는 형태다.

## 로컬 검증

새 PR review 지침에 따라 conflict 해소 검증 전 `target` 하위 항목을 삭제했다.

- `cargo fmt --all -- --check`: 통과, real 2.44s
- `env CARGO_INCREMENTAL=0 cargo test --lib test_634_aift`: 5 passed, real 37.53s
- `env CARGO_INCREMENTAL=0 cargo test --test issue_1692 issue_1692_so_sueop_hwpx_title_ole_renders_from_embedded_preview`: 1 passed, real 15.63s
- `env CARGO_INCREMENTAL=0 cargo test --test svg_snapshot form_002_page_0`: 1 passed, real 1.14s
- `env CARGO_INCREMENTAL=0 cargo test --test issue_rowbreak_chart_overlap`: 20 passed, real 2.05s
- `env CARGO_INCREMENTAL=0 cargo test --test issue_874_ktx_toc_page_number_right_align`: 1 passed, real 0.89s
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과, real 26.91s
- `git diff --check`: 통과
- `git diff --cached --check`: 통과

## GitHub Actions

문서 작성 시점 PR head `49d9e858c2108c76aee8f89ea2d2d7addaa10ba5` 기준:

- CI preflight: success
- CodeQL preflight: success
- Render Diff preflight: success
- WASM Build: skipped
- Analyze (javascript-typescript): success
- Analyze (python): success
- Canvas visual diff: success
- CodeQL: success
- Analyze (rust): success
- Build & Test: success

GitHub Actions 완료 후 `MERGEABLE` + `CLEAN` 상태를 확인했다.

## 범위 판단

이 PR은 #1728 전체 close 대상은 아니다. PR 문서와 이슈 본문 기준으로 다음 갈래가 남는다.

- scattered header RowBreak 6쪽 하단 over-fill / 행 컷 차이
- PDF 하단 rule 라인 계열 잔여 차이

따라서 merge 후에도 #1728은 open 유지하고, 이번 PR이 footer 위치와 giant continuation 상단 spacing 갈래를 부분 해결한
것으로 기록하는 편이 안전하다.

## 최종 처리

최신 head 기준 GitHub Actions가 통과했고 `MERGEABLE` + `CLEAN` 상태를 확인한 뒤 admin merge 완료.

#1728은 닫지 않고 open 유지했다. PR/issue 후속 코멘트에는 이번 PR의 부분 해결 범위와 남은 갈래를 남겼다.
