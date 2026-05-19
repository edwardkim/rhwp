# 구현계획서 v5 — 쪽 분할 표 직후 문단 vpos 팬텀 (4단계 추가)

- 타스크: 로컬 task991
- 선행: `task_m100_991_impl_v4.md` (3단계 — 1행 tac 표 분할 금지, 완료)
- 작성일: 2026-05-19

## 배경

작업지시자 확인: 13쪽 "나. 요구사항 목록" 제목이 표 바로 아래가 아니라 페이지 하단(y≈1047)에 위치한다.

## 문제

13쪽은 쪽 분할된 표 pi=200(13행×3열, 12→13쪽 분할)의 연속분으로 시작한다. 표는 y=630에서 그려져 끝나는데, 직후 문단 pi=201이 y=987에서 시작 — 표와 다음 문단 사이 **~357px 팬텀 공백**.

## 원인

`layout.rs` 1차 패스의 vpos 보정:

- 분할 표 호스트 문단 pi=200 의 LINE_SEG 는 `vpos=725470, lh=1400`(텍스트 줄 높이) — 표 높이(렌더 ~901px)를 반영하지 못한다.
- 다음 문단 pi=201 의 vpos(753641)는 한컴이 표 높이를 포함해 인코딩한 절대 위치다.
- vpos 보정의 lazy_base 산출이 `prev(pi=200) 의 vpos_end`(727710, 표 높이 미반영)를 쓰므로, `vpos_end(pi=201) − prev_vpos_end` 차이에 **표 높이가 그대로 들어가** 다음 문단이 표 높이만큼 추가 점프한다(이미 sequential 로 표를 지난 위치에서 또 점프 → 이중 가산).

기존 `prev_has_overlay_shape` 가드(개체 높이가 vpos에 포함돼 과대보정되는 것을 막음)는 `Shape`·`Picture`만 다루고 표는 누락. 다만 표는 **쪽 분할된 경우(PartialTable)**에만 호스트 LINE_SEG 가 실제 높이를 못 담아 팬텀이 심각하다(분할 안 된 표는 vpos 인코딩이 정합 — issue-157 의 2px 보정은 정상).

## 수정

`layout_column`(1차 패스 루프)에 `prev_item_was_partial_table` 플래그 추가:

- 루프 끝에서 `matches!(item, PageItem::PartialTable { .. })` 로 갱신.
- vpos 보정 진입 조건을 `!prev_has_overlay_shape && !prev_item_was_partial_table` 로 강화.
- 분할 표 직후 첫 문단은 vpos 보정을 건너뛰고 sequential 배치를 신뢰(표를 정확히 그린 y_offset).

분할 안 된 표(`PageItem::Table`) 직후 문단은 불변 → issue-157 등 회귀 없음.

## 검증

- ☞: 13쪽 "나. 요구사항 목록" y=1047 → 707.9(표 직하)로 정상화. SVG 렌더 정합.
- 페이지 수 181 불변(연쇄 없음).
- `cargo test` 전체 1482 passed, 0 failed. `cargo clippy` 경고 0.
- 골든 SVG issue-157 통과(분할 안 된 표 케이스 — 가드 미적용).

## 단계

- 4단계(본 계획): 위 수정 — `task_m100_991_stage4.md` + 커밋.
- 5단계: 최종 결과보고서 + WASM 재빌드 영향 확인 + `orders/` 갱신.

## 범위

- 포함: 쪽 분할 표 직후 문단의 vpos 보정 건너뛰기.
- 제외: 페이지 수 ±쪽 누적 드리프트, HWP3, 파서.
