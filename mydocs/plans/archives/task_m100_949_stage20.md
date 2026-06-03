# Task m100 #949 Stage 20 계획

## 1. 목적

Stage 19 후보는 `hwpx-h-03`의 `memoProperties -> MEMO_SHAPE` 및 `ID_MAPPINGS` 계약을
정적으로 닫았지만, 한컴 판정은 Stage 18과 동일하게 파일손상이다.

따라서 Stage 20은 `MEMO_SHAPE` 축을 종료하고, 남은 직접 후보인 2페이지 이미지 개체 묶기 주변
`GenShape` control contract를 정밀 추적한다.

## 2. 고정된 사실

```text
Stage 18:
- hwpx-h-01, hwpx-h-02: table-axis 계약 반영 후 성공
- hwpx-h-03: TABLE field diff=0인데 파일손상

Stage 19:
- hwpx-h-03 DocInfo MEMO_SHAPE count와 ID_MAPPINGS 72B 계약은 정답지와 일치
- 한컴 판정은 여전히 파일손상
```

따라서 `hwpx-h-03`의 직접 원인을 다음으로 좁힌다.

```text
BodyText.Section0#824
oracle:    CTRL_HEADER GenShape size=60
generated: CTRL_HEADER GenShape size=46
```

이 레코드는 `GenShape#1`이며, control graph상 2페이지 이미지 개체 묶기 주변이다.

## 3. 입력

```text
source HWPX:
samples/hwpx/hwpx-h-03.hwpx

oracle HWP:
samples/hwpx/hancom-hwp/hwpx-h-03.hwp

generated HWP:
output/poc/hwpx2hwp/task949/stage19_h03_contract_trace/hwpx-h-03-stage19-candidate.hwp

success controls:
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-01.hwp
output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-02.hwp
```

## 4. 작업 원칙

```text
1. 후보 HWP를 먼저 만들지 않는다.
2. BodyText.Section0#824의 60B vs 46B 차이를 바이트/필드 단위로 해석한다.
3. 같은 종류의 GenShape가 성공 샘플 h01/h02에서는 어떻게 직렬화되는지 대조한다.
4. HWPX source tree에서 이 GenShape가 어떤 XML 노드에서 온 것인지 연결한다.
5. 정답지 payload를 그대로 graft하는 방식이 아니라, HWPX source 값에서 재구성 가능한 contract를 찾는다.
```

## 5. 산출물

```text
output/poc/hwpx2hwp/task949/stage20_genshape_contract_trace/
```

예상 산출물:

```text
genshape_824_payload_diff.md
genshape_824_field_decode.md
genshape_source_trace.md
success_control_comparison.md
stage20_findings.md
```

## 6. 판정 기준

Stage 20은 원인 분석 단계다. 바로 한컴 판정용 HWP를 만들지 않는다.

후보 구현으로 넘어갈 조건:

```text
1. oracle 60B와 generated 46B의 추가 14바이트 의미가 설명된다.
2. 이 14바이트가 HWPX source 또는 렌더링/조판 계산값에서 재구성 가능하다는 경로가 확인된다.
3. h01/h02 성공 샘플과 모순되지 않는다.
```

이 조건을 만족할 때만 Stage 21에서 단일 후보 구현을 만든다.
