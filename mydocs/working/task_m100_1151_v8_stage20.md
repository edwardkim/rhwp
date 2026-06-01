# Task #1151 v8 Stage 20 완료 보고서 — 한컴 native 시연 비교 + 3 결함 fix

수행계획서: [task_m100_1151_v4.md](../plans/task_m100_1151_v4.md) (v5~v8 모두 v4 PR #1173 머지 전 same-PR fix) · 상위 v7 Stage 16: [task_m100_1151_v7_stage16.md](task_m100_1151_v7_stage16.md)

## 1. 사용자 한컴 native 시연 (2026-05-30)

PR #1173 (v7 까지) 발행 후 사용자가 HWP 5.0 스펙 (`mydocs/tech/한글문서파일형식_5.0_revision1.3.md`) 와 정합 여부 검토 요청 + 한컴 native 시연으로 3 결함 확정.

### 시연 비교

| Step | 한컴 native | rhwp (v7) | 결함 |
|------|------------|-----------|------|
| 1,2 (셀 안 picture 신규 삽입) | 가로/세로 기준 = **종이** (Paper) | 가로/세로 기준 = **쪽** (Page) | ✗ **A** |
| 3,4 (글자처럼 체크) | dialog 비활성 (회색) | dialog 비활성 | OK |
| 5,6 (글자처럼 해제) | 가로 기준 = **문단**, 세로 = 문단 | 가로 = **select 없음**, 세로 = 문단 | ✗ **B** |
| 시각 위치 (사용자가 그린 곳) | 사용자 클릭/드래그 위치 | **표 좌상단 위 (페이지 콘텐츠 시작점)** | ✗ **C** |

## 2. 결함별 root cause + fix

### 결함 A — v1 신규 picture 초기 rel_to = Page (한컴 native = Paper)

**Root cause**: `object_ops.rs:1634` v1 셀 분기에서 `(1 << 3) | (1 << 8)` (Page bits) + `VertRelTo::Page / HorzRelTo::Page` 명시. v1 plan (`task_m100_1151.md:14-15`) 의 incellpicture.hwp dump 분석에서 정답이 **Paper** (offset=11845/15595) 라고 명시했으나 v1 구현이 plan 을 따르지 않은 버그.

**Fix** (Stage 18):
```rust
let common_attr: u32 = (4 << 15) | (2 << 18);  // vert/horz bits = 0 (Paper)
vert_rel_to: VertRelTo::Paper,
horz_rel_to: HorzRelTo::Paper,
```

**Regression test**: `v8_cell_floating_picture_uses_paper_rel_to` (object_ops.rs)
- typed field Paper 검증
- attr bits 3-4 / 8-10 모두 0 검증
- tac=false, wrap=Square 유지

### 결함 B — rhwp dialog 의 horzRelSelect 옵션에 Para 누락

**Root cause**: `picture-props-dialog.ts:495`
```typescript
this.horzRelSelect = this.selectEl([
  ['Paper', '종이'], ['Page', '쪽'], ['Column', '단'],
  // ← Para(문단) 누락! → picture.common.horz_rel_to=Para 시 매칭 실패
]);
```
비교 — `vertRelSelect` (line 515) 는 Paper/Page/Para 포함 ✓.

스펙 (`pack_common_attr_bits` 의 `horz_rel_to_to_bits`): horz_rel_to valid 값 = Paper(0)/Page(1)/Column(2)/**Para(3)**.

**Fix** (Stage 19):
```typescript
this.horzRelSelect = this.selectEl([
  ['Paper', '종이'], ['Page', '쪽'], ['Column', '단'], ['Para', '문단'],
]);
```

### 결함 C — picture 위치 = 사용자 클릭/드래그 좌표 (한컴 native 정합)

**진단**: 결함 A fix 후 picture rel_to=Paper 정합되었으나 실제 시각상 picture 가 표 좌상단 위 (페이지 콘텐츠 시작점) 에 그려짐. 진단 테스트 결과:
```
rhwp picture offset = (30, 35) mm paper-relative = page margin (30, 20) + 표 시작점
한컴 incellpicture.hwp offset = (41.8, 55.0) mm = 사용자가 그린 셀 안 위치
```

**Root cause**:
- `wasm_api.rs:insertPicture` 시그니처에 클릭 위치 매개변수 없음
- `compute_cell_page_offset` (object_ops.rs:1850) 가 **셀 좌상단** 좌표 반환
- studio `finishImagePlacement` (input-handler-table.ts:255) 가 `drag.startClientX/Y` 갖고 있지만 wasm 에 전달 안 함

→ wasm 이 셀 좌상단 좌표만 사용 → 사용자 입력 위치 무시.

**Fix** (Stage 20):

1. **insert_picture_native** (object_ops.rs:1528): `paper_offset_x_hu: Option<i32>, paper_offset_y_hu: Option<i32>` 매개변수 추가:
```rust
let (offset_x_hu, offset_y_hu) = match (paper_offset_x_hu, paper_offset_y_hu) {
    (Some(x), Some(y)) => (x, y),
    _ => self.compute_cell_page_offset(section_idx, para_idx, cell_path),
};
```
None 이면 셀 좌상단 default (기존 동작 호환).

2. **wasm_api.rs:insert_picture**: 동일 매개변수 추가, wasm_bindgen 이 `Option<i32>` → TS `number | undefined` 매핑.

3. **wasm-bridge.ts:insertPicture wrapper**: `paperOffsetXHu?, paperOffsetYHu?` 추가.

4. **input-handler-table.ts:finishImagePlacement**: `inCell` 인 경우 drag.startClientX/Y → page-relative px → paper-relative HU 변환:
```ts
if (inCell) {
  const contentRect = scrollContent.getBoundingClientRect();
  const dragContentX = drag.startClientX - contentRect.left;
  const dragContentY = drag.startClientY - contentRect.top;
  const pageIdx = this.virtualScroll.getPageAtPoint(dragContentX, dragContentY);
  const pageOffset = this.virtualScroll.getPageOffset(pageIdx);
  const pageLeft = this.virtualScroll.getPageLeftResolved(pageIdx, scrollContent.clientWidth);
  const dragPageX = (dragContentX - pageLeft) / zoom;
  const dragPageY = (dragContentY - pageOffset) / zoom;
  paperOffsetXHu = Math.round(dragPageX * 75);
  paperOffsetYHu = Math.round(dragPageY * 75);
}
```

본문 inline (inCell=false) 또는 클립보드 paste 는 undefined → wasm default 동작 유지.

5. 8 개 단위 테스트 caller (`insert_picture_native`) 모두 `None, None` 추가.

## 3. attr stale 결함 (이전 가설) 평가

PR 분석 중 검토했던 v2 `migrate_picture_floating_to_inline` 의 attr bits 미갱신 결함은 **fix 불필요로 확정**:
- 사용자 시연 결과 한컴 native 가 typed field 우선 path 로 동작 + 글자처럼 해제 시 Para reset
- rhwp dialog 의 `format_picture_properties_json` 도 typed field 기반
- → attr bits stale 자체는 실무 영향 없음 (한컴 / rhwp 동일 동작)

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` 전수 | **1446 passed, 0 failed, 6 ignored** (v8 신규 1 추가, 회귀 0) |
| `cargo clippy --lib -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| 사용자 시연 (v8 WASM 재빌드 후) | 정상 작동 확인 ✓ |

## 5. 변경 파일 (v8 Stage 18+19+20)

### Rust
- `src/document_core/commands/object_ops.rs`
  - v1 셀 분기 attr bits + typed field Paper (Stage 18)
  - insert_picture_native 시그니처에 paper_offset_x_hu / y_hu (Stage 20)
  - 8 곳 단위 테스트 caller None 추가 (Stage 20)
  - v8_cell_floating_picture_uses_paper_rel_to regression test 추가 (Stage 18)
- `src/wasm_api.rs:insert_picture` 시그니처 확장 (Stage 20)

### Studio
- `rhwp-studio/src/ui/picture-props-dialog.ts:495` horzRelSelect 에 Para 추가 (Stage 19)
- `rhwp-studio/src/core/wasm-bridge.ts:insertPicture` wrapper 확장 (Stage 20)
- `rhwp-studio/src/engine/input-handler-table.ts:finishImagePlacement` paper offset 변환 (Stage 20)

## 6. Stage 21 진입 조건

- 3 결함 모두 fix + regression 통과 ✓
- 사용자 시연 정상 작동 ✓
- attr stale 가설 reject (사용자 시연 결과 기반)

→ Stage 21 (v8 통합 최종 보고서 갱신 + push origin → PR #1173 자동 갱신) 진행.
