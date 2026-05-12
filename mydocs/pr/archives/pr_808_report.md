---
PR: #808
제목: fix — moveToDocumentEnd 다중 구역 문서 지원 (closes #784)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 8번째 PR
처리: 옵션 A — 2 commits cherry-pick + 자기 정정 commit (Ctrl+↑/↓ 한컴 표준 정합) + no-ff merge
처리일: 2026-05-11
머지 commit: ca729bdc
별 Issue: #837 (빈 문단 캐럿 미표시 — 본 PR 무관 기존 결함)
---

# PR #808 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + 자기 정정 commit + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `ca729bdc` (--no-ff merge) |
| Cherry-pick commits | 2 PR commits + 1 자기 정정 commit (총 3) |
| closes | #784 |
| 별 Issue | #837 (빈 문단 캐럿 미표시 — 본 PR 무관 기존 결함) |
| 시각 판정 | ✅ 작업지시자 인터랙션 검증 통과 (Ctrl+End 다중 구역 + Ctrl+↓ 한 문단씩 이동) |
| 자기 검증 | tsc + cargo test ALL GREEN |
| WASM 재빌드 | 불필요 (TypeScript 단일 변경) |

## 2. 본질 (Issue #784)

`moveToDocumentEnd` 의 `const sec = 0` 하드코드 — 다중 구역 문서 (예: aift.hwp 3 구역) 에서 구역 0 의 paragraphCount 만 조회 → 잘못된 (sec=0, para=N) 위치 → cursor rect 미발견 콘솔 오류.

### 결함 흐름
1. aift.hwp (3 구역) 열기
2. `Ctrl+End` → `moveToDocumentEnd`
3. `sec = 0` 하드코드 → 구역 0 의 paragraphCount=N 조회
4. `lastPara = N - 1` (구역 0 의 마지막 문단)
5. `updateRect` → cursor rect 미발견 오류

### Issue #784 명시된 처리 방향
- `getSectionCount()` 로 마지막 구역 인덱스 도출
- 마지막 구역의 lastPara / paraLen 조회

## 3. 정정 본질 — 2 PR commits

### 3.1 `WasmBridge.getSectionCount()` 래퍼 추가 (commit `e90eba10`)

```typescript
getSectionCount(): number {
  return this.doc?.getSectionCount() ?? 0;
}
```

WASM 측 `hwpdocument_getSectionCount` 이미 노출 (`src/wasm_api.rs:1236`, `rhwp.d.ts:623`) — 본 PR 은 TypeScript 래퍼만 추가.

### 3.2 `moveToDocumentEnd` 정정 (commit `b20c00cc`)

```typescript
const secCount = this.wasm.getSectionCount();
const lastSec = secCount > 0 ? secCount - 1 : 0;
const paraCount = this.wasm.getParagraphCount(lastSec);
if (paraCount > 0) {
  const lastPara = paraCount - 1;
  const paraLen = this.wasm.getParagraphLength(lastSec, lastPara);
  this.position = { sectionIndex: lastSec, paragraphIndex: lastPara, charOffset: paraLen };
} else {
  this.position = { sectionIndex: lastSec, paragraphIndex: 0, charOffset: 0 };
}
```

- `secCount > 0 ? secCount - 1 : 0` 빈 문서 폴백 가드
- 단일 구역 영역 (`secCount = 1` → `lastSec = 0`) 기존 동작 정확히 보존

## 4. 함께 정리 — 자기 정정 commit (`f2ab6e0a`) — Ctrl+↑/↓ 한컴 표준 정합

### 4.1 작업지시자 시각 검증 중 발견
본 PR 검증 중 — `Ctrl+↓` 가 한 문단씩 이동이 아니라 문서 끝으로 이동. PR #746 (Task #260) 의 매핑이 **macOS 표준** (`Cmd+↑/↓ = 문서 시작/끝`) 만 정합, **한컴 표준** (`Ctrl+↑/↓ = 이전/다음 문단`) 어긋남.

### 4.2 `cursor.ts` 영역 — `moveToParagraphBoundary(direction)` 신규
```typescript
moveToParagraphBoundary(direction: -1 | 1): void {
  this.preferredX = null;
  this.atLineEnd = false;
  // 표 셀 경로: cellParaIndex 이동
  // 본문 경로: 현재 문단 charOffset > 0 = 문단 시작 / 0 = 인접 문단 시작 / 구역 경계 = 인접 구역 정합
}
```

- 본문 경로 + 표 셀 경로 (셀 내부 cellParaIndex 이동)
- 구역 경계 정합 (마지막 문단 → 다음 구역 첫 문단 / 첫 문단 → 이전 구역 마지막 문단)
- `atLineEnd = false` 초기화 (PR #807 정합 보존)

### 4.3 `input-handler-keyboard.ts` ArrowUp/Down 분기
```typescript
if (e.metaKey && !e.ctrlKey) {
  this.cursor.moveToDocumentStart();  // macOS Cmd+↑ = 문서 시작
} else {
  this.cursor.moveToParagraphBoundary(-1);  // Windows/Linux Ctrl+↑ = 이전 문단
}
```

- **macOS** `Cmd+↑/↓` (`e.metaKey && !e.ctrlKey`) = 문서 시작/끝 (macOS 표준 보존)
- **Windows/Linux** `Ctrl+↑/↓` = 이전/다음 문단 (한컴 표준)

## 5. 별 Issue #837 — 빈 문단 캐럿 미표시 (본 PR 무관)

자기 정정 commit 시각 검증 중 추가 발견:
- 빈 문단에 마우스 클릭 → 캐럿 안 보임
- 이전 문단에서 화살표로 빈 문단 진입 → 캐럿 안 보임
- 글자 있는 문단은 캐럿 정상

→ 본 PR 무관 **기존 결함**. WASM `getCursorRect(sec, para, 0)` 또는 caret-renderer 측 빈 문단 처리 결함 추정. 별 Issue #837 분리 + 후속 정정.

## 6. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `wasm.getParagraphCount` (기존) | 구역별 문단 수 |
| `wasm.getParagraphLength` (기존) | 마지막 문단 길이 |
| `wasm.getSectionCount` (WASM 측 기존, TypeScript 측 신규 래퍼) | 마지막 구역 인덱스 |
| `wasm.getCellParagraphCount` (기존) | 표 셀 문단 수 (자기 정정 commit 영역) |

→ 신규 WASM API 도입 부재. TypeScript 측 래퍼 1개 + cursor 측 신규 메서드 1개.

## 7. 영역 좁힘 (회귀 부재 가드)

- 단일 구역 문서 영역 `secCount = 1` → `lastSec = 0` 기존 동작 정확히 보존
- 빈 문서 영역 `secCount = 0` → `lastSec = 0` 폴백
- macOS 환경 영역 `Cmd+↑/↓` 매핑 기존 동작 보존 (`e.metaKey && !e.ctrlKey`)
- PR #807 `atLineEnd` 정합 보존 (자기 정정 commit 영역 `atLineEnd = false` 초기화)

## 8. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 2 commits | ✅ auto-merge 충돌 0건 (PR #807 atLineEnd 라인 + PR #808 본문 둘 다 보존) |
| `tsc --noEmit` | ✅ 통과 (본 PR + 자기 정정 commit 후 재확인) |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep | 면제 (TypeScript 단일 변경 — SVG 무영향 자명) |
| WASM 재빌드 | 불필요 |

## 9. 작업지시자 인터랙션 검증 ✅ 통과

### 9.1 본 PR 본질
- aift.hwp (3 구역) → `Ctrl+End` → 콘솔 오류 부재 + 마지막 구역 마지막 문단 끝 (Issue #784 정정)
- 단일 구역 문서 영역 `Ctrl+End` — 기존 동작 보존
- PR #807 `End → Ctrl+End → Home` 영역 회귀 부재

### 9.2 자기 정정 commit (Ctrl+↑/↓)
- 다중 문단 본문 영역 `Ctrl+↓` → 다음 문단 시작 이동 (이전 문서 끝 결함 정정)
- 다중 문단 본문 영역 `Ctrl+↑` → 현재 문단 시작 (charOffset > 0) 또는 이전 문단 시작
- 한 문단씩 이동 정합

### 9.3 빈 문단 캐럿 미표시
별 Issue #837 분리 — 본 PR 무관 기존 결함 (마우스 클릭 + 화살표 진입도 동일 결함).

## 10. CI 통과

✅ Build & Test + CodeQL (js-ts / python / rust) + Canvas visual diff (PR 머지 전 검증)

## 11. 영향 범위

### 11.1 변경 영역
- `rhwp-studio/src/core/wasm-bridge.ts` 영역 `getSectionCount()` 래퍼 (+4)
- `rhwp-studio/src/engine/cursor.ts` 영역 `moveToDocumentEnd` 정정 (+6/-6) + `moveToParagraphBoundary` 신규 (+74)
- `rhwp-studio/src/engine/input-handler-keyboard.ts` 영역 ArrowUp/Down 분기 (+14/-2)

### 11.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (TypeScript 단일 영역 SVG 무영향)
- macOS `Cmd+↑/↓` 기존 매핑 (분기 영역 보존)

## 12. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 8번째 PR — #786 → #787 → #788 → #794 → #795 → #796 → #807 → **#808**) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (WASM `getSectionCount` 이미 노출, TypeScript 래퍼만 추가) — 신규 WASM API 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | `secCount > 0 ? secCount - 1 : 0` 빈 문서 폴백 + `e.metaKey && !e.ctrlKey` macOS/Windows-Linux 분기 |
| `feedback_diagnosis_layer_attribution` | `const sec = 0` 단일 구역 가정 본질 진단 (Issue #784 명시) + Ctrl+↑/↓ 매핑 본질 (macOS 표준만 정합 / 한컴 표준 어긋남) |
| `feedback_visual_judgment_authority` | 작업지시자 시각 검증 영역 본 PR 본질 결함 + 자기 정정 commit 발견 + 빈 문단 캐럿 결함 발견 — **3 단계 발견 patterns** |
| `feedback_pr_supersede_chain` | PR #746 (Ctrl+End 추가) 영역 발견된 오랜 잠재 결함 (Issue #784) 영역 별 PR 정정 + PR #746 매핑 본질 결함 영역 자기 정정 (한컴 표준 정합) — (c) 패턴 + 매핑 본질 정정 |

## 13. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재 (Issue #784 close 완료)
- 별 Issue #837 — 빈 문단 캐럿 미표시 (마우스 클릭 + 화살표 진입 + Ctrl+↓ 모두 동일 본질, WASM `getCursorRect` 또는 caret-renderer 측 점검 필요)

---

작성: 2026-05-11
