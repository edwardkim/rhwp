# PR #6066 검토 기록

- 대상: <https://github.com/edwardkim/rhwp/pull/6066>
- 작성일: 2026-08-25
- 원 PR head: `384b32a98e2e`
- 통합 검토 branch: `review/open-ci-green-20260825`
- 최신 기준선: `upstream/devel@898e75930a6c`
- 통합 code candidate: `1748b5cf33cb`

## 변경 요약

쪽 분할 표 T&B 셀 그림을 잔여 쪽에 강제 소비하지 않도록 해 split cell picture 회귀를 줄인다.

## 코멘트 검토

Codex usage limit 자동 코멘트 외 차단 review 또는 maintainer 요청은 없다. 최신 PR head CI는 실패 없이
완료됐다.

## 로컬 검증

- 전체 Rust nextest 통합 검증 `8350 passed, 43 skipped`.
- `cargo fmt --all -- --check`, `git diff --check` 통과.

## 판정

수용 가능. 통합 후보에 포함한다.
