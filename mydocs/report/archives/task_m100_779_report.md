# Task #779 최종 결과 보고서

**Issue**: #779 — rhwp-studio 스크롤바 드래그 후 마우스 release 시 이전 페이지로 자동 복귀
**브랜치**: `local/task779`
**선행 의존**: 부재 (단독 task)
**완료 commits**: cead11f (Stage 1) + 2e6a832 (Stage 2)

## 1. 결함 영역

### 본질

[`rhwp-studio/src/engine/input-handler.ts`](rhwp-studio/src/engine/input-handler.ts) 의 `updateCaret()` 영역 의 `scrollCaretIntoView()` 호출 영역 이 cursor 변경 trigger 없는 mouseup (drag-during-scroll 패턴) 영역 의 caret 원본 위치 자동 복귀 결함 발동.

### 재현 시나리오

1. 다중 페이지 문서 로드 (예: hwp3-sample10.hwp 763 페이지)
2. 텍스트 클릭 (caret p.1)
3. 마우스 보유 상태 로 scrollbar 까지 drag → page 2+ 로 scroll
4. mouse release → caret 원본 위치 (p.1) 로 자동 scroll back

## 2. 정정 영역

### Stage 2 ([2e6a832](https://github.com/edwardkim/rhwp/commit/2e6a832))

| 파일 | 변경 |
|------|------|
| [`input-handler.ts:1553`](rhwp-studio/src/engine/input-handler.ts#L1553) | `updateCaret(skipScroll: boolean = false)` 시그니처 확장 + 조건부 `scrollCaretIntoView` 호출 |
| [`input-handler-mouse.ts:1390`](rhwp-studio/src/engine/input-handler-mouse.ts#L1390) | `onMouseUp` 끝 `updateCaret()` → `updateCaret(true)` 변경 |

```typescript
// input-handler.ts
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

// input-handler-mouse.ts onMouseUp 끝
this.updateCaret(true);  // [Task #779] mouseup 영역 의 scroll back 차단
```

## 3. 영역 좁힘 (회귀 부재 가드)

| 호출 영역 | skipScroll | 동작 |
|----------|-----------|------|
| `onMouseUp` (드래그 selection 종료) | **true** | scroll skip → 본 결함 차단 |
| 키보드 영역 (input-handler-keyboard.ts 20+ 곳) | false (기본) | 기존 동작 |
| programmatic cursor move (moveCursorTo, enterInlineEditing 등) | false (기본) | 기존 동작 |
| `onMouseDown` 영역 cursor placement (8+ 곳) | false (기본) | 기존 동작 |
| 드래그 selection autoscroll (PR #718) | (별도 path) | 보존 |

→ 30+ 기존 호출 영역 무영향. **opt-in skip** 영역 좁힘 (`feedback_hancom_compat_specific_over_general` 정합).

## 4. 검증

### 결정적 검증

| 검증 | 결과 |
|------|------|
| `tsc --noEmit` | clean |
| `npm run build` | 성공 (WASM 4.6 MB, index.js 707 KB) |
| `cargo test --lib --release` | 1217 passed (rust lib 무영향) |
| `cargo clippy --release --lib` | 신규 경고 0 |

### 시각 판정 (작업지시자) ★

작업지시자 dev 서버 직접 시각 판정 결과 — **"해결 완료"** 통과.

5 시나리오 정합:
1. 본 결함 해소 ✅
2. cursor click 정상 ✅ (회귀 부재)
3. 키보드 navigation 정상 ✅ (회귀 부재)
4. 드래그 selection autoscroll (PR #718) 정상 ✅ (회귀 부재)
5. Wheel scroll 정상 ✅ (회귀 부재)

## 5. 단계별 보고서

| Stage | 보고서 |
|-------|--------|
| Stage 1 (본질 진단) | [task_m100_779_stage1.md](mydocs/working/task_m100_779_stage1.md) |
| Stage 2 (정정 구현) | [task_m100_779_stage2.md](mydocs/working/task_m100_779_stage2.md) |
| Stage 3 (회귀 검증 + 시각 판정) | [task_m100_779_stage3.md](mydocs/working/task_m100_779_stage3.md) |

## 6. 권위 사례 강화

- **`feedback_hancom_compat_specific_over_general`**: opt-in `skipScroll` 으로 영역 좁힘 — 30+ 기존 호출 무영향 영역 보존
- **`feedback_pr_supersede_chain`**: PR #718 (Task #661, 드래그 selection autoscroll) 의 후속 영역 — 동일 patten (`scrollCaretIntoView` 영역) 의 다른 trigger (drag-during-scroll) 영역 정정
- **`feedback_visual_judgment_authority`**: 작업지시자 dev 서버 직접 시각 판정 으로 본 결함 해소 + 회귀 부재 양측 confirm

## 7. 후속 영역 (별도 task)

- e2e 회귀 가드 (`scroll-page-preserve.test.mjs`) — PR #718 의 `drag-selection-autoscroll.test.mjs` 패턴 정합 (선택, 본 task 영역 외)
- wheel scroll + caret 위치 영역 의 분석 (현재 무관, 결함 부재 영역)
