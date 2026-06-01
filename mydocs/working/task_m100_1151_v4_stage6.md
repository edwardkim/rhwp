# Task #1151 v4 Stage 6 완료 보고서 — ImageNode cell context + JSON 직렬화 + cursor_rect hit-test

수행계획서: [task_m100_1151_v4.md](../plans/task_m100_1151_v4.md) · 구현계획서: [task_m100_1151_v4_impl.md](../plans/task_m100_1151_v4_impl.md)

## 1. 진단 보강 (Stage 6-1)

Explore 보고에서 "ImageNode 가 cell_index 필드를 가짐" 이라 했으나 실제 코드 검증 결과 **ImageNode struct 에 cell_index 필드 자체가 없음** (Rectangle/Ellipse/Path 만 [Task #1138] 패턴으로 추가). 따라서 v4 fix 범위가 plan 보다 한 층 더 깊음 — struct 정의도 같이 추가해야 함.

확정된 v4 fix scope (4-layer):

1. `render_tree.rs` 의 ImageNode struct 에 `cell_index` / `cell_para_index` / `outer_table_control_index` 필드 추가 + `ImageNode::new()` default 값.
2. `rendering.rs:1495` Image JSON 직렬화에 `cellIdx`/`cellParaIdx` 추가.
3. `paragraph_layout.rs` 의 3 곳 ImageNode 생성에 cell_index 설정 (cell_ctx 가 None 인 경우 회귀 0).
4. `table_cell_content.rs:731` 의 ImageNode 생성 (핵심 진입점) 에 section/para/cell_index/cell_para_index/outer_table_control_index 명시 설정.
5. `cursor_rect.rs:1218-1223` 의 셀 안 inline shape 분기 — cell_path 가 있어도 hit-test 진입 + 응답에 `cellPath`/`innerControlIdx` 포함.

## 2. 변경 내용

### `src/renderer/render_tree.rs` — ImageNode struct 확장

```rust
pub struct ImageNode {
    // ... 기존 필드
    /// [Task #1151 v4] 표 셀 안 inline picture
    #[serde(default)]
    pub cell_index: Option<usize>,
    #[serde(default)]
    pub cell_para_index: Option<usize>,
    #[serde(default)]
    pub outer_table_control_index: Option<usize>,
}
```

`ImageNode::new(...)` 의 default 값으로 세 필드 모두 None 설정.

### `src/document_core/queries/rendering.rs:1495` — Image JSON 에 cell_str 추가

Rectangle (라인 1524) 의 패턴 그대로 차용:

```rust
let cell_str = match (image_node.cell_index, image_node.cell_para_index) {
    (Some(cei), Some(cpi)) => format!(",\"cellIdx\":{},\"cellParaIdx\":{}", cei, cpi),
    _ => String::new(),
};
controls.push(format!(
    "{{\"type\":\"image\",\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}{}{}{}{}}}",
    ..., cell_str
));
```

### `src/renderer/layout/paragraph_layout.rs` 3 곳 — ImageNode 생성에 cell_index

각 3 위치 (line 2688, 3067, 3215 근방) 에 `cell_ctx.as_ref().and_then(|c| c.path.last().map(|e| e.cell_index))` 패턴.

### `src/renderer/layout/table_cell_content.rs:731` — 핵심 진입점

`enclosing_ctx` 에서 outer 정보 추출 + 셀 정보 명시 설정:

```rust
RenderNodeType::Image(ImageNode {
    section_index: enclosing_ctx.map(|(s, _, _, _)| s),
    para_index: enclosing_ctx.map(|(_, p, _, _)| p),
    control_index: Some(ctrl_idx),  // 셀 안 paragraph 의 picture index
    cell_index: Some(cell_idx),
    cell_para_index: Some(pidx),
    outer_table_control_index: enclosing_ctx.map(|(_, _, _, table_ci)| table_ci),
    // ... 기존 필드
}),
```

이전: `section_index: None, para_index: None` → studio 가 picture 위치 식별 불가.

### `src/document_core/queries/cursor_rect.rs:1218-1223` — 셀 안 picture hit-test

기존 `if !cell_path.is_empty() { continue; }` 제거 → `target_para` 분기:

```rust
let target_para = if cell_path.is_empty() {
    section.paragraphs.get(pi)
} else {
    let last = cell_path.last().copied().unwrap_or((0, 0, 0));
    section.paragraphs.get(pi)
        .and_then(|p| p.controls.get(last.0))
        .and_then(|c| match c {
            Control::Table(t) => t.cells.get(last.1),
            _ => None,
        })
        .and_then(|cell| cell.paragraphs.get(last.2))
};
if let Some(para) = target_para {
    if let Some(ctrl) = para.controls.get(ci) {
        // 기존 hit-test 로직 그대로 (Picture/Shape bbox 검사)
    }
}
```

응답 형식 (라인 1328 근방) 에 cell_path 가 있으면 `cellPath` + `innerControlIdx` 추가:

```rust
let cell_path_str = if cell_path.is_empty() {
    String::new()
} else {
    let entries: Vec<String> = cell_path.iter()
        .map(|(t, c, p)| format!("[{},{},{}]", t, c, p))
        .collect();
    format!(",\"cellPath\":[{}],\"innerControlIdx\":{}", entries.join(","), ci)
};
return Ok(format!(
    "{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{},\"cursorRect\":{{...}}{}}}",
    si, pi, offset, page_num, sx, sy, sh, cell_path_str
));
```

## 3. 회귀 안전성

- ImageNode 신규 필드는 `#[serde(default)]` + Option<usize> → 기존 JSON 호환.
- `paragraph_layout.rs` 의 3 곳 cell_index 설정은 cell_ctx 가 None 인 본문 경로에서 None → 기존 동작 보존.
- `table_cell_content.rs:731` 변경은 셀 안 inline picture 만 영향 (셀 안 picture path 의 outer 정보 보강).
- `cursor_rect.rs` 의 cell_path 분기는 셀 외부 경로에서 기존 동작 그대로, 셀 안 경로만 신규 활성화.

## 4. 자동 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build --lib` | 성공 (10.75s) |
| `cargo test --lib` 전수 | **1442 passed, 0 failed, 6 ignored** (회귀 0) ✓ |
| `cargo clippy --lib -- -D warnings` | clean ✓ |
| `cargo fmt --all -- --check` | clean ✓ |
| v2 통합 테스트 (model 정합 4) | PASS 유지 |
| v3 helper 단위 테스트 (6) | PASS 유지 |
| tac-img-02.hwp SVG export | 66 페이지 정상 출력 (시각 layer 이상 무) |

## 5. Stage 7 진입 조건

- ImageNode struct + 4 곳 ImageNode 생성 + JSON 직렬화 + hit-test 분기 모두 갱신 ✓
- 회귀 0 (1442 PASS) ✓
- clippy/fmt clean ✓
- v1/v2/v3 검증 결과 유지 ✓

→ Stage 7 (WASM 재빌드 + dev server 재시연 + tac-img-02.hwp 클릭 시연) 진행 가능.

## 6. Stage 7 시연 절차

1. `docker compose --env-file .env.docker run --rm wasm` (WASM 재빌드)
2. `cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700 &` (dev server)
3. tac-img-02.hwp 열기 → 표 안 inline picture 위 클릭 → picture select 모드 진입 / cursor 이동 정상 확인.
4. picture select 후 우클릭 → 개체 속성 → 그림 속성 대화상자 cellPath 정상 전달.
5. v3 회귀 4 시나리오 시각 + v1 의 셀 floating 삽입 + 본문 inline picture 회귀.
