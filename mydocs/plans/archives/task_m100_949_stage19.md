# Task M100-949 Stage 19 계획서: hwpx-h-03 파일손상 contract trace

## 1. 목적

Stage 18에서 `hwpx-h-01`, `hwpx-h-02`는 성공했고 `hwpx-h-03`만 파일손상으로 남았다.
또한 `hwpx-h-03`의 TABLE field diff는 oracle 대비 0이므로, 다음 단계는 table-axis가 아니라
`hwpx-h-03` 전용 control contract 위반 지점을 찾는 것이다.

## 2. 고정 사실

```text
1. Stage17/18 table-axis 계약은 성공 후보로 고정한다.
2. hwpx-h-01, hwpx-h-02는 같은 adapter로 한컴/rhwp-studio 모두 성공했다.
3. hwpx-h-03은 한컴에서 파일손상이며 2페이지까지만 출력된다.
4. hwpx-h-03은 한컴에서 이미지 출력과 표/셀 배치가 성공했다.
5. hwpx-h-03은 rhwp-studio에서 1페이지 페이지네이션 실패가 있다.
6. hwpx-h-03 TABLE field diff는 oracle 대비 0이다.
```

## 3. 원칙

```text
1. 여러 변형 HWP를 무작위로 만들지 않는다.
2. 먼저 oracle HWP와 generated HWP의 record/contract 차이를 추적한다.
3. hwpx-h-01/hwpx-h-02는 성공 대조군으로만 사용한다.
4. 첫 번째 명확한 contract mismatch가 확인되기 전에는 원인을 단정하지 않는다.
5. 후보 HWP는 contract가 특정된 뒤 1개만 만든다.
```

## 4. 입력

```text
source HWPX:
samples/hwpx/hwpx-h-03.hwpx

oracle HWP:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated HWP:
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-03.hwp

success controls:
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-02.hwp
```

## 5. 작업 절차

1. `hwpx-h-03` oracle/generated의 record stream을 비교한다.
2. 2페이지 전후의 `CTRL_HEADER`, `CTRL_DATA`, `SHAPE_COMPONENT`, `SHAPE_PICTURE`,
   group/child control, `PARA_HEADER`, `PARA_LINE_SEG` 차이를 우선 확인한다.
3. source HWPX에서 해당 control의 XML subtree를 추출해 record parent/level/order와 연결한다.
4. `hwpx-h-01/02` 성공 파일의 같은 종류 control contract와 대조한다.
5. mismatch가 하나로 좁혀지면 최소 adapter 수정 후보를 만든다.
6. 후보가 만들어진 경우에만 `hwpx-h-03-stage19-candidate.hwp`를 생성한다.

## 6. 산출물

```text
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/
```

예상 보고서:

```text
record_stream_diff.md
control_contract_trace.md
source_xml_trace.md
success_control_comparison.md
stage19_conclusion.md
```

후보가 특정될 경우:

```text
hwpx-h-03-stage19-candidate.hwp
```

## 7. 판정 기준

Stage 19는 다음 중 하나를 만족해야 성공이다.

```text
1. hwpx-h-03 파일손상과 연결되는 첫 contract mismatch를 특정한다.
2. contract mismatch가 불충분하면, 추가로 필요한 진단 도구 또는 diff 축을 명시한다.
3. 후보 HWP를 만든 경우 한컴/rhwp-studio 판정표를 준비한다.
```
