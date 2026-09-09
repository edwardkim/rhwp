# 단계 완료 보고 — Task M100 #2067 2단계: 추출 1+2 (justify + shape_markers)

- 작성일: 2026-07-08 / 브랜치: `local/task2067`

## 수행 내용 (동작 불변, 순수 함수 2건 승격)

- **추출 1 `compute_line_extra_spacing`** (2515~2659 표현식 + 전용 클로저
  `count_dash_leaders` 21줄 통이동): 정렬별 여분 간격 `(word, char, dash)` 계산.
  free fn, 파라미터 12개(Copy 스칼라 9 + &comp_line/&styles + `in_cell: bool` —
  `cell_ctx.is_some()` 치환 1곳, 구현계획서 설계 그대로). 변이 0·외부 탈출 0 확인.
- **추출 2 `collect_shape_marker_labels`** (2815~2877): 조판부호 마커 라벨 수집.
  파라미터 2개(show_ctrl, para). 루프 불변이지만 호출 위치 유지(동작 불변 원칙).
- `ComposedLine` import 1건 추가. 사각지대 체크리스트 6종 재점검 — 신규 해당 없음.

## 게이트 (전수 통과)

fmt ✓ / clippy **0** / `--tests` **2,944/0** / issue_1116 **13/13** /
OVR 5샘플 회귀 **0건** (00014ecf ×4 + a05e6f1b).

> 게이트 스크립트 정정 1건: OVR 검증을 임의 테스트명이 아닌 manifest
> (`task_m100_1904_baseline_manifest.md` §5)의 정식 명령
> (`tools/object_visual_regression.py --baseline`)으로 실행하도록 복원.

## 계측

| 함수 | 시작 (r7) | 현재 |
|---|---|---|
| `layout_composed_paragraph` | 2,093줄 · 분기 365 | **1,876줄 · 분기 304** |
| 신규 `compute_line_extra_spacing` | — | 178줄 · 분기 37 |
| 신규 `collect_shape_marker_labels` | — | 48줄 · 분기 24 |

## 다음 단계

3단계 — 추출 3a+3b (TAC 그림/양식 107줄, x 값 왕복 + &mut reserved height).
