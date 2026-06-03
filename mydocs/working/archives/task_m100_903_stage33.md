# Task m100 #903 Stage 33 작업 기록

## 1. 목적

Stage32의 `08_all_residual_axes`는 Stage30 positive control과 IR 차이가 3건만 남았다.

```text
section 0 paragraph 29 control 0 shape
- text_wrap: TopAndBottom vs Square
- vert_rel: Para vs Paper
- horz_rel: Para vs Paper
```

Stage33은 이 3건이 실제 한컴 파일 읽기 오류의 남은 원인인지 확인하기 위한 probe이다.
특히 serializer가 `CommonObjAttr`의 enum 필드가 아니라 `common.attr` 원시값을 기록하는 점을 검증한다.

## 2. 기준 파일

Positive control:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

Stage32 negative baseline:

```text
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/08_all_residual_axes.hwp
```

Stage33 출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/
```

## 3. 생성 파일

```text
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/01_shape_common_attr_only.hwp
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/02_shape_common_attr_plus_enum.hwp
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/03_shape_common_full.hwp
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/04_shape_common_full_plus_ctrl_data.hwp
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/05_stage30_para29_shape_control_full.hwp
```

파일 크기:

| variant | bytes | rhwp reload |
|---|---:|---|
| 01_shape_common_attr_only | 374272 | ok, pages=9 |
| 02_shape_common_attr_plus_enum | 374272 | ok, pages=9 |
| 03_shape_common_full | 374272 | ok, pages=9 |
| 04_shape_common_full_plus_ctrl_data | 374272 | ok, pages=9 |
| 05_stage30_para29_shape_control_full | 374784 | ok, pages=9 |

해시:

| variant | sha256 |
|---|---|
| 01_shape_common_attr_only | `947533af92246b5dee6320c6a6f615b3710a34e75fb17cb948f5821394c6e10f` |
| 02_shape_common_attr_plus_enum | `947533af92246b5dee6320c6a6f615b3710a34e75fb17cb948f5821394c6e10f` |
| 03_shape_common_full | `f86f0043ffcedbaf9c73362d33f998f193db2334d92b5d9fe2a5a57f318125dc` |
| 04_shape_common_full_plus_ctrl_data | `f86f0043ffcedbaf9c73362d33f998f193db2334d92b5d9fe2a5a57f318125dc` |
| 05_stage30_para29_shape_control_full | `93446e8c0f4c481da38be9a7bbf8c8b5f23519085fc37d0447baf79966b9a2c5` |

## 4. 내부 검증

생성 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage33_generate_shape_attr_probe_variants -- --nocapture
```

결과:

```text
test task903_stage33_generate_shape_attr_probe_variants ... ok
```

IR 비교:

```text
target/debug/rhwp ir-diff \
  output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/01_shape_common_attr_only.hwp \
  output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp \
  --summary
```

결과:

```text
=== 비교 완료: 차이 0 건 ===
```

`01_shape_common_attr_only`만으로도 Stage30 positive control과 전체 IR 차이가 0건이 되었다.
따라서 Stage32에서 남았던 3개 차이는 enum 필드 자체가 아니라 `CommonObjAttr.attr` 원시값 직렬화 반영 문제로 보는 것이 가장 단순하다.

## 5. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_shape_common_attr_only | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 실패 |
| 02_shape_common_attr_plus_enum | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 실패 |
| 03_shape_common_full | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 실패 |
| 04_shape_common_full_plus_ctrl_data | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 실패 |
| 05_stage30_para29_shape_control_full | 파일 읽기 오류 | 실패 | 실패 | 실패 | 성공 | rhwp-studio 이미지 렌더링 실패 |

판정 포인트:

```text
- 01이 한컴에서 정상으로 열리는지
- 01이 마지막 9페이지까지 출력되는지
- 01의 표/셀 배치가 Stage30 positive control 수준인지
- 01이 정상이라면 02~05는 확인용이며 구현 후보는 CommonObjAttr.attr 보존/pack 쪽으로 좁힌다.
- 01이 실패하고 03 또는 05만 성공하면 CommonObjAttr 전체 또는 shape control 전체 payload가 필요하다.
```

## 6. 현재 해석

Stage33은 Stage30 분석을 뒤집는 실험이 아니다.
Stage30에서 확정한 두 축은 유지된다.

```text
1. 마지막 페이지 미출력: DocProperties.section_count 보정 필요
2. 표/셀 세로 배치: HWPX paraPr/margin 자식 요소를 ParaShape로 매핑 필요
```

Stage31/32에서 남은 한컴 파일 읽기 오류는 위 두 축을 반영한 순수 adapter 출력이
Stage30 positive control과 아직 같지 않았기 때문에 발생했다.
Stage33은 그 잔여 차이 중 `section0/paragraph29/shape/common.attr`가 실제 결정점인지 확인하는 단계다.

## 7. 판정 해석

Stage33 판정으로 다음을 확정한다.

```text
1. section0/paragraph29/shape/common.attr 잔여 차이는 한컴 파일 읽기 오류의 직접 원인이 아니다.
2. Stage33 01은 Stage30 positive control과 전체 IR diff 0건이지만, 한컴에서는 여전히 파일 읽기 오류가 난다.
3. 따라서 이제 원인은 IR 모델 필드 차이가 아니라 HWP 저장 결과의 바이트/스트림/CFB/BinData 계층에 있다.
```

특히 `01_shape_common_attr_only`가 positive control과 IR diff 0건인데도 실패했으므로,
다음 단계에서 같은 IR을 가진 두 HWP 파일의 바이너리 구조 차이를 비교해야 한다.

비교 대상은 다음 쌍이다.

```text
Positive control:
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp

Failing candidate:
output/poc/hwpx2hwp/task903/stage33_shape_attr_probe/01_shape_common_attr_only.hwp
```

다음 단계 후보:

```text
1. 두 파일의 CFB stream 목록, stream 크기, 압축 여부를 비교한다.
2. FileHeader/DocInfo/BodyText/BinData stream 단위 SHA와 크기를 비교한다.
3. IR diff가 0인데 한컴 판정이 다른 최소 stream을 찾는다.
4. 특히 rhwp-studio 이미지 렌더링 실패가 같이 나타나므로 BinData stream과 picture storage id/id mapping을 우선 확인한다.
```
