# Task m100 #903 Stage 50 계획

## 1. 목적

Stage49 판정으로 셀 텍스트 클리핑의 직접 축은 `DocInfo/PARA_SHAPE`로 확정되었다.

```text
01_para_shapes_from_positive:
  클리핑 개선

02_char_shapes_from_positive:
  클리핑 유지

03_styles_from_positive:
  클리핑 유지

04_font_faces_from_positive:
  클리핑 유지
```

Stage50의 목적은 `PARA_SHAPE` 85개 전체 중 어떤 필드군 또는 인덱스군이 클리핑 개선에 필요한지
더 좁히는 것이다.

## 2. 현재 확정 사항

```text
이미지 출력:
  DocInfo BIN_DATA metadata

큰 표/개체 배치:
  CTRL_HEADER payload

마지막 페이지 출력:
  DocProperties.section_count

기본 표/셀 배치:
  HWPX paraPr/margin -> ParaShape 매핑

잔여 셀 텍스트 클리핑:
  DocInfo PARA_SHAPE record 내용 차이
```

## 3. 기준 파일

positive:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

baseline:

```text
output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp
```

positive와 baseline의 DocInfo 차이는 `PARA_SHAPE` 85개뿐이다.

## 4. 접근 방식

Stage50은 production 구현을 하지 않는다.

먼저 `PARA_SHAPE` record payload를 필드 offset 단위로 비교한다. HWP ParaShape record는 54 bytes이며
현재 모델/직렬화 기준 필드 배치는 다음이다.

```text
0..4    attr1
4..8    margin_left
8..12   margin_right
12..16  indent
16..20  spacing_before
20..24  spacing_after
24..28  line_spacing
28..30  tab_def_id
30..32  numbering_id
32..34  border_fill_id
34..42  border_spacing[4]
42..46  attr2
46..50  attr3
50..54  line_spacing_v2
```

Stage30에서 margin 계열은 이미 구현 축으로 확인되었고, Stage49에서 남은 차이는
raw `PARA_SHAPE` 전체가 회복시킨다는 것까지 확인되었다.

따라서 Stage50은 다음 필드군을 따로 graft한다.

```text
attr1
line_spacing_fields
border_spacing
attr2_attr3_line_spacing_v2
reference_fields
margin_fields
```

## 5. 후보 HWP 생성

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage50_parashape_field_probe/
```

후보:

```text
01_attr1_only.hwp
02_margin_fields_only.hwp
03_line_spacing_only.hwp
04_reference_fields_only.hwp
05_border_spacing_only.hwp
06_attr2_attr3_lsv2_only.hwp
07_attr1_line_spacing.hwp
08_attr1_attr2_attr3_lsv2.hwp
09_all_except_margins.hwp
10_all_parashape_positive_control.hwp
```

각 후보는 Stage48 `08` baseline의 DocInfo raw_stream에서 `PARA_SHAPE` record payload만 부분 수정한다.
record header/record count는 유지한다.

## 6. 필드군 정의

```text
attr1:
  0..4

margin_fields:
  4..24

line_spacing_only:
  24..28

reference_fields:
  28..34

border_spacing:
  34..42

attr2_attr3_lsv2:
  42..54

all_parashape:
  0..54
```

## 7. 비교 리포트

출력:

```text
output/poc/hwpx2hwp/task903/stage50_parashape_field_probe/stage50_parashape_field_diff.md
```

리포트 포함 항목:

```text
1. 85개 ParaShape record별 first diff offset
2. offset 구간별 diff count
3. BodyText에서 실제 참조되는 para_shape_id와 diff 여부
4. 각 후보별 적용 field range
```

## 8. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage50_generate_parashape_field_probe -- --nocapture
```

## 9. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_attr1_only |  |  |  |  |  |  |
| 02_margin_fields_only |  |  |  |  |  |  |
| 03_line_spacing_only |  |  |  |  |  |  |
| 04_reference_fields_only |  |  |  |  |  |  |
| 05_border_spacing_only |  |  |  |  |  |  |
| 06_attr2_attr3_lsv2_only |  |  |  |  |  |  |
| 07_attr1_line_spacing |  |  |  |  |  |  |
| 08_attr1_attr2_attr3_lsv2 |  |  |  |  |  |  |
| 09_all_except_margins |  |  |  |  |  |  |
| 10_all_parashape_positive_control |  |  |  |  |  | positive control |

## 10. 판정 해석

```text
01에서 개선:
  attr1 bit 중 line spacing type/alignment/heading 외 보존 bit가 직접 원인이다.

03에서 개선:
  line_spacing 값이 직접 원인이다.

06에서 개선:
  attr2/attr3/line_spacing_v2 중 하나가 직접 원인이다.

08에서만 개선:
  attr1과 확장 line spacing 계열 조합 문제다.

09에서 개선:
  margin_fields는 이미 충분하고, 나머지 ParaShape raw bit가 원인이다.

10만 개선:
  더 세분화한 필드군 조합 또는 특정 인덱스 단위 문제다.
```

## 11. 성공 기준

```text
1. PARA_SHAPE 전체가 아니라 필요한 필드군을 좁힌다.
2. production 구현 후보를 HWPX ParaShape parser의 누락 field 또는 adapter 보정 field로 연결한다.
3. 필드군으로 좁혀지지 않으면 Stage51에서 para_shape index 단위로 좁힌다.
```
