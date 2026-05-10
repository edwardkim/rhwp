---
PR: #728
제목: Task #204 — 표 편집 Undo/Redo 스냅샷 기반 이력 등록
컨트리뷰터: @oksure — 20+ 사이클 핵심 컨트리뷰터 (rhwp-studio + WASM API + 디버깅 툴킷 영역)
base / head: devel / contrib/table-undo-redo
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS (Build & Test / Canvas visual diff / CodeQL / Analyze rust+js+py / WASM SKIPPED)
변경 규모: +123 / -101, 1 file (`rhwp-studio/src/command/commands/table.ts`)
검토일: 2026-05-10
---

# PR #728 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #728 |
| 제목 | Task #204 — 표 편집 Undo/Redo 스냅샷 기반 이력 등록 |
| 컨트리뷰터 | @oksure — **20+ 사이클** 핵심 컨트리뷰터 (PR #334/#335/#387/#388/#395/#396/#427/#428/#444/#446/#447/#448/#579/#581/#582/#583/#600/#601/#602/#659/#684/#728 등) |
| base / head | devel / contrib/table-undo-redo |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS (Canvas visual diff 포함) |
| 변경 규모 | +123 / -101, 1 file |
| 커밋 수 | 2 (본질 `25a9d1b2` + Copilot 리뷰 반영 `b8477e5f`) |
| closes | (PR 본문 직접 closes 미명시 — Issue #204 영역 영역) |

## 2. 결함 본질 (Issue #204)

### 2.1 결함 영역
rhwp-studio 영역 의 표 편집 (생성/삭제/행열 삽입·삭제/셀 분할·병합/캡션) 영역 영역 `Ctrl+Z` Undo 영역 미동작.

### 2.2 결함 원인 (Issue #204 진단)
rhwp-studio 영역 영역 두 편집 경로:
- **텍스트 편집** — `history.ts` 영역 의 `EditCommand` 스택 영역 영역 do/undo 정상 등록
- **표 편집** — `command/commands/table.ts` 영역 영역 WASM `doc.deleteTableRow/Column/...` 영역 영역 직접 호출 → `history.execute()` 영역 영역 `saveSnapshot()` 영역 미경유

### 2.3 채택 접근 — Issue #204 의 B안
**스냅샷 + SnapshotCommand 일괄 래핑** — 각 표 편집 영역 영역 `ih.executeOperation({ kind: 'snapshot', ... })` 영역 영역 래핑. 작업 전후 Document 스냅샷 영역 영역 자동 저장 (`saveSnapshot` / `restoreSnapshot`).

## 3. PR 의 정정 — `rhwp-studio/src/command/commands/table.ts` (+123/-101)

### 3.1 헬퍼 도입
```typescript
function safeTableOp(fn: () => void, label: string): void {
  try { fn(); } catch (e) { console.error(`[table] ${label} 실패:`, e); }
}
```

### 3.2 패턴 일관 적용 (11 커맨드)
```typescript
safeTableOp(() => ih.executeOperation({
  kind: 'snapshot',
  operationType: 'insertTableRow',
  operation: (wasm) => {
    wasm.insertTableRow(...);
    return pos;  // 새 cursor 위치 반환
  },
}), '줄 추가');
```

| 커맨드 | operationType |
|--------|---------------|
| `table:create` | `createTable` (대화상자 콜백) |
| `table:insert-row-above` / `-below` | `insertTableRow` (×2) |
| `table:insert-col-left` / `-right` | `insertTableColumn` (×2) |
| `table:delete-row` | `deleteTableRow` |
| `table:delete-col` | `deleteTableColumn` |
| `table:cell-split` | `splitTableCell` (단일/다중 셀 분기) |
| `table:cell-merge` | `mergeTableCells` |
| `table:delete` | `deleteTable` (선택 모드 + 셀 내부 분기) |
| `table:caption-toggle` | `toggleTableCaption` |

### 3.3 Copilot 리뷰 반영 (commit `b8477e5f`)
- `createTable` 영역 영역 `controlIndex: 0` (하드코드) → `result.controlIdx` 사용
- 에러 핸들링 영역 영역 `safeTableOp` 영역 영역 보존 (Copilot 영역 영역 의 try/catch 제거 영역 의 정정)

### 3.4 의도 영역 점검 (위험 점검)
- **`splitTableCell`** 영역 영역 `ih.exitCellSelectionMode?.()` 영역 영역 `executeOperation` **외부** 호출 (`if (isMultiCell)` 가드) — Undo 영역 영역 셀 선택 영역 미복원 영역. 의도적 영역 (UI 상태 영역 영역 Undo 범위 외).
- **`mergeTableCells`** 영역 영역 `ih.exitCellSelectionMode()` 영역 영역 `executeOperation` **외부** 호출 — 동일 의도.
- **`deleteTable`** 영역 영역 `ref ? cursorAt(ref) : cursorAt(pos)` 영역 영역 분기 영역 정합.
- **`toggleTableCaption`** 영역 영역 `hasCaption` 분기 영역 영역 `setTableProperties` 영역 만 snapshot 영역, 제거 영역 영역 기존 try/catch 보존 영역 영역 의도적 영역 (캡션 제거 시 paragraph deletion 영역 영역 별도 경로).

## 4. 인프라 점검 (PR 영역 영역 의 의존성)

### 4.1 `executeOperation({ kind: 'snapshot' })` 인프라 — ✅ 존재
- `rhwp-studio/src/engine/input-handler.ts:1485` 영역 영역 `case 'snapshot'` 영역 영역 `SnapshotCommand` 생성 + `history.execute()` + `cursor.moveTo()` + `afterEdit()`
- `WasmBridge.saveSnapshot() / restoreSnapshot()` (`wasm-bridge.ts:1084-1091`) 영역 영역 ✅ 존재
- 광범위 사용 영역 영역 — `input-handler-table.ts` (4건) / `input-handler-keyboard.ts` (6건) / `input-handler-text.ts` (1건) / `symbols-dialog.ts` (1건)

### 4.2 `document-changed` 이벤트
- 기존 코드 영역 영역 표 커맨드 영역 영역 `services.eventBus.emit('document-changed')` 직접 발행
- 본 PR 영역 영역 `executeOperation` → `afterEdit()` 영역 영역 자동 발행 영역 영역 수동 호출 제거
- 정합 영역 영역 입증: `input-handler.ts::executeOperation` 영역 영역 `afterEdit()` 호출 (ts 1485 후속)

## 5. 회귀 가드 (PR 본문 영역 영역 명시)

| 검증 | 결과 |
|------|------|
| `cargo test` | 전체 통과 (PR 본문) |
| `cargo clippy -- -D warnings` | 경고 0건 (PR 본문) |
| `tsc --noEmit` | 기존 WASM 모듈 타입 오류 외 신규 오류 없음 (PR 본문) |
| CI Canvas visual diff | ✅ SUCCESS |

PR 본문 영역 영역 명시적 회귀 테스트 신규 추가 부재 — TypeScript 변경 영역 영역 `tsc --noEmit` + Canvas visual diff CI 영역 영역 일차 영역 의 가드.

## 6. 본 환경 점검

- merge-base: `30351cdf` — 5/9 영역 매우 가까움
- merge-tree 충돌: **0건** ✓
- 변경 파일 단일 (`rhwp-studio/src/command/commands/table.ts`) — 격리 영역 영역
- 인프라 의존성 (`executeOperation/SnapshotCommand/saveSnapshot`) ✅ 모두 존재

## 7. 영향 범위

### 7.1 변경 영역
- rhwp-studio editor 영역 의 표 편집 11 커맨드 영역 영역 Undo/Redo 정합

### 7.2 무변경 영역
- 텍스트 편집 Undo/Redo (이미 정합)
- WASM 영역 영역 코어 (Rust 영역) — 변경 부재
- 표 편집 영역 영역 의 결과 (셀 구조/캡션) — 동일
- HwpCtrl(hwpctl) API 경로 — 별건 (PR 본문 명시)

### 7.3 위험 영역
- **셀 선택 모드 영역 영역 Undo 후 미복원** — `splitTableCell` / `mergeTableCells` 영역 영역 `exitCellSelectionMode` 영역 영역 snapshot 외부 호출 영역 영역 의도적. UX 영역 영역 후속 평가 가능.
- **`toggleTableCaption` 영역 의 캡션 제거 경로 영역 미정합** — 제거 (`hasCaption=false`) 영역 영역 기존 try/catch 보존 — 의도적 (paragraph deletion 영역 영역 별도 경로).

## 8. 후속 분리 (PR 본문 명시)

### 8.1 HwpCtrl(hwpctl) API 경로
`hwpctl/actions/table-edit.ts` 영역 영역 InputHandler 영역 영역 의 history 영역 미접근 영역 → 아키텍처 변경 필요 영역 영역 별건 — 후속 이슈 영역.

## 9. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 영역 영역 5/10 사이클 영역 영역 진전, 본 PR 영역 영역 단일 파일 변경 영역 영역 충돌 부재

## 10. 처리 옵션

### 옵션 A — 1번째 commit cherry-pick + Copilot 정정 commit cherry-pick + no-ff merge (추천)

PR 영역 영역 2 commits 영역 영역 모두 본질 영역 (Copilot 영역 영역 의 정정 영역 의 의미 보존 영역). 두 commit 모두 cherry-pick.

```bash
git checkout -b local/task204 5cee2615
git cherry-pick 25a9d1b2 b8477e5f
git checkout local/devel
git merge --no-ff local/task204
```

→ **옵션 A 추천**.

## 11. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean (PR 변경 영역 영역 — Rust 영역 변경 부재 영역 영역 무영향)
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] `cd rhwp-studio && npm run build` 통과
- [ ] 광범위 sweep — Rust 변경 부재 영역 영역 영향 없음 (170 페이지 / 회귀 0 보장)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 시각 판정 권장**

본 PR 영역 영역 의 본질 영역 영역 **rhwp-studio editor 영역 의 사용자 인터랙션 (Undo/Redo)**:
- WASM 빌드 후 dev server 영역 영역 표 편집 후 Ctrl+Z 영역 영역 11 커맨드 영역 정합 점검
- Canvas visual diff CI 영역 영역 SUCCESS 영역 영역 일차 가드 통과
- E2E 자동 테스트 신규 영역 영역 부재 → 작업지시자 직접 인터랙션 검증 권장

## 12. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 핵심 컨트리뷰터 (rhwp-studio + WASM API + 디버깅 툴킷 영역) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 파일 변경 — Rust 렌더링 경로 영역 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (B안) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 — Canvas visual diff CI + 작업지시자 인터랙션 검증 |

## 13. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 2 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo build/test + tsc/build + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (Ctrl+Z 표 편집 11 커맨드)
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #728 close + Issue #204 close (closes 자동 정합 부재 영역 영역 수동 close)

---

작성: 2026-05-10
