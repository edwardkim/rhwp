---
PR: #718
제목: Task #661 — 드래그 선택 중 커서/스크롤 튐 정정
컨트리뷰터: @postmelee (Taegyu Lee) — 6+ 사이클 컨트리뷰터 (PR #645/#664 동일)
처리: 옵션 A — 5 commits 단계별 보존 cherry-pick + 메인테이너 통합 정정 + no-ff merge
처리일: 2026-05-09
머지 commit: 95eea5f9
---

# PR #718 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (5 commits 단계별 보존 cherry-pick + 메인테이너 통합 정정 + no-ff merge `95eea5f9`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `95eea5f9` (--no-ff merge) |
| Issue #661 | close 자동 정합 (closes #661) |
| 시각 판정 | ★ **통과 (작업지시자 직접, 웹 에디터)** |
| 자기 검증 | lib **1173** + 통합 ALL GREEN + npm build / tsc clean + clippy 신규 경고 0 |
| WASM 빌드 | 4,607,734 bytes (md5 `38a91f98...`) |
| 분리된 후속 | Issue #717 (PR #725 별도 처리) |

## 2. 정정 본질 — 드래그 중 caret 갱신 + 스크롤 책임 분리

### 2.1 결함 메커니즘

기존 경로 영역 — `mousemove` 마다 hit-test → cursor focus 이동 → caret 갱신 + `scrollCaretIntoView()` 호출 → 선택 focus 의 다른 페이지/문단 순간 이동 시 스크롤 컨테이너 가 caret rect 추적 → 사용자 드래그 위치 ↔ 문서 스크롤 함께 튐.

### 2.2 정정 영역
1. **caret**: 선택 상태 표시만 갱신 (scrollCaretIntoView 부재)
2. **스크롤**: 포인터 edge 감지만 담당 (RAF 루프 + edge 48 px + step 2~20 px)
3. **편집 영역 외부 드래그**: document-level mousemove 등록/해제
4. **포인터 좌표 저장 + 스크롤 발생 시 동일 좌표로 hit-test 재계산**

### 2.3 신규 헬퍼 (`input-handler.ts`, +128 LOC)
- `startTextSelectionDrag` / `stopTextSelectionDrag`
- `updateTextSelectionDragPointer` / `updateTextSelectionDragFromPointer`
- `hitTestFromClientPoint` (clientX/Y parameter 분리)
- `updateTextSelectionDragAutoScroll` / `runTextSelectionDragAutoScroll`
- `updateCaretDuringDrag` (드래그 중 caret 갱신)

### 2.4 input-handler-mouse.ts 정정 (+6/-9)
- 3 곳 `isDragging=true` → `startTextSelectionDrag(e)` (드래그 시작 시 포인터 좌표 보존 + document mousemove 등록)
- mousemove 분기: hit-test 직접 호출 → `updateTextSelectionDragFromPointer` 래퍼
- mouseup: `isDragging=false` → `stopTextSelectionDrag` (auto-scroll 종료 + document mousemove 해제)

### 2.5 회귀 가드 e2e (`drag-selection-autoscroll.test.mjs`, +86 LOC)
- 70줄 문서 + 첫 줄 → 하단 edge 드래그
- scrollTop 0 → 1529 / hasSelection=true / focus 문단 69 / highlight 70

## 3. 메인테이너 통합 정정 (충돌 3건 해결)

본 PR base = `de1c2d00` (PR #645 후속, 5/8 시점). devel HEAD = `93d6c6a7` (5/9 시점, **PR #693/#707/#710/#711/#714/#715 머지 후 영역**).

### 3.1 `input-handler-mouse.ts:1107` (mousemove 분기)

**충돌**: PR #693 의 `getPageAtPoint` 직접 호출 영역 vs PR #718 의 `updateTextSelectionDragFromPointer` 래퍼 영역.

**메인테이너 정정**: PR #718 영역 의 `updateTextSelectionDragFromPointer` 영역 채택 — 포인터 좌표 기반 hit-test 영역 (자동 스크롤 영역과 동기 영역).

### 3.2 `input-handler.ts:980` (`hitTestFromClientPoint`)

**충돌**: PR #693 영역 의 `getPageAtPoint(contentX, contentY)` 영역 vs PR #718 영역 의 `getPageAtY(contentY)` 영역 (PR #718 base 영역 시점 영역에 PR #693 미반영).

**메인테이너 통합 정정**:
```typescript
// PR #718 의 clientX/Y parameter 영역 + devel 의 getPageAtPoint
// (PR #693 그리드 모드 click 좌표 정합) 영역 통합
const contentX = clientX - contentRect.left;
const contentY = clientY - contentRect.top;
const pageIdx = this.virtualScroll.getPageAtPoint(contentX, contentY);
```

### 3.3 `input-handler.ts:1648` (`updateCaretDuringDrag` 함수 본문)

**충돌**: devel 영역 의 `caret.updateLive(rect, zoom) + scrollCaretIntoView(rect)` 영역 vs PR #718 영역 의 `caret.update(rect, zoom)` 영역 + scrollCaretIntoView 부재 영역.

**메인테이너 통합 정정**:
```typescript
this.caret.updateLive(rect, zoom);   // devel: PR #664 (Task #658) 깜박임 타이머 유지 본질 보존
// [Task #661] 드래그 중 스크롤은 caret rect 가 아니라 포인터 edge 기준 경로에서만 처리한다.
// 메인테이너 통합 정정: devel 의 updateLive (PR #664 깜박임 타이머 유지 본질) 보존 +
// PR #718 의 scrollCaretIntoView 부재 본질 적용.
```

→ `feedback_pr_supersede_chain` + `feedback_image_renderer_paths_separate` 권위 사례 영역 (PR #694/#706/#711 패턴 정합 영역의 메인테이너 통합 정정 영역).

## 4. PR supersede / 컨트리뷰터 사이클

@postmelee (Taegyu Lee):
- 누적 7 머지 (#168/#169/#209/#214/#224/#339/PR #664)
- **PR #664 (Task #658) 동일 컨트리뷰터의 후속 영역** (Task #661 영역 분리 영역)
- 6+ 사이클 핵심 컨트리뷰터 (rhwp-chrome / firefox / rhwp-studio 영역)

## 5. 본 환경 cherry-pick + 검증

### 5.1 cherry-pick (5 commits + 1 빈 commit skip)
```
9ee9c552 Task #661: Stage 1 analysis and plans
592f04c5 Task #661: Stage 2 disable caret auto-scroll during drag (충돌 1건 해결)
cbace999 Task #661: Stage 3 add pointer edge auto-scroll (충돌 2건 해결)
5087b789 Task #661: Stage 4 add drag autoscroll e2e
9d0a695b Task #661: Final report and issue split
[skipped]: Task #661: Align drag caret update with devel — 메인테이너 통합 정정 영역에 이미 포함됨
```

### 5.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (28.77s) |
| `cargo test --release` | ✅ lib **1173** + 통합 ALL GREEN, failed 0 |
| `cargo clippy --release --all-targets` | ✅ 신규 경고 0 |
| `npx tsc --noEmit` (rhwp-studio) | ✅ clean |
| `npm run build` (rhwp-studio) | ✅ PWA 정상 (512ms) |
| WASM 빌드 (Docker) | ✅ 4,607,734 bytes |

### 5.3 시각 판정 ★ 통과

작업지시자 직접 시각 판정 (2026-05-09, 웹 에디터):
> "웹 에디터에서 블럭 드래그 선택 기능 통과입니다."

→ 드래그 선택 안정성 / 자동 스크롤 / 편집 영역 외부 드래그 / 선택 focus 정합 / 그리드 모드 정합 / 드래그 중 깜박임 타이머 유지 / 다른 PR 영역 회귀 부재 영역 모두 정합.

## 6. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @postmelee 6+ 사이클 핵심 컨트리뷰터 (PR #645/#664 동일 컨트리뷰터, 다회 사이클 영역) |
| `feedback_pr_supersede_chain` | PR #693 (Task #685+#689) `updateCaretDuringDrag` 본문 영역 + PR #664 (Task #658) `caret.updateLive` 영역과 PR #718 영역 의 충돌 영역 → 메인테이너 통합 정정 (PR #694/#706/#711 패턴 정합) |
| `feedback_image_renderer_paths_separate` | input-handler-mouse.ts + input-handler.ts 두 영역 동기 정정 (드래그 헬퍼 영역) |
| `feedback_process_must_follow` | TDD Stage 1~4 + 후속 분리 (Issue #717 → PR #725) 절차 정합 |
| `feedback_visual_judgment_authority` | 결정적 검증 (CI + e2e 자동 회귀화) + 작업지시자 시각 판정 ★ 통과 (웹 에디터) |
| `feedback_assign_issue_before_work` | Issue #661 / #717 컨트리뷰터 self-등록 패턴 (assignee 부재) |

## 7. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- Issue #717 OPEN — 표 셀 빈 영역 클릭 hit-test (PR #725 별도 처리 영역)

---

작성: 2026-05-09
