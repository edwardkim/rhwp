# Task M100 #1785 최종 결과보고서 — 셀 안 여백 선택 규칙 단일 출처 통일

## 요약

#1772 수정 후 잔여 9.25px 이동 군집(#1785)의 원인은 **height_measurer 와 레이아웃
(resolve_cell_padding)의 셀 안 여백 선택 규칙 불일치**였다. 규칙 본체를
`Cell::use_cell_padding_axis`/`effective_padding` 으로 추출해 두 소비자가 공유하도록
통일했다. 조사 상세: `mydocs/tech/task_m100_1785_root_cause.md` (+ 본 보고서의 접근 변경).

## 결함 메커니즘 (최종)

- 레이아웃 규칙(한컴 정합 튜닝): aim=true → cell.padding(0 은 표 기본 폴백),
  aim=false → table.padding, **단 레거시 보존값(cell > table, <2500HU)은 aim=false 여도
  셀 값 사용**.
- height_measurer 는 단순 `aim ? cell : table` — 레거시 보존값 케이스에서 레이아웃과
  어긋남.
- 36381023 발신명의 표(45열 micro-grid, table.padding=0, cell.padding=140, aim=false):
  레이아웃은 140 으로 그리는데 측정은 0 → 예약 높이 과소. HWPX→HWP 어댑터가 micro-grid
  계약으로 width_ref bit0 을 세팅하면 재파스본은 aim=true 가 되어 측정도 140 → A/B 측정
  차 9.3px = render-diff 9.25px 군집.
- 어댑터의 bit0 계약 자체는 문제가 아님 (규칙 통일 후 자연 정합).

## 변경 사항

- `src/model/table.rs`: `Cell::use_cell_padding_axis`, `Cell::effective_padding` 신설.
- `src/renderer/layout/table_layout.rs`: `should_use_cell_padding_axis_for_context` 가
  규칙 본체를 위 함수로 위임 (동작 불변).
- `src/renderer/height_measurer.rs` 행 높이 계산: **aim=false 만** 레거시 보존값 규칙으로
  통일 (결함 수정 지점). aim=true 는 기존대로 cell.padding 을 0 포함 존중 — 전면 통일은
  `aim=true && 0 → 표 기본 폴백` 의미를 측정에 옮겨와 #493 세로 Shift 리사이즈
  (셀보호2.hwp 셀[20] aim=true pad top/bottom=0)를 깨뜨렸다 (3차에서 축소).
- `tests/issue_1785_cell_padding_rule_consistency.rs`: 규칙 단위 테스트 + 36381023
  라운드트립 표 기하 보존 통합 테스트.

## 검증 결과

| 항목 | 결과 |
|------|------|
| 신규 테스트 2건 (규칙/라운드트립 기하) | 통과 |
| 재현 파일 seoul_071 render-diff | 9.25px OVER → **0.00px PASS** (잔여 3.73px 포함 해소) |
| 코퍼스 300건 (hwpx) | OVER 10→8, 악화 0 |
| 코퍼스 2,500건 (hwpx) | **OVER 55→28, PASS 2404→2432, 개선 34·악화 0** |
| 코퍼스 300건 (hwp) | 변화 없음 |
| 전체 `cargo test` (#493 리사이즈 18건 포함) | 통과, 실패 1건은 기존 이슈* |
| clippy (`--release --lib`) | 경고 없음 |

\* `issue_852::form_01_keeps_nine_cfb_streams` — Windows CFB 경로 구분자 문제 (#1775,
origin/devel 동일 실패, 본 타스크 무관).

## 교훈 (기각된 접근 기록)

- 1차(어댑터 유효 패딩 물질화): 300건 배치 OVER 10→121 즉시 회귀 — 측정-렌더 규칙
  불일치라는 진짜 원인을 어댑터 보정으로 덮으려 했기 때문.
- 2차(측정을 layout 규칙으로 전면 통일): `aim=true && 0 → 표 기본 폴백`까지 옮겨와
  #493 세로 Shift 리사이즈 회귀 (`issue_493_cell_attrs` 1건 실패).
- 대규모 코퍼스 배치 + 전체 테스트 게이트가 두 차례 잘못된 방향을 각 1회 실행으로
  걸러냈다.

## 잔여

- 셀 내부 TextLine 3.73px 소결함으로 지목했던 것도 본 수정으로 함께 해소 (규칙 불일치의
  다른 표면이었음).
- layout `resolve_cell_padding` 의 `aim=true && 0 → 표 기본 폴백`(코드)과 주석("0도
  존중")의 모순 + 측정과의 잔여 불일치 — 별도 조사 항목.
- 나머지 OVER 27건은 별개 원인 — 후속 조사 대상.
