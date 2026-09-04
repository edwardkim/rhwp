# Stage 238: TAC RowBreak frame 범위

## 관찰

Stage 237 뒤 전체 integration 스윕에서 `overflow_cell_baseline`이 두 신규 문서를
발견했다.

| 문서 | 신규 `LAYOUT_OVERFLOW_CELL` |
| --- | ---: |
| `samples/hwp5-tbl-attr-1916.hwp` | 126줄 |
| `samples/hwpx/aift.hwpx` | 64줄 |

동일 스윕은 `upstream/devel`에서 0건으로 통과했다. 두 표본은 포맷은 다르지만 각각
HWP5 `pi=3`, HWPX `pi=911`의 자리차지(TAC) 표에서 같은 현상을 보였다. 표의 남은
셀을 다음 조각으로 넘기지 않고 현재 쪽의 clip 아래에 배치해 텍스트가 보이지 않았다.

## 원인

TAC 배치 경로의 `saved_single_tac_bottom_fits`와
`saved_tac_table_bottom_fits`는 저장 LineSeg만 현재 body tail에 닿으면 normal fit 검사를
생략했다. 그러나 LineSeg는 표의 앵커 줄일 수 있으며, 표 전체의 물리 하단을 뜻하지
않는다. 두 fixture는 짧은 앵커 줄이 현재 tail에 있었지만 실제 표 높이는 각각 889px와
901px여서 다음 physical page까지 이어져야 했다.

## 수정

저장 LineSeg 예외는 포맷과 무관하게 앵커·LineSeg 하단뿐 아니라
`anchor top + table_height`도 현재 body 안에 있어야 한다. 따라서 저장 좌표가 실제 표 전체를
현재 조각에 배치한다고 증명하는 경우에만 normal fit을 생략한다. 표 전체가 다음 쪽으로
이어지는 TAC는 종전 normal fit에 따라 flush된다. 비-TAC RowBreak의 저장 frame 계약과
#3820의 source-owned fragment 보정은 변경하지 않는다.

## 검증 계획

- helper 단위 테스트에서 anchor 줄만 들어가고 표 전체는 넘치는 경우를 거부하는지 확인한다.
- `overflow_cell_baseline`에서 두 신규 문서가 0줄로 돌아오는지 확인한다.
- #3820 rowspan band, #3738 각주 tail, #2813 float stack 집중 회귀를 함께 확인한다.
- 이후 라이브러리와 integration 전체 스윕을 다시 실행한다.

## 단계 검증 결과

- `cargo test --profile release-test --lib saved_tac_table_flow_tail_contract`: 1 passed, 0 failed.
- `cargo test --profile release-test --test overflow_cell_baseline`: 1 passed, 0 failed.
  683개 샘플 전수 계상에서 두 신규 문서는 더 이상 기준선 밖에 없다.
- `cargo test --profile release-test --test issue_3820_rowbreak_rowspan_band`: 4 passed, 0 failed.
