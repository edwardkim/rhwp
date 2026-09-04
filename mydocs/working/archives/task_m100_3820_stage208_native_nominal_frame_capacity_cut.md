# Task M100 #3820 Stage 208 - native HWP5 nominal frame capacity cut

## 문제

전체 회귀에서 `issue_1891_hwp5_origin_hwpx_export_reparse_keeps_page_count`가
`samples/80250_regulatory_analysis.hwp` 원본을 18쪽으로 계산했다. HWP 2020
기준과 기존 계약은 17쪽이며, 내보낸 HWPX 재파싱은 이미 17쪽이었다.

## 관찰

- 원본 HWP의 `pi=15` 2x2 RowBreak 표는 페이지 4에서 머리행만 배치하고, 본문 행은
  페이지 5부터 `[1, 29]`, `[1, 59]`, tail 순으로 세 fragment를 만들었다.
- 같은 문서의 현재 내보낸 HWPX는 페이지 4에 `[1, 28]` 첫 본문 fragment를 배치해
  표를 두 continuation으로 끝내고 다음 본문을 한 페이지 앞에서 시작했다.
- 원본 첫 fragment의 `common.height`는 머리행 가까이의 명목 표 높이여서, 전체
  물리 fragment 경계로 쓰면 안 된다. 다만 그 경계 바로 다음 본문 행의
  `advance_row_cut`은 실제 `CellUnit` 경계에서 논리 예산을 정확히 초과했다.

## 보정 계약

native HWP5 RowBreak 표에서 저장 첫 frame의 가장 가까운 행 끝이 현재 본문 행의
직전일 때만, 첫 `CellUnit` capacity cut이 선택한 정확한 초과분을 허용한다. 선언
frame의 남은 높이를 일반 허용치로 넓히지 않으며, continuation, 다른 행, 편집 후
reflow 표에는 적용하지 않는다.

## 검증 대상

기존 `tests/issue_1891.rs`의 HWP5 원본/내보낸 HWPX 왕복 17쪽 계약이 이 결함을
직접 검출한다. 후속 focused 회귀에는 `issue_1695`, `issue_1733`, 그리고
`issue_3820_rowbreak_rowspan_band`를 함께 포함한다.
