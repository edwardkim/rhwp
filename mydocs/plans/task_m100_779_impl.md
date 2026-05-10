# Task #779 구현 계획서

**선행**: [task_m100_779.md](task_m100_779.md) 수행계획서 승인 (가설 A — `scrollCaretIntoView` 가드 영역 — 진행).

## 단계별 진행

### Stage 1 — 본질 실측 진단

**목적**: scrollbar release → `updateCaret` → `scrollCaretIntoView` chain 의 정확한 trigger + scroll back 시점을 실측으로 확정.

**영역**:
- rhwp-studio dev 서버 기동 (`cd rhwp-studio && npx vite --port 7700`)
- 다중 페이지 문서 로드 (hwp3-sample10.hwp 등)
- console.log 임시 추가:
  - `onMouseUp` 진입 + `updateCaret` 호출 직전
  - `scrollCaretIntoView` 진입 시점 + caret rect / scrollTop / 최종 scroll 위치
- 결함 재현 (caret p.1 → scrollbar drag → p.2 → release)
- log 분석 → trigger chain 정확도 확인

**판정 기준**:
- (A) trigger 가 `document mouseup → onMouseUp → updateCaret → scrollCaretIntoView` 정합 → 가설 A 진행
- (B) 다른 trigger (예: viewport-scroll event handler, 다른 component) → 가설 재정정

**산출물**: `mydocs/working/task_m100_779_stage1.md` — trigger chain 실측 결과 + 가설 A 진행 결정 영역

### Stage 2 — 정정 구현 (가설 A)

**목적**: caret/cursor 위치 변경이 **동반되지 않은** mouseup → `scrollCaretIntoView` skip.

**정정 영역** ([rhwp-studio/src/engine/input-handler.ts](rhwp-studio/src/engine/input-handler.ts)):

#### 정정 1 — `updateCaret` 분기 가드

```typescript
private updateCaret(skipScroll: boolean = false): void {
    const rect = this.cursor.getRect();
    if (rect) {
      ...
      this.caret.update(rect, zoom);
      if (!skipScroll) {
          this.scrollCaretIntoView(rect);
      }
    }
    ...
}
```

#### 정정 2 — `onMouseUp` 호출 영역 좁힘

```typescript
export function onMouseUp(this: any, _e: MouseEvent): void {
  // ... 기존 early return (image placement, table move, picture rotate, etc.)

  // [Task #779] cursor 변경이 동반되지 않은 mouseup (예: scrollbar release) 시
  // scrollCaretIntoView 호출 skip — caret OLD 위치 자동 복귀 결함 정정.
  // cursor 변경 trigger 영역 (selection drag 종료 / cursor click): 기존 scroll 동작 보존.
  const cursorChanged = ...; // selection drag end 또는 click event 동반 여부
  this.updateCaret(!cursorChanged);  // skipScroll = !cursorChanged
}
```

또는 더 간단한 영역:

```typescript
export function onMouseUp(this: any, _e: MouseEvent): void {
  // ... 기존 early return

  // [Task #779] mouseup 영역 자체에서는 scroll back skip — cursor 변경 시점 (mousedown +
  // mouseup chain in editor) 은 다른 경로 (mousemove / click handler) 에서 명시적 호출.
  this.updateCaret(true);  // skipScroll = true
}
```

#### 가드 영역 결정

Stage 1 의 실측 결과에 따라 더 좁은 영역 (특정 케이스만 skip) 또는 더 넓은 영역 (onMouseUp 전체 skip) 결정.

**검증**:
- TypeScript: `tsc` clean
- 본 파일 의 다른 `updateCaret` 호출 영역 영향 검토:
  - cursor click → `onMouseDown` → `onMouseUp` → `updateCaret(skipScroll=true)` → click 위치 cursor 갱신 후 scroll 부재
  - 키보드 이동 → 별도 path (keyboard handler) 의 `updateCaret(skipScroll=false)` → scroll 동반
  - 드래그 selection 종료 → `stopTextSelectionDrag` → 별도 호출 (PR #718 영역) 보존

**산출물**: `mydocs/working/task_m100_779_stage2.md` — 정정 영역 + 영향 분석

### Stage 3 — 회귀 검증

**목적**: 결정적 검증 + 수동 검증 + e2e 회귀 가드 (선택).

**결정적 검증**:
- `cargo test --lib --release`: 1217+ passed (회귀 0)
- `cargo clippy --release --lib`: 신규 경고 0
- TypeScript: `tsc` clean
- WASM 빌드: 정상

**수동 검증 시나리오**:
1. **본 결함 해소**: caret p.1 → scrollbar drag → p.2 → release → **p.2 보존** ✓
2. **cursor click 정상**: caret p.1 → p.2 클릭 → cursor p.2 + scroll 정합 ✓ (회귀 부재)
3. **키보드 이동 정상**: caret p.1 → page-down 키 → cursor + scroll 동반 ✓ (회귀 부재)
4. **드래그 selection autoscroll (PR #718)**: 드래그 selection 시 자동 scroll 정상 ✓ (회귀 부재)
5. **wheel scroll 정상**: 마우스 wheel 로 scroll → caret 위치 무관 보존 ✓ (회귀 부재)

**e2e 회귀 가드** (선택):
- `rhwp-studio/e2e/scroll-page-preserve.test.mjs` (신규):
  - hwp3-sample10.hwp 로드 → caret p.1 → scrollbar drag (puppeteer) → p.2 → mouseup → scrollTop 보존 검증
- 패턴 정합: PR #718 의 `drag-selection-autoscroll.test.mjs`

**산출물**: `mydocs/working/task_m100_779_stage3.md` — 검증 결과 + 회귀 0 확인

### Stage 4 — 최종 결과 보고서 + 시각 판정

**목적**: 단계별 보고서 통합 + 작업지시자 시각 판정 + 최종 결과 보고서.

**영역**:
- 작업지시자 시각 판정: 결함 재현 절차 (1~4) + 정정 후 시나리오 (1~5) 모두 통과
- 최종 결과 보고서: `mydocs/report/task_m100_779_report.md`
- orders 갱신: `mydocs/orders/20260510.md` 본인 영역 등록
- closes #779

**산출물**:
- `mydocs/report/task_m100_779_report.md`
- 작업지시자 ★ 통과
- PR 신규 생성 영역 (메인테이너 본인)

## 단계별 commit 전략

| Stage | commit | 영역 |
|-------|--------|------|
| Stage 1 | (소스 무변경) | 진단 영역, 보고서만 |
| Stage 2 | `Task #779 Stage 2: scrollCaretIntoView 가드 추가 (scrollbar release back-scroll 정정)` | 정정 코드 + 보고서 |
| Stage 3 | `Task #779 Stage 3: 회귀 검증 + e2e 가드 추가` | 검증 + e2e 신규 (선택) |
| Stage 4 | `Task #779 최종 보고서 + orders 갱신 (closes #779)` | 보고서 + orders |

PR 생성 시 모든 commits 포함.

## 위험 영역 + 가드

- **위험 1**: cursor click 영역 의 scroll back 회귀 가능성. **가드**: cursor click trigger 영역 의 scroll 호출은 별도 path (예: `onMouseDown` 후 cursor.moveTo 직접 호출) 에서 보존.
- **위험 2**: 드래그 selection autoscroll 회귀. **가드**: PR #718 의 `updateTextSelectionDragAutoScroll` 영역은 scrollCaretIntoView 우회. 본 정정 영향 부재 검증.
- **위험 3**: 다른 mouseup trigger (예: 표 셀 클릭 → 가드 진입) 회귀. **가드**: Stage 1 의 실측 으로 trigger chain 정확 확인 후 영역 좁힘.
