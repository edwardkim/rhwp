# PR #2345 검토 — 메뉴/도구상자 개체 조작 히스토리 기록 (#2343, 연작 3)

- PR: https://github.com/edwardkim/rhwp/pull/2345 (lpaiu-cs) — Fixes #2343
- 충돌 없음 (연작 1·2 merge 후에도 클린)

## 변경 본질

같은 개체 삭제라도 Delete 키는 기록되고 메뉴/도구상자는 안 되던 **진입점
비대칭** 해소 — `recordObjectMutation` 헬퍼로 8지점(z순서 4·삭제 4분기·
묶기/풀기·회전/대칭)을 Delete 키와 동형인 `executeOperation({kind:'snapshot'})`
에 위임. UI 후처리(선택 해제·afterEdit·재선택)는 호출부 유지 — 최소 diff.
소스 가드(원장의 '표면 증가' 한계를 보완하는 라우팅 텍스트 핀) 동반.

## 로컬 재실증 (merged tree)

| 게이트 | 결과 |
|--------|------|
| 신규 가드 테스트 | 3/3 |
| studio 단위 / tsc | **341/341** / 0 |
| e2e | undo-contracts 24/0 |

## 판단

**merge 권고.** snapshot 기록이라 #2332 예산 관리 하에 안전, 진입점 대칭성
회복. 브라우저 왕복 실측(undoLen 0→1·복원)은 컨트리뷰터 제공.
