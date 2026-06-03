# Task M100 #661 Stage 3 완료 보고서

## 개요

- 이슈: #661 `rhwp-studio: 텍스트 드래그 선택 중 커서와 스크롤 위치가 튀는 현상`
- 기준 브랜치: `local/task661`
- 기반 코드: `upstream/pr/664` (`b1b18c2`) + Stage 1/2 커밋
- 단계 목표: 드래그 선택 중 포인터가 편집 영역 edge에 있을 때만 자동 스크롤되도록 별도 경로 추가

## 변경 내용

### 1. 텍스트 선택 드래그 상태 시작/종료 경로 분리

- 파일: `rhwp-studio/src/engine/input-handler.ts`
- `startTextSelectionDrag()`, `stopTextSelectionDrag()`를 추가했다.
- 드래그 중에는 `document`에도 `mousemove`를 임시 등록해 포인터가 편집 영역 밖으로 나가도 마지막 좌표를 추적할 수 있게 했다.
- 드래그 종료 및 `dispose()`에서 document listener와 RAF를 정리한다.

### 2. 마지막 포인터 좌표 기준 선택 갱신

- 파일: `rhwp-studio/src/engine/input-handler.ts`
- 기존 `hitTestFromEvent()`를 `hitTestFromClientPoint()` 위임 구조로 바꿨다.
- RAF throttle 중 새 마우스 이벤트가 들어오면 마지막 포인터 좌표를 저장하고, 실제 선택 focus 갱신은 최신 좌표 기준으로 수행한다.

### 3. 포인터 edge 자동 스크롤 추가

- 파일: `rhwp-studio/src/engine/input-handler.ts`
- edge 영역: 편집 영역 위/아래 48px
- 스크롤 속도: 프레임당 2~20px 범위에서 edge 침범 거리에 비례
- 스크롤이 실제로 발생한 경우에만 같은 포인터 좌표로 hit-test를 다시 수행해 선택 범위를 이어간다.

### 4. 마우스 드래그 경로 연결

- 파일: `rhwp-studio/src/engine/input-handler-mouse.ts`
- 일반 본문, 글상자 내부 텍스트, 편집 중인 같은 글상자 클릭의 드래그 시작 지점을 `startTextSelectionDrag()`로 교체했다.
- 드래그 중 RAF 갱신은 `updateTextSelectionDragPointer()`와 `updateTextSelectionDragFromPointer()`를 사용하도록 바꿨다.

## 검증

```bash
cd rhwp-studio
npm run build
```

- 결과: 성공
- 비고: Vite의 기존 chunk size 경고는 발생했지만 빌드 실패는 없었다.

```bash
cd rhwp-studio
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" npm run e2e -- --mode=headless
```

- 결과: 성공
- 통과 항목: 새 문서 생성, 텍스트 입력, 줄바꿈, 문단 분리, 페이지 넘김, Backspace 문단 병합
- 비고: Vite dev server는 검증 후 종료했다.

```bash
git diff --check
```

- 결과: 성공

## 영향 범위

- 일반 caret 갱신 경로의 `scrollCaretIntoView()`는 유지했다.
- 드래그 선택 경로에서만 포인터 edge 기반 스크롤을 수행한다.
- 표/그림/선/리사이즈 드래그 플래그는 기존 분기에서 먼저 처리되므로 이번 변경 대상에서 제외된다.

## 남은 작업

1. Stage 4에서 브라우저 수동 또는 전용 E2E로 실제 드래그 edge 스크롤 시나리오를 확인한다.
2. 선택 중 커서 위치, 스크롤 위치, selection highlight가 함께 안정적으로 유지되는지 검증한다.
3. 필요하면 edge 임계값과 속도 상한을 조정한다.

## 승인 요청

Stage 3 변경 내용과 검증 결과를 승인해 주시면 Stage 4 검증 및 보정으로 진행한다.
