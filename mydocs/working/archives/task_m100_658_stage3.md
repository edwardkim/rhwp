# Task M100 #658 단계 3 완료보고서

## 단계명

rhwp-studio 선택 하이라이트 렌더링 비용 완화

## 작업 범위

단계 2에서 native selection rect 정합화를 완료한 뒤, `rhwp-studio` 드래그 선택 중 발생하는 프론트엔드 DOM 갱신 비용을 줄였다.

## 변경 내용

### SelectionRenderer DOM 재사용

`rhwp-studio/src/engine/selection-renderer.ts`를 수정했다.

- 기존 동작: `render()` 호출마다 기존 하이라이트 div를 전부 제거하고 rect 수만큼 새 div 생성
- 변경 동작:
  - 하이라이트 div pool을 유지하고 재사용
  - 선택 rect 수가 줄면 초과 div를 제거하지 않고 `display:none` 처리
  - 동일한 rect layout signature가 반복되면 DOM style 갱신을 생략
  - `clear()`는 active div만 숨기고 pool은 보존

이 변경으로 드래그 중 rect 배열이 매 프레임 바뀌더라도 노드 삭제/생성 비용이 크게 줄어든다.

### 드래그 중 caret 갱신 경량화

`rhwp-studio/src/engine/input-handler.ts`, `input-handler-mouse.ts`, `caret-renderer.ts`를 수정했다.

- `CaretRenderer.updateLive()` 추가
  - 드래그 중 위치만 갱신하고 기존 blink timer는 유지
- `InputHandler.updateCaretDuringDrag()` 추가
  - 캐럿 위치, 선택 하이라이트, 눈금자용 cursor rect 이벤트만 갱신
  - `emitCursorFormatState()`와 `updateFieldMarkers()`는 드래그 중 반복 호출하지 않음
  - mouseup에서는 기존 `updateCaret()`가 실행되므로 최종 상태 갱신은 유지
- 마우스 드래그 rAF 경로에서 `updateCaret()` 대신 `updateCaretDuringDrag()`를 호출

## 검증 결과

### 빌드

```bash
cd rhwp-studio
npm run build
```

결과: 통과

비고: 기존 Vite chunk-size warning은 유지되며 이번 변경과 무관하다.

### Browser 기본 검증

검증 URL:

```text
http://127.0.0.1:7700/
```

확인 항목:

- 페이지 URL: `http://127.0.0.1:7700/`
- 페이지 제목: `rhwp-studio`
- DOM snapshot: 메뉴/서식 도구 모음/상태 표시줄 등 앱 UI 확인
- console warn/error: 없음
- screenshot: 앱 첫 화면 렌더링 확인

### selection layer DOM 계측

Browser 런타임은 앱 내부 `window.__wasm`/`window.__inputHandler`에 접근해 샘플 문서와 선택 상태를 주입하는 `evaluate` 경로가 제한되어 있었다. 따라서 커밋하지 않는 임시 Puppeteer 스크립트(`/private/tmp/rhwp_658_selection_drag_check.mjs`)로 동일 로컬 URL에서 내부 DOM 상태를 계측했다.

계측 대상:

- 샘플: `samples/exam_social.hwp`
- 위치: 페이지 2 오른쪽 자료 박스
- 선택 API: `getSelectionRectsInCell(1, 16, 0, 0, 0, 0, 6, 469)`

계측 결과:

```json
{
  "fullRectCount": 18,
  "shortRectCount": 3,
  "afterFull": { "visible": 18, "total": 18 },
  "afterShort": { "visible": 3, "total": 18 },
  "afterSameShort": { "visible": 3, "total": 18 },
  "afterClear": { "visible": 0, "total": 18 }
}
```

의미:

- 18개 rect 선택 시 visible div가 18개로 맞다.
- 3개 rect 선택으로 줄이면 total div는 18개를 유지하고 visible div만 3개로 줄어든다.
- 같은 선택 상태를 다시 갱신해도 total div가 증가하지 않는다.
- 선택 해제 시 visible div는 0개가 되고 pool은 재사용 대기 상태로 남는다.
- 계측 중 console warn/error는 없었다.

## 남은 작업

단계 4에서 native rect 정합화와 프론트엔드 렌더링 경량화를 통합 검증하고, 최종 결과보고서와 PR 제출 준비를 진행한다.
