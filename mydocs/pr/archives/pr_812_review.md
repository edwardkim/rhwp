---
PR: #812
제목: fix — master page 글상자 더블클릭 시 첫 페이지 jump 방지 (closes #686)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 12번째 PR
base / head: devel / contrib/master-page-dblclick-fix
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff
변경 규모: +12 / -0, 1 file
커밋: 2
검토일: 2026-05-11
---

# PR #812 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #812 |
| 제목 | fix: master page 글상자 더블클릭 시 첫 페이지 jump 방지 (#686) |
| 컨트리뷰터 | @oksure (Hyunwoo Park) — 20+ 사이클 (5/11 사이클 **12번째 PR**) |
| base / head | devel / contrib/master-page-dblclick-fix |
| mergeable | MERGEABLE (BEHIND — base 갱신만) |
| CI | ✅ 전 항목 통과 |
| 변경 규모 | +12 / -0, 1 file |
| 커밋 수 | 2 (1 본질 + 1 리뷰 반영) |
| closes | #686 |
| fixture | `samples/hwpctl_Action_Table__v1.1.hwp` 16p |

## 2. 본질 (Issue #686)

master page 자동번호 글상자 (`sec=0, para=0, control=0` anchored — 모든 페이지 표시) 더블클릭 시 viewport 영역 영역 첫 페이지로 점프.

### 결함 흐름 (e2e 측정, Issue #686 명시)
1. 페이지 16 자동번호 글상자 더블클릭
2. `enterTextboxEditing` 영역 영역 cursor 영역 `sec=0, para=0, control=0, cellIdx=0` 으로 이동 (master page anchor)
3. `cursor.getRect().pageIndex = 0` (master page anchor 영역 영역 page 0 좌표)
4. `scrollCaretIntoView` 영역 영역 page 0 부근 (scroll −11288px) 영역 영역 점프

### 한컴 정합
한컴 영역 영역 master page 글상자 더블클릭 시 텍스트 편집 모드 진입 부재 — 본문 마지막 caret 으로 fallback.

## 3. 정정 본질 — `input-handler-mouse.ts` +12 (1 file)

### 3.1 `onDblClick` 영역 영역 ppi=0 가드 추가
```typescript
// #686: ppi=0 앵커 도형 (master page 글상자 등)은 모든 페이지에 반복 표시됨.
// 텍스트 진입 시 cursor가 page 0으로 잡혀 뷰가 점프하므로, page 0이 아닐 때 차단.
if (ref.ppi === 0) {
  const cursorPage = this.cursor.getRect()?.pageIndex ?? -1;
  if (cursorPage !== 0) {
    this.cursor.exitPictureObjectSelection();
    this.pictureObjectRenderer?.clear();
    this.eventBus.emit('picture-object-selection-changed', false);
    this.textarea.focus();
    return;
  }
}
```

- `ref.ppi === 0` 영역 영역 master page anchor 영역 영역 식별
- `cursorPage !== 0` 영역 영역 현재 페이지 0 외 영역 영역만 차단 (page 0 자체 영역 영역 정상 진입 정합)
- 텍스트 편집 진입 차단 + 선택 해제 + focus 복원 + return

### 3.2 Copilot 리뷰 반영 commit (`87452378`)
ppi=0 가드 개선:
- 주석 정정 — 본 환경 master page anchor 본질 명확화
- updateCaret 제거 — 텍스트 편집 진입 차단 영역 영역 caret 갱신 불필요

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `cursor.getRect().pageIndex` (기존) | 현재 cursor 페이지 점검 |
| `cursor.exitPictureObjectSelection` (기존) | 객체 선택 모드 해제 |
| `pictureObjectRenderer?.clear` (기존) | 선택 표시 해제 |
| `eventBus.emit('picture-object-selection-changed', false)` (기존) | 상태 동기 |
| `textarea.focus()` (기존) | focus 복원 |

→ 신규 인프라 도입 부재 — 기존 인프라 영역 영역 분기 가드만 추가.

## 5. 영역 좁힘 (회귀 부재 가드)

- **`ref.ppi === 0` 영역 영역만 가드** — 일반 글상자 (ppi != 0) 영역 영역 기존 텍스트 편집 진입 보존
- **`cursorPage !== 0` 영역 영역만 차단** — page 0 자체 영역 영역 정상 진입 정합 (첫 페이지 영역 영역 master page anchor 영역 영역 충돌 부재)
- early return — 기존 흐름 변경 부재
- 다른 ref.type (picture / table / equation 등) 영역 영역 영향 부재

## 6. 본 환경 점검

### 6.1 변경 격리
- 순수 TypeScript `input-handler-mouse.ts` 단일 — WASM/Rust 변경 부재
- HWP3/HWPX 변환본 시각 정합 (sweep 무영향 자명)

### 6.2 CI 통과
- ✅ Build & Test + CodeQL (js-ts / python / rust) + Canvas visual diff

### 6.3 mergeStateStatus = BEHIND
base 갱신만 필요 — 충돌 부재 (MERGEABLE). PR #795 영역 영역 onDblClick 영역 영역 무관한 부분 영역 영역 cherry-pick auto-merge 예상.

## 7. 처리 옵션

### 옵션 A (권장) — 2 commits cherry-pick + no-ff merge

```bash
git checkout local/devel
git cherry-pick 30eacaa3 87452378
git checkout devel
git merge local/devel --no-ff
```

본질 commit + 리뷰 반영 commit 영역 이력 보존.

### 옵션 B — squash cherry-pick (단일 commit)

본 환경 영역 영역 commit 이력 보존 권장 옵션 A.

## 8. 검증 게이트

### 8.1 자기 검증
- [ ] cherry-pick 2 commits (auto-merge 예상)
- [ ] tsc --noEmit
- [ ] cargo test (Rust 변경 부재 영역 영역 회귀 자명)
- [ ] WASM 재빌드 불필요 (TypeScript 단일)

### 8.2 시각/인터랙션 판정 게이트 — **작업지시자 인터랙션 검증 권장**
- `samples/hwpctl_Action_Table__v1.1.hwp` (16p) 열기
- 페이지 16 자동번호 글상자 (페이지 하단) 영역 영역 더블클릭 → 점프 부재 + 선택 해제
- 페이지 1 영역 영역 자동번호 글상자 더블클릭 → page 0 영역 영역 진입 가능 (cursorPage === 0 영역 영역 가드 부재)
- 일반 글상자 (ppi != 0) 더블클릭 → 기존 텍스트 편집 진입 보존
- 다른 객체 (picture / table / equation) 더블클릭 영역 영역 회귀 부재

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 12번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (기존 cursor / pictureObject / eventBus) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | `ref.ppi === 0 && cursorPage !== 0` 2중 가드 영역 영역 영역 좁힘 — 일반 글상자 영역 페이지 0 영역 영역 회귀 부재 |
| `feedback_diagnosis_layer_attribution` | master page anchor (ppi=0) 영역 영역 모든 페이지 반복 표시 + cursor pageIndex = 0 본질 정확 진단 (Issue #686 e2e 측정 명시) |
| `feedback_visual_judgment_authority` | 작업지시자 인터랙션 검증 권장 (master page 글상자 더블클릭) |
| `feedback_pr_supersede_chain` | PR #795 (셀 드래그 보호) → **PR #812** (master page 글상자 더블클릭 jump 방지) — 마우스 인터랙션 단계적 진전 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick 2 commits (`30eacaa3` + `87452378`)
2. 자기 검증 (tsc + cargo test)
3. 작업지시자 웹 에디터 인터랙션 검증 (master page 글상자 더블클릭)
4. 검증 통과 → no-ff merge + push + archives + 5/11 orders + Issue #686 close
5. PR #812 close

---

작성: 2026-05-11
