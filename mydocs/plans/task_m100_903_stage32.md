# Task m100 #903 Stage 32 계획

## 1. 목적

Stage31 restart에서 다음 세 항목을 실제 adapter 경로에 반영했다.

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

따라서 Stage32의 목적은 파일 크기/압축 문제가 아닌 잔여 구조 차이를 분리하는 것이다.

## 2. 기준 파일

Positive control:

```text
output/poc/hwpx2hwp/task903/stage30_minimal_docinfo_probe/05_section_count_para_shapes_no_raw.hwp
```

이 파일은 작업지시자 판정에서 한컴 정상, 마지막 페이지 정상, 표/셀 배치 정상으로 확인되었다.

Negative control:

```text
output/poc/hwpx2hwp/task903/stage31_restart/hwpx-h-01.hwp
```

이 파일은 압축 헤더 보정 후 366K로 줄었지만 한컴 파일 읽기 오류가 남아 있다.

한컴 정답:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

## 3. Stage31에서 확인된 잔여 차이

Stage30 positive control vs Stage31 negative control:

```text
50건  vpos
26건  tbl outer_margin
18건  id
 9건  tbl horz_rel
 9건  tbl vert_rel
 8건  tbl wrap
 5건  tbl page_break
 4건  cc
 4건  char_offsets len
 4건  pos
 4건  text
 3건  char_shapes count
 1건  shape horz_rel
 1건  shape vert_rel
 1건  shape wrap
```

해석:

```text
Stage31 파일 읽기 오류는 section_count, ParaShape margin, 압축 플래그만으로는 해결되지 않는다.
Stage30 정상 기준선에는 남아 있었고 Stage31 실제 adapter 경로에는 빠진
배치/표 컨트롤 계열 보정이 다음 후보이다.
```

## 4. 검증 전략

이번 Stage32는 구현 확정이 아니라 최소 variant 검증 단계다.

```text
1. Stage31 restart 산출물을 baseline으로 둔다.
2. Stage30 positive control에서 특정 축만 가져와 graft한다.
3. 각 variant를 output/ 아래에 생성한다.
4. 작업지시자가 한컴 에디터와 rhwp-studio에서 판정한다.
5. 정상화되는 최소 축을 찾은 뒤 다음 stage에서 구현 계획을 세운다.
```

## 5. Variant 설계

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/
```

생성 후보:

| variant | 목적 |
|---|---|
| 01_vpos_only | line_seg vpos 차이만 Stage30 positive control 기준으로 보정 |
| 02_table_placement_tuple | table outer_margin, horz_rel, vert_rel, wrap, page_break 축만 보정 |
| 03_char_shape_ref_tuple | char shape id/count/position 축만 보정 |
| 04_text_tuple | cc, char_offsets, text 축만 보정 |
| 05_shape_placement_tuple | shape horz_rel, vert_rel, wrap 축만 보정 |
| 06_vpos_plus_table_placement | 01 + 02 조합 |
| 07_table_placement_plus_char_shape_ref | 02 + 03 조합 |
| 08_all_residual_axes | 01~05 전체 조합 |

## 6. 판정 항목

각 variant에 대해 작업지시자가 기록한다.

```text
- 한컴 파일 읽기 오류 여부
- 한컴 파일손상 여부
- 한컴 출력이 어디까지 진행되는지
- 마지막 9페이지 출력 여부
- 표/셀 배치 정상 여부
- rhwp-studio 재로드/렌더링 여부
```

판정표:

| variant | 한컴 판정 유형 | 한컴 출력 위치 | 마지막 페이지 출력 | 표/셀 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_vpos_only |  |  |  |  |  |  |
| 02_table_placement_tuple |  |  |  |  |  |  |
| 03_char_shape_ref_tuple |  |  |  |  |  |  |
| 04_text_tuple |  |  |  |  |  |  |
| 05_shape_placement_tuple |  |  |  |  |  |  |
| 06_vpos_plus_table_placement |  |  |  |  |  |  |
| 07_table_placement_plus_char_shape_ref |  |  |  |  |  |  |
| 08_all_residual_axes |  |  |  |  |  |  |

## 7. 승인 전 하지 않을 것

```text
- serializer/control.rs 추가 수정
- table/object record 구조 추정 구현
- Stage32 variant 결과 없이 local/devel 반영
- 파일 읽기 오류 원인을 table/object로 단정
```

## 8. 승인 요청

승인되면 Stage32 variant 생성 테스트를 작성하고,
`output/poc/hwpx2hwp/task903/stage32_residual_axis_probe/` 아래에 판정 파일을 생성한다.
