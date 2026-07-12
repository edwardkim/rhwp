# 구현계획서 — Task M100 #2026: typeset_section_endnotes 해체 (라운드 5)

- 이슈: #2026 / 수행계획서: `task_m100_2026.md` / 작성일: 2026-07-07
- 1단계 의존 재실측 결과 반영 (r4 사각지대 보정 스캐너: 문자열 리터럴 제거 + 인덱스 대입 포함).

## 1. 의존 재실측 결과

**en_para 루프 (4447~9380, 4,934줄)** — 라운드 1 이연 본체:

- **읽기 32** (라운드 1 실측과 일치): fn 파라미터군(st/paragraphs/composed/styles/
  page_def/measured_tables/endnote_shape/…) + 참조 루프 반복자(en_ref/en_ref_idx/en_ctrl/
  prev_para) + 프리앰블 산출(shape/profile/endnote_flow_profile/endnote_start/
  endnote_refs/…) + 루프-간 플래그(boundary_prev…/continued_…/default_… 계열).
- **mut 8** (라운드 1의 9에서 오탐 1 제거): `vpos_offset` / `prev_en_bottom_vpos` /
  `prev_en_content_bottom_vpos` / `emitted_endnote_count` /
  `last_render_endnote_para_local_idx` / `pre_emitted_endnote_para_indices`(컬렉션) /
  `cleared_single_line_internal_rewind_split` / `current_endnote_had_inline_object_vpos_overestimate`
  — **정확히 "미주-간 흐름 캐리" 8종 = `EndnoteFlowState` 필드**.
- 제어 흐름: 라벨 0 / return·`?` 전부 클로저(and_then 체인) 내부 — **통이동 안전**
  (en_para 루프 자체를 옮기므로 내부 break/continue는 자체 대상).

## 2. 추출 설계

### `EndnoteFlowState` (미주-간 캐리, 값 왕복 — RunEmitState 전례)
```rust
/// [#2026] 미주 참조 루프의 미주-간 흐름 캐리 상태 (#1904 이연 설계).
struct EndnoteFlowState {
    vpos_offset: _, prev_en_bottom_vpos: _, prev_en_content_bottom_vpos: _,
    emitted_endnote_count: _, last_render_endnote_para_local_idx: _,
    cleared_single_line_internal_rewind_split: bool,
    current_endnote_had_inline_object_vpos_overestimate: bool,
}
// pre_emitted_endnote_para_indices(컬렉션)는 &mut 별도 파라미터 (char_x_map 전례)
```
필드 구체 타입은 추출 시 컴파일러 확정(라운드 2~4 검증 절차).

### 추출 1 (2단계) — `typeset_endnote_paragraphs` (en_para 루프 통추출, 4,934줄)
- `&self` + `st: &mut TypesetState` + fn 파라미터 통과군 + 반복자 4종 + 프리앰블 산출
  참조군 + **읽기 스칼라/플래그 → `EndnoteEmitVars`**(Copy 묶음, 12개 초과 시 —
  RunEmitVars 전례) + `flow: EndnoteFlowState`(값) → `EndnoteFlowState` 반환.
- 이것 하나로 함수가 5,539 → **~600줄대** (프리앰블+마무리) — 라운드 1 이연분의 완결.

### 추출 2 (3단계) — 프리앰블 분리 (3916~4446, ~530줄)
- 구분자 방출/예산 준비를 `prepare_endnote_flow`(가칭)로 — 추출 1 후 재실측으로 축소
  여부 결정 (v2 §0 규칙 3).

## 3. 게이트/완료 기준

수행계획서 §4와 동일. 표적: issue_1116 등 미주 핀 + golden SVG + 페이지 오라클.
예상 효과: `typeset_section_endnotes` CC 179 → **두 자릿수** (전체 1위 5라운드 연속 해소),
전체 최대 CC는 `layout_partial_table`(163)로 이동 예상.
