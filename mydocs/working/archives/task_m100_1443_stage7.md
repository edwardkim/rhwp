# Task M100 #1443 Stage 7 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `b4b03be5 task 1443: 셀 모양복사 확장`

## 1. Stage 7 목표

표 전체가 객체로 선택된 상태에서 표 내부 셀을 다시 클릭하면 한컴처럼 해당 셀 편집 상태로 들어갈 수 있어야 한다.

현재 증상은 표 전체 선택 핸들이 표시된 상태에서 셀을 클릭해도 표 객체 선택 상태가 유지되거나 셀 안 커서가 활성화되지 않아,
사용자가 표 내부 편집으로 복귀할 수 없는 것이다.

## 2. 회귀 방지 기준

- 표 외곽선/핸들 클릭을 통한 표 객체 선택과 리사이즈 후보 처리는 유지한다.
- 표 객체 선택 상태에서 표 내부 셀 본문을 클릭하면 표 선택을 해제하고 클릭한 셀로 커서를 이동한다.
- 보호 셀은 기존처럼 셀 선택 상태로 진입하고 입력 차단을 유지한다.
- 마우스 드래그 셀 블록 선택, 셀 내부 텍스트 드래그 선택, 표 경계선 리사이즈를 깨지 않는다.

## 3. 구현 계획

- 표 객체 선택 상태에서 `mousedown` hit-test 결과가 같은 표 내부 셀을 가리키는지 확인한다.
- 내부 셀 클릭이면 현재 표 객체 선택을 해제하고 기존 셀 클릭/커서 이동 경로로 넘긴다.
- 핸들 또는 표 외곽선 클릭은 기존 표 객체 선택/리사이즈 경로를 유지한다.
- headless Chrome 실제 클릭 검증으로 표 선택 후 셀 진입 가능 여부를 확인한다.

## 4. 구현 결과

- 표 객체 선택 상태의 좌클릭 처리에서 클릭 지점이 선택된 표의 내부 셀인지 먼저 판정하도록 했다.
- 핸들 위 클릭과 표 외곽선 클릭은 기존 객체 선택 경로를 유지한다.
- 선택된 표 내부 셀 본문 클릭이면 표 객체 선택을 해제하고 아래쪽 일반 hit-test/커서 이동 경로로 넘긴다.
- 이로써 표 전체 선택 핸들이 보이는 상태에서 셀을 다시 클릭하면 해당 셀 편집 상태로 복귀한다.

## 5. 검증 결과

- `cd rhwp-studio && npx tsc --noEmit` 통과.
- Browser plugin 경로는 `Browser is not available: iab`로 연결되지 않아 headless Chrome/Puppeteer 검증으로 대체했다.
- `node /private/tmp/rhwp_1443_table_object_reentry_check.mjs` 통과.
  - 표 객체 선택 상태에서 `.table-object-layer` 핸들 9개 확인.
  - 선택된 표 내부 셀 중앙을 실제 마우스로 클릭.
  - 클릭 후 표 객체 선택 해제, 선택 핸들 제거, `cellIndex=11` 셀 커서 진입, caret 표시 확인.
- `node /private/tmp/rhwp_1443_cell_drag_check.mjs` 통과.
  - F5 없이 셀 드래그 선택, 선택 후 `Control+ArrowRight` 크기 조정, 같은 셀 텍스트 드래그 선택 유지 확인.
- `node /private/tmp/rhwp_1443_protected_cell_drag_check.mjs` 통과.
  - 보호 셀 클릭 후 드래그 선택 유지 확인.
- `cd rhwp-studio && npm test` 통과.
- `cd rhwp-studio && npm run build` 통과.
- `git diff --check` 통과.

최종 PR 준비 단계가 아니므로 `cargo clippy --all-targets -- -D warnings`는 수행하지 않았다.
