# Task m100 #903 Stage 32

## 1. 단계 목적

Stage31 restart는 Stage30의 핵심 조건 세 가지를 실제 adapter 경로에 반영했다.

```text
1. DocProperties.section_count 보정
2. HWPX paraPr/margin child -> ParaShape margin 파싱
3. HWP FileHeader compressed 플래그 보정
```

결과:

```text
파일 크기: 665K -> 366K
rhwp 재로드: 9페이지
한컴 에디터: 파일 읽기 오류
```

Stage32는 한컴 정상 positive control과 Stage31 negative control의 잔여 차이를 축별로
graft하여, 파일 읽기 오류를 좌우하는 최소 축을 찾는다.

## 2. 기준 파일

Positive control:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

Negative control:

```text
output/poc/hwpx2hwp/task903/stage31_restart/hwpx-h-01.hwp
```

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/
```

## 3. 구현 메모

계획서 작성 후 `ir-diff` 구현을 확인한 결과, summary의 `id` 항목은 table/control id가 아니라
`char_shapes[].char_shape_id`에서 나온 항목이었다.

따라서 Stage32 variant 이름을 다음처럼 보정했다.

```text
03_table_id_tuple -> 03_char_shape_ref_tuple
07_table_placement_plus_id -> 07_table_placement_plus_char_shape_ref
```

## 4. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/01_vpos_only.hwp
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/02_table_placement_tuple.hwp
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/03_char_shape_ref_tuple.hwp
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/04_text_tuple.hwp
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/05_shape_placement_tuple.hwp
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/06_vpos_plus_table_placement.hwp
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/07_table_placement_plus_char_shape_ref.hwp
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/08_all_residual_axes.hwp
```

각 파일은 366K 수준이며, rhwp 재로드 기준 9페이지를 유지한다.

## 5. 내부 검증

실행:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage32_generate_residual_axis_probe_variants -- --nocapture
```

결과:

```text
test result: ok. 1 passed
```

생성 로그:

```text
01_vpos_only.hwp bytes=374272 pages=9
02_table_placement_tuple.hwp bytes=374272 pages=9
03_char_shape_ref_tuple.hwp bytes=374272 pages=9
04_text_tuple.hwp bytes=374272 pages=9
05_shape_placement_tuple.hwp bytes=374272 pages=9
06_vpos_plus_table_placement.hwp bytes=374272 pages=9
07_table_placement_plus_char_shape_ref.hwp bytes=374272 pages=9
08_all_residual_axes.hwp bytes=374272 pages=9
```

바이트 해시는 모두 달라 variant가 실제로 분리되었다.

`08_all_residual_axes`와 Stage30 positive control의 `ir-diff --summary`:

```text
1건 shape horz_rel
1건 shape vert_rel
1건 shape wrap
```

Stage31 restart 대비 잔여 차이는 147건에서 3건까지 줄었다.

## 6. 작업지시자 판정 요청

다음 8개 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_vpos_only | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 비정상 렌더링 |
| 02_table_placement_tuple | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 실패 |
| 03_char_shape_ref_tuple | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 비정상 렌더링 |
| 04_text_tuple | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 비정상 렌더링 |
| 05_shape_placement_tuple | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 비정상 렌더링 |
| 06_vpos_plus_table_placement | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 실패 |
| 07_table_placement_plus_char_shape_ref | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 실패 |
| 08_all_residual_axes | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 실패 |

판정 포인트:

```text
1. 한컴 파일 읽기 오류가 사라지는 최소 variant
2. 한컴 파일손상으로 바뀌는 variant
3. 파일은 열리지만 마지막 페이지/표 배치가 깨지는 variant
4. rhwp-studio에서 9페이지가 유지되는지
```

## 7. 판정 해석

Stage32의 8개 variant는 모두 한컴 에디터에서 파일 읽기 오류 판정을 받았다.
따라서 이번 Stage32에서 분리한 다음 축들은 단독 또는 단순 조합으로 충분하지 않다.

```text
- vpos
- table placement tuple
- char shape ref tuple
- text tuple
- shape placement enum tuple
```

중요한 추가 관찰:

`08_all_residual_axes`는 Stage30 positive control과의 `ir-diff`가 3건만 남는다.

```text
문단 0.29
ctrl[0] shape wrap: A=TopAndBottom vs B=Square
ctrl[0] shape vert_rel: A=Para vs B=Paper
ctrl[0] shape horz_rel: A=Para vs B=Paper
```

즉 Stage32 generator는 대부분의 잔여 IR 차이를 제거했지만, 문단 `0.29`의 Shape placement가
저장 후에도 정답 쪽으로 바뀌지 않았다.

원인 후보:

```text
Stage32 generator는 ShapeObject.common의 enum 필드(vert_rel_to, horz_rel_to, text_wrap)를 graft했다.
하지만 HWP serializer/control.rs의 shape CommonObjAttr 직렬화는 enum을 다시 pack하지 않고
common.attr를 그대로 기록한다.
```

확인 위치:

```text
src/serializer/control.rs:1364-1367
fn serialize_common_obj_attr(common: &CommonObjAttr) -> Vec<u8> {
    ...
    w.write_u32(common.attr).unwrap();
```

반면 adapter용 `common_obj_attr_writer`는 attr가 0일 때만 enum으로부터 pack한다.

```text
src/document_core/converters/common_obj_attr_writer.rs:31-38
let attr = if common.attr != 0 {
    common.attr
} else {
    pack_common_attr_bits(common)
};
```

따라서 Stage32의 shape placement variant는 실제 저장 payload를 충분히 바꾸지 못했을 수 있다.

Stage32 결론:

```text
1. 파일 크기/압축 문제는 해결된 상태다.
2. section_count, ParaShape margin, FileHeader compressed 보정은 유지한다.
3. vpos/table/text/char_shape_ref 단순 graft는 한컴 파일 읽기 오류를 해결하지 못했다.
4. 남은 유력 후보는 문단 0.29 Shape CommonObjAttr.attr 또는 CTRL_HEADER payload 축이다.
5. 다음 Stage33은 문단 0.29 shape의 common.attr/raw payload를 정답 기준으로 맞추는
   최소 variant를 먼저 만든다.
```
