---
PR: #794
제목: feat — Alt/Option+Arrow 단어 단위 커서 이동 (Part of #223)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/11 사이클 4번째 PR)
처리: 옵션 A — 2 commits cherry-pick + 자기 정정 commit + no-ff merge
처리일: 2026-05-11
머지 commit: 76e242c3
---

# PR #794 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + 자기 정정 commit + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `76e242c3` (--no-ff merge) |
| Cherry-pick commits | `6b2fcd78` (feat) + `a32895b1` (Copilot 리뷰) |
| 자기 정정 commit | `7c0418fc` (Alt+Delete 표 안/외 분기) |
| Part of | Issue #223 (macOS 단축키 지원, PR #746 후속) |
| 시각 판정 | ✅ 작업지시자 시각 검증 통과 (자기 정정 후) |
| 자기 검증 | tsc + cargo test ALL GREEN + sweep 170/170 same + WASM 4.68 MB |

## 2. 본 PR 본질 — 4 단축키

| 단축키 | 동작 |
|--------|------|
| Option+←/→ | 단어 단위 커서 이동 |
| Option+Shift+←/→ | 단어 단위 선택 확장 |
| Option+Backspace | 이전 단어 삭제 |
| Option+Delete | 다음 단어 삭제 (표 외 영역) |

## 3. 신규 인프라 — `CursorState.moveToWordBoundary`

`rhwp-studio/src/engine/cursor.ts` 영역 영역:
- 본문 (paragraph): `getTextRange` + `findWordBoundary{Forward,Backward}`
- 표 셀 내부: `getTextInCell` + 동일 헬퍼
- 문자 클래스 5종: 공백 / 한글 (완성형+자모+첫가끝) / Latin / Digit / Punct
- 슬라이스 50 char cap (성능 가드)
- 경계 규칙: 전진 (현재 단어 끝 + 후행 공백 건너뜀) / 후진 (선행 공백 건너뜀 + 이전 단어 시작까지)

## 4. 본 환경 자기 정정 (commit `7c0418fc`)

### 4.1 결함 진단
작업지시자 시각 검증 영역 영역 발견:
- "Alt+Delete 만 동작하지 않습니다."
- "칸 지우기와 단어 지우기가 겹치는 군요!"

본질: `shortcut-map.ts:97` 영역 영역 `Alt+Delete → table:delete-col` 매핑 (5/10 이전 등록) 영역 영역 일반 편집 영역 영역 `dispatcher.dispatch` 영역 영역 silently fail (canExecute=inTable 차단) + return → switch case 'Delete' 영역 영역 도달 부재 → 단어 삭제 미동작.

### 4.2 정정 — `input-handler-keyboard.ts` Alt 조합 가드 분기

```typescript
const isAltWordKey = e.altKey && (
  e.key.startsWith('Arrow') ||
  e.key === 'Backspace' ||
  (e.key === 'Delete' && !this.cursor.isInCell())
);
if (e.altKey && !isAltWordKey && this.dispatcher) {
  // shortcut-map 영역 dispatch
}
```

### 4.3 동작 매트릭스 (정정 후, 옵션 1 작업지시자 결정)

| 위치 | Alt+Backspace | Alt+Delete | Alt+Arrow |
|------|---------------|------------|-----------|
| 표 외 | 이전 단어 삭제 (PR #794) | 다음 단어 삭제 (PR #794) | 단어 이동 (PR #794) |
| **표 안** | 이전 단어 삭제 (PR #794) | **칸 지우기 (table:delete-col, 기존 동작 보존)** | 단어 이동 (PR #794) |

### 4.4 옵션 결정 배경 — 본질 충돌 (`Alt+Delete`)

| 의도 | 출처 | 동작 |
|------|------|------|
| 한컴 표 칸 지우기 | shortcut-map.ts:97 (이전 등록) | 표 안 칸 삭제 |
| macOS 단어 삭제 | PR #794 (Issue #223) | 다음 단어 삭제 |

작업지시자 결정 (옵션 1) — 표 안/외 분기 유지. 위치별 분기 사용자 혼란 trade-off 영역 영역 현재 동작 유지.

## 5. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `getTextRange` / `getTextInCell` WASM API (기존) | 단어 경계 탐색 영역 텍스트 슬라이스 |
| `getParagraphLength` / `getCellParagraphLength` (기존) | 문단/셀 끝 점검 |
| `cursor.moveHorizontal` (기존) | 문단/셀 경계 영역 영역 이전/다음 이동 |
| `cursor.setAnchor` / `clearSelection` (PR #746 인프라) | 선택 영역 확장/해제 |
| `deleteSelection` (기존) | Option+Backspace/Delete 영역 영역 활용 |

→ 신규 인프라 — `moveToWordBoundary` + `findWordBoundary{Forward,Backward}` + `classifyChar` (cursor.ts 단일 영역 영역 격리).

## 6. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (2 commits) | ✅ auto-merge 충돌 0건 |
| `tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |
| WASM 빌드 (Docker, 재빌드) | ✅ 4.68 MB |
| 자기 정정 commit | ✅ Alt+Delete 표 안/외 분기 정정 + sweep 170/170 same 보존 |

## 7. 작업지시자 시각 검증 ✅ 통과 (자기 정정 후)
- 표 외 영역 영역 Alt+Delete: 다음 단어 삭제 (본 결함 정정)
- 표 안 영역 영역 Alt+Delete: 칸 지우기 (기존 동작 보존)
- Alt+Backspace / Alt+Arrow: 모든 위치 영역 영역 단어 삭제/이동

## 8. 영향 범위

### 8.1 변경 영역
- `rhwp-studio/src/engine/cursor.ts` 영역 영역 moveToWordBoundary + 유틸 (단어 경계 탐색)
- `rhwp-studio/src/engine/input-handler-keyboard.ts` 영역 영역 Alt+Arrow / Alt+Backspace/Delete 분기 + Alt 가드 정정

### 8.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 입증)

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 4번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_pr_supersede_chain` | PR #746 (Ctrl/Cmd+Arrow) → PR #794 (Alt/Option+Arrow) Issue #223 단계적 진전 |
| `feedback_process_must_follow` | 인프라 재사용 + 신규 인프라 격리 (cursor.ts 단일 영역) |
| `feedback_visual_judgment_authority` | **권위 사례 강화** — 작업지시자 시각 검증 영역 영역 Alt+Delete 결함 + 본질 충돌 (칸 지우기 vs 단어 삭제) 발견 → 자기 정정 (PR #740 패턴) |
| `feedback_diagnosis_layer_attribution` | shortcut-map 영역 dispatcher silently fail 영역 영역 본질 진단 (canExecute 차단 + return) |

## 10. 잔존 후속

- Issue #223 OPEN 유지 — 한컴 주요 단축키 등 후속 단계 잔존
- 본 PR 본질 정정의 잔존 결함 부재

---

작성: 2026-05-11
