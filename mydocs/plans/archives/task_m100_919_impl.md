# Task M100-919 구현 계획서

- 이슈: [#919](https://github.com/edwardkim/recv/rhwp/issues/919)
- 브랜치: `local/task919`
- 수행 계획서: `mydocs/plans/task_m100_919.md` (승인)
- Stage 1 진단: `mydocs/working/task_m100_919_stage1.md`

## 1. 단계 분해 (5 단계)

### Stage 2.1 — Native: `RenderNodeType::TextBox` 수집 + `getShapeBBox` API (예상 1시간)

#### 변경 위치

**1. `src/document_core/queries/cursor_rect.rs`**

`collect_runs` (line 564~593) 에 `TextBox` 노드 수집:

```rust
struct TextBoxBboxInfo {
    section_index: usize,
    parent_para_index: usize,
    control_index: usize,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    // 글상자 첫 paragraph (셀 진입 시 cursor 위치 기본값)
    first_inner_para_index: Option<usize>,
}

fn collect_runs(...) {
    // 기존 TableCell 처리 (line 565~593) 동일
    // 신규: TextBox 노드 처리
    if let RenderNodeType::TextBox = node.node_type {
        // 글상자의 메타 정보를 부모 traverse 컨텍스트에서 추출
        // (Shape 컨트롤의 section/parent_para/control_idx 전파 필요)
    }
}
```

**메타 정보 전달 패턴**: `cell_bboxes` 의 `table_meta` 와 유사하게 `textbox_meta` 컨텍스트 전파 — Shape control 진입 시 (sec, ppi, ci) 기록 후 자식 `TextBox` 노드에서 활용.

`hit_test_native` 의 매칭 로직 추가 (line 880~923):

```rust
// 신규 — 텍스트박스 빈 영역 매칭 (cell 매칭 후, 본문 fall-through 전)
// 텍스트 hit 가 없고 텍스트박스 외부 경계선이 아닐 때 → 글상자 첫 paragraph 시작
let clicked_textbox: Option<&TextBoxBboxInfo> = textbox_bboxes.iter()
    .filter(|tb| x >= tb.x && x <= tb.x + tb.w && y >= tb.y && y <= tb.y + tb.h)
    .min_by_key(|tb| ...);

if hit_body.is_none() && hit_cell.is_none() && clicked_cell.is_none() {
    if let Some(tb) = clicked_textbox {
        // 글상자 안 첫 paragraph 시작 위치 반환
        return Ok(format_textbox_entry(tb, page_num));
    }
}
```

**2. `src/wasm_api.rs`** — `getShapeBBox` 신규 API

```rust
/// 글상자/도형의 페이지 좌표 바운딩박스를 반환한다.
///
/// 반환: JSON `{"pageIndex":<N>,"x":<f>,"y":<f>,"width":<f>,"height":<f>}`
#[wasm_bindgen(js_name = getShapeBBox)]
pub fn get_shape_bbox(
    &self,
    section_idx: u32,
    parent_para_idx: u32,
    control_idx: u32,
) -> Result<String, JsValue> {
    self.get_shape_bbox_native(
        section_idx as usize,
        parent_para_idx as usize,
        control_idx as usize,
    ).map_err(|e| e.into())
}

pub fn get_shape_bbox_native(...) -> Result<String, HwpError> {
    // page_layout / render_tree 에서 Shape 컨트롤의 bbox 추출
    // (getTableBBox 와 유사 패턴)
}
```

### Stage 2.2 — Studio: `isShapeBorderClick` + 클릭 흐름 정정 (예상 1시간)

**1. `rhwp-studio/src/engine/input-handler.ts`** — 신규 헬퍼

```typescript
/** 클릭 좌표가 글상자/도형 외곽 경계선 위인지 판별한다. */
private isShapeBorderClick(
  pageX: number, pageY: number,
  sec: number, ppi: number, ci: number,
): boolean {
  try {
    const bbox = this.wasm.getShapeBBox(sec, ppi, ci);
    const tolerance = 5;
    // isTableBorderClick 과 동일 로직
    const nearLeft = Math.abs(pageX - bbox.x) <= tolerance;
    const nearRight = Math.abs(pageX - (bbox.x + bbox.width)) <= tolerance;
    const nearTop = Math.abs(pageY - bbox.y) <= tolerance;
    const nearBottom = Math.abs(pageY - (bbox.y + bbox.height)) <= tolerance;
    const inVertRange = pageY >= bbox.y - tolerance && pageY <= bbox.y + bbox.height + tolerance;
    const inHorzRange = pageX >= bbox.x - tolerance && pageX <= bbox.x + bbox.width + tolerance;
    return (nearLeft && inVertRange) || (nearRight && inVertRange)
        || (nearTop && inHorzRange) || (nearBottom && inHorzRange);
  } catch { return false; }
}

/** 외곽 근처 클릭 시 글상자 후보 탐색. */
private findShapeByOuterClick(
  pageX: number, pageY: number,
  sec: number, paragraphIndex: number,
): { sec: number; ppi: number; ci: number } | null {
  // findTableByOuterClick 동일 패턴
  for (let offset = 0; offset <= 2; offset++) {
    const candidates = offset === 0 ? [paragraphIndex] : [paragraphIndex - offset, paragraphIndex + offset];
    for (const ppi of candidates) {
      if (ppi < 0) continue;
      // paragraph 의 controls 중 Shape 검사 — ci 알 수 없으면 0..N 시도
      for (let ci = 0; ci < 10; ci++) {
        if (this.isShapeBorderClick(pageX, pageY, sec, ppi, ci)) {
          return { sec, ppi, ci };
        }
      }
    }
  }
  return null;
}
```

**2. `rhwp-studio/src/engine/input-handler-mouse.ts`** — 클릭 흐름 정정

기존 (line 662~677):
```typescript
// 글상자 내부 텍스트 직접 히트 → 바로 캐럿 진입
if (hit.isTextBox) {
  this.exitPictureObjectSelectionIfNeeded();
  this.cursor.moveTo(hit);
  ...
}
```

신규 추가 (글상자 빈 영역 hit 처리 + 외부 경계선 객체 선택):
```typescript
// 표 경계선 (기존 line 627~) 이후, 글상자 경계선 검사 추가
if (hit.parentParaIndex !== undefined && hit.controlIndex !== undefined && hit.isTextBox) {
  if (this.isShapeBorderClick(pageX, pageY, hit.sectionIndex, hit.parentParaIndex, hit.controlIndex)) {
    // 한컴 UX: 외부 경계선 클릭 → 글상자 객체 선택
    this.cursor.clearSelection();
    this.cursor.enterPictureObjectSelectionDirect(hit.sectionIndex, hit.parentParaIndex, hit.controlIndex, 'shape');
    this.caret.hide();
    this.selectionRenderer.clear();
    this.renderPictureObjectSelection();
    this.eventBus.emit('picture-object-selection-changed', true);
    this.textarea.focus();
    return;
  }
}

// 글상자 외부 (paragraph hit 가 본문) → 외곽 근처 검사
if (!hit.isTextBox) {
  const shapeHit = this.findShapeByOuterClick(pageX, pageY, hit.sectionIndex, hit.paragraphIndex);
  if (shapeHit) {
    this.cursor.clearSelection();
    this.cursor.enterPictureObjectSelectionDirect(shapeHit.sec, shapeHit.ppi, shapeHit.ci, 'shape');
    ...
    return;
  }
}

// 글상자 내부 (isTextBox=true) — 즉시 cursor 진입
if (hit.isTextBox) {
  this.exitPictureObjectSelectionIfNeeded();
  this.cursor.moveTo(hit);
  ...
}
```

**기존 글상자 객체 선택 흐름 (line 679~720)** — 도형 (image/shape) 1차 클릭 처리 분기:
- 한컴 UX 변경: 글상자 (`shape`) 의 경우 즉시 객체 선택 (1차 클릭) 분기 **제거**.
- Image/Equation 등은 기존 패턴 유지 (1차 클릭 → 객체 선택, 더블클릭 → 편집).
- 글상자만 한컴 UX 적용: 외부 경계선 → 객체 선택, 내부 → 텍스트 편집 진입.

### Stage 2.3 — Studio: 객체 선택 → 편집 진입 (Enter/내부 클릭) (예상 30분)

**`rhwp-studio/src/engine/input-handler-keyboard.ts:590`** — Enter 키로 글상자 편집 진입 이미 구현 확인.

**`rhwp-studio/src/engine/input-handler-mouse.ts:866~896`** — `onDblClick` 의 글상자 객체 선택 → `enterTextboxEditing` 정합 유지 (한컴 UX: 객체 선택 상태에서 다시 클릭 또는 더블클릭 둘 다 진입 가능).

추가: 객체 선택 상태에서 **다시 클릭** (single click) 시 텍스트 편집 진입:
```typescript
// onMouseDown 의 객체 선택 상태 분기 확장
if (this.cursor.isInPictureObjectSelection()) {
  const ref = this.cursor.getSelectedPictureRef();
  if (ref && ref.type === 'shape') {
    // 클릭한 좌표가 글상자 내부면 → 텍스트 편집 진입
    if (!this.isShapeBorderClick(pageX, pageY, ref.sec, ref.ppi, ref.ci)) {
      // 글상자 안 클릭 → 편집 진입
      this.cursor.exitPictureObjectSelection();
      this.enterTextboxEditing(ref.sec, ref.ppi, ref.ci);
      // hit_test 재호출하여 cursor 위치 결정
      const hit2 = this.wasm.hitTest(pageIdx, pageX, pageY);
      this.cursor.moveTo(hit2);
      this.textarea.focus();
      return;
    }
  }
}
```

### Stage 2.4 — 회귀 가드 + CI (예상 30분)

**`tests/issue_919_textbox_hit_test.rs`** 신규:

```rust
//! Issue #919: 글상자 hit_test 한컴 UX 정합 회귀 가드

use std::fs;
use std::path::Path;

#[test]
fn issue_919_textbox_inner_text_hit_returns_textbox_path() {
    // (200, 200) → 글상자 안 텍스트 위 → cellPath + isTextBox=true
}

#[test]
fn issue_919_textbox_inner_empty_hit_returns_first_inner_para() {
    // (100, 180) → 글상자 안 빈 영역 → 글상자 첫 paragraph 진입
}

#[test]
fn issue_919_textbox_outer_text_hit_returns_body() {
    // 글상자 외부 명백히 → 본문 paragraph
}

#[test]
fn issue_919_get_shape_bbox_returns_correct_dimensions() {
    // getShapeBBox(0, 0, 2) → x≈75.6, y≈75.6, w≈628.5, h≈976.3
}
```

**CI 패턴** (`feedback_push_full_test_required`):
- `cargo test --release --lib + --tests`
- `cargo clippy --release --lib`
- `cargo fmt --all -- --check`

### Stage 2.5 — WASM 빌드 + 작업지시자 시각 판정 + 머지 (예상 30분)

1. WASM Docker 빌드 + rhwp-studio/public 동기화
2. **작업지시자 시각 판정** (`feedback_visual_judgment_authority`):
   - `samples/table-in-tbox.hwp` page 1+2 의 글상자 클릭 동선:
     - 외부 경계선 → 객체 선택 ✓
     - 내부 (텍스트/빈 영역/안 표) → 즉시 텍스트 편집 진입 ✓
     - 글상자 안 표 셀 / 이미지 직접 클릭 → 정상 hit (글상자 가로채기 부재) ✓
     - 편집 중 Esc → 객체 선택 ✓ (이미 구현)
   - 기존 표 fixture (`exam_social.hwp`) 회귀 부재
3. 최종 보고서 `mydocs/report/task_m100_919_report.md`
4. orders/20260521.md 갱신
5. no-ff merge + push + close

## 2. 회귀 위험 평가

| 영역 | 위험도 | 근거 |
|------|--------|------|
| 글상자 hit_test (본 PR 본질) | **의도된 변경** | TextBox 노드 수집 + 빈 영역 매칭 추가 |
| 표 hit_test | **낮음** | TextBox 처리는 신규 분기, cell_bboxes 무변경 |
| 본문 paragraph hit | **낮음** | hit_body fall-through 변경 없음, textbox 매칭은 cell 매칭 후 |
| Image/Equation 객체 선택 | **영향 없음** | 글상자 (`shape`) 만 한컴 UX 적용, Image/Equation 기존 패턴 유지 |
| Esc 편집 모드 탈출 | **영향 없음** | 이미 구현된 패턴 (input-handler-keyboard.ts:937) |
| 글상자 안 표 셀 hit (중첩) | **개선** | 글상자 가로채기 제거로 정상 hit 가능 |

## 3. 잠재 위험 — Stage 1 진단 부수 발견

`hit_test_native` 가 페이지 본문 영역 밖 (y > body_area.bottom) 클릭을 글상자 안 마지막 paragraph 로 매핑. 본 task 범위 외 — 글상자 안 paragraph 좌표 정밀도 별도 검토 후속.

## 4. 메모리 룰 정합

- ✅ `feedback_visual_judgment_authority` — rhwp-studio 시각 판정 게이트
- ✅ `feedback_diagnosis_layer_attribution` — 4 코드 갭 정확 식별 (cursor_rect / wasm_api / mouse handler / border helper)
- ✅ `feedback_hancom_compat_specific_over_general` — 글상자 (`shape`) 한정, Image/Equation 무변경
- ✅ `feedback_pr_supersede_chain` — 표 UX 패턴 재사용
- ✅ `feedback_push_full_test_required` — cargo test --tests + clippy + fmt
- ✅ `feedback_assign_issue_before_work` — assignee @edwardkim
- ✅ `project_output_folder_structure` — `output/poc/task919/`

## 5. 참고 자료

- `src/document_core/queries/cursor_rect.rs:564~593` (collect_runs)
- `src/document_core/queries/cursor_rect.rs:880~923` (hit_test_native 매칭)
- `src/wasm_api.rs:2036` (`getTableBBox`)
- `rhwp-studio/src/engine/input-handler.ts:1105` (`isTableBorderClick`)
- `rhwp-studio/src/engine/input-handler.ts:1130` (`findTableByOuterClick`)
- `rhwp-studio/src/engine/input-handler-mouse.ts:617~720` (클릭 흐름)
- `rhwp-studio/src/engine/input-handler-keyboard.ts:932~963` (Esc 패턴)
- `samples/table-in-tbox.hwp` (재현 fixture)
