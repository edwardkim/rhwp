# PR #3280 v2 통합 보정 계획

## 적용 범위

- contributor source head: `11a2edddb764c6dd101be5cf1c0450ff79d06b66` (원 PR에는 추가 push하지 않음)
- v2 적용: `42fe6b6b7`(다중 입력), `90c8e5a7c`(table control·containerPath 역참조)
- #3258·#3262·#3276·#3282 feature를 v2에 누적한 뒤 처리한다.

## 순서

1. v2에서 두 code/test 보정과 review 기록을 함께 유지한다.
2. `table_extract_json_contract`와 full CI를 통합 head에서 확인한다.
3. merge 전 JSON schema additive 변경과 기존 consumer 호환성을 재검토한다. 필요 시 두 v2 보정 commit만 역순 revert한다.
