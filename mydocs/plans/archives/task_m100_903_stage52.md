# Task m100 #903 Stage 52 계획

## 1. 목적

Stage51 판정으로 셀 텍스트 클리핑의 직접 원인은 `ParaShape.attr1 bits 20..21`로 확정되었다.

```text
07_vertical_align_bits_20_21:
  클리핑 해결

13_vertical_align_plus_font_line_height:
  클리핑 해결
  07과 동일 hash
```

Stage52의 목적은 이 비트군을 HWPX의 어떤 속성에서 가져와야 하는지 구현 전에 확정하는 것이다.

## 2. 현재 의심 지점

현재 HWPX parser에는 다음 매핑이 존재한다.

```text
src/parser/hwpx/header.rs

<autoSpacing eAsianEng="..."> -> ps.attr1 |= 1 << 20
<autoSpacing eAsianNum="..."> -> ps.attr1 |= 1 << 21
```

그러나 공식 HWP 5.0 스펙에서 `ParaShape.attr1 bits 20..21`은 문단 세로 정렬이다.

따라서 현재 구현은 다음 가능성이 있다.

```text
1. autoSpacing을 잘못된 HWP attr1 bit에 쓰고 있다.
2. HWPX align.vertical 값을 HWP ParaShape.attr1 bits 20..21로 매핑하지 않고 있다.
3. 그 결과 셀 안 문단의 baseline/vertical positioning이 한컴에서 다르게 해석된다.
```

## 3. 확인 대상

원본 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

현재 positive HWP:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

현재 baseline:

```text
output/poc/hwpx2hwp/task903/stage48_residual_text_layout_probe/08_plus_text_char_line_seg_table.hwp
```

## 4. 분석 작업

### 4.1 HWPX paraPr 분석

`samples/hwpx/hwpx-h-01.hwpx` 내부 `header.xml`의 `paraPr`를 조사한다.

확인 항목:

```text
paraPr id
align.horizontal
align.vertical
autoSpacing.eAsianEng
autoSpacing.eAsianNum
breakSetting.*
snapToGrid
fontLineHeight
```

출력:

```text
output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/para_pr_inventory.md
```

### 4.2 HWP ParaShape attr1 분석

정답 HWP와 baseline/positive의 `DocInfo/PARA_SHAPE` record를 비교한다.

확인 항목:

```text
para_shape_id
BodyText 참조 여부
정답 attr1 bits 20..21
positive attr1 bits 20..21
baseline attr1 bits 20..21
attr1 전체값
```

출력:

```text
output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/parashape_vertical_bits.md
```

### 4.3 매핑 후보 확정

다음 후보 중 어느 것이 정답 HWP와 일치하는지 판단한다.

```text
A. HWPX align.vertical -> HWP ParaShape.attr1 bits 20..21
B. HWPX autoSpacing.eAsianEng/eAsianNum -> HWP ParaShape.attr1 bits 20..21
C. HWPX에는 직접 속성이 없고 한컴 저장기가 default로 채우는 값
D. 특정 paraPr id에만 적용되는 예외
```

## 5. 구현 후보

분석 결과가 A로 확인되면 다음 최소 구현을 적용한다.

```text
1. parse_para_shape_child의 <align> 처리에서 vertical 속성을 읽는다.
2. vertical 값을 ParaShape.attr1 bits 20..21에 매핑한다.
3. autoSpacing이 attr1 bits 20..21을 건드리는 현재 코드를 제거하거나 다른 올바른 필드로 이동한다.
```

단, autoSpacing의 올바른 HWP 비트 위치가 확인되지 않으면 이번 #903에서는 제거만 하고 별도 이슈로 분리한다.

## 6. 검증 산출물

구현 변경 전 분석 리포트:

```text
output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/
```

구현 변경 후 후보 HWP:

```text
output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/hwpx-h-01.hwp
```

## 7. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage52_generate_vertical_attr_mapping_probe -- --nocapture
```

필요 시 구현 후 다음도 수행한다.

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage52_export_hwpx_h_01_after_parser_fix -- --nocapture
```

## 8. 작업지시자 판정 요청

| 파일 | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/hwpx-h-01.hwp` |  |  |  |  |  |  |  |

## 9. 성공 기준

```text
1. HWPX 속성과 HWP attr1 bits 20..21의 대응을 문서화한다.
2. 구현 변경이 필요한 정확한 코드 위치를 확정한다.
3. 구현 후 생성 HWP가 다음을 만족한다.
   - 한컴 열기 성공
   - 이미지 출력 성공
   - 표/셀 배치 성공
   - 셀 텍스트 클리핑 해결
   - 마지막 페이지 출력 성공
   - rhwp-studio 재열기 성공
```
