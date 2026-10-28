# task_m100_2259 — 1단계 완료보고서: 배율 상태 모델 분리

- **이슈**: [#2259](https://github.com/edwardkim/rhwp/issues/2259)
- **단계**: 1/4 (UI 변경 없음, 순수 상태 모델 분리)

## 변경 내용

`rhwp-vscode/src/webview/viewer.ts`

| 항목 | 내용 |
|------|------|
| 상태 추가 | `type ZoomMode = "manual" \| "fitWidth" \| "fitPage"`, `let zoomMode: ZoomMode = "manual"` |
| 상수 추가 | `ROW_GAP = 12`, `CONTENT_PADDING = 12`, `SIDE_MARGIN = 12` (CSS 값과 일치) |
| `clampZoom()` | `ZOOM_MIN`~`ZOOM_MAX` 클램프 헬퍼 |
| `computeFitZoom(mode, availW?, availH?)` | 맞춤 배율 계산. 문서 전체의 최대 폭·높이 기준 |
| `applyZoom()` → `setZoom(zoom, anchorY?, relayoutAnyway?)` | 배율 적용. `relayoutAnyway` 로 조기 반환 우회 |
| `setManualZoom()` | 수동 모드 전환 + 배율 적용 (−/+ 버튼, Ctrl+휠) |
| `applyZoomMode(mode, viewMode, zoom?)` | 쪽 배치 + 맞춤 모드 동시 설정 |
| `setViewMode()` | 내부를 `applyZoomMode()` 경유로 변경 |

`currentZoom` 은 계속 "실제 적용 배율" 을 담으므로 `makePageWrapper` / `renderPage` / 썸네일 등 하위 렌더 경로는 수정하지 않았다.

## 구현 계획서에서 예고한 함정 처리

기존 `applyZoom()` 의 `if (newZoom === currentZoom) return;` 조기 반환은, **배율이 우연히 같은 채로 1쪽↔2쪽 배치만 바뀌는 경우** 레이아웃 재구성을 통째로 건너뛴다.

→ `setZoom()` 에 `relayoutAnyway` 인자를 추가하고, `applyZoomMode()` 가 배치 변경을 감지하면(`layoutChanged`) 이 값을 `true` 로 넘겨 강제 재구성하도록 했다.

## 부수 수정

`viewer.ts` 180행 주석에 기존 깨진 바이트(`(기본: ��포트 중앙)`)가 있어, 해당 블록을 재작성하는 김에 `(기본: 뷰포트 중앙)` 으로 바로잡았다.

## 검증

- `npx tsc --noEmit -p tsconfig.webview.json` → 통과 (exit 0)
- 맞춤 진입점이 아직 없어 `zoomMode` 는 항상 `manual` 이며, `[−]/[+]` · Ctrl+휠 · 1쪽/2쪽 토글 동작은 기존과 동일하다.

## 다음 단계

2단계 — 상태 표시줄 통합 배율 드롭다운 UI.
