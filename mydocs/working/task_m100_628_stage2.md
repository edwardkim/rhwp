# Task #628 Stage 2: 수정 + 검증

## 수정 내용

### `src/renderer/render_tree.rs`

키 타입 신설:

```rust
pub type InlineShapeKey = (usize, usize, usize, Vec<(usize, usize, usize)>);
//                          section, para, control, cell_path
```

`cell_path` 는 외→내 nesting 순서로 `(control_index, cell_index, cell_para_index)` 튜플 목록. 섹션 단위는 빈 Vec.

`set/get_inline_shape_position` 시그니처에 `cell_ctx: Option<&CellContext>` 추가, 내부에서 `cell_ctx.path` → `cell_path` 변환:

```rust
fn cell_path_from_ctx(cell_ctx: Option<&CellContext>) -> Vec<(usize, usize, usize)> {
    cell_ctx.map(|ctx| {
        ctx.path.iter()
            .map(|e| (e.control_index, e.cell_index, e.cell_para_index))
            .collect()
    }).unwrap_or_default()
}
```

### 호출처 패치 (13곳)

**셀 컨텍스트 (`cell_context` / `cell_ctx` 전달, 9곳)**:

- `paragraph_layout.rs:1849, 1906, 1926, 2183, 2225, 2342` (set 6곳, run_tacs / empty-runs / Task #287 분기)
- `table_layout.rs:1831, 1897` (get 2곳, dedup 체크)
- `table_partial.rs:768` (get 1곳, 분할 표 경로)

**섹션 단위 (`None` 전달, 4곳)**:

- `layout.rs:2292, 2538, 2825, 2877` (paginator + 본문 흐름)
- `shape_layout.rs:135, 219` (Equation/Shape dedup)
- `cursor_rect.rs:180` (hit-test 단건 조회)

**hit-test 루프 (`cursor_rect.rs:532`)**:

```rust
for (key, &(sx, sy)) in tree.inline_shape_positions() {
    let (si, pi, ci, ref cell_path) = *key;
    // 셀 내부 inline shape 은 cursor hit-test 에서 별도 처리 — 섹션 단위만 검사
    if !cell_path.is_empty() { continue; }
    ...
}
```

## 검증

### 타겟 결함 해결

`./target/release/rhwp export-svg samples/exam_science.hwp -p 3` (page 4):

```
수정 전: 3 images
<image x=279.68 y=1003.39 width=57.28  ...>
<image x=426.59 y=1003.39 width=59.52  ...>
<image x=560.45 y=415.48  width=377.33 ...>  ← 19번

수정 후: 4 images
<image x=279.68 y=1003.39 width=57.28  ...>
<image x=426.59 y=1003.39 width=59.52  ...>
<image x=560.45 y=415.48  width=377.33 height=78.29  ...>   ← 19번
<image x=568.00 y=783.92  width=376.65 height=101.81 ...>   ← 20번 ✓ NEW
```

20번 이미지 위치 검증:
- y=783.92 → 외부 1x1 표 영역 (y=770.71 ~ 1071.91) 내부 ✓
- height=101.81 px = 26.9 mm × 96/25.4 ✓
- width=376.65 px ≈ 99.7 mm × 96/25.4 ≈ 376.95 ✓

### 회귀 sweep (4 시험지 샘플)

수정 전후 SVG 직접 비교:

```
exam_eng:    8p 14imgs  diff_files=0   (byte-identical)
exam_math:  20p  9imgs  diff_files=0   (byte-identical)
exam_kor:   20p 49imgs  diff_files=0   (byte-identical)
exam_social: 4p  7imgs  diff_files=0   (byte-identical)
```

### exam_science 페이지 1-3

byte-identical (page 4 만 의도된 변화).

### 단위 테스트

```
$ cargo test --release --lib
test result: ok. 1134 passed; 0 failed; 2 ignored
```

전체 cargo test (workspace):
```
test result: ok. 1134 passed; ...
test result: ok. 14 passed; ...
test result: ok. 25 passed; ...
... (다수)
```

모두 GREEN, failure 0.

### Clippy

```
$ cargo clippy --release --lib
error: this call to `unwrap()` will always panic
    --> src/document_core/commands/table_ops.rs:1007
error: this call to `unwrap()` will always panic
    --> src/document_core/commands/object_ops.rs:298
```

**사전 존재 경고 2건** (base branch `upstream/devel` 에서 동일 발생, `git stash` + clippy 재실행으로 확인). 본 변경은 신규 경고 0.

## 회귀 위험 평가

- **낮음** — 키 namespace 분리만 수행, 값/계산 로직 무변경
- 섹션 단위 호출(`None` 전달)은 기존 `(sec, para, ctrl, [])` 와 동등 → 기존 동작 유지
- 셀 단위 호출(`Some(ctx)` 전달)은 기존 stale-key 충돌 차단 → 의도된 변화만 (20번 이미지 +1)
- 4 샘플 byte-identical 결과로 잠재 회귀 영역 부재 확인

## 다음 단계

최종 보고서 작성 → 브랜치 push → PR 등록.
