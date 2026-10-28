# task_m100_2259_impl — 구현 계획서

- **이슈**: [#2259](https://github.com/edwardkim/rhwp/issues/2259)
- **수행계획서**: [`task_m100_2259.md`](task_m100_2259.md)
- **브랜치**: `local/task2259`

수행계획서 3장(설계)을 4단계로 나눈다. 각 단계는 `npm run compile` 통과를 완료 조건에 포함하며, 단계마다 완료보고서(`working/task_m100_2259_stage{N}.md`)를 소스와 함께 커밋한다.

---

## 1단계 — 배율 상태 모델 분리 (UI 변경 없음)

**목표**: `currentZoom` 단일 숫자를 `zoomMode` + 계산된 배율로 분리한다. 이 단계에서는 화면 동작이 기존과 동일해야 한다(순수 리팩터링 + 계산 함수 추가).

`src/webview/viewer.ts`

1. 상태 추가: `type ZoomMode = "manual" | "fitWidth" | "fitPage"`, `let zoomMode: ZoomMode = "manual"`.
2. 상수 추가: `SIDE_MARGIN`, `ROW_GAP = 12` (CSS 값과 일치).
3. `computeFitZoom(mode: "fitWidth" | "fitPage"): number` 추가 — 수행계획서 3.4 식.
   - 기준 쪽 크기는 `pageInfos` 전체의 최대 폭 / 최대 높이.
   - `pagesPerRow` 는 `viewMode` 에서 도출.
   - `ZOOM_MIN`~`ZOOM_MAX` 클램프.
4. `applyZoom(newZoom, anchorY?)` → 내부를 `setZoom(newZoom, anchorY?)` 로 두고, 진입점(`[−]/[+]`, Ctrl+휠)에서 `zoomMode = "manual"` 을 설정하도록 분리.
5. `applyZoomMode(mode: ZoomMode, zoom?: number)` 추가 — 맞춤 모드면 `computeFitZoom()` 결과를, 수동이면 인자 배율을 적용.

**완료 조건**: 컴파일 통과. 기존 `[−]/[+]`, Ctrl+휠, 1쪽/2쪽 토글 동작이 그대로 유지된다(맞춤 진입점이 아직 없으므로 `zoomMode` 는 항상 `manual`).

**주의**: `applyZoom` 의 기존 `if (newZoom === currentZoom) return;` 조기 반환은 맞춤 모드 전환 시 배율이 우연히 같을 때 레이아웃 재구성을 건너뛰게 만든다. 배율 동일 + 배치(`viewMode`) 변경 케이스를 별도로 다뤄야 한다.

---

## 2단계 — 통합 배율 드롭다운 UI

**목표**: 상태 표시줄 마크업/스타일 교체. 메뉴 항목 → 상태 매핑 연결.

`src/hwp-editor-provider.ts`

1. `.stb-right` 에서 `stb-view-mode` 버튼 제거.
2. `stb-zoom-val` 을 `<button id="stb-zoom-menu">` 드롭다운 트리거로 교체 (`100% ▾`).
3. 드롭다운 마크업 추가: 항목 5+5 (맞춤 3 + 구분선 + % 프리셋 5), 각 항목에 `data-action`.
4. CSS 추가 (`stb-` 접두어 준수): 상태 표시줄 위로 펼쳐지는 popup, 체크 표시(`✓`) 슬롯, hover/selected 스타일. VS Code 테마 변수(`--vscode-menu-*`) 사용.

`src/webview/viewer.ts`

5. 드롭다운 열기/닫기: 트리거 클릭 토글, 바깥 클릭 / `Esc` 로 닫기.
6. 항목 클릭 → 수행계획서 3.3 매핑대로 `viewMode` / `zoomMode` 설정 후 `applyZoomMode()` 호출.
7. `updateStatusBar()` — 트리거 라벨에 실제 배율 %, 현재 선택 항목에 체크 표시 갱신.

**완료 조건**: 컴파일 통과. 메뉴에서 폭 맞춤 / 쪽 맞춤 / 두 쪽 맞춤 / % 선택이 모두 동작하고, 2쪽 보기 선택 시 가로 스크롤 없이 두 쪽이 들어온다 (요구사항 1·2·3).

---

## 3단계 — 창 크기 변화 대응

**목표**: 요구사항 4 — 뷰포트 크기가 바뀌면 맞춤 배율을 재계산한다.

`src/webview/viewer.ts`

1. `scrollContainer` 에 `ResizeObserver` 부착.
2. 콜백: `zoomMode === "manual"` 이면 무시. 아니면 `contentRect` 기준으로 `computeFitZoom()` 재계산.
3. 히스테리시스: 새 배율이 현재 배율과 1% 미만 차이면 무시 (스크롤바 진동 방지).
4. 재계산 적용 시 현재 쪽 위치 유지 (`scrollToPage(currentPage)`).
5. `requestAnimationFrame` 으로 연속 리사이즈 합치기.

**완료 조건**: 컴파일 통과. 에디터 패널 폭/높이 변경, 창 리사이즈, 사이드바 접기/펼치기에서 맞춤 배율(%)이 따라 바뀌고 진동이 없다.

---

## 4단계 — 검증 및 마무리

1. 수행계획서 4장 수동 검증 시나리오 1~7 전부 수행 (F5 확장 개발 호스트).
   - 세로/가로 문서, 쪽 크기가 섞인 문서 포함.
2. `npm run compile` 무경고 확인.
3. `rhwp-vscode/CHANGELOG.md` 갱신.
4. 최종 결과보고서 `mydocs/report/task_m100_2259_report.md` 작성.

**완료 조건**: 전 시나리오 통과. 미해결 항목이 있으면 보고서에 명시.

---

## 커밋 계획

| 단계 | 커밋 |
|------|------|
| 1 | `Task #2259: 배율 상태 모델 분리 (zoomMode + 맞춤 배율 계산)` + stage1 보고서 |
| 2 | `Task #2259: 상태 표시줄 통합 배율 드롭다운` + stage2 보고서 |
| 3 | `Task #2259: 맞춤 배율 뷰포트 리사이즈 대응` + stage3 보고서 |
| 4 | `Task #2259: CHANGELOG 및 최종 보고서` |

머지 전 `git status` 로 미커밋 파일이 없는지 확인한다. PR 은 `planet6897:pr-task2259` → `edwardkim:devel` 로 별도 브랜치를 push 하여 생성한다.

---

**승인 요청**: 위 4단계 분할로 진행해도 될지 확인 부탁드립니다. 승인 시 1단계부터 착수합니다.
