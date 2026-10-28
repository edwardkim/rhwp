# Stage 162: RowBreak 고정 px 예산 전수 감사

## 목적

Stage 161에서 제거한 landscape profile reserve와 같은 성격의 고정 px 예산이 RowBreak
fragment의 owner 또는 fit 결정을 다시 왜곡하지 않는지 확인한다.

## 분석 범위

- `typeset.rs`의 `*_TOLERANCE_PX`, `*_ALLOWANCE_PX`, `*_RESERVE_PX`와 이에 준하는
  고정 px 조건
- RowBreak 표의 whole-row, split-row, stored frame, footer/body fit 경로
- source layout, cell spacing, stored bounds, viewport geometry에서 직접 유도 가능한 값인지

## 원칙

- 문서, 페이지, 표 식별자, 행·열 번호, HWP/HWPX profile로 px budget을 선택하지 않는다.
- 고정 상수가 실제 표현 가능한 최소 단위가 아니라면 source 또는 paint geometry predicate로
  치환한다.
- 일반적인 부동소수점 비교 오차와 문서별 layout reserve를 혼동하지 않는다.

## 완료 기준

- 각 고정 px 값의 역할과 source 근거를 분류한다.
- layout reserve 성격인 값은 공통 geometry로 교체하고, 해당 코드와 결과 문서를 같은 Stage
  커밋에 포함한다.
- 이 Stage도 분석 문서만으로 커밋하지 않는다.

## 분석 결과

- `BOTTOM_SQUEEZE_TOLERANCE_PX=13`, `BOTTOM_SQUEEZE_MAX_REST_PX=100`,
  `BOTTOM_SQUEEZE_MIN_HEADROOM_PX=12`는 whole row, rowspan block, partial row cut에
  공통으로 적용되어 있었다.
- 해당 경로의 기존 주석은 같은 `잔여/초과/콘텐츠` 수치가 한컴에서 서로 반대의 결과를
  보인다고 기록한다. 즉 세 값은 source layout이나 paint geometry에서 유도된 predicate가
  아니며, 일반 RowBreak 페이지네이션에 적용할 근거가 없다.
- `advance_row_cut`, stored-frame strict boundary, row content height는 이미 물리 fragment를
  결정하는 공통 경로다. 근거 없는 squeeze가 이 결정을 덮어쓴 경우, 한 행 또는 한 block이
  body bottom을 넘겨 적재될 수 있다.

## 구현

- `BOTTOM_SQUEEZE_*` 세 상수와 full row, rowspan block, partial row cut의 세 적용 경로를
  제거했다.
- 이제 행 또는 block이 일반 budget에 맞지 않으면 기존 `advance_row_cut`/다음 fragment
  경로가 owner를 결정한다. stored frame, 중첩 표, rowspan 행에 별도 px budget을 주지 않는다.

## 잔여 감사 대상

- trailing empty spacer의 `40px` 허용은 source-empty row의 paint border와 stored row height를
  함께 비교하는 별도 공통 계약이 필요하다.
- declared-height trust, near-anchor resync, mixed nested owner drift 값은 각각 다른 source
  좌표계 계약을 가지므로 다음 Stage에서 한 그룹씩 분석한다.
- `0.1/0.5px` 비교는 layout 좌표 반올림을 위한 일반 허용인지, layout reserve인지 분리해서
  재검토한다.

## 결과

- 판별 근거가 반증된 하단 squeeze reserve는 RowBreak 세 경로에서 모두 제거됐다.
- 전체 export 및 test는 이 Stage에서 실행하지 않았다. 다음 Stage에서 2025 편람 HWP/HWPX와
  RowBreak fixture의 page/fragment 변화를 별도 검증한다.
