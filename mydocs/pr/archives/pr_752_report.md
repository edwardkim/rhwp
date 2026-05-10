---
PR: #752
제목: feat — edit:delete (지우기) 커맨드 구현 (선택 영역/개체 삭제)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 20번째 PR)
처리: 옵션 A — 2 commits cherry-pick + 자기 정정 commit + no-ff merge
처리일: 2026-05-10
머지 commit: 3eb10f4e
---

# PR #752 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + 자기 정정 commit + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `3eb10f4e` (--no-ff merge) |
| Cherry-pick commits | `d7928e13` (feat) + `3b5f6965` (Copilot 리뷰) |
| 자기 정정 commit | `2aafebc8` (devel HEAD 잠재 결함 정정) |
| Issue 연결 | 부재 (편집 메뉴 stub 활성화 영역 자기완결) |
| 시각 판정 | ✅ 작업지시자 웹 에디터 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN + sweep 170/170 same + WASM 4.66 MB |

## 2. 정정 본질 — 3 files, +42/-2 (PR) + 자기 정정 +3/-3

### 2.1 `rhwp-studio/src/engine/input-handler.ts` (+36, performDelete 신규)

`performCut` 패턴 정합 — 클립보드 복사만 제외:

| 상태 | 동작 |
|------|------|
| 그림/도형 개체 선택 | snapshot + `deleteObjectControl(ref)` |
| 표 개체 선택 (cellPath.length ≤ 1) | snapshot + `deleteTableControl` |
| 중첩 표 선택 (cellPath.length > 1) | 선택 해제만 (삭제 차단 가드) |
| 텍스트 선택 | `deleteSelection()` |
| 선택 없음 | `canExecute` 차단 |

### 2.2 `rhwp-studio/src/command/commands/edit.ts` (+4/-2)
stub `() => false` → 실제 `canExecute` + `performDelete` 호출.

### 2.3 `rhwp-studio/src/command/shortcut-map.ts` (+2)
`Ctrl+E` 단축키 매핑 추가.

### 2.4 자기 정정 commit `2aafebc8` (+3/-3)
`input-handler.ts:1831/1836/1841` 의 `getObjectProperties` / `setObjectProperties` / `deleteObjectControl` 래퍼 시그니처 영역 영역 `'line'` 타입 추가 — devel HEAD 잠재 결함 정정.

**원인**: `getSelectedPictureRef` 시그니처 (`input-handler.ts:1819`) + `_picture.deleteObjectControl` 실제 구현 영역 영역 `'line'` 포함 영역 영역, **input-handler.ts 의 래퍼 3개** 영역 영역 `'line'` 누락. PR #752 의 신규 `this.deleteObjectControl(ref)` 호출 영역 영역 tsc 오류로 노출.

## 3. Copilot 리뷰 반영 (commit `3b5f6965`)
- `deleteObjectControl()` 헬퍼 사용 (image/equation→deletePictureControl, shape/group/line→deleteShapeControl 분기)
- 중첩 표 (`cellPath.length > 1`) 삭제 시도 차단 — 선택 해제만 수행

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `executeOperation({ kind: 'snapshot' })` (PR #728 인프라) | snapshot Undo 기록 |
| `deleteSelection()` (기존 private 메서드) | 텍스트 선택 영역 삭제 |
| `deleteObjectControl()` 헬퍼 (기존) | image/equation/shape/group/line 분기 삭제 |
| `deleteTableControl` WASM API (기존) | 표 개체 삭제 |
| `cursor.moveOutOfSelectedPicture/Table` (기존) | 개체 선택 해제 |
| EditorContext 가드 (기존) | `canExecute` 가드 |
| `performCut` 패턴 (기존) | `performDelete` 동일 구조 |

→ 신규 인프라 도입 부재 영역 영역 위험 좁힘 (`feedback_process_must_follow` 정합).

## 5. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (2 commits) | ✅ auto-merge 충돌 0건 |
| `tsc --noEmit` (자기 정정 전) | ❌ 1 오류 (line 2399, deleteObjectControl 타입 미정합) |
| 자기 정정 commit | ✅ `2aafebc8` (래퍼 3개 영역 영역 `'line'` 추가) |
| `tsc --noEmit` (자기 정정 후) | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |
| WASM 빌드 (Docker) | ✅ 4.66 MB |

## 6. 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과
- 텍스트 선택 → Ctrl+E → 선택 영역 삭제 (Undo 가능)
- 그림 선택 → Ctrl+E → 그림 삭제 (Undo 가능)
- 표 선택 → Ctrl+E → 표 삭제 (Undo 가능)
- 중첩 표 선택 → Ctrl+E → 선택 해제만 (삭제 차단)
- 메뉴 → "편집 → 지우기" → 동일 동작

## 7. 영향 범위

### 7.1 변경 영역
- rhwp-studio editor 영역 영역 편집 커맨드 (TypeScript 단일 영역)

### 7.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 입증)

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 20번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (`performCut` 패턴 + `deleteObjectControl` 헬퍼 + `executeOperation snapshot`) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | 중첩 표 가드 (`cellPath.length > 1`) — 일반화 영역 영역 결함 위험 좁힘 |
| `feedback_visual_judgment_authority` | 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과 |

## 9. 자기 정정 commit 패턴 (PR #740 정합)

devel HEAD 영역 영역 잠재 결함 (래퍼 시그니처 영역 `'line'` 누락) — PR #752 의 신규 호출 영역 영역 노출 → 본 환경 자기 정정 commit 으로 해결. PR #740 자기 정정 패턴 정합 (`feedback_diagnosis_layer_attribution` 정합 — PR 본질 영역 영역 분리된 잠재 결함을 본 환경 영역 영역 함께 처리).

## 10. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- 자기 정정으로 devel HEAD 잠재 결함도 해결

---

작성: 2026-05-10
