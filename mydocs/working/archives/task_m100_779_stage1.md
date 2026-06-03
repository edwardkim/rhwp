# Task #779 Stage 1 — 본질 진단 (소스 정밀 추적)

## 진단 영역

본 environment 의 native browser 부재로 (CLI), source-only 정밀 추적 + 가설 테스트 영역 으로 진행. Stage 2 에서 가설 기반 정정 + 작업지시자 browser 시각 검증 영역 으로 confirm.

## scroll back trigger chain 분석

### 모든 `scrollTop = ...` 호출 영역

| 위치 | 영역 | 영향 |
|------|------|------|
| [viewport-manager.ts:92](rhwp-studio/src/view/viewport-manager.ts#L92) | `setScrollTop(y)` API | 명시적 설정 (zoom 등) |
| [canvas-view.ts:75](rhwp-studio/src/view/canvas-view.ts#L75) | 초기 로드 `scrollTop = 0` | 문서 로드 시점만 |
| [input-handler.ts:1074](rhwp-studio/src/engine/input-handler.ts#L1074) | drag autoscroll (PR #718) | 드래그 selection 영역 |
| [input-handler.ts:1952, 1955](rhwp-studio/src/engine/input-handler.ts#L1952) | **`scrollCaretIntoView`** | **본 결함 의 추정 영역** |

### `scrollCaretIntoView` 호출 영역

[input-handler.ts:1609](rhwp-studio/src/engine/input-handler.ts#L1609) `updateCaret()` 안 단일 호출.

### `updateCaret()` 호출 영역 (전체)

- input-handler-keyboard.ts: 20+ callers (키보드 영역, intentional scroll)
- input-handler-mouse.ts: 9 callers (마우스 영역)
- input-handler.ts: 5+ callers (programmatic cursor move 영역)

### `onMouseUp` 안의 `updateCaret` ([input-handler-mouse.ts:1384](rhwp-studio/src/engine/input-handler-mouse.ts#L1384))

```typescript
export function onMouseUp(this: any, _e: MouseEvent): void {
  // ... 8 early returns (image placement, table move, picture rotate, 등)

  if (!this.isDragging) return;  // ← 1363
  this.stopTextSelectionDrag();
  // ... selection cleanup

  this.updateCaret();  // ← 1384
}
```

**조건**: `this.isDragging === true` 필수. line 1363 early return 으로 가드.

### `isDragging` 활성화 영역

[input-handler.ts:999~1004](rhwp-studio/src/engine/input-handler.ts#L999):
```typescript
private startTextSelectionDrag(e: MouseEvent): void {
    this.isDragging = true;
    ...
}
```

`startTextSelectionDrag` 호출 영역 ([input-handler-mouse.ts](rhwp-studio/src/engine/input-handler-mouse.ts)):
- line 650: 글상자 내부 텍스트 클릭
- line 700: 그림 안 텍스트 클릭
- line 760: **regular text click (가장 일반적)**

→ **사용자가 텍스트 영역을 클릭할 때마다 `isDragging = true`** 됨 (드래그 시작 안 해도).

### scroll back trigger chain 결론

**가설 A**: 사용자 시나리오:
1. 텍스트 클릭 (cursor placement, p.1) → `isDragging=true` + `mouseup` listener 등록
2. mouseup 즉시 (사용자 release 클릭) → `onMouseUp` 발동 → isDragging=true 이므로 line 1363 통과 → line 1384 `updateCaret` → `scrollCaretIntoView` (caret 이미 viewport 안 → no-op)
3. listener `{ once: true }` 자동 제거. isDragging=false 됨.

이후 scrollbar 사용:
4. mousedown on browser scrollbar → **document mousedown 발동 안 됨** (native scrollbar — `#scroll-container { overflow-y: auto }` 영역, [editor.css:37](rhwp-studio/src/styles/editor.css#L37))
5. scrollbar drag → 페이지 scroll (browser 직접 처리)
6. mouseup on scrollbar → **document mouseup 발동 안 됨** (native scrollbar)

→ **순수 native scrollbar 영역 에서 onMouseUp chain 발동 부재**. 가설 A 의 trigger chain 영역 으로 본 결함 설명 부재 영역.

### 가설 B (수정) — drag-during-scroll

사용자 시나리오 가능성:
1. 텍스트 클릭 (cursor) → `isDragging=true` + mouseup listener 등록
2. **즉시 mouseup 안 함** — 사용자 가 마우스 버튼 보유 상태 로 scrollbar 까지 drag
3. scrollbar 위에서 mousemove → `mousemove` listener 가 (현재 native scrollbar 위 영역) hit-test 시도
4. scrollbar 위에서 mouseup → `onMouseUp` 발동 → isDragging=true → updateCaret → scrollCaretIntoView → caret p.1 위치 로 scroll back

→ 본 가설 (B) 가능성 영역 — 사용자 가 클릭-앤드-드래그 패턴으로 scrollbar 사용 시.

### 가설 C — 다른 trigger 영역 (Stage 2 진단 영역)

`scrollCaretIntoView` 호출 영역 의 trigger 가 onMouseUp 외 영역 (focus / scrollend / requestAnimationFrame 영역) 가능성. Stage 2 의 정정 + 작업지시자 시각 검증 영역 에서 confirm.

## Stage 2 진행 영역

가설 A/B 가 정합 영역 일 경우 — `updateCaret(skipScroll: boolean = false)` 시그니처 확장 + `onMouseUp` 안 호출 영역 `updateCaret(true)` 변경 으로 영역 좁힘.

가설 C 영역 (다른 trigger) 일 경우 — Stage 2 의 정정 으로 본 결함 미해소 → Stage 2 후속 로 추가 진단 영역.

## 위험 영역

- 가설 A/B 가설 정정 시 드래그 selection 끝 영역 의 cursor scroll 영역 회귀 위험. 가드: PR #718 (Task #661) 의 `updateTextSelectionDragAutoScroll` 영역 (별도 path) 보존.
- `onMouseUp` 의 line 1384 `updateCaret` 호출 영역 의 의도 영역 (드래그 selection 끝 → cursor focus scroll) 부분 손실 위험. 가드: cursor 가 viewport 안 영역 일 경우 scroll skip 자체 가 영향 부재 영역 (현재 `scrollCaretIntoView` 본질 동작 정합).
