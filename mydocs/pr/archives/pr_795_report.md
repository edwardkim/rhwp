---
PR: #795
제목: fix — 표 셀 내부 드래그 선택 시 셀 컨텍스트 이탈 방지 (closes #669)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/11 사이클 5번째 PR)
처리: 옵션 A — 1 commit cherry-pick + 충돌 수동 해결 + no-ff merge
처리일: 2026-05-11
머지 commit: 58176ed3
---

# PR #795 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (1 commit cherry-pick + 충돌 수동 해결 + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `58176ed3` (--no-ff merge) |
| Cherry-pick commit | `2efe20cb` (충돌 수동 해결) |
| closes | #669 |
| 시각 판정 | ✅ 작업지시자 웹 에디터 시각 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN + sweep 170/170 same + WASM 4.68 MB |

## 2. 본질 (Issue #669)

표 셀 내부 텍스트 드래그 선택 시 선택 하이라이트 미렌더링.

### 원인
드래그 중 `hitTest` 영역 영역 셀 내부 빈 영역 (텍스트 라인 외부) 영역 영역 본문 레벨 위치 반환:
- anchor: 셀 내부 (`parentParaIndex` 있음)
- focus: 본문 (`parentParaIndex` 없음)

`updateSelection()` 영역 영역 "셀↔본문 혼합 선택" 판별 → `selectionRenderer.clear()` 호출 → 선택 미렌더링.

## 3. 정정 본질 — input-handler.ts +13, input-handler-mouse.ts +1

### 3.1 `input-handler.ts:1014` `updateTextSelectionDragFromPointer` 래퍼 안 셀 가드

```typescript
private updateTextSelectionDragFromPointer(): void {
  if (!this.isDragging) return;
  const hit = this.hitTestFromClientPoint(this.dragLastClientX, this.dragLastClientY);
  if (hit && hit.paragraphIndex < 0xFFFFFF00) {
    // [Issue #669] 셀 내부 드래그: anchor와 같은 셀 컨텍스트인 경우만 커서 이동.
    const sel = this.cursor.getSelection();
    if (sel) {
      const anchorInCell = sel.anchor.parentParaIndex !== undefined;
      const hitInSameCell = anchorInCell &&
        hit.parentParaIndex === sel.anchor.parentParaIndex &&
        hit.controlIndex === sel.anchor.controlIndex &&
        hit.cellIndex === sel.anchor.cellIndex;
      if (anchorInCell && !hitInSameCell) {
        return;  // 셀 내 선택 유지
      }
    }
    this.cursor.moveTo(hit);
    this.updateCaretDuringDrag();
  }
}
```

### 3.2 `input-handler-mouse.ts` 영역 주석 갱신
[Issue #669] 셀 가드는 input-handler.ts 의 래퍼 내부에 적용됨.

## 4. 본 환경 충돌 수동 해결

### 4.1 본질
- PR #795 base = `30351cdf` (5/9 시점) — onMouseMove 영역 영역 직접 `hit + moveTo + 셀 가드`
- HEAD (devel) 영역 영역 PR #718 (Task #661, 5/9 머지) `updateTextSelectionDragFromPointer()` 래퍼 사용 (PR #693 의 직접 hit + moveTo 영역 영역 래퍼 영역 포함)
- → 충돌: 같은 영역 (onMouseMove rAF 콜백) 영역 영역 두 본질 (래퍼 호출 vs 직접 hit + 셀 가드)

### 4.2 해결 — 본질 정합 영역 영역 셀 가드 래퍼 영역 영역 이전

| 영역 | HEAD (devel) | incoming (PR #795) | 본 환경 해결 |
|------|--------------|---------------------|--------------|
| `onMouseMove` rAF 콜백 | `updateTextSelectionDragFromPointer()` 호출 (PR #718) | 직접 `hit + moveTo + 셀 가드` | HEAD 보존 (래퍼 호출) + 주석 갱신 |
| `input-handler.ts:1014` 래퍼 | `hit + cursor.moveTo + updateCaretDuringDrag` | (해당 영역 부재) | **셀 가드 14 라인 이전 적용** |

→ PR #718 정합성 보존 + Issue #669 셀 가드 본질 적용.

## 5. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `this.cursor.getSelection()` (기존) | anchor / focus 점검 |
| `hitTestFromClientPoint` (기존) | 드래그 위치 → 셀 컨텍스트 판별 |
| `updateTextSelectionDragFromPointer` 래퍼 (PR #718 인프라) | 셀 가드 영역 영역 정합 위치 |

→ 신규 인프라 도입 부재.

## 6. 영역 좁힘 (회귀 부재 가드)

- anchor 셀 내부 (`parentParaIndex` 있음) 일 때만 발동
- 본문 드래그 (anchor 본문) 변경 부재 — 기존 동작 보존
- 같은 셀 컨텍스트 내 드래그 정상 동작
- 다른 셀 / 본문 영역 드래그 시 커서 이동만 건너뜀 (anchor 셀 유지)
- PR #718 자동 스크롤 영역 영역 래퍼 호출 보존 영역 영역 정합성 유지
- PR #781 scrollbar release 회귀 부재 (다른 위치)

## 7. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (충돌 수동 해결) | ✅ 셀 가드 영역 영역 래퍼 안 적용 |
| `tsc --noEmit` | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |
| WASM 빌드 (Docker, 재빌드) | ✅ 4.68 MB |

## 8. 작업지시자 웹 에디터 시각 검증 ✅ 통과
- 표 셀 내부 텍스트 드래그 선택 → 선택 하이라이트 정상 (Issue #669 정정)
- 본문 드래그 선택 회귀 부재
- 셀 내부 → 다른 셀/본문 드래그 → 커서 셀 내 유지

## 9. 영향 범위

### 9.1 변경 영역
- `rhwp-studio/src/engine/input-handler.ts` 영역 영역 `updateTextSelectionDragFromPointer` 래퍼 안 셀 가드 (+13)
- `rhwp-studio/src/engine/input-handler-mouse.ts` 영역 영역 주석 갱신 (+1)

### 9.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 입증)

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 5번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (cursor.getSelection + hitTest + PR #718 래퍼) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | anchor 셀 내부 영역 영역 case 가드 (`anchorInCell && !hitInSameCell`) 영역 영역 회귀 위험 좁힘 |
| `feedback_diagnosis_layer_attribution` | 본질 진단 — 셀↔본문 혼합 선택 영역 영역 selectionRenderer.clear 호출 본질 정확 |
| `feedback_visual_judgment_authority` | 작업지시자 웹 에디터 시각 검증 ✅ 통과 |
| `feedback_pr_supersede_chain` | PR #718 (Task #661 드래그 자동 스크롤) + PR #781 (drag-during-scroll) → PR #795 (셀 컨텍스트 이탈 방지) 영역 영역 드래그 인터랙션 점진적 진전 |

## 11. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #669 close 완료

---

작성: 2026-05-11
