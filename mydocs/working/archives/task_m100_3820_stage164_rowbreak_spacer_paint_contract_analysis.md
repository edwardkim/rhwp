# Stage 164: RowBreak source-empty spacer의 paint 계약 분석

## 목적

source-empty spacer 행을 일반 행과 다르게 owner할 수 있는 유일한 경우, 즉 stored geometry는
남아 있어도 실제 partial-table paint가 없는 경우를 IR과 layout 계층에서 확인한다.

## 분석 범위

- `TableCell`의 border/fill/height/spacing source 필드
- `layout_partial_table`의 마지막 행 border 및 background paint
- RowBreak fragment의 `end_row`, partial height, row clip의 관계
- source-empty spacer가 text-only empty, border-only, 완전 비가시 중 어느 경우인지

## 원칙

- 단순 텍스트 없음은 paint 없음의 증거가 아니다.
- border, fill, declared height 중 하나라도 paint되면 일반 fragment budget을 지킨다.
- 완전 비가시 spacer가 구조적으로 표현될 수 있다면, 행 번호가 아니라 그 source/paint
  속성으로만 owner를 정한다.

## 완료 기준

- source-empty spacer에 대한 IR-to-paint 신호를 확인한다.
- 가시 여부를 공통 predicate로 판별할 수 있을 때만 코드 경로를 추가한다.
- 분석, 구현, 결과를 같은 Stage 커밋으로 남긴다.

## 분석 결과

- `Cell`은 text/control과 별개로 `height`, `padding`, `border_fill_id`를 보존한다. 표와
  zone도 별도의 `border_fill_id`를 가지므로, 셀 문단이 비어 있어도 table-level fallback을
  포함한 border/fill paint가 남을 수 있다.
- `layout_partial_table`은 fragment 범위의 모든 cell에 box를 만들고 `render_cell_background`
  를 호출한다. 따라서 텍스트 없음은 partial-table에서 행 높이 또는 background가 사라진다는
  신호가 아니다.
- 기존 helper는 text와 control 부재만 확인하는데 이름이 `empty_trailing_spacer`여서 paint-free
  row로 오해될 여지가 있었다. source-empty spacer 전체를 이전 fragment에 흡수했던 Stage 163
  경로는 이미 제거됐으며, 이 Stage에서도 border/fill을 추정하는 새 owner 예외를 추가하지 않는다.

## 구현

- helper를 `row_has_no_text_or_controls`로 변경하고, 문서화 주석에 text-flow predicate일 뿐
  fragment-height overflow를 승인하는 paint predicate가 아님을 명시했다.
- terminal visible response의 stored-tail 경로는 여전히 이 text-flow 신호를 사용하지만,
  selected CellUnit과 stored frame을 함께 확인하는 기존 조건을 유지한다.

## 결과

- source-empty spacer라는 단일 신호로 border, fill, declared height를 생략하는 경로가 없다.
- 전체 export 및 test는 이 Stage에서 실행하지 않았다. 다음 Stage에서는 midpage declared-height
  trust의 고정 tolerance가 source geometry로 분해 가능한지 별도로 분석한다.
