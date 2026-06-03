# Task m100 #903 Stage 7

## 1. 단계 목적

Stage 6 산출물은 rhwp-studio에서 렌더링에 성공했지만 한컴 에디터에서는 여전히
파일 손상 판정을 받았다.

따라서 Stage 7은 문단/표 배치 로직을 더 고치는 단계가 아니라, 한컴 에디터가
엄격하게 검증하는 HWP 바이너리 레코드 길이와 tail 필드 호환성을 확인하는
RED/Probe 단계로 진행한다.

## 2. 입력과 기준 파일

입력 HWPX:

```text
samples/hwpx/hwpx-h-01.hwpx
```

한컴 정답 HWP:

```text
samples/hwpx/hancom-hwp/hwpx-h-01.hwp
```

Stage 6 산출물:

```text
output/poc/hwpx2hwp/task903/stage6/hwpx-h-01_adapter.hwp
```

Stage 7 산출물 위치:

```text
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/
```

작업지시자 시각 판정용 파일은 반드시 `output/` 아래에 생성한다.

## 3. Stage 6 이후 확정된 사실

Stage 6 산출물 재로드 결과:

```text
rhwp-studio: 렌더링 성공
한컴 에디터: 파일손상 판정
```

문단 `0:10` 표의 핵심 배치 속성은 Stage 6에서 복구되었다.

```text
wrap=TopAndBottom
vert=Para(off=289)
horz=Para(off=501)
outer_margin=(141,141,141,141)
```

따라서 Stage 5에서 보였던 `Square/Paper/outer_margin=0`류의 IR 붕괴는 현재
주 원인이 아니다.

## 4. 레코드 덤프 단서

`dump-records`로 정답 HWP와 Stage 6 산출물을 비교하면, 초반부터 레코드 크기
차이가 반복된다.

```text
정답 HWP:
  PARA_HEADER sz=24
  TABLE sz=24 또는 30
  LIST_HEADER sz=47 또는 65
  SectionDef CTRL_HEADER sz=47

Stage 6 산출물:
  PARA_HEADER sz=22
  TABLE sz=22 또는 28
  LIST_HEADER sz=34
  SectionDef CTRL_HEADER sz=28
```

특히 표 셀의 `LIST_HEADER`가 Stage 6에서 34바이트로 출력된다.
기존 저장 가이드와 HTML 표 생성 경로에서는 한컴 호환 표 셀을
`LIST_HEADER = 34B + raw_list_extra(13B) = 47B`로 다룬 이력이 있다.

또한 `TABLE` 레코드는 기존 HTML 표 생성 경로에서 `raw_table_record_extra = 2B`
를 표준 추가 바이트로 사용한다.

## 5. 주요 가설

### H1. 셀 LIST_HEADER tail 누락

HWPX에서 만든 셀은 `raw_list_extra`가 비어 있어 `LIST_HEADER`가 34바이트로
직렬화된다.

한컴 정답 HWP와 기존 저장 가이드는 표 셀 `LIST_HEADER` 47바이트 패턴을 보인다.
즉 34바이트 이후 13바이트 tail이 한컴 호환성에 필요할 수 있다.

우선순위: 매우 높음.

### H2. TABLE record tail 2바이트 누락

Stage 6의 `TABLE` 레코드는 행 수에 따라 정답보다 2바이트 짧다.
기존 HTML 표 생성 경로는 `raw_table_record_extra = vec![0, 0]`를 사용한다.

우선순위: 높음.

### H3. PARA_HEADER tail 2바이트 누락

정답 HWP의 `PARA_HEADER`는 24바이트이고 Stage 6은 22바이트다.
현 serializer는 `raw_header_extra`가 없으면 `instanceId(4)`까지만 기록한다.
한컴 저장본은 일부 문단에서 추가 2바이트 tail을 기대할 수 있다.

우선순위: 중간.

### H4. SectionDef CTRL_HEADER tail 누락

정답 HWP의 SectionDef `CTRL_HEADER`는 47바이트이고 Stage 6은 28바이트다.
다만 Stage 6은 rhwp-studio에서 전체 문서 구조를 정상 재로드하므로, 먼저 표/문단
레코드 tail을 좁힌 뒤 마지막에 검증한다.

우선순위: 낮음.

## 6. 진행 원칙

- 한 번에 한 종류의 레코드 tail만 보강한다.
- variant마다 한컴 손상 판정 변화와 rhwp-studio 재로드 상태를 같이 기록한다.
- 문단/표 배치 로직은 Stage 6에서 회복된 상태를 유지해야 한다.
- 한컴 정답 HWP와의 비교는 레코드 길이와 1페이지 주요 문단/표 IR을 함께 본다.
- 통과 variant가 나와도 즉시 production 코드로 확정하지 않고, 어떤 tail이 필요한지
  기록한 뒤 최소 구현으로 정리한다.

## 7. Probe variant 계획

생성 위치:

```text
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/
```

### 01_cell_list_header_tail_13

HWPX에서 생성된 표 셀 중 `raw_list_extra`가 비어 있는 경우 13바이트 tail을
materialize한다.

예상 레코드 변화:

```text
LIST_HEADER: 34B -> 47B
```

tail 초안:

```text
raw_list_extra[0..4] = cell.width (u32 LE)
raw_list_extra[4..13] = 0
```

### 02_table_record_tail_2

`TABLE` 레코드에서 `raw_table_record_extra`가 비어 있는 경우 2바이트 tail을
materialize한다.

예상 레코드 변화:

```text
TABLE: 22B/28B -> 24B/30B
```

### 03_cell_list_header_tail_13_plus_table_tail_2

01과 02를 동시에 적용한다.

목표는 표 관련 레코드 길이를 한컴 정답 패턴에 맞추는 것이다.

### 04_para_header_tail_2

`PARA_HEADER`가 22바이트로 생성되는 HWPX-origin 문단에 2바이트 tail을
추가하는 별도 probe다.

예상 레코드 변화:

```text
PARA_HEADER: 22B -> 24B
```

단, 이 변경은 문단 전반에 영향을 주므로 01~03 결과를 먼저 본 뒤 진행한다.

### 05_section_ctrl_header_tail

SectionDef `CTRL_HEADER` tail을 정답 HWP 패턴과 비교해 보강하는 마지막 probe다.

Stage 7의 우선 목표는 표/문단 손상 원인 분리이므로, 01~04에서 한컴 손상이
사라지지 않을 때만 생성한다.

## 8. 내부 검증

각 variant 생성 후 다음을 확인한다.

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
cargo test --test hwpx_roundtrip_integration -- --nocapture
```

추가로 `dump-records` 또는 보조 스크립트로 다음을 확인한다.

```text
PARA_HEADER size histogram
TABLE size histogram
LIST_HEADER size histogram
CTRL_HEADER size histogram
```

`ir-diff` 기준:

```text
cargo run --bin rhwp -- ir-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/stage7_record_tail_probe/<variant>.hwp \
  -s 0 -p 10
```

Stage 6보다 문단/표 배치 IR이 악화되면 실패로 본다.

## 9. 작업지시자 판정 항목

작업지시자는 생성된 variant를 한컴 에디터와 rhwp-studio에서 판정한다.

판정 항목:

- 한컴 에디터에서 파일 손상 판정이 사라지는지
- rhwp-studio에서 1페이지 렌더링이 Stage 6 수준을 유지하는지
- 1페이지 표 안 이미지가 유지되는지
- 문단/표 배치가 다시 무너지지 않는지

판정 기록 형식:

```text
| variant | 한컴 판정 | rhwp-studio 판정 | 비고 |
|---|---|---|---|
| 01 |  |  |  |
| 02 |  |  |  |
| 03 |  |  |  |
| 04 |  |  |  |
| 05 |  |  |  |
```

## 10. 승인 요청

Stage 7은 위 probe 계획에 따라 진행한다.

승인되면 먼저 01~03만 생성하고, 한컴 판정 결과에 따라 04~05 진행 여부를 결정한다.

## 11. 승인 및 01~03 생성 결과

작업지시자 승인 후 01~03 probe만 생성했다.

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/01_cell_list_header_tail_13.hwp
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/02_table_record_tail_2.hwp
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/03_cell_list_header_tail_13_plus_table_tail_2.hwp
```

생성 로그:

```text
01_cell_list_header_tail_13.hwp:
  bytes=699392
  changed_cells=1453
  changed_tables=0
  pages=9

02_table_record_tail_2.hwp:
  bytes=680448
  changed_cells=0
  changed_tables=26
  pages=9

03_cell_list_header_tail_13_plus_table_tail_2.hwp:
  bytes=699392
  changed_cells=1453
  changed_tables=26
  pages=9
```

CFB 파일 크기는 섹터 패딩 영향을 받으므로 variant 간 단순 크기 비교를 판단 기준으로
쓰지 않는다.

## 12. 레코드 크기 확인

`dump-records` 초반 레코드에서 의도한 차이가 확인됐다.

### 01_cell_list_header_tail_13

```text
TABLE       sz=22 또는 28
LIST_HEADER sz=47
```

`TABLE` tail은 적용하지 않고, 셀 `LIST_HEADER`만 34B에서 47B로 늘렸다.

### 02_table_record_tail_2

```text
TABLE       sz=24 또는 30
LIST_HEADER sz=34
```

`TABLE` record tail 2B만 적용했다. 이 2바이트는 재로드 파서에서는 `nZones=0`으로
소비될 수 있으므로, 모델의 `raw_table_record_extra` 보존 여부가 아니라 raw record
크기로 판단한다.

### 03_cell_list_header_tail_13_plus_table_tail_2

```text
TABLE       sz=24 또는 30
LIST_HEADER sz=47
```

Stage 6에서 짧았던 표 관련 두 레코드가 정답 HWP의 대표 패턴에 가까워졌다.

## 13. 내부 검증

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
=> 38 passed

cargo test --test hwpx_roundtrip_integration -- --nocapture
=> 17 passed
```

variant 03의 `ir-diff`:

```text
cargo run --bin rhwp -- ir-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task903/stage7_record_tail_probe/03_cell_list_header_tail_13_plus_table_tail_2.hwp \
  -s 0 -p 10
```

결과:

```text
차이 40건
주요 차이: ls[0].vpos, ParaShape indent/ml/mr/sb/sa
```

Stage 6에서 보였던 수준의 차이이며, 표 CommonObjAttr 붕괴는 재발하지 않았다.

## 14. 작업지시자 판정 요청

다음 3개 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/01_cell_list_header_tail_13.hwp
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/02_table_record_tail_2.hwp
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/03_cell_list_header_tail_13_plus_table_tail_2.hwp
```

판정 항목:

- 한컴 에디터 파일 손상 판정이 사라지는지
- rhwp-studio에서 렌더링 성공 상태가 유지되는지
- 1페이지 문단/표 배치가 Stage 6 수준을 유지하는지
- 1페이지 표 안 이미지가 유지되는지

판정 기록:

```text
| variant | 한컴 판정 | rhwp-studio 판정 | 비고 |
|---|---|---|---|
| 01_cell_list_header_tail_13 | 파일 손상 | 렌더링 성공 | LIST_HEADER 47B |
| 02_table_record_tail_2 | 파일 손상 | 렌더링 성공 | TABLE 24/30B |
| 03_cell_list_header_tail_13_plus_table_tail_2 | 파일 손상 | 렌더링 성공 | LIST_HEADER 47B + TABLE 24/30B |
```

판정 후:

- 01 또는 03에서 한컴 손상이 사라지면 `LIST_HEADER` tail 13B를 production 후보로 올린다.
- 02 또는 03에서 한컴 손상이 사라지면 `TABLE` tail 2B를 production 후보로 올린다.
- 01~03 모두 손상이면 Stage 7 계획의 04 `PARA_HEADER tail 2B`로 진행한다.

## 15. 01~03 판정 결과에 따른 04 생성

01~03 모두 한컴 파일 손상 판정이 유지됐다.
따라서 표 `LIST_HEADER`/`TABLE` tail만으로는 한컴 손상 원인이 아니라고 판단하고,
계획된 04 `PARA_HEADER tail 2B` probe를 생성했다.

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/04_para_header_tail_2.hwp
```

생성 로그:

```text
04_para_header_tail_2.hwp:
  bytes=683520
  changed_cells=0
  changed_tables=0
  changed_paragraphs=1582
  pages=9
```

레코드 덤프 확인:

```text
Stage 6 / 01~03:
  PARA_HEADER sz=22

04_para_header_tail_2:
  PARA_HEADER sz=24
```

04는 표 관련 레코드 tail을 바꾸지 않고, 문단 `PARA_HEADER`만 한컴 정답 HWP의
대표 패턴인 24바이트로 맞춘다.

내부 검증:

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
=> 38 passed
```

## 16. 04 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/04_para_header_tail_2.hwp
```

판정 항목:

- 한컴 에디터 파일 손상 판정이 사라지는지
- rhwp-studio에서 렌더링 성공 상태가 유지되는지
- 1페이지 문단/표 배치가 Stage 6 수준을 유지하는지
- 1페이지 표 안 이미지가 유지되는지

판정 후:

- 04에서 한컴 손상이 사라지면 `PARA_HEADER` 24B materialize를 production 후보로 올린다.
- 04도 손상이면 05 `SectionDef CTRL_HEADER tail` probe로 진행한다.

## 17. 04 판정 결과

```text
한컴 에디터: 파일손상 판정
rhwp-studio: 정상 렌더링
```

판단:

- `PARA_HEADER` 22B → 24B 보강만으로는 한컴 손상 원인이 해결되지 않는다.
- rhwp-studio 정상 렌더링은 계속 유지되므로, 손상 원인은 아직 한컴의 바이너리
  레코드 검증 조건 쪽에 남아 있다.

## 18. 05 SectionDef CTRL_HEADER tail probe

정답 HWP의 첫 SectionDef `CTRL_HEADER`는 47바이트다.

정답 HWP 덤프:

```text
CTRL_HEADER(secd) sz=47
64 63 65 73 00 00 00 00 6e 04 00 00 00 00 40 1f
00 00 01 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
```

Stage 6/04 계열 산출물은 28바이트였다.

```text
CTRL_HEADER(secd) sz=28
```

05는 SectionDef `CTRL_HEADER` tail만 19바이트 추가해 47바이트로 맞춘다.
문단/표 관련 tail은 추가하지 않는다.

생성 파일:

```text
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/05_section_def_ctrl_header_tail_19.hwp
```

생성 로그:

```text
05_section_def_ctrl_header_tail_19.hwp:
  bytes=680448
  changed_cells=0
  changed_tables=0
  changed_paragraphs=0
  changed_sections=4
  pages=9
```

레코드 덤프 확인:

```text
05_section_def_ctrl_header_tail_19:
  CTRL_HEADER(secd) sz=47
```

주의:

- 처음에는 `section.section_def.raw_ctrl_extra`만 바꿨으나 실제 `BodyText` 직렬화는
  문단 내 `Control::SectionDef`를 사용한다.
- 따라서 05는 `section.section_def`와 문단 내 `Control::SectionDef` 양쪽을 맞춰
  생성했다.
- 정답 HWP와 비교하면 tail 외에도 SectionDef core field 차이가 보인다.
  예: 정답은 `column_spacing=0x046e`, `outline_numbering_id=1`인데 Stage 산출물은
  해당 값이 0이다. 05가 실패하면 이 차이를 다음 가설로 분리한다.

내부 검증:

```text
cargo test --test hwpx_to_hwp_adapter -- --nocapture
=> 38 passed

cargo test --test hwpx_roundtrip_integration -- --nocapture
=> 17 passed
```

## 19. 05 작업지시자 판정 요청

다음 파일을 한컴 에디터와 rhwp-studio에서 판정한다.

```text
output/poc/hwpx2hwp/task903/stage7_record_tail_probe/05_section_def_ctrl_header_tail_19.hwp
```

판정 항목:

- 한컴 에디터 파일 손상 판정이 사라지는지
- rhwp-studio에서 렌더링 성공 상태가 유지되는지
- 1페이지 문단/표 배치가 Stage 6 수준을 유지하는지
- 1페이지 표 안 이미지가 유지되는지

판정 후:

- 05에서 한컴 손상이 사라지면 SectionDef `CTRL_HEADER` tail을 production 후보로 올린다.
- 05도 손상이면 SectionDef core field 차이와 첫 표의 `LIST_HEADER 65B` 패턴을
  Stage 8 후보 가설로 분리한다.

## 20. 05 판정 결과

```text
한컴 에디터: 파일손상 판정
```

판단:

- SectionDef `CTRL_HEADER`를 47바이트로 맞추는 tail 보강만으로는 한컴 손상 원인이
  해결되지 않는다.
- Stage 7의 레코드 길이 tail 단독 가설은 모두 실패했다.
- 다음 단계는 정답 HWP와 실제 core field 값이 다른 영역을 대상으로 한다.

Stage 8 후보:

- SectionDef core field 차이
  - 정답: `column_spacing=0x046e`, `outline_numbering_id=1`
  - Stage 산출물: 둘 다 0
- 첫 표 첫 셀 `LIST_HEADER 65B` 패턴
  - 정답 첫 표 첫 셀은 `LIST_HEADER sz=65`
  - Stage 7의 13B tail 보강은 `LIST_HEADER sz=47`까지만 맞춘다.
- 도형/그림 `CTRL_HEADER` description/extra 차이
  - 정답 HWP의 일부 그림 `CTRL_HEADER`는 긴 설명문 포함으로 246B까지 확장된다.
  - Stage 산출물은 46B 수준이다.
