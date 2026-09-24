# #7333 Stage 13 — PR 및 Studio 선 선택 검증

## 완료

- PR [#7377](https://github.com/edwardkim/rhwp/pull/7377)을 현재 integration head
  `3a78dbe0c2322ef54e0a648ce0e5ff55f3cb4d8b`로 생성했다.
- 8쪽 화살표는 넓은 도형 bbox보다 실제 line path를 먼저 판정한다. 클릭은 z-order 변경과 Undo 기록을 만들지 않으며,
  표 안 line도 `cellPath`·쪽 소유를 보존한다.
- `npm run e2e:issue-7333-line`은 실제 클릭, 마우스 드래그, Ctrl+Z, Command+Z를 검사해 통과했다.
- PR 본문에는 Native p19/p41과 fresh WASM p14의 review·overlay를 현재 head raw URL로 직접 표시했다.

## 다음 조건

PR의 최신 head CI가 통과하고 작업지시자 승인을 받은 뒤 merge한다. merge 뒤에는 실제 merge SHA와 CI URL을
사용해 asset 여섯 장을 표시하는 contributor PR comment를 게시하고, #7375 잔여 곡률 기능을 함께 안내한다.
