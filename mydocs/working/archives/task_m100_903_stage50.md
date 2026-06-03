# Task m100 #903 Stage 50 작업 기록

## 1. 목적

Stage49에서 셀 텍스트 클리핑의 직접 축이 `DocInfo/PARA_SHAPE`로 확정되었다.

Stage50은 `PARA_SHAPE` record 전체가 아니라, payload 내부 필드군 중 어느 값이
클리핑 개선에 필요한지 분리하기 위한 probe다.

## 2. 기준 파일

positive:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

baseline:

```text
output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp
```

baseline은 다음 상태다.

```text
한컴 열기 성공
이미지 출력 성공
큰 표/개체 배치 성공
셀 텍스트 클리핑 유지
```

## 3. ParaShape field diff 요약

생성된 diff:

```text
output/poc/hwpx2hwp/task903/stage50_parashape_field_probe/stage50_parashape_field_diff.md
```

필드 범위:

| field | payload range |
|---|---|
| attr1 | 0..4 |
| margin_fields | 4..24 |
| line_spacing | 24..28 |
| reference_fields | 28..34 |
| border_spacing | 34..42 |
| attr2_attr3_lsv2 | 42..54 |

필드별 차이 개수:

| field | differing ParaShape records |
|---|---:|
| attr1 | 80 |
| margin_fields | 0 |
| line_spacing | 0 |
| reference_fields | 0 |
| border_spacing | 0 |
| attr2_attr3_lsv2 | 85 |

즉 Stage50 기준에서 실제 차이가 있는 필드는 두 그룹뿐이다.

```text
attr1
attr2 / attr3 / line_spacing_v2
```

다음 필드군은 positive와 baseline 사이에 차이가 없다.

```text
margin_fields
line_spacing
reference_fields
border_spacing
```

## 4. 생성 파일

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage50_parashape_field_probe/
```

생성 파일:

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

모든 파일은 `375808` bytes이고, rhwp 내부 reload는 모두 `pages=9`로 통과했다.

## 5. 실제 해시 그룹

### attr1 적용군

```text
01_attr1_only.hwp
07_attr1_line_spacing.hwp
```

hash:

```text
51df40b5397c71ec00a397ca44a4da0cc55f5d6eac2867b8873fc4ad09980c52
```

`line_spacing` 필드에는 차이가 없으므로 `07`은 `01`과 동일하다.

### baseline 동일군

```text
02_margin_fields_only.hwp
03_line_spacing_only.hwp
04_reference_fields_only.hwp
05_border_spacing_only.hwp
```

hash:

```text
c1e1a97c69fbee9ae90c4449cde441d9406a1995945408fbfb27750149bd5485
```

해당 필드군에는 실제 차이가 없으므로 baseline과 동일하다.

### attr2/attr3/line_spacing_v2 적용군

```text
06_attr2_attr3_lsv2_only.hwp
```

hash:

```text
3a326ba922c1d4acbbf895abf4379b83387ded0a84073965755232228a1af45d
```

### attr1 + 확장 필드 적용군

```text
08_attr1_attr2_attr3_lsv2.hwp
09_all_except_margins.hwp
10_all_parashape_positive_control.hwp
```

hash:

```text
704189795d001214a39b4ff065eb0ef1b99396978e6c475341c106adb3d97a86
```

`margin_fields`, `line_spacing`, `reference_fields`, `border_spacing`에는 차이가 없으므로
`08`, `09`, `10`은 같은 파일이 된다.

## 6. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage50_generate_parashape_field_probe -- --nocapture
```

결과:

```text
pass
```

## 7. 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| 01_attr1_only | 성공 | 성공 | 성공 | 해결 | 성공 | 실질 후보 |
| 02_margin_fields_only | 성공 | 성공 | 성공 | 실패 | 성공 | baseline 동일군 |
| 03_line_spacing_only | 성공 | 성공 | 성공 | 실패 | 성공 | baseline 동일군 |
| 04_reference_fields_only | 성공 | 성공 | 성공 | 실패 | 성공 | baseline 동일군 |
| 05_border_spacing_only | 성공 | 성공 | 성공 | 실패 | 성공 | baseline 동일군 |
| 06_attr2_attr3_lsv2_only | 성공 | 성공 | 성공 | 실패 | 성공 | 실질 후보 |
| 07_attr1_line_spacing | 성공 | 성공 | 성공 | 해결 | 성공 | 01과 동일 hash |
| 08_attr1_attr2_attr3_lsv2 | 성공 | 성공 | 성공 | 해결 | 성공 | 실질 후보 |
| 09_all_except_margins | 성공 | 성공 | 성공 | 해결 | 성공 | 08과 동일 hash |
| 10_all_parashape_positive_control | 성공 | 성공 | 성공 | 해결 | 성공 | 08과 동일 hash |

실질 판정 포인트는 다음 네 파일이다.

```text
01_attr1_only.hwp
02_margin_fields_only.hwp
06_attr2_attr3_lsv2_only.hwp
08_attr1_attr2_attr3_lsv2.hwp
```

`02`는 baseline 동일군 확인용이다.

## 8. 판정 해석

```text
01에서 개선:
  attr1이 직접 원인이다.

06에서 개선:
  attr2/attr3/line_spacing_v2가 직접 원인이다.

08에서만 개선:
  attr1과 attr2/attr3/line_spacing_v2의 조합이 필요하다.

08도 개선되지 않음:
  Stage49의 ParaShape 전체 적용 개선과 충돌하므로, Stage50 patch 방식 또는 기준 파일을 재검증한다.
```

## 9. 현재 해석

Stage50 생성 기준으로는 클리핑 후보가 크게 줄었다.

```text
후보:
  ParaShape attr1
  ParaShape attr2
  ParaShape attr3
  ParaShape line_spacing_v2

비후보:
  margin_left/right/indent/spacing_before/spacing_after
  line_spacing
  tab_def_id/numbering_id/border_fill_id
  border_spacing
```

다음 판정으로 production 구현 후보를 HWPX ParaShape parser의 누락 bit 또는 serializer의 attr 보존 문제로
더 좁힐 수 있다.

## 10. 작업지시자 판정 반영

판정 결과 `attr1` 적용군에서 셀 텍스트 클리핑이 해결되었다.

```text
01_attr1_only:
  해결

06_attr2_attr3_lsv2_only:
  실패

08_attr1_attr2_attr3_lsv2:
  해결
```

해시 그룹상 `07`은 `01`과 동일하고, `09`, `10`은 `08`과 동일하다.
따라서 확장 필드군은 필요하지 않고, `attr1`만으로 충분하다.

최종 해석:

```text
확정:
  셀 텍스트 baseline/클리핑 문제의 직접 원인은 ParaShape attr1 비트다.

기각:
  margin_fields
  line_spacing
  reference_fields
  border_spacing
  attr2
  attr3
  line_spacing_v2
```

`attr1`은 공식 스펙상 다음 값을 포함한다.

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

다음 단계는 `attr1` 내부 비트군을 분해해서 어떤 비트가 셀 텍스트 baseline 계산에 필요한지 확정한다.
