# Stage 6 - #5036 insert-header-footer 파생 suite 제외

## 목표

Kevin 기여 PR #5036의 머리말·꼬리말 생성 CLI와 계약 테스트를 누적 후보에 적용한다.

## 충돌 보정

`agent_knowledge_map.md`의 전수 사전 카운트는 누적 명령 필드를 포함한 최신 값인 299개로 합쳤다. 기능 설명과 이전에 적용된 항목은 유지한다.

## 반영 범위

- `rhwp edit insert-header-footer` 구현과 CLI 문서
- `tests/cases/insert_header_footer_contract.rs` 및 관련 provenance 계약
- `tests/suites/unit-test-tiers.json`의 원본 tier 입력

## 파생 산출물 처리

`tests/generated/regression_suite_022.rs` 및 `tests/suites/manifest.json`은 기준 브랜치 상태로 복원했다. 최종 PR 검토/CI가 한 번 생성하는 결과이므로 커밋하지 않는다.

## 다음 단계

동일 stacked 계열의 다음 PR도 기능 커밋만 골라 누적하고, generated suite/manifest는 최종 검토 전까지 변경하지 않는다.
