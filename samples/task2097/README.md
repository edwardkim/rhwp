# Task #2097 재현 샘플

## none_table_declared_fits.hwpx (합성)
- 출처: 수작업 합성 (`samples/tac-host-spacing.hwpx` 골격). 실문서 재현원은 hwpdocs
  정책연구 서식류 6건 (예: `1730000-201800001 새만금`, rhwp 2쪽 vs 한글 1쪽).
- 형상: 3행 표(pageBreak=NONE, 자리차지) — 선언 높이 69700HU(929.3px) ≤ 본문 933.6px,
  r1/r2 셀은 저장 높이(1200/500HU)보다 내용 실측이 커서 측정 합 946.2px 로 본문 초과.
  표 뒤 "AFTER TABLE" 문단 1개.
- 결함(수정 전): 측정 fit 실패 → 쪽나눔=None 임에도 행 분할(PartialTable rows 0..2/2..3),
  마지막 행이 2쪽으로 조각.
- 기대(한글 정합): 한글은 None 표를 행 컷하지 않고 실제 행높이 합 = 저장 선언 높이
  (1730000 COM 3자 비교: 한글 910.6px = 저장 910.5px vs rhwp 실측 954.1px).
  표는 1쪽 통째, AFTER TABLE 은 2쪽, 전체 2쪽.
- 검증: `rhwp dump-pages samples/task2097/none_table_declared_fits.hwpx` /
  `cargo test --test issue_2097_none_table_declared_fits`
