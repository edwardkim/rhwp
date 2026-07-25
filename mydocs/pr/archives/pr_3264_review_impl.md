# PR #3264 collaborator 보정 실행 계획

## 적용 범위

- contributor source head: `d8b8fee39149926be6338f65b9c582c5921aebe7`
- 선행: #3258, #3262, #3276, #3282, #3280, #3285, #3288 merge
- 보정 예정: `31e3d5c36` — JSON command와 MCP tool inventory 정합

## 순서

1. 모든 선행 command PR merge 후 Update branch가 필요하면 사용자 승인 뒤 최신 devel을 반영한다.
2. source head 3-way 일치 확인 후 `review/pr3264-maintainer`에서 보정 code/test와 review/오늘할일 docs를 분리한다.
3. latest full CI에서 command/MCP coverage를 확인한 뒤 사용자 승인으로 merge한다. rollback은 보정 commit만 revert한다.
