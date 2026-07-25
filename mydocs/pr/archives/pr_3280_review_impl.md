# PR #3280 collaborator 보정 실행 계획

## 적용 범위

- contributor source head: `11a2edddb764c6dd101be5cf1c0450ff79d06b66`
- 적용 예정: `517895c6d`(다중 입력), `90c8e5a7c`(table control·containerPath 역참조)
- #3258·#3262·#3276·#3282를 먼저 merge해 최신 `devel` 기준으로 처리한다.

## 순서

1. source SHA 3-way 확인 후 `review/pr3280-maintainer`에 두 code/test commit을 순서대로 적용한다.
2. docs/오늘할일은 별도 commit으로 만들고, `table_extract_json_contract`와 full CI를 최신 head에서 확인한다.
3. merge 전 JSON schema additive 변경과 기존 consumer 호환성을 재검토한다. 필요 시 두 collaborator commit만 역순 revert한다.
