---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 53 — issue1891 direct HWPX 보정 범위

## 시작 조건

- 시작 commit: `5eecffce2`
- [Stage 52](task_m100_3820_stage52_full_regression_and_native_gates.md)의 전체 integration은
  마지막 `overflow_cell_baseline`에서만 실패했다.
- 유일한 증가: `samples/issue1891_external_bindata_link.hwpx` 34→56줄
- 현재 페이지별 증가 신호: p6 1, p27 5, p47 6, p70 44로 합계 56줄

## 분석

Stage 51은 #3637의 31쪽과 p26→p28 source owner를 함께 복원하기 위해 direct HWPX에서
canonical `CellUnit`의 높이와 세분성을 재사용하되 HWP5 hard/stored cursor를 제거했다.
그러나 이 분기가 direct HWPX의 모든 1×1 중첩 표에 적용되어, 저장 reset이 없는 표도
PR #4122 이전 scalar 측정에서 벗어났다.

issue1891의 p70은 3단계 셀 경로 아래 여러 하위 셀 줄이 같은 페이지 하단 밖으로 밀리며,
Stage 51 적용 뒤 추가된 22줄과 일치한다. 이 문서에서 helper에 들어온 1×1 wrapper들은
저장 reset이 모두 0개였다. #3637의 핵심 wrapper는 1740.6px로 body 971.3px보다 크고
단일 reset이 있으며, 다른 wrapper에는 reset이 2개다. 따라서 direct HWPX canonical 높이
투영은 반복 reset 또는 물리 multi-page 단일 reset 표에만 필요하다.

## 수정

1. direct HWPX의 canonical-height/scalar-cursor 변환을 반복 저장 reset(2개 이상),
   authoritative reset, 또는 물리 한 쪽을 넘는 단일 reset 표에만 적용한다.
2. reset이 없는 중첩 표와 물리 한 쪽에 못 미치는 단일 reset 표는 검증된 legacy scalar
   fallback을 유지한다.
3. `issue_1891`에 전 페이지 `overflowCellLines <= 34` 집중 회귀를 추가한다.
4. issue1891, #3637, 59043, issue2007 focused 회귀와 전수 overflow gate를 먼저 실행한다.
5. 통과하면 새 Stage에서 전체 integration, full Clippy, native Skia를 재개한다.

최초에는 반복 reset(2개 이상)만 canonical 투영 대상으로 제한했으나, 이 조건은 #3637의
단일 reset·물리 multi-page wrapper를 legacy 경로로 되돌려 31→32쪽 회귀를 만들었다.
두 fixture의 구조를 함께 추적해 다음 최소 조건으로 확정했다.

```text
stored reset >= 2
또는 authoritative reset
또는 stored reset == 1 && nested table height > page body height
```

이 조건은 #3637의 1740.6px wrapper(body 971.3px)를 포함하지만, reset 0개이고
908.4px인 issue1891 p70 wrapper는 제외한다. direct HWPX의 canonical 단위에서는
HWP5 전용 hard/stored cursor를 계속 제거하므로 높이·세분성만 재사용한다.

`issue_1891_external_link_overflow_cell_lines_do_not_grow`를 추가해 전 페이지 렌더 뒤
`LAYOUT_OVERFLOW_CELL` 합계가 기존 상한 34줄을 넘지 않도록 고정했다.

## 검증 결과

- `parent_projection` 단위 회귀: 2 passed
- `issue_1891`: 4 passed, 0 failed
- `issue_3637_nested_table_starts_inside_parent_cell`: 31쪽, passed
- `issue_3637_para_topbottom_vpos_base`: passed
- `issue_3637_split_cell_nested_table_vpos`: passed
- `issue_1921_59043_pagination_pin`: 5 passed, 0 failed
- `issue_2007_nested_cell_pagination`: 15 passed, 0 failed
- `overflow_cell_baseline`: 1 passed, 0 failed, 110.18s

```text
overflow-cell 스윕: 샘플 675건(스킵 3) / 0 아닌 문서 17종 / 총 691줄
test result: ok. 1 passed; 0 failed
```

Stage 52에서 713줄로 증가했던 issue1891의 22줄이 제거됐고, #3637의 31쪽 계약과
59043·issue2007 집중 회귀도 유지됐다. 전체 integration·Clippy·native Skia는 다음
Stage에서 동일 체크포인트를 대상으로 실행한다.
