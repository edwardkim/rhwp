---
PR: #812
제목: fix — master page 글상자 더블클릭 시 첫 페이지 jump 방지 (closes #686)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 12번째 PR
처리: 옵션 A — 2 commits cherry-pick + no-ff merge
처리일: 2026-05-11
머지 commit: 744ca7b0
---

# PR #812 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `744ca7b0` (--no-ff merge) |
| Cherry-pick commits | 2 (본질 + Copilot 리뷰 반영) |
| closes | #686 |
| 시각 판정 | ✅ 작업지시자 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN |
| WASM 재빌드 | 불필요 (TypeScript 단일) |

## 2. 본질 (Issue #686)

master page 자동번호 글상자 (`sec=0, para=0, control=0` anchored — 모든 페이지 반복 표시) 더블클릭 시 viewport 영역 영역 첫 페이지로 점프 (e2e 측정 scroll -11288px).

### 결함 흐름
1. 페이지 16 자동번호 글상자 더블클릭
2. `enterTextboxEditing` 영역 cursor 영역 master page anchor (sec=0, para=0) 이동
3. `cursor.getRect().pageIndex = 0` (master page 좌표)
4. `scrollCaretIntoView` 영역 page 0 부근 점프

### 한컴 정합
한컴 영역 영역 master page 글상자 더블클릭 시 텍스트 편집 모드 진입 부재 — 본문 마지막 caret fallback.

## 3. 정정 본질 — `input-handler-mouse.ts` +12 (1 file)

### 3.1 `onDblClick` 영역 ppi=0 가드
```typescript
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

### 3.2 2중 가드 (영역 좁힘)
- `ref.ppi === 0` — master page anchor 식별
- `cursorPage !== 0` — page 0 외 영역 영역만 차단

### 3.3 Copilot 리뷰 반영 commit (`eab22908`)
- 주석 정정 — master page anchor 본질 명확화
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

- `ref.ppi === 0` 영역 영역만 가드 — 일반 글상자 (ppi != 0) 영역 기존 텍스트 편집 진입 보존
- `cursorPage !== 0` 영역 영역만 차단 — page 0 자체 영역 정상 진입 정합
- early return — 기존 흐름 변경 부재
- 다른 ref.type (picture / table / equation) 영역 영영 영향 부재

## 6. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 2 commits | ✅ auto-merge 충돌 0건 |
| `tsc --noEmit` | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep | 면제 (TypeScript 단일 영역 영역 SVG 무영향 자명) |
| WASM 재빌드 | 불필요 |

## 7. 작업지시자 인터랙션 검증 ✅ 통과

- `samples/hwpctl_Action_Table__v1.1.hwp` (16p) 페이지 16 자동번호 글상자 더블클릭 — 점프 부재 + 선택 해제
- 페이지 1 자동번호 글상자 더블클릭 — page 0 정상 진입 (가드 부재)
- 일반 글상자 (ppi != 0) 더블클릭 — 기존 텍스트 편집 진입 보존
- 다른 객체 (picture / table / equation) 더블클릭 — 회귀 부재
- 기존 마우스 인터랙션 (PR #795 셀 드래그 등) 회귀 부재

## 8. CI 통과

✅ Build & Test + CodeQL (js-ts / python / rust) + Canvas visual diff

## 9. 영향 범위

### 9.1 변경 영역
- `rhwp-studio/src/engine/input-handler-mouse.ts` (+12)

### 9.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (TypeScript 단일 영역 SVG 무영향)
- 일반 글상자 더블클릭 (ppi != 0)
- page 0 자체 영역 master page 글상자 더블클릭 (cursorPage === 0)
- 다른 객체 (picture / table / equation)

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 12번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (cursor / pictureObject / eventBus) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | `ref.ppi === 0 && cursorPage !== 0` 2중 가드 영역 영역 영역 좁힘 — 일반 글상자 + page 0 영역 회귀 부재 |
| `feedback_diagnosis_layer_attribution` | master page anchor (ppi=0) 영역 모든 페이지 반복 표시 + cursor pageIndex = 0 본질 정확 진단 (Issue #686 e2e 측정 명시) |
| `feedback_visual_judgment_authority` | 작업지시자 인터랙션 검증 ✅ 통과 |
| `feedback_pr_supersede_chain` | PR #795 (셀 드래그 보호) → **PR #812** (master page 글상자 더블클릭 jump 방지) — 마우스 인터랙션 단계적 진전 |

## 11. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #686 close 완료

---

작성: 2026-05-11
