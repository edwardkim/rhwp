# PR #3288 v2 통합 보정 계획

## 적용 범위

- contributor source head: `e26c9564191ba8e6bcbabc395ad8c2874da241e2` (원 PR에는 추가 push하지 않음)
- v2 보정: `0ee48afa5` — `export-svg --json` 실패 stdout 0-byte 계약과 help 노출

## 순서

1. v2에서 보정과 review 기록을 함께 유지한다.
2. `render_manifest_json_contract`와 latest full CI를 통과시킨다. 오늘할일은 통합 PR 최종 묶음에서만 추가한다.
3. 사용자 승인 뒤 통합한다. #3264의 v2 보정이 최종 command/MCP manifest를 반영한다.
