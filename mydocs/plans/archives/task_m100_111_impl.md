# 구현 계획서 — Task #111 + #112

**이슈**: [#111](https://github.com/edwardkim/rhwp/issues/111) + [#112](https://github.com/edwardkim/rhwp/issues/112)
**타이틀**: 양식 컨트롤 인터랙션 — 셀 커서 진입 + 체크박스 클릭 토글
**마일스톤**: M100
**작성일**: 2026-04-13
**브랜치**: `local/task111`

---

## 수정 대상

| 파일 | 단계 |
|------|------|
| `src/document_core/queries/cursor_rect.rs` | 1단계 |
| (검증만) `rhwp-studio/src/engine/input-handler-mouse.ts` | 2단계 |

---

## 구현 단계

### 1단계: #111 — 셀 커서 진입 (`cursor_rect.rs`)

#### 원인 요약

`collect_runs()` (line 335)이 `RenderNodeType::FormObject` 노드를 무시한다.
FormObject만 있는 셀은 `cell_bboxes`에 등록되지만,
`parent_para_index`/`control_index` 보완 로직 (line 452-462)이 TextRun(`runs`)에서만 값을 찾으므로
보완되지 않은 채 0으로 남는다.

`hit_test_native()` (line 589-638):
```rust
let clicked_cell: Option<&CellBboxInfo> = cell_bboxes.iter()
    .find(|cb| x >= cb.x && x <= cb.x + cb.w && y >= cb.y && y <= cb.y + cb.h);
```
`clicked_cell`은 찾아지지만:
```rust
let cell_runs: Vec<&RunInfo> = runs.iter()
    .filter(|r| r.cell_context.as_ref().map(|ctx| {
        ctx.parent_para_index == cb.parent_para_index // 0 vs 실제값
            && ctx.path[0].control_index == cb.control_index // 0 vs 실제값
    }).unwrap_or(false))
    .collect();
// → cell_runs is always empty for form-only cells
```
`cell_runs`가 비어서 이 분기가 실패하고 셀 외부 hitTest로 넘어간다.

#### 수정 방향

**접근 1**: `collect_runs()`에서 `FormObject` 노드도 처리하여 `cell_context` 정보를 추출한다.

`FormObjectNode`에는 `section_index`, `para_index`, `control_index` (line 233-238 of render_tree.rs)가 있다.
그러나 `cell_context`(어느 셀에 속하는지)는 FormObject 노드에 없고, 부모 `TableCell` 노드에서만 알 수 있다.

**접근 2**: `cell_bboxes` 보완 로직을 FormObject에서도 수행한다.

`collect_runs()`에서 FormObject 노드를 만날 때, 부모 TableCell의 인덱스와 연결하기 위해
`cell_context`를 FormObjectNode에서도 추출하거나, 별도 `FormCellInfo` 구조를 도입한다.

**채택 방향**: `collect_runs()`에 `FormObject` 분기 추가 + `cell_bboxes` 보완에 FormObject 정보 활용

#### 구체적 수정 내용

##### 1-1. `collect_runs()`에 FormObject 분기 추가

`FormObjectNode`가 TableCell 내에 있을 때, 해당 셀의 `cell_bboxes` 보완에 쓸 정보를 담은
`form_cell_infos`를 수집한다.

```rust
// hit_test_native() 내부에 추가할 구조체
struct FormCellInfo {
    section_index: usize,
    para_index: usize,    // FormObjectNode.para_index
    control_index: usize, // FormObjectNode.control_index
    cell_index: usize,    // 부모 TableCell의 model_cell_index
}
```

`collect_runs()` 함수 시그니처 변경:
```rust
fn collect_runs(
    node: &RenderNode,
    runs: &mut Vec<RunInfo>,
    guide_runs: &mut Vec<GuideRunInfo>,
    cell_bboxes: &mut Vec<CellBboxInfo>,
    form_cell_infos: &mut Vec<FormCellInfo>,  // 추가
    current_column: Option<u16>,
    current_cell_index: Option<usize>,        // 추가: 현재 순회 중인 TableCell 인덱스
)
```

FormObject 분기:
```rust
if let RenderNodeType::FormObject(ref fo) = node.node_type {
    if let Some(cell_idx) = current_cell_index {
        form_cell_infos.push(FormCellInfo {
            section_index: fo.section_index,
            para_index: fo.para_index,
            control_index: fo.control_index,
            cell_index: cell_idx,
        });
    }
}
```

TableCell 진입 시 `current_cell_index` 전파:
```rust
let cell_idx = if let RenderNodeType::TableCell(ref tc) = node.node_type {
    tc.model_cell_index.map(|i| i as usize)
} else {
    current_cell_index
};
// 재귀 호출 시 cell_idx 전달
for child in &node.children {
    collect_runs(child, runs, guide_runs, cell_bboxes, form_cell_infos, col, cell_idx);
}
```

##### 1-2. `cell_bboxes` 보완 로직 확장

기존 runs 기반 보완 (line 452-462) 이후, `form_cell_infos`로도 보완:

```rust
// runs로 보완되지 않은 cell_bboxes를 form_cell_infos로 보완
for cb in &mut cell_bboxes {
    if cb.parent_para_index == 0 && cb.control_index == 0 {
        if let Some(fi) = form_cell_infos.iter().find(|fi| fi.cell_index == cb.cell_index) {
            // FormObject의 para_index는 셀 내 문단 인덱스가 아니라
            // 표를 포함하는 부모 문단 인덱스이므로 직접 사용 불가.
            // 대신 form_query.rs의 find_form_node_at()와 동일한 방식으로
            // cell_bboxes를 찾아 parent_para_index를 추론한다.
            // → 아래 대안 채택
        }
    }
}
```

> **주의**: `FormObjectNode.para_index`는 셀 내 단락이 아닌 **표를 포함한 상위 문단**의 인덱스다.
> 따라서 `cb.parent_para_index`에 직접 대입하면 충돌이 없다.
> `control_index`는 해당 상위 문단 내 표 컨트롤 인덱스이므로 이 역시 직접 사용 가능하다.

실제로 `FormObjectNode`가 렌더링될 때의 `para_index`/`control_index`가 **표를 담은 문단**의 값인지 확인 필요 → render_tree 생성 코드에서 확인.

##### 1-3. 빈 `cell_runs`인 경우 fallback 추가

`cell_runs`가 비어도 셀 안의 FormObject 위치로 커서를 보낼 수 있도록 fallback 추가:

```rust
if cell_runs.is_empty() {
    // FormObject만 있는 셀: runs에서 같은 표/셀 소속 runs를 찾되 없으면
    // 셀 자체의 문단 시작 위치(charOffset=0)를 반환
    if let Some(cb) = clicked_cell {
        if cb.parent_para_index > 0 || cb.control_index > 0 {
            // parent_para_index가 보완된 경우: charOffset=0으로 진입
            return Ok(format!(
                "{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":0,\
                 \"cellPath\":[{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":0}}],\
                 \"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                section_index, cb.parent_para_index,
                cb.control_index, cb.cell_index,
                page_num,
                cb.x + 2.0, cb.y + 2.0, cb.h - 4.0
            ));
        }
    }
}
```

> `section_index`는 `runs`가 비어있지 않으면 다른 runs에서, 비어있으면 `find_page()`에서 얻는다.

---

### 2단계: #112 — 체크박스 클릭 토글 검증

#### 현황 분석

`input-handler-mouse.ts` line 697-705:
```typescript
const formHit = this.wasm.getFormObjectAt(pageIdx, pageX, pageY);
if (formHit.found) {
    this.handleFormObjectClick(formHit, pageIdx, zoom);
    this.textarea.focus();
    return;  // ← hitTest 전에 먼저 처리
}
```

`getFormObjectAt()`은 `form_query.rs`의 `find_form_node_at()` (recursive, children-first)을 사용하며,
이는 `hit_test_native()`와 **독립적**으로 `FormObject` bbox collision을 직접 검사한다.

따라서 `hit_test_native()`(#111)가 수정되지 않아도 `getFormObjectAt()`은 정상 동작해야 한다.

#### 검증 항목

1. **form-002.hwpx 로드 후** 체크박스 클릭 시 `getFormObjectAt()` 반환값 확인 (브라우저 콘솔)
2. `handleFormObjectClick()` (input-handler.ts line 2377-2382) CheckBox 분기:
   ```typescript
   case 'CheckBox': {
       const newValue = formHit.value === 0 ? 1 : 0;
       this.wasm.setFormValue(formHit.sec, formHit.para, formHit.ci, newValue);
       this.afterEdit();
       break;
   }
   ```
   이 로직이 이미 올바르게 구현되어 있으므로 **수정 불필요** 예상.

3. `setFormValue()` → `set_form_value_native()` → `recompose_section()` 호출 후 재렌더링 되는지 확인.

#### 수정 필요 시

만약 `getFormObjectAt()`이 null/found=false를 반환한다면:
- `find_form_node_at()`의 bbox 좌표 계산 문제 조사
- WASM 바인딩에서 좌표 변환 오류 확인

---

## 검증 계획

### 1단계 검증

```bash
# 빌드
cargo build --release --bin rhwp

# SVG 내보내기로 렌더 구조 확인 (선택)
cargo run --release --bin rhwp -- export-svg samples/hwpx/form-002.hwpx -p 0

# 회귀 테스트
cargo test
```

WASM 빌드 후 브라우저에서:
- `form-002.hwpx` 로드
- 체크박스 셀 클릭 → 커서가 셀 안으로 진입하는지 확인

### 2단계 검증

브라우저에서:
- 체크박스 클릭 → 체크 상태 토글 확인 (□ ↔ ☑)
- 토글 후 페이지 재렌더링 확인

---

## 예상 diff 규모

- `cursor_rect.rs`: ~40줄 (구조체 1개 + `collect_runs` 시그니처 변경 + FormObject 분기 + 보완 로직 + fallback)
- `input-handler-mouse.ts`: 수정 없음 예상

---

## 승인 요청

위 구현 계획서를 검토 후 승인해주시면 1단계 구현을 시작하겠습니다.
