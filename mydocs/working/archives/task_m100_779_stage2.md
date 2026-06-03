# Task #779 Stage 2 — 정정 구현

## 정정 영역

### 1. `updateCaret` 시그니처 확장

[rhwp-studio/src/engine/input-handler.ts:1553~1614](rhwp-studio/src/engine/input-handler.ts#L1553)

```typescript
/**
 * 캐럿 위치를 갱신한다.
 *
 * @param skipScroll true 시 `scrollCaretIntoView` 호출 skip — cursor 변경 trigger 가 동반되지 않은
 *                   onMouseUp (예: drag-during-scroll 영역, scrollbar release 영역) 의 자동 scroll back
 *                   결함 차단 영역. (Task #779)
 */
private updateCaret(skipScroll: boolean = false): void {
    const rect = this.cursor.getRect();
    if (rect) {
      ...
      if (!skipScroll) {
        this.scrollCaretIntoView(rect);
      }
    }
    ...
}
```

**기본값 `false`** — 기존 호출 30+ 곳 (키보드 영역, programmatic cursor move 등) 무영향. **opt-in skip** 영역 좁힘.

### 2. `onMouseUp` 호출 영역 변경

[rhwp-studio/src/engine/input-handler-mouse.ts:1390](rhwp-studio/src/engine/input-handler-mouse.ts#L1390)

```typescript
// [Task #779] mouseup 영역 의 updateCaret 은 scrollCaretIntoView skip.
// 본질: cursor 변경 trigger 영역 (mousedown / drag selection move 등) 에서 이미 cursor 위치
// 갱신 + scroll 호출 영역 동반. mouseup 영역 의 updateCaret 은 selection 종료 영역 의
// visual cleanup 만 담당 — caret 위치 자체는 변경 부재 영역. scrollCaretIntoView 가 호출 시
// 사용자 의도적 scrollbar drag (drag-during-scroll 패턴) 영역 의 caret 원본 위치 자동 복귀
// 결함 발동.
this.updateCaret(true);
```

## 영역 좁힘 분석

| 호출 영역 | skipScroll | 동작 |
|----------|-----------|------|
| `onMouseUp` 의 line 1390 (드래그 selection 종료) | **true** | scroll skip → 본 결함 차단 |
| 키보드 입력 영역 (input-handler-keyboard.ts 20+ 곳) | false (기본) | 기존 동작 보존 (cursor 따라 scroll) |
| programmatic cursor move (moveCursorTo, enterInlineEditing 등) | false (기본) | 기존 동작 보존 |
| `onMouseDown` 영역 의 cursor placement (input-handler-mouse.ts 8+ 곳) | false (기본) | 기존 동작 보존 (click 위치 따라 scroll) |
| 드래그 selection autoscroll (PR #718, `updateTextSelectionDragAutoScroll`) | (별도 path) | 기존 동작 보존 |

## 검증

| 검증 | 결과 |
|------|------|
| `tsc --noEmit` | clean |
| `npm run build` | 성공 (4.6 MB WASM, 707 KB index.js) |
| `cargo test --lib --release` | 1217 passed (rust lib 무영향 확인) |

## 수동 검증 시나리오 (Stage 3 영역)

작업지시자 시각 검증 영역:

1. **본 결함 해소**:
   - 다중 페이지 문서 로드 (예: hwp3-sample10.hwp 763 페이지)
   - 텍스트 클릭 (caret p.1) → 마우스 보유 상태 로 scrollbar 까지 drag → release
   - 기대: scrollbar 위치 보존 (이전 페이지 자동 복귀 부재)

2. **cursor click 정상 동작**:
   - 페이지 1 클릭 (caret 설정)
   - 페이지 5 클릭 (caret 변경)
   - 기대: 페이지 5 caret 위치 정상 + 페이지 5 scroll 정합 (회귀 부재)

3. **키보드 navigation 정상 동작**:
   - 페이지 1 caret → page-down 키 / arrow keys
   - 기대: cursor 따라 scroll 정합 (회귀 부재)

4. **드래그 selection autoscroll (PR #718) 정상**:
   - 텍스트 드래그 selection 영역 의 자동 스크롤 (페이지 끝 영역)
   - 기대: PR #718 동작 보존 (회귀 부재)

5. **Wheel scroll 정상**:
   - 마우스 wheel 로 page scroll
   - 기대: caret 위치 무관 보존 (기존 동작)

## 후속 (Stage 3)

- 결정적 검증 통과 (1217 passed + tsc + build) ✓
- 작업지시자 시각 판정 영역 (수동 시나리오 1~5)
- e2e 회귀 가드 (선택) — `rhwp-studio/e2e/scroll-page-preserve.test.mjs` (PR #718 패턴 정합)
