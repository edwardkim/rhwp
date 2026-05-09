---
PR: #718
제목: Task #661 — 드래그 선택 중 커서/스크롤 튐 정정 (closes #661)
컨트리뷰터: @postmelee (Taegyu Lee) — 6+ 사이클 컨트리뷰터 (PR #645/#664 동일 컨트리뷰터)
base / head: devel / pr-task661
mergeStateStatus: DIRTY (충돌)
mergeable: CONFLICTING
CI: ALL SUCCESS
변경 규모: +1228 / -11, 12 files (소스 2 + 신규 e2e 1 + 보고서 6 + plans 2 + package.json 1 + orders 1)
검토일: 2026-05-09
---

# PR #718 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #718 |
| 제목 | Task #661 — 드래그 선택 중 커서/스크롤 튐 정정 |
| 컨트리뷰터 | @postmelee (Taegyu Lee) — 6+ 사이클 (PR #645/#664 동일 컨트리뷰터, 다회 사이클 영역) |
| base / head | devel / pr-task661 |
| mergeStateStatus | **DIRTY (충돌)**, mergeable: CONFLICTING |
| CI | ALL SUCCESS (Build & Test, CodeQL ×3, Render Diff, Canvas visual diff) |
| 변경 규모 | +1228 / -11, 12 files |
| 커밋 수 | 6 (Stage 1~4 + 최종 보고서 + devel 정합 commit) |
| closes | #661 |
| 분리된 후속 | Issue #717 (PR #725 별도 처리 영역) |

## 2. 결함 본질 (Issue #661)

### 2.1 결함 메커니즘

기존 드래그 선택 경로 영역:
- `mousemove` 마다 hit-test 영역 → cursor focus 이동 → caret 갱신 영역 → `scrollCaretIntoView()` 영역 호출 영역
- 드래그 중 선택 focus 영역 의 다른 페이지/문단 영역 순간 이동 영역 → 스크롤 컨테이너 영역 caret rect 영역 추적 영역 → 사용자 영역 의 드래그 위치 영역 ↔ 문서 스크롤 영역 영역 함께 튐

### 2.2 정정 본질

drag 중 **caret 갱신 + 스크롤 책임 분리**:
- caret: 선택 상태 표시 영역 만 갱신 (scrollCaretIntoView 부재)
- 스크롤: 포인터 edge 감지 영역 만 담당 (RAF 루프 + edge 48 px + step 2~20 px)
- 편집 영역 밖 영역 포인터 영역 도 드래그 지속 영역 (document-level mousemove)

## 3. PR 의 정정

### 3.1 신규 헬퍼 (`input-handler.ts`, +128 LOC)

```typescript
private startTextSelectionDrag(e: MouseEvent)
private updateTextSelectionDragPointer(e: MouseEvent)
private updateTextSelectionDragFromPointer()
private stopTextSelectionDrag()

// 포인터 edge 자동 스크롤
private getTextSelectionDragScrollDeltaY()
private scaleTextSelectionDragScrollStep(distance)
private updateTextSelectionDragAutoScroll()
private runTextSelectionDragAutoScroll()
private stopTextSelectionDragAutoScroll()

// 화면 좌표 hit-test 영역 분리
private hitTestFromClientPoint(clientX, clientY)
```

### 3.2 input-handler-mouse.ts 정정 (-9/+6)

3 곳 영역 `this.isDragging = true;` → `this.startTextSelectionDrag(e);` (드래그 시작 시 포인터 좌표 영역 보존 + document-level mousemove 등록)

mousemove 영역 (line 1097): hit-test 영역 직접 호출 → `updateTextSelectionDragFromPointer()` 래퍼 (저장된 포인터 좌표 영역 의 hit-test).

mouseup 영역: `this.isDragging = false;` → `this.stopTextSelectionDrag();` (auto-scroll 종료 + document mousemove 해제).

### 3.3 회귀 가드 e2e (`drag-selection-autoscroll.test.mjs`, +86 LOC)

70줄 문서 영역 + 첫 줄 영역 → 하단 edge 영역 드래그 영역 + scrollTop / hasSelection / focus 문단 영역 / highlight 카운트 검증.

PR 본문 측정 영역 정합:
- scrollTop: 0 → 1529
- hasSelection: true
- selection.end.paragraphIndex: 69
- highlightCount: 70

## 4. 충돌 원인 분석 ⚠️

### 4.1 충돌 위치

| 파일 | 충돌 |
|------|------|
| `rhwp-studio/src/engine/input-handler-mouse.ts` | content (mousemove 분기 영역) |
| `rhwp-studio/src/engine/input-handler.ts` | content (`updateCaretDuringDrag` 함수 본문 영역) |
| `mydocs/orders/20260508.md` | auto-merge 정합 (충돌 부재) |

### 4.2 충돌 본질 — PR #693 (Task #685+#689) 머지 영역과의 충돌

본 PR base = `de1c2d00` (PR #645 후속, 5/8 시점). devel HEAD = `93d6c6a7` (5/9 시점, **PR #693 / #707 / #710 / #711 / #714 / #715 머지 후 영역**).

#### A. `input-handler-mouse.ts:1107` 영역 (mousemove 드래그 분기 영역)

**devel** (PR #693 머지):
```typescript
const hit = this.hitTestFromEvent(e);
if (hit && hit.paragraphIndex < 0xFFFFFF00) {
    this.cursor.moveTo(hit);
    this.updateCaretDuringDrag();
}
```

**PR #718**:
```typescript
this.updateTextSelectionDragFromPointer();
// 내부 영역: hit + moveTo + updateCaretDuringDrag (포인터 좌표 영역 사용 영역)
```

→ **PR #718 본질 정정** — `updateTextSelectionDragFromPointer` 영역 의 `dragLastClientX/Y` 영역 사용 영역 (포인터 edge 자동 스크롤 영역과 동기 영역). PR #718 영역 채택.

#### B. `input-handler.ts` `updateCaretDuringDrag` 함수 본문 영역

**devel** (PR #693 영역 의 의도된 회귀 영역, PR #645 머지 영역 의 `updateLive` 영역 보존):
```typescript
this.caret.updateLive(rect, zoom);   // 깜박임 타이머 유지 (PR #664 본질)
this.scrollCaretIntoView(rect);       // 드래그 중 스크롤
```

**PR #718** (base de1c2d00 영역 시점, `updateLive` 부재):
```typescript
this.caret.update(rect, zoom);        // 깜박임 리셋 매번
// scrollCaretIntoView 부재 (PR #718 본질 — 드래그 중 스크롤 분리)
```

→ **메인테이너 통합 정정 필요**:
- `scrollCaretIntoView` 부재 영역 보존 (PR #718 본질) ✓
- `this.caret.updateLive(rect, zoom)` 영역 사용 영역 (devel 영역 의 PR #664 본질 보존) ✓

```typescript
// 메인테이너 통합 정정 영역
this.caret.updateLive(rect, zoom);    // PR #664 본질 보존 (깜박임 타이머 유지)
// scrollCaretIntoView 부재 (PR #718 본질 — 드래그 중 스크롤 분리)
```

→ `feedback_pr_supersede_chain` 권위 룰 정합 (PR #694 패턴 정합 영역 의 메인테이너 통합 정정).

## 5. 컨트리뷰터 사이클 (`feedback_contributor_cycle_check`)

@postmelee (Taegyu Lee):
- 누적 7 머지 (#168/#169/#209/#214/#224/#339/PR #664)
- 누적 close: #167/#243/#385/#437/#510/#531/#642/#663/#664
- **현재 6+ 사이클 핵심 컨트리뷰터** (rhwp-chrome / firefox / rhwp-studio 영역)
- PR #664 (Task #658) 동일 컨트리뷰터의 후속 영역 (Task #661 영역 분리 영역)

## 6. Issue #661 + #717

| Issue | 상태 | 설명 |
|-------|------|------|
| #661 | OPEN | 드래그 선택 중 커서/스크롤 튐 (본 PR closes) |
| #717 | OPEN | 표 셀 빈 영역 클릭 시 커서 다른 위치 이동 (PR #725 별도 처리) |

→ scope 정확 분리 정합 (`feedback_process_must_follow` 정합).

## 7. 처리 옵션

### 옵션 A — 6 commits cherry-pick + 메인테이너 통합 정정 + no-ff merge (추천)

PR #694 / #706 / #711 패턴 정합 영역 — 메인테이너 충돌 통합 정정.

```bash
git branch local/task718 93d6c6a7
git checkout local/task718
git cherry-pick c488f516^..6d1c1891  # 6 commits
# 충돌 해결:
#   - input-handler-mouse.ts: PR #718 본질 채택 (updateTextSelectionDragFromPointer)
#   - input-handler.ts: PR #718 본질 (scrollCaretIntoView 부재) + devel 의 updateLive 보존
git checkout local/devel
git merge --no-ff local/task718
```

### 옵션 B — close + 컨트리뷰터에게 rebase 요청

위험: 컨트리뷰터 영역 의 rebase 작업 부담 영역. 권장하지 않음.

→ **옵션 A 추천**.

## 8. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cd rhwp-studio && npm run build` 통과
- [ ] `cd rhwp-studio && npx tsc --noEmit` clean
- [ ] e2e `drag-selection-autoscroll.test.mjs` 통과 (host CDP 또는 headless)
- [ ] `cargo test --release` ALL GREEN
- [ ] `cargo clippy --release` clean
- [ ] form-002 / test_634 / test_705 / test_712 / test_713 회귀 가드 영역 보존

### 시각 판정 게이트
- 본 PR 영역 의 본질 정정 영역 — 드래그 선택 영역 의 UX 정합 영역
- 작업지시자 시각 판정 권장 — 페이지 경계 영역 드래그 영역 / 표 셀 영역 드래그 영역 / IME composition 중 드래그 영역 / 페이지 경계 영역 의 자동 스크롤 영역
- `feedback_visual_judgment_authority` 정합 (CI ALL SUCCESS + e2e 자동 회귀화 영역 통과 + 작업지시자 시각 판정 게이트)

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @postmelee 6+ 사이클 핵심 컨트리뷰터 (PR #645/#664 동일 컨트리뷰터, 다회 사이클 영역) |
| `feedback_pr_supersede_chain` | PR #693 (Task #685+#689) `updateCaretDuringDrag` 본문 영역과 PR #718 영역 의 충돌 영역 → 메인테이너 통합 정정 (PR #694 패턴 정합) |
| `feedback_image_renderer_paths_separate` | input-handler-mouse.ts + input-handler.ts 두 영역 동기 정정 (드래그 헬퍼 영역) |
| `feedback_process_must_follow` | TDD Stage 1~4 + 후속 분리 (Issue #717 → PR #725) 절차 정합 |
| `feedback_assign_issue_before_work` | Issue #661 / #717 컨트리뷰터 self-등록 패턴 (assignee 부재) |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI + e2e 자동 회귀화) + 작업지시자 시각 판정 (권장) |

## 10. 처리 순서 (승인 후)

1. `local/task718` 임시 브랜치 + 6 commits cherry-pick
2. **충돌 해결** (메인테이너 통합 정정):
   - `input-handler-mouse.ts`: PR #718 본질 채택 (updateTextSelectionDragFromPointer)
   - `input-handler.ts`: PR #718 본질 (scrollCaretIntoView 부재) + devel 의 `updateLive` 보존
3. 자기 검증 (npm build / tsc / e2e / cargo test / clippy)
4. WASM 빌드 + 작업지시자 시각 판정
5. 시각 판정 통과 → no-ff merge + push + archives 이동 + 5/9 orders 갱신
6. PR #718 close (closes #661 자동 정합)

---

작성: 2026-05-09
