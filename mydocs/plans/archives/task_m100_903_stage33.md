# Task m100 #903 Stage 33 계획

## 1. 목적

Stage32의 모든 variant가 한컴 에디터에서 파일 읽기 오류 판정을 받았다.

하지만 `08_all_residual_axes`는 Stage30 positive control과의 `ir-diff`가 3건만 남았다.

```text
문단 0.29
ctrl[0] shape wrap: A=TopAndBottom vs B=Square
ctrl[0] shape vert_rel: A=Para vs B=Paper
ctrl[0] shape horz_rel: A=Para vs B=Paper
```

Stage33의 목적은 이 3건이 실제 원인인지, 그리고 단순 enum 필드가 아니라
`CommonObjAttr.attr` 또는 CTRL_HEADER payload가 원인인지 확인하는 것이다.

## 2. 배경

Stage32 generator는 ShapeObject.common의 enum 필드를 graft했다.

```text
common.vert_rel_to
common.horz_rel_to
common.text_wrap
```

그러나 HWP serializer의 shape CommonObjAttr 직렬화는 enum을 pack하지 않고
`common.attr`를 그대로 기록한다.

```text
src/serializer/control.rs:1364-1367
fn serialize_common_obj_attr(common: &CommonObjAttr) -> Vec<u8> {
    ...
    w.write_u32(common.attr).unwrap();
```

따라서 Stage32의 shape placement graft는 저장 payload에 반영되지 않았을 가능성이 높다.

## 3. 기준 파일

Positive control:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

Negative control:

```text
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/08_all_residual_axes.hwp
```

비교 대상 위치:

```text
section 0
paragraph 29
control 0
Control::Shape
```

## 4. Variant 설계

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/
```

생성 후보:

| variant | 목적 |
|---|---|
| 01_shape_common_attr_only | 문단 0.29 shape의 `common.attr`만 positive control에서 복사 |
| 02_shape_common_attr_plus_enum | `common.attr` + `vert_rel_to/horz_rel_to/text_wrap` 복사 |
| 03_shape_common_full | `CommonObjAttr` 전체를 positive control에서 복사 |
| 04_shape_common_full_plus_ctrl_data | 03 + 문단 `ctrl_data_records[0]` 복사 |
| 05_stage30_para29_shape_control_full | 문단 0.29의 shape control 전체를 positive control에서 복사 |

## 5. 판정 항목

작업지시자 판정표:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_shape_common_attr_only |  |  |  |  |  |  |
| 02_shape_common_attr_plus_enum |  |  |  |  |  |  |
| 03_shape_common_full |  |  |  |  |  |  |
| 04_shape_common_full_plus_ctrl_data |  |  |  |  |  |  |
| 05_stage30_para29_shape_control_full |  |  |  |  |  |  |

## 6. 기대 해석

```text
01 또는 02가 한컴 정상화:
  CommonObjAttr.attr packing 누락이 직접 원인.

03만 정상화:
  attr 외 CommonObjAttr payload 필드도 필요.

04만 정상화:
  CTRL_DATA와 Shape CommonObjAttr의 결합 문제.

05만 정상화:
  shape control 전체 레코드 단위가 필요하며 개별 필드 graft로는 부족.

모두 실패:
  ir-diff에 드러나지 않는 DocInfo/BinData/CFB stream 축으로 이동.
```

## 7. 승인 전 하지 않을 것

```text
- serializer/control.rs 수정
- adapter 구현 확정
- stage32 결과를 table/object 문제로 단정
- 대량 probe 재개
```

## 8. 승인 요청

승인되면 Stage33 variant 생성 테스트를 작성하고,
`output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/` 아래에 판정 파일을 생성한다.
