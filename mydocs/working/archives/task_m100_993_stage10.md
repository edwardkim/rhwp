# Task M100-993 Stage 10 완료 보고서

## 1. 목적

Stage 9에서 `tb-org-02`와 `mel-001`의 조직도 셀 줄나눔은 개선되었다.
그러나 `mel-001`에서는 다음 잔존 문제가 확인되었다.

```text
현상:
  한컴 에디터에서 2페이지 "2. 인원 현황" 다음 표의 병합된 아래쪽 셀 높이가 과도하게 렌더링된다.

결과:
  rhwp-studio에서는 같은 페이지에 들어가는 다음 표가 한컴 에디터에서는 다음 페이지로 밀린다.
```

Stage 10의 목적은 이 문제를 셀 높이 직접 보정으로 처리하지 않고, 정답 HWP와 Stage 9 생성 HWP의
HWP5 raw record contract 차이로 분리하는 것이다.

## 2. 대상

정답지:

```text
samples/hwpx/hancom-hwp/mel-001.hwp
```

생성본:

```text
output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/mel-001.hwp
```

문제 위치:

```text
section: 0
paragraph: 22
table: 8x12, cells=87
위치: "󰊲  인원 현황(’25.11.30. 기준)" 바로 다음 표
```

## 3. raw 비교 결과

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/
```

비교 문서:

```text
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/pi22_table_bundle_diff.md
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/pi22_cell_payload_diff.md
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/pi22_suspicious_cells.md
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/table_diff.jsonl
```

`hwp5-inventory-diff --report table-fields --focus table` 기준으로 TABLE payload hash가 다른 후보는
3개다.

```text
1. 10x65 조직도 표
2. 8x12 인원현황 표
3. 12x5 예산현황 표
```

이번 문제 표는 2번이다.

## 4. pi=22 의심 축

### TABLE tail

TABLE record의 기본 필드는 같다.

```text
table_attr:     0x0000000c
rows/cols:      8x12
cell_spacing:   0
inner margins:  140/140/140/140
```

차이는 tail이다.

```text
oracle:
  0a 00 0c 00 0a 00 0c 00 0a 00 0c 00 0a 00 00 00

generated:
  0a 00 0c 00 0a 00 0c 00 0a 00 0c 00 61 00 00 00
```

### Cell LIST_HEADER

pi=22 첫 번째 셀 LIST_HEADER는 Stage 9 보강 비트가 정답지와 다르다.

```text
oracle:
  02 00 00 00 20 00 00 00 ...

generated:
  02 00 00 00 20 00 01 00 ...
```

즉 `list_header_width_ref bit 0`이 조직도 표에서는 개선 축이었지만, 이 인원현황 표에서는
과보강일 가능성이 생겼다.

### Cell PARA_HEADER

첫 번째 셀의 PARA_HEADER는 record 크기부터 다르다.

```text
oracle:
  size = 24
  head24 = 0e 00 00 00 00 00 00 80 15 00 00 00 01 00 00 00 01 00 00 00 00 80 00 00

generated:
  size = 22
  head22 = 0e 00 00 00 00 00 00 00 14 00 00 00 01 00 00 00 01 00 00 00 00 00
```

### Cell PARA_TEXT

첫 번째 셀의 PARA_TEXT도 정답지와 다르다.

```text
oracle:
  20 00 1f 00 20 00 20 00 20 00 20 00 20 00 20 00 20 00 20 00 c1 c9 09 ae c4 bc 0d 00

generated:
  20 00 07 20 20 00 20 00 20 00 20 00 20 00 20 00 20 00 20 00 c1 c9 09 ae c4 bc 0d 00
```

이는 공백/특수 공백의 HWP5 저장 contract 차이일 가능성이 있다.

## 5. 생성한 probe

생성 명령:

```text
cargo run --quiet --bin rhwp -- hwp5-mel-personnel-probe samples/hwpx/hancom-hwp/mel-001.hwp output/poc/hwpx2hwp/task993/stage9_cell_list_header_materialization/mel-001.hwp --out-dir output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract --section 0
```

생성 파일:

| file | 목적 |
|---|---|
| `01_stage9_current.hwp` | Stage 9 기준 파일 |
| `02_pi22_without_list_extra.hwp` | pi=22 표에서만 Stage 9 LIST_HEADER extra 제거 |
| `03_pi22_oracle_list_extra.hwp` | pi=22 표에서만 정답지 LIST_HEADER extra 적용 |
| `04_pi22_table_tail_only.hwp` | pi=22 TABLE tail만 정답지로 투영 |
| `05_pi22_para_header_tail.hwp` | pi=22 셀 내부 PARA_HEADER tail만 정답지로 투영 |
| `06_pi22_table_tail_plus_oracle_list.hwp` | TABLE tail + LIST_HEADER 정답지 투영 |
| `07_pi22_table_list_para.hwp` | TABLE tail + LIST_HEADER + PARA_HEADER tail 정답지 투영 |
| `08_pi22_para_text_only.hwp` | 차이가 확인된 PARA_TEXT 1개만 정답지로 투영 |
| `09_pi22_text_table_list_para.hwp` | PARA_TEXT + TABLE tail + LIST_HEADER + PARA_HEADER tail 투영 |

생성 로그:

```text
output/poc/hwpx2hwp/task993/stage10_mel_personnel_table_contract/stage10_generation.md
```

모든 probe는 rhwp 기준으로 재로드에 성공했고 `pages=20`으로 확인되었다.

## 6. 작업지시자 판정표

| variant | 한컴 판정 유형 | 2페이지 인원현황 표 높이 | 다음 표 페이지 분리 | 1x1 표 배경 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `01_stage9_current.hwp` |  |  |  |  |  | 기준 |
| `02_pi22_without_list_extra.hwp` |  |  |  |  |  | LIST_HEADER 과보강 여부 |
| `03_pi22_oracle_list_extra.hwp` |  |  |  |  |  | LIST_HEADER 정답지 투영 |
| `04_pi22_table_tail_only.hwp` |  |  |  |  |  | TABLE tail 단독 |
| `05_pi22_para_header_tail.hwp` |  |  |  |  |  | PARA_HEADER tail 단독 |
| `06_pi22_table_tail_plus_oracle_list.hwp` |  |  |  |  |  | TABLE tail + LIST_HEADER |
| `07_pi22_table_list_para.hwp` |  |  |  |  |  | TABLE tail + LIST_HEADER + PARA_HEADER |
| `08_pi22_para_text_only.hwp` |  |  |  |  |  | PARA_TEXT 단독 |
| `09_pi22_text_table_list_para.hwp` |  |  |  |  |  | 전체 positive 후보 |

## 7. 우선 판정 순서

한컴 판정은 다음 순서로 진행하는 것이 좋다.

```text
1. 04_pi22_table_tail_only.hwp
   - TABLE tail 단독으로 과도한 셀 높이가 해결되는지 확인

2. 03_pi22_oracle_list_extra.hwp
   - Stage 9 LIST_HEADER 보강이 pi=22에서는 과보강인지 확인

3. 05_pi22_para_header_tail.hwp
   - 셀 내부 PARA_HEADER tail이 한컴 높이 산정에 관여하는지 확인

4. 08_pi22_para_text_only.hwp
   - 공백/특수 공백 payload 차이가 한컴 줄나눔/높이에 영향을 주는지 확인

5. 09_pi22_text_table_list_para.hwp
   - 관찰 축 전체를 투영했을 때 positive가 되는지 확인
```

## 8. 구현 변경

Stage 10에서는 판정용 diagnostic command를 추가했다.

```text
src/diagnostics/hwp5_mel_personnel_probe.rs
src/diagnostics/mod.rs
src/main.rs
```

추가 명령:

```text
rhwp hwp5-mel-personnel-probe <oracle.hwp> <generated.hwp> --out-dir <dir> --section 0
```

이 명령은 `mel-001`의 8x12 인원현황 표를 대상으로 TABLE tail, cell LIST_HEADER, cell
PARA_HEADER tail, PARA_TEXT 차이를 제한적으로 graft한 HWP probe를 생성한다.

## 9. 검증

실행:

```text
cargo check
```

결과:

```text
success
```

## 10. 해석

Stage 10으로 확인한 사실은 다음과 같다.

```text
1. rhwp IR 레벨에서는 pi=22 표 배치가 정답지와 거의 같은 형태다.
2. 한컴 에디터에서만 병합 셀 높이가 과도하게 커진다.
3. 따라서 원인은 표 높이 계산 자체가 아니라 한컴이 민감하게 해석하는 HWP5 raw record contract에 있다.
4. 현재 가장 의심되는 축은 TABLE tail, cell LIST_HEADER width_ref/extra, cell PARA_HEADER tail,
   그리고 공백/특수 공백 PARA_TEXT payload다.
```

다음 단계는 작업지시자의 한컴 판정 결과를 받아 positive 축을 확정하고, 해당 축만 adapter 구현에
핀셋 반영하는 것이다.

## 11. 작업지시자 판정 결과

작업지시자 판정:

```text
03, 06, 07, 09가 개선된 케이스다.
4개 케이스 모두 셀 대각선 처리가 누락되어 있다.
이 누락이 첫 페이지의 정체 모를 선으로 표시되는 것이 아닌지 의심된다.
```

판정 해석:

```text
개선 케이스:
  03_pi22_oracle_list_extra
  06_pi22_table_tail_plus_oracle_list
  07_pi22_table_list_para
  09_pi22_text_table_list_para

공통 축:
  OracleListExtra
```

따라서 `pi=22` 인원현황 표 높이 개선의 최소 positive 축은 다음으로 판단한다.

```text
pi=22 셀 LIST_HEADER의 정답지 width_ref/raw_list_extra contract
```

반대로 다음 축은 단독 해결축으로 보기 어렵다.

```text
TABLE tail 단독
PARA_HEADER tail 단독
PARA_TEXT 단독
```

`06`, `07`, `09`가 함께 개선된 것은 이들이 모두 `OracleListExtra`를 포함하기 때문이다.
그러므로 추가 축이 반드시 필요하다고 해석하면 안 된다.

## 12. 대각선 누락 분리

대각선 누락은 `pi=22` 높이 문제와 별도 축으로 분리한다.

현재 코드 구조상 셀 대각선은 셀 record 자체가 아니라 셀이 참조하는 DocInfo `BORDER_FILL`의
다음 두 정보가 함께 맞아야 표현된다.

```text
1. BorderFill.attr의 slash/backSlash 비트
2. DiagonalLine payload(type, width, color)
```

따라서 다음 단계에서는 `BORDER_FILL` contract를 별도 stage로 분리해 확인한다.

검증할 가설:

```text
HWPX의 hh:slash / hh:backSlash가 HWP5 BORDER_FILL.attr 비트와 diagonal payload로
정확히 materialize되지 않아 셀 대각선이 누락된다.
```

첫 페이지의 정체 모를 선과의 관계는 아직 확정하지 않는다. 다음 stage에서 대각선 contract를
정답지와 맞춘 probe를 생성한 뒤, 그 선이 사라지는지 함께 판정한다.
