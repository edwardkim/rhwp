# PR #3288 collaborator 보정 실행 계획

## 적용 범위

- contributor source head: `e26c9564191ba8e6bcbabc395ad8c2874da241e2`
- 보정 예정: `0ee48afa5` — `export-svg --json` 실패 stdout 0-byte 계약과 help 노출

## 순서

1. source head 3-way 일치를 확인하고 `review/pr3288-maintainer`에서만 보정을 commit한다.
2. `render_manifest_json_contract`와 latest full CI를 통과시킨다. review/오늘할일은 별도 docs commit으로 추가한다.
3. 사용자 승인 뒤 merge한다. 이후 #3264가 최종 command/MCP manifest를 반영하도록 Update branch 및 보정을 처리한다.
