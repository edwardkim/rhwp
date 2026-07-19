# PR Title (안)

`Task #1151: 표 + picture 한컴 정합 (삽입 + 토글 + 시각 + 클릭)`

---

# PR Body (안)

## Summary

closes #1151

표 + picture 시나리오에서 한컴과 동작이 다르던 4개 측면 (셀 안 picture **삽입** / floating ↔ inline **토글** / paragraph 안 sibling 표 + picture **시각 layout** / 셀 안 inline picture **click hit-test**) 을 한컴 정합으로 정렬. 사용자 시연 중 발견된 후속 결함 2개 (page tree cache invalidate 누락 / Table::common 동기화 누락) 도 same-PR 으로 fix. 머지 전 audit 기반 코드 품질 정리 (4 helper 추출 + 의도된 분리 분석 명시) 적용.

검증 자료: `samples/tac-verify/scenario-{a,b,c,d}-{before,after}.hwp` (8 개, 한컴 2022 산출물) + `samples/tac-img-02.hwp` (v4 검증).

## Phase 별 작업 (누적)

| Phase | 변경 요지 | Rust 변경 규모 |
|------|---------|----------------|
| **v1** | 셀 안 picture 신규 삽입 — 한컴 패턴 (표 sibling floating, tac=false, wrap=Square, Page-relative offset) | `insert_picture_native` 셀 분기 (~90 줄) + WASM/TS bridge |
| **v2** | floating→inline tac 토글 model 정합. 한컴 Scenario A~D dump 비교로 4 필드 갱신만 (`tac=true`, `h/v_rel_to=Para`, `h/v_offset=0`, `parent.line_segs[0].line_height = picture_height`) 확정 | `migrate_picture_floating_to_inline` helper + `set_picture_properties_native` 의 was_tac → now_tac 분기 |
| **v3** | `paragraph` 안 sibling `wrap=TopAndBottom` 표 + `tac=true` picture 의 시각 layout — picture y 가 표 영역만큼 아래로 보정 (한컴 정합) | `calc_sibling_topandbottom_table_reserved_hu` helper + paragraph_layout 의 2 path |
| **v4** | 셀 안 inline picture 의 click hit-test + 개체 속성 dialog (5-layer fault: ImageNode struct + 8 caller + cursor_rect + rendering JSON + studio cellPath 분기) | ImageNode 3 필드 확장 + 8 caller 갱신 + by_path API 신설 + studio 분기 |
| **v5** | 사용자 v4 머지 직전 시연 발견 — tac toggle 시 시각 변화 없음. Rust 측 model+composer+layout 모두 정합이지만 `set_picture_properties_native` 의 `invalidate_page_tree_cache` 호출 누락 (다른 6 setter 와 일관성 결함) | 1 줄 + regression 1 |
| **v6** | v5 fix 후 추가 시연 발견 — 셀 size 조절 후에도 정합 안 됨. `Table::update_ctrl_dimensions` 가 `raw_ctrl_data` bytes 만 갱신하고 `self.common.width/height` 미동기화. paragraph_layout 의 v3 helper 가 stale `t.common.height` 사용 | 2 줄 + regression 2 |
| **v7** | Audit (Explore agent 2 회 분석, 9 항목 평가) 기반 코드 품질 정리. 4 helper 추출 (`CellContext::last_image_indices` / `parse_cell_path_json` / `resolve_cell_paragraph_mut` / `make_picture_image_node`) + 의도된 분리 2 항목 (Table dual maintenance / invalidate 분산) 분석 명시 | 4 helper + 주석 보강 (순 -48 줄) |
| **v8** | PR 발행 후 한컴 native 직접 시연 — 3 결함 발견. A: 셀 분기 picture rel_to=Page (한컴=Paper). B: dialog 가로 select 옵션에 Para 누락. C: picture 위치 = 셀 좌상단 (한컴은 사용자 클릭 위치) | A: attr+typed Paper / B: Para 옵션 / C: insertPicture 시그니처에 paper_offset_x/y_hu (Option) + studio drag 좌표 변환 |
| **v9** | 추가 한컴 native 시연 — 3 결함 발견. D: 동일 paragraph sibling 2 TAC picture 가로 분배 안 됨 + line wrap. E: 본문 picture default tac=true (한컴=tac=false). F: insert_picture_native invalidate_page_tree_cache 누락 (v5 패턴) | D: ParaInlineState struct + 가로 분배 cursor + line wrap (pic_y 결정 순서 보강) / E: 본문 path 셀 분기와 통합 (tac=false default, sibling control) + studio paper offset 본문도 적용 / F: invalidate 호출 추가 |

## 핵심 변경 파일

### Rust
- `src/model/table.rs` — `update_ctrl_dimensions` 에서 `self.common.width/height` 동기화 (v6) + 의도된 dual maintenance 주석 보강 (v7)
- `src/renderer/render_tree.rs` — `ImageNode` 에 `cell_index / cell_para_index / outer_table_control_index` 필드 추가 + inline_shape_positions key 정규화 (v4)
- `src/renderer/layout.rs` — `CellContext::last_image_indices()` helper (v7)
- `src/renderer/layout/picture_footnote.rs` — `layout_picture` / `layout_picture_full` 시그니처에 `cell_ctx` 추가 + `set_inline_shape_position` 호출 (v4) + last_image_indices 사용 (v7)
- `src/renderer/layout/paragraph_layout.rs` — 3 곳 ImageNode 생성 + `calc_sibling_topandbottom_table_reserved_hu` helper (v3, v4) + `make_picture_image_node` helper 추출 (v7)
- `src/renderer/layout/{table_layout, table_partial, shape_layout, table_cell_content, layout}.rs` — layout_picture caller 8 곳 갱신 (v4)
- `src/document_core/commands/object_ops.rs`
  - v1: `insert_picture_native` 셀 분기
  - v2: `migrate_picture_floating_to_inline` helper + tac toggle migration
  - v4: `get/set_cell_picture_properties_by_path_native`
  - v5: `invalidate_page_tree_cache()` 호출 (1 줄)
  - v7: `parse_cell_path_json` + `resolve_cell_paragraph_mut` helper + 4 함수 적용 (cell picture/shape × set/get)
- `src/document_core/queries/rendering.rs` — Image JSON 직렬화에 `cellIdx/cellParaIdx/outerTableControlIdx` 추가 (v4)
- `src/document_core/queries/cursor_rect.rs` — 셀 안 inline shape hit-test 분기 + JSON 응답 cellPath/innerControlIdx (v4)
- `src/wasm_api.rs` — `insertPicture` (v1) + `getCellPicturePropertiesByPath` / `setCellPicturePropertiesByPath` export (v4)

### Studio (rhwp-studio)
- `src/core/wasm-bridge.ts` — 셀 picture get/set wrapper 2 개 (v4)
- `src/ui/picture-props-dialog.ts` — cellPath getter/setter 분기 (v4)
- `src/command/commands/insert.ts` — cellPath 구성 조건에 `image` 추가 (v4)

### 테스트
- `issue_1151_v2_tac_toggle_tests` (object_ops.rs) — 6 단위 (v2) + 4 통합 (v2 한컴 산출물 양방향 정합) + 1 regression (v5) + 1 baseline + 1 regression (v6) = **13 개**
- `issue_1151_v3_helper_tests` (paragraph_layout.rs) — 6 helper 단위 (v3)
- 합계 v1~v6 신규 테스트 **19 개**, 회귀 0

### 자료
- `samples/tac-verify/scenario-{a,b,c,d}-{before,after}.hwp` (v2 한컴 정합 검증 산출물 8 개)
- `mydocs/tech/investigations/issue-1151/hancom_picture_tac_toggle.md` — Scenario A~D dump 분석 (v2 H1 확정 근거)
- `mydocs/plans/task_m100_1151_{,v2,v3,v4}{,_impl}.md` — 4 phase 계획서
- `mydocs/working/task_m100_1151_{stage1, stage2, stage3, v2_stage1, v2_stage2, v3_stage3, v3_stage4, v4_stage6, v4_stage7, v5_stage9, v6_stage10}.md` — 11 단계별 보고서
- `mydocs/report/task_m100_1151_{report, v6_report}.md` — v1 최종 + v6 통합 최종

## 자동 검증 (필수)

| Command | 결과 |
|---------|------|
| `cargo fmt --all -- --check` | clean |
| `cargo test --lib` | **1454 passed, 0 failed, 6 ignored** (v1~v9 신규 8, 회귀 0) |
| `cargo clippy --lib -- -D warnings` | clean |

## 시각 검증 (참고)

검증 환경:

| 측면 | 환경 |
|------|------|
| rhwp 빌드/렌더 | **macOS Darwin 25.5.0** — `cargo build` 네이티브 + `docker compose --env-file .env.docker run --rm wasm` WASM + rhwp-studio dev (`npx vite --port 7700`) + 호스트 Chrome |
| 한컴 비교 기준 | **Windows + 한글 2022 편집기** — `samples/tac-verify/scenario-*.hwp` 산출 + rhwp 산출물 한컴 연 양방향 정합 검증 |

검증 시나리오:
1. **v1** — 신규 빈 문서 → 1×1 표 → 셀 안 picture 삽입 → floating sibling 으로 떠있는 위치 확인 (incellpicture.hwp 정합)
2. **v2** — 한컴 산출물 (scenario-{a..d}-before.hwp) 을 rhwp 가 파싱 → tac=true 토글 → model 이 한컴 scenario-{a..d}-after.hwp 와 동치 (4 통합 테스트)
3. **v3** — `paragraph` 안 sibling TopAndBottom 표 + tac picture 4 시나리오 시각 통과 (사용자 시연)
4. **v4** — tac-img-02.hwp 의 7 페이지 셀 안 picture click 모두 정상 (5 페이지 = TAC 표 + 14 페이지 3 개 picture 포함)
5. **v5/v6** — 신규 표 + 셀 크기 조절 + 셀 안 picture 삽입 + tac toggle → picture 가 표 아래 inline 위치로 정확 이동. 양방향 검증 (rhwp 산출물을 한컴에서 연 결과도 정합).
6. **v7** — 위 4 시나리오 모두 (v1 / v2/v5 / v6 / v4) helper 추출 후 동일 정합 유지 확인 (행위 무변경 검증).
7. **v8 (A+B+C)** — 한컴 native 시연 비교: 셀 안 picture 신규 삽입 시 dialog 가로/세로 = "종이" + 위치 = 사용자가 그린 곳. 글자처럼 해제 시 가로 = "문단" 정상 표시.
8. **v9 (D)** — 동일 셀 picture 2 장 + 글자처럼 토글: 작은 picture (둘 합 ≤ 페이지 폭) 는 가로 분배, 큰 picture (페이지 폭 초과) 는 다음 line 으로 wrap. 한컴 native 동작 정확 매칭.
9. **v9 (E+F)** — 본문 picture 신규 삽입 시 글자처럼 미체크 + 가로/세로 = 종이 + 위치 = 사용자가 그린 곳. rhwp 산출물을 한컴에서 열어도 동일 정합.

## 후속 task (별도 이슈 분리)

- [#1171](https://github.com/edwardkim/rhwp/issues/1171) — **사각형 글상자 안의 picture click hit-test 미지원 (이중 nested)**. v4 진단 중 발견되어 별도 분리. tac-img-02.hwp 6/7 페이지에서 재현. Shape(InFrontOfText) → text_box → paragraph → Picture 의 이중 nested 구조 + 머리말/꼬리말 5-tuple 패턴 유사 필요.

## 작업 과정 기록 (Hyper-Waterfall)

- 계획서: `mydocs/plans/task_m100_1151_{,v2,v3,v4}{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1151_*_stage{N}.md` (총 11 개)
- 최종 통합 보고서: `mydocs/report/task_m100_1151_v6_report.md`

## Commits 요약 (27 개)

```
v1 (4): a1b60239 e47052c0 5b179e7b 3b14d869
v2 (5): af5e222a 503214e4 19d33b42 0388dd75 0fe6fe49
v3 (3): 3a878e89 4ea8ecf4 20b54845
v4 (4): 568cefcc 4e990fa5 0b698d8c f7e4f98a
v5 (2): cbda6660 bd17d0f9
v6 (2): 784e0063 f5b1d3a6
v7 (5): 78638bfe 23037825 18930ccf 06dfc91d 72d2e614
final (1): 0de53e73 (v6 시점 보고서, v7 commit 으로 갱신됨)
```
