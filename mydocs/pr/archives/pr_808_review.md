---
PR: #808
제목: fix — moveToDocumentEnd 다중 구역 문서 지원 (closes #784)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 8번째 PR
base / head: devel / contrib/multi-section-doc-end
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: ✅ Build & Test + CodeQL (js-ts/python/rust) + Canvas visual diff
변경 규모: +10 / -6, 2 files
커밋: 2
검토일: 2026-05-11
---

# PR #808 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #808 |
| 제목 | fix: moveToDocumentEnd 다중 구역 문서 지원 (#784) |
| 컨트리뷰터 | @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/11 사이클 **8번째 PR** — #786 → #787 → #788 → #794 → #795 → #796 → #807 → **#808**) |
| base / head | devel / contrib/multi-section-doc-end |
| mergeable | MERGEABLE (BEHIND — base 갱신 필요) |
| CI | ✅ 전 항목 통과 |
| 변경 규모 | +10 / -6, 2 files |
| 커밋 수 | 2 (1 본질 cursor.ts + 1 WasmBridge 래퍼) |
| closes | #784 |

## 2. 본질 (Issue #784)

`moveToDocumentEnd` 영역 영역 `const sec = 0` 하드코드 — 다중 구역 문서 (예: aift.hwp 3 구역) 영역 영역 구역 0 의 paragraphCount 만 조회 → 잘못된 (sec=0, para=N) 위치 영역 영역 cursor rect 미발견 → `[CursorState] updateRect 실패 → rect=null` 콘솔 오류.

### 결함 흐름
1. aift.hwp (3 구역) 열기
2. `Ctrl+End` → `moveToDocumentEnd`
3. `sec = 0` 하드코드 → 구역 0 의 paragraphCount=N 조회
4. `lastPara = N - 1` (구역 0 의 마지막 문단)
5. `paraLen = getParagraphLength(0, N-1)` 조회
6. `position = { sectionIndex: 0, paragraphIndex: N-1, charOffset: paraLen }`
7. `updateRect` → cursor rect 미발견 (사실은 구역 0 영역 영역 N-1 영역 영역 영역 영역 paragraphCount 영역 영역 다를 수 있음)

## 3. 정정 본질 — cursor.ts +6/-6 + wasm-bridge.ts +4 (2 files)

### 3.1 `WasmBridge.getSectionCount()` 래퍼 추가 (commit `e90eba10`)

```typescript
getSectionCount(): number {
  return this.doc?.getSectionCount() ?? 0;
}
```

WASM 측 `hwpdocument_getSectionCount` 이미 노출 (`src/wasm_api.rs:1236`, `rhwp.d.ts:623`) — 본 PR 영역 영역 TypeScript 래퍼만 추가.

### 3.2 `moveToDocumentEnd` 정정 (commit `b20c00cc`)

```typescript
moveToDocumentEnd(): void {
  this.preferredX = null;
  try {
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
    this.updateRect();
  } catch (e) { ... }
}
```

- `getSectionCount()` 영역 영역 마지막 구역 인덱스 도출
- `secCount > 0 ? secCount - 1 : 0` 가드 — 구역 0건 영역 영역 폴백
- 마지막 구역 영역 영역 lastPara / paraLen 조회 — Issue #784 영역 영역 명시된 처리 방향 정합

### 3.3 `moveToDocumentStart` 변경 부재

Issue #784 영역 영역 명시 — `moveToDocumentStart` 는 `sectionIndex: 0, paragraphIndex: 0, charOffset: 0` 영역 영역 정합 (첫 구역 첫 문단 영역 영역 항상 정합).

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `wasm.getParagraphCount` (기존) | 마지막 구역 영역 영역 paragraphCount |
| `wasm.getParagraphLength` (기존) | 마지막 문단 영역 영역 길이 |
| `wasm.getSectionCount` (WASM 측 기존, TypeScript 측 신규 래퍼) | 마지막 구역 인덱스 |

→ 신규 WASM API 도입 부재 — TypeScript 측 래퍼만 추가.

## 5. 영역 좁힘 (회귀 부재 가드)

- 단일 구역 문서 영역 영역 `secCount = 1` 영역 영역 `lastSec = 0` — 기존 동작 정확히 보존
- 다중 구역 문서 영역 영역 `lastSec = secCount - 1` — 본 결함 정정
- 구역 0건 영역 영역 `lastSec = 0` 폴백 — 빈 문서 정합

## 6. 본 환경 점검

### 6.1 변경 격리
- **순수 TypeScript** — cursor.ts + wasm-bridge.ts 만 변경
- WASM/Rust 변경 부재 (WASM 측 `getSectionCount` 이미 노출)
- HWP3/HWPX 변환본 시각 정합 (sweep 무영향 자명)

### 6.2 CI 통과
- ✅ Build & Test
- ✅ CodeQL (js-ts / python / rust)
- ✅ Canvas visual diff

### 6.3 mergeStateStatus = BEHIND + 충돌 가능성

PR base 영역 영역 PR #807 머지 이전 영역 영역. 현재 devel HEAD (PR #807 머지 후) 영역 영역:
- `moveToDocumentEnd` 영역 영역 `this.atLineEnd = false;` 라인 추가됨 (라인 500)

→ cherry-pick 시 3-way merge 충돌 가능 — incoming 영역 영역 PR #808 정정 + HEAD 영역 영역 PR #807 의 `atLineEnd` 라인 둘 다 보존 필요.

### 6.4 PR #807 정합성
PR #807 (Issue #785, 머지 commit `3960b3b6`) 영역 영역 `moveToDocumentEnd` 영역 `atLineEnd = false` 초기화 추가. PR #808 영역 영역 함수 본문 전반 변경 영역 영역 `atLineEnd` 라인 보존 필요.

## 7. 처리 옵션

### 옵션 A (권장) — 2 commits cherry-pick + 충돌 수동 해결 (PR #807 `atLineEnd` 라인 보존) + no-ff merge

```bash
git checkout local/devel
git cherry-pick e90eba10 b20c00cc
# b20c00cc 영역 영역 충돌 발생 시 — HEAD `atLineEnd = false` 라인 보존 + incoming `getSectionCount` 본문 정합
git checkout devel
git merge local/devel --no-ff
```

### 옵션 B — squash cherry-pick

2 commits 영역 영역 단일 commit 영역 영역. 본 환경 영역 영역 commit 이력 보존 권장 영역 영역 옵션 A 권장.

## 8. 검증 게이트

### 8.1 자기 검증
- [ ] cherry-pick 2 commits (1 충돌 수동 해결 예상)
- [ ] tsc --noEmit
- [ ] cargo test (Rust 변경 부재 영역 영역 회귀 자명, 형식 점검)
- [ ] WASM 재빌드 불필요 (TypeScript 단일)

### 8.2 시각/인터랙션 판정 게이트 — **작업지시자 인터랙션 검증 권장**
- aift.hwp (3 구역) 열기 → `Ctrl+End` → 콘솔 오류 부재 + 커서 정상 이동 (마지막 구역 마지막 문단 끝)
- 단일 구역 문서 영역 영역 `Ctrl+End` — 기존 동작 보존
- `Ctrl+Home` — 변경 부재 (기존 동작 보존 검증)
- PR #807 영역 영역 도입된 atLineEnd 정합 — `End → Ctrl+End → Home` 영역 영역 회귀 부재

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 8번째 PR — PR #807 후속) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (WASM 측 getSectionCount 이미 노출, TypeScript 래퍼만 추가) — 신규 WASM API 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | `secCount > 0 ? secCount - 1 : 0` 가드 영역 영역 빈 문서 폴백 |
| `feedback_diagnosis_layer_attribution` | `const sec = 0` 단일 구역 가정 본질 진단 정확 (Issue #784 영역 영역 명시) |
| `feedback_visual_judgment_authority` | 작업지시자 시각 검증 영역 영역 발견된 결함 (Issue #784 영역 영역 PR #746 검증 중 발견) |
| `feedback_pr_supersede_chain` | PR #746 (Ctrl+End 추가) 영역 영역 발견된 오랜 잠재 결함 영역 영역 별 PR 정정 — (c) 패턴 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 cherry-pick 2 commits (`e90eba10` WasmBridge + `b20c00cc` cursor.ts) + 충돌 수동 해결
2. 자기 검증 (tsc + cargo test)
3. 작업지시자 웹 에디터 인터랙션 검증 (aift.hwp Ctrl+End / 단일 구역 회귀 부재 / PR #807 atLineEnd 정합)
4. 검증 통과 → no-ff merge + push + archives + 5/11 orders + Issue #784 close
5. PR #808 close

---

작성: 2026-05-11
