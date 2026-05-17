# Task m100 #903 Stage 53 작업 기록

## 1. 목적

Stage52 순수 구현 산출물은 한컴에서 열리고 마지막 9페이지도 출력되지만,
이미지 출력, 표/셀 배치, 셀 텍스트 클리핑이 실패했다.

Stage53은 구현을 새로 더하지 않고, 현재 구현 산출물 위에 이전 stage에서
검증된 성공 축을 하나씩 graft하여 실제 누락 축을 확인한다.

## 2. 기준

현재 구현 baseline:

```text
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/00_current_impl_baseline.hwp
```

positive:

```text
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

## 3. 생성 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage53_generate_current_impl_gap_probe -- --nocapture
```

결과:

```text
test task903_stage53_generate_current_impl_gap_probe ... ok
```

## 4. 생성 파일

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/
```

판정 대상:

```text
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/01_current_plus_bindata.hwp
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/02_current_plus_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/03_current_plus_bindata_ctrl_header.hwp
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/04_current_plus_parashape_vertical_bits.hwp
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/05_current_plus_bindata_ctrl_header_vertical_bits.hwp
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/06_current_plus_parashape_full.hwp
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/07_current_plus_bindata_ctrl_header_parashape_full.hwp
```

리포트:

```text
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/current_vs_positive_docinfo.md
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/current_vs_positive_section0.md
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/current_parashape_vertical_bits.md
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/stage53_generation.md
```

## 5. 생성 결과

| variant | bytes | hash | rhwp reload |
|---|---:|---|---|
| `01_current_plus_bindata.hwp` | 374784 | `3e035742bbde64fe` | ok, pages=9 |
| `02_current_plus_ctrl_header.hwp` | 374784 | `cd657b389399aee1` | ok, pages=9 |
| `03_current_plus_bindata_ctrl_header.hwp` | 374784 | `e296c971cd41d28d` | ok, pages=9 |
| `04_current_plus_parashape_vertical_bits.hwp` | 374784 | `818408f0d0818650` | ok, pages=9 |
| `05_current_plus_bindata_ctrl_header_vertical_bits.hwp` | 374784 | `e296c971cd41d28d` | ok, pages=9 |
| `06_current_plus_parashape_full.hwp` | 374784 | `b2df459fc5d79bab` | ok, pages=9 |
| `07_current_plus_bindata_ctrl_header_parashape_full.hwp` | 374784 | `29874d1e0063dfc0` | ok, pages=9 |

참고:

```text
00_current_impl_baseline.hwp 와 04_current_plus_parashape_vertical_bits.hwp 는 SHA-256 동일
03_current_plus_bindata_ctrl_header.hwp 와 05_current_plus_bindata_ctrl_header_vertical_bits.hwp 는 SHA-256 동일
```

따라서 Stage52의 vertical bits 매핑은 현재 HWP 산출물에 이미 반영되어 있으며,
Stage53의 `vertical_bits` graft는 no-op으로 해석한다.

## 6. 리포트 핵심

### 6.1 DocInfo

current와 positive의 구조 수는 같다.

```text
section_count: 2 / 2
bin_data model count: 5 / 5
bin_data_content count: 5 / 5
para_shape model count: 85 / 85
DocInfo raw bytes: 26474 / 26474
DocInfo record count: 523 / 523
```

하지만 `BIN_DATA` payload가 다르다.

```text
current: raw_data 없음, attr=0x0, status=NotAccessed
positive: raw_data 있음, attr=0x101, status=Success
```

이미지 실패는 여전히 `BIN_DATA` metadata/raw_data 축이 유력하다.

### 6.2 BodyText Section0

record count는 같다.

```text
section0 records: 7879 / 7879
```

하지만 payload 차이가 크다.

```text
PARA_HEADER: 570
PARA_TEXT: 102
PARA_CHAR_SHAPE: 190
PARA_LINE_SEG: 42
CTRL_HEADER: 29
LIST_HEADER: 524
SHAPE_COMPONENT: 6
TABLE: 21
SHAPE_PICTURE: 5
```

표/개체 배치 실패는 Stage47에서 확인된 `CTRL_HEADER` 축을 우선 판정해야 한다.

### 6.3 ParaShape vertical bits

`align.vertical=CENTER`에 대응하는 `attr1 bits 20..21`은 current, reference,
positive가 모두 일치한다.

따라서 현재 남은 셀 텍스트 클리핑은 vertical bits 자체의 누락이라기보다,
표/셀 배치 실패 또는 ParaShape의 다른 attr1 하위 비트 누락과 결합되어 나타나는
현상으로 본다.

## 7. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_current_plus_bindata | 성공 | 실패 | 실패 | 실패 | 성공 | 이미지 출력 실패, 표 배치 실패 |  |
| 02_current_plus_ctrl_header | 성공 | 성공 | 실패 | 실패 | 성공 | 이미지 출력 일부, 표 배치 실패 |  |
| 03_current_plus_bindata_ctrl_header | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | `05`와 동일 SHA-256 |
| 04_current_plus_parashape_vertical_bits | 성공 | 실패 | 실패 | 실패 | 성공 | 이미지 출력 실패, 표 배치 실패 | baseline과 동일 SHA-256 |
| 05_current_plus_bindata_ctrl_header_vertical_bits | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | `03`과 동일 SHA-256 |
| 06_current_plus_parashape_full | 성공 | 실패 | 실패 | 실패 | 성공 | 이미지 출력 실패, 표 배치 실패 |  |
| 07_current_plus_bindata_ctrl_header_parashape_full | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |  |

## 8. 판정 전 해석 후보

```text
01에서 이미지만 회복:
  BIN_DATA metadata/raw_data 구현 필요

02에서 표 배치가 회복:
  CTRL_HEADER payload 보존/합성 구현 필요

03에서 이미지와 표 배치가 함께 회복:
  최소 후보는 BIN_DATA + CTRL_HEADER

04는 baseline과 동일:
  Stage52 vertical bits 구현은 산출물에 반영됨

05는 03과 동일:
  BIN_DATA + CTRL_HEADER 상태에서도 vertical bits 추가는 no-op

06 또는 07에서만 클리핑이 회복:
  vertical bits 외 ParaShape attr1 하위 비트 또는 raw PARA_SHAPE payload가 필요
```

## 9. 판정 후 해석

### 9.1 확정된 점

`04_current_plus_parashape_vertical_bits.hwp`는 baseline과 SHA-256이 동일하다.

```text
00_current_impl_baseline.hwp
04_current_plus_parashape_vertical_bits.hwp

SHA-256:
70a30deb689347135140ee6d2283b5f29f2a1482fc123e37937e400ccd5c1e30
```

따라서 Stage52에서 구현한 `align.vertical -> attr1 bits 20..21` 매핑은
현재 HWP 산출물에 이미 들어가 있다. 이 축은 남은 실패의 직접 원인이 아니다.

### 9.2 재판정으로 정리된 점

`03_current_plus_bindata_ctrl_header.hwp`와
`05_current_plus_bindata_ctrl_header_vertical_bits.hwp`는 SHA-256이 동일하다.

```text
03_current_plus_bindata_ctrl_header.hwp
05_current_plus_bindata_ctrl_header_vertical_bits.hwp

SHA-256:
42ecb237d15923ccfbb35427776814ecf75e1d2e6e8c4cfa30edfe8aff69130c
```

초기 판정은 두 파일 사이에 차이가 있었지만, 재판정으로 둘 다 성공임을 확인했다.

```text
03: 한컴 성공, rhwp-studio 성공
05: 한컴 이미지 출력 성공, rhwp-studio 성공
```

바이트 단위로 같은 파일이므로 이 재판정이 논리적으로 맞다.

### 9.3 현재 유력 구현 축

판정 결과:

```text
01 BIN_DATA 단독: 이미지 회복 실패
02 CTRL_HEADER 단독: 이미지 일부 회복, 표 배치 실패
03 BIN_DATA + CTRL_HEADER: 이미지/표/클리핑/마지막 페이지 모두 성공
06 PARA_SHAPE full 단독: 회복 실패
07 BIN_DATA + CTRL_HEADER + PARA_SHAPE full: 성공
```

따라서 Stage53 기준 최소 구현 후보는 다음 둘이다.

```text
1. BIN_DATA metadata/raw_data 보강
2. BodyText CTRL_HEADER payload 보존/합성
```

`04`가 baseline과 동일하고 `03`과 `05`가 동일하므로, Stage52의 vertical bits 매핑은
이미 current 산출물에 들어가 있으며 Stage53 실패의 직접 원인이 아니다.

`06`이 실패하고 `07`이 성공한 것은 `PARA_SHAPE full` 단독 효과가 아니라
`BIN_DATA + CTRL_HEADER` 조합 위에서 함께 성공한 상한 후보로 해석한다. 즉 다음 구현
단계에서는 먼저 `BIN_DATA + CTRL_HEADER`를 최소 변경으로 적용하고 검증하는 것이 맞다.
