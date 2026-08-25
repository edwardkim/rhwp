# PR #5970 검토 기록

- 대상: <https://github.com/edwardkim/rhwp/pull/5970>
- 작성일: 2026-08-25
- 원 PR head: `9e9966dec822`
- 통합 검토 branch: `review/open-ci-green-20260825`
- 최신 기준선: `upstream/devel@898e75930a6c`
- 통합 code candidate: `1748b5cf33cb`

## 변경 요약

HWPX 저장 line segment의 좌표 축을 읽기와 저장 양쪽에서 HWP5 축과 정합하도록 보정하고, #5961 회귀
테스트를 `tests/cases`로 옮긴다.

## 코멘트 검토

이전 maintainer 코멘트는 Archive B/C 실패와 HWP roundtrip 축 혼선을 이유로 보류했다. 이후 contributor가
원인을 두 차례 설명하고 최신 `devel`을 반영해 destination 축 보정과 양방향 교차 비교를 수정했다. 최신
head CI는 실패 없이 완료됐다.

## 통합 적용

첫 #5943 계열 commit은 최신 `devel`에 이미 반영된 내용과 patch-equivalent라 cherry-pick에서 skip했다.
#5961 고유 보정 commit들은 통합 branch에 적용했다.

## 로컬 검증

- 전체 Rust nextest 통합 검증 `8350 passed, 43 skipped`.
- `cargo fmt --all -- --check`, `git diff --check` 통과.
- manifest prepare/check와 unit-tier check 통과.
- WASM locked build 통과.

## 판정

수용 가능. 중복 적용 없이 통합 후보에 포함한다.
