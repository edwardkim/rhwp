# PR #6047 검토 기록

- 대상: <https://github.com/edwardkim/rhwp/pull/6047>
- 작성일: 2026-08-25
- 원 PR head: `5fe19318fea2`
- 통합 검토 branch: `review/open-ci-green-20260825`
- 최신 기준선: `upstream/devel@898e75930a6c`
- 통합 code candidate: `1748b5cf33cb`

## 변경 요약

fallback 폭 분류에서 반각 형태를 전각 blanket보다 먼저 분류해 반각 낫표의 과전진을 줄인다.

## 코멘트 검토

PR comment와 review 요청 중 차단 사유는 확인되지 않았다. 최신 PR head CI는 실패 없이 완료됐다.

## 통합 적용

IR field sweep baseline 충돌은 #5941, #6023 rows를 모두 보존했다.

## 로컬 검증

- 전체 Rust nextest 통합 검증 `8350 passed, 43 skipped`.
- manifest prepare/check와 unit-tier check 통과.

## 판정

수용 가능. 통합 후보에 포함한다.
