# Stage 5 - #5028 delete-table 파생 suite 제외

## 목표

Kevin 기여 PR #5028의 `rhwp edit delete-table` 기능과 원본 계약 테스트를 누적 후보에 반영한다.

## 반영 범위

- `src/main.rs` 및 CLI 문서: 본문 최상위 표 번호(`--table`)를 기준으로 표를 삭제하는 명령
- `tests/cases/delete_table_contract.rs`: 표 삭제 원본 회귀 계약
- `tests/suites/unit-test-tiers.json`: 원본 테스트 tier 입력

## 파생 산출물 처리

기여 커밋의 `tests/generated/regression_suite_026.rs` 및 `tests/suites/manifest.json` 변경은 기준 브랜치로 복원했다. 파생 suite는 최종 PR 검토/CI에서 한 번 생성하며, contributor·integration PR 커밋에는 포함하지 않는다.

## 다음 단계

다음 stacked head도 기능 커밋을 하나씩 분리 적용하고, 원본 `tests/cases/**`와 정책 입력만 유지한다.
