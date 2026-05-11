---
PR: #795
제목: fix — 표 셀 내부 드래그 선택 시 셀 컨텍스트 이탈 방지 (closes #669)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/11 사이클 5번째 PR)
base / head: devel / contrib/table-cell-drag-select
mergeStateStatus: DIRTY
mergeable: CONFLICTING
CI: 결과 부재
변경 규모: +14 / -0, 1 file
검토일: 2026-05-11
---

# PR #795 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #795 |
| 제목 | fix — 표 셀 내부 드래그 선택 시 셀 컨텍스트 이탈 방지 |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/11 사이클 5번째 PR — PR #794 후속) |
| base / head | devel / contrib/table-cell-drag-select |
| mergeStateStatus | DIRTY, mergeable: CONFLICTING |
| CI | 결과 부재 |
| 변경 규모 | +14 / -0, 1 file (input-handler-mouse.ts) |
| 커밋 수 | 1 |
| closes | #669 |

## 2. 본질 (Issue #669)

표 셀 내부 텍스트 드래그 선택 시 선택이 보이지 않고 커서만 이동.

### 원인 분석 (PR 본문)
드래그 중 `hitTest`가 셀 내부 빈 영역 (텍스트 라인 외부) 에서 본문 레벨 위치를 반환:
- anchor: 셀 내부 (`parentParaIndex` 있음)
- focus: 본문 (`parentParaIndex` 없음)

`updateSelection()` 에서 "셀↔본문 혼합 선택" 으로 판별 → `selectionRenderer.clear()` 호출 → 선택 하이라이트 미렌더링.

## 3. 정정

`onMouseMove` 핸들러 (`input-handler-mouse.ts:1073`) 에서 anchor가 셀 내부일 때, hit가 같은 셀 컨텍스트가 아니면 커서 이동 건너뜀:

```typescript
const sel = this.cursor.getSelection();
if (sel) {
  const anchorInCell = sel.anchor.parentParaIndex !== undefined;
  const hitInSameCell = anchorInCell &&
    hit.parentParaIndex === sel.anchor.parentParaIndex &&
    hit.controlIndex === sel.anchor.controlIndex &&
    hit.cellIndex === sel.anchor.cellIndex;
  if (anchorInCell && !hitInSameCell) {
    return;  // anchor가 셀이지만 hit가 다른 위치 → 무시
  }
}
this.cursor.moveTo(hit);
this.updateCaret();
```

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `this.cursor.getSelection()` (기존) | anchor / focus 점검 |
| `hitTest` (기존) | 드래그 위치 → 셀 컨텍스트 판별 |
| `onMouseMove` 드래그 핸들러 (기존) | 가드 추가 |

→ 신규 인프라 도입 부재.

## 5. 영역 좁힘 (회귀 부재 가드)

- anchor가 셀 내부 (`parentParaIndex` 있음) 일 때만 발동
- 본문 드래그 선택 (anchor 본문) 경로는 변경 부재 — 기존 동작 보존
- 같은 셀 컨텍스트 내 드래그는 정상 동작
- 다른 셀 / 본문 영역 드래그 시 커서 이동만 건너뜀 (anchor 셀 유지)

## 6. 충돌 분석

### 본질
PR #795 base = `30351cdf` (5/9 시점). devel HEAD 영역 영역 PR #781 (scrollbar release back-scroll, 5/11 머지) input-handler-mouse.ts 영역 영역 누적 변경 → 충돌.

### 해결 방식
- input-handler-mouse.ts 영역 영역 `onMouseUp` 끝 `updateCaret(true)` (PR #781) 영역 영역 다른 위치 (`onMouseMove`) 영역 영역 본 PR 영역 영역 다른 영역 영역 작은 충돌만 예상
- 옵션 A 시도 → 충돌 발생 시 수동 해결

## 7. 본 환경 점검

### 변경 격리
- TypeScript 단일 파일 단일 위치 (input-handler-mouse.ts `onMouseMove` 영역)
- Rust / WASM / 렌더링 경로 무관

### CI 결과 부재
mergeable=CONFLICTING 영역 영역 CI 미실행. cherry-pick 후 자기 검증 필요.

## 8. 처리 옵션

### 옵션 A — 1 commit cherry-pick + 충돌 수동 해결 + no-ff merge

```bash
git checkout local/devel
git cherry-pick 49361b64  # 충돌 발생 가능
# 수동 해결 (PR #781 onMouseUp 영역 영역 별 영역)
git checkout devel
git merge local/devel --no-ff
```

→ **권장**.

## 9. 검증 게이트

### 자기 검증
- [ ] cherry-pick 충돌 수동 해결
- [ ] tsc + cargo test ALL GREEN
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (TypeScript 영역 SVG 무영향 자명)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 본질 영역 영역 rhwp-studio editor 표 셀 드래그 선택:
- WASM 빌드 후 dev server 영역 영역:
  - 표 셀 내부 텍스트 드래그 선택 → 선택 하이라이트 정상 렌더링 (Issue #669 정정)
  - 본문 드래그 선택 회귀 부재
  - 셀 내부 → 다른 셀 / 본문 영역 드래그 → 커서 셀 내 유지 (선택 보존)
  - PR #781 (scrollbar drag) 회귀 부재

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/11 사이클 5번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (cursor.getSelection + hitTest + onMouseMove 가드) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | anchor 셀 내부 영역 영역 case 가드 (`anchorInCell && !hitInSameCell`) 영역 영역 회귀 위험 좁힘 |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 영역 작업지시자 인터랙션 검증 권장 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick `49361b64` + 충돌 수동 해결
2. 자기 검증 (tsc + cargo test + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (표 셀 드래그 선택 + 본문 드래그 회귀 부재 + PR #781 회귀 부재)
4. 인터랙션 검증 통과 → no-ff merge + push + archives + 5/11 orders + Issue #669 close
5. PR #795 close

---

작성: 2026-05-11
