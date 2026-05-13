---
PR: #811
제목: feat — F5 본문 블록 선택 모드 + F3 영역 확장 선택 (closes #220)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 11번째 PR
base / head: devel / contrib/f5-f3-block-select
mergeStateStatus: DIRTY
mergeable: CONFLICTING
CI: ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff
변경 규모: +120 / -3, 3 files
커밋: 2
검토일: 2026-05-11
---

# PR #811 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #811 |
| 제목 | feat: F5 본문 블록 선택 모드 + F3 영역 확장 선택 (#220) |
| 컨트리뷰터 | @oksure (Hyunwoo Park) — 20+ 사이클 (5/11 사이클 **11번째 PR**) |
| base / head | devel / contrib/f5-f3-block-select |
| mergeable | CONFLICTING (DIRTY — 3 파일 충돌) |
| CI | ✅ 전 항목 통과 |
| 변경 규모 | +120 / -3, 3 files |
| 커밋 수 | 2 (1 본질 + 1 Copilot 리뷰 반영) |
| closes | #220 |

## 2. 본질 (Issue #220)

한컴 HWP F5/F3 블록 선택 영역 영역 rhwp-studio 영역 정합.

### F5 블록 선택 모드 (본문)
- 본문 영역 F5: 현재 위치 anchor 설정 + 블록 선택 모드 진입
- 이후 화살표 키 영역 영역 Shift 없이 선택 확장
- F5 재입력 / Esc: 모드 해제 + 선택 해제
- 표 셀 영역 영역 기존 F5 셀 선택 모드 유지 (`isInCell()` 분기)

### F3 선택 영역 확장
| 입력 횟수 | 확장 범위 |
|-----------|----------|
| F3 1회 | 현재 위치 단어 |
| F3 2회 | 현재 문단 전체 |
| F3 3회 | 현재 구역 전체 |
| F3 4회 | 문서 전체 |

⚠️ **Issue #220 영역 영역 5단계 명시** (단어 → **문장** → 문단 → 섹션 → 문서) — 본 PR 영역 4단계 (문장 단계 누락). 후속 점검 필요 영역 영역 사항.

## 3. 정정 본질 — 3 files, +120/-3

### 3.1 `cursor.ts` (+85, 신규 메서드 + 헬퍼)

**상태 변수**:
```typescript
private _blockSelectionMode = false;
private _expandPhase = 0; // 0=none, 1=word, 2=paragraph, 3=section, 4=document
```

**메서드**:
- `isInBlockSelectionMode()` — 상태 점검
- `enterBlockSelectionMode()` — `anchor = { ...position }` 설정 + phase 0
- `exitBlockSelectionMode()` — anchor null + phase 0
- `expandSelection()` — phase 증가 + 각 단계별 anchor/position 설정

**헬퍼 (파일 끝)**:
- `isWordChar(c)` — Digit/A-Z/a-z/Hangul (AC00-D7AF) / Hangul Jamo (3131-318E) 영역 영역 단어 문자 판정
- `findWordAt(text, offset)` — atWord 여부 영역 양방향 확장 단어 경계 검색

### 3.2 `input-handler-keyboard.ts` (+30/-2)

**F5 분기**:
```typescript
if (e.key === 'F5') {
  e.preventDefault();
  if (this.cursor.isInCell() && !this.cursor.isInTextBox()) {
    // 기존 셀 선택 모드 (분기 보존)
  } else {
    // 본문 블록 선택 모드 토글
    if (this.cursor.isInBlockSelectionMode()) {
      this.cursor.exitBlockSelectionMode();
      this.selectionRenderer.clear();
      this.updateCaret();
    } else {
      this.cursor.enterBlockSelectionMode();
      this.updateSelection();
    }
  }
  return;
}
```

**F3 분기**:
```typescript
if (e.key === 'F3') {
  e.preventDefault();
  if (!this.cursor.isInBlockSelectionMode()) {
    this.cursor.enterBlockSelectionMode();  // F3 첫 입력 시 자동 진입
  }
  this.cursor.expandSelection();
  this.updateSelection();
  return;
}
```

**Escape 분기**:
```typescript
if (this.cursor.isInBlockSelectionMode() && e.key === 'Escape') {
  e.preventDefault();
  this.cursor.exitBlockSelectionMode();
  this.selectionRenderer.clear();
  this.updateCaret();
  return;
}
```

### 3.3 `wasm-bridge.ts` (+5) — `getSectionCount` 추가

⚠️ **PR #808 영역 영역 이미 추가됨** (devel HEAD 영역 영역 라인 231) — 본 PR 영역 영역 중복. 충돌 정합 영역 영역 devel 측 보존.

### 3.4 Copilot 리뷰 반영 commit (`c4874f8c`)
`enterBlockSelectionMode` 영역 영역 anchor 강제 설정 (`this.anchor = { ...this.position }`) — 모드 진입 시점 anchor 정합 (이후 화살표 이동 시 선택 확장 정합).

## 4. 본 환경 충돌 분석

### 4.1 3 파일 충돌

| 파일 | base | our (devel) | their (PR) |
|------|------|-------------|------------|
| `rhwp-studio/src/core/wasm-bridge.ts` | 874d01a0 | 773b03db | 25dc7ed8 |
| `rhwp-studio/src/engine/cursor.ts` | 2bcec9b7 | 0119e037 | 6893b971 |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | bfb1a4c4 | 3bb50958 | 4be7cb15 |

### 4.2 정합 전략

**`wasm-bridge.ts`** — PR #808 영역 `getSectionCount` 이미 노출 → 본 PR 의 중복 추가본 무시 (devel 측 보존).

**`cursor.ts`** — devel 측 영역 PR #794 (Alt+Arrow 단어 이동, `moveToWordBoundary`) + PR #807 (`atLineEnd`) + PR #808 (`moveToParagraphBoundary`) 누적. PR #811 영역 영역 신규 영역 영역 (블록 선택 상태 변수 + 메서드 + 헬퍼). 정합 영역 영역 양측 모두 보존.

**`input-handler-keyboard.ts`** — devel 측 영역 PR #794 (Alt+Arrow 분기) + PR #808 (Ctrl+↑/↓ 자기 정정 분기) 누적. PR #811 영역 영역 F5/F3 분기 추가. 정합 영역 영역 양측 모두 보존.

## 5. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `isInCell()` / `isInTextBox()` (기존) | F5 분기 (셀 vs 본문) |
| `enterCellSelectionMode` / `advanceCellSelectionPhase` (기존) | 표 셀 영역 분기 보존 |
| `cursor.anchor` / `cursor.position` (기존) | 블록 선택 anchor/focus |
| `selectionRenderer.clear` (기존) | 선택 해제 표시 |
| `wasm.getParagraphLength` / `getParagraphCount` / `getTextRange` (기존) | 단어/문단/구역 범위 |
| `wasm.getSectionCount` (PR #808 영역 영역 추가) | 문서 전체 범위 |

→ 신규 인프라 도입 부재 — 기존 cursor / WASM / selection 영역 영역 블록 선택 상태 + 헬퍼 추가만.

## 6. 영역 좁힘 (회귀 부재 가드)

- 표 셀 영역 F5 — 기존 셀 선택 모드 유지 (`isInCell() && !isInTextBox()` 분기 우선)
- 블록 모드 진입 시 `anchor = { ...position }` 강제 설정 (Copilot 리뷰)
- F5 토글 (재입력 영역 영역 해제) + Esc 해제 영역 영역 일관성
- 블록 모드 아닌 상태 영역 영역 F5/F3 영역 영역 회귀 부재 (early return + 신규 모드 진입)
- F3 첫 입력 시 자동 블록 모드 진입 — 사용자 영역 영역 F5 누락 영역 영역 정합

## 7. ⚠️ 점검 영역

### 7.1 Issue #220 영역 5단계 vs PR 4단계
Issue #220 영역 영역 5단계 (단어 → **문장** → 문단 → 섹션 → 문서) 명시 — PR 영역 영역 4단계 (문장 단계 누락). 한컴 HWP F3 표준 정합 영역 영역 문장 단계 영역 영역 후속 추가 필요.

작업지시자께 결정 권장:
- **옵션 1**: 4단계 영역 영역 머지 (문장 영역 영역 후속 PR/별 Issue 분리)
- **옵션 2**: 본 PR 영역 영역 5단계 (문장) 영역 영역 추가 요청 후 재머지

### 7.2 expandSelection 영역 영역 phase 4 cap
- phase 4 이상 영역 영역 `this._expandPhase = 4` 영역 영역 cap — 추가 F3 입력 영역 영역 문서 전체 유지
- phase 4 후 F3 → 변경 부재 (재 호출만 무영향)

### 7.3 selectionRenderer 갱신 점검
PR 영역 영역 `updateSelection()` 호출 영역 영역 selection 표시 — 본 환경 기존 `updateSelection` 영역 영역 anchor/focus 영역 영역 selection 렌더링 정합 영역 영역 점검 필요.

## 8. CI 통과

✅ Build & Test + CodeQL (js-ts / python / rust) + Canvas visual diff

## 9. 처리 옵션

### 옵션 A (권장) — 2 commits cherry-pick + 3 파일 충돌 수동 해결 + no-ff merge

```bash
git checkout local/devel
git cherry-pick 55cb52c4 c4874f8c
# 3 파일 충돌 수동 해결:
#   - wasm-bridge.ts: PR #808 측 보존 (incoming 중복 추가 무시)
#   - cursor.ts: devel + PR #811 양측 보존
#   - input-handler-keyboard.ts: devel + PR #811 양측 보존
git checkout devel
git merge local/devel --no-ff
```

Issue #220 의 5단계 (문장) 영역 영역 별 Issue 분리 후속.

### 옵션 B — squash cherry-pick + 충돌 수동 해결

### 옵션 C — 컨트리뷰터에 문장 단계 추가 요청 후 재제출

본 환경 영역 영역 옵션 A 권장 — 4단계 영역 영역 본질 정합 + 문장 단계 영역 별 Issue 후속.

## 10. 검증 게이트

### 10.1 자기 검증
- [ ] cherry-pick 2 commits + 3 파일 충돌 수동 해결
- [ ] tsc --noEmit
- [ ] cargo test (Rust 변경 부재 영역 영역 회귀 자명)
- [ ] WASM 재빌드 불필요 (TypeScript 단일)

### 10.2 시각/인터랙션 판정 게이트 — **작업지시자 인터랙션 검증 권장**
- 본문 영역 F5 → 블록 선택 모드 + anchor 설정 → 화살표 이동 시 선택 확장
- F5 재입력 → 모드 해제 + 선택 해제
- Esc → 모드 해제 + 선택 해제
- F3 1회 → 단어 선택
- F3 2회 → 문단 전체
- F3 3회 → 구역 전체
- F3 4회 → 문서 전체
- F3 5회 → 문서 전체 유지 (phase 4 cap)
- 표 셀 영역 F5 — 기존 셀 선택 모드 동작 보존
- 일반 텍스트 입력 / 기존 단축키 회귀 부재

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 11번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (cursor / WASM / selection) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | F5 영역 영역 `isInCell() && !isInTextBox()` 분기 영역 영역 영역 좁힘 — 표 셀 영역 영역 기존 셀 선택 모드 보존 |
| `feedback_diagnosis_layer_attribution` | F5 본문 블록 vs F5 셀 선택 두 본질 분리 (분기 패턴) |
| `feedback_visual_judgment_authority` | F5/F3 블록 선택 영역 영역 작업지시자 인터랙션 검증 권장 |
| `feedback_pr_supersede_chain` | PR #758 (F5 표 셀 영역 영역 도입) → **PR #811** (본문 F5 + F3 영역 확장) — 한컴 표준 단계적 진전 |

## 12. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick 2 commits + 3 파일 충돌 수동 해결
2. 자기 검증 (tsc + cargo test)
3. 작업지시자 웹 에디터 인터랙션 검증 (F5 본문/셀 + F3 4단계 + 회귀 부재)
4. 검증 통과 → no-ff merge + push + archives + 5/11 orders + Issue #220 close 점검 (5단계 영역 영역 4단계 영역 영역 close 가능 여부)
5. PR #811 close
6. (선택) 문장 단계 영역 별 Issue 생성

---

작성: 2026-05-11
