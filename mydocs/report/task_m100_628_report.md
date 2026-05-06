# Task #628 최종 보고서: 글상자 안 이미지 미렌더링 — `inline_shape_positions` 키 충돌 정정

## 결함

`samples/exam_science.hwp` 페이지 4 (20번 문항) 글상자 안 실린더 이미지 (`bin_id=2`, 99.7×26.9mm) 가 SVG 출력에서 누락. 같은 페이지 19번 문항의 동일 형태 이미지 (`bin_id=1`) 는 정상 렌더링.

## 근본 원인

`PageRenderTree.inline_shape_positions` 의 키 `(section, para, control)` 에서 `para` 가 **두 가지 의미로 혼용**:

1. `paragraph_layout` 호출 시 → 섹션 단위 paragraph 인덱스
2. `layout_table` → 셀 paragraph 호출 시 → 셀 내부 paragraph 인덱스 (`cp_idx`)

서로 다른 셀 컨텍스트가 동일 키 namespace 를 공유하여 `(0, 0, 1)` 등의 키가 충돌. 다른 paragraph 의 double-nested 셀 처리가 `(0, 0, 1)` 을 점유 → 20번 외부 1x1 표 처리 시 stale 값을 보고 `already_rendered_inline=true` 오판 → `table_layout.rs:1900` 분기에서 내부 2x3 표의 `layout_table` 재귀 호출이 스킵 → 그 안의 그림 미렌더.

19번(단일 nesting)은 영향 없음, 20번(이중 nesting)만 발현.

## 수정

`inline_shape_positions` 키에 `cell_path` 추가:

```rust
pub type InlineShapeKey = (usize, usize, usize, Vec<(usize, usize, usize)>);
//                          section, para, control, cell_path
```

`cell_path` = 외→내 nesting 순서의 `(control_index, cell_index, cell_para_index)` 튜플 목록. 섹션 단위 호출은 빈 Vec, 셀 단위 호출은 `CellContext.path` 전체를 변환.

`set/get_inline_shape_position` 시그니처에 `cell_ctx: Option<&CellContext>` 추가, 호출처 13곳 일괄 패치:

| 컨텍스트 | 호출 수 | 파일 |
|---|---|---|
| 셀 단위 (`cell_ctx` 전달) | 9 | paragraph_layout.rs (6) + table_layout.rs (2) + table_partial.rs (1) |
| 섹션 단위 (`None` 전달) | 7 | layout.rs (4) + shape_layout.rs (2) + cursor_rect.rs (1) |

`cursor_rect.rs:532` 의 hit-test 루프는 `cell_path.is_empty()` 가드 추가 (셀 내부 inline shape 은 별도 처리, 섹션 단위만 검사).

## 변경 통계

```
 src/document_core/queries/cursor_rect.rs |  6 ++--
 src/renderer/layout.rs                   |  8 ++---
 src/renderer/layout/paragraph_layout.rs  | 12 +++----
 src/renderer/layout/shape_layout.rs      |  4 +--
 src/renderer/layout/table_layout.rs      |  4 +--
 src/renderer/layout/table_partial.rs     |  2 +-
 src/renderer/render_tree.rs              | 54 ++++++++++++++++++++++++-------
 7 files changed, 62 insertions(+), 28 deletions(-)
```

## 검증

### 타겟 결함 해결

| 페이지 | 이전 | 수정 후 |
|---|---|---|
| exam_science page 4 | 3 images | **4 images** ✓ |

20번 이미지 위치 검증:
- `x=568, y=783.92` (외부 1x1 표 영역 y=770.71~1071.91 내부)
- `width=376.65 height=101.81` = 99.7×26.9mm (HWPUNIT IR 정확 매칭)

### 회귀 sweep (5 샘플 56 페이지)

| 샘플 | 페이지 | 이미지 수 | SVG diff |
|---|---|---|---|
| exam_eng | 8 | 14 | byte-identical |
| exam_math | 20 | 9 | byte-identical |
| exam_kor | 20 | 49 | byte-identical |
| exam_social | 4 | 7 | byte-identical |
| exam_science page 1-3 | 3 | 16 | byte-identical |
| exam_science page 4 | 1 | **3 → 4** | 의도된 +1 |

회귀 0건.

### 단위 테스트

```
cargo test --release: 1134+ passed, 0 failed
```

### Clippy

신규 경고 0. 사전 존재 경고 2건 (`table_ops.rs:1007`, `object_ops.rs:298` 의 panic-on-unwrap) 은 base branch 에서 동일 발생, 본 변경 영향 없음.

## 회귀 위험 평가

- **낮음** — 키 namespace 분리만 수행, 값/계산 로직 무변경
- 섹션 단위 호출은 기존 `(sec, para, ctrl, [])` 와 동등 → 기존 동작 유지
- 셀 단위 호출은 stale-key 충돌 차단 → 의도된 변화만 (20번 이미지 +1)
- 5 샘플 56 페이지 byte-identical 결과로 잠재 회귀 영역 부재 확인

## 브랜치 / PR

- base: `upstream/devel` (`9b490634`)
- task: `local/task628`
- PR: 등록 예정 (`pr-task628`, base=devel)
