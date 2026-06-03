# Task m100 #903 Stage 8

## 1. 단계 목적

Stage 7에서 다음 tail 단독 probe는 모두 한컴 파일 손상 판정을 해소하지 못했다.

```text
01_cell_list_header_tail_13
02_table_record_tail_2
03_cell_list_header_tail_13_plus_table_tail_2
04_para_header_tail_2
05_section_def_ctrl_header_tail_19
```

따라서 Stage 8은 단순 레코드 길이 보강이 아니라, 정답 HWP와 실제 core field 값이
다른 지점을 대상으로 한 RED/Probe 단계로 진행한다.

## 2. 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 7 기준 산출물:

```text
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/05_section_def_ctrl_header_tail_19.hwp
```

Stage 8 산출물 위치:

```text
output/poc/hwpx2hwp/task903/stage8_core_field_probe/
```

작업지시자 시각 판정용 파일은 반드시 `output/` 아래에 생성한다.

## 3. Stage 7 결론

Stage 7에서 확인한 실패 가설:

```text
LIST_HEADER 34B -> 47B 단독: 실패
TABLE 22/28B -> 24/30B 단독: 실패
PARA_HEADER 22B -> 24B 단독: 실패
SectionDef CTRL_HEADER 28B -> 47B 단독: 실패
```

공통 결과:

```text
한컴 에디터: 파일손상 판정
rhwp-studio: 렌더링 성공 또는 기존 성공 상태 유지
```

판단:

- rhwp 렌더링/IR 구조는 이미 회복됐다.
- 한컴 손상 원인은 레코드 길이 하나가 아니라, 한컴 저장본이 기대하는 필드 조합
  또는 특정 객체의 확장 metadata에 있을 가능성이 높다.

## 4. 새 가설

### H1. SectionDef core field 차이

정답 HWP의 첫 SectionDef `CTRL_HEADER`:

```text
CTRL_HEADER(secd) sz=47
64 63 65 73 00 00 00 00 6e 04 00 00 00 00 40 1f
00 00 01 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
```

Stage 산출물의 대표 패턴:

```text
CTRL_HEADER(secd) sz=47 또는 28
64 63 65 73 00 00 00 00 00 00 00 00 00 00 40 1f
00 00 00 00 ...
```

차이:

```text
column_spacing:
  정답 = 0x046e
  Stage = 0

outline_numbering_id:
  정답 = 1
  Stage = 0
```

가설:

- 한컴은 SectionDef의 일부 core field가 0인 경우를 더 엄격하게 본다.
- 또는 SectionDef `CTRL_HEADER` 47B tail은 core field 값과 함께 맞아야 의미가 있다.

### H2. 첫 표 첫 셀 LIST_HEADER 65B 패턴

정답 HWP 첫 표 첫 셀:

```text
LIST_HEADER sz=65
```

Stage 7 최대 보강:

```text
LIST_HEADER sz=47
```

정답의 첫 셀은 34B 기본 셀 데이터 뒤에 31B tail이 붙는다.
Stage 7의 13B tail은 이 중 앞부분만 맞춘 것이다.

가설:

- 첫 표 첫 셀은 그림/필드/확장 정보가 있어 65B 패턴이 필요하다.
- 한컴 손상 판정이 첫 페이지 첫 표를 처리하는 중 이 확장 정보를 기대해서 발생할 수 있다.

### H3. 그림 CTRL_HEADER description/extra 차이

정답 HWP의 첫 표 안 첫 그림 `CTRL_HEADER`는 긴 설명문을 포함해 246B다.

Stage 산출물은 같은 위치 그림 `CTRL_HEADER`가 46B 수준이다.

가설:

- 설명문 자체는 보통 필수는 아니지만, HWPX 원본이 가진 이미지 대체 텍스트/설명 정보가
  HWP 저장 시 특정 필드 조합으로 함께 들어가야 할 수 있다.
- H1/H2 실패 후 확인한다.

## 5. Stage 8 variant 계획

생성 위치:

```text
output/poc/hwpx2hwp/task903/stage8_core_field_probe/
```

### 01_section_def_core_fields

SectionDef `CTRL_HEADER`를 다음처럼 맞춘다.

```text
column_spacing = 0x046e
outline_numbering_id = 1
raw_ctrl_extra = 19B zero tail
```

표/문단 tail은 추가하지 않는다.

목표:

- SectionDef core field 차이만으로 한컴 손상 판정이 바뀌는지 확인한다.

### 02_section_def_core_plus_para_header_tail

01에 `PARA_HEADER 24B` 보강을 더한다.

목표:

- SectionDef core field와 문단 header 길이가 함께 맞아야 하는지 확인한다.

### 03_first_cell_list_header_65

첫 표 첫 셀만 31B `raw_list_extra`를 부여해 `LIST_HEADER 65B`로 맞춘다.

초안:

```text
raw_list_extra[0..4] = cell.width
raw_list_extra[4..31] = 정답 HWP 첫 셀 tail 패턴에서 추출한 값
```

주의:

- 이 variant는 첫 표 첫 셀만 건드린다.
- 전체 셀에 31B tail을 적용하지 않는다.

### 04_section_def_core_plus_first_cell_65

01과 03을 함께 적용한다.

목표:

- SectionDef core field와 첫 표 첫 셀 확장 tail의 조합이 필요한지 확인한다.

### 05_first_picture_description_probe

H1~H2가 실패하면 첫 표 안 첫 그림 `CTRL_HEADER` description/extra를 정답 패턴에
가깝게 맞추는 probe를 만든다.

이 variant는 범위가 커지므로 01~04 결과 후 진행한다.

## 6. 내부 검증

각 variant 생성 후 다음을 실행한다.

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
cargo test --test hwpx_roundtrip_integration -- --nocapture
```

레코드 덤프 확인:

```text
cargo run --bin rhwp -- dump-records output/poc/hwpx2hwp/task903/stage8_core_field_probe/<variant>.hwp
```

`ir-diff` 확인:

```text
cargo run --bin rhwp -- ir-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/stage8_core_field_probe/<variant>.hwp \
  -s 0 -p 10
```

검증 기준:

- rhwp-studio 재로드/렌더링은 Stage 6 수준을 유지해야 한다.
- 1페이지 문단/표 배치가 다시 무너지면 실패다.
- 한컴 손상 판정이 사라지는 variant가 나오면 해당 field 조합을 production 후보로 올린다.

## 7. 생성 결과

생성 테스트:

```text
cargo test --test hwpx_to_hwp_adapter task903_stage8_generate_core_field_probe_variants -- --nocapture
```

결과:

```text
[#903 Stage 8] output/poc/hwpx2hwp/task903/stage8_core_field_probe/01_section_def_core_fields.hwp: bytes=680448, changed_sections=4, changed_paragraphs=0, changed_first_cells=0, changed_first_pictures=0, pages=9
[#903 Stage 8] output/poc/hwpx2hwp/task903/stage8_core_field_probe/02_section_def_core_plus_para_header_tail.hwp: bytes=683520, changed_sections=4, changed_paragraphs=1582, changed_first_cells=0, changed_first_pictures=0, pages=9
[#903 Stage 8] output/poc/hwpx2hwp/task903/stage8_core_field_probe/03_first_cell_list_header_65.hwp: bytes=680448, changed_sections=0, changed_paragraphs=0, changed_first_cells=1, changed_first_pictures=0, pages=9
[#903 Stage 8] output/poc/hwpx2hwp/task903/stage8_core_field_probe/04_section_def_core_plus_first_cell_65.hwp: bytes=680448, changed_sections=4, changed_paragraphs=0, changed_first_cells=1, changed_first_pictures=0, pages=9
[#903 Stage 8] output/poc/hwpx2hwp/task903/stage8_core_field_probe/05_first_picture_common_from_reference.hwp: bytes=680448, changed_sections=0, changed_paragraphs=0, changed_first_cells=0, changed_first_pictures=1, pages=9
[#903 Stage 8] output/poc/hwpx2hwp/task903/stage8_core_field_probe/06_section_def_first_cell_picture_common.hwp: bytes=680960, changed_sections=4, changed_paragraphs=0, changed_first_cells=1, changed_first_pictures=1, pages=9
```

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage8_core_field_probe/01_section_def_core_fields.hwp
output/poc/hwpx2hwp/task903/stage8_core_field_probe/02_section_def_core_plus_para_header_tail.hwp
output/poc/hwpx2hwp/task903/stage8_core_field_probe/03_first_cell_list_header_65.hwp
output/poc/hwpx2hwp/task903/stage8_core_field_probe/04_section_def_core_plus_first_cell_65.hwp
output/poc/hwpx2hwp/task903/stage8_core_field_probe/05_first_picture_common_from_reference.hwp
output/poc/hwpx2hwp/task903/stage8_core_field_probe/06_section_def_first_cell_picture_common.hwp
```

레코드 덤프 확인:

```text
01_section_def_core_fields.hwp
  CTRL_HEADER(secd) sz=47
  column_spacing = 0x046e
  outline_numbering_id = 1

03_first_cell_list_header_65.hwp
  첫 표 첫 셀 LIST_HEADER sz=65

04_section_def_core_plus_first_cell_65.hwp
  CTRL_HEADER(secd) sz=47
  첫 표 첫 셀 LIST_HEADER sz=65

05_first_picture_common_from_reference.hwp
  첫 표 첫 그림 CTRL_HEADER(gso) sz=246
  정답 HWP의 첫 표 첫 그림 CommonObjAttr(description 포함)를 graft

06_section_def_first_cell_picture_common.hwp
  CTRL_HEADER(secd) sz=47
  첫 표 첫 셀 LIST_HEADER sz=65
  첫 표 첫 그림 CTRL_HEADER(gso) sz=246
```

추가 내부 검증:

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
  39 passed

cargo test --test hwpx_roundtrip_integration -- --nocapture
  17 passed
```

`ir-diff` 확인:

```text
01_section_def_core_fields.hwp
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp 대비 -s 0 -p 10 차이 40건

02_section_def_core_plus_para_header_tail.hwp
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp 대비 -s 0 -p 10 차이 40건

03_first_cell_list_header_65.hwp
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp 대비 -s 0 -p 10 차이 40건

04_section_def_core_plus_first_cell_65.hwp
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp 대비 -s 0 -p 10 차이 40건

05_first_picture_common_from_reference.hwp
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp 대비 -s 0 -p 10 차이 40건

06_section_def_first_cell_picture_common.hwp
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp 대비 -s 0 -p 10 차이 40건
```

해석:

- Stage 8 probe는 SectionDef/첫 셀/첫 그림 `CommonObjAttr` record payload를 바꿨고, IR 비교 관점에서는 모든 variant가 동일한 문단 0.10 주변 차이를 유지한다.
- 작업지시자 판정은 한컴의 손상 판정이 이 low-level record 조합에 반응하는지 확인하는 용도다.

## 8. 작업지시자 판정 항목

판정 파일은 `output/` 아래에 생성한다.

판정 항목:

- 한컴 에디터 파일 손상 판정이 사라지는지
- rhwp-studio에서 렌더링 성공 상태가 유지되는지
- 1페이지 문단/표 배치가 Stage 6 수준을 유지하는지
- 1페이지 표 안 이미지가 유지되는지

판정 기록 형식:

```text
| variant | 한컴 판정 | rhwp-studio 판정 | 비고 |
|---|---|---|---|
| 01_section_def_core_fields | 파일을 읽거나 저장하는데 오류가 있습니다. | 정상 |  |
| 02_section_def_core_plus_para_header_tail | 파일을 읽거나 저장하는데 오류가 있습니다. | 정상 |  |
| 03_first_cell_list_header_65 | 파일을 읽거나 저장하는데 오류가 있습니다. | 정상 |  |
| 04_section_def_core_plus_first_cell_65 | 파일을 읽거나 저장하는데 오류가 있습니다. | 정상 |  |
| 05_first_picture_common_from_reference | 파일을 읽거나 저장하는데 오류가 있습니다. | 정상 |  |
| 06_section_def_first_cell_picture_common | 파일을 읽거나 저장하는데 오류가 있습니다. | 정상 |  |
```

판정 결론:

- 01~06 모두 한컴 손상 판정을 해소하지 못했다.
- rhwp-studio는 모두 정상 렌더링하므로 IR/rhwp 렌더러 관점의 구조 붕괴가 아니라, 한컴 HWP reader가 요구하는 record 조합 또는 payload 누락이 남아 있다.
- Stage 8의 `SectionDef core field`, 첫 셀 `LIST_HEADER 65B`, 첫 그림 `CTRL_HEADER(gso) 246B description` 가설은 단독/조합 모두 실패로 본다.

## 9. 승인 요청

Stage 8은 위 계획에 따라 먼저 01~04를 생성한다.

05/06은 01~04 외부 판정이 모두 실패하거나 추가 probe가 필요할 때 별도 승인 후 진행한다.
