# PR #6065 검토 기록

- 대상: <https://github.com/edwardkim/rhwp/pull/6065>
- 작성일: 2026-08-25
- 원 PR head: `1ec411755e48`
- 통합 검토 branch: `review/open-ci-green-20260825`
- 최신 기준선: `upstream/devel@898e75930a6c`
- 통합 code candidate: `1748b5cf33cb`

## 변경 요약

양쪽 정렬 줄에서 PUA 책괄호 측정이 반각으로 처리되어 다음 글자가 겹치는 문제를 보정한다.

## 코멘트 검토

contributor가 CI Lint 실패 원인을 `src/renderer/composer.rs` source-side unit test 증가로 설명했고,
테스트를 `tests/cases/issue_6057_justify_mixed_hangul_digit_overlap.rs`로 이동했다고 기록했다. 이후
Archive C의 IR field sweep 신규 fixture 발산 3건을 baseline으로 기록한 후 최신 CI가 성공했다. Codex usage
limit 자동 코멘트는 차단 사유가 아니다.

## 통합 적용

세 번째 commit의 IR field sweep baseline 충돌은 기존 rows와 issue 6057의 3 rows를 모두 보존했다.

## 로컬 검증

- 전체 Rust nextest 통합 검증 `8350 passed, 43 skipped`.
- `node scripts/rust-unit-test-tiers.mjs --check` 통과, 4221 tests.
- manifest prepare/check 통과.
- WASM locked build 통과.

## 판정

수용 가능. 통합 후보에 포함한다.
