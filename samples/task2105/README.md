# Task #2105 재현 샘플

## rowbreak_table_declared_fits.hwpx (합성)
- 출처: `samples/task2097/none_table_declared_fits.hwpx` 의 pageBreak=CELL(→IR RowBreak) 판.
  실문서 재현원은 hwpdocs `19378753_[별지 제12호서식] 기관명(밀양시…).hwp`
  (24×27 RowBreak 표, 선언 907.7px ≤ 본문 933.5px vs 실측 955.9px → rows 22..24
  sliver, rhwp 2쪽 vs 한글 1쪽).
- 형상: 3행 RowBreak 표 — 선언 69700HU(929.3px) ≤ 본문 933.6px, r1/r2 내용 실측
  팽창으로 측정 합 946.2px 초과. 표 뒤 "AFTER TABLE" 문단.
- 결함(수정 전): 측정 fit 실패 → 행 분할 → 마지막 행 sliver 2쪽 조각.
- 기대(한글 정합): RowBreak 는 나눔 허용이지 강제가 아님 — 선언 fit 시 통째 1쪽,
  AFTER TABLE 2쪽, 전체 2쪽.
- 검증: `rhwp dump-pages samples/task2105/rowbreak_table_declared_fits.hwpx` /
  `cargo test --test issue_2105_rowbreak_table_declared_fits`
