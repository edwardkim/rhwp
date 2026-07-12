# PR #2109 리뷰 — RowBreak 표 선언-fit 통째 배치

- 작성 시각: 2026-07-10 KST
- PR: https://github.com/edwardkim/rhwp/pull/2109
- 작성자: `planet6897`
- base / head: `devel` / `fix/2105-rowbreak-declared-fit`
- 문서 작성 시점 참고 head: `26c17381e17cbebb26103eb46ae6d5935550a5fa`
- 문서 작성 시점 참고 mergeable: `MERGEABLE`
- 처리 경로: `codex/planet6897-cherrypick-20260710` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- #2097의 None 표 선언 높이 신뢰 게이트를 RowBreak 쪽 상단 표까지 확장한다.
- RowBreak는 `current_height <= 0.5`인 쪽 상단 배치에 한정한다.
- 실측 초과가 `min(선언 높이 * 10%, 64px)` 이내일 때만 선언 높이를 신뢰한다.
- `samples/task2105/rowbreak_table_declared_fits.hwpx`와 회귀 테스트를 추가한다.

## 체리픽 검토

- 누적 체리픽 순서: 2/4.
- 중복 선행 커밋: `6b9e3fa9edc4735d148854f2b5bf0b3915f60ad5`는 현재 `devel`에 더 보강된 형태로 이미 반영되어 있어 skip했다.
- 적용 커밋: `ff2d937ca` (`1c1d2330796652c2e771690a598af10a945aa080`에서 `-x` 체리픽).
- 충돌: `src/renderer/typeset.rs`.
- 충돌 해소: 기존 #2097 None 전용 `declared_none_table_whole_fits`를 #2105의 `declared_table_whole_fits` 확장 게이트로 교체했다.
- 선행 PR 의존: #2097 None 표 선언-fit 계열은 현재 `devel`에 이미 존재한다.

## 검증

- 원 PR GitHub Actions: 문서 작성 시점 기준 `CI`, `CodeQL`, `Render Diff` 계열 check 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통과.
- `cargo fmt --check`: 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --test issue_2105_rowbreak_table_declared_fits --test issue_2097_3080901_real_doc_pin --test issue_2097_rowbreak_midpage_declared_fits --test issue_1842`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과(exit 0). `tests/issue_2105_rowbreak_table_declared_fits.rs`도 release-test에서 통과.

## 판단

- 체리픽 통합 가능.
- RowBreak 확대는 쪽 상단과 실측 드리프트 한도로 제한되어 있으며, 기존 #2097/#2093/#2015 계열 회귀 테스트를 통과했다.
- 원 PR은 통합 PR이 merge된 뒤 supersede close/comment 처리 대상이다.
