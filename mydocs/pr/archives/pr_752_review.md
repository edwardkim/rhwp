---
PR: #752
제목: feat — edit:delete (지우기) 커맨드 구현 (선택 영역/개체 삭제)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 20번째 PR)
base / head: devel / contrib/edit-delete-command
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: SUCCESS (Build & Test + CodeQL js/ts/py/rust + Canvas visual diff)
변경 규모: +42 / -2, 3 files
검토일: 2026-05-10
---

# PR #752 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #752 |
| 제목 | feat — edit:delete (지우기) 커맨드 구현 |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 20번째 PR) |
| base / head | devel / contrib/edit-delete-command |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 부재 |
| CI | ✅ Build & Test + CodeQL (js/ts/py/rust) + Canvas visual diff 통과 |
| 변경 규모 | +42 / -2, 3 files |
| 커밋 수 | 2 (feat + Copilot 리뷰) |
| Issue 연결 | 부재 (편집 메뉴 stub 활성화 영역 영역 자기완결 정정) |

## 2. 결함 본질

`edit:delete` 커맨드 영역 영역 `canExecute: () => false` stub 영역 영역 메뉴 비활성 상태. 편집 > 지우기 메뉴 + Ctrl+E 단축키 미동작.

## 3. 채택 접근 — `performCut` 패턴 정합

기존 `performCut()` (오려두기, `input-handler.ts:2350`) 영역 영역 동일 구조 영역 영역 클립보드 복사만 제외:

| 상태 | performCut (기존) | performDelete (본 PR 신규) |
|------|-------------------|---------------------------|
| 그림/도형 개체 선택 | performCopy + snapshot 삭제 | snapshot 삭제만 |
| 표 개체 선택 | performCopy + snapshot 삭제 | snapshot 삭제만 |
| 텍스트 선택 | `execCommand('cut')` (clipboard + 삭제) | `deleteSelection()` (삭제만) |
| 선택 없음 | `canExecute` 차단 | 동일 |

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `executeOperation({ kind: 'snapshot' })` (기존, PR #728 인프라) | snapshot Undo 기록 |
| `deleteSelection()` (기존 private 메서드, `input-handler.ts:1441`) | 텍스트 선택 영역 삭제 |
| `deleteObjectControl()` 헬퍼 (기존, `input-handler.ts:1841`) | image/equation → deletePictureControl, shape/group/line → deleteShapeControl 분기 (Copilot 리뷰 영역 영역 사용) |
| `deleteTableControl` WASM API (기존) | 표 개체 삭제 |
| `cursor.moveOutOfSelectedPicture/Table` (기존) | 개체 선택 해제 |
| EditorContext (`hasSelection` / `inPictureObjectSelection` / `inTableObjectSelection`) (기존) | `canExecute` 가드 |

→ 신규 인프라 도입 부재 — `performCut` 패턴 정합 영역 영역 위험 좁힘 (`feedback_process_must_follow` 정합).

## 5. PR 의 정정 — 3 files, +42/-2

### 5.1 `rhwp-studio/src/engine/input-handler.ts` (+36, 신규 메서드)

```typescript
performDelete(): void {
  if (this.cursor.isInPictureObjectSelection()) {
    const ref = this.cursor.getSelectedPictureRef();
    if (ref) {
      this.cursor.moveOutOfSelectedPicture();
      this.pictureObjectRenderer?.clear();
      this.eventBus.emit('picture-object-selection-changed', false);
      this.executeOperation({ kind: 'snapshot', operationType: 'deleteObject', operation: (wasm: WasmBridge) => {
        this.deleteObjectControl(ref);  // image/equation/shape/group/line 분기
        return this.cursor.getPosition();
      }});
    }
    return;
  }
  if (this.cursor.isInTableObjectSelection()) {
    const ref = this.cursor.getSelectedTableRef();
    if (!ref) return;
    if (ref.cellPath && ref.cellPath.length > 1) {  // 중첩 표 가드 (Copilot 리뷰)
      this.cursor.moveOutOfSelectedTable();
      this.eventBus.emit('table-object-selection-changed', false);
      return;
    }
    this.cursor.moveOutOfSelectedTable();
    this.eventBus.emit('table-object-selection-changed', false);
    this.executeOperation({ kind: 'snapshot', operationType: 'deleteTable', operation: (wasm: WasmBridge) => {
      wasm.deleteTableControl(ref.sec, ref.ppi, ref.ci);
      return this.cursor.getPosition();
    }});
    return;
  }
  if (this.cursor.hasSelection()) {
    this.deleteSelection();
  }
}
```

### 5.2 `rhwp-studio/src/command/commands/edit.ts` (+4/-2)

```typescript
canExecute: (ctx) => ctx.hasDocument && (ctx.hasSelection || ctx.inPictureObjectSelection || ctx.inTableObjectSelection),
execute(services) {
  services.getInputHandler()?.performDelete();
},
```

stub `() => false` → 실제 `canExecute` + `performDelete` 호출.

### 5.3 `rhwp-studio/src/command/shortcut-map.ts` (+2)

```typescript
[{ key: 'e', ctrl: true }, 'edit:delete'],
```

`Ctrl+E` 단축키 매핑 추가 (커맨드 영역 영역 기존 `shortcutLabel` 정합).

## 6. Copilot 리뷰 반영 (commit `d31b4512`)

- **`deleteObjectControl()` 헬퍼 사용**: 직접 `deletePictureControl` / `deleteShapeControl` 호출 영역 영역 `deleteObjectControl(ref)` 영역 영역 위임 — equation/group/line 타입 영역 영역 올바른 삭제 API 호출 (image/equation→deletePictureControl, shape/group/line→deleteShapeControl)
- **중첩 표 가드**: `cellPath.length > 1` 영역 영역 삭제 시도 차단 + 선택 해제만 수행 (중첩 표 영역 영역 deleteTableControl 영역 영역 결함 가능성 차단)

## 7. 충돌 / mergeable

mergeStateStatus = `BEHIND`, mergeable = `MERGEABLE`.

본 환경 점검:
- `rhwp-studio/src/engine/input-handler.ts` — devel HEAD 영역 영역 PR #748 영역 영역 안정 (table 영역 영역 변경, performCut 영역 영역 변경 부재)
- `rhwp-studio/src/command/commands/edit.ts` — devel 5/10 사이클 영역 영역 안정
- `rhwp-studio/src/command/shortcut-map.ts` — PR #749/#750/#751 누적 (Ctrl+O, Ctrl+Alt+Enter, Alt+Shift+ㅗ/ㅊ/ㅇ) — Ctrl+E 영역 영역 다른 영역 영역 충돌 부재

→ cherry-pick 충돌 0건 예상 (auto-merge 정합).

## 8. 본 환경 점검

### 8.1 변경 격리
- TypeScript 단일 영역 (rhwp-studio editor)
- Rust / WASM / 렌더링 경로 무관 (`feedback_image_renderer_paths_separate` 정합)

### 8.2 CI 결과
- Build & Test ✅
- CodeQL (js/ts/py/rust) ✅
- Canvas visual diff ✅
- WASM Build SKIPPED (변경 무관)

### 8.3 의도적 제한
- **중첩 표 (cellPath.length > 1) 삭제 차단** — 명시적 가드. `feedback_hancom_compat_specific_over_general` 정합 — 일반화 알고리즘 영역 영역 결함 위험 좁힘
- **클립보드 복사 부재** — `edit:delete` 본질 (Cut 과 구분되는 Delete) 정합

## 9. 처리 옵션

### 옵션 A — 2 commits 개별 cherry-pick + no-ff merge

```bash
git checkout local/devel
git cherry-pick 0db460db d31b4512
git checkout devel
git merge local/devel --no-ff -m "Merge PR #752: feat edit:delete 커맨드 구현"
```

→ **권장**.

## 10. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] cherry-pick 충돌 0건
- [ ] `cargo build --release` 통과 (Rust 변경 부재)
- [ ] `cargo test --release` ALL GREEN
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (TypeScript 영역 영역 SVG 무영향 입증)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 본질은 **rhwp-studio editor 편집 커맨드**:
- WASM 빌드 후 dev server 영역 영역:
  - 텍스트 선택 → Ctrl+E → 선택 영역 삭제 (Undo 가능)
  - 그림 선택 → Ctrl+E → 그림 삭제 (Undo 가능)
  - 표 선택 → Ctrl+E → 표 삭제 (Undo 가능)
  - 중첩 표 선택 → Ctrl+E → 선택 해제만 (삭제 차단)
  - 메뉴 영역 영역 "편집 → 지우기" → 동일 동작
- E2E 자동 테스트 신규 부재 → 작업지시자 직접 인터랙션 검증 권장

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 20번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 영역 Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (`performCut` 패턴 + `deleteObjectControl` 헬퍼 + `executeOperation snapshot` PR #728 인프라) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | 중첩 표 가드 (`cellPath.length > 1`) — 일반화 영역 영역 결함 위험 좁힘 |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 영역 작업지시자 인터랙션 검증 권장 |

## 12. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 옵션 A cherry-pick (2 commits)
2. 자기 검증 (cargo test + tsc + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (텍스트 / 그림 / 표 / 중첩 표 + Undo)
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #752 close

---

작성: 2026-05-10
