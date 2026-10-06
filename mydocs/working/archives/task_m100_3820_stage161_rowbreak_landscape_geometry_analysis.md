# Stage 161: landscape RowBreak geometry 계약

## 목적

landscape RowBreak에 남아 있는 whole-row 및 short-row tolerance 상수
(`36/48/260/320px`)를 source profile 선택이 아닌 실제 viewport, row height, stored frame,
fragment geometry로 분해한다.

## 분석 범위

- `LANDSCAPE_ROWBREAK_WHOLE_ROW_TOLERANCE_PX`
- `HWPX_LANDSCAPE_ROWBREAK_WHOLE_ROW_TOLERANCE_PX`
- `LANDSCAPE_ROWBREAK_SHORT_ROW_TOLERANCE_PX`
- `HWPX_LANDSCAPE_ROWBREAK_SHORT_ROW_TOLERANCE_PX`
- short-row height 상수와 landscape body viewport의 관계

## 원칙

- HWPX/native profile은 저장 source 의미를 선택할 수 있으나 px budget을 직접 정하지 않는다.
- 특정 문서, 페이지, 행·열 수 또는 fixture를 조건으로 추가하지 않는다.
- whole-row와 short-row의 물리 fragment 범위를 서로 혼동하지 않는다.

## 완료 기준

- 각 tolerance가 실제 source/viewport geometry에서 유도되는지 판정한다.
- 일반 규칙으로 치환할 수 있는 경우 코드와 결과 보고를 같은 Stage 커밋에 남긴다.
- 분석 문서만 커밋하지 않는다.

## 분석 결과

- 과거 `260/320px` short-row 허용은 source frame이나 table geometry가 아니라 HWP/HWPX
  profile로 갈라진 고정 예산이었다. #2291 분석에서 이 예산은 rowspan 행에 발동했을 때
  페이지당 약 `260px`를 과적재해 24쪽이어야 할 표를 19쪽으로 축소한 원인이었다.
- `36/48px` whole-row 허용도 같은 profile 고정값이었다. landscape 판별마저 body 높이
  `700px`로 고정되어 페이지 실제 방향을 보지 않았다.
- continuation의 반복 머리행은 `header_overhead`로 실제 행 높이와 cell spacing을 사용해
  계산된다. 행 경계를 넘길 수 있는 유일한 공통 기하는 후보 행 앞의 실제 `cs_before`다.
  이는 반복 머리행과 다음 행이 공유하는 table boundary이며, 행 자체나 임의 페이지 reserve를
  다시 허용하는 근거가 되지 않는다.

## 구현

- `LANDSCAPE_ROWBREAK_*` 및 `HWPX_LANDSCAPE_ROWBREAK_*`의 six pixel reserve를 모두
  제거했다.
- landscape는 `body_area.width > body_area.height`로 판정한다.
- RowBreak continuation은 반복 머리행, full pure row, 저장 frame reset 없음 조건을 유지하고,
  후보 행 시작이 일반 row budget 안에 있으면서 후보 끝이 실제 `cs_before`만큼만 넘을 때만
  현재 fragment에 둔다. 한 행 높이 또는 HWP/HWPX profile에서 추가 reserve를 만들지 않는다.
- rowspan 행은 whole-row/short-row 구분 없이 이 경로에서 제외한다.

## 결과

- 이번 Stage는 기존 임시 profile tolerance를 공통 table geometry로 교체했다.
- 전체 export 및 test는 이 Stage에서 실행하지 않았다. 다음 Stage에서 2025 편람 HWP/HWPX와
  landscape RowBreak 회귀 fixture를 별도 검증 범위로 확인한다.
