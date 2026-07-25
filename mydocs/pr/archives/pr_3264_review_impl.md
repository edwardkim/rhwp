# PR #3264 v2 통합 보정 계획

## 적용 범위

- contributor source head: `d8b8fee39149926be6338f65b9c582c5921aebe7` (원 PR에는 추가 push하지 않음)
- 선행: v2에 #3258, #3262, #3276, #3282, #3280, #3285, #3288 feature 누적
- v2 보정: `31e3d5c36` — JSON command와 MCP tool inventory 정합

## 순서

1. 모든 선행 command feature가 v2에 있는 상태에서 manifest 보정을 유지한다.
2. review docs는 v2에 두고 오늘할일은 통합 PR 최종 묶음에서만 추가한다.
3. latest full CI에서 command/MCP coverage를 확인한 뒤 사용자 승인으로 merge한다. rollback은 v2 보정 commit만 revert한다.
