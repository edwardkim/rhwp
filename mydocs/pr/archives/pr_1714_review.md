# PR #1714 리뷰 — #1706 표 직후 빈 문단 rhwp_pNone 수정

- PR: #1714 `Task #1706: 표 직후 빈 문단 누락(rhwp_pNone) 수정 — drop 대신 0-높이 흡수`
- 작성자: @planet6897
- 기준: `devel`
- 검토 대상 head: `352713dce4c4d97cbbefb7965f60dc049412dccd` (문서 작성 시점 참고값)
- 규모: 5 files, +97/-2
- 관련 이슈: #1706
- 처리 결과: merge 완료, merge commit `1314b90f0e51fb38464bfd9216a8c615eec72be8`
- 관련 이슈 처리: #1706 수동 close 완료 (`2026-07-01T07:22:43Z`)

## 변경 요약

블록 표(`treat_as_char`/TopAndBottom) 직후 또는 쪽나누기 직전의 빈 문단이 `TypesetEngine`에서
fit 실패 시 `continue`로 드롭되어 `dump-pages` 문단→페이지 매핑에서 `rhwp_pNone`으로 남던 문제를
수정한다.

핵심 변경은 `src/renderer/typeset.rs`의 빈 문단 fit 실패 분기 2곳이다.

- `next_will_vpos_reset` 경로: `!(height_fits && vpos_fits)`일 때 빈 문단을 드롭하지 않음
- Task #967 force-break 경로: `empty_h_px > avail`일 때 빈 문단을 드롭하지 않음

두 경로 모두 `hidden_empty_paras`에 문단을 등록하고 `PageItem::FullParagraph`를 현재 페이지에 추가한다.
`layout.rs`의 기존 hidden empty paragraph 처리에 의해 높이 증가는 0으로 유지되므로, 페이지 advance 없이
문단 매핑만 보존하는 방향이다.

## 변경 범위

- 코드: `src/renderer/typeset.rs`
- 검증 샘플:
  - `samples/task1706/empty_para_between_tac_tables.hwp`
  - `samples/task1706/empty_para_before_pagebreak.hwpx`
  - `samples/task1706/README.md`
- 보고서: `mydocs/report/task_m100_1706_report.md`

## 로컬 검증

임시 worktree `/tmp/rhwp-pr1714-review`에서 PR head를 가져와 검증했다.

- `git merge upstream/devel --no-commit --no-ff`: 충돌 없음 (`Already up to date`)
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo build --profile release-test --bin rhwp`: 통과
- `rhwp dump-pages samples/task1706/empty_para_between_tac_tables.hwp`: 2 pages, `FullParagraph pi=3 "(빈)"` 확인
- `rhwp dump-pages samples/task1706/empty_para_before_pagebreak.hwpx`: 3 pages, `FullParagraph pi=3 "(빈)"` 확인
- 관련 회귀 subset:
  - `issue_1488_rowbreak_empty_overlay_pages`
  - `issue_1549`
  - `issue_676_trailing_empty_para`
  - `issue_703`
  - `issue_1070_tac_table_post_text_overflow`
  - `issue_rowbreak_chart_overlap`
  - 결과: 32 passed
- `cargo fmt --check`: 통과
- `git diff --check upstream/devel...HEAD`: 통과
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --tests`: 통과
  - `real 334.90`

## GitHub Actions

PR head `352713dce4c4d97cbbefb7965f60dc049412dccd` 기준:

- CI preflight: success
- Render Diff preflight: success
- Canvas visual diff: success
- CodeQL preflight / Analyze: success
- Build & Test: success (`2026-07-01T07:20:44Z`)

## 리뷰 결과

Blocking finding 없음.

코드 변경은 문제 경로 2곳에 국한되어 있고, 기존 hidden empty paragraph 레이아웃 시맨틱을 재사용한다.
동봉 샘플 2건에서 `pi=3` 빈 문단이 페이지 매핑에 복구되는 것을 확인했으며, 관련 빈 문단·TAC·rowbreak
회귀 subset과 전체 integration test도 통과했다.

## 리스크 / 후속 확인

- PR에 Rust 자동 회귀 테스트 파일은 추가되지 않았다. 대신 실제 HWP/HWPX 샘플과 로컬 `dump-pages`
  검증으로 동작을 확인했다.
- `gh pr view --json closingIssuesReferences`에서는 #1706 auto-close 참조가 비어 있었다. PR 본문과 커밋에는
  `closes #1706`가 있었지만 merge 후에도 이슈가 open 상태라 수동 close 처리했다.

## 최종 판단

수용 완료. GitHub `Build & Test`가 최신 head 기준 성공했고, 로컬 검증도 통과했으므로 #1714를 merge했다.

최종 처리 결과:

- PR #1714 merge 완료: `1314b90f0e51fb38464bfd9216a8c615eec72be8`
- merge 시각: `2026-07-01T07:22:13Z`
- PR 후속 코멘트: https://github.com/edwardkim/rhwp/pull/1714#issuecomment-4851436454
- #1706 auto-close 실패 확인 후 수동 close 완료
- #1706 close comment: https://github.com/edwardkim/rhwp/issues/1706#issuecomment-4851434898
