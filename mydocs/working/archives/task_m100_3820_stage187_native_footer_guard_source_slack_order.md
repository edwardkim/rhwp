# #3820 Stage 187 - native footer guard and source slack order

## 목적

Stage 186 뒤 p106 표 29에서 native first-fragment paint boundary가 generic source
slack으로 무효화되는 회귀를 보정한다.

## 원인

`first_fragment_painted_row_footer_guard`는 stored rewind를 가진 native HWP5 표에서
마지막 whole row를 다음 physical fragment에 남기도록 page budget을 줄인다. 그러나
이어지는 `source_first_fragment_overflow_allowance`가 saved object frame 아래 slack을
다시 더해 같은 행을 허용했다.

두 값은 대체 관계가 아니다. footer guard는 실제 painted row boundary의 더 구체적인
source 계약이고, object-frame slack은 guard가 없는 일반 first-fragment frame에만
적용할 수 있다.

## 수정

- native painted footer guard가 양수이면 generic first-fragment source slack을 0으로
  제한한다.
- continuation, partial row, HWPX stored-layout 경로는 변경하지 않는다.

## 검증 상태

- `issue_3820_rewinding_rowbreak_uses_painted_first_fragment_boundary`: 통과
- `issue_3820_rowbreak_rowspan_band`: 4 passed
- `issue_3930_hwpx_hwp_save_layout`: native HWP와 HWPX 저장본이 모두 384쪽으로 실패

HWPX도 같은 방식으로 실패했으므로, 공통 원인은 Stage 186의 near-fit source geometry
guard다. HWPX stored-layout에서의 valid near-fit은 Stage 188에서 별도 보존한다.
