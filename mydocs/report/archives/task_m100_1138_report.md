# Task #1138 최종 결과 보고서 — 표 셀 내부 도형 '개체 속성' 다이얼로그 지원

- 이슈: [#1138](https://github.com/edwardkim/rhwp/issues/1138)
- 수행/구현 계획서: [task_m100_1138.md](../plans/task_m100_1138.md) (v2, 5-layer)
- 브랜치: `local/task1138` (base: `local/devel`)
- 마일스톤: v1.0.0
- 참조: [#825](https://github.com/edwardkim/rhwp/issues/825) (PR #832, 머리말/꼬리말 평행 케이스)

## 1. 요약

rhwp-studio 표 셀 내부의 **도형(shape — 사각형/타원/직선/다각형/곡선)** 객체에 대해 우클릭 → "개체 속성(P)..." 클릭 시 다이얼로그가 미표시되던 결함을 정정. PR #832 패턴 (머리말/꼬리말 그림) 을 표 셀 내부 도형에 확장 적용.

**사용자 검증 결과**:
- ✅ `inner-table-01.hwp` 표 셀[5] 사각형(도형) → 개체 속성 다이얼로그 정상 표시
- ✅ UI 속성 변경 동작 정상
- ✅ 표 밖 일반 그림/도형 정상 동작 (회귀 없음)
- ✅ 콘솔 에러 사라짐

**Picture path 분석 결과**: 셀 안 picture (image) 는 `paragraph_layout` path 가 이미 처리하여 외부 좌표로 정상 동작 — 본 PR 의 picture 관련 변경은 검증을 통해 **불필요**가 확정되어 정리됨. PR scope 가 도형(shape) 으로 한정됨.

## 2. 변경 내역

### 2.1 commits (10개, `local/devel` 위)

| Hash | 작업 |
|------|------|
| `5e6209c4` | Plan v2 (5-layer) + orders |
| `82cd36bb` | Stage 1.2a (cherry-pick): layout_shape_object cell ctx + Rectangle propagate |
| `6697ffb4` | Stage 1.2b (cherry-pick): Line/Ellipse/Path 7곳 cell info |
| `953cd8a0` | Stage 2.1 (cherry-pick): rendering.rs export 5 노드 cell info |
| `ba61f602` | Stage 1 완료: layout_picture_full cell ctx + 셀 안 picture 사이트 |
| `94e77095` | Stage 1 보고서 |
| `e76465f5` | Stage 2 picture by_path (cherry-pick) |
| `4fdc6f5d` | Stage 2 shape helper + by_path + tests (8 passed) |
| `d1a21d24` | Stage 3: TS bridge + dialog + handler |
| (본 commit) | Stage 4: 최종 보고서 + orders 최종 갱신 |

### 2.2 변경 파일 (Rust)

| 파일 | 변경 |
|------|------|
| `src/renderer/render_tree.rs` | Rectangle/Line/Ellipse/Path 4 노드에 `cell_index` / `cell_para_index` / `outer_table_control_index` 필드 신규 |
| `src/renderer/layout/shape_layout.rs` | `layout_shape_object` 시그니처에 `table_cell_ref` 매개변수 추가, 4 노드 생성 사이트 cell 정보 채움 |
| `src/renderer/layout/table_cell_content.rs` | `layout_cell_shape` 시그니처 확장 |
| `src/renderer/layout/table_layout.rs` | layout_cell_shape 2 호출자에 셀 컨텍스트 전달 |
| `src/renderer/layout/table_partial.rs` | 분할 표 layout_cell_shape 2 호출자에 셀 컨텍스트 전달 |
| `src/document_core/queries/rendering.rs` | Rectangle/Line/Ellipse/Path export JSON 에 `cellIdx`/`cellParaIdx`/`outerTableControlIdx` 추가 |
| `src/document_core/commands/object_ops.rs` | `format_shape_props_inner` + `apply_shape_props_inner` helper 신규 + `get/set_cell_shape_properties_by_path_native` 신규 |
| `src/wasm_api.rs` | `getCellShapePropertiesByPath` / `setCellShapePropertiesByPath` binding 신규 |
| `tests/issue_1138.rs` | 단위 테스트 7개 신규 |

### 2.3 변경 파일 (TypeScript)

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/core/types.ts` | `CellPathSegment` / `CellPath` 타입 신규 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `getCellShapePropertiesByPath` / `setCellShapePropertiesByPath` 메서드 신규 |
| `rhwp-studio/src/ui/picture-props-dialog.ts` | `open()` 시그니처에 `cellPath?` / `innerControlIdx?` 추가, getter/setter 3-way 분기 |
| `rhwp-studio/src/engine/input-handler-picture.ts` | `findPictureAtClick` 반환 타입에 `outerTableControlIdx` 추가, picHit propagate |
| `rhwp-studio/src/engine/cursor.ts` | `selectedPictureRef` 타입 + `enterPictureObjectSelectionDirect` 시그니처 확장 |
| `rhwp-studio/src/engine/input-handler-mouse.ts` | enterPictureObjectSelectionDirect 호출자 2곳에 outerTableControlIdx 전달 |
| `rhwp-studio/src/command/commands/insert.ts` | handler 에서 cellPath 구성 + dialog 에 전달 |

## 3. 아키텍처

### 3.1 path JSON 스키마

```json
[{"controlIdx": N, "cellIdx": M, "cellParaIdx": P}, ...]
```

- 마지막 segment 로 셀 도달 (기존 `resolve_cell_by_path` 호환)
- `inner_control_idx` 매개변수가 셀 paragraph 내 shape control idx
- nested table 자동 확장 (path 배열 길이 늘림 — 본 task 1-level 만 검증)

### 3.2 dialog 3-way 분기 우선순위

`cellPath` (표 셀 내) > `headerFooter` (머리말/꼬리말) > 외부 (기존)

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | **1405 passed, 0 failed, 6 ignored** (회귀 0) |
| `cargo test --release --test issue_1138` | **7 passed, 0 failed** (picture API 제거 후 type_mismatch_picture_api 테스트 제거) |
| `cargo test --release` (integration + svg snapshot 포함) | **모두 통과** (회귀 0) |
| `cargo clippy --release --lib -- -D warnings` | 경고 0 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 (wasm-opt 완료) |
| `npx tsc --noEmit` | 우리 변경 부분 에러 0 (기존 canvaskit-wasm 등 환경 이슈 무관) |
| **E2E (사용자 검증)** | **inner-table-01.hwp 셀[5] 사각형 → 개체 속성 다이얼로그 정상 표시, UI 속성 변경 동작 정상** |
| 회귀: 표 밖 그림/도형 | **정상 동작 유지** |
| 콘솔 에러 | **사라짐** (이전 `[CommandDispatcher] ... 지정된 컨트롤이 Shape이 아닙니다` + `[CursorState] updateRect 실패 → 표 컨트롤이 아닙니다`) |

## 5. 영향 / 호환성

- HWPX/HWP5/HWP3 모두 동일 `Document` IR → 한 번의 변경으로 3개 포맷 자동 지원
- HWP3 파서 디렉토리 (`src/parser/hwp3/`) 무수정 (CLAUDE.md 규칙)
- 기존 API 시그니처 무변경 (신규 함수만 추가) → 호환성 100%
- 머리말/꼬리말 그림 path (별도 5-tuple API) 와 독립 — 영향 없음

## 6. Picture path 분석 (불필요 확정 + 제거)

본 task 작업 중 picture 도 동일 패턴으로 코드 변경 (~150줄) 했으나, **handler 에서 picture 만 cellPath 우회 실험** (`ref.type === 'shape' || ref.type === 'line'` 조건) 결과 picture 다이얼로그 여전히 정상 작동 — `paragraph_layout` path 가 picture 를 외부 좌표 으로 처리하기 때문.

→ 본 PR 의 picture 관련 변경 (Rust by_path API 2 fn + WASM binding 2 + render_tree ImageNode cell 필드 + rendering.rs Image export + layout_picture_full table_cell_ref 매개변수 + 셀 안 picture 4 사이트 + table_cell_content.rs:714 ImageNode cell 채움 + TS dialog/bridge picture cellPath) **모두 정리됨**. PR scope 가 **셀 안 도형(shape)** 으로 한정됨.

유지 항목 (shape path 위해 필요):
- `render_tree.rs` Rectangle/Line/Ellipse/Path 노드 cell 정보 필드 (3개씩)
- `layout_shape_object` / `layout_cell_shape` 시그니처 + 호출자 propagate
- `rendering.rs` Rectangle/Line/Ellipse/Path 노드 export cell info
- shape by_path WASM API 2개 (`getCellShapePropertiesByPath` / `setCellShapePropertiesByPath`)
- TS bridge / dialog cellPath shape 분기 / handler

## 7. 미완 (별도 후속 task 권장)

| 항목 | 위치 | 이유 |
|------|------|------|
| 셀 안 글상자 안 picture cell propagate | `shape_layout.rs:2175, 2195` | 셀 안 글상자 안 picture (드문 nested) — 별도 후속 |
| `layout_group_child_affine` propagate | `shape_layout.rs:513-` | 그룹 자식 cell ctx propagate |
| `layout_textbox_content` propagate | `shape_layout.rs:1597-` | 글상자 nested case |
| nested table (셀 안 표 안 도형) | by_path 배열 2단계 이상 | 본 task 1-level 만 검증, 구조는 nested 지원 가능 |
| `paragraph_layout.rs` TAC inline picture 의 `para_index` 가 셀 내부 idx | line 2698, 3076, 3214 | 외부 좌표 처리 효과 분석 권장 (picture 가 작동하지만 좌표 의미 불일치 가능) |

## 8. 결정 이력 요약

1. **최초**: 4-layer fix (handler / dialog / WASM API / 호출 사이트)
2. **작업 중 발견**: render_tree 5 노드 cell 정보 부재 ([#issuecomment-4553872425](https://github.com/edwardkim/rhwp/issues/1138#issuecomment-4553872425))
3. **초기화** (5-layer 시도 → 작업 범위 큼)
4. **분량 측정**: PR #832 와 동일 규모 (~300 줄) → **5-layer 한 task 로 진행** ([#issuecomment-4553934132](https://github.com/edwardkim/rhwp/issues/1138#issuecomment-4553934132))
5. **단계 진행**: Stage 1 (render_tree + layout) → Stage 2 (Rust API + tests) → Stage 3 (TS) → Stage 4 (WASM 빌드 + E2E + 회귀)

## 9. 다음 단계

- [ ] `devel` 으로 merge (작업지시자 승인 후)
- [ ] 후속 task (위 미완 항목) 별도 이슈 등록 권장
