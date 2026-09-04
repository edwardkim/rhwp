# task_m100_2259 — 2단계 완료보고서: 통합 배율 드롭다운 UI

- **이슈**: [#2259](https://github.com/edwardkim/rhwp/issues/2259)
- **단계**: 2/4

## 변경 내용

### `rhwp-vscode/src/hwp-editor-provider.ts` (상태 표시줄 마크업 + CSS)

- `stb-view-mode` (1쪽/2쪽 토글) 버튼 **제거**.
- `stb-zoom-val` (단순 텍스트) → `stb-zoom-menu` 드롭다운 트리거 버튼으로 교체 (`100% ▾`).
- `stb-zoom-popup` 추가 — 맞춤 3항목 + 구분선 + % 프리셋 5항목, 각 항목에 `data-mode` / `data-zoom`.
- CSS 추가 (`stb-` 접두어 준수): `.stb-zoom-menu-wrap`, `.stb-popup`, `.stb-popup-item`, `.stb-check`, `.stb-popup-sep`, `.stb-caret`.
  - 상태 표시줄 위로 펼치도록 `bottom: 28px` 로 배치.
  - 색상은 VS Code 테마 변수(`--vscode-menu-*`) 사용.
- 접근성: 트리거에 `aria-haspopup` / `aria-expanded`, 팝업에 `role="menu"`, 항목에 `role="menuitem"`.

최종 배치:

```
[−]  [ 100% ▾ ]  [+]
          ├─ ✓ 폭 맞춤
          ├─   쪽 맞춤 (전체 보기)
          ├─   두 쪽 맞춤
          ├────────────────
          └─   50 / 75 / 100 / 150 / 200%
```

### `rhwp-vscode/src/webview/viewer.ts`

| 항목 | 내용 |
|------|------|
| DOM 참조 | `stbZoomVal` / `stbViewMode` → `stbZoomLabel` / `stbZoomMenu` / `stbZoomPopup` |
| `setViewMode()` | 제거 (토글 버튼과 함께 사라짐, 배치는 메뉴가 결정) |
| `currentMenuKey()` | 현재 `zoomMode` + `viewMode` → 메뉴 항목 키 |
| `updateZoomMenuChecks()` | 현재 항목에 `✓` 표시 |
| `setZoomMenuOpen()` | 팝업 열기/닫기 (+ `aria-expanded` 동기화) |
| 이벤트 | 트리거 클릭 토글, 바깥 클릭 / `Esc` 닫기, 항목 클릭 → `applyZoomMode()` |
| `updateStatusBar()` | 트리거 라벨에 실제 배율 %, 체크 갱신 |

### 메뉴 항목 → 상태 매핑 (수행계획서 3.3 그대로)

| 항목 | `viewMode` | `zoomMode` |
|------|-----------|-----------|
| 폭 맞춤 | `single` | `fitWidth` |
| 쪽 맞춤 (전체 보기) | `single` | `fitPage` |
| 두 쪽 맞춤 | `double` | `fitPage` |
| 50~200% | 현재 값 유지 | `manual` |

## 검증

- `npx tsc --noEmit -p tsconfig.webview.json` → 통과
- `npx tsc --noEmit -p tsconfig.json` → 통과

이 단계로 요구사항 1(2쪽 자동 맞춤) · 2(폭/쪽 맞춤) · 3(메뉴 통합)이 충족된다. 요구사항 4(창 크기 대응)는 3단계에서 처리한다.

## 다음 단계

3단계 — `ResizeObserver` 로 뷰포트 크기 변화 시 맞춤 배율 재계산.
