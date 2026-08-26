# Stage 160: RowBreak split budget geometry 계약

## 목적

HWPX stored-layout의 `64px`와 native HWP5의 `2px` RowBreak split-row allowance를
고정 profile 상수가 아닌 cell unit, painted fragment, 저장 행 geometry의 차이로
판정한다.

## 분석 범위

- `HWPX_ROWBREAK_SPLIT_ROW_OVERFLOW_TOLERANCE_PX`
- `NATIVE_HWP5_ROWBREAK_ROUNDING_TOLERANCE_PX`
- split row의 content height, painted height, declaration/LINE_SEG geometry

## 금지 조건

- source profile마다 새로운 px 상수를 만들지 않는다.
- HWPX/HWP5 fixture, page index, 행·열 수를 조건에 추가하지 않는다.
- 분석 문서만 커밋하지 않는다.

## 완료 기준

- profile은 저장 계약 선택에만 쓰고 허용량은 실제 fragment geometry에서 구한다.
- 고정 64px 및 2px allowance를 제거한다.
- 구현과 결과 보고서를 같은 Stage 커밋으로 남긴다.

## 상태

완료.

## 분석 결과

- HWPX/LINE_SEG 부재 RowBreak의 `64px`는 split cut의 logical consumed height와 실제
  painted fragment height의 차이를 한 번에 덮는 profile allowance였다.
- native HWP5의 `2px`는 저장 행 높이가 fit하지만 browser 측정 행 높이가 미세하게
  넘는 경우를 허용하려는 값이었다.
- 둘 다 profile이 px 값을 결정하고 있어, 같은 source geometry라도 내용 높이에 따라
  과소 또는 과대 허용될 수 있었다.
- rebase로 들어온 `#4333` 공통 인라인 개체 줄높이 규칙은 typeset과 layout이 같은
  `row_cut_content_height` 물리 정의를 쓰게 한다. 이 규칙은 측정 일치성을 높이지만,
  저장 `row_height`만으로 실제 paint overflow를 허용할 근거는 제공하지 않는다.

## 구현

- 일반 RowBreak split candidate와 retry candidate는 별도 overflow allowance를 쓰지
  않는다. common line-height 정의로 계산한 `row_cut_content_height`가 page budget의
  정본이다.
- 저장 frame으로 선택된 terminal tail만 `split_candidate_rows_height - avail_for_rows`의
  실제 양수 초과분을 사용한다. retry는 같은 source frame budget을 재사용한다.
- native HWP5 whole-row fit에서 저장 `mt.row_heights[r]`만 fit하면 측정된 행 전체를
  허용하던 경로를 제거했다. 저장 frame tail은 기존의 source-tail 경로로만 허용한다.
- 1행 object declaration 및 rowspan의 stored/content 선택도 고정 64px 대신 선언·저장
  height와 측정 height의 실제 대소 관계를 쓴다.
- `HWPX_ROWBREAK_SPLIT_ROW_OVERFLOW_TOLERANCE_PX`와
  `NATIVE_HWP5_ROWBREAK_ROUNDING_TOLERANCE_PX`, 잔여
  `ROWBREAK_SPLIT_ROW_OVERFLOW_TOLERANCE_PX`를 제거했다.

## 검증

- 이번 Stage에서는 사용자 지시에 따라 build 또는 test를 실행하지 않았다.

## 결과

RowBreak split 예산은 source profile이 정한 고정 px 값이 아니라 common line-height
정의와 저장 frame evidence에서 계산된다. 일반 fragment는 paint padding을 allowance로
전환하지 않으며, source가 선택한 terminal frame만 정확한 실제 초과분을 쓸 수 있다.
