# Task m100 #903 Stage 51 계획

## 1. 목적

Stage50 판정으로 셀 텍스트 클리핑의 직접 원인은 `ParaShape.attr1`로 확정되었다.

```text
01_attr1_only:
  클리핑 해결

06_attr2_attr3_lsv2_only:
  클리핑 실패
```

Stage51의 목적은 `attr1` 32비트 중 어떤 비트군이 클리핑 해결에 필요한지 분리하는 것이다.

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

셀 텍스트 클리핑:
  DocInfo ParaShape.attr1
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

positive와 baseline의 `ParaShape.attr1` 값을 bit mask 단위로 조합한다.

## 4. attr1 비트 의미

공식 스펙 `mydocs/tech/한글문서파일형식_5.0_revision1.3.md`의 문단 모양 속성1 기준이다.

```text
bit 0..1   줄 간격 종류
bit 2..4   정렬 방식
bit 5..6   줄 나눔 기준 영어 단위
bit 7      줄 나눔 기준 한글 단위
bit 8      편집 용지의 줄 격자 사용 여부
bit 9..15  공백 최소값
bit 16     외톨이줄 보호 여부
bit 17     다음 문단과 함께 여부
bit 18     문단 보호 여부
bit 19     문단 앞 쪽 나눔 여부
bit 20..21 세로 정렬
bit 22     글꼴에 어울리는 줄 높이 여부
bit 23..24 문단 머리 모양 종류
bit 25..27 문단 수준
bit 28     문단 테두리 연결 여부
bit 29     문단 여백 무시 여부
bit 30     문단 꼬리 모양
```

현재 HWPX parser에서 명확히 취급하지 않는 후보:

```text
breakSetting.breakLatinWord / breakNonLatinWord
paraPr.snapToGrid
align.vertical
paraPr.fontLineHeight
paraPr.condense / suppressLineNumbers / checked
border.connect / border.ignoreMargin
```

특히 한컴 HWPX 참조 문서에서는 `snapToGrid=true`가 `ParaShapeType`의 유일한 true 기본값으로 확인되어 있다.

## 5. 후보 HWP 생성

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage51_parashape_attr1_bit_probe/
```

후보:

```text
01_line_spacing_type_bits_0_1.hwp
02_alignment_bits_2_4.hwp
03_break_unit_bits_5_7.hwp
04_snap_to_grid_bit_8.hwp
05_minimum_space_bits_9_15.hwp
06_paragraph_flow_bits_16_19.hwp
07_vertical_align_bits_20_21.hwp
08_font_line_height_bit_22.hwp
09_heading_bits_23_27.hwp
10_border_tail_bits_28_30.hwp
11_break_unit_plus_snap_to_grid.hwp
12_snap_to_grid_plus_font_line_height.hwp
13_vertical_align_plus_font_line_height.hwp
14_attr1_positive_control.hwp
```

각 후보는 Stage48 `08` baseline의 DocInfo raw_stream에서 `PARA_SHAPE` record의 attr1 4바이트 중
지정 bit mask만 positive 값으로 교체한다.

## 6. mask 정의

```text
line_spacing_type_bits_0_1:
  0x0000_0003

alignment_bits_2_4:
  0x0000_001c

break_unit_bits_5_7:
  0x0000_00e0

snap_to_grid_bit_8:
  0x0000_0100

minimum_space_bits_9_15:
  0x0000_fe00

paragraph_flow_bits_16_19:
  0x000f_0000

vertical_align_bits_20_21:
  0x0030_0000

font_line_height_bit_22:
  0x0040_0000

heading_bits_23_27:
  0x0f80_0000

border_tail_bits_28_30:
  0x7000_0000

attr1_positive_control:
  0xffff_ffff
```

## 7. 비교 리포트

출력:

```text
output/poc/hwpx2hwp/task903/stage51_parashape_attr1_bit_probe/stage51_attr1_bit_diff.md
```

리포트 포함 항목:

```text
1. bit group별 diff count
2. 실제 BodyText에서 참조되는 ParaShape id의 bit group diff count
3. 각 후보별 적용 mask
4. positive/baseline attr1 값 예시
```

## 8. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage51_generate_parashape_attr1_bit_probe -- --nocapture
```

## 9. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_line_spacing_type_bits_0_1 |  |  |  |  |  |  |
| 02_alignment_bits_2_4 |  |  |  |  |  |  |
| 03_break_unit_bits_5_7 |  |  |  |  |  |  |
| 04_snap_to_grid_bit_8 |  |  |  |  |  |  |
| 05_minimum_space_bits_9_15 |  |  |  |  |  |  |
| 06_paragraph_flow_bits_16_19 |  |  |  |  |  |  |
| 07_vertical_align_bits_20_21 |  |  |  |  |  |  |
| 08_font_line_height_bit_22 |  |  |  |  |  |  |
| 09_heading_bits_23_27 |  |  |  |  |  |  |
| 10_border_tail_bits_28_30 |  |  |  |  |  |  |
| 11_break_unit_plus_snap_to_grid |  |  |  |  |  |  |
| 12_snap_to_grid_plus_font_line_height |  |  |  |  |  |  |
| 13_vertical_align_plus_font_line_height |  |  |  |  |  |  |
| 14_attr1_positive_control |  |  |  |  |  | positive control |

## 10. 판정 해석

```text
04에서 해결:
  snapToGrid bit 8이 직접 원인이다.

08에서 해결:
  fontLineHeight bit 22가 직접 원인이다.

07 또는 13에서 해결:
  vertical align bits 20..21이 직접 원인이다.

03 또는 11에서 해결:
  break unit bits 5..7이 직접 원인이다.

14에서만 해결:
  단일 bit group이 아니라 attr1 조합 또는 특정 ParaShape index 문제다.
```

## 11. 성공 기준

```text
1. 셀 텍스트 클리핑을 해결하는 attr1 bit group을 찾는다.
2. HWPX parser에서 누락된 paraPr/align/breakSetting 속성과 연결한다.
3. production 구현 전에 Stage52에서 최소 bit만 적용한 통합 후보를 검증한다.
```
