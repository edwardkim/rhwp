# 단계별 완료 보고서 — Task M100-1161 Stage 3

## 목표

ImageNode 가 **전체 다단계 cellPath** 를 보유·방출하도록 하여, 중첩 표/글상자 안 picture 의
복사·선택 기반을 마련(#1171 공유 기반). 계약 (A) cellPath 단일 진실원 / (B) 공유 채움 / (C) hit-test 불간섭.

## 변경 사항

### 1. ImageNode 필드 추가 — `src/renderer/render_tree.rs`
- `cell_context: Option<CellContext>` 추가(TextRunNode 정합, render_tree 는 이미 `use super::layout::CellContext`).
  기존 단일 레벨 스칼라(cell_index/cell_para_index/outer_table_control_index)는 **유지**(innermost 투영, 계약 A).
- `ImageNode::new` 에 `cell_context: None` 기본값.

### 2. 셀 picture 생성 site 3곳에 `cell_context` 채움 (계약 B)
조사 결과 셀 picture 는 한 곳이 아닌 3개 경로로 생성됨(각자 `cell_ctx` 보유):
- **`layout_picture_full`** (`picture_footnote.rs:64`) — **실제 활성 경로**(Task #1151 v4 셀 picture 렌더러). `cell_context: cell_ctx.cloned()`.
- `make_picture_image_node` (`paragraph_layout.rs:4163`) — inline TAC 경로. `cell_context: cell_ctx.cloned()`.
- `table_cell_content.rs` full 리터럴 — `cell_ctx` 를 먼저 구성해 ImageNode 와 inline_shape_position 등록에 공유.

> 디버그 계측으로 활성 경로가 `layout_picture_full` 임을 확정(make_picture_image_node/table_cell_content 는
> 본 샘플 페이지에서 미발화). 3곳 모두 채워 경로 누락 방지.

### 3. rendering.rs cellPath 방출 — `src/document_core/queries/rendering.rs`
- ImageNode 방출부에 `parentParaIdx` + `cellPath:[{controlIndex,cellIndex,cellParaIndex},...]` 추가
  (TextRun cell_coords 방출과 동일 포맷). 스칼라(cellIdx/cellParaIdx/outerTableControlIdx)도 계속 방출(하위호환).

### 4. 회귀 테스트 — `tests/issue_1161_image_cellpath.rs`
- 전 페이지 스캔 → image 컨트롤에 **2-엔트리 이상 cellPath** + parentParaIdx 존재 단언.
  `pic-in-table-01.hwp` p16(외부표→셀5→내부표 3×9→셀1/3/5/7 picture)에서 검증.

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 1471 passed, 0 failed |
| `cargo test --tests` (전 통합) | ✅ 0 failed |
| `cargo test --test issue_1161_image_cellpath` | ✅ 1 passed (2-엔트리 cellPath 확인) |
| `cargo test --test issue_1161_copy_picture_in_cell` | ✅ 4 passed |
| `cargo fmt --check`(변경 파일) | ✅ |
| `cargo clippy --lib` + 새 테스트 | ✅ 0 warning |

**렌더 픽셀 무변경**: cell_context 는 메타데이터 추가일 뿐 좌표/렌더 로직 무변경. 전 렌더러 테스트(skia/canvaskit/svg) 통과.

## #1171 영향

- additive(새 필드 + 새 방출, 스칼라 불변, 3 site 모두 cell_ctx 기준) → 회귀 위험 낮음.
- `resolve_paragraph_by_path` + cellPath 가 표 셀·글상자 모두 표현 가능 → #1171(사각형→글상자 picture)도 이 기반 소비 가능.

## 다음 단계

Stage 4 — TS: `findPictureAtClick` 가 layout image 의 cellPath 를 읽어 선택 ref 에 보유,
복사/오려두기/컨텍스트 메뉴 전 지점이 cellPathJson 전달, 타입 보정.
