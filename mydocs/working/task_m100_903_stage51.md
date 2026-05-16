# Task m100 #903 Stage 51 작업 기록

## 1. 목적

Stage50에서 셀 텍스트 클리핑의 직접 원인이 `DocInfo`의 `PARA_SHAPE.attr1`로 좁혀졌다.

Stage51은 `attr1` 32비트를 의미 단위 mask로 나누어, 어떤 비트군이 클리핑 해결에 실제로 필요한지 판정하기 위한 probe를 생성한다.

## 2. 기준 파일

positive:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

baseline:

```text
output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp
```

baseline의 `DocInfo` raw stream에서 `PARA_SHAPE` record의 첫 4바이트인 `attr1`만 mask 단위로 positive 값으로 교체했다.

## 3. 생성 위치

```text
output/poc/hwpx2hwp/task903/stage51_parashape_attr1_bit_probe/
```

부가 리포트:

```text
output/poc/hwpx2hwp/task903/stage51_parashape_attr1_bit_probe/stage51_attr1_bit_diff.md
output/poc/hwpx2hwp/task903/stage51_parashape_attr1_bit_probe/parashape_attr1_bit_probe_detail.md
```

## 4. 생성 결과

검증 명령:

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage51_generate_parashape_attr1_bit_probe -- --nocapture
```

결과:

```text
test result: ok. 1 passed
모든 산출물 rhwp reload ok, pages=9
각 파일 크기 375,808 bytes
```

## 5. mask별 diff 요약

| variant | mask | 전체 diff | BodyText 참조 diff | 해시 | 1차 판정 우선도 |
|---|---:|---:|---:|---|---|
| 01_line_spacing_type_bits_0_1 | `0x00000003` | 0 | 0 | `20c2002306a9895e` | 낮음, no-op |
| 02_alignment_bits_2_4 | `0x0000001c` | 0 | 0 | `20c2002306a9895e` | 낮음, no-op |
| 03_break_unit_bits_5_7 | `0x000000e0` | 54 | 32 | `e77718131ebb326e` | 높음 |
| 04_snap_to_grid_bit_8 | `0x00000100` | 71 | 50 | `efde493901204c00` | 높음 |
| 05_minimum_space_bits_9_15 | `0x0000fe00` | 4 | 0 | `7b7cd02e0fe3be0b` | 낮음, 참조 record diff 없음 |
| 06_paragraph_flow_bits_16_19 | `0x000f0000` | 0 | 0 | `20c2002306a9895e` | 낮음, no-op |
| 07_vertical_align_bits_20_21 | `0x00300000` | 11 | 6 | `28771eabb68d536d` | 중간 |
| 08_font_line_height_bit_22 | `0x00400000` | 0 | 0 | `20c2002306a9895e` | 낮음, no-op |
| 09_heading_bits_23_27 | `0x0f800000` | 0 | 0 | `20c2002306a9895e` | 낮음, no-op |
| 10_border_tail_bits_28_30 | `0x70000000` | 0 | 0 | `20c2002306a9895e` | 낮음, no-op |
| 11_break_unit_plus_snap_to_grid | `0x000001e0` | 79 | 50 | `aa33700b134792fa` | 높음 |
| 12_snap_to_grid_plus_font_line_height | `0x00400100` | 71 | 50 | `efde493901204c00` | 04와 동일 hash |
| 13_vertical_align_plus_font_line_height | `0x00700000` | 11 | 6 | `28771eabb68d536d` | 07과 동일 hash |
| 14_attr1_positive_control | `0xffffffff` | 80 | 51 | `98ba35feae9d9f40` | positive control |

## 6. 현재 해석

실제 판정 우선순위는 다음과 같다.

```text
1. 04_snap_to_grid_bit_8
2. 03_break_unit_bits_5_7
3. 11_break_unit_plus_snap_to_grid
4. 07_vertical_align_bits_20_21
5. 14_attr1_positive_control
```

`12`는 `04`와 동일 hash, `13`은 `07`과 동일 hash다. `01`, `02`, `06`, `08`, `09`, `10`은 diff count가 0이고 같은 hash로 묶였으므로 1차 판정에서는 우선순위가 낮다.

## 7. 작업지시자 판정 결과

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_line_spacing_type_bits_0_1 | 성공 | 성공 | 성공 | 실패 | 성공 | no-op 후보 |
| 02_alignment_bits_2_4 | 성공 | 성공 | 성공 | 실패 | 성공 | no-op 후보 |
| 03_break_unit_bits_5_7 | 성공 | 성공 | 성공 | 실패 | 성공 | 우선 판정 |
| 04_snap_to_grid_bit_8 | 성공 | 성공 | 성공 | 실패 | 성공 | 우선 판정 |
| 05_minimum_space_bits_9_15 | 성공 | 성공 | 성공 | 실패 | 성공 | 참조 diff 없음 |
| 06_paragraph_flow_bits_16_19 | 성공 | 성공 | 성공 | 실패 | 성공 | no-op 후보 |
| 07_vertical_align_bits_20_21 | 성공 | 성공 | 성공 | 성공 | 성공 | 우선 판정 |
| 08_font_line_height_bit_22 | 성공 | 성공 | 성공 | 실패 | 성공 | no-op 후보 |
| 09_heading_bits_23_27 | 성공 | 성공 | 성공 | 실패 | 성공 | no-op 후보 |
| 10_border_tail_bits_28_30 | 성공 | 성공 | 성공 | 실패 | 성공 | no-op 후보 |
| 11_break_unit_plus_snap_to_grid | 성공 | 성공 | 성공 | 실패 | 성공 | 우선 판정 |
| 12_snap_to_grid_plus_font_line_height | 성공 | 성공 | 성공 | 실패 | 성공 | 04와 동일 hash |
| 13_vertical_align_plus_font_line_height | 성공 | 성공 | 성공 | 성공 | 성공 | 07과 동일 hash |
| 14_attr1_positive_control | 성공 | 성공 | 성공 | 성공 | 성공 | positive control |

## 8. 판정 해석 규칙

```text
04 단독 해결:
  HWPX paraPr.snapToGrid -> HWP ParaShape.attr1 bit 8 매핑 누락이 직접 원인이다.

03 단독 해결:
  HWPX breakSetting의 breakLatinWord/breakNonLatinWord 계열 매핑 누락이 직접 원인이다.

03 실패, 04 실패, 11 해결:
  break unit + snapToGrid 조합이 필요하다.

07 해결:
  세로 정렬 bits 20..21 매핑도 셀 텍스트 기준선에 관여한다.

14만 해결:
  단일 비트군이 아니라 ParaShape attr1 조합 또는 특정 ParaShape index 조건 문제다.
```

## 9. 판정 해석

Stage51 판정으로 셀 텍스트 클리핑의 직접 원인은 `ParaShape.attr1`의 `bits 20..21`로 확정되었다.

```text
07_vertical_align_bits_20_21:
  클리핑 해결

13_vertical_align_plus_font_line_height:
  클리핑 해결
  07과 동일 hash

14_attr1_positive_control:
  클리핑 해결
```

반대로 다음 후보들은 클리핑을 해결하지 못했다.

```text
03_break_unit_bits_5_7:
  실패

04_snap_to_grid_bit_8:
  실패

11_break_unit_plus_snap_to_grid:
  실패
```

따라서 `breakSetting`의 줄 나눔 단위나 `snapToGrid`는 이번 셀 텍스트 클리핑의 직접 원인이 아니다.

## 10. 구현 연결점

현재 HWPX header parser에는 위험한 매핑이 있다.

```text
src/parser/hwpx/header.rs

<autoSpacing eAsianEng="..."> -> ps.attr1 |= 1 << 20
<autoSpacing eAsianNum="..."> -> ps.attr1 |= 1 << 21
```

하지만 공식 HWP 5.0 스펙에서 `ParaShape.attr1`의 `bits 20..21`은 문단 세로 정렬이다. Stage51의 결과도 이 비트군이 셀 텍스트 기준선/클리핑에 직접 영향을 준다는 것을 보여준다.

즉 다음 단계의 구현 후보는 `autoSpacing`을 attr1 bit 20/21에 넣는 현재 매핑을 제거하거나 정정하고, HWPX `align vertical` 값을 HWP `ParaShape.attr1 bits 20..21`로 매핑하는 것이다.

## 11. 다음 단계 후보

Stage52에서는 실제 HWPX parser/adapter 구현을 변경하기 전에 다음을 확인한다.

```text
1. samples/hwpx/hwpx-h-01.hwpx 의 paraPr/align vertical 값 분포 확인
2. 정답 HWP의 ParaShape.attr1 bits 20..21 값 분포 확인
3. 현재 parser가 autoSpacing으로 bits 20..21을 오염시키는지 확인
4. 최소 구현 방향 확정:
   - autoSpacing -> attr1 bits 20..21 매핑 제거
   - align vertical -> attr1 bits 20..21 매핑 추가
```
