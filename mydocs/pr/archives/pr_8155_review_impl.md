# PR #8155 구현 검토 — 중첩 overlay 가로 앵커

## 적용된 변경

- `src/renderer/layout/table_layout.rs` 셀의 `Control::Table` 분기에서 중첩 overlay의 `inline_x_override`를 제거했다.
- `ctrl_area.x`는 overlay일 때 셀 단 원점, 비 overlay일 때 기존 텍스트 pen을 포함한다.
- 중첩 `layout_table`이 기존 `compute_table_x_position`의 바깥여백·정렬 처리를 소비한다. 이후 paint의 별도 보정은 없다.
- 측정·세로 원점·페이지 흐름·분할 컷·예약 높이는 기존 계약을 유지한다.

## 독립 기대값과 검사

원본 소유 정보는 section 3 / paragraph 71 / table 1723619577 / r2c0 paragraph 4의 도장 3개다.
원본 왼여백 283HU와 offset 4868/13553/22830HU, 4251HU 정사각형을 확인했다.
Print의 왼쪽 선 중심 139.289/225.956/318.616pt와 윗변 271.654pt를 96dpi로 환산해 ±2px로 검사한다.
기존 `issue_3820_hwpx_behind_text_stamp_placeholders_keep_common_y_and_offsets`는 크기 기반 선택 대신 원본 소유로 식별하며,
수량·공통 y·원본 크기 계약을 유지했다. 신규 `nested_overlay_left_margin_is_consumed_once_without_advancing_flow`는
왼여백 0 대조군의 x 이동량을 원본 283HU와 대조하고 y·크기·쪽수·순서·비겹침을 확인한다.
동일 소스 base FAIL(exit 101) / 후보 PASS(exit 0), 정식 자동 suite 기존 2 PASS·신규 1 PASS다.

## 검증 연결

- [정확한 code CI와 전체 회귀·공통 준수 판정](pr_8155_review.md).
- [원본·Print·최종 TSV·실제 PNG·producer](../assets/issue_8151/README.md).
- [전후 변화와 작은 raw 하락의 독립 분석](../assets/issue_8151/page-change-analysis.md).
- [명령·결과](../assets/issue_8151/validation-commands.md).

이번 144쪽 완료와 전쪽 100% 목표를 구분한다. 기존 저장 왕복·바탕쪽·내용 소유 검사는 좌표 검사로 대체하지 않는다.
