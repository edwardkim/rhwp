# PR #2119 리뷰 — 셀 저장 줄높이 신뢰

- 작성 시각: 2026-07-10 KST
- PR: https://github.com/edwardkim/rhwp/pull/2119
- 작성자: `planet6897`
- base / head: `devel` / `fix/2112-stored-line-height-trust`
- 문서 작성 시점 참고 head: `99737966573174ed689159d87b9054e1db3e24cb`
- 문서 작성 시점 참고 mergeable: `MERGEABLE`
- 처리 경로: `codex/planet6897-cherrypick-20260710` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- 실제 저장 LINE_SEG를 보유한 셀 문단은 raw 줄높이를 신뢰하도록 측정 경로를 수정한다.
- `corrected_line_height` 보정은 lineseg 부재 또는 synthetic lineSeg 폴백 목적에만 유지한다.
- `src/renderer/height_measurer.rs`, `src/renderer/layout/table_layout.rs`의 표 측정 경로를 갱신한다.

## 체리픽 검토

- 누적 체리픽 순서: 3/4.
- 중복 선행 커밋: `e6165a9d82e6cf0f2f74073af2ce9fc9a665070c`는 현재 기준에서 empty cherry-pick으로 판정되어 skip했다.
- 적용 커밋: `a2d07df2c` (`e11455ae25095ef00b4ffe9ee32ff44ea89f3414`에서 `-x` 체리픽).
- 충돌: 없음.
- 선행 PR 의존: #1842 synthetic line-height 계열은 현재 `devel`에 이미 존재한다.

## 검증

- 원 PR GitHub Actions: 문서 작성 시점 기준 `CI`, `CodeQL`, `Render Diff` 계열 check 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통과.
- `cargo fmt --check`: 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --test issue_2105_rowbreak_table_declared_fits --test issue_2097_3080901_real_doc_pin --test issue_2097_rowbreak_midpage_declared_fits --test issue_1842`: 통과. `issue_1842` 2 tests passed.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과(exit 0). `tests/issue_1842.rs`도 release-test에서 통과.

## 판단

- 체리픽 통합 가능.
- 렌더 경로가 아니라 측정 경로 보정 범위를 좁히는 변경이며, #1842 보호 테스트와 전체 release-test를 통과했다.
- 원 PR은 통합 PR이 merge된 뒤 supersede close/comment 처리 대상이다.
