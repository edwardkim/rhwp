# PR #5997 검토 기록

- 대상: <https://github.com/edwardkim/rhwp/pull/5997>
- 작성일: 2026-08-25
- 원 PR head: `ec223cf2b065`
- 통합 검토 branch: `review/open-ci-green-20260825`
- 최신 기준선: `upstream/devel@898e75930a6c`
- 통합 code candidate: `1748b5cf33cb`

## 변경 요약

셀 안 중첩 표의 글자 캡션을 depth 조건과 무관하게 그려 2181727 7, 8쪽 정합을 보강한다.

## 코멘트 검토

초기 maintainer 코멘트는 overflow baseline과 issue 1891 회귀 때문에 보류했다. contributor 후속 코멘트에서
최신 `devel`로 rebase했고, #6001로 이미 들어간 overflow 한도와 중복되는 부분을 유지하지 않도록 정리했음을
확인했다. Codex usage limit 자동 코멘트는 검토 차단 사유가 아니다.

## 로컬 검증

- 전체 Rust nextest 통합 검증 `8350 passed, 43 skipped`.
- `cargo fmt --all -- --check`, `git diff --check` 통과.
- manifest prepare/check와 unit-tier check 통과.

## 판정

수용 가능. 통합 후보에 포함한다.
