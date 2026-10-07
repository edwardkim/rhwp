# Task #2105 재현 샘플

## rowbreak_table_declared_fits.hwpx (합성 — 쪽 시작의 짧은 마지막 셀 반례)

- 출처: `samples/task2097/none_table_declared_fits.hwpx`의 pageBreak=CELL(→IR RowBreak) 판이다. 수동 원본은 보존한다.
- 셀68000/1200/500HU와 표69700HU를 수동으로 선언했지만 마지막 두 셀의 저장 줄1200HU·양쪽 안여백282HU는 선언 셀보다 크다. 선언 합이 본문에 들어간다는 이유만으로 실제 내용도 수용한다고 기대하지 않는다.
- [동일 입력 한컴2020 PDF](../../pdf/issue2097/rowbreak-fragment-start-original-2020.pdf)는2쪽이다.1쪽은 BIG/MID ROW,2쪽은 TAIL ROW EXPANDING/AFTER TABLE이다. 이전의 통째1쪽 기대는 이 독립 기준과 실제 표 하단에 맞지 않아 교정했다.
- [독립 한컴2020 재저장 HWPX](../issue2097/rowbreak-fragment-start-resaved-2020.hwpx)와 [해당 PDF](../../pdf/issue2097/rowbreak-fragment-start-resaved-2020.pdf)는 원본과 별도로 보존한다. 재저장은 표 높이69282HU·줄높이1000HU/vpos0을 저장했다. 이 대조군에서 빈 단의 빈 배열을 배경 도형만 있는 단으로 오인해12px 추가 예산을 허용하던 실제 결함이 검출됐다. 실제 도형이 하나 이상 있는 조건을 공통으로 소비하게 수정했다.
- 쪽 시작과 중간 쪽의 행 소유·실제 표 하단·뒤 글줄·누락/중복을 같은 정식 경계 검사로 확인한다. 수동 원본의 저장 글자 높이/셀 괘선 차이를 한컴 재조판과의 전체 일치로 보고하지 않는다.
- 과거 언급한 `19378753_[별지 제12호서식] 기관명(밀양시…).hwp` 실제 원본은 Mac 및 Windows의 지정 기준 자료 경로를 모두 검색했으나 확인하지 못했다. 그 문서의24×27 표 정합을 이 합성 입력의 통과로 대신하지 않으며 미검증으로 남긴다.
- 검증: `tests/issue_2105_rowbreak_table_declared_fits.rs`, `tests/cases/issue_2097_terminal_row_physical_ownership.rs`. 근거와 전후 증거는 [PR7382 개별 기록](../../mydocs/pr/archives/pr_7382_review.md)의 보정38을 따른다.
