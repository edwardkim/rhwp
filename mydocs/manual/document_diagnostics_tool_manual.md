---
kind: guide
status: active
canonical: mydocs/manual/document_diagnostics_tool_manual.md
last_verified: 2026-07-17
---

# 문서 진단 도구 매뉴얼

## 1. 목적

이 문서는 rhwp에서 문서 호환성 문제를 조사할 때 사용하는 진단 도구의 사용법을 정리한다.

대상은 다음과 같은 상황이다.

```text
HWP 또는 HWPX가 rhwp-studio에서는 열리지만 한컴 에디터에서는 다르게 보인다.
HWPX -> IR -> HWP 저장 결과가 한컴 에디터에서 파일손상 또는 파일 읽기 오류가 된다.
한컴에서 변환한 정답 HWP와 rhwp가 생성한 HWP 사이의 record contract 차이를 찾아야 한다.
```

진단 도구의 목적은 추측을 줄이고, 한컴 oracle에서 관찰되는 HWP5 record/control contract를
재현 가능한 근거로 남기는 것이다.

이 도구들은 구현 자체가 아니라 구현 결정을 내리기 위한 계측 도구다. 한컴 에디터 시각 판정,
정답 HWP, 생성 HWP, HWPX 원본을 함께 놓고 해석해야 한다.

## 2. 기본 용어

| 용어 | 의미 |
|---|---|
| source HWPX | 원본 HWPX 파일 |
| oracle HWP | 한컴 에디터가 source HWPX를 HWP로 저장한 정답 파일 |
| generated HWP | rhwp가 source HWPX를 IR로 읽은 뒤 HWP로 저장한 파일 |
| inventory | HWP5 DocInfo/BodyText stream을 record 단위로 펼친 목록 |
| contract | 한컴이 특정 HWPX construct를 HWP5 record tuple로 낮출 때 요구하는 암묵 규칙 |
| probe | oracle record 일부를 generated HWP에 graft해서 한컴 판정 경계를 확인하는 실험 파일 |
| sentinel | 이미 성공 조건을 알고 있는 대표 샘플. 확장 검증 전에 반드시 먼저 통과해야 한다. |

## 3. 준비물

기본 입력은 세 가지다.

```text
samples/hwpx/<sample>.hwpx
samples/hwpx/hancom-hwp/<sample>.hwp
output/poc/hwpx2hwp/task<issue>/<stage>/<generated>.hwp
```

정답 HWP가 없으면 먼저 한컴 에디터에서 HWPX를 열고 HWP로 저장해 `samples/hwpx/hancom-hwp/`에
준비한다.

진단 산출물은 `output/poc/` 아래에 둔다.

```text
output/poc/hwpx2hwp/task<issue>/<stage>/
```

## 4. 전체 흐름

권장 흐름은 다음과 같다.

```text
1. source HWPX를 현재 rhwp로 HWP 저장한다.
2. oracle HWP와 generated HWP의 inventory를 만든다.
3. record diff, hints, bundles, table-fields를 만든다.
4. source HWPX / oracle HWP / generated HWP의 control graph를 비교한다.
5. 필요한 경우 probe 파일을 생성한다.
6. 작업지시자가 한컴 에디터와 rhwp-studio에서 시각 판정한다.
7. 성공 후보가 나오면 sentinel부터 다시 통과시킨 뒤 샘플을 확장한다.
```

중요한 원칙:

```text
이미 sentinel에서 실패한 후보를 다른 샘플에 확장하지 않는다.
probe 성공은 구현 성공이 아니다. 어떤 contract가 필요한지 알려주는 판정 자료다.
```

## 5. 빌드

진단 명령은 `rhwp` 바이너리로 실행한다.

```bash
cargo build --quiet --bin rhwp
```

이후 예시는 `target/debug/rhwp` 기준으로 작성한다.

## 6. HWP5 inventory

### 6.1 목적

`hwp5-inventory`는 HWP 파일의 DocInfo와 BodyText record를 안정적인 행 단위 목록으로 만든다.
record index, tag, level, size, owner, parent, tuple role, payload hash를 확인할 수 있다.

### 6.2 사용법

```bash
target/debug/rhwp hwp5-inventory <파일.hwp> \
  --format md \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/oracle.inventory.md
```

JSONL이 필요할 때:

```bash
target/debug/rhwp hwp5-inventory <파일.hwp> \
  --format jsonl \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage1/hwpx-h-01/oracle.inventory.jsonl
```

### 6.3 주요 옵션

| 옵션 | 설명 |
|---|---|
| `--format md` | 사람이 읽는 Markdown inventory |
| `--format jsonl` | 후속 분석에 쓰기 좋은 행 단위 JSON |
| `--section N` | BodyText Section N만 분석 |
| `--out <path>` | 파일로 저장 |

## 7. HWP5 inventory diff

### 7.1 목적

`hwp5-inventory-diff`는 oracle HWP와 generated HWP의 inventory를 비교한다.

단순 차이뿐 아니라, 특정 contract 후보를 보기 위한 보고서를 만들 수 있다.

### 7.2 기본 diff

```bash
target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-01.hwp \
  --align lcs \
  --report diff \
  --format md \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage18_table_axis_regression/inventory.diff.md
```

### 7.3 contract hint

```bash
target/debug/rhwp hwp5-inventory-diff \
  <oracle.hwp> <generated.hwp> \
  --align lcs \
  --report hints \
  --focus all \
  --window 4 \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage5/hwpx-h-01/contract_violation_hints.md
```

`hints`는 확정 규칙이 아니다. 다음 probe 또는 source trace로 검증할 후보 목록이다.

### 7.4 주변 bundle

```bash
target/debug/rhwp hwp5-inventory-diff \
  <oracle.hwp> <generated.hwp> \
  --align lcs \
  --report bundles \
  --focus table \
  --window 6 \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage6/hwpx-h-01/table_bundles.md
```

`bundles`는 변경 record 주변의 control tuple을 함께 보기 위한 보고서다.

### 7.5 table fields

```bash
target/debug/rhwp hwp5-inventory-diff \
  <oracle.hwp> <generated.hwp> \
  --align lcs \
  --report table-fields \
  --focus table \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage18_table_axis_regression/oracle_vs_hwpx-h-01_table_fields.md
```

TABLE/CTRL_HEADER(Table) 축의 필드 차이를 좁힐 때 사용한다.

### 7.6 table probe plan

```bash
target/debug/rhwp hwp5-inventory-diff \
  <oracle.hwp> <generated.hwp> \
  --align lcs \
  --report table-probe-plan \
  --focus table \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage8/hwpx-h-01/table_probe_plan.md
```

`table-probe-plan`은 어느 TABLE 축을 probe로 나눌지 제안한다.

### 7.7 report / focus 옵션

| 옵션 | 값 | 설명 |
|---|---|---|
| `--align` | `index`, `lcs` | record 정렬 방식. 중간 삽입/누락이 있으면 `lcs` 우선 |
| `--report` | `diff` | 일반 diff |
| `--report` | `hints` | contract 후보 힌트 |
| `--report` | `bundles` | 주변 record 묶음 |
| `--report` | `table-fields` | TABLE 관련 필드 차이 |
| `--report` | `table-probe-plan` | TABLE probe 축 제안 |
| `--focus` | `all` | 전체 |
| `--focus` | `table` | 표 관련 record |
| `--focus` | `shape` | 그림/도형 관련 record |
| `--focus` | `ctrl` | CTRL_HEADER 중심 |
| `--focus` | `missing` | 누락 record 중심 |
| `--focus` | `docinfo` | DocInfo 중심 |

## 8. HWPX/HWP contract graph 분석

### 8.1 목적

`hwp5-contract-analyze`는 source HWPX, oracle HWP, generated HWP를 함께 비교한다.

생성되는 보고서:

```text
docinfo_contract.md
bodytext_control_graph.md
hwpx_ir_serializer_trace.md
stage12_index.md
```

### 8.2 사용법

```bash
target/debug/rhwp hwp5-contract-analyze \
  samples/hwpx/hwpx-h-03.hwpx \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-03.hwp \
  --section 0 \
  --out-dir output/poc/hwpx2hwp/task949/stage19_h03_contract_trace
```

### 8.3 해석 기준

이 명령은 HWP probe를 만들지 않는다. 다음 질문에 답하기 위한 정적 분석이다.

```text
source HWPX에는 어떤 construct가 있는가?
oracle HWP에는 어떤 record/control tuple로 내려갔는가?
generated HWP에는 어떤 tuple이 빠지거나 달라졌는가?
```

## 9. CTRL_DATA trace

### 9.1 목적

`hwp5-ctrl-data-trace`는 oracle/generated의 CTRL_DATA record를 비교한다.
그림, 도형, 특수 control의 ParameterSet payload가 oracle과 같은 위치/레벨/크기로 들어갔는지
확인할 때 사용한다.

### 9.2 사용법

```bash
target/debug/rhwp hwp5-ctrl-data-trace \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task949/stage18_table_axis_regression/hwpx-h-03.hwp \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage16_adapter_regression/ctrl_data_trace_hwpx-h-03.md
```

특정 record만 볼 때:

```bash
target/debug/rhwp hwp5-ctrl-data-trace \
  <oracle.hwp> <generated.hwp> \
  --section 0 \
  --record-index 833 \
  --out output/poc/hwpx2hwp/task949/stage15/hwpx-h-03/ctrl_data_trace.md
```

## 10. Probe 생성 도구

Probe는 oracle record 일부를 generated HWP에 이식해 한컴 판정 경계가 어떻게 움직이는지 보는
실험 파일이다.

Probe 결과가 성공해도 그대로 구현하면 안 된다. 성공한 probe가 어떤 record contract를 암시하는지
source HWPX와 oracle HWP를 다시 비교한 뒤 구현한다.

### 10.1 hwp5-table-probe

표 관련 축을 나누어 HWP probe를 만든다.

```bash
target/debug/rhwp hwp5-table-probe \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-01.hwp \
  --section 0 \
  --out-dir output/poc/hwpx2hwp/task949/stage9/hwpx-h-01
```

생성 variant:

| 파일 | 의미 |
|---|---|
| `01_ctrl_outer_margin_only.hwp` | TABLE CTRL_HEADER outer margin |
| `02_table_attr_only.hwp` | TABLE attr |
| `03_table_tail_only.hwp` | TABLE tail |
| `04_ctrl_common_attr_only.hwp` | TABLE CTRL_HEADER common attr |
| `05_outer_margin_table_attr.hwp` | outer margin + table attr |
| `06_outer_margin_table_tail.hwp` | outer margin + table tail |
| `07_table_attr_tail.hwp` | table attr + table tail |
| `08_all_table_axes.hwp` | 모든 TABLE 축 |

### 10.2 hwp5-contract-probe

DocInfo ID mapping, MEMO_SHAPE, shape CTRL_DATA 축을 나누어 HWP probe를 만든다.

```bash
target/debug/rhwp hwp5-contract-probe \
  samples/hwpx/hancom-hwp/hwpx-h-03.hwp \
  output/poc/hwpx2hwp/task949/stage10/hwpx-h-03/08_all_table_axes.hwp \
  --section 0 \
  --out-dir output/poc/hwpx2hwp/task949/stage11/hwpx-h-03_contract_probe
```

이 도구는 범용 수정기가 아니라 정해진 축의 probe 생성기다.

## 11. 보조 덤프 명령

### 11.1 dump

IR/control 구조를 사람이 읽는 형태로 본다.

```bash
target/debug/rhwp dump <파일.hwp|파일.hwpx> --section 0 --para 29
```

표/그림/글상자 control의 모델 속성을 확인할 때 유용하다.

### 11.2 dump-pages

페이지네이션 결과를 문단/표 배치 목록으로 본다.

```bash
target/debug/rhwp dump-pages <파일.hwp> -p 7
```

한컴에서는 정상인데 rhwp-studio 페이지가 다르게 나뉘는 경우에 사용한다.

### 11.3 dump-records

HWP BodyText record stream을 raw record 순서로 본다.

```bash
target/debug/rhwp dump-records <파일.hwp> > output/poc/hwpx2hwp/task949/stage20/oracle_dump_records.txt
```

record index, tag, level, size를 빠르게 확인할 때 쓴다.

### 11.4 ir-diff

HWP/HWPX가 생성하는 IR 차이를 본다.

```bash
target/debug/rhwp ir-diff \
  samples/hwpx/hwpx-h-01.hwpx \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --summary
```

`ir-diff`는 parser/IR 차이 확인용이다. HWP5 binary contract 판정은 `hwp5-*` 진단으로 이어서
확인한다.

## 12. 판정 기록 양식

작업지시자에게 판정을 요청할 때는 표를 준비한다.

```markdown
| file | 한컴 판정 유형 | 이미지 출력 | 표/셀 배치 | 셀 텍스트 클리핑 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|---|
| `output/poc/.../sample.hwp` |  |  |  |  |  |  |  |
```

`파일손상`과 `파일 읽기 오류`는 구분한다.

```text
파일 읽기 오류:
  한컴이 record stream을 초기 단계에서 못 읽는 경우가 많다.

파일손상:
  일부 문단/표/그림을 출력한 뒤 다음 record 진입 시점에 실패하는 경우가 많다.
```

출력 위치는 반드시 남긴다.

```text
예: 1페이지 첫 표까지 출력
예: 2페이지 이미지 개체 묶기 전에서 중단
예: 7페이지 2. 연도별 동향까지 출력
```

## 13. 성공 후보 승격 규칙

다음 조건을 만족할 때만 구현 후보로 승격한다.

```text
1. sentinel 샘플에서 한컴 에디터와 rhwp-studio가 모두 성공한다.
2. oracle/generated field diff가 의도한 축에서 닫힌다.
3. probe 성공 이유를 source HWPX construct와 HWP5 record tuple로 설명할 수 있다.
4. 이전에 성공한 샘플에서 회귀가 없어야 한다.
```

반대로 다음 경우는 구현 후보가 아니다.

```text
1. rhwp-studio만 성공하고 한컴 에디터가 실패한다.
2. probe 파일은 성공하지만 어떤 source construct를 어떻게 낮춰야 하는지 설명하지 못한다.
3. sentinel에서 이미 실패한 후보를 다른 샘플에 확장한다.
4. 여러 축을 한 번에 바꿔 성공했지만 축별 원인을 분리하지 못했다.
```

## 14. #949 기준 예시 흐름

`hwpx-h-01`의 table-axis 문제는 다음 흐름으로 정리되었다.

```bash
# 1. table probe plan 생성
target/debug/rhwp hwp5-inventory-diff \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-01.hwp \
  --align lcs \
  --report table-probe-plan \
  --focus table \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage8/hwpx-h-01/table_probe_plan.md

# 2. table probe 생성
target/debug/rhwp hwp5-table-probe \
  samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-01.hwp \
  --section 0 \
  --out-dir output/poc/hwpx2hwp/task949/stage9/hwpx-h-01

# 3. 성공 probe와 current adapter 차이 비교
target/debug/rhwp hwp5-inventory-diff \
  output/poc/hwpx2hwp/task949/stage9/hwpx-h-01/08_all_table_axes.hwp \
  output/poc/hwpx2hwp/task949/stage16_adapter_regression/hwpx-h-01.hwp \
  --align lcs \
  --report table-fields \
  --focus table \
  --section 0 \
  --out output/poc/hwpx2hwp/task949/stage17_h01_table_axis_gap/stage9_success_vs_current_table_fields.md
```

이후 `hwpx-h-01` sentinel을 먼저 성공시킨 뒤 `hwpx-h-02`, `hwpx-h-03`으로 확장했다.

## 15. 문서화 규칙

각 stage 문서는 다음 정보를 남긴다.

```text
목적
입력 파일
생성 파일
실행 명령
정적 diff 결과
작업지시자 시각 판정
해석
다음 단계
```

발견한 규칙은 stage 문서에만 두지 않는다. 장기 규칙은 다음 문서로 승격한다.

```text
mydocs/troubleshootings/hwpx2hwp-rule.md
```

명령 사용법 또는 반복 워크플로우는 이 매뉴얼에 반영한다.
