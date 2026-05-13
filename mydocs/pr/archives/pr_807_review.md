---
PR: #807
제목: fix — 줄 끝 이동 후 줄 시작 이동 시 커서 미동작 수정 (closes #785)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 7번째 PR
base / head: devel / contrib/cursor-line-boundary-fix
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff
변경 규모: +28 / -1, 1 file
커밋: 2
검토일: 2026-05-11
---

# PR #807 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #807 |
| 제목 | fix: 줄 끝 이동 후 줄 시작 이동 시 커서 미동작 수정 (#785) |
| 컨트리뷰터 | @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/11 사이클 7번째 PR — #786 → #787 → #788 → #794 → #795 → #796 → **#807**) |
| base / head | devel / contrib/cursor-line-boundary-fix |
| mergeable | MERGEABLE (BEHIND — base 갱신 필요) |
| CI | ✅ 전 항목 통과 |
| 변경 규모 | +28 / -1, 1 file |
| 커밋 수 | 2 (1 본질 + 1 리뷰 반영) |
| closes | #785 |

## 2. 본질 (Issue #785)

soft-wrap 줄 경계에서 **`charEnd(line N) == charStart(line N+1)` 동일 위치** 영역 영역 End → Home 연속 호출 시 커서 미이동.

### 결함 흐름
1. 줄 N 영역 영역 `End` → `charEnd` (= 줄 N+1 의 `charStart`) 이동
2. 영역 영역 `Home` → `getLineInfo(charOffset=charEnd)` 호출
3. `compute_line_info_struct` 영역 영역 `char_offset >= line_char_starts[i]` 영역 영역 줄 N+1 판정
4. `moveToLineStart` → 줄 N+1 의 `charStart` (= 현재 위치) → 미이동 시각

### Issue #785 영역 영역 명시된 3 옵션
- **A — `at_line_end: bool` 플래그** (cursor affinity 패턴) ← **본 PR 채택**
- B — `compute_line_info_struct` 영역 `>=` → `>` (회귀 위험)
- C — `moveToLineEnd` 영역 `charEnd - 1` (강제 줄바꿈 영역 영역 별 처리 필요)

## 3. 정정 본질 — `cursor.ts` +28/-1 (cursor affinity 패턴)

### 3.1 `atLineEnd` 플래그 도입

```typescript
/** 줄 끝 이동 후 경계 위치 판별용 — soft-wrap 줄 경계에서 charEnd == 다음 줄 charStart 동일 문제 해결 */
private atLineEnd = false;
```

### 3.2 `moveToLineEnd()` — `atLineEnd = true` 설정

```typescript
moveToLineEnd(): void {
  // ...
  this.position = { ...this.position, charOffset: lineInfo.charEnd };
  this.atLineEnd = true;
  // ...
}
```

### 3.3 `moveToLineStart()` — 경계 감지 시 이전 줄 정보 조회

```typescript
moveToLineStart(): void {
  this.preferredX = null;
  try {
    const pos = this.position;
    let lineInfo = this.getLineInfoAtCursor();
    if (this.atLineEnd && pos.charOffset === lineInfo.charStart && pos.charOffset > 0) {
      const prevLineInfo = this.isInCell()
        ? this.wasm.getLineInfoInCell(
            pos.sectionIndex, pos.parentParaIndex!, pos.controlIndex!,
            pos.cellIndex!, pos.cellParaIndex!, pos.charOffset - 1)
        : this.wasm.getLineInfo(pos.sectionIndex, pos.paragraphIndex, pos.charOffset - 1);
      if (prevLineInfo.charEnd === pos.charOffset) {
        lineInfo = prevLineInfo;
      }
    }
    this.atLineEnd = false;
    this.position = { ...this.position, charOffset: lineInfo.charStart };
    this.updateRect();
  } catch (e) { ... }
}
```

- `atLineEnd && pos.charOffset === lineInfo.charStart && pos.charOffset > 0` 가드 3중 점검 — 영역 좁힘
- 표 셀 내부 영역 영역 `getLineInfoInCell` 분기 정합
- 이전 줄 영역 영역 `prevLineInfo.charEnd === pos.charOffset` 재확인 — 안전망

### 3.4 기타 이동 메서드 — `atLineEnd = false` 초기화

| 메서드 | 본질 |
|--------|------|
| `moveTo` | 직접 위치 설정 |
| `resetPreferredX` | 수평 이동 시 호출 |
| `moveHorizontal` | 좌/우 이동 |
| `moveVertical` | **위/아래 이동 — Copilot 리뷰 반영 commit `011bab06` 영역 영역 추가** |
| `moveToDocumentStart` | Ctrl+Home |
| `moveToDocumentEnd` | Ctrl+End |
| `moveToCellNext` | Tab (셀 다음) |
| `moveToCellPrev` | Shift+Tab (셀 이전) |

## 4. 리뷰 반영 commit (`011bab06`)

Copilot 리뷰 영역 영역 발견 결함 — `moveVertical` 영역 영역 `atLineEnd = false` 초기화 누락:
- End → ArrowDown 후 Home 호출 시 `atLineEnd` 가 true 유지 → 이전 줄로 잘못 판정
- → 8개 이동 메서드 영역 영역 일관성 정합

## 5. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `getLineInfoAtCursor` (기존) | 줄 정보 조회 (본문/셀 분기) |
| `wasm.getLineInfo` / `getLineInfoInCell` (기존) | charOffset 영역 영역 줄 정보 |
| cursor 영역 affinity 패턴 (편집기 표준) | `atLineEnd` 플래그 |

→ 신규 인프라 도입 부재 — `atLineEnd: boolean` 영역 영역 cursor 영역 격리. WASM 변경 부재.

## 6. 영역 좁힘 (회귀 부재 가드)

- `atLineEnd` 영역 영역 `moveToLineEnd` 시점만 true
- `moveToLineStart` 영역 영역 가드 3중 (atLineEnd + charOffset === charStart + charOffset > 0)
- 이전 줄 정보 영역 영역 `prevLineInfo.charEnd === pos.charOffset` 재확인 — 잘못된 위치 영역 영역 폴백 부재
- 다른 8개 이동 메서드 영역 영역 `atLineEnd = false` 초기화 — End → 다른 이동 → Home 정합

## 7. 본 환경 점검

### 7.1 변경 격리
- **순수 TypeScript `cursor.ts` 단일** — WASM/Rust 변경 부재
- HWP3/HWPX 변환본 시각 정합 (sweep 무영향 자명)

### 7.2 CI 통과
- ✅ Build & Test
- ✅ CodeQL (js-ts / python / rust)
- ✅ Canvas visual diff

### 7.3 mergeStateStatus = BEHIND
base 갱신만 필요 — 충돌 부재 (MERGEABLE). cherry-pick auto-merge 예상.

## 8. 처리 옵션

### 옵션 A (권장) — 2 commits cherry-pick + no-ff merge

```bash
git checkout local/devel
git cherry-pick fe78fec1 011bab06
git checkout devel
git merge local/devel --no-ff
```

본질 commit + 리뷰 반영 commit 영역 영역 이력 보존.

### 옵션 B — squash cherry-pick

2 commits 영역 영역 단일 commit 영역 영역. 본 환경 영역 영역 commit 이력 보존 권장 영역 영역 옵션 A 권장.

## 9. 검증 게이트

### 9.1 자기 검증
- [ ] cherry-pick 2 commits (auto-merge 예상)
- [ ] tsc --noEmit
- [ ] cargo test (Rust 변경 부재 영역 영역 회귀 자명, 형식 점검)
- [ ] WASM 빌드 (cursor.ts 변경 영역 영역 WASM 재빌드 불필요, 그러나 머지 commit 영역 영역 dev server 동작 점검)

### 9.2 시각/인터랙션 판정 게이트 — **작업지시자 인터랙션 검증 권장**
- End → Home (본문) — 커서 줄 시작 이동
- End → Home (표 셀 내부) — 동일
- End → ArrowDown → Home — `moveVertical` atLineEnd 초기화 검증
- End → 클릭 → Home — `moveTo` atLineEnd 초기화 검증
- 강제 줄바꿈 (`\n`) 영역 영역 End → Home — 기존 동작 보존

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 7번째 PR — PR #786~#796 → #807) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (getLineInfo + cursor affinity 패턴) — 신규 인프라 격리 (`atLineEnd: boolean`) |
| `feedback_hancom_compat_specific_over_general` | `atLineEnd && charOffset === charStart && charOffset > 0` 3중 가드 영역 영역 영역 좁힘 |
| `feedback_diagnosis_layer_attribution` | `compute_line_info_struct` 영역 `>=` 본질 진단 정확 (Issue #785 영역 영역 명시) |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 영역 작업지시자 검증 권장 |

## 11. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 cherry-pick 2 commits (`fe78fec1` + `011bab06`)
2. 자기 검증 (tsc + cargo test)
3. 작업지시자 웹 에디터 인터랙션 검증 (End → Home / End → ArrowDown → Home / 표 셀 내부)
4. 검증 통과 → no-ff merge + push + archives + 5/11 orders + Issue #785 close
5. PR #807 close

---

작성: 2026-05-11
