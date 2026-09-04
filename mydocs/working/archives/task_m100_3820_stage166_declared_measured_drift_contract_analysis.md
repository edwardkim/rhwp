# Stage 166: declared-measured drift의 source contract 분석

## 목적

선언 높이 whole-fit에 남아 있는 `declared_object_total * 10%` 및 `64px` 상한을 폐기하고,
declared object·stored frame·painted row footprint 사이의 실제 관계로 신뢰 범위를 결정한다.

## 분석 범위

- `declared_excess_within_drift`
- `declared_object_total`, `table_total`, `whole_fit_table_total`
- MeasuredTable 행 높이와 stored object height의 관계
- saved object bottom이 있는/없는 HWP·HWPX RowBreak 표

## 원칙

- 표 전체 크기에 대한 임의 비율이나 최대 px 값으로 font/layout drift를 추정하지 않는다.
- source frame이 paint footprint를 지지하면 해당 frame을 우선하고, 그렇지 않으면 row cut을
  사용한다.
- declared height가 단순 최소 셀 높이인 경우에는 actual row footprint를 억제하지 않는다.

## 완료 기준

- declared와 measured의 차이를 source ownership 또는 actual paint geometry로 설명한다.
- 고정 ratio/cap을 공통 predicate로 대체할 수 있을 때만 코드와 결과를 같은 Stage 커밋에 남긴다.

## 분석 결과

- `MeasuredTable`은 각 행을 stored `cell.height`에서 시작한 뒤, 실제 content/padding/nested
  footprint가 더 클 때만 확장한다. 따라서 `table_total - declared_object_total` 자체는
  browser metric drift와 source-owned row growth를 구별하지 못한다.
- `10%` 및 `64px` 상한은 이 두 경우를 표 전체 크기로 추정한 값이었다. 큰 stale-min cell과
  작은 폰트 metric drift가 같은 비율 구간에 들어갈 수 있으므로 공통 source 계약이 아니다.
- 텍스트 셀의 non-synthetic stored `lineSeg` 마지막 bottom이 declared `cell.height` 안에 있으면,
  source는 해당 텍스트를 cell box 안에 저장했다는 직접 근거를 제공한다. 반대로 rowspan,
  control, source lineSeg 부재는 독립 physical ownership을 가질 수 있어 measured-row path를
  유지해야 한다.

## 구현

- `declared_object_total * 10%`와 `64px` 상한을 제거했다.
- `table_declared_height_has_stored_cell_content_frame`을 추가했다. 이 predicate는 모든
  non-rowspan text cell이 control 없이 stored lineSeg를 가지며 그 bottom이 declared cell
  height 안에 있는 경우에만 true다. 빈 text cell은 자체 content expansion이 없으므로
  declared geometry를 유지한다.
- declared whole-fit은 기존 fragment/source-bottom/painted-footprint 가드에 더해 이 predicate를
  요구한다. source cell frame이 불완전하거나 실제 row ownership이 복잡한 표는 일반 row cut을
  사용한다.

## 결과

- declared와 measured의 차이를 표 전체 비율이 아니라 각 셀의 저장 frame으로 판정한다.
- 전체 export 및 test는 이 Stage에서 실행하지 않았다. 다음 Stage에서 native HWP5 near-anchor
  RowBreak의 `24px`/`16px` tolerance를 source anchor와 measured excess 관계로 분해한다.
