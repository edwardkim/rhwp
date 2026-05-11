---
PR: #794
제목: feat — Alt/Option+Arrow 단어 단위 커서 이동 (Part of #223)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/11 사이클 4번째 PR)
base / head: devel / contrib/word-movement
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: SUCCESS (Build & Test + CodeQL js/ts/py/rust + Canvas visual diff)
변경 규모: +144 / -2, 2 files
검토일: 2026-05-11
---

# PR #794 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #794 |
| 제목 | feat — Alt/Option+Arrow 단어 단위 커서 이동 |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/11 사이클 4번째 PR — PR #788 후속) |
| base / head | devel / contrib/word-movement |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 부재 |
| CI | ✅ Build & Test + CodeQL (js/ts/py/rust) + Canvas visual diff 통과 |
| 변경 규모 | +144 / -2, 2 files |
| 커밋 수 | 2 (feat + Copilot 리뷰) |
| Part of | Issue #223 (macOS 단축키 지원) |

## 2. 본질

macOS 표준 단어 이동 단축키 구현 — PR #746 (Ctrl/Cmd+Arrow 줄/문서 시작·끝) 후속 영역 영역 Issue #223 의 다음 단계.

| 단축키 | 동작 |
|--------|------|
| **Option+←/→** | 단어 단위 커서 이동 |
| **Option+Shift+←/→** | 단어 단위 선택 확장 |
| **Option+Backspace** | 이전 단어 삭제 |
| **Option+Delete** | 다음 단어 삭제 |

## 3. 채택 접근

### 3.1 신규 인프라 — `CursorState.moveToWordBoundary`
`rhwp-studio/src/engine/cursor.ts` 영역 영역 단어 경계 탐색 + 커서 이동:
- 본문 (paragraph): `getTextRange` + `findWordBoundary{Forward,Backward}`
- 표 셀 내부: `getTextInCell` + 동일 헬퍼

### 3.2 문자 클래스 기반 경계 탐지 (5종)

```typescript
const enum CharClass { Space, Hangul, Latin, Digit, Punct }

function classifyChar(ch: string): CharClass {
  // 공백: 0x20, 0x09, 0x0A, 0x0D, 0xA0
  // 한글: 완성형 (0xAC00~0xD7AF) + 자모 (0x3131~0x318E) + 첫가끝 (0x1100~0x11FF)
  // Latin: A-Z, a-z
  // Digit: 0-9
  // Punct: 그 외
}
```

같은 클래스 문자열 영역 영역 하나의 "단어" 영역 영역 취급.

### 3.3 경계 규칙
- **전진**: 현재 단어 끝 + 후행 공백 건너뜀
- **후진**: 선행 공백 건너뜀 + 이전 단어 시작까지

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `getTextRange` / `getTextInCell` WASM API (기존) | 단어 경계 탐색 영역 영역 텍스트 슬라이스 |
| `getParagraphLength` / `getCellParagraphLength` (기존) | 문단/셀 끝 점검 |
| `cursor.moveHorizontal` (기존) | 문단/셀 경계 영역 영역 이전/다음 이동 |
| `cursor.setAnchor` / `clearSelection` (기존, PR #746 인프라) | 선택 영역 확장/해제 |
| `deleteSelection` (기존) | Option+Backspace/Delete 영역 영역 활용 |

→ 신규 인프라 — `moveToWordBoundary` + `findWordBoundary{Forward,Backward}` + `classifyChar` (단일 모듈 영역 영역 격리).

## 5. PR 의 정정 — 2 files, +144/-2

### 5.1 `cursor.ts` (+128)
- `moveToWordBoundary(direction)` — 본문 경로
- `moveToWordBoundaryInCell` — 표 셀 경로
- 유틸: `CharClass` enum + `classifyChar` + `findWordBoundaryForward` + `findWordBoundaryBackward`
- 슬라이스 크기 50 char 영역 영역 cap (성능 가드)

### 5.2 `input-handler-keyboard.ts` (+16/-2)
- Alt 조합 단축키 처리 영역 영역 `!e.key.startsWith('Arrow')` 가드 추가 (Alt+Arrow 영역 영역 chord 발동 차단)
- Arrow case 영역 영역 `e.altKey && (←/→)` 분기 — `cursor.moveToWordBoundary` 호출 + Shift 영역 영역 선택 확장
- Backspace/Delete case 영역 영역 `e.altKey` 분기 — `moveToWordBoundary` + `deleteSelection`

## 6. Copilot 리뷰 반영 (commit `262bf5c8`)
- 주석 수정 + 셀 텍스트 슬라이스 최적화

## 7. 영역 좁힘 (회귀 부재 가드)

- Alt 조합 단축키 (chord V/G 등) 영역 영역 `!e.key.startsWith('Arrow')` 가드 영역 영역 회귀 차단
- 슬라이스 크기 50 char 영역 영역 cap — 긴 문단 영역 영역 성능 영향 좁힘
- 셀 내부 (textbox 제외) 영역 영역 별 path — `moveToWordBoundaryInCell`

## 8. 충돌 / mergeable

mergeStateStatus = `BEHIND`, mergeable = `MERGEABLE`. cursor.ts + input-handler-keyboard.ts 영역 영역 5/10 사이클 영역 영역 PR #746 (Ctrl/Cmd+Arrow) 영역 영역 누적 — 본 PR 영역 영역 다른 영역 (moveToWordBoundary 신규 + Alt+Arrow 분기) 영역 영역 충돌 부재 예상.

## 9. 본 환경 점검

### 9.1 변경 격리
- TypeScript 단일 영역 (rhwp-studio editor)
- Rust / WASM / 렌더링 경로 무관

### 9.2 CI 결과
- 모두 ✅

### 9.3 의도적 제한
- `e.altKey` 영역 영역 macOS 영역 영역 Option key — Windows/Linux 영역 영역 Alt key 영역 영역 동일 동작
- 슬라이스 50 char cap — 단어 영역 영역 50 char 초과 시 부분 이동 (정합 — 사용자 의도 영역 영역 단어 단위 영역 영역 50 char 영역 영역 단어 영역 영역 영역)

## 10. 처리 옵션

### 옵션 A — 2 commits 개별 cherry-pick + no-ff merge

```bash
git checkout local/devel
git cherry-pick 3250b619 262bf5c8
git checkout devel
git merge local/devel --no-ff
```

→ **권장**.

## 11. 검증 게이트

### 11.1 자기 검증
- [ ] cherry-pick 충돌 0건
- [ ] tsc + cargo test ALL GREEN
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (TypeScript 영역 영역 SVG 무영향 자명)

### 11.2 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 본질 영역 영역 rhwp-studio editor 단어 단위 커서 이동:
- WASM 빌드 후 dev server 영역 영역:
  - **Option/Alt+→/←** — 단어 단위 이동 (한글/영문/숫자/공백/구두점 경계)
  - **Option/Alt+Shift+→/←** — 단어 단위 선택 확장
  - **Option/Alt+Backspace** — 이전 단어 삭제
  - **Option/Alt+Delete** — 다음 단어 삭제
  - 표 셀 내부 영역 영역 동일 동작
  - 기존 Alt+V (보기) / Alt+G (조판) chord 회귀 부재 (Alt+Arrow 영역 영역 chord 차단 가드)

## 12. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/11 사이클 4번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_pr_supersede_chain` | PR #746 (Ctrl/Cmd+Arrow) → PR #794 (Alt/Option+Arrow) Issue #223 단계적 진전 |
| `feedback_process_must_follow` | 인프라 재사용 (getTextRange + getTextInCell + cursor.moveHorizontal + setAnchor) + 신규 인프라 격리 (cursor.ts 단일 영역 영역 moveToWordBoundary + findWordBoundary 유틸) |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 영역 작업지시자 인터랙션 검증 권장 |

## 13. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick (`3250b619` + `262bf5c8`)
2. 자기 검증 (tsc + cargo test + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (4 단축키 + 한글/영문/숫자/공백/구두점 경계 + Shift 선택 + 표 셀 내부 + 기존 chord 회귀 부재)
4. 인터랙션 검증 통과 → no-ff merge + push + archives + 5/11 orders
5. PR #794 close (Issue #223 OPEN 유지 — 한컴 주요 단축키 등 후속 단계 잔존)

---

작성: 2026-05-11
