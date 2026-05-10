---
PR: #728
제목: Task #204 — 표 편집 Undo/Redo 스냅샷 기반 이력 등록
컨트리뷰터: @oksure — 20+ 사이클 핵심 컨트리뷰터 (rhwp-studio + WASM API + 디버깅 툴킷 영역)
처리: 옵션 A — 2 commits cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: 8db18f58
---

# PR #728 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge `8db18f58`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `8db18f58` (--no-ff merge) |
| Issue | #204 (PR 본문 직접 closes 미명시 영역 영역 본질 정정 영역 close) |
| 시각 판정 | 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN + sweep 170/170 same + WASM 빌드 |

## 2. 정정 본질 — `rhwp-studio/src/command/commands/table.ts` (+123/-101)

### 2.1 결함 본질 (Issue #204)
표 편집 (생성/삭제/행열 삽입·삭제/셀 분할·병합/캡션) 영역 영역 WASM 직접 호출 영역 영역 `history.execute()` 미경유 → `Ctrl+Z` Undo 미동작.

### 2.2 정정 (B안 — 스냅샷 + SnapshotCommand 일괄 래핑)

11 표 커맨드 영역 영역 동일 패턴 영역 영역 일관 적용:

```typescript
safeTableOp(() => ih.executeOperation({
  kind: 'snapshot',
  operationType: 'insertTableRow',
  operation: (wasm) => {
    wasm.insertTableRow(...);
    return pos;  // 새 cursor 위치
  },
}), '줄 추가');
```

### 2.3 대상 커맨드 (11)

| 커맨드 | operationType |
|--------|---------------|
| `table:create` | `createTable` |
| `table:insert-row-above` / `-below` | `insertTableRow` (×2) |
| `table:insert-col-left` / `-right` | `insertTableColumn` (×2) |
| `table:delete-row` | `deleteTableRow` |
| `table:delete-col` | `deleteTableColumn` |
| `table:cell-split` | `splitTableCell` (단일/다중 셀 분기) |
| `table:cell-merge` | `mergeTableCells` |
| `table:delete` | `deleteTable` (선택 모드 + 셀 내부 분기) |
| `table:caption-toggle` | `toggleTableCaption` |

### 2.4 Copilot 리뷰 반영 (`b8477e5f`)
- `createTable` 영역 영역 `controlIndex: 0` (하드코드) → `result.controlIdx` 사용
- 에러 핸들링 영역 영역 `safeTableOp` 영역 영역 보존

## 3. 인프라 재사용 (B안 영역 권위 사례)

### 3.1 `executeOperation({ kind: 'snapshot' })` 영역 영역 ✅ 존재
- `rhwp-studio/src/engine/input-handler.ts:1485` — `case 'snapshot'` 영역 `SnapshotCommand` 생성 + `history.execute()` + `cursor.moveTo()` + `afterEdit()`
- `WasmBridge.saveSnapshot/restoreSnapshot` (`wasm-bridge.ts:1084-1091`) ✅
- 광범위 사용: `input-handler-table.ts` (4건) + `input-handler-keyboard.ts` (6건) + `input-handler-text.ts` (1건) + `symbols-dialog.ts` (1건)

### 3.2 신규 인프라 도입 부재
PR 영역 영역 영역 신규 인프라 도입 부재 → **위험 좁힘** (`feedback_process_must_follow` 영역 의 정합).

## 4. 본 환경 cherry-pick + 검증

### 4.1 cherry-pick (2 commits)
```
2d13fdca Task #204: 표 편집 Undo/Redo — 스냅샷 기반 이력 등록
93d9bcc6 Task #204: Copilot 리뷰 반영 — controlIdx 사용 + 에러 핸들링 복원
```
충돌 0건.

### 4.2 결정적 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (2 commits) | ✅ 충돌 0건 |
| `tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN (Rust 변경 부재) |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |
| WASM 빌드 (Docker) | ✅ 4.60 MB (`pkg/rhwp_bg.wasm`) |

### 4.3 작업지시자 웹 에디터 검증 ✅ 통과
- dev server 영역 영역 11 표 커맨드 영역 영역 Ctrl+Z 영역 영역 정합 점검
- 실제 인터랙션 영역 영역 정합 입증

## 5. 영향 범위

### 5.1 변경 영역
- rhwp-studio editor 영역 의 표 편집 11 커맨드 영역 영역 Undo/Redo 정합

### 5.2 무변경 영역
- 텍스트 편집 Undo/Redo (이미 정합)
- WASM 코어 (Rust) — 변경 부재
- 표 편집 결과 (셀 구조/캡션) — 동일
- HwpCtrl(hwpctl) API 경로 — 별건 (PR 본문 명시)

### 5.3 의도적 미정합
- `splitTableCell` / `mergeTableCells` 영역 영역 `exitCellSelectionMode` 영역 영역 snapshot 외부 호출 — 셀 선택 모드 영역 영역 Undo 범위 외 (UI 상태)
- `toggleTableCaption` 영역 영역 캡션 제거 경로 영역 미정합 — paragraph deletion 영역 영역 별도 경로 (의도적)

## 6. 후속 분리 (PR 본문 명시)

### 6.1 HwpCtrl(hwpctl) API 경로
`hwpctl/actions/table-edit.ts` 영역 영역 InputHandler 영역 영역 의 history 영역 미접근 영역 → 아키텍처 변경 필요 영역 별건 — 후속 이슈 영역.

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** 핵심 컨트리뷰터 (rhwp-studio + WASM API + 디버깅 툴킷 영역) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 파일 변경 — Rust 렌더링 경로 영역 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (B안) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 + 후속 분리 (HwpCtrl API) 명시 |
| `feedback_visual_judgment_authority` | 작업지시자 웹 에디터 인터랙션 검증 영역 영역 시각 판정 권위 영역 (Canvas visual diff CI 영역 영역 일차 가드 + 인터랙션 영역 영역 본질 가드) |

## 8. 잔존 후속

- 본 PR 본질 정정 영역 의 잔존 결함 부재
- HwpCtrl(hwpctl) API 경로 영역 영역 별건 (이슈 등록 권장)

---

작성: 2026-05-10
