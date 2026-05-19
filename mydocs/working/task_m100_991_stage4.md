# 4단계 보고서 — 쪽 분할 표 직후 문단 vpos 팬텀 해소

- 타스크: 로컬 task991
- 단계: 4/5
- 구현계획서: `task_m100_991_impl_v5.md`
- 작성일: 2026-05-19

## 1. 문제

13쪽 "나. 요구사항 목록" 제목이 표 바로 아래가 아니라 페이지 하단(y≈1047)에 위치. 13쪽은 쪽 분할 표 pi=200(13×3, 12→13쪽 분할)의 연속분으로 시작하며, 표(그려진 하단 y=630)와 다음 문단 pi=201(y=987) 사이에 ~357px 팬텀 공백이 있었다.

## 2. 원인

`layout.rs` 1차 패스 vpos 보정. 분할 표 호스트 문단 pi=200 의 LINE_SEG(`vpos=725470 lh=1400`)는 텍스트 줄 높이만 담고 표 높이를 반영하지 못한다. 다음 문단 pi=201 의 vpos(753641)는 한컴이 표 높이 포함해 인코딩한 값이라, lazy_base 산출 시 `vpos_end(pi=201) − prev_vpos_end(pi=200)` 차이에 표 높이가 통째로 들어가 표 높이만큼 추가 점프(sequential 로 이미 표를 지난 위치에서 이중 가산).

`prev_has_overlay_shape` 가드는 `Shape`·`Picture`만 다루고 표를 누락. 쪽 분할 표(`PartialTable`)에서만 호스트 LINE_SEG 가 실제 높이를 못 담아 팬텀이 심각하다.

## 3. 1차 시도와 정정

`prev_has_overlay_shape` 에 `Control::Table` 분기를 추가(모든 비-TAC TopAndBottom+Para 표)했더니 골든 테스트 `issue_157_page_1` 이 깨졌다 — issue-157 의 "(대리참석 위임)" 줄이 2.05px 이동. issue-157 은 **분할 안 된 표**(`PageItem::Table`)로, 이 경우 vpos 인코딩이 정합하여 2px 보정이 정상이었다.

→ 가드를 **직전 항목이 `PartialTable`(쪽 분할 표)일 때**로 좁혔다.

## 4. 수정

`src/renderer/layout.rs` 1차 패스 루프(`layout_column`):

- `prev_item_was_partial_table: bool` 추가 — 루프 끝에서 `matches!(item, PageItem::PartialTable { .. })` 로 갱신.
- vpos 보정 진입 조건에 `&& !prev_item_was_partial_table` 추가.
- 분할 표 직후 첫 문단은 vpos 보정을 건너뛰고 sequential 배치(정확히 그린 y_offset)를 신뢰.

분할 안 된 표(`PageItem::Table`) 직후 문단은 불변.

## 5. 검증

- 13쪽 "나. 요구사항 목록" y=1047 → **707.9**(표 직하)로 정상화. SVG 렌더 — 표, 제목, 다음 표가 연속 배치.
- 페이지 수 **181 불변**(연쇄 페이지 변동 없음).
- `cargo test --release` 전체 **1482 passed, 0 failed** — 골든 SVG issue-157 포함 통과.
- `cargo clippy --release` 경고 0.

## 6. 다음 단계

5단계: 최종 결과보고서 + WASM 재빌드 영향 확인 + `orders/20260519.md` 갱신.
