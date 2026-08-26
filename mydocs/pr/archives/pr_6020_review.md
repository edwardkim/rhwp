# PR #6020 검토 기록

- 대상: <https://github.com/edwardkim/rhwp/pull/6020>
- 작성일: 2026-08-25
- 원 PR head: `0f4e8f127810`
- 통합 검토 branch: `review/open-ci-green-20260825`
- 최신 기준선: `upstream/devel@898e75930a6c`
- 통합 code candidate: `1748b5cf33cb`

## 변경 요약

조각 중간 행도 저장 프레임 끝 직전의 근소 cut을 frame까지 당겨 한 유닛 sliver로 쪽이 늘어나는 문제를
줄인다.

## 코멘트 검토

PR comment와 review 요청 중 차단 사유는 확인되지 않았다. 최신 PR head CI는 실패 없이 완료됐다.

## 로컬 검증

- 전체 Rust nextest 통합 검증 `8350 passed, 43 skipped`.
- `cargo fmt --all -- --check`, `git diff --check` 통과.
- manifest prepare/check와 unit-tier check 통과.

## 판정

수용 가능. 통합 후보에 포함한다.
