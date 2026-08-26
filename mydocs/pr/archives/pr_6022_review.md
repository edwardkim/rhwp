# PR #6022 검토 기록

- 대상: <https://github.com/edwardkim/rhwp/pull/6022>
- 작성일: 2026-08-25
- 원 PR head: `285483b3a264`
- 통합 검토 branch: `review/open-ci-green-20260825`
- 최신 기준선: `upstream/devel@898e75930a6c`
- 통합 code candidate: `1748b5cf33cb`

## 변경 요약

TAC 표 중첩 표 셀의 저장 anchor가 담길 때 저장 흐름 기준으로 배치해 Studio/SVG 마지막 줄 소실을 줄인다.

## 코멘트 검토

PR comment와 review 요청 중 차단 사유는 확인되지 않았다. 최신 PR head CI는 실패 없이 완료됐다.

## 통합 적용

`tests/fixtures/ir_field_sweep_baseline.tsv` 충돌은 #5782 rows와 #5601 row를 모두 보존하고 중복 없이
정리했다.

## 로컬 검증

- 전체 Rust nextest 통합 검증 `8350 passed, 43 skipped`.
- manifest prepare/check와 unit-tier check 통과.

## 판정

수용 가능. 통합 후보에 포함한다.
