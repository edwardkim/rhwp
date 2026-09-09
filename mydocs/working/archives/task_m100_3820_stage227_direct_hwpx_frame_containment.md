# Stage 227 - 직접 HWPX 저장 frame의 선언 셀 수용성

## 목적

#3820의 HWPX RowBreak 저장 frame 판정이 #3930 Q5 응답 셀의 writer-local reset을
물리 fragment로 오인해 첫 응답 줄을 p283에서 밀어내는 회귀를 제거한다. 동시에
실제 다중 source frame인 1x1 목차 표는 세 물리 조각으로 유지한다.

## 회귀 추적

- `upstream/devel`의 `issue_3930_preserves_page_count_and_inherited_even_master_page`는 통과한다.
- 현재 브랜치의 실패를 이분 탐색한 결과 최초 회귀 커밋은
  `d6dd4347b fix: HWPX 저장 frame 표 조각 계약을 보존한다`다.
- 해당 커밋은 direct HWPX RowBreak cell의 저장 reset을 물리 fragment로 승격했고,
  이후 RowBreak short-row scanner가 raw reset을 별도로 읽으면서 single-reset
  local cursor까지 source owner로 취급하게 됐다.

## 구조 비교

- 383쪽 정본을 지키는 section 12, paragraph 16, `r5,c1`은 reset 전후 frame 합계가
  `26,280HU`이고 선언 셀 높이는 `29,176HU`다. source frame이 선언 box 안에 있다.
- Q5 응답 cell은 reset 전후 frame 합계가 `20,242HU`인데 선언 셀 높이는 `2,949HU`다.
  뒤의 여러 응답 문단이 local cursor를 이어 붙인 것이므로 물리 page frame이 아니다.
- 같은 구역의 1x1 목차 표는 `46,468 + 53,456 + 52,072HU`의 세 source frame이
  선언 셀 `159,642HU`를 채운다. 이 표의 두 reset은 실제 물리 frame이므로 Q5를
  고치려고 제거하면 Q27 이후 owner가 한 쪽 앞당겨진다.
- direct frame 승격과 landscape short-row bleed가 서로 다른 reset 의미를 쓰면
  single-reset local cursor가 다시 물리 경계가 된다. 두 경로가 같은 predicate를
  사용해야 한다.

## 수정

- single reset은 source frame 합계가 선언 셀의 대부분을 채우면서 선언 높이를 넘지
  않을 때만 direct HWPX physical fragment로 승격한다. 비율은 페이지·문단·표 ID가
  아니라 저장 source 기하와 선언 cell box의 일관성 검증이다.
- reset이 둘 이상인 직접 1x1 표는 다중 source frame의 합이 선언 box를 채우는
  목차와 같은 물리 continuation이므로 기존 fragment 경계를 보존한다.
- landscape short-row bleed도 같은 선언-cell 수용성 predicate가 참일 때만 막는다.
- `cell_units`, `stored_frame_cut_for_row`와 행 scanner가 서로 다른 reset 의미를
  쓰지 않도록 direct HWPX cell 판정을 `LayoutEngine` helper 하나로 통합한다.
- nested table 제외와 HWP5-origin 분리는 그대로 유지한다.
- 문단 번호, 표 ID, 페이지 번호, pixel tail allowance를 사용하지 않는다.

## 검증 계획

1. `issue_3930_hwpx_hwp_save_layout`: 2건 통과. Q5 첫 응답 줄은 p283, HWPX/HWP는
   모두 383쪽이다.
2. `issue_3820_rowbreak_rowspan_band`: 4건 통과.
3. `issue_2006_1790387_prep_pagination_pin`: 최신 HWP 2020 MCP PDF 기준 140쪽 pin 통과.
4. 코드와 이 문서를 하나의 Stage 227 커밋으로 고정한 뒤 전체 회귀 및 MCP 2020
   PDF 시각 대조를 수행한다.
