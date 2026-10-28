# Stage 225: 저장 rewind RowBreak 첫 fragment의 행 경계

## 목적

`issue2006/1790387_prep_final_report.hwpx` section 0 `pi=304`가 첫 source fragment의
`Multiplier` 행을 다음 page로 잘라, continuation과 후속 본문을 과대 소비하는 문제를 해결한다.

## 증거

- 최신 HWP 2020 PDF source 17쪽은 표의 `Multiplier` 행까지 포함하고, source 18쪽은
  반복 header와 `General/Network/Single survey` 세 행으로 시작한다.
- `pi=304`의 source anchor는 현재 flow 518.9px과 같고 common object bottom은 924.7px로
  현재 body 안에 있다. `common.height=405.8px`는 전체 measured table 710.4px가 아니라
  첫 physical fragment다.
- source frame의 최근접 measured row end는 `r=4`다. 이 row end는 422.6px이고 현 scanner
  budget 407.2px보다 15.4px 크다.
- cell 내부 reset은 없지만, 다음 host `pi=306`의 positive vpos 25694가 table anchor 38914보다
  작다. 이는 continuation이 새 source page에서 이어진다는 물리 rewind다.
- object가 전체 row geometry를 덮지 않아 common height를 full table object로 해석할 수 없다.

## 수정

저장 anchor와 common object bottom이 현재 first-fragment flow bound에 정확히 맞는 RowBreak
표에서, 후속 host positive-vpos rewind가 있고 다음 host가 page-top reset은 아니며 object가 전체
row geometry를 덮지 않을 때만 source frame에 가장 가까운 행 끝을 선택한다. 해당 measured row
boundary와 current budget의 차이만 scanner allowance로 전달한다.

## 안전 경계

- 첫 fragment, 빈 start cut, non-inline RowBreak, source anchor 일치, declared bottom body 내,
  row geometry owner 일치, table footnote 없음이 모두 필요하다.
- page-top reset으로 표가 끝나는 경우와 full object frame은 제외한다.
- allowance는 source frame과 선택된 measured row-end의 계산 결과이며 고정 px 상수가 아니다.

## 검증 계획

1. `pi=304` 첫 fragment가 rows `0..4`, continuation이 source 18쪽의 rows `4..7`이 되는지 확인한다.
2. `pi=316`이 같은 source 18쪽에 남아 tail page가 사라지는지 확인한다.
3. 남은 `pi=330` source-tail만 다음 stage에서 분석한다.

## 검증 결과

`rhwp dump-pages`에서 문서 전체는 142쪽에서 141쪽으로 줄었다. `pi=304`는 source와
같이 first `rows=0..4`, continuation `rows=4..7`로 나뉘었고, `pi=316`은 continuation과
같은 source 18쪽에 남았다. source 17·18쪽 PDF raster 비교에서는 row ownership이 맞지만,
로컬 fallback metric의 continuation-table text overlap은 최종 font 환경 raster 대조에서
별도 확인해야 할 시각 항목으로 남긴다.
