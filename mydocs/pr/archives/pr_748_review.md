---
PR: #748
제목: Task #158 — 표 크기 조절 Undo/Redo (SnapshotCommand 적용)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 16번째 PR)
base / head: devel / contrib/table-resize-undo
mergeStateStatus: DIRTY
mergeable: CONFLICTING — PR #728 영역 영역 머지 영역 영역 동일 파일 누적 변경
CI: 결과 부재 영역 영역
변경 규모: +31 / -15, 1 file
검토일: 2026-05-10
---

# PR #748 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #748 |
| 제목 | Task #158 — 표 크기 조절 Undo/Redo (SnapshotCommand 적용) |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 16번째 PR) |
| base / head | devel / contrib/table-resize-undo |
| mergeStateStatus | **DIRTY**, mergeable: CONFLICTING — PR #728 영역 영역 동일 파일 누적 변경 |
| CI | 결과 부재 |
| 변경 규모 | +31 / -15, 1 file |
| 커밋 수 | 2 (Task + Copilot 리뷰) |
| closes | #158 |

## 2. 결함 본질 (Issue #158)

표 크기 조절 (마우스 드래그, 키보드, Ctrl+방향키 비율 리사이즈) 후 Ctrl+Z 영역 영역 되돌리기 부재. `resizeTableCells` WASM API 영역 영역 Undo 이력 없이 직접 호출.

### 2.1 채택 접근 — PR #728 패턴 정합

PR #728 (closes #204) 영역 영역 표 편집 11 커맨드 영역 영역 SnapshotCommand 적용 영역 영역 인프라 활용. 세 가지 크기 조절 경로 모두 `executeOperation({ kind: 'snapshot' })` 경유:

| 경로 | operationType | 함수 |
|------|---------------|------|
| 마우스 드래그 리사이즈 | `resizeTable` | `finishResizeDrag` |
| 키보드 셀 리사이즈 | `resizeCell` | `resizeCellByKeyboard` |
| Ctrl+방향키 비율 리사이즈 | `resizeTableProportional` | `resizeTableProportional` |

## 3. PR 의 정정 — `rhwp-studio/src/engine/input-handler-table.ts` (+31/-15)

세 함수 동일 패턴:

```typescript
const pos = this.cursor.getPosition();
try {
    this.executeOperation({
        kind: 'snapshot', operationType: 'resizeTable',
        operation: (wasm: any) => {
            wasm.resizeTableCells(sec, ppi, ci, updates);
            return pos;
        },
    });
} catch (err) {
    console.warn('[InputHandler] finishResizeDrag 실패:', err);
}
```

기존 `wasm.resizeTableCells` 직접 호출 + `eventBus.emit('document-changed')` 영역 영역 영역 → `executeOperation` 경유 (afterEdit() 영역 영역 자동 발행).

### 3.1 Copilot 리뷰 반영 (commit `a15336d2`)
- 중복 `document-changed` 이벤트 발행 제거
- `try/catch` 추가 (안전성)

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| PR #728 (closes #204) `executeOperation({ kind: 'snapshot' })` | 동일 패턴 정합 |
| `WasmBridge.saveSnapshot/restoreSnapshot` | 자동 활용 |
| `SnapshotCommand` (`input-handler.ts:1485`) | 자동 활용 |

→ `feedback_process_must_follow` 정합 — 신규 인프라 도입 부재.

## 5. 충돌 분석

### 5.1 본질
PR #748 base = `30351cdf` (5/9 시점) — devel HEAD 영역 영역 PR #728 (closes #204) 머지 영역 영역 동일 파일 (`input-handler-table.ts`) 영역 영역 누적 변경 → 충돌 발생.

### 5.2 충돌 영역 영역 의도
- PR #728 영역 영역 11 표 커맨드 영역 영역 SnapshotCommand 적용
- PR #748 영역 영역 표 크기 조절 (3 함수) 영역 영역 SnapshotCommand 적용
- **동일 파일 다른 함수** — 의도 영역 영역 호환 (양쪽 모두 동일 패턴 적용)

### 5.3 cherry-pick 전략

옵션 A (개별 cherry-pick) 시도 → 충돌 발생 시 옵션 B (PR HEAD squash cherry-pick + 수동 충돌 해결).

## 6. 본 환경 점검

- merge-base: `30351cdf` (5/9 시점)
- merge-tree 충돌: **CONFLICTING** (`input-handler-table.ts`)
- 변경 격리: TypeScript 단일 파일 — Rust/렌더링 경로 무관

## 7. 처리 옵션

### 옵션 A — 2 commits 개별 cherry-pick (충돌 시 옵션 B)

```bash
git checkout -b local/task158 7a1074c2
git cherry-pick 70cc0209 a15336d2
# 충돌 발생 시 squash cherry-pick + 수동 통합
```

### 옵션 B — PR HEAD squash cherry-pick + 수동 통합 (PR #729/#730/#732 동일 패턴)

```bash
git cherry-pick --no-commit 30351cdf..pr748-head
# 충돌 영역 영역 수동 해결 (PR #728 + PR #748 영역 영역 동일 패턴 영역 영역 통합)
git commit
```

→ **옵션 A 시도 후 충돌 발생 시 옵션 B 권장**.

## 8. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] 충돌 수동 해결 — PR #728 영역 영역 11 커맨드 + 본 PR 영역 영역 3 크기 조절 함수 영역 영역 동시 보존
- [ ] `cargo build --release` 통과 (Rust 변경 부재)
- [ ] `cargo test --release` ALL GREEN
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 본질은 **rhwp-studio editor 표 크기 조절 인터랙션**:
- WASM 빌드 후 dev server 영역 영역 3가지 크기 조절 경로 영역 영역 Ctrl+Z Undo 정합 점검
  - 마우스 드래그 리사이즈 → Ctrl+Z
  - 키보드 셀 리사이즈 → Ctrl+Z
  - Ctrl+방향키 비율 리사이즈 → Ctrl+Z
- E2E 자동 테스트 신규 부재 → 작업지시자 직접 인터랙션 검증 권장

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 16번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 파일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (PR #728 SnapshotCommand 패턴) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 |
| `feedback_pr_supersede_chain` | (PR #728 후속 정정 — 본 PR 영역 영역 별 함수 영역 영역 동일 패턴 적용) |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 영역 작업지시자 인터랙션 검증 권장 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 옵션 A cherry-pick 시도 (충돌 시 옵션 B 영역 영역 squash)
2. 자기 검증 (cargo test + tsc + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (3가지 크기 조절 경로 + Ctrl+Z)
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #748 close (closes #158 자동 정합)

---

작성: 2026-05-10
