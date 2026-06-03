# Task m100 #946 Stage 1 계획서: HWPX to HWP Contract 추출 개발도구

## 1. 배경

#903의 후반 probe는 여러 한컴 통과 산출물을 만들었지만, HWPX construct가 어떤 HWP5
record tuple로 lowering되어야 하는지 정확한 contract를 확정하지 못했다.

작업지시자의 정정에 따라 문제를 다시 정의한다.

```text
HWPX는 rhwp-studio에서 정상 렌더링된다.
문제는 HWPX -> IR -> HWP 저장 시 한컴 에디터가 요구하는 HWP5 control/record contract를 만족하지 못하는 것이다.
```

이 문제의 배경에는 HWP와 HWPX의 포맷 거버넌스 차이가 있다.

```text
HWP:
  문서 의미 + 스타일 + 조판 결과 + HWP5 binary record contract를 함께 저장한다.

HWPX:
  문서 의미 + 스타일 + 객체 관계 + 일부 조판 힌트를 저장한다.
  HWP5 binary record contract는 한컴 엔진이 열기/렌더링/저장 시점에 채우는 값일 수 있다.
```

따라서 HWPX에 없는 값을 단순 파싱 누락으로 보지 않는다. 한컴 HWP oracle을 기준으로
HWPX construct가 어떤 HWP5 target contract로 lowering되는지 확인해야 한다.

작업 순서도 이 관점에 맞춘다. HWPX를 다시 HWPX로 저장하는 same-format round-trip이
제품 기능으로는 더 자연스러울 수 있지만, 본 타스크에서는 더 어려운 HWP target backend를 먼저 푼다.

```text
먼저:
  HWPX -> IR -> HWP
  HWPX construct -> HWP5 record/control lowering contract 확정

나중:
  HWPX -> IR -> HWPX
  확정된 의미론을 바탕으로 same-format 저장 경계 정리
```

이유는 명확하다. HWPX -> HWP를 먼저 풀어야 한컴이 HWPX에 명시하지 않은 조판 결과,
기본값, binary record tuple을 어떻게 보완하는지 알 수 있다. 이 지식이 있어야 HWPX 저장에서도
무엇을 보존하고 무엇을 canonicalize해야 하는지 정확히 결정할 수 있다.

이 작업은 단기 버그픽스가 아니라 장기 contract map 구축의 시작이다.

```text
HWPX -> IR -> HWP 성공 경로를 많은 contract unit으로 나눈다.
각 unit은 HWPX construct 하나와 한컴 HWP oracle의 HWP5 record/control tuple 하나를 연결한다.
빠르게 통과시키는 것보다, 다시 틀리지 않게 지식을 남기는 것이 우선이다.
```

따라서 Stage 1 개발도구는 실험을 많이 만들기 위한 도구가 아니라, 각 실험이 반드시
`construct -> oracle tuple -> generated tuple -> contract -> violation -> rule` 형태로
축적되게 만드는 도구다.

따라서 이번 시도에서는 저장기 구현을 바로 시작하지 않는다.
먼저 다음을 정확하게 수행할 개발도구를 만든다.

```text
1. 한컴 변환 HWP oracle의 HWP5 record/control inventory 추출
2. rhwp 생성 HWP의 HWP5 record/control inventory 추출
3. oracle/generated HWP5 inventory diff 생성
4. HWPX control inventory를 3-way 대조의 세 번째 축으로 추가
5. HWPX construct -> HWP5 record tuple lowering contract 확정
6. rhwp generated HWP의 contract 만족/위반 여부 판정
7. output/poc 아래 작업지시자 판정용 파일과 report를 표준 생성
```

## 2. 목표

Stage 1의 목표는 저장기 로직을 고치는 것이 아니라, 구현 전에 반드시 거칠
HWP5 record/control contract 추출 CLI를 추가하는 것이다.

피드백에 따라 우선순위를 바로잡는다. 첫 도구는 HWPX inventory가 아니라
한컴 HWP oracle과 rhwp generated HWP를 같은 기준으로 읽는 HWP5 inventory여야 한다.

Stage 1의 P0 산출물:

```text
rhwp hwp5-inventory <file.hwp> [--format csv|jsonl|md] [--section N]
rhwp hwp5-inventory-diff \
  --oracle <hancom_converted.hwp> \
  --generated <rhwp_generated.hwp> \
  [--hwpx-source <source.hwpx>] \
  --out <output-dir>
```

P1 이후 산출물:

```text
rhwp hwpx-control-inventory <file.hwpx> [--format jsonl|md]
rhwp hwp5-probe-gen --template <base.hwp> --mutate <contract_unit.toml>
```

P2 장기 산출물:

```text
contract corpus registry
contract_unit schema
rhwp hwp5-contract-check
dashboard
```

명령 이름은 구현 중 조정 가능하지만, 기능 경계와 우선순위는 유지한다.

## 3. 비목표

Stage 1에서는 다음을 하지 않는다.

```text
HWPX -> HWP 저장기 본 로직 수정
contract가 확정되지 않은 한컴 통과 산출물을 production 코드로 승격
시각 판정 없이 성공 판정
rhwp-studio reload 성공을 한컴 호환 성공으로 간주
```

## 4. 기존 도구와의 관계

### ir-diff

`ir-diff`는 HWPX/HWP를 파싱한 뒤 IR 의미 차이를 찾는 도구다.

이번 도구는 `ir-diff`를 대체하지 않는다. `ir-diff`가 보는 것은 의미 IR이고,
이번 도구가 보는 것은 저장 직전/직후의 HWP5 record contract다.

```text
ir-diff:
  HWPX와 HWP가 같은 의미 IR을 만드는가?

hwp5-inventory-diff:
  한컴 HWP oracle과 rhwp generated HWP의 HWP5 record/control tuple이 어떻게 다른가?
  차이가 HWPX construct lowering contract 위반인지 판단할 근거가 무엇인가?
```

따라서 `ir-diff` 결과가 같아도 한컴 HWP 로딩 성공을 보장하지 않는다.
`ir-diff`는 frontend/IR 검증 도구이고, `hwp5-inventory-diff`는 target ABI 검증 도구다.

### HWPX round-trip 예제

`examples/hwpx_roundtrip.rs`와 유사한 자기 round-trip 도구는 HWPX same-format 저장의
자체 보존성 확인에는 유용하다. 그러나 한컴 HWP oracle 검증이 아니므로 HWPX -> HWP
lowering contract의 충분 조건으로 사용하지 않는다.

### dump / dump-records

`dump`와 `dump-records`는 사람이 읽는 단품 진단 도구다.
Stage 1 도구는 이를 자동화하여 issue/stage 단위 report와 machine-readable JSON을 생성한다.

## 5. 기능 설계

### 5.1 P0: hwp5-inventory

하나의 HWP5 파일을 읽고 HWP5 record/control inventory를 출력한다.

사용 예:

```bash
cargo run --bin rhwp -- hwp5-inventory samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --format jsonl \
  --out output/poc/hwpx2hwp/task946/stage1/hwpx-h-01/oracle.inventory.jsonl
```

지원 옵션:

```text
--format csv|jsonl|md
--section N
--out <path>
```

출력 컬럼:

```text
sample
source_path
section
record_index
level
tag
size
owner
parent_scope
key_payload
payload_sha256
raw_offset_or_order_key
note
```

핵심 추출 항목:

```text
CFB stream 목록과 BodyText/DocInfo stream 존재 여부
section index
record index
level
tag id / tag name
payload size
owner paragraph/control/table/cell 추정
parent_scope
CTRL_HEADER -> 다음 concrete control 연결
LIST_HEADER -> child paragraph 범위
TABLE -> row/cell subtree 범위
SHAPE_COMPONENT / SHAPE_PICTURE subtree 범위
key_payload(attr/count/id/tail hex)
payload sha256
```

이 도구는 HWPX를 읽지 않는다. Stage 1의 첫 목표는 한컴 oracle HWP와 rhwp generated HWP를
같은 HWP5 record/control 언어로 설명하는 것이다.

### 5.2 P0: hwp5-inventory-diff

한컴 HWP oracle과 rhwp generated HWP의 HWP5 inventory를 비교한다.

사용 예:

```bash
cargo run --bin rhwp -- hwp5-inventory-diff \
  --oracle samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --generated output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --hwpx-source samples/hwpx/hwpx-h-01.hwpx \
  --out output/poc/hwpx2hwp/task946/stage1/hwpx-h-01
```

생성 산출물:

```text
<out>/
  oracle.inventory.jsonl
  oracle.inventory.md
  generated.inventory.jsonl
  generated.inventory.md
  inventory_diff.jsonl
  inventory_diff.csv
  inventory_diff.md
  contract_violation_hints.md
```

비교 관점:

```text
record 누락/추가
record 순서 차이
level / parent scope 차이
tag별 핵심 payload 차이
count / size / reference 차이
DocInfo BinData record와 CFB BinData stream 차이
마지막 정상 출력 위치 직후의 첫 mismatch 후보
failure_class A-F 자동 분류 힌트
```

주의:

```text
hwp5-inventory-diff는 contract 확정 도구가 아니라 contract 후보를 좁히는 도구다.
최종 rule 승격은 한컴 oracle record tuple과 작업지시자 판정이 함께 맞을 때만 한다.
```

### 5.3 P1: hwpx-control-inventory

HWPX construct inventory를 생성한다. 이 도구는 P0 HWP5 inventory가 먼저 안정화된 뒤
3-way 대조의 세 번째 축으로 추가한다.

사용 예:

```bash
cargo run --bin rhwp -- hwpx-control-inventory samples/hwpx/hwpx-h-01.hwpx \
  --format jsonl \
  --out output/poc/hwpx2hwp/task946/stage2/hwpx-h-01/hwpx.inventory.jsonl
```

HWPX 입력에서 추출할 항목:

```text
section index
paragraph index
HWPX XML path
construct id
control kind: table / picture / shape / group / section / field / unknown
text summary
table row/column/cell/span summary
object id / instance id / binData reference
positioning attributes
style references
lineSegArray presence
expected HWP5 tag 후보
mapping status: known / guessed / unknown
```

### 5.4 P1: hwp5-probe-gen

단일 contract unit을 격리 검증하기 위한 synthetic/probe HWP 생성기다.

사용 예:

```bash
cargo run --bin rhwp -- hwp5-probe-gen \
  --template samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --mutate mydocs/tech/hwpx2hwp_contract_corpus/unit-0001/contract_unit.toml \
  --out output/poc/hwpx2hwp/task946/probes/unit-0001.hwp
```

원칙:

```text
한 번에 하나의 record/control tuple만 변형한다.
probe는 contract 확인용이지 production 구현 근거가 아니다.
한컴 통과 여부와 inventory diff를 함께 기록한다.
```

### 5.5 P2: contract corpus / runner / dashboard

장기 운영 단계에서는 contract unit을 축적하고 회귀를 자동 검증한다.

후보 구조:

```text
mydocs/tech/hwpx2hwp_contract_corpus/
  unit-0001/
    construct.md
    oracle_tuple.hex
    generated_tuple.hex
    lowering_contract.md
    contract_status.toml
```

장기 명령:

```text
rhwp hwp5-contract-check
```

검증 항목:

```text
unit별 satisfied / violated / unknown 상태
failure_class A-F 분포
샘플별 coverage
회귀 여부
대시보드 HTML/Markdown 생성
```

## 6. 데이터 모델 초안

도구 내부 구조체는 JSON 출력 가능해야 한다.

```text
Hwp5InventoryItem
  sample
  source_path
  section
  record_index
  level
  tag
  size
  owner
  parent_scope
  key_payload
  refs
  payload_sha256
  note

Hwp5InventoryDiffItem
  sample
  section
  oracle_record_index
  generated_record_index
  mismatch: missing / extra / value_diff / order_diff / level_diff / size_diff / hash_diff
  failure_class: A / B / C / D / E / F / unknown
  oracle_item
  generated_item
  hint

HwpxControlInventoryItem
  sample
  section
  paragraph
  xml_path
  construct_id
  kind
  text_summary
  key_values
  refs
  expected_hwp5_tags
  mapping_status

ContractUnit
  unit_id
  hwpx_construct
  oracle_tuple
  generated_tuple
  lowering_contract
  contract_violation
  hancom_judgement
  rule
  regression_sample
```

## 7. 구현 위치

`src/main.rs`가 이미 크므로 명령 본체는 새 모듈로 분리한다.

구현 위치:

```text
src/diagnostics/mod.rs
src/diagnostics/hwp5_inventory.rs
src/diagnostics/hwp5_inventory_diff.rs
src/diagnostics/hwpx_control_inventory.rs
src/diagnostics/hwp5_probe_gen.rs
```

CLI 진입점은 `src/main.rs`에 얇게 추가한다.

```text
Some("hwp5-inventory") => diagnostics::hwp5_inventory::run(&args[2..])
Some("hwp5-inventory-diff") => diagnostics::hwp5_inventory_diff::run(&args[2..])
Some("hwpx-control-inventory") => diagnostics::hwpx_control_inventory::run(&args[2..])
Some("hwp5-probe-gen") => diagnostics::hwp5_probe_gen::run(&args[2..])
```

## 8. Stage 1 작업 순서

1. 기존 HWP5 parser record API와 `dump-records` 구현을 확인한다.
2. `src/diagnostics` 모듈을 만들고 `hwp5-inventory`를 먼저 구현한다.
3. 한컴 oracle HWP와 rhwp generated HWP 양쪽에서 동일 inventory 포맷을 생성한다.
4. `hwp5-inventory-diff`를 구현하여 누락/추가/값 다름/순서 다름을 report로 만든다.
5. `failure_class` A-F 자동 분류 힌트와 "첫 mismatch 후보"를 출력한다.
6. `samples/hwpx/hancom-hwp/hwpx-h-01.hwp`와 기존 generated HWP로 smoke test한다.
7. HWPX 3-way 대조는 P0이 안정화된 뒤 `hwpx-control-inventory`로 이어간다.
8. 결과를 `mydocs/working/task_m100_946_stage1.md`에 기록한다.

## 9. 검증 기준

Stage 1 완료 기준:

```text
cargo run --bin rhwp -- hwp5-inventory samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --format jsonl \
  --out output/poc/hwpx2hwp/task946/stage1/hwpx-h-01/oracle.inventory.jsonl

cargo run --bin rhwp -- hwp5-inventory-diff \
  --oracle samples/hwpx/hancom-hwp/hwpx-h-01.hwp \
  --generated output/poc/hwpx2hwp/task903/web_save_repro/hwpx-h-01-web-save.hwp \
  --hwpx-source samples/hwpx/hwpx-h-01.hwpx \
  --out output/poc/hwpx2hwp/task946/stage1/hwpx-h-01
```

위 명령이 다음을 생성해야 한다.

```text
oracle.inventory.jsonl
oracle.inventory.md
generated.inventory.jsonl
generated.inventory.md
inventory_diff.jsonl
inventory_diff.csv
inventory_diff.md
contract_violation_hints.md
```

그리고 report에는 최소한 다음 항목이 보여야 한다.

```text
CFB stream 요약
DocInfo / BodyText section record 수
CTRL_HEADER / LIST_HEADER / TABLE / SHAPE / PIC record tuple 후보
oracle 대비 generated의 누락/추가/값 다름/순서 다름
failure_class A-F 힌트
첫 mismatch 후보
```

## 10. 작업지시자 판정 요청 시점

Stage 1에서는 한컴 시각 판정용 새 HWP를 만들지 않는다.
기존 oracle/generated HWP의 inventory와 diff 산출물이 작업지시자가 보던 현상을 설명하는지 확인한다.

작업지시자에게 요청할 항목:

```text
output/poc/hwpx2hwp/task946/stage1/hwpx-h-01/inventory_diff.md
output/poc/hwpx2hwp/task946/stage1/hwpx-h-01/contract_violation_hints.md
```

판정 항목:

```text
한컴에서 관찰한 실패 위치와 첫 mismatch 후보가 연결되는지
failure_class A-F 힌트가 실제 현상과 맞는지
oracle/generated record tuple 차이가 다음 probe 범위를 정할 만큼 구체적인지
```

이 판정이 맞으면 다음 Stage에서 `hwpx-control-inventory`와 3-way 대조로 확장한다.

## 11. 승인 요청

이 계획은 저장기 구현 전 개발도구를 만드는 Stage 1 계획이다.

승인 후에는 구현을 시작한다.
