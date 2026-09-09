# PR #6883/#6909 누적 검토 및 통합 계획

## 기준과 적용

- 기준 branch: `upstream/devel` `ad84192839eb7b8534715ab085dd91d69a5c4a38`
- 검토 branch: `review/6883-6909-20260909`
- 적용 순서: #6883 `0e862ed` -> #6909 `b0ab0ea`
- 누적 head: `a6d4bdcf10a834cacfed0ba59e52714c168a4dd4`
- 두 cherry-pick은 충돌 없이 적용했고 `git diff --check`를 통과했다.
- 원 PR 파일별 검토 anchor는 [#6883 review](pr_6883_review.md#원-pr-변경-링크)와 [#6909 review](pr_6909_review.md#원-pr-변경-링크)에 고정한다.

## 수행 결과

- #6883과 #6909에 `jangster77` reviewer를 요청했다.
- Rust lint, targeted regression, security corpus, 전체 release-test, Native Skia를 누적 head에서 성공시켰다.
- 전체 release-test 결과는 9,334 passed, 46 skipped다.
- 새 HWP5 fixture는 Hancom Office 2020으로 직접 PDF 변환했고 2쪽 스윕에서 구조 이상 플래그가 없었다.
- Docker가 설치되어 있지 않아 compose wrapper는 미실행이지만, lint 묶음의 `wasm32-unknown-unknown` Clippy는 통과했다. Docker 부재는 로컬 수용 판단의 차단 사유가 아니다. object visual regression은 불필요한 중간 산출물 생성을 피하기 위해 기준 브랜치 빌드 중 중단했다.

## 자산 범위

- 최종 기준 PDF `pdf/synth_cell_enter_table_growth-2020.pdf`와 최종 review PNG 두 장 `mydocs/pr/assets/pr_6883_6909_20260909/`은 장기 재현과 PR 판독에 필요한 증적으로 포함한다.
- `pdf/pr_6883_6909_20260909/` 아래의 원시 raster, 중복 compare·overlay·review, contact sheet, export SVG, render-tree·metric JSON, MCP 응답 JSON은 중간 산출물이며 stage, commit, PR 첨부 대상이 아니다.
- 각 원 PR review 문서, 통합 계획, 오늘 할 일, 위 최종 증적만 후속 commit 후보로 둔다.

## 다음 단계

1. 이 세 문서와 오늘 할 일을 문서 전용 commit으로 고정한다.
2. 사용자 승인 후 `review/6883-6909-20260909`를 원본 저장소 branch로 push하고 `devel` 대상 통합 PR을 만든다.
3. 통합 PR의 최신 head CI에서 `Build & Test`, lint/WASM, Native Skia를 확인한다.
4. CI 녹색 뒤 사용자의 병합 승인에 따라 병합하고, 별도 `post_merge.md` 절차로 merge SHA와 comment 계획을 기록한다.

## Merge 후 확정 기록

- 통합 PR [#6944](https://github.com/edwardkim/rhwp/pull/6944)는 `2026-09-09T08:54:54Z`에 merge되었으며 merge commit은 [`74d0a68b74919761cc30343f7511dbc5d0fe32d3`](https://github.com/edwardkim/rhwp/commit/74d0a68b74919761cc30343f7511dbc5d0fe32d3)이다.
- 병합 직전 `44b2c1def8dc43ff4bd4dfcce7264dfa725eb8a5`의 필수 CI가 모두 green이었고, merge SHA가 `upstream/devel`의 조상임을 확인했다.
- `pdf/synth_cell_enter_table_growth-2020.pdf`와 `mydocs/pr/assets/pr_6883_6909_20260909/`의 대표 PNG 두 장만 영구 증적으로 보존한다. 원시 sweep 출력은 PR 범위 밖 중간 산출물이므로 제외한다.
- 원 PR #6883/#6909에는 #6944 수용 사실을 게시한 뒤 superseded로 close한다. 이슈 #6882는 close하고, #6806은 잔여 범위 때문에 OPEN으로 유지한다.

## Merge 후 contributor PR comment 계획

- #6883: 통합 merge SHA, green CI, Hancom 2020 기준 PDF와 대표 PNG 링크, fixture 범위를 게시한다.
- #6909: 통합 merge SHA, green CI와 focused test 3/3, #6806 OPEN 유지 사유를 게시한다.
- #6882: 해결 근거를 게시한 뒤 close한다. #6806: 이번 하위 범위만 해결됐음을 게시하고 OPEN 상태를 유지한다.
