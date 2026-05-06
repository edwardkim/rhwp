# Task #628: 구현계획서

## 키 타입 변경

```rust
// 기존
inline_shape_positions: HashMap<(usize, usize, usize), (f64, f64)>

// 신규
pub type InlineShapeKey = (usize, usize, usize, Vec<(usize, usize, usize)>);
//                          section, para, control, cell_path
inline_shape_positions: HashMap<InlineShapeKey, (f64, f64)>
```

`cell_path` 는 외→내 nesting 순서로 `(control_index, cell_index, cell_para_index)` 튜플 목록. 섹션 단위 호출은 빈 Vec.

## API 변경

```rust
pub fn set_inline_shape_position(
    &mut self,
    sec: usize, para: usize, ctrl: usize,
    cell_ctx: Option<&CellContext>,    // ← 신규
    x: f64, y: f64,
)

pub fn get_inline_shape_position(
    &self,
    sec: usize, para: usize, ctrl: usize,
    cell_ctx: Option<&CellContext>,    // ← 신규
) -> Option<(f64, f64)>
```

내부에서 `cell_ctx.path` → `Vec<(ctrl_idx, cell_idx, cell_para_idx)>` 로 변환하여 키 구성.

## 호출처 패치 (13곳)

### 셀 컨텍스트 (`cell_context` / `cell_ctx` 전달)

| 파일 | 라인 | 호출 |
|---|---|---|
| `paragraph_layout.rs` | 1849 | set Shape inline (run_tacs 경로) |
| `paragraph_layout.rs` | 1906 | set Equation inline (run_tacs 경로) |
| `paragraph_layout.rs` | 1926 | set Table inline (run_tacs 경로) |
| `paragraph_layout.rs` | 2183 | set Shape inline (empty-runs 경로) |
| `paragraph_layout.rs` | 2225 | set Picture inline (empty-runs 경로) |
| `paragraph_layout.rs` | 2342 | set Equation inline (Task #287 분기) |
| `table_layout.rs` | 1831 | get Equation dedup |
| `table_layout.rs` | 1897 | get Table dedup ← **본 결함 진입점** |
| `table_partial.rs` | 768 | get Equation dedup (분할 표 경로) |

### 섹션 단위 (`None` 전달)

| 파일 | 라인 | 호출 |
|---|---|---|
| `layout.rs` | 2292 | get Table inline (paginator) |
| `layout.rs` | 2538 | get Table inline (paragraph 흐름) |
| `layout.rs` | 2825 | get Picture dedup |
| `layout.rs` | 2877 | set Picture inline |
| `shape_layout.rs` | 135 | get Equation dedup |
| `shape_layout.rs` | 219 | get Shape inline |
| `cursor_rect.rs` | 180 | get hit-test |

### hit-test 루프 (`cursor_rect.rs:532`)

`inline_shape_positions()` iterate 시 `cell_path.is_empty()` 가드 추가 — 셀 내부 inline shape 은 hit-test 에서 별외 (섹션 단위만 검사).

## 검증

1. exam_science page 4 SVG 재생성 → 이미지 4개 확인
2. exam_eng/math/kor/social 4개 샘플 sweep → byte-identical
3. `cargo test --release` 전체 통과
4. clippy 신규 경고 없음

## 회귀 위험 평가

- **낮음**: 키 namespace 분리만 수행 (값/계산 로직 무변경)
- 섹션 단위 호출(`None` 전달)은 기존 `(sec, para, ctrl, [])` 와 동등 → 기존 동작 유지
- 셀 단위 호출(`Some(ctx)` 전달)은 기존 stale-key 충돌 차단 → 의도된 변화만
