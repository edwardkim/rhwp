# Task M100 #1443 Stage 1 완료 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 작성일: 2026-06-19

## 1. Stage 1 목표

표 셀 선택 상태에서 마우스 드래그로 셀 블록 범위를 선택할 수 있게 한다.

## 2. 코드 수정 전 확인

- #1443 이슈 확인 완료
- 열린 PR 확인 완료
  - #1438, #1366, #1170 확인
  - 이번 Stage 1의 `rhwp-studio` 마우스 셀 선택 경로와 직접 충돌하는 열린 PR은 확인되지 않음
- #669 표 셀 내부 텍스트 드래그 선택 회귀 방지 필요 확인
- #493/#1442 보호 셀 클릭 선택과 입력 차단 유지 필요 확인
- #792 메뉴 hotkey 인프라와 #1140 F5 포커스/셀 크기 조정 이력 확인

## 3. 구현 내용

- `InputHandler`에 셀 블록 드래그 상태를 추가했다.
- 셀 선택 모드에서 일반 좌클릭 후 드래그하면 클릭 셀을 anchor로 잡고, 마우스가 지나간 셀을 focus로 갱신한다.
- 보호 셀 클릭 선택 후 곧바로 드래그할 수 있도록 #493의 보호 셀 선택 경로에 셀 드래그 후보 시작을 연결했다.
- `onMouseMove`에서 셀 블록 드래그 중인 경우 기존 `CellSelectionRenderer`로 선택 범위를 갱신한다.
- `onMouseUp`에서 셀 블록 드래그 상태와 document-level mousemove listener를 정리한다.
- `CursorState`에 `setCellSelectionAnchor`, `setCellSelectionFocus`를 추가해 마우스 드래그 의도를 명시했다.

## 4. 검증 결과

- `cd rhwp-studio && npm test` 통과
- `cd rhwp-studio && npx tsc --noEmit` 통과
- `cd rhwp-studio && npm run build` 통과
- `cargo fmt --check` 통과
- `git diff --check` 통과
- Puppeteer fallback 실제 브라우저 검증 통과
  - 대상: `http://localhost:7700`
  - 방식: headless Chrome + 실제 `page.mouse` 드래그
  - 결과: 4x4 표에서 0,0 셀부터 2,2 셀까지 드래그 후 선택 범위가
    `{ startRow: 0, startCol: 0, endRow: 2, endCol: 2 }`로 갱신됨
  - `.cell-selection-highlight` 9개 확인
  - 드래그 선택 후 `Control+ArrowRight` 입력 시 선택 셀 폭이 `10488 -> 10788`로 증가하고 인접 셀 폭이 `10488 -> 10188`로 보정됨 확인
  - 같은 표의 일반 셀 텍스트를 실제 마우스로 드래그했을 때 텍스트 선택 범위가 `charOffset 1 -> 17`로 유지되고 셀 선택 모드로 전환되지 않음 확인
  - relevant console/page error 없음
- Puppeteer fallback 보호 셀 클릭-드래그 검증 통과
  - 0,0 셀에 `cellProtect = true` 설정
  - F5 없이 보호 셀 클릭 후 2,2 셀까지 드래그
  - 선택 범위 `{ startRow: 0, startCol: 0, endRow: 2, endCol: 2 }` 확인
  - `.cell-selection-highlight` 9개 확인
  - relevant console/page error 없음

## 5. 남은 검증

in-app Browser 연결 시도 중 `iab` 세션을 사용할 수 없어 Codex Browser 경로는 사용하지 못했다.
대신 Puppeteer fallback으로 실제 브라우저 마우스 드래그 검증을 완료했다.

수동 검증 권장 대상:

- 표 경계선 리사이즈 회귀 없음
