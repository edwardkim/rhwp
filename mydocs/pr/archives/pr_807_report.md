---
PR: #807
제목: fix — 줄 끝 이동 후 줄 시작 이동 시 커서 미동작 수정 (closes #785)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 7번째 PR
처리: 옵션 A — 2 commits cherry-pick + Task #516 진단 로그 제거 commit + no-ff merge
처리일: 2026-05-11
머지 commit: 3960b3b6
---

# PR #807 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + Task #516 진단 로그 제거 commit + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `3960b3b6` (--no-ff merge) |
| Cherry-pick commits | 2 PR commits + 1 진단 로그 제거 commit (총 3) |
| closes | #785 |
| 시각 판정 | ✅ 작업지시자 웹 에디터 인터랙션 검증 통과 (2회 — 본 PR + 진단 로그 제거 후 재검증) |
| 자기 검증 | tsc + cargo test ALL GREEN |
| WASM 재빌드 | 불필요 (cursor.ts 단일 변경 영역 영역 PR #799 빌드 pkg 사용) |

## 2. 본질 (Issue #785)

soft-wrap 줄 경계 영역 영역 `charEnd(line N) == charStart(line N+1)` 동일 위치 영역 영역 `End → Home` 시 `getLineInfo` 영역 영역 다음 줄 판정 → `moveToLineStart` 영역 영역 현재 위치 동일 → 커서 미이동.

### 결함 흐름
1. 줄 N 영역 영역 `End` → `charEnd` (= 줄 N+1 의 `charStart`) 이동
2. 영역 영역 `Home` → `getLineInfo(charOffset=charEnd)` 호출
3. `compute_line_info_struct` 영역 영역 `char_offset >= line_char_starts[i]` 영역 영역 줄 N+1 판정
4. `moveToLineStart` → 줄 N+1 의 `charStart` (= 현재 위치) → 미이동 시각

### Issue #785 명시된 3 옵션
- **A — `at_line_end: bool` 플래그** (cursor affinity 패턴) ← **본 PR 채택**
- B — `compute_line_info_struct` 영역 `>=` → `>` (회귀 위험)
- C — `moveToLineEnd` 영역 `charEnd - 1` (강제 줄바꿈 영역 영역 별 처리 필요)

## 3. 정정 본질 — `cursor.ts` +28/-1 (cursor affinity 패턴)

### 3.1 `atLineEnd: boolean` 플래그
```typescript
/** 줄 끝 이동 후 경계 위치 판별용 — soft-wrap 줄 경계에서 charEnd == 다음 줄 charStart 동일 문제 해결 */
private atLineEnd = false;
```

### 3.2 `moveToLineEnd()` — `atLineEnd = true` 설정

### 3.3 `moveToLineStart()` — 경계 감지 시 이전 줄 정보 조회

```typescript
if (this.atLineEnd && pos.charOffset === lineInfo.charStart && pos.charOffset > 0) {
  const prevLineInfo = this.isInCell()
    ? this.wasm.getLineInfoInCell(...charOffset - 1)
    : this.wasm.getLineInfo(...charOffset - 1);
  if (prevLineInfo.charEnd === pos.charOffset) {
    lineInfo = prevLineInfo;
  }
}
```

3중 가드 (`atLineEnd` + `charOffset === charStart` + `charOffset > 0`) + 이전 줄 `charEnd` 재확인 안전망.

### 3.4 8개 이동 메서드 — `atLineEnd = false` 초기화

| 메서드 | 본질 |
|--------|------|
| `moveTo` | 직접 위치 설정 |
| `resetPreferredX` | 수평 이동 시 호출 |
| `moveHorizontal` | 좌/우 이동 |
| `moveVertical` | **위/아래 이동 — Copilot 리뷰 반영 commit `011bab06` 영역 영역 추가** |
| `moveToDocumentStart` | Ctrl+Home |
| `moveToDocumentEnd` | Ctrl+End |
| `moveToCellNext` | Tab |
| `moveToCellPrev` | Shift+Tab |

## 4. 리뷰 반영 commit (`011bab06`)

Copilot 리뷰 영역 영역 발견 — `moveVertical` 영역 영역 `atLineEnd = false` 초기화 누락:
- End → ArrowDown → Home 호출 시 `atLineEnd` true 유지 → 이전 줄 잘못 판정
- → 8개 이동 메서드 영역 영역 일관성 정합

## 5. 함께 처리 — Task #516 진단 로그 3개소 제거 (`6aa4afd2`)

작업지시자 영역 영역 본 PR 시각 검증 영역 영역 발견 — 웹 콘솔 영역 영역 Task #516 진단 로그 출력:
```
[Task#516] applyOverlays page=N behind=M front=K
[Task#516] JSON image ops=N, wrap=behindText=M
[Task#516] collected behind=N front=M
```

### 본질
- Task #516 (옵션 C HTML Hybrid 다층 레이어, PR `cc2a376b` 영역 영역 도입) 영역 영역 주석 명시 — "시각 판정 통과 후 제거"
- 시각 판정 통과 영역 영역 잔존 console.log 영역 영역 본 PR 머지 cycle 영역 영역 함께 정리

### 정정
`rhwp-studio/src/view/page-renderer.ts` 영역 영역 3 console.log 제거:
- `applyOverlays` 진입 영역 (line 60)
- `getOverlayImages` JSON 점검 (line 170)
- `getOverlayImages` 수집 결과 (line 182)

기능 주석 (Task #516 Stage 5.2 본질 설명) + `console.warn('PageLayerTree JSON parse 실패')` 영역 영역 보존.

## 6. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `getLineInfoAtCursor` (기존) | 줄 정보 조회 (본문/셀 분기) |
| `wasm.getLineInfo` / `getLineInfoInCell` (기존) | charOffset 영역 영역 줄 정보 |
| cursor affinity 패턴 (편집기 표준) | `atLineEnd` 플래그 — cursor 영역 격리 |

→ 신규 인프라 도입 부재 — WASM 변경 부재.

## 7. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 2 commits | ✅ auto-merge 충돌 0건 |
| `tsc --noEmit` | ✅ 통과 (본 PR + 진단 로그 제거 후 재확인) |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep | 면제 (TypeScript 단일 영역 영역 SVG 무영향 자명) |
| WASM 재빌드 | 불필요 (cursor.ts 단일 변경) |

## 8. 작업지시자 시각/인터랙션 검증 ✅ 통과 (2회)

### 8.1 본 PR 검증
- 본문 다중 줄 문단 영역 영역 End → Home — 줄 시작 정상 이동 (이전 미동작 결함 정정)
- 표 셀 내부 다중 줄 영역 영역 End → Home — 정합
- End → ArrowDown → Home — `moveVertical` atLineEnd 초기화 검증 통과
- End → 클릭 → Home — `moveTo` atLineEnd 초기화 검증 통과
- 강제 줄바꿈 (`\n`) End → Home — 기존 동작 보존
- 기존 단축키 / 기존 기능 회귀 부재

### 8.2 진단 로그 제거 후 재검증
- 웹 콘솔 영역 영역 `[Task#516]` 로그 출력 부재
- BehindText / InFrontOfText 그림 정상 표시 (Task #516 기능 자체 영역 영역 영향 부재)

## 9. CI 통과

✅ Build & Test + CodeQL (js-ts / python / rust) + Canvas visual diff (PR 머지 전 검증)

## 10. 영향 범위

### 10.1 변경 영역
- `rhwp-studio/src/engine/cursor.ts` 영역 영역 cursor affinity 패턴 (+28/-1)
- `rhwp-studio/src/view/page-renderer.ts` 영역 영역 Task #516 진단 로그 제거 (-7)

### 10.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (TypeScript 단일 영역 영역 SVG 무영향 자명)
- Task #516 본질 (옵션 C HTML Hybrid 다층 레이어) — 진단 로그만 제거

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 7번째 PR — #786 → #787 → #788 → #794 → #795 → #796 → **#807**) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (getLineInfo + cursor affinity 패턴) + 신규 인프라 격리 (`atLineEnd: boolean`) |
| `feedback_hancom_compat_specific_over_general` | 3중 가드 (`atLineEnd && charOffset === charStart && charOffset > 0`) + `prevLineInfo.charEnd === pos.charOffset` 재확인 안전망 영역 영역 영역 좁힘 |
| `feedback_diagnosis_layer_attribution` | `compute_line_info_struct` `>=` 본질 진단 정확 (Issue #785 영역 영역 명시) |
| `feedback_visual_judgment_authority` | 작업지시자 인터랙션 검증 ✅ 통과 — **본 PR 시각 검증 영역 영역 Task #516 진단 로그 부수 발견** + 함께 정리 |

## 12. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #785 close 완료

---

작성: 2026-05-11
