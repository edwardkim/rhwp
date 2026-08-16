# task_m100_4947 stage 17: PR #4970 병합 후 테스트 기준선 재계산

## 배경

PR #4970이 `upstream/devel`에 병합된 뒤 GitHub의 PR 합성 merge ref에는
`tests/issue_3893_bindata_sequence_ref.rs`가 포함되었지만, PR #4976 브랜치의 test suite
manifest에는 아직 등록되지 않아 lint가 실패했다.

## 기준선 동기화

- 기준 `upstream/devel`: `b9efb7c203bc533897a6b627896927b48eb92d8c`
- 리베이스: 충돌 없이 완료
- 신규 source: `tests/issue_3893_bindata_sequence_ref.rs`
- 자동 배정: `regression_suite_027`

## 재계산 결과

- integration source: 565
- 정적 integration `#[test]`: 2,478
- generated suite: 32
- 단독 예외 target: 9
- Cargo integration target: 41 / 48
- source unit `#[test]`: 4,225
- source unit module: 298
- nextest 전체: 6,559
- `minimumNextestCases`: 6,559로 상향

## 검증

- manifest 현재 상태 및 `upstream/devel` 기준 검사
- manifest 단위 계약
- CI 아카이브 계획기와 독립 느린 target 배치
- 전체 nextest 회귀 테스트
- rustfmt 및 diff whitespace 검사

검증 결과:

- integration manifest 현재 상태 및 `upstream/devel` 기준 검사 통과
- source-unit tier 현재 상태 및 `upstream/devel` 기준 검사 통과
- manifest 단위 계약 9건, source-unit tier 단위 계약 11건 통과
- CI 아카이브 계획기에서 느린 독립 target 1개 배치 확인
- 전체 nextest: 6,521건 통과, 38건 제외, 실패 0건(3분 43.75초)
- rustfmt 및 diff whitespace 검사 통과
