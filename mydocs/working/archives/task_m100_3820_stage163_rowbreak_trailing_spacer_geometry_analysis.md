# Stage 163: RowBreak 마지막 빈 spacer의 stored geometry 계약

## 목적

RowBreak 표의 마지막 source-empty spacer 행에 남아 있는 `40px` overflow 허용을 폐기하고,
blank row의 stored height, border paint, 현재 fragment body bottom으로 owner를 판정하는
공통 규칙을 찾는다.

## 분석 범위

- `ROWBREAK_TRAILING_EMPTY_ROW_OVERFLOW_TOLERANCE_PX`
- `row_is_empty_trailing_spacer`의 source-empty 판정
- 마지막 행의 declared/stored height와 실제 border footprint
- partial-table 마지막 fragment의 page/column bottom

## 원칙

- 빈 행이라는 사실만으로 임의 높이를 이전 fragment에 흡수하지 않는다.
- 문서, 페이지, 표 ID, 행 번호, HWP/HWPX profile을 조건으로 사용하지 않는다.
- 텍스트가 없더라도 table border가 paint되는 경우에는 그 물리 height를 예산에서 제외하지 않는다.

## 완료 기준

- source-empty spacer가 실제로 가시 border를 갖는지와 stored row height의 권위를 분리한다.
- 고정 px reserve를 source/paint geometry predicate로 대체할 수 있을 때만 구현한다.
- 분석, 코드, 결과 문서를 같은 Stage 커밋에 남긴다.

## 분석 결과

- `row_is_empty_trailing_spacer`는 해당 행의 문단 텍스트와 control이 비어 있는지만
  확인한다. stored row height, cell border/fill, row spacing, partial-table paint footprint는
  판정하지 않는다.
- 따라서 source-empty는 “텍스트 잉크 없음”일 뿐 “행이 paint되지 않음”을 뜻하지 않는다.
  `40px` 이내라는 조건으로 행 전체를 이전 fragment에 넣으면, 행의 border 또는 declared
  height를 body bottom 밖으로 보내는 결과가 될 수 있다.
- Stage 132에서 383쪽 수에 기여했던 `pi=030/056/074`의 continuation-only tail은 모두
  같은 6×5 RowBreak 구조와 `outer_margin_bottom=566HU`를 가졌지만, 이 관찰은 어떤
  source-empty spacer에도 적용할 공통 paint 계약을 증명하지 못한다.
- 반대로 앞선 visible response에는 `terminal_response_before_empty_spacer`와 stored tail
  cut 경로가 이미 있다. 이 경로는 source frame과 실제 selected CellUnit을 대조하므로
  empty spacer 행 전체 overflow보다 좁고 검증 가능한 owner 신호다.

## 구현

- `ROWBREAK_TRAILING_EMPTY_ROW_OVERFLOW_TOLERANCE_PX=40`과 마지막 spacer 행 전체를
  이전 fragment에 강제하던 경로를 제거했다.
- 마지막 empty row는 이제 일반 RowBreak budget과 `advance_row_cut`에 따라 처리한다.
  visible response의 stored tail을 정확히 보존하는 기존 공통 경로는 유지한다.

## 결과

- source-empty만으로 border/height를 생략하거나 body bottom 밖으로 넘기는 고정 reserve가
  사라졌다.
- 전체 export 및 test는 이 Stage에서 실행하지 않았다. 다음 Stage에서는 빈 spacer row의
  실제 border paint와 stored declaration을 함께 표현할 IR/fragment contract가 있는지
  별도 분석한다.
