# 프론트 metrics 요약

- generatedAt: 2026-07-11T06:21:30.736Z
- commit: 3077f96d1f9931c50d6d62be77b389d4f66470a9
- measured source clean: true
- advisory: Phase 0 기준선 고정용 snapshot이며 CI fail gate가 아니다.

## Cognitive Complexity 총량

| Reported functions | Total CC | Top 20 sum | CC>25 count | CC>25 sum | CC>100 count | Max CC |
|-------------------:|---------:|-----------:|------------:|----------:|-------------:|-------:|
| 2282 | 11805 | 2581 | 62 | 3932 | 6 | 453 |

## Group 합계

| Group | Files | Lines | Functions | Total CC | Top 20 sum | CC>25 | CC>25 sum | Max CC | any | exports |
|------|------:|------:|----------:|---------:|-----------:|------:|----------:|-------:|----:|--------:|
| Studio runtime | 145 | 59783 | 3952 | 9505 | 2523 | 47 | 3346 | 453 | 455 | 595 |
| Chrome extension | 15 | 2330 | 130 | 444 | 295 | 4 | 132 | 40 | 0 | 31 |
| Firefox extension | 15 | 2325 | 137 | 444 | 295 | 4 | 132 | 40 | 0 | 31 |
| Safari extension | 3 | 1305 | 107 | 299 | 259 | 3 | 115 | 43 | 0 | 0 |
| Shared frontend | 9 | 864 | 40 | 107 | 98 | 0 | 0 | 14 | 0 | 19 |
| VS Code extension | 4 | 1387 | 86 | 159 | 126 | 0 | 0 | 23 | 8 | 5 |
| npm editor wrapper | 2 | 266 | 25 | 19 | 19 | 0 | 0 | 8 | 0 | 6 |
| legacy /web | 10 | 6592 | 251 | 828 | 442 | 4 | 207 | 86 | 0 | 6 |

## Cognitive Complexity 상위

| CC | Function LOC | Function | Location |
|---:|-------------:|----------|----------|
| 453 | 995 | `onClick` | `rhwp-studio/src/engine/input-handler-mouse.ts:262` |
| 444 | 909 | `onKeyDown` | `rhwp-studio/src/engine/input-handler-keyboard.ts:371` |
| 348 | 381 | `handleOk` | `rhwp-studio/src/ui/picture-props-dialog.ts:1930` |
| 212 | 286 | `populateFromProps` | `rhwp-studio/src/ui/picture-props-dialog.ts:2316` |
| 142 | 354 | `finishResizeDrag` | `rhwp-studio/src/engine/input-handler-table.ts:708` |
| 133 | 328 | `onMouseMove` | `rhwp-studio/src/engine/input-handler-mouse.ts:1405` |
| 88 | 326 | `fillSnapshotFromWasm` | `rhwp-studio/src/compare/diff-engine.ts:929` |
| 86 | 206 | `<anonymous>` | `web/editor.js:187` |
| 73 | 129 | `findPictureAtClick` | `rhwp-studio/src/engine/input-handler-picture.ts:135` |
| 73 | 124 | `renderPictureObjectSelection` | `rhwp-studio/src/engine/input-handler-picture.ts:302` |
| 63 | 81 | `annotateDiffSectionPages` | `rhwp-studio/src/compare/diff-engine.ts:2895` |
| 61 | 142 | `handleCtrlKey` | `rhwp-studio/src/engine/input-handler-keyboard.ts:1281` |
| 55 | 190 | `onInput` | `rhwp-studio/src/engine/input-handler-text.ts:413` |
| 55 | 113 | `_onKeyDown` | `web/text_selection.js:772` |
| 54 | 36 | `comparePositions` | `rhwp-studio/src/engine/cursor.ts:184` |
| 52 | 63 | `processPendingNav` | `rhwp-studio/src/engine/input-handler-text.ts:74` |
| 51 | 113 | `handleF11` | `rhwp-studio/src/engine/input-handler-keyboard.ts:1755` |
| 47 | 74 | `matchSegmentDp` | `rhwp-studio/src/compare/diff-engine.ts:1959` |
| 46 | 207 | `cleanupParagraphAlignStepsToDiffItems` | `rhwp-studio/src/compare/diff-engine.ts:2264` |
| 45 | 124 | `collectMods` | `rhwp-studio/src/ui/char-shape-dialog.ts:923` |

## 함수 LOC 상위

| LOC | Function | Location |
|----:|----------|----------|
| 995 | `onClick` | `rhwp-studio/src/engine/input-handler-mouse.ts:262` |
| 909 | `onKeyDown` | `rhwp-studio/src/engine/input-handler-keyboard.ts:371` |
| 706 | `<anonymous>` | `web/font_substitution.js:10` |
| 625 | `<anonymous>` | `rhwp-firefox/content-script.js:4` |
| 610 | `<anonymous>` | `rhwp-safari/src/content-script.js:5` |
| 606 | `<anonymous>` | `rhwp-chrome/content-script.js:4` |
| 381 | `handleOk` | `rhwp-studio/src/ui/picture-props-dialog.ts:1930` |
| 354 | `finishResizeDrag` | `rhwp-studio/src/engine/input-handler-table.ts:708` |
| 329 | `getHtml` | `rhwp-vscode/src/hwp-editor-provider.ts:172` |
| 328 | `onMouseMove` | `rhwp-studio/src/engine/input-handler-mouse.ts:1405` |
| 326 | `fillSnapshotFromWasm` | `rhwp-studio/src/compare/diff-engine.ts:929` |
| 317 | `setupEventListeners` | `web/editor.js:77` |
| 298 | `buildBorderTab` | `rhwp-studio/src/ui/para-shape-tab-builders.ts:387` |
| 286 | `buildTabSettingsTab` | `rhwp-studio/src/ui/para-shape-tab-builders.ts:78` |
| 286 | `populateFromProps` | `rhwp-studio/src/ui/picture-props-dialog.ts:2316` |
| 265 | `buildPicturePanel` | `rhwp-studio/src/ui/picture-props-dialog.ts:1481` |
| 257 | `buildBasicPanel` | `rhwp-studio/src/ui/picture-props-dialog.ts:416` |
| 232 | `buildFillPanel` | `rhwp-studio/src/ui/picture-props-dialog.ts:980` |
| 226 | `initRhwpDev` | `rhwp-studio/src/core/rhwp-dev.ts:25` |
| 207 | `cleanupParagraphAlignStepsToDiffItems` | `rhwp-studio/src/compare/diff-engine.ts:2264` |

## 비고

- 이 결과는 Phase 0 advisory snapshot이며 CI fail gate가 아니다.
- JSON 전체 경로: `mydocs/metrics/frontend/2026-07-11/metrics.json`.
- `any`/`as any`/`this: any`와 export 수는 TypeScript AST 기준이며 일부 항목은 서로 포함 관계다.
- ESLint parse/fatal diagnostics는 `metrics.json`에 보존한다.
