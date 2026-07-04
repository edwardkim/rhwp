# Task #1772 재현 샘플

## table_outer_margin_common_sync.hwpx
- 출처: 서울 정보소통광장 정보공개 결재문서(공개) 36381023 — opengov 결재문서 계열,
  PII 방침 A(그대로 동결).
- 특성: 쪽 고정(vert=Page) 자리차지(TopAndBottom) 헤더 표 + `outMargin bottom=852`(3mm).
- 결함(수정 전): HWPX 파서가 outMargin 을 `table.outer_margin_*` 에만 기록하고
  `table.common.margin` 을 0 으로 방치 → 예약 하단 계산(`calc_shape_bottom_y`,
  common.margin 참조)에서 아래 여백 누락 → 본문 첫 줄 y=295.4px (표 하단에 밀착).
- 기대(한글 정합): 본문 첫 줄 y≈306.7px — 저장 lineseg pi=0 vpos=17478(= 상단여백
  75.6px + 233.0px)과 일치. 동일 문서의 HWP5 재파스본(어댑터가 common.margin 동기화)도
  306.7 로 렌더됨.
- 검증: `cargo test --test issue_1772_table_outer_margin_sync` /
  `rhwp export-render-tree samples/task1772/table_outer_margin_common_sync.hwpx -p 0`
