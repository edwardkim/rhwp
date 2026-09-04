# 구현계획서 — Task #1853

## 목표

`preceded_by_same_para_float` 술어를 같은 문단의 **진짜 flow 스택 float**으로 한정하여
PR #1844 이 유발한 표 통째-이월 +1쪽 회귀 3건을 해소한다(별표4 정합 유지).

## 단계

### Stage 1 — 술어 정정 (`src/renderer/typeset.rs`)

선행 항목의 소스 컨트롤을 `para.controls[control_index]` 로 조회하여
`is_para_topbottom_float`(`!tac && TopAndBottom && vert=Para`)인 표만 선행 float 로 센다.
- tac 캡션 상자 제외 → 156767631·78842 해소
- vert=용지(Paper/Page) 절대 앵커 제외 → 3143097 해소
- 기존 헬퍼 `is_para_topbottom_float`(float_placement.rs) 재사용 — 정의 일원화

### Stage 2 — 회귀 게이트 (`samples/` + `tests/issue_1853.rs`)

- fixture: `samples/issue1853_caption_precedes_body_split.hwpx` (실문서 78842, 285KB,
  pi=371 = tac 캡션 ci=0 + 본체 자리차지 표 ci=1)
- 테스트 2건:
  1. `body_float_splits_on_caption_page_not_deferred_whole` — 캡션 쪽에 본체 분할 시작
     (구조 단언, 페이지 드리프트 강건)
  2. `caption_over_deferral_does_not_add_a_page` — 총 52쪽 (회귀 시 53쪽)
- 게이트 유효성: 구(PR#1844) 바이너리에서 캡션 쪽(44)에 본체 없음·총 53쪽 확인 → 테스트가
  버그를 실제로 포착함을 검증

### Stage 3 — 검증

- 회귀 3건 dump-pages: 156767631→5, 78842→52, 3143097→3 (전부 목표 달성)
- 캘리브레이션 불변: 2448877 별표4 2쪽, float-stack-defer 2쪽
- 기존 테스트 무회귀: issue_1156/1488/1510/1549/1639/1663/1748 통과
- lib 단위 테스트, clippy -D warnings, rustfmt(변경 파일) clean

## 영향 범위

`typeset_block_table` 내 술어 1곳(선행 항목 필터)만 변경. 이월 발동 조건(row_count>1,
remaining 초과)·prefill·advance 로직은 불변. 술어를 **좁히기만** 하므로 기존에 정상 분할되던
문서에는 영향 없고, 과잉 이월되던 문서만 정상 분할로 복귀한다.
