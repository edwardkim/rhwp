# Task #1151 최종 결과 보고서 (v9 통합) — 표 + picture 한컴 정합

이슈: [#1151](https://github.com/edwardkim/rhwp/issues/1151)
브랜치: `local/task1151` → upstream `edwardkim:devel` PR

v1 보고서 (1차): [task_m100_1151_report.md](task_m100_1151_report.md) — 셀 안 floating picture 삽입만 정합
본 v9 통합 보고서는 v1 + v2 + v3 + v4 + v5 + v6 + v7 + v8 + v9 의 누적 산출물 전체를 정리한다.

## 1. 사용자 의도 (원 task scope)

> 표 + picture 시나리오의 한컴 정합 — 삽입 + 토글 + 시각 + 클릭 + 다중 picture + 본문 path

본 task 는 9 phase 누적으로 진행 (v1~v6 + v8/v9: 기능 정합, v7: 코드 품질 정리):

| Phase | 범위 | 산출 |
|------|------|------|
| v1 | 셀 안 picture 신규 삽입 (한컴 패턴: floating sibling) | `insert_picture_native` 셀 분기 + WASM/TS bridge |
| v2 | tac false→true 토글 시 outer paragraph inline 마이그레이션 | `migrate_picture_floating_to_inline` helper + 6 단위 + 4 통합 테스트 |
| v3 | `wrap=TopAndBottom` 표 + sibling inline picture 의 시각 layout 정합 | `calc_sibling_topandbottom_table_reserved_hu` helper + 6 단위 테스트 |
| v4 | 셀 안 inline picture 의 click hit-test + 개체 속성 dialog | 8 caller 갱신 + ImageNode 확장 + cursor_rect 셀 분기 + by_path API |
| v5 | `set_picture_properties_native` 의 page tree cache invalidate 누락 | 1 줄 fix + 1 regression 테스트 |
| v6 | `Table::update_ctrl_dimensions` 의 `self.common` 동기화 누락 | 2 줄 fix + 2 regression 테스트 |
| v7 | Audit 기반 코드 품질 정리 (머지 전) | 4 helper 추출 + 1 주석 보강 |
| **v8** | **한컴 native 시연 비교 — 결함 A, B, C fix** | v1 셀 분기 rel_to=Paper + dialog Para 옵션 + 사용자 클릭 좌표 사용 |
| **v9** | **한컴 native 추가 결함 — D, E, F fix** | 가로 분배 + line wrap + 본문 path 재설계 + cache invalidate |

## 2. v1 ~ v4 요약

### v1 (셀 floating picture 삽입)
- Fix: `insert_picture_native` 가 표 sibling 의 같은 paragraph 에 floating picture (tac=false, wrap=Square, Paper-relative offset) 를 control 로 추가.
- 자료: [Stage 1](../../working/archives/task_m100_1151_stage1.md) · [Stage 2](../../working/archives/task_m100_1151_stage2.md) · [Stage 3](../../working/archives/task_m100_1151_stage3.md).

### v2 (floating → inline 토글 model 정합)
- 한컴 정합 (Scenario A~D dump 비교): 4 필드만 갱신 — `treat_as_char`, `h/v_rel_to=Para`, `h/v_offset=0`, `parent.line_segs[0].line_height = picture_height`.
- Fix: `migrate_picture_floating_to_inline` helper + `set_picture_properties_native` 의 tac false→true migration 분기.
- 자료: [hancom_picture_tac_toggle.md](../../tech/investigations/issue-1151/hancom_picture_tac_toggle.md) · [v2 Stage 1](../../working/archives/task_m100_1151_v2_stage1.md) · [Stage 2](../../working/archives/task_m100_1151_v2_stage2.md).

### v3 (TopAndBottom 표 + sibling inline picture 시각 layout)
- Fix: `calc_sibling_topandbottom_table_reserved_hu` helper — paragraph 내 sibling TopAndBottom 표의 outer_margin top + height + bottom 합산 → inline picture 의 y_baseline 을 그만큼 아래로 밀어 표 아래 그림.
- 자료: [Stage 3](../../working/archives/task_m100_1151_v3_stage3.md) · [Stage 4](../../working/archives/task_m100_1151_v3_stage4.md).

### v4 (셀 안 inline picture click + 개체 속성)
- Root cause (5-layer fault): ImageNode struct 미확장 + layout caller 8 곳 미전달 + cursor_rect skip + rendering.rs JSON 누락 + dialog cellPath 분기 부재.
- Fix: ImageNode 확장 + 8 caller 갱신 + cursor_rect 셀 분기 + JSON 직렬화 확장 + by_path API + dialog 분기.
- 진짜 진입점 확정: `picture_footnote.rs:57` `layout_picture_full`.
- 자료: [Stage 6](../../working/archives/task_m100_1151_v4_stage6.md) · [Stage 7](../../working/archives/task_m100_1151_v4_stage7.md).
- 검증: tac-img-02.hwp 7 페이지 표 셀 안 picture 모두 click ✓ (6/7 페이지 사각형 글상자 안 picture 는 #1171 별도).

## 3. v5 — page tree cache invalidate 누락 (사용자 시연 발견)

### 증상
v4 PR 직전 사용자가 rhwp-studio 시연 중 발견: 신규 표 + 셀 안 picture 삽입 + tac toggle 시 **시각 변화 없음**.

### 진단 (4 layer)
| Layer | 상태 |
|-------|------|
| model + composer + paragraph_layout + render tree | ✓ 정상 (Rust 단까지 모두 정합) |
| page tree cache | ✗ **stale** |

### Root cause
`set_picture_properties_native` 가 `invalidate_page_tree_cache()` 호출 누락 (다른 6 setter 는 모두 호출).

### Fix
1 줄 추가 + regression test 1 개.
자료: [v5 Stage 9 보고서](../../working/archives/task_m100_1151_v5_stage9.md).

## 4. v6 — Table::update_ctrl_dimensions self.common 미동기화 (사용자 시연 발견)

### 증상
v5 fix 후 사용자 재시연: 표 1×1 + **셀 크기 조절** + 셀 안 picture 삽입 + tac toggle 시 표 박스가 picture 영역까지 포함한 큰 박스 + picture 가 표 박스 안 좌상단.

### Root cause
`Table::update_ctrl_dimensions` 가 raw_ctrl_data bytes 만 갱신하고 self.common.width/height 미동기화 → paragraph_layout 의 v3 helper 가 stale `t.common.height` 사용.

### Fix
2 줄 추가 + regression test 2 개.
자료: [v6 Stage 10 보고서](../../working/archives/task_m100_1151_v6_stage10.md).

### 사용자 시연 검증 통과 (2026-05-30)
> "정확함. rhwp에서 그렇게 만든 문서 한컴에서도 정합"

## 5. v7 — Audit 기반 코드 품질 정리 (머지 전)

### Context
v1~v6 작업 완료 후 사용자 요청으로 코드 audit 진행. 다른 PR 의 코드 패턴과 비교하여 anti-pattern / 추상화 여지 식별 (Explore agent 2 회 분석).

### Audit 결과 (9 항목 평가)

| # | 항목 | 처리 | Stage |
|---|------|------|-------|
| 1 | cell_ctx → ImageNode 3 필드 매핑 4 곳 반복 | **Fix** — `CellContext::last_image_indices()` helper | 12 |
| 4 | by_path setter/getter 4 함수의 cell_path 파싱 + traversal 95 줄 중복 | **Fix** — `parse_cell_path_json` + `resolve_cell_paragraph_mut` helper | 13 |
| 7 | paragraph_layout 3 곳의 ImageNode 생성 boilerplate | **Fix** — `make_picture_image_node` private helper (picture_footnote 의 helper 패턴 정합) | 14 |
| 2 | Table::update_ctrl_dimensions 의 dual maintenance | **유지 + 주석 보강** — serializer 의 source-of-truth 정책 차이로 의도된 구조 | 15 |
| 3 | invalidate_page_tree_cache 호출 분산 | **유지** — 책임 분리 원칙으로 의도된 분산 | — |
| 5 | dialog setter 4 종 분기 | Skip (UI 리팩토링, 별도) | — |
| 6 | layout_picture 매개변수 11 개 | Skip (`#[allow(clippy::too_many_arguments)]` 표준) | — |
| 8 | migrate_picture_floating_to_inline 위치 | OK | — |
| 9 | calc_sibling_topandbottom_table_reserved_hu 가시성 | OK | — |

### 정량 효과
- 4 helper 추출 + 1 주석 보강
- 순 -48 줄 (보일러플레이트 제거 효과)
- 4 항목 fix + 2 항목 의도된 유지 (분석 명시) + 3 항목 OK/skip

자료: [v7 Stage 16 보고서](../../working/archives/task_m100_1151_v7_stage16.md).

## 5'. v8/v9 — 한컴 native 시연 + 6 결함 fix

PR #1173 (v7 까지) 발행 후 사용자 한컴 윈도우 편집기 직접 시연으로 6 결함 발견 + 동일 PR 내 fix.

### 결함별 fix

| # | 결함 | Stage | Fix |
|---|------|-------|-----|
| **A** | v1 셀 분기 picture rel_to=Page (한컴 native = Paper) | 18 | attr bits + typed field 모두 Paper |
| **B** | dialog horzRelSelect 옵션에 Para 누락 | 19 | Para 옵션 추가 |
| **C** | picture 위치 = 셀 좌상단 (한컴은 사용자 클릭 위치) | 20 | insertPicture 시그니처에 paper_offset_x/y_hu (Option<i32>) + studio 가 drag 좌표 변환 후 전달 |
| **D** | 동일 paragraph sibling 2 TAC picture 가 가로 분배 안 됨 | 22-25 + wrap 순서 fix | layout_shape_item 에 ParaInlineState 도입 + 가로 분배 cursor + line wrap |
| **E** | 본문 picture default tac=true (한컴 native = tac=false) | 27a + 27b | 본문 inline path 전체 삭제 + 셀 분기와 통합 (floating sibling) + studio paper offset 변환 inCell 제한 제거 |
| **F** | `insert_picture_native` 의 invalidate_page_tree_cache 호출 누락 (v5 와 동일 패턴) | 26 | 셀 분기 + 본문 분기에 호출 추가 (1 줄 × 2) |

### v9 핵심 — 결함 D (가로 분배 + line wrap)

진단:
- paragraph_layout 의 v3 path 호출 안 됨 — 진짜 path = `layout.rs:layout_shape_item` (build_columns → layout_column_item → 각 control 별 호출)
- `has_prior_tac_in_para` 가 모든 TAC 종류 동일 처리 → Pic2 호출 시 `para_start_y` 갱신 → 세로 진행

Fix (Option 4):
- `ParaInlineState { cursor_x, line_top_y, line_height }` HashMap<para_index, _> 도입
- `collect_sibling_tac_picture_widths_px` helper (paragraph_layout.rs)
- 시퀀스 위치 판별 (`is_single_pic / is_first_in_seq / is_subsequent_in_seq / is_last_in_seq`)
- 가로 분배 + alignment (시퀀스 첫 picture 의 total_tac_width 기반) + line wrap
- **pic_y 결정을 pic_x 처리 뒤로 옮김** — wrap 후 갱신된 state.line_top_y 사용 (추가 발견 결함)

자료:
- [v8 Stage 20 보고서](../../working/archives/task_m100_1151_v8_stage20.md)
- [v8/v9 Stage 28 보고서](../../working/archives/task_m100_1151_v8v9_stage28.md)

### 사용자 시연 검증 통과 (2026-05-30)

| 시나리오 | rhwp 정합 |
|---------|---------|
| 신규 셀 picture 삽입 (한컴 비교: 종이 기준, 사용자 그린 위치) | ✓ (v8 A/B/C) |
| 셀 안 picture 글자처럼 토글 (한컴 비교: 표 아래 inline) | ✓ (v6) |
| 셀 안 picture 글자처럼 해제 (한컴 비교: 가로/세로 = 문단) | ✓ (v8 B) |
| 동일 셀 picture 2 장 토글 — 작은 크기 (한컴 비교: 가로 분배) | ✓ (v9 D) |
| 동일 셀 picture 2 장 토글 — 큰 크기 (한컴 비교: line wrap) | ✓ (v9 D wrap 순서 fix) |
| 본문 picture 신규 삽입 (한컴 비교: 종이, 글자처럼 미체크, 사용자 그린 위치) | ✓ (v9 E + 27b) |
| rhwp 저장 → 윈도우 한컴 열기 (양방향 정합) | ✓ |

## 6. 회귀 검증 (전체 누적)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` (1454 tests) | passed 1454 / failed 0 / ignored 6 |
| `cargo clippy --lib -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| v1 셀 안 picture 신규 삽입 시각 시연 | PASS |
| v2 통합 테스트 4 (한컴 산출물 양방향 정합) | PASS |
| v3 helper 단위 테스트 6 + 4 시각 시나리오 | PASS |
| v4 사용자 시연 (tac-img-02.hwp 7개 셀 picture) | PASS |
| v5 신규 regression 1 (cache invalidate) | PASS |
| v6 신규 regression 2 (table.common 동기화) | PASS |
| v6 사용자 시연 (한컴 정합 양방향) | PASS |
| v7 회귀 (모든 기존 + 진단 테스트) | PASS |

## 7. 후속 task

- [#1171](https://github.com/edwardkim/rhwp/issues/1171) 사각형 글상자 안의 picture click hit-test (이중 nested). v4 진단 중 발견되어 별도 분리.

## 8. 변경 파일 (v1~v7 통합)

### Rust 모델
- `src/model/table.rs` — Table::update_ctrl_dimensions 의 self.common 동기화 (v6) + 의도된 dual 구조 주석 보강 (v7)
- `src/renderer/render_tree.rs` — ImageNode struct + inline_shape_positions 정규화 (v4)
- `src/renderer/layout.rs` — `CellContext::last_image_indices()` helper (v7)

### Rust 렌더 layer
- `src/renderer/layout/picture_footnote.rs` — cell_ctx 시그니처 + set_inline_shape_position (v4) + last_image_indices 사용 (v7)
- `src/renderer/layout/paragraph_layout.rs` — 3 곳 ImageNode + v3 sibling TopAndBottom helper (v3, v4) + make_picture_image_node helper 추출 (v7)
- `src/renderer/layout/{table_layout, table_partial, shape_layout, table_cell_content, layout}.rs` — layout_picture caller 8 곳 갱신 (v4)

### Rust API layer
- `src/document_core/commands/object_ops.rs`
  - v1: 셀 floating insert + v2: migrate helper + tac toggle 분기
  - v4: get/set_cell_picture_properties_by_path_native
  - v5: invalidate_page_tree_cache 호출 (1 줄)
  - v7: parse_cell_path_json + resolve_cell_paragraph_mut helper + 4 함수 적용
- `src/document_core/queries/rendering.rs` — Image JSON 직렬화 확장 (v4)
- `src/document_core/queries/cursor_rect.rs` — 셀 안 inline shape hit-test 분기 (v4)
- `src/wasm_api.rs` — insert_picture + 셀 picture by_path export (v1, v4)

### Studio
- `rhwp-studio/src/core/wasm-bridge.ts` (v4)
- `rhwp-studio/src/ui/picture-props-dialog.ts` (v4)
- `rhwp-studio/src/command/commands/insert.ts` (v4)

### 테스트
- `issue_1151_v2_tac_toggle_tests` — 6 단위 (v2) + 4 통합 (v2) + 1 regression (v5) + 1 baseline + 1 regression (v6) = **13 개**
- `issue_1151_v3_helper_tests` — 6 (v3)
- 합계 v1~v7 신규 테스트 **19 개**, 회귀 0

### 자료
- `mydocs/tech/investigations/issue-1151/hancom_picture_tac_toggle.md` (v2)
- `mydocs/plans/task_m100_1151_{,v2,v3,v4}{,_impl}.md` (4 phase 계획서)
- `mydocs/working/task_m100_1151_{stage1, stage2, stage3, v2_stage1, v2_stage2, v3_stage3, v3_stage4, v4_stage6, v4_stage7, v5_stage9, v6_stage10, v7_stage16}.md`

## 9. 결론

원 task scope ("표 + picture 한컴 정합 — 삽입 + 토글 + 시각 + 클릭") 의 4 측면 모두 완성. 사용자 시연 중 발견된 2 개 추가 결함 (v5 cache invalidate + v6 table.common 동기화) 도 same-PR 으로 fix. 머지 전 audit 기반 4 helper 추출로 코드 품질 정리 + 2 항목 의도된 분리 분석 명시. 1 개 별도 결함 (#1171) 만 후속 task 로 이관.

→ 통합 PR (devel) 발행 + Issue #1151 close.
