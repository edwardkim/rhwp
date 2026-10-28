# #2015 Stage 3 완료보고서 — 부동 RowBreak 표 vert_offset 이중계상 수정

- 이슈: #2015 / 브랜치: `fix/2015-saved-bounds-rowbreak-overflow`
- 범위: 발원지 ①(부동 tac=false RowBreak 표 91.2px 오버플로우) 수정. 발원지 ②(HWPX 인라인 표
  합성 줄 피치)는 후속.

## 수정 내용

pre-emit 된 host 텍스트가 `current_height` 를 `para_start → para_start+host_h` 로 전진시킨 뒤,
typeset 예산과 layout 배치가 각각 `vert_offset`(para_start 기준)을 **재차감/재적용**해 host_h 만큼
이중계상되던 것을 보정. 표의 참 오프셋은 current_height 기준 `vert_off − host_h`.

### 데이터 흐름 (host_h 전파)
- `TypesetState.pre_emitted_host_heights: HashMap<usize,f64>` 추가, `pre_emit_visible_rowbreak_host_text`
  에서 host_h 기록.
- `PaginationResult.pre_emitted_host_heights` 로 전달 → `LayoutEngine.set_pre_emitted_host_heights`.

### 보정 지점 (typeset ↔ layout 정합)
- `typeset.rs` `vert_offset_overhead`: `(vert_off − host_h).max(0)` — page_avail 정정.
- `table_partial.rs` `layout_partial_table` 첫 fragment: `(vert_off − host_h).max(0)` — 표 y 정정.
- host pre-emit 아니면 `host_h=0` → `(vert_off−0)` 종전과 동일 → **비대상 문서 회귀 0**.

### 파일
`typeset.rs`, `pagination.rs`, `pagination/engine.rs`, `layout.rs`,
`layout/table_partial.rs`, `document_core/queries/rendering.rs`.

## 결과 (base=origin/devel 대비)

| 지표 | 수정 전 | 수정 후 |
|---|---|---|
| `LAYOUT_OVERFLOW` para=52 | **91.2px** | **2.1px** (엔진 tolerance 수준) |
| 표 top (render tree, p4) | y=1056.4 (host 끝+vert_off) | **y=912.1** (host 끝 직후) |
| HWPX end_cut | `[1]` | **`[3]`** (= HWP 참조·PDF) |
| 페이지 수 | 5 | 5 (유지) |
| 시각 오라클 p5 ink_match | 11.62% | **13.12%** |
| 시각 오라클 p4 ink_match | 13.87% | 13.83%* |

*p4 총 ink 는 상단 인라인 표 pi=50(발원지 ②, 미수정)의 드리프트가 지배 → 하단 pi=52 개선이
집계에 덜 반영. overlay(`oracle_fixed/review_004.png`)에서 하단 "< 사회기여 봉사활동 아이디어 >"
박스가 rhwp/PDF 정렬됨을 시각 확인(수정 전 완전 분리·오버플로우).

## 잔여 2.1px 판단
HWPX end_cut=[3] 이 HWP 저장 LINE_SEG 참조([3]) 및 한컴 PDF 와 동일한 컷. 잔여 2.1px 는 마지막
유닛이 경계에 걸치는 행높이 측정 드리프트(엔진 `ROWBREAK_SPLIT_ROW_OVERFLOW_TOLERANCE=2.0px`
수준)로, 컷 오류가 아니다.

## 회귀 검증
- 전체 테스트: **2923 passed / 0 failed** (210 바이너리, release-test, --no-fail-fast).
- `issue_1749_saved_bounds_page_break`: `issue_1811_hwpx_pi52_rowbreak_cut_matches_hwp_reference`
  의 HWPX 기대값을 `[1]→[3]` 으로 교정(종전 `[1]` 은 이중계상 버그값. 주석에 #2015 근거 명시).
- `issue_1035_alignment`, `issue_1139_inline_picture_duplicate` 등 유지.
- 형제 샘플 `saved_bounds_cumulative_vpos`(HWPX/HWP): 2쪽·overflow 0 유지.
- 신규 `issue_2015_page4_rowbreak_table_stays_in_body`: 통과(≤5px 게이트, 91px 회귀 방지).
- `cargo clippy --all-targets -- -D warnings`: 통과(무경고).

## 다음
Stage 4: 발원지 ②(HWPX 인라인 표 pi=50 합성 lineSeg 줄 피치) — p4 상단 드리프트. 별도 조사 후 진행.
