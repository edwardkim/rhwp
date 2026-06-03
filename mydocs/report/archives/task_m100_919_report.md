# Task M100-919 최종 보고서 — 글상자 한컴 UX 정합 (안 표 셀 + Esc escape stack)

- 이슈: [#919](https://github.com/edwardkim/rhwp/issues/919)
- 브랜치: `local/task919` (base: `origin/devel = 4a76f5a1` PR #1047 머지 후)
- 일시: 2026-05-21
- 마일스톤: v1.0.0
- assignee: @edwardkim
- 단계별 문서:
  - `mydocs/plans/task_m100_919.md` (수행 계획서, 승인)
  - `mydocs/plans/task_m100_919_impl.md` (구현 계획서, 승인)
  - `mydocs/working/task_m100_919_stage1.md` (Stage 1 정밀 진단)

## 1. 결과 요약

`samples/table-in-tbox.hwp` 의 글상자 클릭 동작이 한컴 UX 와 정합되도록 정정.

### 1.1 한컴 UX 정합 (작업지시자 시각 판정 통과)

| 동작 | BEFORE | AFTER |
|------|--------|-------|
| 글상자 외곽 경계선 클릭 | 객체 선택 | 객체 선택 ✓ |
| 글상자 내부 (텍스트 위) 클릭 | 객체 선택 (1차) → 더블클릭 필요 | **즉시 텍스트 편집 진입** |
| 글상자 내부 (빈 영역) 클릭 | 본문 paragraph 0 fall-through ❌ | **글상자 텍스트 편집 진입** ✓ |
| 글상자 안 표 셀 클릭 | 글상자가 가로채기 ❌ | **안 표 셀 cursor 진입** ✓ |
| 글상자 안 표 셀 → Esc | 글상자 객체 선택 ❌ | **안 표 객체 선택** ✓ |
| 표 객체 선택 → Esc | 표 밖 cursor (글상자 안 본문) | **isTextBox 유지** → 다음 Esc 시 글상자 객체 선택 자연 전이 |
| 글상자 객체 선택 → 내부 클릭 | 더블클릭 필요 | **즉시 텍스트 편집 진입** |

### 1.2 한컴 UX Escape Stack

글상자 안 표 셀 위치에서 Esc 연타:
1. **1차 Esc** → 안 표 객체 선택 (표 핸들 표시)
2. **2차 Esc** → cursor 가 글상자 안 본문으로 이동 (isTextBox 유지)
3. **3차 Esc** → 글상자 객체 선택
4. **4차 Esc** → cursor 가 본문으로 이동

## 2. 결함 본질 (Stage 1 정밀 진단)

`samples/table-in-tbox.hwp` page 1 의 글상자 (paragraph 0, control_index 2):
- 글상자 외곽 (검정 테두리): x=75.6, y=75.6, w=628.5, h=976.3
- 글상자 안 큰 표 (pi=6): y=159.7~841.3

**4 코드 갭 정확 식별**:

### Gap-1 — `src/document_core/queries/cursor_rect.rs:564~593`

`collect_runs` 가 `TableCell` 노드만 `cell_bboxes` 수집 — **`RenderNodeType::Rectangle/Ellipse/Path` (글상자) 노드 미수집**. 글상자 안 빈 영역 클릭 시 본문 paragraph 0 fall-through.

### Gap-2 — `src/document_core/queries/cursor_rect.rs:871~939`

`0.5 인라인 Shape 히트 검사` 가 글상자 hit 시 본문 TextRun (paragraph 0) 으로 매핑. 글상자 안 표 셀 영역도 가로채.

### Gap-3 — `src/wasm_api.rs` 에 `getShapeBBox` API 부재

studio 의 `isShapeBorderClick` 동등 헬퍼 (sec/ppi/ci 시그니처) 가 글상자 bbox 조회 불가.

### Gap-4 — `rhwp-studio/src/engine/input-handler-mouse.ts` 의 글상자 1차 클릭 흐름

`findPictureAtClick` 이 글상자도 picHit 으로 잡아 무조건 객체 선택. 한컴 UX 위반.

### Gap-5 — `rhwp-studio/src/engine/input-handler-keyboard.ts:932` Esc 처리

`isInTextBox` 가 `isInCell` 보다 먼저 검사되어 글상자 안 표 셀 Esc 시 글상자가 가로채기.

### Gap-6 — `rhwp-studio/src/engine/cursor.ts:1131` `enterTableObjectSelection`

`isInTextBox()` 시 무조건 `false` 반환 → 글상자 안 표 선택 불가.

### Gap-7 — `src/document_core/queries/cursor_nav.rs:447` `resolve_table_by_path`

path 의 모든 항목이 Table 이어야 함 → 글상자 안 표 traverse 시 `controls[2]가 표가 아닙니다` 에러.

## 3. 구현 (Stage 2.1~2.5)

### Stage 2.1 — Native: TextBox 수집 + getShapeBBox API

**`src/document_core/queries/cursor_rect.rs`**:
- `TextBoxBboxInfo` struct 신규
- `collect_runs` 에 `RenderNodeType::Rectangle/Ellipse/Path` 메타 수집 분기 추가
- `format_textbox_entry` 헬퍼 신규 — 글상자 첫 paragraph 진입 응답
- hit_test_native 매칭 우선순위 정정 (hit_cell → clicked_cell → textbox_hit → hit_body)
- 0.5 inline Shape 분기에서 글상자(text_box 보유 Shape)는 fall-through (`break`) — 메인 매칭이 처리

**`src/document_core/commands/table_ops.rs`**:
- `get_shape_bbox_native` 신규 (Rectangle/Ellipse/Path 노드 검색, `getTableBBox` 동등 패턴)

**`src/wasm_api.rs`**:
- `getShapeBBox` 공개 API

### Stage 2.2 — Studio: isShapeBorderClickByRef + 클릭 흐름 정정

**`rhwp-studio/src/engine/input-handler.ts`**:
- `isShapeBorderClickByRef` (sec/ppi/ci 시그니처, `getShapeBBox` + 5px tolerance)
- `findShapeByOuterClick` (외곽 근처 클릭 시 글상자 탐색)

**`rhwp-studio/src/engine/input-handler-mouse.ts`**:
- 글상자 외곽 경계선 (`isShapeBorderClickByRef`) → 객체 선택
- 글상자 외곽 근처 (`findShapeByOuterClick`) → 객체 선택
- 글상자 내부 (`hit.isTextBox`) → 즉시 cursor 진입 (한컴 UX)
- `picHit.type === 'shape'` 분기에서 외곽 경계선만 객체 선택, 내부는 fall-through (가로채기 제거)

**`rhwp-studio/src/core/wasm-bridge.ts`**:
- `getShapeBBox` 메서드 추가

### Stage 2.3 — Studio: 객체 선택 → 편집 진입

**`rhwp-studio/src/engine/input-handler-mouse.ts`**:
- 글상자 객체 선택 상태에서 같은 글상자 내부 클릭 시 즉시 텍스트 편집 진입 (객체 선택 해제 + cursor 이동)

### Stage 2.4 — 회귀 가드 + CI

**`tests/issue_919_textbox_hit_test.rs`** 신규 (5 케이스):
- `issue_919_textbox_inner_text_hit_returns_textbox_path` — 글상자 안 텍스트 hit
- `issue_919_textbox_inner_empty_hit_returns_textbox_entry` — 글상자 안 빈 영역 hit
- `issue_919_textbox_outside_hit_returns_body` — 글상자 외부 hit (회귀 가드)
- `issue_919_get_shape_bbox_returns_correct_dimensions` — getShapeBBox bbox 정합
- `issue_919_inner_table_cell_in_textbox_has_two_path_entries` — 글상자 안 표 셀 cellPath

### Stage 2.5 — Esc 우선순위 정정 + resolve_table_by_path 글상자 traverse

**`rhwp-studio/src/engine/input-handler-keyboard.ts`**:
- Esc 처리 우선순위 정정: 글상자 안 표 셀 (nestingDepth >= 2 + isTextBox) → 표 객체 선택 우선
- 본문 표 셀 / 글상자 안 본문 / 본문 paragraph → 기존 처리

**`rhwp-studio/src/engine/cursor.ts`**:
- `enterTableObjectSelection`: `cellPath.length >= 2` (글상자 안 표) 면 isTextBox 상태에서도 허용
- `moveOutOfSelectedTable`: 글상자 안 표 → 외부 이동 시 `isTextBox` 상태 유지 (escape stack 자연 전이)

**`src/document_core/queries/cursor_nav.rs`**:
- `resolve_table_by_path`: path 중간 항목 Shape (글상자) 면 `shape.text_box.paragraphs` 로 traverse, 마지막 항목만 Table 필수
- 5 ShapeObject variants (Rectangle/Ellipse/Polygon/Arc/Curve) 모두 지원

## 4. 검증 결과 (Stage 2.4 + 2.5)

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` | **1319 passed** |
| `cargo test --release --tests` | 모든 통합 passed (issue_852 / issue_874 / issue_915 / issue_1008 / issue_919 등) |
| `cargo test --release --test issue_919_textbox_hit_test` | **5/5 passed** |
| `cargo fmt --all --check` | clean |
| TypeScript `npx tsc --noEmit` | 0 errors |
| WASM Docker 빌드 | **4.90MB** (PR #1047 4.89MB + getShapeBBox API 추가) + rhwp-studio 동기화 |
| 작업지시자 시각 판정 (글상자 진입) | **통과** |
| 작업지시자 시각 판정 (Esc escape stack) | **통과** |

## 5. 사용자 영향

`samples/table-in-tbox.hwp` 와 유사한 글상자 보유 HWP 문서:
- 글상자 BBox 테두리 클릭 → 글상자 객체 선택
- 글상자 내부 클릭 (어느 위치든) → 즉시 텍스트 편집 진입
- 글상자 안 표/이미지/기타 콘텐츠 정상 hit
- Esc → 가장 안쪽 컨테이너부터 자연스러운 escape stack

한컴 한글 편집기와 동일한 UX 정합 — 작업지시자 시각 판정 권위 게이트 통과.

## 6. 메모리 룰 정합

- ✅ `feedback_visual_judgment_authority` — 글상자 진입 + Esc 동선 작업지시자 시각 판정 게이트
- ✅ `feedback_diagnosis_layer_attribution` — 7 코드 갭 정확 식별 (cursor_rect / wasm_api / mouse handler / border helper / keyboard / cursor / cursor_nav)
- ✅ `feedback_hancom_compat_specific_over_general` — 글상자 한정 fix (Image/Equation 무영향)
- ✅ `feedback_pr_supersede_chain` — 표 UX 패턴 (`isTableBorderClick` / `enterTableObjectSelection`) 재사용 + 일반화
- ✅ `feedback_push_full_test_required` — cargo test --tests + clippy + fmt 전체 CI 패턴
- ✅ `feedback_assign_issue_before_work` — assignee @edwardkim
- ✅ `feedback_search_troubleshootings_first` — Stage 0 + Stage 1 사전 진단 (hit_test_native + RenderNodeType + studio onDblClick + Esc)
- ✅ `feedback_image_renderer_paths_separate` — N/A (단일 hit_test 경로)
- ✅ `feedback_self_verification_not_hancom` — 작업지시자 시각 판정 필수 (한컴 호환 무관 — 웹 에디터 UX, 그러나 한컴 UX 정합 권위)

## 7. 후속 / 잠재 위험 (본 task 범위 외)

- 글상자 외부 근처 클릭 (`findShapeByOuterClick`): 0~9 ci 범위 시도. 정밀 매핑은 후속 task
- 머리말/꼬리말 안 글상자 (현재 본문만 fix 범위)
- 글상자 안 글상자 중첩 (재귀 hit_test 일반화 — 본 PR 가 부분 지원: cellPath traversal)
- Stage 1 부수 발견: `hit_test_native` 가 page body_area 밖 좌표를 글상자 안 paragraph 로 매핑 (글상자 안 paragraph 좌표 정밀도 별도 검토)
- 도형 (직사각형/원) Shape 의 hit_test 동일 UX — 본 task 는 글상자 (text_box 보유) 한정

## 8. 변경 파일 요약

**Native (Rust)** — 4 파일:
- `src/document_core/queries/cursor_rect.rs` — TextBoxBboxInfo + collect_runs + hit_test 우선순위 + format_textbox_entry
- `src/document_core/queries/cursor_nav.rs` — resolve_table_by_path Shape traverse
- `src/document_core/commands/table_ops.rs` — get_shape_bbox_native
- `src/wasm_api.rs` — getShapeBBox API

**Studio (TypeScript)** — 4 파일:
- `rhwp-studio/src/engine/input-handler.ts` — isShapeBorderClickByRef + findShapeByOuterClick
- `rhwp-studio/src/engine/input-handler-mouse.ts` — 글상자 클릭 흐름 정정 + 객체→편집 전환
- `rhwp-studio/src/engine/input-handler-keyboard.ts` — Esc 우선순위 정정
- `rhwp-studio/src/engine/cursor.ts` — enterTableObjectSelection + moveOutOfSelectedTable

**테스트** — 1 파일:
- `tests/issue_919_textbox_hit_test.rs` 신규 — 5 회귀 가드

**문서** — 4 파일:
- `mydocs/plans/task_m100_919.md` (수행 계획서)
- `mydocs/plans/task_m100_919_impl.md` (구현 계획서)
- `mydocs/working/task_m100_919_stage1.md` (정밀 진단)
- `mydocs/report/task_m100_919_report.md` (본 최종 보고서)

**WASM 동기화** — 3 파일:
- `rhwp-studio/public/{rhwp.d.ts, rhwp.js, rhwp_bg.wasm}` — getShapeBBox 자동 노출
