# PR #1717 리뷰 — #1716 반복 제목행 overhead 누적 폭주 수정

- PR: #1717 `Task #1716: 반복 제목행 overhead 를 상단 연속 제목행 블록으로 제한 (페이지당 1행 폭주 수정)`
- 작성자: @planet6897
- 기준: `devel`
- 검토 대상 head: `3c449d3935b0d2e3b79f3c9367656e313712cb85` (문서 작성 시점 참고값)
- 규모: 13 files, +406/-23
- 관련 이슈: #1716
- 문서 작성 시점 상태: `MERGEABLE`, `mergeStateStatus=BLOCKED` (`Build & Test` 진행 중)
- 처리 결과: 2026-07-01 `5e3b1ec652fda14a74af7cf9afd77962e3bb7903` merge 완료
- 후속 처리: #1716 수동 close 완료, PR 감사 코멘트 완료

## 변경 요약

다수 행에 `header="1"`이 흩어진 RowBreak 표에서 반복 제목행 overhead가 cursor 전진마다 누적되어
페이지당 1행만 배치되는 폭주를 정정한다. 대표 샘플은 173쪽에서 53쪽으로 줄어 한글 52쪽에 근접한다.

핵심 변경은 반복 제목행을 `is_header` 전체가 아니라 표 상단부터 연속된 제목행 블록으로 제한하는 것이다.

- `src/model/table.rs`: `Table::leading_header_rows()` 추가
- `src/model/table/tests.rs`: 상단 연속 제목행, 흩어진 본문 header, rowspan header 단위테스트 추가
- `src/renderer/typeset.rs`: `header_overhead` 계산을 `leading_header_rows()` 기반으로 변경
- `src/renderer/layout/table_partial.rs`: 렌더러 반복 제목행도 같은 helper 사용
- `samples/task1716/`: 재현 샘플과 README 추가

## 로컬 검증

임시 worktree `/private/tmp/rhwp-pr1717-review`에서 PR head를 가져와 검증했다.

- `git merge upstream/devel --no-commit --no-ff`: 충돌 없음 (`Already up to date`)
- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 항목 삭제
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo build --profile release-test --bin rhwp`: 통과
- `/Users/tsjang/rhwp/target/release-test/rhwp dump-pages samples/task1716/table_scattered_header_rowbreak.hwpx`: 53 pages 확인
- 같은 dump에서 `pi=12` RowBreak 표 분할 확인:
  - page 10: rows `0..6`
  - page 11: rows `6..40`
  - page 12: rows `40..86`
  - page 13: rows `86..132`
  - page 14: rows `132..176`
  - page 15: rows `176..183`
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --lib leading_header_rows -- --nocapture`: 4 passed
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo test --profile release-test --test issue_rowbreak_chart_overlap -- --nocapture`: 20 passed
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/tsjang/rhwp/target cargo fmt --check`: 통과
- `git diff --check upstream/devel...HEAD`: 통과

## GitHub Actions

최신 head `3c449d3935b0d2e3b79f3c9367656e313712cb85` 기준 문서 작성 시점:

- CI preflight: success
- CodeQL preflight: success
- Render Diff preflight: success
- CodeQL: success
- Analyze (javascript-typescript): success
- Analyze (python): success
- Analyze (rust): success
- Canvas visual diff: success
- WASM Build: skipped
- Build & Test: success

최종 merge 전 최신 head 기준 GitHub Actions 통과를 확인했다. `Build & Test`는 23분 1초 소요됐다.

## 리뷰 결과

Blocking finding 없음.

페이지네이터와 렌더러가 같은 `leading_header_rows()` helper를 사용하므로 반복 제목행 계산의 양쪽 정합이
맞는다. 본문 중간의 흩어진 `is_header` 행을 반복 제목행에서 제외하면서도, 상단 연속 다중 제목행과
rowspan 제목행은 단위테스트로 보존했다.

## 리스크 / 후속 확인

- `closingIssuesReferences`가 비어 있어 #1716 auto-close가 실패했다. merge 후 수동 close 처리했다.
- 최신 GitHub Actions `Build & Test`까지 통과 확인 후 merge했다.
- 임시 worktree `/private/tmp/rhwp-pr1717-review`는 작업지시자 승인 후 제거 완료했다.

## 최종 판단

수용 및 merge 완료.

- PR merge: https://github.com/edwardkim/rhwp/pull/1717
- merge commit: `5e3b1ec652fda14a74af7cf9afd77962e3bb7903`
- #1716 close comment: https://github.com/edwardkim/rhwp/issues/1716#issuecomment-4852331846
- PR 후속 comment: https://github.com/edwardkim/rhwp/pull/1717#issuecomment-4852334183
