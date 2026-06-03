# Task m100 #903 Stage 53 계획

## 1. 목적

Stage52의 순수 구현 산출물은 다음 판정을 받았다.

```text
한컴 열기: 성공
마지막 페이지 출력: 성공
이미지 출력: 실패
표/셀 배치: 실패
셀 텍스트 클리핑: 실패
rhwp-studio: 이미지 출력 안 되고 표 엉망 배치
```

Stage52에서 `align.vertical -> ParaShape.attr1 bits 20..21` 매핑은 정답 HWP와
논리적으로 맞는 것으로 확인되었다. 그러나 최종 산출물은 아직 Stage46~49에서 확인한
성공 조건을 만족하지 못했다.

Stage53의 목적은 **현재 구현 산출물에 어떤 성공 축이 실제로 누락되어 있는지**
다시 좁히는 것이다.

## 2. 기준 파일

현재 구현 산출물:

```text
output/poc/hwpx2hwp/task903/stage52_vertical_attr_mapping/hwpx-h-01.hwp
```

정답/positive:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
output/poc/hwpx2hwp/task903/stage40_table_min_leave_one_out/12_without_6596.hwp
```

비교 기준으로 쓸 성공 축:

```text
Stage46:
  DocInfo BIN_DATA metadata/raw_data가 이미지 출력 직접 원인

Stage47:
  BodyText CTRL_HEADER payload가 큰 표/개체 배치 회복의 직접 전제

Stage49~51:
  DocInfo PARA_SHAPE attr1 bits 20..21이 셀 텍스트 클리핑 직접 원인
```

## 3. Stage53 접근 원칙

이번 단계에서는 바로 추가 구현하지 않는다.

```text
1. Stage52 산출물과 positive HWP를 비교한다.
2. Stage52 산출물 위에 이미 검증된 성공 축을 하나씩 graft한다.
3. 어떤 축을 넣을 때 한컴/rhwp-studio 판정이 회복되는지 확인한다.
4. 그 결과로 구현 범위를 확정한다.
```

이전 stage에서 찾은 사실을 버리지 않고, 현재 구현 산출물 기준으로 재검증한다.

## 4. 분석 리포트

출력 폴더:

```text
output/poc/hwpx2hwp/task903/stage53_current_impl_gap_probe/
```

생성 리포트:

```text
current_vs_positive_docinfo.md
current_vs_positive_section0.md
current_parashape_vertical_bits.md
```

### 4.1 DocInfo 비교

확인 항목:

```text
BIN_DATA record payload
BinData model attr/status/raw_data 존재 여부
PARA_SHAPE attr1 bits 20..21
```

목적:

```text
이미지 실패가 Stage52 산출물의 BIN_DATA metadata 누락인지 확인한다.
vertical bits가 최종 HWP에도 실제로 직렬화되었는지 확인한다.
```

### 4.2 BodyText Section0 비교

확인 tag:

```text
66 PARA_HEADER
67 PARA_TEXT
68 PARA_CHAR_SHAPE
69 PARA_LINE_SEG
71 CTRL_HEADER
72 LIST_HEADER
77 TABLE
```

목적:

```text
Stage52 산출물의 표/개체 배치 실패가 CTRL_HEADER payload 누락 때문인지 확인한다.
```

## 5. Probe 생성

Stage52 산출물을 baseline으로 두고, positive HWP에서 검증된 축을 graft한다.

```text
01_current_plus_bindata.hwp
02_current_plus_ctrl_header.hwp
03_current_plus_bindata_ctrl_header.hwp
04_current_plus_parashape_vertical_bits.hwp
05_current_plus_bindata_ctrl_header_vertical_bits.hwp
06_current_plus_parashape_full.hwp
07_current_plus_bindata_ctrl_header_parashape_full.hwp
```

각 probe 의미:

| variant | 의미 |
|---|---|
| 01 | 이미지 실패가 BIN_DATA만으로 회복되는지 확인 |
| 02 | 표 배치 실패가 CTRL_HEADER만으로 회복되는지 확인 |
| 03 | 이미지 + 큰 표 배치가 둘의 조합으로 회복되는지 확인 |
| 04 | Stage52 vertical bits가 최종 HWP에 이미 들어갔는지 no-op/회복 확인 |
| 05 | 최소 유력 조합: BIN_DATA + CTRL_HEADER + vertical bits |
| 06 | ParaShape attr1이 아니라 전체 PARA_SHAPE raw가 필요한지 확인 |
| 07 | 상한 후보: BIN_DATA + CTRL_HEADER + PARA_SHAPE full |

## 6. 예상 해석

```text
01에서 이미지 회복:
  구현에서 BIN_DATA attr/status/raw_data 보강 필요

02에서 표 배치 회복:
  구현에서 CTRL_HEADER payload 보존/합성 필요

04가 no-op이고 클리핑 실패 유지:
  Stage52 vertical bits는 산출물에 이미 들어갔지만 표 배치 실패 때문에 클리핑 판정이 오염된 것

04에서 클리핑 회복:
  vertical bits가 최종 HWP 직렬화 경로에서 누락된 것

05가 성공:
  최소 구현 후보는 BIN_DATA + CTRL_HEADER + vertical bits

07만 성공:
  ParaShape attr1 일부만으로는 부족하고 PARA_SHAPE raw payload 보존/합성이 필요
```

## 7. 검증 명령

```bash
cargo test --test hwpx_to_hwp_adapter task903_stage53_generate_current_impl_gap_probe -- --nocapture
```

## 8. 작업지시자 판정 요청

| variant | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| 01_current_plus_bindata | 성공 | 실패 | 실패 | 실패 | 성공 | 이미지 출력 실패, 표 배치 실패 |  |
| 02_current_plus_ctrl_header | 성공 | 성공 | 실패 | 실패 | 성공 | 이미지 출력 일부, 표 배치 실패 |  |
| 03_current_plus_bindata_ctrl_header | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | `05`와 동일 SHA-256 |
| 04_current_plus_parashape_vertical_bits | 성공 | 실패 | 실패 | 실패 | 성공 | 이미지 출력 실패, 표 배치 실패 | baseline과 동일 SHA-256 |
| 05_current_plus_bindata_ctrl_header_vertical_bits | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 | `03`과 동일 SHA-256 |
| 06_current_plus_parashape_full | 성공 | 실패 | 실패 | 실패 | 성공 | 이미지 출력 실패, 표 배치 실패 |  |
| 07_current_plus_bindata_ctrl_header_parashape_full | 성공 | 성공 | 성공 | 성공 | 성공 | 성공 |  |

판정 후 확인:

```text
03_current_plus_bindata_ctrl_header.hwp
05_current_plus_bindata_ctrl_header_vertical_bits.hwp

SHA-256:
42ecb237d15923ccfbb35427776814ecf75e1d2e6e8c4cfa30edfe8aff69130c
```

따라서 `03`과 `05`는 바이트 단위로 같은 파일이다. 재판정 결과 두 파일 모두 성공으로
정렬되었다.

Stage53 판정 결론:

```text
03 BIN_DATA + CTRL_HEADER: 한컴 성공, rhwp-studio 성공
05 BIN_DATA + CTRL_HEADER + vertical bits: 03과 동일 파일, 성공
07 BIN_DATA + CTRL_HEADER + PARA_SHAPE full: 성공
```

`04`가 baseline과 동일 파일이므로 vertical bits는 이미 current 산출물에 반영되어 있다.
따라서 현 단계 최소 구현 후보는 `BIN_DATA metadata/raw_data 보강 + CTRL_HEADER payload 보존/합성`이다.

## 9. 승인 후 작업

승인되면 다음을 수행한다.

```text
1. Stage53 probe 생성 테스트 추가
2. current-vs-positive 리포트 생성
3. 7개 HWP 후보 생성
4. 작업지시자 판정 요청
```

판정 후에만 실제 구현 범위를 확정한다.
