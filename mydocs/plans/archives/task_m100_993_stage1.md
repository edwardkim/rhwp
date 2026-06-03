# Task M100-993 Stage 1 계획서

## 1. 목적

#993 P1 작업을 `mel-001` 샘플로 재개한다.

대상 증상:

```text
source HWPX: samples/hwpx/mel-001.hwpx
oracle HWP: samples/hwpx/hancom-hwp/mel-001.hwp
generated HWP: rhwp HWPX -> HWP 저장 결과
```

현재 작업지시자 관찰:

```text
- rhwp가 HWPX -> HWP로 저장한 mel-001.hwp가 한컴 에디터에서 파일손상 판정을 받는다.
- 2페이지 첫 표의 셀 테두리/배경 처리가 generated HWP에서 잘못 저장되는 것으로 보인다.
- 한컴 에디터는 2페이지 "󰊳  예산 현황" 까지 출력한 뒤 중지한다.
```

## 2. 작업 원칙

이번 단계는 구현을 바로 시작하지 않는다. 먼저 한컴 정답 HWP와 rhwp generated HWP의
차이를 record contract 관점에서 분리한다.

```text
1. HWPX 원본이 rhwp IR로 정상 파싱/렌더링되는지 확인한다.
2. 현재 adapter로 generated HWP를 만든다.
3. generated HWP가 rhwp로 재로드되는지 확인한다.
4. 한컴 중단 지점인 2페이지 "󰊳  예산 현황" 주변 문단/표를 기준으로 oracle/generated
   record bundle을 비교한다.
5. 특히 2페이지 첫 표의 셀 테두리/배경 관련 record와 table/cell payload를 우선 비교한다.
```

## 3. 산출물

Stage 1 산출물 디렉터리:

```text
output/poc/hwpx2hwp/task993/stage1_mel001_damage_trace/
```

예정 산출물:

```text
generated-mel-001.hwp
generation.md
dump_pages_oracle.md
dump_pages_generated.md
inventory_oracle_section0.jsonl
inventory_generated_section0.jsonl
inventory_diff_lcs.md
record_window_budget_table_oracle.md
record_window_budget_table_generated.md
contract_findings.md
```

## 4. 우선 확인할 가설

### 가설 A: 표 셀 테두리/배경 payload 계약 불일치

작업지시자 관찰상 2페이지 첫 표의 셀 테두리/배경 처리가 잘못 저장된다.
따라서 `TABLE`, `LIST_HEADER`, `PARA_HEADER`, 셀 문단, border fill 참조/실체 record를
정답 HWP와 비교한다.

### 가설 B: 표 앞 문단과 표 control 순서/level 계약 불일치

한컴 에디터가 특정 텍스트 직후 중단하므로, 해당 문단 다음에 오는 표 control tuple의
record 순서, level, size, tail, child count가 깨졌을 가능성을 확인한다.

### 가설 C: HWPX 의미값은 충분하지만 HWP5 저장 contract materialization이 빠짐

#949에서 확인했듯, rhwp-studio가 렌더링 가능한 IR이라도 한컴 HWP5 loader가 요구하는
binary contract를 만족하지 못하면 파일손상으로 이어질 수 있다. 이번에도 정답 HWP를 기준으로
누락된 binary payload를 찾는다.

## 5. Stage 1 완료 조건

```text
- 현재 generated mel-001.hwp를 재현한다.
- 한컴 중단 위치에 대응되는 HWP5 record window를 특정한다.
- oracle/generated 차이를 최소 1개 이상의 구체적인 record contract 후보로 좁힌다.
- 구현 변경이 필요한 파일 범위를 제안한다.
```

## 6. 다음 단계 후보

Stage 1 결과에 따라 다음 중 하나로 진행한다.

```text
1. 표/셀 border fill contract 보강
2. table/cell payload materialization 보강
3. 표 앞뒤 문단/lineSeg/컨트롤 순서 보강
4. 위 세 항목이 아니면 해당 record tuple을 더 작은 단위로 분해하는 Stage 2 probe 작성
```

