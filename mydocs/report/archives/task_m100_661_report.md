# Task M100 #661 최종 보고서

## 개요

- 이슈: #661 `rhwp-studio: 텍스트 드래그 선택 중 커서와 스크롤 위치가 튀는 현상`
- 작업 브랜치: `local/task661`
- PR 브랜치: `pr-task661`
- 기반: 최신 `upstream/devel` (`de1c2d0`)에서 #661 범위만 재적용
- 관련 신규 이슈: #717 `rhwp-studio: 표 셀 빈 영역 클릭 시 커서가 다른 위치로 이동`

## 최종 변경 요약

### 1. 드래그 중 caret 기준 자동 스크롤 제거

- 파일: `rhwp-studio/src/engine/input-handler.ts`
- `updateCaretDuringDrag()`에서 `scrollCaretIntoView(rect)` 호출을 제거했다.
- 일반 클릭/키보드 입력의 caret 자동 스크롤은 유지했다.

### 2. 포인터 edge 기준 자동 스크롤 추가

- 파일: `rhwp-studio/src/engine/input-handler.ts`
- 텍스트 선택 드래그 중 마지막 포인터 좌표를 저장한다.
- 포인터가 편집 영역 위/아래 48px edge에 들어온 경우에만 별도 RAF 루프로 스크롤한다.
- 스크롤이 실제 발생한 경우 같은 포인터 좌표로 hit-test를 다시 수행해 선택 focus를 이어간다.

### 3. 드래그 상태 시작/종료 정리

- 파일: `rhwp-studio/src/engine/input-handler.ts`, `rhwp-studio/src/engine/input-handler-mouse.ts`
- `startTextSelectionDrag()`, `stopTextSelectionDrag()` 경로로 일반 본문/글상자 텍스트 드래그 시작과 종료를 정리했다.
- 드래그 중에는 document-level `mousemove`를 임시 등록해 편집 영역 밖 포인터도 추적하고, 종료/`dispose()`에서 해제한다.

### 4. 회귀 E2E 추가

- 파일: `rhwp-studio/e2e/drag-selection-autoscroll.test.mjs`
- script: `npm run e2e:drag-autoscroll`
- 새 문서에 70줄을 입력한 뒤 첫 줄에서 하단 edge까지 드래그한다.
- `scrollTop`, 선택 상태, 하이라이트, 선택 focus 문단 확장을 검증한다.

## 신규 이슈 등록

- 등록 이슈: #717
- URL: https://github.com/edwardkim/rhwp/issues/717
- 제목: `rhwp-studio: 표 셀 빈 영역 클릭 시 커서가 다른 위치로 이동`
- 상태: OPEN
- 비고: upstream 권한 제한으로 `bug` 라벨 추가는 실패했다.

## 최종 검증

```bash
cd rhwp-studio
npm run build
```

- 결과: 성공
- 비고: Vite의 기존 chunk size 경고는 발생했지만 빌드 실패는 없었다.

```bash
cd rhwp-studio
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" npm run e2e:drag-autoscroll -- --mode=headless
```

- 결과: 성공
- 주요 결과:
  - `scrollTop`: `0 → 1529`
  - 선택 상태: `true`
  - 선택 focus: `paragraphIndex=69`
  - 선택 하이라이트: `70`

```bash
git diff --check
```

- 결과: 성공

## 커밋

- `c488f51` Task #661: Stage 1 analysis and plans
- `af16ec7` Task #661: Stage 2 disable caret auto-scroll during drag
- `18ee180` Task #661: Stage 3 add pointer edge auto-scroll
- `8342c52` Task #661: Stage 4 add drag autoscroll e2e

## 미수행 항목

- #661 이슈는 닫지 않았다.
- `local/task661`을 `local/devel` 또는 `devel`로 merge하지 않았다.
- `devel` 직접 push는 하지 않았다.

위 항목은 작업지시자 승인 후 별도 절차로 진행한다.

## 결론

#661의 직접 증상인 드래그 선택 중 caret 기반 스크롤 튐은 드래그 전용 경로에서 제거했다. 포인터 edge 자동 스크롤과 전용 E2E를 추가해 회귀 검증 기준을 남겼다. 검증 중 발견된 표 빈 영역 클릭 hit-test 문제는 #717로 분리 등록했다.
