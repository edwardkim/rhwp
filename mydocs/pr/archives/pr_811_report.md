---
PR: #811
제목: feat — F5 본문 블록 선택 + F3 영역 확장 (closes #220 partial — 4단계)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 11번째 PR
처리: 옵션 A — 2 commits cherry-pick + 1 충돌 수동 해결 + 중복 제거 commit + no-ff merge
처리일: 2026-05-11
머지 commit: d8e641a4
별 Issue: #839 (F3 문장 단계, Issue #220 5단계 영역 영역 문장 누락)
---

# PR #811 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 PR commits + 1 정정 commit + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `d8e641a4` (--no-ff merge) |
| Cherry-pick commits | 2 PR + 1 정정 (중복 제거) |
| Issue #220 | OPEN 유지 (4단계 정합, 문장 단계 후속) |
| 별 Issue #839 | 신규 — F3 문장 단계 추가 |
| 시각 판정 | ✅ 작업지시자 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN |
| WASM 재빌드 | 불필요 (TypeScript 단일) |

## 2. 본질 (Issue #220)

한컴 HWP F5/F3 블록 선택 영역 영역 rhwp-studio 정합. PR 영역 **4단계 정합** — Issue #220 영역 영역 5단계 (단어 → **문장** → 문단 → 섹션 → 문서) 명시 영역 영역 문장 단계 누락 → 별 Issue #839 분리 후속.

### F5 블록 선택 (본문)
- 본문 F5 → 블록 모드 + anchor 설정 → 화살표 영역 영역 Shift 없이 선택 확장
- F5 재입력 / Esc → 모드 해제
- 표 셀 F5 — 기존 셀 선택 모드 보존 (`isInCell() && !isInTextBox()` 분기)

### F3 4단계 확장
| F3 | 범위 |
|----|------|
| 1회 | 단어 (findWordAt) |
| 2회 | 문단 전체 |
| 3회 | 구역 전체 |
| 4회 | 문서 전체 (phase 4 cap) |

## 3. 정정 본질 — 3 files

### 3.1 `cursor.ts` (+85)

**상태 변수**:
```typescript
private _blockSelectionMode = false;
private _expandPhase = 0; // 0=none, 1=word, 2=paragraph, 3=section, 4=document
```

**메서드**:
- `isInBlockSelectionMode()` — 상태 점검
- `enterBlockSelectionMode()` — `anchor = { ...position }` 설정 (Copilot 리뷰 반영)
- `exitBlockSelectionMode()` — anchor null + phase 0
- `expandSelection()` — phase 증가 + 각 단계별 anchor/position 설정

**헬퍼**:
- `isWordChar(c)` — Digit/A-Z/a-z/Hangul (AC00-D7AF) / Hangul Jamo (3131-318E)
- `findWordAt(text, offset)` — atWord 영역 양방향 단어 경계 검색

### 3.2 `input-handler-keyboard.ts` (+30/-2)

**F5 분기**:
- `isInCell() && !isInTextBox()` → 기존 셀 선택 모드 (분기 보존)
- 본문 → 블록 모드 토글 (enter/exit)

**F3 분기**:
- 블록 모드 아니면 자동 진입
- `expandSelection()` 호출

**Escape 분기**:
- 블록 모드 영역 영역 모드 해제 + 선택 해제

### 3.3 Copilot 리뷰 반영 commit (`59a01de4`)
`enterBlockSelectionMode` 영역 영역 `anchor = { ...this.position }` 강제 설정 — 모드 진입 시점 anchor 정합.

## 4. 본 환경 충돌 수동 해결 (1 파일)

### 4.1 `cursor.ts` 파일 끝 영역 영역 헬퍼 충돌
- **devel 측 (PR #794)**: `findWordBoundaryForward/Backward` — Alt+Arrow 단어 이동 (CharClass 5종 분류)
- **incoming (PR #811)**: `findWordAt` — F3 단어 선택 (단순 isWordChar)

→ **양측 모두 보존** (두 함수 영역 영역 별 본질, 별 호출처)

### 4.2 정정 commit `dc6aa7a4` — getSectionCount 중복 제거

PR #808 (commit `ca729bdc`) 영역 영역 `wasm-bridge.ts:231` 영역 영역 `getSectionCount` 이미 추가됨. PR #811 영역 영역 같은 메서드 영역 영역 재추가 (auto-merge 영역 영역 두 정의 모두 보존). PR #808 측 (`this.doc?.getSectionCount() ?? 0`) 영역 보존 + PR #811 측 제거.

## 5. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `isInCell()` / `isInTextBox()` (기존) | F5 분기 (셀 vs 본문) |
| `enterCellSelectionMode` 등 (PR #758 이전) | 표 셀 영역 분기 보존 |
| `cursor.anchor` / `cursor.position` (기존) | 블록 선택 anchor/focus |
| `selectionRenderer.clear` (기존) | 선택 해제 표시 |
| `wasm.getParagraphLength` / `getParagraphCount` / `getTextRange` (기존) | 단어/문단/구역 범위 |
| `wasm.getSectionCount` (PR #808 영역 영역 추가) | 문서 전체 범위 |

→ 신규 인프라 도입 부재.

## 6. 영역 좁힘 (회귀 부재 가드)

- 표 셀 영역 F5 — 기존 셀 선택 모드 유지 (분기 우선)
- 블록 모드 진입 시 `anchor` 강제 설정 (Copilot 리뷰)
- F5 토글 + Esc 해제 일관성
- 블록 모드 아닌 상태 영역 F5/F3 영역 영역 회귀 부재 (early return + 신규 모드 진입)
- F3 첫 입력 시 자동 블록 모드 진입 — 사용자 F5 누락 영역 영역 정합
- phase 4 cap — 추가 F3 영역 영역 문서 전체 유지

## 7. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 2 commits + 1 충돌 수동 해결 | ✅ |
| 정정 commit (중복 제거) | ✅ |
| `tsc --noEmit` | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep | 면제 (TypeScript 단일 영역 영역 SVG 무영향 자명) |
| WASM 재빌드 | 불필요 |

## 8. 작업지시자 인터랙션 검증 ✅ 통과

- 본문 F5 → 블록 모드 + 화살표 선택 확장
- F5 재입력 / Esc → 모드 해제
- F3 1~4회 → 단어/문단/구역/문서 확장
- F3 5회 이상 → 문서 전체 유지 (phase 4 cap)
- 표 셀 F5 — 기존 셀 선택 모드 보존
- 일반 텍스트 입력 / 기존 단축키 회귀 부재

## 9. CI 통과

✅ Build & Test + CodeQL (js-ts / python / rust) + Canvas visual diff

## 10. ⚠️ 잔존 후속 — F3 문장 단계 (별 Issue #839)

Issue #220 영역 5단계 (단어 → **문장** → 문단 → 섹션 → 문서) 명시 — 본 PR 영역 영역 문장 단계 누락.

별 Issue #839 영역 영역 후속 정정:
- `findSentenceAt(text, offset)` 헬퍼 신규
- phase 매핑 정정 (phase 5 cap)
- 한컴 표준 정합

Issue #220 영역 영역 OPEN 유지 — 별 Issue #839 close 시 동시 close 영역 영역.

## 11. 영향 범위

### 11.1 변경 영역
- `rhwp-studio/src/engine/cursor.ts` (+85)
- `rhwp-studio/src/engine/input-handler-keyboard.ts` (+30/-2)

### 11.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (TypeScript 단일 영역 SVG 무영향)
- 표 셀 F5 셀 선택 모드 (분기 보존)
- 기존 단축키 (Alt+Arrow / Ctrl+Arrow / Ctrl+End 등)

## 12. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 11번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (cursor / WASM / selection) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | F5 영역 `isInCell() && !isInTextBox()` 분기 영역 영역 셀 선택 모드 보존 + Copilot 리뷰 anchor 강제 설정 |
| `feedback_diagnosis_layer_attribution` | F5 본문 블록 vs F5 셀 선택 두 본질 분리 + F3 4단계 vs Issue #220 5단계 (문장 누락) 점검 |
| `feedback_visual_judgment_authority` | 작업지시자 인터랙션 검증 ✅ 통과 |
| `feedback_pr_supersede_chain` | PR #758 (F5 표 셀 도입) → **PR #811** (본문 F5 + F3) → **별 Issue #839** (F3 문장 단계 후속) — 단계적 진전 |
| `feedback_close_issue_verify_merged` | Issue #220 영역 영역 4단계 정합만 → OPEN 유지 — 5단계 완성 시 close 점검 |

## 13. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #220 영역 영역 OPEN 유지 (4단계 정합 + 문장 단계 후속)
- 별 Issue #839 — F3 문장 단계 추가 (한컴 표준 정합)

---

작성: 2026-05-11
